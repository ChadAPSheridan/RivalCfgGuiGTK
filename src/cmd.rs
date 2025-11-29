// PathBuf is not needed at top-level in this module right now

#[derive(Debug, Clone)]
pub struct CommandOutput {
    pub stdout: String,
    pub stderr: String,
    pub success: bool,
    // exit code if available; currently unused but kept for future diagnostics
    #[allow(dead_code)]
    pub _code: Option<i32>,
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
                _code: o.status.code(),
            },
            Err(e) => CommandOutput {
                stdout: String::new(),
                stderr: format!("Failed to spawn {}: {}", program, e),
                success: false,
                _code: None,
            },
        }
    }
}

/// Build arguments for `rivalcfg` from Settings. Returns only the args (no program name).
pub fn build_rivalcfg_args(s: &crate::Settings) -> Vec<String> {
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

pub fn get_battery_status(stdout: &str) -> Option<bool> {
    if stdout.contains("Discharging") {
        Some(false)
    } else if stdout.contains("Charging") {
        Some(true)
    } else {
        None
    }
}

// get_battery_status is public already; no re-export needed here

pub fn get_battery_level_with_runner(runner: &dyn CommandRunner) -> Option<(u8, bool)> {
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

pub fn get_battery_level() -> Option<(u8, bool)> {
    let runner = RealCommandRunner::default();
    get_battery_level_with_runner(&runner)
}

pub fn get_mouse_name_with_runner(runner: &dyn CommandRunner) -> Option<String> {
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

pub fn get_mouse_name() -> Option<String> {
    let runner = RealCommandRunner::default();
    get_mouse_name_with_runner(&runner)
}

// Tests were moved into `src/tests.rs` so this module is intentionally empty.