use std::sync::LazyLock;
use std::time::Duration;

use regex::Regex;
use tauri::AppHandle;
use tauri_plugin_shell::ShellExt;
use tokio::time::timeout;

use crate::error::AppError;
use crate::pm3::output_parser::strip_ansi;

/// Maximum time to wait for a PM3 subprocess to complete (30 seconds).
const PM3_COMMAND_TIMEOUT: Duration = Duration::from_secs(30);

/// Returns the ordered list of Tauri shell scope names to try when spawning the
/// PM3 binary. The first entry (`"proxmark3"`) resolves via PATH; subsequent
/// entries are platform-specific absolute paths registered in the shell scope.
/// This lets users who installed PM3 outside of PATH (common on Windows) still
/// use the app without manually editing their system PATH.
fn pm3_scope_names() -> Vec<&'static str> {
    let mut names = vec!["proxmark3"];

    if cfg!(target_os = "windows") {
        names.push("proxmark3-win-c");
        names.push("proxmark3-win-progfiles");
    } else if cfg!(target_os = "macos") {
        names.push("proxmark3-mac-local");
        names.push("proxmark3-mac-brew");
    } else {
        // Linux and other unix-like
        names.push("proxmark3-linux-local");
        names.push("proxmark3-linux-usr");
    }

    names
}

/// Validates that a port string matches expected serial port patterns.
/// Accepts COM1-COM256+ (Windows), /dev/ttyACM0-99, /dev/ttyUSB0-99 (Linux),
/// and /dev/tty.usbmodem* (macOS).
static PORT_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(COM[1-9]\d*|/dev/tty(ACM|USB)\d{1,2}|/dev/tty\.usbmodem\w+)$")
        .expect("bad port regex")
});

/// Run a single PM3 command: spawns `proxmark3 -p {port} -f -c "{cmd}"`,
/// waits for the process to exit (with a 30-second timeout), then returns cleaned stdout.
/// If the subprocess hangs (e.g., USB cable pulled), it will be killed after the timeout.
///
/// **Known limitation — subprocess cancellation on reset:**
/// This function uses `tauri_plugin_shell`'s `.output()` which internally spawns a child
/// process and collects stdout/stderr until exit. Because the child handle is owned by the
/// shell plugin's future and not exposed to callers, we cannot kill the subprocess from
/// outside (e.g., when the user presses Reset during a write operation). Refactoring to
/// `spawn()` + async stream reading would be needed for true cancellation support.
///
/// In practice this is acceptable because:
/// - T5577/EM4305 LF writes complete in under 2 seconds.
/// - The 30-second timeout (`PM3_COMMAND_TIMEOUT`) already protects against hangs.
/// - When the Tauri future is dropped (timeout or app shutdown), the shell plugin
///   cleans up the child process.
pub async fn run_command(app: &AppHandle, port: &str, cmd: &str) -> Result<String, AppError> {
    // Validate port format to prevent command injection via subprocess args
    if !PORT_RE.is_match(port) {
        return Err(AppError::CommandFailed(format!(
            "Invalid port: {}",
            port
        )));
    }

    // Reject command strings containing PM3 command separators or newlines.
    // The PM3 CLI's `-c` flag treats `;` as a delimiter, so a crafted value
    // like "AA;lf t55xx wipe" would execute two commands. Block this at the
    // chokepoint so no caller can accidentally pass through unsanitised input.
    if cmd.contains(';') || cmd.contains('\n') || cmd.contains('\r') {
        return Err(AppError::CommandFailed(
            "Invalid characters in command".into(),
        ));
    }
    if port.contains(';') || port.contains('\n') || port.contains('\r') {
        return Err(AppError::CommandFailed(
            "Invalid characters in command".into(),
        ));
    }

    // Try PATH-based lookup first, then fall back to common install locations.
    // Each scope name maps to a binary path registered in capabilities/default.json.
    let scope_names = pm3_scope_names();
    let mut first_spawn_error: Option<AppError> = None;

    for scope_name in &scope_names {
        let output_future = app
            .shell()
            .command(scope_name)
            .args(["-p", port, "-f", "-c", cmd])
            .output();

        // Note: When the timeout fires and the future is dropped, Tauri's shell plugin
        // handles cleanup of the child process. The `tokio::time::timeout` wrapper
        // ensures we don't wait forever, and the Tauri runtime drops the child on cancel.
        let output = match timeout(PM3_COMMAND_TIMEOUT, output_future).await {
            Err(_) => {
                return Err(AppError::Timeout(format!(
                    "PM3 command timed out after {}s: {}",
                    PM3_COMMAND_TIMEOUT.as_secs(),
                    cmd
                )));
            }
            Ok(Err(e)) => {
                // Spawn failed — binary not found at this path. Record the error
                // from the first attempt (PATH lookup) and try the next location.
                if first_spawn_error.is_none() {
                    first_spawn_error = Some(AppError::CommandFailed(format!(
                        "Failed to spawn proxmark3: {}",
                        e
                    )));
                }
                continue;
            }
            Ok(Ok(output)) => output,
        };

        // Binary was found and executed — process the result immediately.
        // No further fallback attempts needed regardless of exit code.
        let code = output.status.code().unwrap_or(-1);
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        return match code {
            0 => Ok(strip_ansi(&stdout)),
            -5 | 251 => Err(AppError::Timeout(format!(
                "PM3 timed out running: {}",
                cmd
            ))),
            _ => {
                let detail = if stderr.is_empty() {
                    strip_ansi(&stdout)
                } else {
                    strip_ansi(&stderr)
                };
                Err(AppError::CommandFailed(format!(
                    "Exit code {}: {}",
                    code, detail
                )))
            }
        };
    }

    // All scope names exhausted — return the first spawn error (from PATH lookup)
    // so the error message is the most user-recognizable one.
    Err(first_spawn_error.unwrap_or_else(|| {
        AppError::CommandFailed("Failed to spawn proxmark3: binary not found".into())
    }))
}

