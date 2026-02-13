use std::sync::{LazyLock, Mutex};
use std::time::Duration;

use regex::Regex;
use serde::Serialize;
use tauri::{AppHandle, Emitter};
use tauri_plugin_shell::process::{CommandChild, CommandEvent};
use tauri_plugin_shell::ShellExt;
use tokio::time::timeout;

use crate::error::AppError;
use crate::pm3::output_parser::strip_ansi;

/// Payload emitted as `pm3-output` events for the live terminal panel.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Pm3OutputPayload {
    pub text: String,
    pub is_error: bool,
}

/// Emit raw PM3 output to the frontend terminal panel.
pub fn emit_output(app: &AppHandle, text: &str, is_error: bool) {
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let _ = app.emit(
            "pm3-output",
            Pm3OutputPayload {
                text: trimmed.to_string(),
                is_error,
            },
        );
    }
}

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

/// Internal PM3 execution that does NOT emit to the frontend.
/// Handles: port validation, command sanitization, sidecar fallback, PATH lookup,
/// process spawn, output collection, ANSI stripping, and timeout.
/// Returns the cleaned output string on success.
async fn execute_pm3(app: &AppHandle, port: &str, cmd: &str) -> Result<String, AppError> {
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

    // 1) Try bundled sidecar binary first (available in production builds).
    //    In dev mode the sidecar won't exist, so this silently falls through.
    match try_sidecar_silent(app, port, cmd).await {
        Ok(output) => return Ok(output),
        Err(_) => { /* sidecar not available -- fall through to PATH/scope lookup */ }
    }

    // 2) Fall back to PATH-based lookup, then common install locations.
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
                // Spawn failed -- binary not found at this path. Record the error
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

        // Binary was found and executed -- process the result immediately.
        // No further fallback attempts needed regardless of exit code.
        let code = output.status.code().unwrap_or(-1);
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        return match code {
            0 => {
                let cleaned = strip_ansi(&stdout);
                Ok(cleaned)
            }
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

    // All scope names exhausted -- return the first spawn error (from PATH lookup)
    // so the error message is the most user-recognizable one.
    Err(first_spawn_error.unwrap_or_else(|| {
        AppError::CommandFailed("Failed to spawn proxmark3: binary not found".into())
    }))
}

/// Run a single PM3 command: spawns `proxmark3 -p {port} -f -c "{cmd}"`,
/// waits for the process to exit (with a 30-second timeout), then returns cleaned stdout.
/// If the subprocess hangs (e.g., USB cable pulled), it will be killed after the timeout.
///
/// Emits the command being run and its output to the frontend terminal panel.
///
/// **Known limitation -- subprocess cancellation on reset:**
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
    emit_output(app, &format!("pm3 --> {}", cmd), false);
    match execute_pm3(app, port, cmd).await {
        Ok(output) => {
            emit_output(app, &output, false);
            Ok(output)
        }
        Err(e) => {
            emit_output(app, &e.to_string(), true);
            Err(e)
        }
    }
}

// ---------------------------------------------------------------------------
// HF Operation State — holds child process for cancellation + dump file path
// ---------------------------------------------------------------------------

/// Managed state for long-running HF operations (autopwn, dump, write).
/// Stored via `app.manage()` in `lib.rs`.
pub struct HfOperationState {
    /// Running child process — `take()` to kill via `CommandChild::kill(self)`.
    pub child: Mutex<Option<CommandChild>>,
    /// Dump file path set by autopwn after completion (e.g. "hf-mf-01020304-dump.bin").
    pub dump_path: Mutex<Option<String>>,
}

impl HfOperationState {
    pub fn new() -> Self {
        Self {
            child: Mutex::new(None),
            dump_path: Mutex::new(None),
        }
    }
}

// ---------------------------------------------------------------------------
// Streaming command execution (HF operations)
// ---------------------------------------------------------------------------

