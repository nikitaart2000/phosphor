use std::sync::Mutex;
use std::time::Instant;

use serde::Serialize;
use tauri::{AppHandle, Emitter, State};

use crate::cards::types::{AutopwnEvent, BlankType, CardType, ProcessPhase, RecoveryAction};
use crate::error::AppError;
use crate::pm3::connection::HfOperationState;
use crate::pm3::{command_builder, connection, output_parser};
use crate::state::{WizardAction, WizardMachine, WizardState};

/// Payload emitted as `hf-progress` events during autopwn.
#[derive(Debug, Clone, Serialize)]
struct HfProgressPayload {
    phase: String,
    keys_found: u32,
    keys_total: u32,
    elapsed_secs: u32,
}

/// Run `hf mf autopwn` with streaming progress. Recovers all keys and dumps
/// the card memory. Long-running (seconds to hours depending on PRNG type).
///
/// Transitions: CardIdentified -> HfProcessing -> HfDumpReady (or Error).
#[tauri::command]
pub async fn hf_autopwn(
    app: AppHandle,
    machine: State<'_, Mutex<WizardMachine>>,
    hf_state: State<'_, HfOperationState>,
) -> Result<WizardState, AppError> {
    // Extract port + card_type from current state, then transition to HfProcessing
    let (port, card_type) = {
        let mut m = machine.lock().map_err(|e| {
            AppError::CommandFailed(format!("State lock poisoned: {}", e))
        })?;
        let (port, card_type) = match &m.current {
            WizardState::CardIdentified { card_type, .. } => {
                match card_type {
                    CardType::MifareClassic1K | CardType::MifareClassic4K => {}
                    _ => {
                        return Err(AppError::CommandFailed(format!(
                            "Autopwn only supports MIFARE Classic, got {:?}",
                            card_type
                        )));
                    }
                }
                let port = m.port.clone().ok_or_else(|| {
                    AppError::InvalidTransition("No port in machine state".to_string())
                })?;
                (port, card_type.clone())
            }
            _ => {
                return Err(AppError::InvalidTransition(
                    "Must be in CardIdentified to run autopwn".to_string(),
                ));
            }
        };
        m.transition(WizardAction::StartHfProcess)?;
        (port, card_type)
    };

    let cmd = command_builder::build_hf_autopwn(&card_type);
    let start_time = Instant::now();

    // Progress state tracked across lines via the closure
    let mut current_phase = ProcessPhase::KeyCheck;
    let mut keys_found: u32 = 0;
    // Initialize keys_total based on card type so individual KeyFound events
    // produce visible progress (Classic 1K = 16 sectors × 2 keys = 32,
    // Classic 4K = 40 sectors × 2 keys = 80). Without this, keys_total stays 0
    // until the summary line "found X/Y keys (D)" which arrives at the END.
    let mut keys_total: u32 = match card_type {
        CardType::MifareClassic4K => 80,
        _ => 32,
    };
    let mut dump_file: Option<String> = None;
    let mut dump_complete = false;
    let mut dump_partial = false;

    let app_for_closure = app.clone();

    // Emit initial progress so the frontend shows 0/32 (or 0/80) immediately
    let _ = app.emit(
        "hf-progress",
        HfProgressPayload {
            phase: format!("{:?}", current_phase),
            keys_found: 0,
            keys_total,
            elapsed_secs: 0,
        },
    );

    // Run streaming command with per-line autopwn parsing (1h timeout for hardnested)
    let result = connection::run_command_streaming(
        &app,
        &port,
        &cmd,
        3600,
        &hf_state,
        |line| {
            if let Some(event) = output_parser::parse_autopwn_line(line) {
                let elapsed = start_time.elapsed().as_secs() as u32;

                match &event {
                    AutopwnEvent::DictionaryProgress { found, total } => {
                        current_phase = ProcessPhase::KeyCheck;
                        keys_found = *found;
                        keys_total = *total;
                    }
                    AutopwnEvent::KeyFound { .. } => {
                        keys_found += 1;
                    }
                    AutopwnEvent::DarksideStarted => {
                        current_phase = ProcessPhase::Darkside;
                    }
                    AutopwnEvent::NestedStarted => {
                        current_phase = ProcessPhase::Nested;
                    }
                    AutopwnEvent::HardnestedStarted => {
                        current_phase = ProcessPhase::Hardnested;
                    }
                    AutopwnEvent::StaticnestedStarted => {
                        current_phase = ProcessPhase::StaticNested;
                    }
                    AutopwnEvent::DumpComplete { file_path } => {
                        dump_complete = true;
                        if !file_path.is_empty() {
                            dump_file = Some(file_path.clone());
                        }
                        current_phase = ProcessPhase::Dumping;
                    }
                    AutopwnEvent::DumpPartial { file_path } => {
                        dump_partial = true;
                        if !file_path.is_empty() {
                            dump_file = Some(file_path.clone());
                        }
                        current_phase = ProcessPhase::Dumping;
                    }
                    AutopwnEvent::Failed { .. } | AutopwnEvent::Finished { .. } => {}
                }

                // Emit progress event to frontend
                let _ = app_for_closure.emit(
                    "hf-progress",
                    HfProgressPayload {
                        phase: format!("{:?}", current_phase),
                        keys_found,
                        keys_total,
                        elapsed_secs: elapsed,
                    },
                );
            }
        },
    )
    .await;

    match result {
        Ok(_output) => {
            // Store dump file path in HfOperationState for the write phase
            if let Some(ref path) = dump_file {
                if let Ok(mut lock) = hf_state.dump_path.lock() {
                    *lock = Some(path.clone());
                }
            }

            let dump_info = if dump_complete {
                format!(
                    "All keys recovered ({}/{}). Full dump saved.",
                    keys_found, keys_total
                )
            } else if dump_partial {
                format!(
                    "Partial key recovery ({}/{}). Partial dump saved.",
                    keys_found, keys_total
                )
            } else if keys_found > 0 {
                format!("Keys recovered: {}/{}.", keys_found, keys_total)
            } else {
                "Key recovery completed.".to_string()
            };

            // Transition to HfDumpReady
            let mut m = machine.lock().map_err(|e| {
                AppError::CommandFailed(format!("State lock poisoned: {}", e))
            })?;
            m.transition(WizardAction::HfProcessComplete { dump_info })?;
            Ok(m.current.clone())
        }
        Err(e) => {
            let mut m = machine.lock().map_err(|e| {
                AppError::CommandFailed(format!("State lock poisoned: {}", e))
            })?;
            m.transition(WizardAction::ReportError {
                message: e.to_string(),
                user_message: "Key recovery failed. Check device connection and try again."
                    .to_string(),
                recoverable: true,
                recovery_action: Some(crate::cards::types::RecoveryAction::Retry),
            })?;
            Ok(m.current.clone())
        }
    }
}

