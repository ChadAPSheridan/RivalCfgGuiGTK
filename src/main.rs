use std::env;
use std::collections::HashMap;
use std::sync::{Mutex, LazyLock};
use std::sync::Arc;
use std::time::SystemTime;

// settings includes
use serde::{Deserialize, Serialize};
use serde_json;
use dirs;
use std::fs;

// Global cache for PNG conversions
static PNG_CACHE: LazyLock<Mutex<HashMap<String, (String, SystemTime)>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

// Track last known battery state to avoid unnecessary updates
static LAST_BATTERY_STATE: LazyLock<Mutex<Option<(u8, bool)>>> = LazyLock::new(|| Mutex::new(None));

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
struct Settings {
    sensitivity: Option<String>,
    polling_rate: Option<String>,
    sleep_timer: Option<String>,
    dim_timer: Option<String>,
    // reserved for future settings like icon colour
    colour_switch: Option<bool>,
}

fn settings_file_path() -> Option<PathBuf> {
    // Use XDG config directory if available, otherwise fallback to home/.config
    let base = dirs::config_dir()?;
    let dir = base.join("rivalcfg-tray");
    Some(dir.join("settings.json"))
}

// Abstraction for running external commands so we can mock in tests
#[derive(Debug, Clone)]
pub struct CommandOutput {
    pub stdout: String,
    pub stderr: String,
    pub success: bool,
    pub code: Option<i32>,
}

pub trait CommandRunner: Send + Sync {
    fn run(&self, program: &str, args: &[&str]) -> CommandOutput;
}

#[derive(Debug, Default)]
pub struct RealCommandRunner {}

impl CommandRunner for RealCommandRunner {
    fn run(&self, program: &str, args: &[&str]) -> CommandOutput {
        let output = std::process::Command::new(program).args(args).output();
        match output {
            Ok(o) => CommandOutput {
                stdout: String::from_utf8_lossy(&o.stdout).to_string(),
                stderr: String::from_utf8_lossy(&o.stderr).to_string(),
                success: o.status.success(),
                code: o.status.code(),
            },
            Err(e) => CommandOutput {
                stdout: String::new(),
                stderr: format!("Failed to spawn {}: {}", program, e),
                success: false,
                code: None,
            },
        }
    }
}

/// Build arguments for `rivalcfg` from Settings. Returns only the args (no program name).
fn build_rivalcfg_args(s: &Settings) -> Vec<String> {
    let mut args = Vec::new();
    if let Some(ref sens) = s.sensitivity {
        if !sens.is_empty() {
            args.push("--sensitivity".to_string());
            args.push(sens.clone());
        }
    }
    if let Some(ref rate) = s.polling_rate {
        if !rate.is_empty() {
            args.push("--polling-rate".to_string());
            args.push(rate.clone());
        }
    }
    if let Some(ref sleep) = s.sleep_timer {
        if !sleep.is_empty() {
            args.push("--sleep-timer".to_string());
            args.push(sleep.clone());
        }
    }
    if let Some(ref dim) = s.dim_timer {
        if !dim.is_empty() {
            args.push("--dim-timer".to_string());
            args.push(dim.clone());
        }
    }
    args
}

fn load_settings() -> Option<Settings> {
    let path = settings_file_path()?;
    if !path.exists() {
        return Some(Settings::default());
    }
    let data = fs::read_to_string(&path).ok()?;
    let s: Settings = serde_json::from_str(&data).ok()?;
    Some(s)
}

fn save_settings(s: &Settings) -> Result<(), anyhow::Error> {
    if let Some(path) = settings_file_path() {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let data = serde_json::to_string_pretty(s)?;
        fs::write(&path, data)?;
        eprintln!("[rivalcfg-tray] Saved settings to {}", path.display());
        return Ok(());
    }
    Err(anyhow::anyhow!("Could not determine settings file path"))
}

// Validation helpers
fn validate_sensitivity(s: &str) -> Result<(), String> {
    if s.trim().is_empty() {
        return Ok(());
    }
    match s.parse::<u32>() {
        Ok(v) if v >= 100 && v <= 16000 => Ok(()),
        _ => Err("Sensitivity must be a number between 100 and 16000".to_string()),
    }
}

fn validate_polling_rate(s: &str) -> Result<(), String> {
    if s.trim().is_empty() {
        return Ok(());
    }
    match s {
        "125" | "250" | "500" | "1000" => Ok(()),
        _ => Err("Polling rate must be one of: 125, 250, 500, 1000".to_string()),
    }
}