/// Run a PM3 command with streaming output, supporting long timeouts and
/// cancellation. Unlike `run_command()` which uses `.output()` (blocks until
/// exit), this uses `.spawn()` + async line reading.
///
/// - Each stdout/stderr line is emitted as a `pm3-output` event (live terminal).
/// - A per-line callback `on_line` is invoked for real-time parsing (e.g. autopwn
///   progress events). The callback receives the cleaned line text.
/// - The child process is stored in `hf_state.child` so `cancel_hf_operation`
///   can kill it mid-run.
/// - Returns the accumulated cleaned output on success.
pub async fn run_command_streaming<F>(
    app: &AppHandle,
    port: &str,
    cmd: &str,
    timeout_secs: u64,
    hf_state: &HfOperationState,
    mut on_line: F,
) -> Result<String, AppError>
where
    F: FnMut(&str),
{
    // Validate port
    if !PORT_RE.is_match(port) {
        return Err(AppError::CommandFailed(format!("Invalid port: {}", port)));
    }

    // Reject command separators
    if cmd.contains(';') || cmd.contains('\n') || cmd.contains('\r') {
        return Err(AppError::CommandFailed(
            "Invalid characters in command".into(),
        ));
    }

    emit_output(app, &format!("pm3 --> {}", cmd), false);

    // Try sidecar first, then scope names — same strategy as execute_pm3()
    let (rx, child) = spawn_pm3(app, port, cmd)?;

    // Store child for cancellation
    {
        let mut lock = hf_state.child.lock().map_err(|e| {
            AppError::CommandFailed(format!("HF state lock poisoned: {}", e))
        })?;
        *lock = Some(child);
    }

    // Read lines with timeout
    let result = read_stream_with_timeout(app, rx, timeout_secs, &mut on_line).await;

    // Clear child on completion (process already exited or was killed)
    {
        let mut lock = hf_state.child.lock().unwrap_or_else(|e| e.into_inner());
        *lock = None;
    }

    match result {
        Ok(output) => Ok(output),
        Err(e) => {
            emit_output(app, &e.to_string(), true);
            Err(e)
        }
    }
}

/// Spawn PM3 via sidecar or scope names, returning the event receiver + child.
fn spawn_pm3(
    app: &AppHandle,
    port: &str,
    cmd: &str,
) -> Result<(tauri::async_runtime::Receiver<CommandEvent>, CommandChild), AppError> {
    let args = ["-p", port, "-f", "-c", cmd];

    // Try sidecar first
    if let Ok(sidecar_cmd) = app.shell().sidecar("binaries/proxmark3") {
        if let Ok(result) = sidecar_cmd.args(&args).spawn() {
            return Ok(result);
        }
    }

    // Fall back to scope names
    let scope_names = pm3_scope_names();
    let mut first_err: Option<String> = None;

    for scope_name in &scope_names {
        match app.shell().command(scope_name).args(&args).spawn() {
            Ok(result) => return Ok(result),
            Err(e) => {
                if first_err.is_none() {
                    first_err = Some(format!("{}", e));
                }
            }
        }
    }

    Err(AppError::CommandFailed(format!(
        "Failed to spawn proxmark3: {}",
        first_err.unwrap_or_else(|| "binary not found".into())
    )))
}