/// Cancel a running HF operation (autopwn, dump, write) by killing the child process.
#[tauri::command]
pub async fn cancel_hf_operation(
    hf_state: tauri::State<'_, HfOperationState>,
) -> Result<(), AppError> {
    let child = {
        let mut lock = hf_state.child.lock().map_err(|e| {
            AppError::CommandFailed(format!("HF state lock poisoned: {}", e))
        })?;
        lock.take()
    };

    match child {
        Some(child) => {
            child.kill().map_err(|e| {
                AppError::CommandFailed(format!("Failed to kill HF process: {}", e))
            })?;
            Ok(())
        }
        None => Ok(()),
    }
}

// ---------------------------------------------------------------------------
// HF Write Clone — 7 workflows
// ---------------------------------------------------------------------------

/// Write a dump to a magic blank card. Selects the correct write workflow based
/// on `blank_type`. The dump file path is retrieved from `HfOperationState`
/// (stored by `hf_autopwn` or `hf_dump`).
///
/// Transitions: BlankDetected -> Writing -> Verifying (or Error).
///
/// `source_uid` is passed from the frontend XState context because the Rust FSM
/// doesn't persist `card_data` after state transitions.
#[tauri::command]
pub async fn hf_write_clone(
    app: AppHandle,
    source_uid: String,
    card_type: CardType,
    blank_type: BlankType,
    machine: State<'_, Mutex<WizardMachine>>,
    hf_state: State<'_, HfOperationState>,
) -> Result<WizardState, AppError> {
    // Extract port from machine, validate state
    let port = {
        let mut m = machine.lock().map_err(|e| {
            AppError::CommandFailed(format!("State lock poisoned: {}", e))
        })?;
        match &m.current {
            WizardState::BlankDetected { .. } => {}
            _ => {
                return Err(AppError::InvalidTransition(
                    "Must be in BlankDetected to write clone".to_string(),
                ));
            }
        }
        let port = m.port.clone().ok_or_else(|| {
            AppError::InvalidTransition("No port in machine state".to_string())
        })?;
        m.transition(WizardAction::StartWrite)?;
        port
    };

    // Get dump file path from HfOperationState (set by hf_autopwn or hf_dump)
    let dump_path = {
        let lock = hf_state.dump_path.lock().map_err(|e| {
            AppError::CommandFailed(format!("HF state lock poisoned: {}", e))
        })?;
        lock.clone().ok_or_else(|| {
            AppError::CommandFailed("No dump file available. Run key recovery first.".to_string())
        })?
    };

    // Run the write workflow, catching errors to report via FSM
    let result = match blank_type {
        BlankType::MagicMifareGen1a => {
            write_gen1a(&app, &port, &dump_path, &machine).await
        }
        BlankType::MagicMifareGen2 => {
            write_gen2(&app, &port, &dump_path, &source_uid, &card_type, &machine).await
        }
        BlankType::MagicMifareGen3 => {
            write_gen3(&app, &port, &dump_path, &source_uid, &card_type, &machine).await
        }
        BlankType::MagicMifareGen4GTU => {
            write_gen4_gtu(&app, &port, &dump_path, &machine).await
        }
        BlankType::MagicMifareGen4GDM => {
            write_gen4_gdm(&app, &port, &dump_path, &machine).await
        }
        BlankType::MagicUltralight => {
            write_ultralight(&app, &port, &dump_path, &machine).await
        }
        BlankType::IClassBlank => {
            write_iclass(&app, &port, &dump_path, &machine).await
        }
        _ => {
            Err(AppError::CommandFailed(format!(
                "Unsupported HF blank type: {:?}",
                blank_type
            )))
        }
    };

    match result {
        Ok(state) => Ok(state),
        Err(e) => {
            report_error(
                &machine,
                &e.to_string(),
                "Write failed. Do not remove the card — try again.",
                true,
                Some(RecoveryAction::Retry),
            )
        }
    }
}