fn validate_timer(s: &str, name: &str) -> Result<(), String> {
    if s.trim().is_empty() {
        return Ok(());
    }
    match s.parse::<u32>() {
        Ok(_) => Ok(()),
        Err(_) => Err(format!("{} must be a whole number", name)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Mutex;

    #[derive(Debug, Default)]
    struct MockCommandRunner {
        responses: Mutex<HashMap<String, CommandOutput>>,
        calls: Mutex<Vec<(String, Vec<String>)>>,
    }

    impl MockCommandRunner {
        fn new() -> Self {
            Self {
                responses: Mutex::new(HashMap::new()),
                calls: Mutex::new(Vec::new()),
            }
        }

        fn set_response(&self, program: &str, args: &[&str], out: CommandOutput) {
            let key = format!("{}|{}", program, args.join("|"));
            self.responses.lock().unwrap().insert(key, out);
        }

        fn get_calls(&self) -> Vec<(String, Vec<String>)> {
            self.calls.lock().unwrap().clone()
        }
    }

    impl CommandRunner for MockCommandRunner {
        fn run(&self, program: &str, args: &[&str]) -> CommandOutput {
            let args_vec = args.iter().map(|s| s.to_string()).collect::<Vec<_>>();
            self.calls.lock().unwrap().push((program.to_string(), args_vec.clone()));
            let key = format!("{}|{}", program, args.join("|"));
            if let Some(out) = self.responses.lock().unwrap().get(&key) {
                return out.clone();
            }
            CommandOutput {
                stdout: String::new(),
                stderr: format!("No mock response for {} {:?}", program, args),
                success: false,
                code: None,
            }
        }
    }

    #[test]
    fn test_validate_sensitivity() {
        assert!(validate_sensitivity("").is_ok());
        assert!(validate_sensitivity("800").is_ok());
        assert!(validate_sensitivity("100").is_ok());
        assert!(validate_sensitivity("16000").is_ok());
        assert!(validate_sensitivity("99").is_err());
        assert!(validate_sensitivity("abc").is_err());
    }

    #[test]
    fn test_validate_polling_rate() {
        assert!(validate_polling_rate("").is_ok());
        assert!(validate_polling_rate("125").is_ok());
        assert!(validate_polling_rate("250").is_ok());
        assert!(validate_polling_rate("500").is_ok());
        assert!(validate_polling_rate("1000").is_ok());
        assert!(validate_polling_rate("42").is_err());
    }

    #[test]
    fn test_validate_timer() {
        assert!(validate_timer("", "Sleep Timer").is_ok());
        assert!(validate_timer("10", "Sleep Timer").is_ok());
        assert!(validate_timer("abc", "Dim Timer").is_err());
    }

    #[test]
    fn test_settings_roundtrip() {
        let s = Settings {
            sensitivity: Some("800".to_string()),
            polling_rate: Some("1000".to_string()),
            sleep_timer: Some("15".to_string()),
            dim_timer: Some("5".to_string()),
            colour_switch: Some(true),
        };
        let json = serde_json::to_string(&s).expect("serialize");
        let parsed: Settings = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed.sensitivity, s.sensitivity);
        assert_eq!(parsed.polling_rate, s.polling_rate);
        assert_eq!(parsed.sleep_timer, s.sleep_timer);
        assert_eq!(parsed.dim_timer, s.dim_timer);
        assert_eq!(parsed.colour_switch, s.colour_switch);
    }

    #[test]
    fn test_get_battery_level_with_mock_runner_charging() {
        let mock = MockCommandRunner::new();
        let stdout = "SteelSeries Rival Options:\nMouse battery: 75% Charging\n".to_string();
        mock.set_response(
            "rivalcfg",
            &["--battery-level"],
            CommandOutput {
                stdout: stdout.clone(),
                stderr: String::new(),
                success: true,
                code: Some(0),
            },
        );

        let res = get_battery_level_with_runner(&mock);
        assert!(res.is_some());
        let (percent, charging) = res.unwrap();
        assert_eq!(percent, 75);
        assert!(charging);
    }

    #[test]
    fn test_get_battery_level_with_mock_runner_discharging() {
        let mock = MockCommandRunner::new();
        let stdout = "Mouse battery: 12% Discharging\n".to_string();
        mock.set_response(
            "rivalcfg",
            &["--battery-level"],
            CommandOutput {
                stdout: stdout.clone(),
                stderr: String::new(),
                success: true,
                code: Some(0),
            },
        );
        let res = get_battery_level_with_runner(&mock);
        assert!(res.is_some());
        let (percent, charging) = res.unwrap();
        assert_eq!(percent, 12);
        assert!(!charging);
    }

    #[test]
    fn test_get_mouse_name_with_mock_runner() {
        let mock = MockCommandRunner::new();
        let stdout = "Some header\nMyMouse Options:\n more text\n".to_string();
        mock.set_response(
            "rivalcfg",
            &["--help"],
            CommandOutput {
                stdout: stdout.clone(),
                stderr: String::new(),
                success: true,
                code: Some(0),
            },
        );
        let res = get_mouse_name_with_runner(&mock);
        assert_eq!(res.unwrap(), "MyMouse");
    }

    #[test]
    fn test_build_rivalcfg_args_variations() {
        let s = Settings {
            sensitivity: Some("800".to_string()),
            polling_rate: Some("500".to_string()),
            sleep_timer: Some("10".to_string()),
            dim_timer: Some("3".to_string()),
            colour_switch: None,
        };
        let args = build_rivalcfg_args(&s);
        assert_eq!(args, vec![
            "--sensitivity".to_string(),
            "800".to_string(),
            "--polling-rate".to_string(),
            "500".to_string(),
            "--sleep-timer".to_string(),
            "10".to_string(),
            "--dim-timer".to_string(),
            "3".to_string(),
        ]);
    }
}

// Function to cleanup temp files
fn cleanup_temp_files() {
    if let Ok(mut cache) = PNG_CACHE.lock() {
        let mut to_remove = Vec::new();
        for (svg_path, (png_path, _)) in cache.iter() {
            if !std::path::Path::new(png_path).exists() {
                to_remove.push(svg_path.clone());
            } else {
                // Try to remove the temp file
                if let Err(e) = std::fs::remove_file(png_path) {
                    eprintln!("[rivalcfg-tray] Warning: Failed to cleanup temp file {}: {}", png_path, e);
                } else {
                    eprintln!("[rivalcfg-tray] Cleaned up temp file: {}", png_path);
                    to_remove.push(svg_path.clone());
                }
            }
        }
        for key in to_remove {
            cache.remove(&key);
        }
    }
}

fn generate_tray_icon(indicator: &Indicator) -> Option<(u8, bool)> {
    let (level, charging) = get_battery_level().unwrap_or((0, false));
    
    // Check if battery state has changed
    if let Ok(mut last_state) = LAST_BATTERY_STATE.lock() {
        if let Some((last_level, last_charging)) = *last_state {
            if last_level == level && last_charging == charging {
                eprintln!("[rivalcfg-tray] Battery state unchanged ({}%, charging: {}), skipping icon update", level, charging);
                return Some((level, charging));
            }
        }
        *last_state = Some((level, charging));
    }
    
    let icon_path = if charging {
        let charging_svg = find_icon("charging.svg")
            .unwrap_or_else(|| PathBuf::from("icons/charging.svg"));
        composite_battery_charging_svg(&battery_icon_path(level), &charging_svg)
            .unwrap_or(battery_icon_path(level))
    } else {
        battery_icon_path(level)
    };
    // Retry up to 5 times with exponential backoff if conversion fails
    let mut tries = 0;
    let png_path = loop {
        match svg_to_png_temp(&icon_path) {
            Some(p) => break Some(p),
            None if tries < 5 => {
                tries += 1;
                let delay_ms = 100_u64 << tries; // Exponential backoff: 200ms, 400ms, 800ms, 1600ms, 3200ms
                eprintln!("[rivalcfg-tray] SVG conversion failed (attempt {}), retrying in {}ms", tries, delay_ms);
                std::thread::sleep(std::time::Duration::from_millis(delay_ms));
            }
            None => {
                eprintln!("[rivalcfg-tray] Failed to convert SVG after {} attempts, giving up", tries + 1);
                break None;
            }
        }
    };
    if let Some(png_path) = png_path {
        // eprintln!("[rivalcfg-tray] Setting icon: {}", png_path);
        use std::io::Write;
        std::io::stderr().flush().ok();
        indicator.set_icon(&png_path);
    } else {
        eprintln!(
            "[rivalcfg-tray] Warning: Failed to convert SVG to PNG for icon: {} after retries",
            icon_path.display()
        );
        use std::io::Write;
        std::io::stderr().flush().ok();
    }
    Some((level, charging))
}

// use std::io::Stdout;
fn svg_to_png_temp(svg_path: &PathBuf) -> Option<String> {
    use std::process::Command;

    // Check cache first
    let svg_path_str = svg_path.to_string_lossy().to_string();
    let svg_modified = std::fs::metadata(svg_path).ok()?.modified().ok()?;
    
    if let Ok(cache) = PNG_CACHE.lock() {
        if let Some((cached_png_path, cached_time)) = cache.get(&svg_path_str) {
            // Check if cached version is still valid (file exists and SVG hasn't been modified)
            if std::path::Path::new(cached_png_path).exists() && *cached_time >= svg_modified {
                eprintln!("[rivalcfg-tray] Using cached PNG: {}", cached_png_path);
                return Some(cached_png_path.clone());
            }
        }
    }

    // Create a temp file with a unique name
    let temp_file = match tempfile::Builder::new()
        .prefix("rivalcfg-tray-")
        .suffix(".png")
        .tempfile() {
            Ok(file) => file,
            Err(e) => {
                eprintln!("[rivalcfg-tray] Failed to create temp file: {}", e);
                return None;
            }
    };

    let temp_path = temp_file.path().to_path_buf();
    eprintln!("[rivalcfg-tray] Converting SVG to PNG: {} -> {}", svg_path.display(), temp_path.display());

    // Convert SVG to PNG
    let output = Command::new("rsvg-convert")
        .arg("-w")
        .arg("64")
        .arg("-h")
        .arg("64")
        .arg("-o")
        .arg(&temp_path)
        .arg(svg_path)
        .output()
        .ok()?;

    if !output.status.success() {
        eprintln!(
            "[rivalcfg-tray] rsvg-convert failed:\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        return None;
    }

    if !temp_path.exists() {
        eprintln!("[rivalcfg-tray] PNG file was not created: {}", temp_path.display());
        return None;
    }

    eprintln!("[rivalcfg-tray] Successfully created PNG: {}", temp_path.display());
    
    // Keep the temp file around by leaking it
    std::mem::forget(temp_file);
    
    let png_path_str = temp_path.to_str()?.to_string();
    
    // Update cache
    if let Ok(mut cache) = PNG_CACHE.lock() {
        cache.insert(svg_path_str, (png_path_str.clone(), svg_modified));
    }
    
    Some(png_path_str)
}
use appindicator3::prelude::*;
use appindicator3::{Indicator, IndicatorCategory, IndicatorStatus};
use glib::ControlFlow;
use gtk::prelude::*;
use std::path::PathBuf;
// use std::process::Command; (moved to RealCommandRunner)
use std::time::Duration;

fn get_battery_level_with_runner(runner: &dyn CommandRunner) -> Option<(u8, bool)> {
    eprintln!("[rivalcfg-tray] Attempting to run rivalcfg --battery-level");
    let out = runner.run("rivalcfg", &["--battery-level"]);
    if !out.success {
        eprintln!("[rivalcfg-tray] rivalcfg command failed:\nstdout: {}\nstderr: {}", out.stdout, out.stderr);
        return None;
    }
    eprintln!("[rivalcfg-tray] rivalcfg output: {}", out.stdout);
    let charging_status = get_battery_status(&out.stdout)?;
    let second_last_word = out.stdout.split_whitespace().rev().nth(1)?;
    let trimmed = second_last_word.trim_end_matches('%');
    let percent = trimmed.parse::<u8>().ok()?;
    Some((percent, charging_status))
}

fn get_battery_level() -> Option<(u8, bool)> {
    let runner = RealCommandRunner::default();
    get_battery_level_with_runner(&runner)
}

fn get_battery_status(stdout: &str) -> Option<bool> {
    if stdout.contains("Discharging") {
        Some(false)
    } else if stdout.contains("Charging") {
        Some(true)
    } else {
        None
    }
}

fn get_mouse_name_with_runner(runner: &dyn CommandRunner) -> Option<String> {
    let out = runner.run("rivalcfg", &["--help"]);
    if !out.success {
        eprintln!("[rivalcfg-tray] rivalcfg command failed:\nstdout: {}\nstderr: {}", out.stdout, out.stderr);
        return None;
    }

    let stdout = out.stdout;
    // Find the line ending with "Options:"
    let options_line = stdout.lines().find(|line| line.ends_with("Options:"));
    if options_line.is_none() {
        eprintln!("[rivalcfg-tray] Warning: Could not find 'Options:' line in rivalcfg output");
        return None;
    }
    eprintln!("[rivalcfg-tray] Found 'Options:' line in rivalcfg output: {}", options_line.unwrap());
    // Extract mouse name from the output (trim "Options:" from the end of the line.)
    let mouse_name = options_line.unwrap().trim_end_matches("Options:").trim().to_string();
    eprintln!("[rivalcfg-tray] rivalcfg Mouse: {}", mouse_name);

    Some(mouse_name)
}

fn get_mouse_name() -> Option<String> {
    let runner = RealCommandRunner::default();
    get_mouse_name_with_runner(&runner)
}

fn find_icon(name: &str) -> Option<PathBuf> {
    let mut possible_paths = vec![
        // Standard freedesktop.org icon theme directories (where PKGBUILD installs icons)
        PathBuf::from(format!("/usr/share/icons/hicolor/scalable/apps/{}", name)),
        PathBuf::from(format!("/usr/share/icons/hicolor/symbolic/apps/{}", name)),
        // Check size-specific directories (16x16, 22x22, 24x24, 32x32, 48x48, 64x64, 128x128, 256x256)
        PathBuf::from(format!("/usr/share/icons/hicolor/16x16/apps/{}", name)),
        PathBuf::from(format!("/usr/share/icons/hicolor/22x22/apps/{}", name)),
        PathBuf::from(format!("/usr/share/icons/hicolor/24x24/apps/{}", name)),
        PathBuf::from(format!("/usr/share/icons/hicolor/32x32/apps/{}", name)),
        PathBuf::from(format!("/usr/share/icons/hicolor/48x48/apps/{}", name)),
        PathBuf::from(format!("/usr/share/icons/hicolor/64x64/apps/{}", name)),
        PathBuf::from(format!("/usr/share/icons/hicolor/128x128/apps/{}", name)),
        PathBuf::from(format!("/usr/share/icons/hicolor/256x256/apps/{}", name)),
        // Current directory (for development/testing)
        PathBuf::from(format!("icons/{}", name)),
        // Executable directory relative
        PathBuf::from(format!("bin/icons/{}", name)),
        // Flatpak directories
        PathBuf::from(format!("/app/bin/icons/{}", name)),
        PathBuf::from(format!("/app/share/icons/rivalcfgtray/{}", name)),
        PathBuf::from(format!("/app/share/icons/hicolor/scalable/apps/{}", name)),
        // System-wide installation (legacy path)
        PathBuf::from(format!("/usr/share/rivalcfgtray/icons/{}", name)),
    ];
    
    // Also try relative to the executable
    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            possible_paths.push(exe_dir.join("icons").join(name));
            // Try one directory up
            if let Some(parent) = exe_dir.parent() {
                possible_paths.push(parent.join("icons").join(name));
                possible_paths.push(parent.join("share").join("icons").join("rivalcfgtray").join(name));
            }
        }
    }
    
    // Try relative to the current working directory with more parent directories
    let mut current = std::env::current_dir().ok();
    while let Some(dir) = current {
        possible_paths.push(dir.join("icons").join(name));
        current = dir.parent().map(|p| p.to_path_buf());
    };

    for path in &possible_paths {
        if path.exists() {
            eprintln!("[rivalcfg-tray] Found icon at: {}", path.display());
            return Some(path.clone());
        }
    }
    eprintln!("[rivalcfg-tray] Warning: Could not find icon '{}' in any of these locations:", name);
    for path in &possible_paths {
        eprintln!("[rivalcfg-tray]   - {}", path.display());
    }
    None
}