/// Read from a `CommandEvent` receiver, accumulating output and emitting lines.
/// Returns the full cleaned output when the process terminates.
async fn read_stream_with_timeout<F>(
    app: &AppHandle,
    mut rx: tauri::async_runtime::Receiver<CommandEvent>,
    timeout_secs: u64,
    on_line: &mut F,
) -> Result<String, AppError>
where
    F: FnMut(&str),
{
    let deadline = Duration::from_secs(timeout_secs);
    let mut accumulated = String::new();
    let mut exit_code: Option<i32> = None;

    loop {
        match timeout(deadline, rx.recv()).await {
            Err(_) => {
                // Timeout expired
                return Err(AppError::Timeout(format!(
                    "HF operation timed out after {}s",
                    timeout_secs
                )));
            }
            Ok(None) => {
                // Channel closed — process exited
                break;
            }
            Ok(Some(event)) => match event {
                CommandEvent::Stdout(bytes) => {
                    let line = String::from_utf8_lossy(&bytes);
                    let cleaned = strip_ansi(&line);
                    let trimmed = cleaned.trim();
                    if !trimmed.is_empty() {
                        emit_output(app, trimmed, false);
                        on_line(trimmed);
                        accumulated.push_str(trimmed);
                        accumulated.push('\n');
                    }
                }
                CommandEvent::Stderr(bytes) => {
                    let line = String::from_utf8_lossy(&bytes);
                    let cleaned = strip_ansi(&line);
                    let trimmed = cleaned.trim();
                    if !trimmed.is_empty() {
                        emit_output(app, trimmed, true);
                        on_line(trimmed);
                        accumulated.push_str(trimmed);
                        accumulated.push('\n');
                    }
                }
                CommandEvent::Error(msg) => {
                    emit_output(app, &msg, true);
                    return Err(AppError::CommandFailed(format!(
                        "Process error: {}",
                        msg
                    )));
                }
                CommandEvent::Terminated(payload) => {
                    exit_code = payload.code;
                    break;
                }
                _ => {} // Future CommandEvent variants — ignore
            },
        }
    }

    // Check exit code
    match exit_code {
        Some(0) | None => Ok(accumulated),
        Some(-5) | Some(251) => Err(AppError::Timeout("PM3 subprocess timed out".into())),
        Some(code) => Err(AppError::CommandFailed(format!(
            "PM3 exited with code {}",
            code
        ))),
    }
}

/// Scan common COM/serial ports trying `hw version` to find a connected PM3.
/// Returns (port, model, firmware) on success.
///
/// Uses friendly, hacker-casual terminal output. All probe messages are green
/// (non-error) except the final "not found" message.
pub async fn detect_device(app: &AppHandle) -> Result<(String, String, String), AppError> {
    let candidates = build_port_candidates();

    // Pick a random init message for personality
    let init_msgs = [
        "[=] Sniffing USB bus... come out, Proxmark",
        "[=] Deploying port tentacles...",
        "[=] Hunting for hardware... stay still",
        "[=] Scanning the wire... don't be shy",
        "[=] Reaching out to the other side...",
    ];
    let idx = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos() as usize
        % init_msgs.len();
    emit_output(app, init_msgs[idx], false);

    for port in &candidates {
        emit_output(app, &format!("[=] Knocking on {}...", port), false);

        match execute_pm3(app, port, "hw version").await {
            Ok(output) => {
                if let Some((model, firmware)) = parse_hw_version(&output) {
                    emit_output(app, &format!("[+] Target acquired: {} on {}", model, port), false);
                    emit_output(app, &format!("[+] Firmware: {}", firmware), false);
                    return Ok((port.clone(), model, firmware));
                }
                // Got output but couldn't parse hw version -- wrong device
                emit_output(app, &format!("[-] {} -- wrong device", port), false);
            }
            Err(e) => {
                // Capabilities mismatch means the PM3 device IS present on this
                // port but the firmware doesn't match the client version. Treat
                // it as a successful detection -- the firmware check step will
                // handle the mismatch and offer to flash.
                let err_msg = e.to_string();
                if err_msg.to_lowercase().contains("capabilities") {
                    emit_output(app, &format!("[+] Target acquired: Proxmark3 on {} (firmware mismatch)", port), false);
                    return Ok((
                        port.clone(),
                        "Proxmark3".to_string(),
                        "mismatched".to_string(),
                    ));
                }

                // Distinguish "no response" (spawn succeeded but device didn't respond)
                // from other errors. If spawn itself failed (binary not found), that
                // affects ALL ports, so propagate immediately.
                if err_msg.contains("Failed to spawn proxmark3") {
                    emit_output(app, "[!!] Proxmark3 binary not found. Check installation.", true);
                    return Err(e);
                }

                emit_output(app, &format!("[-] {} -- no response", port), false);
            }
        }
    }

    emit_output(app, "[!!] No Proxmark3 found.", true);
    emit_output(app, "[=] Try a different USB cable (some are charge-only)", false);
    emit_output(app, "[=] Check Device Manager for a COM port", false);
    emit_output(app, "[=] PM3 Easy: may need CH340 driver (wch-ic.com)", false);
    Err(AppError::DeviceNotFound)
}