// ---------------------------------------------------------------------------
// HF Dump (UL/NTAG + iCLASS — no autopwn needed)
// ---------------------------------------------------------------------------

/// Dump an unencrypted HF card (Ultralight/NTAG or iCLASS Legacy).
/// These cards don't need autopwn key recovery — UL/NTAG is unencrypted,
/// iCLASS Legacy uses the leaked master key (key index 0).
///
/// Transitions: CardIdentified -> HfProcessing -> HfDumpReady (or Error).
#[tauri::command]
pub async fn hf_dump(
    app: AppHandle,
    machine: State<'_, Mutex<WizardMachine>>,
    hf_state: State<'_, HfOperationState>,
) -> Result<WizardState, AppError> {
    // Extract port + card_type, transition to HfProcessing
    let (port, card_type) = {
        let mut m = machine.lock().map_err(|e| {
            AppError::CommandFailed(format!("State lock poisoned: {}", e))
        })?;
        let (port, card_type) = match &m.current {
            WizardState::CardIdentified { card_type, .. } => {
                match card_type {
                    CardType::MifareUltralight | CardType::NTAG | CardType::IClass => {}
                    _ => {
                        return Err(AppError::CommandFailed(format!(
                            "hf_dump only supports UL/NTAG/iCLASS, got {:?}",
                            card_type
                        )));
                    }
                }
                let port = m.port.clone().ok_or_else(|| {
                    AppError::InvalidTransition("No port in machine state".to_string())
                })?;
                (port, card_type.clone())
            }
            _ => {
                return Err(AppError::InvalidTransition(
                    "Must be in CardIdentified to run dump".to_string(),
                ));
            }
        };
        m.transition(WizardAction::StartHfProcess)?;
        (port, card_type)
    };

    // Select dump command based on card type
    let cmd = match card_type {
        CardType::IClass => command_builder::build_iclass_dump(),
        _ => command_builder::build_mfu_dump(), // UL + NTAG
    };

    let result = connection::run_command(&app, &port, cmd).await;

    match result {
        Ok(output) => {
            // Extract dump file path from output
            let dump_file = output_parser::extract_dump_file_path(&output);

            // Store dump path in HfOperationState for the write phase
            if let Some(ref path) = dump_file {
                if let Ok(mut lock) = hf_state.dump_path.lock() {
                    *lock = Some(path.clone());
                }
            }

            let dump_info = match &card_type {
                CardType::IClass => "iCLASS dump complete.".to_string(),
                CardType::NTAG => "NTAG dump complete.".to_string(),
                _ => "Ultralight dump complete.".to_string(),
            };

            let mut m = machine.lock().map_err(|e| {
                AppError::CommandFailed(format!("State lock poisoned: {}", e))
            })?;
            m.transition(WizardAction::HfProcessComplete { dump_info })?;
            Ok(m.current.clone())
        }
        Err(e) => {
            let mut m = machine.lock().map_err(|e| {
                AppError::CommandFailed(format!("State lock poisoned: {}", e))
            })?;
            m.transition(WizardAction::ReportError {
                message: e.to_string(),
                user_message: "Dump failed. Check device connection and try again.".to_string(),
                recoverable: true,
                recovery_action: Some(RecoveryAction::Retry),
            })?;
            Ok(m.current.clone())
        }
    }
}