fn battery_icon_path(level: u8) -> PathBuf {
    let name = if level > 90 {
        "battery-100.svg"
    } else if level > 74 {
        "battery-75.svg"
    } else if level > 49 {
        "battery-50.svg"
    } else if level > 24 {
        "battery-25.svg"
    } else if level > 9 {
        "battery-warn.svg"
    } else {
        "battery-0.svg"
    };

    find_icon(name).unwrap_or_else(|| PathBuf::from(format!("icons/{}", name)))
}

fn composite_battery_charging_svg(
    battery_svg: &PathBuf,
    charging_svg: &PathBuf,
) -> Option<PathBuf> {
    use std::fs;
    use std::io::Write;

    let battery_content = fs::read_to_string(battery_svg).ok()?;
    let mut charging_src = fs::read_to_string(charging_svg).ok()?;
    // Strip everything before the path element
    if let Some(pos) = charging_src.find("<path") {
        charging_src = charging_src[pos..].to_string();
    }
    // Strip everything after the path element
    if let Some(pos) = charging_src.rfind("</svg>") {
        charging_src = charging_src[..pos].to_string();
    }

    let charging_content = charging_src;

    // Simple SVG overlay by inserting charging SVG into battery SVG
    let composite_svg = battery_content.replace("</svg>", &format!("{}\n</svg>", charging_content));

    let mut tmp_path = env::temp_dir();
    let file_stem = battery_svg
        .file_stem()
        .and_then(std::ffi::OsStr::to_str)
        .unwrap_or("icon");
    tmp_path.push(format!("{}_charging.svg", file_stem));

    let mut file = fs::File::create(&tmp_path).ok()?;
    file.write_all(composite_svg.as_bytes()).ok()?;

    Some(tmp_path)
}

