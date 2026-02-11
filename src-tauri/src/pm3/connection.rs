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

/// Validates that a port string matches expected serial port patterns.
/// Accepts COM1-COM99 (Windows), /dev/ttyACM0-99, /dev/ttyUSB0-99 (Linux),
/// and /dev/tty.usbmodem* (macOS).
static PORT_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(COM\d{1,2}|/dev/tty(ACM|USB)\d{1,2}|/dev/tty\.usbmodem\w+)$")
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

    let output_future = app
        .shell()
        .command("proxmark3")
        .args(["-p", port, "-f", "-c", cmd])
        .output();

    // Note: When the timeout fires and the future is dropped, Tauri's shell plugin
    // handles cleanup of the child process. The `tokio::time::timeout` wrapper
    // ensures we don't wait forever, and the Tauri runtime drops the child on cancel.
    let output = timeout(PM3_COMMAND_TIMEOUT, output_future)
        .await
        .map_err(|_| {
            AppError::Timeout(format!(
                "PM3 command timed out after {}s: {}",
                PM3_COMMAND_TIMEOUT.as_secs(),
                cmd
            ))
        })?
        .map_err(|e| AppError::CommandFailed(format!("Failed to spawn proxmark3: {}", e)))?;

    let code = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    match code {
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
    }
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
    // Windows COM ports
    for i in 1..=20 {
        ports.push(format!("COM{}", i));
    }
    // Linux / macOS
    for i in 0..=5 {
        ports.push(format!("/dev/ttyACM{}", i));
        ports.push(format!("/dev/ttyUSB{}", i));
    }
    ports.push("/dev/tty.usbmodemiceman1".to_string());
    ports
}

fn parse_hw_version(output: &str) -> Option<(String, String)> {
    let mut model = String::from("Proxmark3");
    let mut firmware = String::new();

    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.contains("Prox") && trimmed.contains("RFID") {
            // Lines like "[ Proxmark3 RFID instrument ]"
            let cleaned = trimmed.trim_matches(|c: char| !c.is_alphanumeric() && c != ' ');
            if !cleaned.is_empty() {
                model = cleaned.to_string();
            }
        }
        if trimmed.starts_with("firmware") || trimmed.contains("FW Version") {
            firmware = trimmed.to_string();
        }
        // Iceman fork: "os: ..."
        if trimmed.starts_with("os:") {
            firmware = trimmed.to_string();
        }
    }

    if firmware.is_empty() {
        // Accept any output that mentions proxmark
        if output.to_lowercase().contains("proxmark") {
            firmware = "unknown".to_string();
        } else {
            return None;
        }
    }

    Some((model, firmware))
}