// ---------------------------------------------------------------------------
// Write workflow implementations
// ---------------------------------------------------------------------------

/// Gen1a: single `hf mf cload` via magic wakeup backdoor.
async fn write_gen1a(
    app: &AppHandle,
    port: &str,
    dump_path: &str,
    machine: &State<'_, Mutex<WizardMachine>>,
) -> Result<WizardState, AppError> {
    update_write_progress(app, machine, 0.3, Some(1), Some(2))?;

    let cmd = command_builder::build_mf_cload(dump_path);
    let output = connection::run_command(app, port, &cmd).await?;
    check_write_output(&output)?;

    finish_write(app, machine).await
}

/// Gen2/CUID: config force -> wrbl0 -> restore -> config reset.
async fn write_gen2(
    app: &AppHandle,
    port: &str,
    dump_path: &str,
    _source_uid: &str,
    _card_type: &CardType,
    machine: &State<'_, Mutex<WizardMachine>>,
) -> Result<WizardState, AppError> {
    let total: u16 = 5;

    // Step 1: Force 14a config to allow block 0 write
    update_write_progress(app, machine, 0.1, Some(1), Some(total))?;
    let cmd = command_builder::build_mf_gen2_config_force();
    connection::run_command(app, port, cmd).await?;

    // Step 2: Read block 0 from dump and force-write it
    update_write_progress(app, machine, 0.3, Some(2), Some(total))?;
    let block0 = read_block0_from_dump(dump_path)?;
    let cmd = command_builder::build_mf_wrbl0("FFFFFFFFFFFF", &block0);
    let output = connection::run_command(app, port, &cmd).await?;
    check_write_output(&output)?;

    // Step 3: Restore all blocks from dump
    update_write_progress(app, machine, 0.6, Some(3), Some(total))?;
    let cmd = command_builder::build_mf_restore(dump_path);
    let output = connection::run_command(app, port, &cmd).await?;
    check_write_output(&output)?;

    // Step 4: Reset 14a config to standard
    update_write_progress(app, machine, 0.85, Some(4), Some(total))?;
    let cmd = command_builder::build_mf_gen2_config_reset();
    connection::run_command(app, port, cmd).await?;

    finish_write(app, machine).await
}

