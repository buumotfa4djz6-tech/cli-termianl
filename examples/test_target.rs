use std::io::{self, BufRead, Write};

/// Simulated device state.
struct DeviceState {
    light_on: bool,
    current_a: f64,
    voltage_v: f64,
    uptime_ticks: u64,
}

impl DeviceState {
    fn new() -> Self {
        Self {
            light_on: false,
            current_a: 0.0,
            voltage_v: 3.3,
            uptime_ticks: 0,
        }
    }

    fn status(&self) -> String {
        format!(
            "LIGHT: {}\nCURRENT: {:.2} A\nVOLTAGE: {:.2} V\nUPTICK: {}\n",
            if self.light_on { "ON" } else { "OFF" },
            self.current_a,
            self.voltage_v,
            self.uptime_ticks,
        )
    }
}

/// Command dispatch result.
#[derive(Debug, PartialEq)]
enum CmdResult {
    Output(String),
    Exit,
    Noop,
}

fn main() {
    let mut state = DeviceState::new();
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut out = stdout.lock();

    writeln!(out, "=== Test Target v0.2 ===").unwrap();
    writeln!(out, "Type 'help' for commands.").unwrap();
    out.flush().unwrap();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                writeln!(out, "ERROR: read stdin: {e}").unwrap();
                break;
            }
        };

        state.uptime_ticks += 1;

        match dispatch(&line, &mut state) {
            CmdResult::Output(s) => {
                write!(out, "{s}").unwrap();
            }
            CmdResult::Exit => {
                writeln!(out, "OK: Exiting").unwrap();
                let _ = out.flush();
                return;
            }
            CmdResult::Noop => {}
        }
        let _ = out.flush();
    }
}

fn dispatch(input: &str, state: &mut DeviceState) -> CmdResult {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return CmdResult::Noop;
    }

    let parts: Vec<&str> = trimmed.splitn(2, char::is_whitespace).collect();
    match parts[0] {
        "quit" | "exit" => CmdResult::Exit,
        "help" => CmdResult::Output(help_text()),
        "echo" => match parts.get(1) {
            Some(msg) => CmdResult::Output(format!("ECHO: {msg}\n")),
            None => CmdResult::Output("ECHO: (empty)\n".to_string()),
        },
        "switchls" => match parts.get(1).copied() {
            Some("0") => {
                state.light_on = false;
                CmdResult::Output("OK: Light OFF\n".to_string())
            }
            Some("1") => {
                state.light_on = true;
                CmdResult::Output("OK: Light ON\n".to_string())
            }
            _ => CmdResult::Output("ERROR: usage: switchls <0|1>\n".to_string()),
        },
        "setcurr" => match parts.get(1).and_then(|v| v.parse::<f64>().ok()) {
            Some(v) if (0.0..=10.0).contains(&v) => {
                state.current_a = v;
                CmdResult::Output(format!("OK: current={v:.2} A\n"))
            }
            Some(_) => CmdResult::Output("ERROR: current out of range (0-10 A)\n".to_string()),
            None => CmdResult::Output("ERROR: usage: setcurr <0.0-10.0>\n".to_string()),
        },
        "setvolt" => match parts.get(1).and_then(|v| v.parse::<f64>().ok()) {
            Some(v) if (1.0..=5.0).contains(&v) => {
                state.voltage_v = v;
                CmdResult::Output(format!("OK: voltage={v:.2} V\n"))
            }
            Some(_) => CmdResult::Output("ERROR: voltage out of range (1-5 V)\n".to_string()),
            None => CmdResult::Output("ERROR: usage: setvolt <1.0-5.0>\n".to_string()),
        },
        "readvolt" => CmdResult::Output(format!("VOLTAGE: {:.2} V\n", state.voltage_v)),
        "readcurr" => CmdResult::Output(format!("CURRENT: {:.2} A\n", state.current_a)),
        "status" => CmdResult::Output(state.status()),
        _ => CmdResult::Output(format!("WARN: unknown command: {trimmed}\n")),
    }
}

fn help_text() -> String {
    [
        "Commands:",
        "  echo <text>      - Echo text back",
        "  switchls <0|1>   - Toggle light source",
        "  setcurr <0-10>   - Set current (A)",
        "  setvolt <1-5>    - Set voltage (V)",
        "  readvolt         - Read voltage",
        "  readcurr         - Read current",
        "  status           - Show device status",
        "  help             - Show this help",
        "  quit             - Exit",
        "",
    ]
    .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_echo() {
        let mut s = DeviceState::new();
        assert_eq!(dispatch("echo hello", &mut s), CmdResult::Output("ECHO: hello\n".to_string()));
    }

    #[test]
    fn test_switchls() {
        let mut s = DeviceState::new();
        assert!(matches!(dispatch("switchls 1", &mut s), CmdResult::Output(_)));
        assert!(s.light_on);
        assert!(matches!(dispatch("switchls 0", &mut s), CmdResult::Output(_)));
        assert!(!s.light_on);
        assert!(matches!(dispatch("switchls 2", &mut s), CmdResult::Output(_)));
    }

    #[test]
    fn test_setcurr_valid_and_range() {
        let mut s = DeviceState::new();
        assert!(matches!(dispatch("setcurr 5.0", &mut s), CmdResult::Output(o) if o.contains("5.00")));
        assert!((s.current_a - 5.0).abs() < f64::EPSILON);
        assert!(matches!(dispatch("setcurr 99", &mut s), CmdResult::Output(o) if o.contains("out of range")));
    }

    #[test]
    fn test_setcurr_invalid() {
        let mut s = DeviceState::new();
        assert!(matches!(dispatch("setcurr abc", &mut s), CmdResult::Output(o) if o.contains("usage")));
    }

    #[test]
    fn test_setvolt() {
        let mut s = DeviceState::new();
        assert!(matches!(dispatch("setvolt 4.2", &mut s), CmdResult::Output(o) if o.contains("4.20")));
        assert!((s.voltage_v - 4.2).abs() < f64::EPSILON);
    }

    #[test]
    fn test_readvolt_readcurr() {
        let mut s = DeviceState::new();
        s.voltage_v = 3.5;
        s.current_a = 1.2;
        assert!(matches!(dispatch("readvolt", &mut s), CmdResult::Output(o) if o.contains("3.50")));
        assert!(matches!(dispatch("readcurr", &mut s), CmdResult::Output(o) if o.contains("1.20")));
    }

    #[test]
    fn test_status() {
        let mut s = DeviceState::new();
        s.light_on = true;
        let CmdResult::Output(out) = dispatch("status", &mut s) else { panic!() };
        assert!(out.contains("LIGHT: ON"));
        assert!(out.contains("CURRENT:"));
    }

    #[test]
    fn test_quit() {
        let mut s = DeviceState::new();
        assert!(matches!(dispatch("quit", &mut s), CmdResult::Exit));
    }

    #[test]
    fn test_unknown() {
        let mut s = DeviceState::new();
        assert!(matches!(dispatch("foobar", &mut s), CmdResult::Output(o) if o.contains("unknown")));
    }

    #[test]
    fn test_empty() {
        let mut s = DeviceState::new();
        assert!(matches!(dispatch("", &mut s), CmdResult::Noop));
    }
}