/// Scan common COM/serial ports trying `hw version` to find a connected PM3.
/// Returns (port, model, firmware) on success.
pub async fn detect_device(app: &AppHandle) -> Result<(String, String, String), AppError> {
    let candidates = build_port_candidates();
    let mut first_error: Option<AppError> = None;

    for port in &candidates {
        match run_command(app, port, "hw version").await {
            Ok(output) => {
                if let Some((model, firmware)) = parse_hw_version(&output) {
                    return Ok((port.clone(), model, firmware));
                }
            }
            Err(e) => {
                if first_error.is_none() {
                    first_error = Some(e);
                }
            }
        }
    }

    // Propagate the first error so callers can distinguish "binary not found"
    // from "device not connected" by inspecting the error message.
    match first_error {
        Some(e) => Err(e),
        None => Err(AppError::DeviceNotFound),
    }
}

fn build_port_candidates() -> Vec<String> {
    let mut ports = Vec::new();

    if cfg!(target_os = "windows") {
        // Windows COM ports — extend to 40 to cover USB hub reassignment
        for i in 1..=40 {
            ports.push(format!("COM{}", i));
        }
    } else if cfg!(target_os = "macos") {
        // macOS: /dev/tty.usbmodem* — cover common PM3 suffixes
        for suffix in &[
            "iceman1",
            "14101",
            "14201",
            "14301",
            "1",
            "2",
            "3",
        ] {
            ports.push(format!("/dev/tty.usbmodem{}", suffix));
        }
    } else {
        // Linux: /dev/ttyACM* and /dev/ttyUSB*
        for i in 0..=5 {
            ports.push(format!("/dev/ttyACM{}", i));
            ports.push(format!("/dev/ttyUSB{}", i));
        }
    }

    ports
}

fn parse_hw_version(output: &str) -> Option<(String, String)> {
    let mut model = String::from("Proxmark3");
    let mut firmware = String::new();
    let lower_output = output.to_lowercase();

    for line in output.lines() {
        let trimmed = line.trim();
        let lower = trimmed.to_lowercase();

        // Model detection: original "Prox" + "RFID" pattern, plus broader "Proxmark" match
        if (trimmed.contains("Prox") && trimmed.contains("RFID"))
            || lower.contains("proxmark")
        {
            // Lines like "[ Proxmark3 RFID instrument ]" or "Proxmark3 RDV4"
            let cleaned = trimmed.trim_matches(|c: char| !c.is_alphanumeric() && c != ' ');
            if !cleaned.is_empty() {
                model = cleaned.to_string();
            }
        }

        // Firmware detection: original patterns + broader resilience
        if trimmed.starts_with("firmware")
            || trimmed.contains("FW Version")
            || trimmed.starts_with("bootrom:")
            || lower.contains("compiled")
            || lower.contains("version")
        {
            firmware = trimmed.to_string();
        }
        // Iceman fork: "os: ..."
        if trimmed.starts_with("os:") {
            firmware = trimmed.to_string();
        }
    }

    if firmware.is_empty() {
        // Accept any output that mentions proxmark — it's a valid device
        if lower_output.contains("proxmark") {
            firmware = "unknown".to_string();
        } else {
            return None;
        }
    }

    Some((model, firmware))
}