/// Gen3: gen3uid -> gen3blk -> restore.
async fn write_gen3(
    app: &AppHandle,
    port: &str,
    dump_path: &str,
    source_uid: &str,
    _card_type: &CardType,
    machine: &State<'_, Mutex<WizardMachine>>,
) -> Result<WizardState, AppError> {
    let total: u16 = 4;

    // Step 1: Set UID via APDU
    update_write_progress(app, machine, 0.1, Some(1), Some(total))?;
    // Extract UID without spaces/colons for gen3uid command
    let clean_uid: String = source_uid.chars().filter(|c| c.is_ascii_hexdigit()).collect();
    let cmd = command_builder::build_mf_gen3uid(&clean_uid);
    let output = connection::run_command(app, port, &cmd).await?;
    check_write_output(&output)?;

    // Step 2: Write block 0 via APDU
    update_write_progress(app, machine, 0.35, Some(2), Some(total))?;
    let block0 = read_block0_from_dump(dump_path)?;
    let cmd = command_builder::build_mf_gen3blk(&block0);
    let output = connection::run_command(app, port, &cmd).await?;
    check_write_output(&output)?;

    // Step 3: Restore all blocks from dump
    update_write_progress(app, machine, 0.65, Some(3), Some(total))?;
    let cmd = command_builder::build_mf_restore(dump_path);
    let output = connection::run_command(app, port, &cmd).await?;
    check_write_output(&output)?;

    finish_write(app, machine).await
}

/// Gen4 GTU/UMC: single `hf mf gload` (GTU-specific file load).
async fn write_gen4_gtu(
    app: &AppHandle,
    port: &str,
    dump_path: &str,
    machine: &State<'_, Mutex<WizardMachine>>,
) -> Result<WizardState, AppError> {
    update_write_progress(app, machine, 0.3, Some(1), Some(2))?;

    let cmd = command_builder::build_mf_gload(dump_path);
    let output = connection::run_command(app, port, &cmd).await?;
    check_write_output(&output)?;

    finish_write(app, machine).await
}

/// Gen4 GDM: uses `hf mf cload` via Gen1a backdoor (factory default 7AFF
/// has Gen1a enabled). Single command instead of block-by-block gdmsetblk.
async fn write_gen4_gdm(
    app: &AppHandle,
    port: &str,
    dump_path: &str,
    machine: &State<'_, Mutex<WizardMachine>>,
) -> Result<WizardState, AppError> {
    update_write_progress(app, machine, 0.3, Some(1), Some(2))?;

    let cmd = command_builder::build_mf_cload(dump_path);
    let output = connection::run_command(app, port, &cmd).await?;
    check_write_output(&output)?;

    finish_write(app, machine).await
}

/// UL/NTAG: single `hf mfu restore` with special pages + engineering mode.
async fn write_ultralight(
    app: &AppHandle,
    port: &str,
    dump_path: &str,
    machine: &State<'_, Mutex<WizardMachine>>,
) -> Result<WizardState, AppError> {
    update_write_progress(app, machine, 0.3, Some(1), Some(2))?;

    let cmd = command_builder::build_mfu_restore(dump_path);
    let output = connection::run_command(app, port, &cmd).await?;
    check_write_output(&output)?;

    finish_write(app, machine).await
}