fn main() -> anyhow::Result<()> {
    gtk::init()?;

    // Create AppIndicator
    let (level, charging) = get_battery_level().unwrap_or((0, false));
    let mouse_name = get_mouse_name().unwrap_or_else(|| "SteelSeries Mouse".to_string());
    eprintln!(
        "[rivalcfg-tray] Starting tray for device: {} with battery level: {}%, charging: {}",
        mouse_name, level, charging
    );
    // Create menu
    let menu = gtk::Menu::new();
    let percent_item = gtk::MenuItem::with_label(&format!("Battery: {}%", level));
    percent_item.set_sensitive(false);
    menu.append(&percent_item);

    let status_item = gtk::MenuItem::with_label(&format!(
        "Status: {}",
        if charging { "Charging" } else { "Discharging" }
    ));
    status_item.set_sensitive(false);
    menu.append(&status_item);

    let mouse_name = mouse_name.clone();
    let config_item = gtk::MenuItem::with_label("Config");
    menu.append(&config_item);

    let separator = gtk::SeparatorMenuItem::new();
    menu.append(&separator);

    let colour_switch_item = gtk::MenuItem::with_label("Icon Colour Switch");
    colour_switch_item.set_sensitive(true);
    menu.append(&colour_switch_item);

    menu.append(&separator);

    let quit_item = gtk::MenuItem::with_label("Quit");
    menu.append(&quit_item);
    quit_item.connect_activate(|_| {
        gtk::main_quit();
    });
    menu.show_all();

    let indicator = Indicator::builder("rivalcfg-tray")
        .category(IndicatorCategory::ApplicationStatus)
        .menu(&menu)
        .status(IndicatorStatus::Active)
        .title(&format!("Battery: {}%", level))
        .build();

    // Create a shared command runner and apply any saved settings on startup
    let runner: Arc<dyn CommandRunner> = Arc::new(RealCommandRunner::default());
    if let Some(s) = load_settings() {
        let args = build_rivalcfg_args(&s);
        if !args.is_empty() {
            eprintln!("[rivalcfg-tray] Applying saved settings on startup: {:?}", &args);
            let slices = args.iter().map(|s| s.as_str()).collect::<Vec<&str>>();
            let out = runner.run("rivalcfg", &slices);
            if !out.success {
                eprintln!("[rivalcfg-tray] Failed to apply saved settings: {}", out.stderr);
            }
        }
    }

    generate_tray_icon(&indicator);

    // Config window logic
    let runner_for_ui = runner.clone();
    config_item.connect_activate(move |_| {
        use gtk::prelude::*;
        use gtk::{
            Box as GtkBox, Button, ButtonsType, ComboBoxText, DialogFlags, Entry, Label,
            MessageDialog, MessageType, Orientation, Window, WindowType,
        };
        use std::rc::Rc;

        let win = Rc::new(Window::new(WindowType::Toplevel));
        win.set_title("Rivalcfg GUI");
        win.set_default_size(400, 300);

        let vbox = GtkBox::new(Orientation::Vertical, 8);
        vbox.set_margin_top(10);
        vbox.set_margin_bottom(10);
        vbox.set_margin_start(10);
        vbox.set_margin_end(10);

        let title = Label::new(Some("SteelSeries Mouse Configuration"));
        title.set_markup("<span size='large'><b>SteelSeries Mouse Configuration</b></span>");
        vbox.pack_start(&title, false, false, 0);

        // Battery level
        let battery_label = Label::new(Some("Battery Level: N/A"));
        vbox.pack_start(&battery_label, false, false, 0);

        // Sensitivity (DPI)
        let sens_box = GtkBox::new(Orientation::Horizontal, 4);
        sens_box.pack_start(&Label::new(Some("Sensitivity (DPI):")), false, false, 0);
        let sensitivity_entry = Entry::new();
        sens_box.pack_start(&sensitivity_entry, true, true, 0);
        vbox.pack_start(&sens_box, false, false, 0);

        // Polling rate
        let poll_box = GtkBox::new(Orientation::Horizontal, 4);
        poll_box.pack_start(&Label::new(Some("Polling Rate (Hz):")), false, false, 0);
        let polling_rate_combo = ComboBoxText::new();
        for rate in &["125", "250", "500", "1000"] {
            polling_rate_combo.append_text(rate);
        }
        // polling_rate_combo default; we'll overwrite from saved settings below
        polling_rate_combo.set_active(Some(3));
        poll_box.pack_start(&polling_rate_combo, true, true, 0);
        vbox.pack_start(&poll_box, false, false, 0);

        // Sleep timer
        let sleep_box = GtkBox::new(Orientation::Horizontal, 4);
        sleep_box.pack_start(&Label::new(Some("Sleep Timer (minutes):")), false, false, 0);
        let sleep_timer_entry = Entry::new();
        sleep_box.pack_start(&sleep_timer_entry, true, true, 0);
        vbox.pack_start(&sleep_box, false, false, 0);

        // Dim timer
        let dim_box = GtkBox::new(Orientation::Horizontal, 4);
        dim_box.pack_start(&Label::new(Some("Dim Timer (seconds):")), false, false, 0);
        let dim_timer_entry = Entry::new();
        dim_box.pack_start(&dim_timer_entry, true, true, 0);
        vbox.pack_start(&dim_box, false, false, 0);

        // Buttons
        let btn_box = GtkBox::new(Orientation::Horizontal, 8);
        let apply_btn = Button::with_label("Apply Settings");
        let reset_btn = Button::with_label("Reset Settings");
        btn_box.pack_start(&apply_btn, true, true, 0);
        btn_box.pack_start(&reset_btn, true, true, 0);
        vbox.pack_start(&btn_box, false, false, 0);

        let show_btn = Button::with_label("Show Connected Devices");
        vbox.pack_start(&show_btn, false, false, 0);

        win.add(&vbox);
        win.show_all();

        // Helper to update battery label
        let battery_label_rc = Rc::new(battery_label);
        let win_apply = win.clone();
        let win_reset = win.clone();
        let win_show = win.clone();
        let update_battery = {
            let battery_label = battery_label_rc.clone();
            let runner = runner_for_ui.clone();
            move || {
                let out = runner.run("rivalcfg", &["--battery-level"]);
                let text = if out.success {
                    format!("Battery Level: {}", out.stdout.trim())
                } else {
                    "Battery Level: N/A".to_string()
                };
                battery_label.set_text(&text);
            }
        };
        update_battery();

        // Now fill UI from stored settings (after widgets are created)
        if let Some(s) = load_settings() {
            if let Some(ref pr) = s.polling_rate {
                let idx = match pr.as_str() {
                    "125" => 0,
                    "250" => 1,
                    "500" => 2,
                    "1000" => 3,
                    _ => 3,
                };
                polling_rate_combo.set_active(Some(idx));
            }
            if let Some(ref sens) = s.sensitivity {
                sensitivity_entry.set_text(sens);
            }
            if let Some(ref sleep_t) = s.sleep_timer {
                sleep_timer_entry.set_text(sleep_t);
            }
            if let Some(ref dim_t) = s.dim_timer {
                dim_timer_entry.set_text(dim_t);
            }
        }

        // Apply button logic
        let battery_label_apply = battery_label_rc.clone();
        let win_apply_clone = win_apply.clone();
        let sensitivity_entry_apply = sensitivity_entry.clone();
        let polling_rate_combo_apply = polling_rate_combo.clone();
        let sleep_timer_entry_apply = sleep_timer_entry.clone();
        let dim_timer_entry_apply = dim_timer_entry.clone();
        let runner_apply = runner_for_ui.clone();

        apply_btn.connect_clicked(move |_| {
            let sensitivity = sensitivity_entry_apply.text().to_string();

            // Validate fields before proceeding
            if let Err(msg) = validate_sensitivity(&sensitivity) {
                let dialog = MessageDialog::new(
                    Some(&*win_apply_clone),
                    DialogFlags::MODAL,
                    MessageType::Error,
                    ButtonsType::Ok,
                    &msg,
                );
                dialog.run();
                unsafe { dialog.destroy(); }
                return;
            }
            // sensitivity will be saved in Settings and applied below via runner
            let polling_rate = polling_rate_combo_apply.active_text().map(|s| s.to_string());
            if let Some(ref prate) = polling_rate {
                if let Err(msg) = validate_polling_rate(prate) {
                    let dialog = MessageDialog::new(
                        Some(&*win_apply_clone),
                        DialogFlags::MODAL,
                        MessageType::Error,
                        ButtonsType::Ok,
                        &msg,
                    );
                    dialog.run();
                    unsafe { dialog.destroy(); }
                    return;
                }
            }
            // polling_rate will be saved in Settings and applied below via runner
            let sleep_timer = sleep_timer_entry_apply.text().to_string();
            if let Err(msg) = validate_timer(&sleep_timer, "Sleep Timer") {
                let dialog = MessageDialog::new(
                    Some(&*win_apply_clone),
                    DialogFlags::MODAL,
                    MessageType::Error,
                    ButtonsType::Ok,
                    &msg,
                );
                dialog.run();
                unsafe { dialog.destroy(); }
                return;
            }
            // sleep_timer will be saved in Settings and applied below via runner
            let dim_timer = dim_timer_entry_apply.text().to_string();
            if let Err(msg) = validate_timer(&dim_timer, "Dim Timer") {
                let dialog = MessageDialog::new(
                    Some(&*win_apply_clone),
                    DialogFlags::MODAL,
                    MessageType::Error,
                    ButtonsType::Ok,
                    &msg,
                );
                dialog.run();
                unsafe { dialog.destroy(); }
                return;
            }
            // dim_timer will be saved in Settings and applied below via runner
            // Update battery using runner
            let out = runner_apply.run("rivalcfg", &["--battery-level"]);
            let text = if out.success {
                format!("Battery Level: {}", out.stdout.trim())
            } else {
                "Battery Level: N/A".to_string()
            };
            battery_label_apply.set_text(&text);
            // Save settings to disk
            let settings = Settings {
                sensitivity: if sensitivity.is_empty() { None } else { Some(sensitivity) },
                polling_rate: polling_rate.clone(),
                sleep_timer: if sleep_timer.is_empty() { None } else { Some(sleep_timer) },
                dim_timer: if dim_timer.is_empty() { None } else { Some(dim_timer) },
                colour_switch: None,
            };
            if let Err(e) = save_settings(&settings) {
                eprintln!("[rivalcfg-tray] Failed to save settings: {}", e);
            }
            // Apply settings via runner
            let args = build_rivalcfg_args(&settings);
            if !args.is_empty() {
                let slices = args.iter().map(|s| s.as_str()).collect::<Vec<&str>>();
                let out = runner_apply.run("rivalcfg", &slices);
                if !out.success {
                    let dialog = MessageDialog::new(
                        Some(&*win_apply_clone),
                        DialogFlags::MODAL,
                        MessageType::Error,
                        ButtonsType::Ok,
                        &format!("Error running the command: {}", out.stderr),
                    );
                    dialog.run();
                    unsafe {
                        dialog.destroy();
                    }
                }
            }
        });

        // Reset button logic
        reset_btn.connect_clicked(move |_| {
            let result = std::process::Command::new("rivalcfg").arg("-r").output();
            if let Ok(out) = result {
                let msg = String::from_utf8_lossy(&out.stdout).to_string();
                let dialog = MessageDialog::new(
                    Some(&*win_reset),
                    DialogFlags::MODAL,
                    MessageType::Info,
                    ButtonsType::Ok,
                    &msg,
                );
                dialog.run();
                unsafe {
                    dialog.destroy();
                }
            } else {
                let dialog = MessageDialog::new(
                    Some(&*win_reset),
                    DialogFlags::MODAL,
                    MessageType::Error,
                    ButtonsType::Ok,
                    "Error resetting settings",
                );
                dialog.run();
                unsafe {
                    dialog.destroy();
                }
            }
        });

        // Show devices button logic
        let mouse_name_clone = mouse_name.clone();
        show_btn.connect_clicked(move |_| {
            
            let dialog = MessageDialog::new(
                Some(&*win_show),
                DialogFlags::MODAL,
                MessageType::Info,
                ButtonsType::Ok,
                &mouse_name_clone,
            );
            dialog.run();
            unsafe {
                dialog.destroy();
            }
        });
    });

    colour_switch_item.connect_activate(move |_| {
        eprintln!("[rivalcfg-tray] Icon Colour Switch clicked - functionality not implemented yet.");
        // Placeholder for future functionality
    });

    // Update icon every 30 seconds
    let percent_item_clone = percent_item.clone();
    glib::timeout_add_local(Duration::from_secs(30), move || {
        let (level, charging) = generate_tray_icon(&indicator).unwrap_or((0, false));
        indicator.set_title(Some(&format!("Battery: {}%", level)));
        percent_item_clone.set_label(&format!("Battery: {}%", level));
        let status_text = format!(
            "Status: {}",
            if charging { "Charging" } else { "Discharging" }
        );
        status_item.set_label(&status_text);
        ControlFlow::Continue
    });

    // Cleanup temp files every 10 minutes
    glib::timeout_add_local(Duration::from_secs(600), move || {
        cleanup_temp_files();
        ControlFlow::Continue
    });

    gtk::main();
    
    // Cleanup temp files on exit
    cleanup_temp_files();
    Ok(())
}