fn build_port_candidates() -> Vec<String> {
    let mut ports = Vec::new();

    if cfg!(target_os = "windows") {
        // Windows COM ports -- extend to 40 to cover USB hub reassignment
        for i in 1..=40 {
            ports.push(format!("COM{}", i));
        }
    } else if cfg!(target_os = "macos") {
        // macOS: /dev/tty.usbmodem* -- cover common PM3 suffixes
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

/// Attempt to run a PM3 command via the bundled sidecar binary (silent -- no emit).
/// Returns Ok(stdout) on success, Err on any failure (sidecar not found, spawn
/// error, non-zero exit code). Callers should fall through to PATH-based lookup
/// on failure.
///
/// The sidecar binary depends on DLLs (Qt5, ICU, etc.) bundled in the same
/// directory via `bundle.resources`. The Windows DLL loader finds them
/// automatically since they share the sidecar's directory.
async fn try_sidecar_silent(app: &AppHandle, port: &str, cmd: &str) -> Result<String, AppError> {
    let sidecar = app
        .shell()
        .sidecar("binaries/proxmark3")
        .map_err(|e| AppError::CommandFailed(format!("Sidecar not available: {}", e)))?;

    let output_future = sidecar
        .args(["-p", port, "-f", "-c", cmd])
        .output();

    let output = match timeout(PM3_COMMAND_TIMEOUT, output_future).await {
        Err(_) => {
            return Err(AppError::Timeout(format!(
                "PM3 command timed out after {}s: {}",
                PM3_COMMAND_TIMEOUT.as_secs(),
                cmd
            )));
        }
        Ok(Err(e)) => {
            return Err(AppError::CommandFailed(format!(
                "Failed to run sidecar: {}",
                e
            )));
        }
        Ok(Ok(output)) => output,
    };

    let code = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    match code {
        0 => {
            let cleaned = strip_ansi(&stdout);
            Ok(cleaned)
        }
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

fn parse_hw_version(output: &str) -> Option<(String, String)> {
    use crate::pm3::version::parse_detailed_hw_version;

    let info = parse_detailed_hw_version(output);

    // Pick best version source: os_version (device firmware) > client_version
    let version_str = if !info.os_version.is_empty() {
        info.os_version
    } else if !info.client_version.is_empty() {
        info.client_version
    } else {
        // No version found — check if it's at least a PM3 device
        if output.to_lowercase().contains("proxmark") {
            return Some((info.model, "unknown".to_string()));
        }
        return None;
    };

    // Extract clean short version like "v4.20728" for sidebar display
    let firmware = extract_short_version(&version_str);
    Some((info.model, firmware))
}

/// Extract a short version string like "v4.20728" from a full version string
/// like "Iceman/master/v4.20728-358-ga2ba91043-suspect".
fn extract_short_version(version_str: &str) -> String {
    // Find 'v' followed by a digit
    let v_pos = version_str.char_indices().find(|&(i, c)| {
        c == 'v'
            && version_str
                .get(i + 1..i + 2)
                .map_or(false, |s| s.as_bytes().first().map_or(false, |b| b.is_ascii_digit()))
    });

    if let Some((pos, _)) = v_pos {
        let rest = &version_str[pos..];
        // Version is "v" + digits/dots, stop at anything else
        let end = rest
            .find(|c: char| c != 'v' && !c.is_ascii_digit() && c != '.')
            .unwrap_or(rest.len());
        rest[..end].to_string()
    } else {
        version_str.to_string()
    }
}