/// iCLASS: single `hf iclass restore` with default key (key index 0).
async fn write_iclass(
    app: &AppHandle,
    port: &str,
    dump_path: &str,
    machine: &State<'_, Mutex<WizardMachine>>,
) -> Result<WizardState, AppError> {
    update_write_progress(app, machine, 0.3, Some(1), Some(2))?;

    let cmd = command_builder::build_iclass_restore(dump_path);
    let output = connection::run_command(app, port, &cmd).await?;
    check_write_output(&output)?;

    finish_write(app, machine).await
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Read the first 16 bytes of a binary dump file and return as a 32-char hex string.
/// Used by Gen2 (`wrbl0 --force`) and Gen3 (`gen3blk`) to extract block 0 data.
fn read_block0_from_dump(dump_path: &str) -> Result<String, AppError> {
    let data = std::fs::read(dump_path).map_err(|e| {
        AppError::CommandFailed(format!("Failed to read dump file '{}': {}", dump_path, e))
    })?;

    if data.len() < 16 {
        return Err(AppError::CommandFailed(format!(
            "Dump file too small ({} bytes, need at least 16)",
            data.len()
        )));
    }

    // Convert first 16 bytes to uppercase hex (32 chars)
    Ok(data[..16].iter().map(|b| format!("{:02X}", b)).collect())
}

/// Transition FSM: Writing -> Verifying (write finished).
async fn finish_write(
    app: &AppHandle,
    machine: &State<'_, Mutex<WizardMachine>>,
) -> Result<WizardState, AppError> {
    update_write_progress(app, machine, 1.0, None, None)?;

    let mut m = machine.lock().map_err(|e| {
        AppError::CommandFailed(format!("State lock poisoned: {}", e))
    })?;
    m.transition(WizardAction::WriteFinished)?;
    Ok(m.current.clone())
}

/// Emit write progress and update FSM.
fn update_write_progress(
    app: &AppHandle,
    machine: &State<'_, Mutex<WizardMachine>>,
    progress: f32,
    current_step: Option<u16>,
    total_steps: Option<u16>,
) -> Result<(), AppError> {
    let mut m = machine.lock().map_err(|e| {
        AppError::CommandFailed(format!("State lock poisoned: {}", e))
    })?;
    m.transition(WizardAction::UpdateWriteProgress {
        progress,
        current_block: current_step,
        total_blocks: total_steps,
    })?;
    let _ = app.emit(
        "write-progress",
        serde_json::json!({
            "progress": progress,
            "current_block": current_step,
            "total_blocks": total_steps,
        }),
    );
    Ok(())
}

/// Check PM3 write output for critical errors (`[!!]`).
fn check_write_output(output: &str) -> Result<(), AppError> {
    if output.contains("[!!]") {
        // Extract the error line for diagnostics
        let err_line = output
            .lines()
            .find(|l| l.contains("[!!]"))
            .unwrap_or("Unknown error");
        return Err(AppError::CommandFailed(format!(
            "PM3 write error: {}",
            err_line.trim()
        )));
    }
    Ok(())
}

/// Report an error via FSM transition and return the resulting state.
fn report_error(
    machine: &State<'_, Mutex<WizardMachine>>,
    message: &str,
    user_message: &str,
    recoverable: bool,
    recovery_action: Option<RecoveryAction>,
) -> Result<WizardState, AppError> {
    let mut m = machine.lock().map_err(|e| {
        AppError::CommandFailed(format!("State lock poisoned: {}", e))
    })?;
    m.transition(WizardAction::ReportError {
        message: message.to_string(),
        user_message: user_message.to_string(),
        recoverable,
        recovery_action,
    })?;
    Ok(m.current.clone())
}

// ---------------------------------------------------------------------------
// HF Verification — read back + compare
// ---------------------------------------------------------------------------

/// Verify an HF clone by reading back the card and comparing with the source.
///
/// Strategy:
/// 1. `hf search` — confirm card responds, extract UID
/// 2. UID comparison with source (primary check)
/// 3. Readback via type-appropriate command (cview for Gen1a, dump for others)
/// 4. Dump file comparison if both original and readback files are available
///
/// Transitions: Verifying -> VerificationComplete.
#[tauri::command]
pub async fn hf_verify_clone(
    app: AppHandle,
    source_uid: String,
    _card_type: CardType,
    blank_type: BlankType,
    machine: State<'_, Mutex<WizardMachine>>,
    hf_state: State<'_, HfOperationState>,
) -> Result<WizardState, AppError> {
    // Guard: must be in Verifying state
    let port = {
        let m = machine.lock().map_err(|e| {
            AppError::CommandFailed(format!("State lock poisoned: {}", e))
        })?;
        match &m.current {
            WizardState::Verifying => {}
            other => {
                return Err(AppError::InvalidTransition(format!(
                    "Must be in Verifying state to verify clone, currently in {:?}",
                    std::mem::discriminant(other)
                )));
            }
        }
        m.port.clone().ok_or_else(|| {
            AppError::InvalidTransition("No port in machine state".to_string())
        })?
    };

    // Step 1: hf search — confirm card responds and extract UID
    let search_output =
        connection::run_command(&app, &port, command_builder::build_hf_search()).await;

    let uid_match = match &search_output {
        Ok(output) => {
            if let Some((_, card_data)) = output_parser::parse_hf_search(output) {
                let clean_source: String = source_uid
                    .chars()
                    .filter(|c| c.is_ascii_hexdigit())
                    .collect::<String>()
                    .to_uppercase();
                let clean_detected: String = card_data
                    .uid
                    .chars()
                    .filter(|c| c.is_ascii_hexdigit())
                    .collect::<String>()
                    .to_uppercase();
                clean_source == clean_detected
            } else {
                false
            }
        }
        Err(_) => false,
    };

    if !uid_match {
        let mut m = machine.lock().map_err(|e| {
            AppError::CommandFailed(format!("State lock poisoned: {}", e))
        })?;
        m.transition(WizardAction::VerificationResult {
            success: false,
            mismatched_blocks: vec![0], // block 0 = UID mismatch sentinel
        })?;
        return Ok(m.current.clone());
    }

    // Step 2: Deeper readback verification by blank type
    let mismatched_blocks = match blank_type {
        BlankType::MagicMifareGen1a => {
            // Gen1a: read all blocks via backdoor (no keys needed)
            verify_readback(
                &app,
                &port,
                command_builder::build_mf_cview(),
                &hf_state,
                16,
            )
            .await
        }
        BlankType::MagicMifareGen2
        | BlankType::MagicMifareGen3
        | BlankType::MagicMifareGen4GTU
        | BlankType::MagicMifareGen4GDM => {
            // Gen2/Gen3/Gen4: read back using recovered keys
            verify_readback(
                &app,
                &port,
                command_builder::build_mf_dump(),
                &hf_state,
                16,
            )
            .await
        }
        BlankType::MagicUltralight => {
            // UL/NTAG: dump pages and compare
            verify_readback(
                &app,
                &port,
                command_builder::build_mfu_dump(),
                &hf_state,
                4,
            )
            .await
        }
        BlankType::IClassBlank => {
            // iCLASS: dump blocks and compare
            verify_readback(
                &app,
                &port,
                command_builder::build_iclass_dump(),
                &hf_state,
                8,
            )
            .await
        }
        _ => vec![],
    };

    let success = mismatched_blocks.is_empty();

    let mut m = machine.lock().map_err(|e| {
        AppError::CommandFailed(format!("State lock poisoned: {}", e))
    })?;
    m.transition(WizardAction::VerificationResult {
        success,
        mismatched_blocks,
    })?;
    Ok(m.current.clone())
}

/// Run a readback command and optionally compare the resulting dump with the original.
/// Returns empty vec on success, vec of mismatched block indices on failure.
/// Readback errors are non-fatal — UID already matched as the primary check.
async fn verify_readback(
    app: &AppHandle,
    port: &str,
    readback_cmd: &str,
    hf_state: &State<'_, HfOperationState>,
    block_size: usize,
) -> Vec<u16> {
    let output = match connection::run_command(app, port, readback_cmd).await {
        Ok(o) => o,
        Err(_) => return vec![], // Readback failed, fall back to UID-only
    };

    // Check for critical PM3 errors
    if output.contains("[!!]") {
        return vec![0];
    }

    // Try dump file comparison if both original and readback files are available
    let readback_path = output_parser::extract_dump_file_path(&output);
    let original_path = hf_state.dump_path.lock().ok().and_then(|l| l.clone());

    match (original_path, readback_path) {
        (Some(ref orig), Some(ref readback)) => {
            compare_dump_files(orig, readback, block_size)
        }
        _ => vec![], // No files to compare, UID matched = success
    }
}

/// Compare two binary dump files block by block.
/// Returns mismatched block indices (empty = all blocks match).
fn compare_dump_files(original: &str, readback: &str, block_size: usize) -> Vec<u16> {
    let orig_data = match std::fs::read(original) {
        Ok(d) => d,
        Err(_) => return vec![],
    };
    let readback_data = match std::fs::read(readback) {
        Ok(d) => d,
        Err(_) => return vec![],
    };

    if orig_data.is_empty() || readback_data.is_empty() || block_size == 0 {
        return vec![];
    }

    let compare_len = orig_data.len().min(readback_data.len());
    let blocks = compare_len / block_size;
    let mut mismatched = Vec::new();

    for i in 0..blocks {
        let start = i * block_size;
        let end = start + block_size;
        if orig_data[start..end] != readback_data[start..end] {
            mismatched.push(i as u16);
        }
    }

    mismatched
}
