use std::env;

fn generate_tray_icon(indicator: &Indicator) -> Option<(u8, bool)> {
    let (level, charging) = get_battery_level().unwrap_or((0, false));
    let icon_path = if charging {
        let charging_svg = find_icon("charging.svg")
            .unwrap_or_else(|| PathBuf::from("icons/charging.svg"));
        composite_battery_charging_svg(&battery_icon_path(level), &charging_svg)
            .unwrap_or(battery_icon_path(level))
    } else {
        battery_icon_path(level)
    };
    // Retry up to 5 times with 200ms delay if conversion fails
    let mut tries = 0;
    let png_path = loop {
        match svg_to_png_temp(&icon_path) {
            Some(p) => break Some(p),
            None if tries < 5 => {
                tries += 1;
                std::thread::sleep(std::time::Duration::from_millis(200));
            }
            None => break None,
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
    
    Command::new("rsvg-convert")
        .arg("64")
        .arg("-o")
        .arg(&temp_path)
        .arg(svg_path)
        .output()
        .ok()?;

    if !temp_path.exists() {
        eprintln!("[rivalcfg-tray] PNG file was not created: {}", temp_path.display());
        return None;
    }

    eprintln!("[rivalcfg-tray] Successfully created PNG: {}", temp_path.display());
    Some(temp_path.to_str()?.to_string())
}
use appindicator3::prelude::*;
use appindicator3::{Indicator, IndicatorCategory, IndicatorStatus};
use glib::ControlFlow;
use gtk::prelude::*;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

fn get_battery_level() -> Option<(u8, bool)> {
    eprintln!("[rivalcfg-tray] Attempting to run rivalcfg --battery-level");
    let output = Command::new("rivalcfg")
        .arg("--battery-level")
        .output()
        .map_err(|e| {
            eprintln!("[rivalcfg-tray] Failed to execute rivalcfg: {}", e);
            e
        })
        .ok()?;

    if !output.status.success() {
        eprintln!("[rivalcfg-tray] rivalcfg command failed:\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr));
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    eprintln!("[rivalcfg-tray] rivalcfg output: {}", stdout);
    let charging_status = get_battery_status(&stdout)?;
    let second_last_word = stdout.split_whitespace().rev().nth(1)?;
    let trimmed = second_last_word.trim_end_matches('%');
    let percent = trimmed.parse::<u8>().ok()?;

    Some((percent, charging_status))
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

fn get_mouse_name() -> Option<String> {
    let output = Command::new("rivalcfg")
        .arg("--help")
        .output()
        .ok()?;

    if !output.status.success() {
        eprintln!("[rivalcfg-tray] rivalcfg command failed:\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr));
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
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

    generate_tray_icon(&indicator);

    // Config window logic
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
            move || {
                let output = std::process::Command::new("rivalcfg")
                    .arg("--battery-level")
                    .output();
                let text = if let Ok(out) = output {
                    if out.status.success() {
                        format!(
                            "Battery Level: {}",
                            String::from_utf8_lossy(&out.stdout).trim()
                        )
                    } else {
                        "Battery Level: N/A".to_string()
                    }
                } else {
                    "Battery Level: N/A".to_string()
                };
                battery_label.set_text(&text);
            }
        };
        update_battery();

        // Apply button logic
        let battery_label_apply = battery_label_rc.clone();
        apply_btn.connect_clicked(move |_| {
            let mut command = vec!["rivalcfg".to_string()];
            let sensitivity = sensitivity_entry.text().to_string();
            if !sensitivity.is_empty() {
                command.push("--sensitivity".to_string());
                command.push(sensitivity);
            }
            let polling_rate = polling_rate_combo.active_text().map(|s| s.to_string());
            if let Some(rate) = polling_rate {
                command.push("--polling-rate".to_string());
                command.push(rate);
            }
            let sleep_timer = sleep_timer_entry.text().to_string();
            if !sleep_timer.is_empty() {
                command.push("--sleep-timer".to_string());
                command.push(sleep_timer);
            }
            let dim_timer = dim_timer_entry.text().to_string();
            if !dim_timer.is_empty() {
                command.push("--dim-timer".to_string());
                command.push(dim_timer);
            }
            // Update battery
            let output = std::process::Command::new("rivalcfg")
                .arg("--battery-level")
                .output();
            let text = if let Ok(out) = output {
                if out.status.success() {
                    format!(
                        "Battery Level: {}",
                        String::from_utf8_lossy(&out.stdout).trim()
                    )
                } else {
                    "Battery Level: N/A".to_string()
                }
            } else {
                "Battery Level: N/A".to_string()
            };
            battery_label_apply.set_text(&text);
            // Apply settings
            if command.len() > 1 {
                let result = std::process::Command::new(&command[0])
                    .args(&command[1..])
                    .output();
                if let Err(e) = result {
                    let dialog = MessageDialog::new(
                        Some(&*win_apply),
                        DialogFlags::MODAL,
                        MessageType::Error,
                        ButtonsType::Ok,
                        &format!("Error running the command: {}", e),
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

    gtk::main();
    Ok(())
}
