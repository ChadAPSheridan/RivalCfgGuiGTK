use super::*;
use crate::cmd::{CommandOutput, get_battery_level_with_runner, get_mouse_name_with_runner, build_rivalcfg_args};
use std::collections::HashMap;
use std::sync::Mutex;
use std::fs;

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

    #[allow(dead_code)]
    fn get_calls(&self) -> Vec<(String, Vec<String>)> {
        self.calls.lock().unwrap().clone()
    }
}

impl crate::cmd::CommandRunner for MockCommandRunner {
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
            _code: None,
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
fn settings_serde_roundtrip() {
    let s = Settings {
        sensitivity: Some("800".to_string()),
        polling_rate: Some("1000".to_string()),
        sleep_timer: Some("15".to_string()),
        dim_timer: Some("5".to_string()),
        colour_mode: Some("custom".to_string()),
        custom_color: Some("#ff8800".to_string()),
    };
    let json = serde_json::to_string(&s).expect("serialize");
    let parsed: Settings = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(parsed.colour_mode, s.colour_mode);
    assert_eq!(parsed.custom_color, s.custom_color);
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
            _code: Some(0),
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
            _code: Some(0),
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
            _code: Some(0),
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
        colour_mode: None,
        custom_color: None,
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

#[test]
fn recolor_svg_temp_creates_file_and_contains_color() {
    // Minimal SVG with a rect using fill="#000"
    let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" width="10" height="10">
<rect width="10" height="10" fill="#000"/>
</svg>"##;
    let tmp = std::env::temp_dir().join("rivalcfg-test-input.svg");
    fs::write(&tmp, svg).expect("write sample svg");

    let out = recolor_svg_to_temp(&tmp, "#ff8800");
    assert!(out.is_some(), "recolor_svg_to_temp returned None");
    let path = out.unwrap();
    let data = fs::read_to_string(&path).expect("read recolored svg");
    assert!(data.contains("#ff8800"), "recolored svg should contain the new color");
    // cleanup
    let _ = fs::remove_file(tmp);
    let _ = fs::remove_file(path);
}
