use std::sync::Mutex;
use tauri::{AppHandle, Emitter, State};

use crate::cards::types::{BlankType, CardType, RecoveryAction};
use crate::error::AppError;
use crate::pm3::{command_builder, connection, output_parser};
use crate::state::{WizardAction, WizardMachine, WizardState};

/// Total progress steps for the T5577 write flow:
/// detect -> check password -> wipe -> verify wipe -> clone -> done
const T5577_TOTAL_STEPS: u16 = 6;

/// Total progress steps for the EM4305 write flow:
/// detect -> wipe -> verify wipe -> clone -> done
const EM4305_TOTAL_STEPS: u16 = 5;

/// Stub that returns an error directing callers to write_clone_with_data.
/// Kept registered so the frontend gets a clear message if it calls without params.
#[tauri::command]
pub async fn write_clone(
    _app: AppHandle,
    _machine: State<'_, Mutex<WizardMachine>>,
) -> Result<WizardState, AppError> {
    Err(AppError::CommandFailed(
        "write_clone is deprecated: use write_clone_with_data instead".into(),
    ))
}

/// Write clone with explicit parameters from the frontend.
/// This is the preferred entry point. Handles T5577 password safety and EM4305 blanks.
#[tauri::command]
pub async fn write_clone_with_data(
    app: AppHandle,
    port: String,
    card_type: CardType,
    uid: String,
    decoded: std::collections::HashMap<String, String>,
    blank_type: Option<BlankType>,
    machine: State<'_, Mutex<WizardMachine>>,
) -> Result<WizardState, AppError> {
    log::debug!("write_clone_with_data: port={}, card_type={:?}, uid={}, blank_type={:?}", port, card_type, uid, blank_type);

    // Guard: reject absurdly large decoded maps (prevents DoS via oversized IPC payload)
    if decoded.len() > 50 {
        return Err(AppError::CommandFailed(
            "Too many decoded fields".into(),
        ));
    }

    // Validate uid: must be non-empty alphanumeric with optional colons.
    // HID UIDs use format "FC65:CN29334" (contains non-hex letters like N).
    // Blocks semicolons, spaces, newlines — prevents command injection.
    if uid.is_empty() || !uid.chars().all(|c| c.is_ascii_alphanumeric() || c == ':') {
        return Err(AppError::CommandFailed(
            "Invalid UID: must contain only alphanumeric characters and colons".into(),
        ));
    }
    if uid.len() > 200 {
        return Err(AppError::CommandFailed("UID too long".into()));
    }

    // Validate port format
    if port.is_empty()
        || port.len() > 50
        || port.contains(';')
        || port.contains('\n')
        || port.contains('\r')
    {
        return Err(AppError::CommandFailed("Invalid port".into()));
    }

    let blank = blank_type.unwrap_or_else(|| card_type.recommended_blank());

    // Guard: reject EM4305 blank for card types that don't support the --em flag.
    // Only the original 11 LF types support EM4305. The newer types (Presco, Nedap,
    // GProxII, Gallagher, PAC, Noralsy, Jablotron, SecuraKey, Visa2000, Motorola,
    // IDTECK) will fail silently or error when --em is passed.
    if blank == BlankType::EM4305 && !card_type.supports_em4305() {
        return Err(AppError::CommandFailed(format!(
            "{} does not support EM4305 blanks. Please use a T5577 blank instead.",
            card_type.display_name()
        )));
    }

    // Transition: BlankDetected -> Writing
    {
        let mut m = machine.lock().map_err(|e| {
            AppError::CommandFailed(format!("State lock poisoned: {}", e))
        })?;
        m.transition(WizardAction::StartWrite)?;
    }

    // Branch based on blank type.
    // Errors from the write flow are caught and reported as FSM Error state
    // to keep the backend FSM in sync with the frontend XState machine.
    match blank {
        BlankType::T5577 => {
            match write_t5577_flow(&app, &port, &card_type, &uid, &decoded, &machine).await {
                Ok(state) => Ok(state),
                Err(e) => {
                    let err_detail = e.to_string();
                    log::warn!("T5577 flow error: {}", err_detail);
                    // Show the actual PM3 error to the user for debugging
                    let user_msg = format!(
                        "Write failed: {}",
                        err_detail.lines().last().unwrap_or("unknown error")
                    );
                    let _ = report_error(
                        &machine,
                        &err_detail,
                        &user_msg,
                        true,
                        Some(RecoveryAction::Retry),
                    );
                    let m = machine.lock().map_err(|e| {
                        AppError::CommandFailed(format!("State lock poisoned: {}", e))
                    })?;
                    Ok(m.current.clone())
                }
            }
        }
        BlankType::EM4305 => {
            match write_em4305_flow(&app, &port, &card_type, &uid, &decoded, &machine).await {
                Ok(state) => Ok(state),
                Err(e) => {
                    let err_detail = e.to_string();
                    let user_msg = format!(
                        "Write failed: {}",
                        err_detail.lines().last().unwrap_or("unknown error")
                    );
                    let _ = report_error(
                        &machine,
                        &err_detail,
                        &user_msg,
                        true,
                        Some(RecoveryAction::Retry),
                    );
                    let m = machine.lock().map_err(|e| {
                        AppError::CommandFailed(format!("State lock poisoned: {}", e))
                    })?;
                    Ok(m.current.clone())
                }
            }
        }
        _ => {
            // Other blank types not yet supported for LF
            let mut m = machine.lock().map_err(|e| {
                AppError::CommandFailed(format!("State lock poisoned: {}", e))
            })?;
            m.transition(WizardAction::ReportError {
                message: format!("Unsupported blank type {:?} for LF cloning", blank),
                user_message: "This blank type is not supported for LF card cloning.".to_string(),
                recoverable: false,
                recovery_action: None,
            })?;
            Ok(m.current.clone())
        }
    }
}

/// T5577 write flow:
/// - No password: detect -> clone (clone overwrites config + data blocks directly)
/// - Password: detect -> find password -> wipe -> verify wipe -> clone
async fn write_t5577_flow(
    app: &AppHandle,
    port: &str,
    card_type: &CardType,
    uid: &str,
    decoded: &std::collections::HashMap<String, String>,
    machine: &State<'_, Mutex<WizardMachine>>,
) -> Result<WizardState, AppError> {
    // Step 1: Detect T5577
    log::debug!("T5577 flow: Step 1 detect");
    update_progress(app, machine, 0.1, Some(0), Some(T5577_TOTAL_STEPS))?;

    let detect_out =
        connection::run_command(app, port, command_builder::build_t5577_detect()).await?;
    let t5577_status = output_parser::parse_t5577_detect(&detect_out);
    log::debug!("T5577 detect: detected={}, pw={}", t5577_status.detected, t5577_status.password_set);

    if !t5577_status.detected {
        return report_error(
            machine,
            "T5577 not detected on writer",
            "No T5577 blank found. Place blank card on the reader.",
            true,
            Some(RecoveryAction::Retry),
        );
    }

    // Step 2: Check for password protection
    update_progress(app, machine, 0.2, Some(1), Some(T5577_TOTAL_STEPS))?;

    let password: Option<String> = if t5577_status.password_set {
        // Password detected -- run chk to find it
        let chk_out = connection::run_command(app, port, command_builder::build_t5577_chk()).await;
        match chk_out {
            Ok(output) => {
                let found = output_parser::parse_t5577_chk(&output);
                if found.is_none() {
                    // Password set but could not be recovered
                    return report_error(
                        machine,
                        "Card is password-locked, cannot recover password",
                        "This T5577 is password-protected and the password could not be found. \
                         Use a different blank card.",
                        true,
                        Some(RecoveryAction::Retry),
                    );
                }
                found
            }
            Err(_) => {
                return report_error(
                    machine,
                    "Password check failed",
                    "Could not check T5577 password. Try again.",
                    true,
                    Some(RecoveryAction::Retry),
                );
            }
        }
    } else {
        None
    };

    // Step 3-4: Wipe + verify (ONLY when password-protected).
    // For clean T5577s the clone command overwrites config + data blocks directly.
    // Skipping wipe avoids an extra write cycle that can fail on weaker LF antennas
    // (PM3 Easy) and eliminates two subprocess spawns (fewer serial port open/close).
    if password.is_some() {
        update_progress(app, machine, 0.35, Some(2), Some(T5577_TOTAL_STEPS))?;

        let wipe_cmd =
            command_builder::build_wipe_command(&BlankType::T5577, password.as_deref())
                .ok_or_else(|| {
                    AppError::CommandFailed("No wipe command for this blank type".into())
                })?;
        connection::run_command(app, port, &wipe_cmd).await?;

        // Verify wipe — ensure T5577 is detected and no longer password-protected.
        // PM3 can return exit code 0 even when a password-protected wipe fails silently.
        update_progress(app, machine, 0.5, Some(3), Some(T5577_TOTAL_STEPS))?;

        let verify_wipe_out =
            connection::run_command(app, port, command_builder::build_t5577_detect()).await?;
        let verify_status = output_parser::parse_t5577_detect(&verify_wipe_out);

        if !verify_status.detected || verify_status.password_set {
            return report_error(
                machine,
                "T5577 wipe verification failed — card may still be password-protected",
                "Wipe verification failed. The card may still be password-protected. \
                 Do not remove the card — try again or use a different blank.",
                true,
                Some(RecoveryAction::Retry),
            );
        }
    }

    // SAFETY: If password was set, it was cleared by wipe above.
    // Shadow the variable to prevent accidental re-lock on clone command.
    let password: Option<String> = None;

    // Step 5: Clone
    update_progress(app, machine, 0.7, Some(4), Some(T5577_TOTAL_STEPS))?;

    log::debug!("Clone: uid={}, type={:?}, decoded={:?}", uid, card_type, decoded);

    let base_clone_cmd = command_builder::build_clone_command(card_type, uid, decoded);

    log::debug!("clone_cmd={:?}", base_clone_cmd);

    match base_clone_cmd {
        Some(cmd) => {
            let final_cmd = match &password {
                Some(pw) => command_builder::build_clone_with_password(&cmd, pw)
                    .map_err(|e| AppError::CommandFailed(format!("Password validation failed: {}", e)))?,
                None => cmd,
            };
            log::debug!("sending={}", final_cmd);
            let clone_output = connection::run_command(app, port, &final_cmd).await;
            log::debug!("clone_result={:?}", clone_output.as_ref().map(|s| s.chars().take(500).collect::<String>()).map_err(|e| e.to_string()));
            let clone_output = clone_output?;
            // Check for failure indicators in PM3 output
            if clone_output.contains("[!!]")
                || clone_output.to_lowercase().contains("fail")
            {
                return report_error(
                    machine,
                    &format!("Clone command may have failed: {}", clone_output.chars().take(200).collect::<String>()),
                    "Write may have failed. Do not remove the card — try again.",
                    true,
                    Some(RecoveryAction::Retry),
                );
            }
        }
        None => {
            return report_error(
                machine,
                &format!("No clone command for {:?}", card_type),
                "This card type cannot be cloned with the current method.",
                false,
                None,
            );
        }
    }

    // Step 6: Done writing -> Verifying transition
    update_progress(app, machine, 1.0, Some(5), Some(T5577_TOTAL_STEPS))?;
    {
        let mut m = machine.lock().map_err(|e| {
            AppError::CommandFailed(format!("State lock poisoned: {}", e))
        })?;
        m.transition(WizardAction::WriteFinished)?;
    }

    Ok({
        let m = machine.lock().map_err(|e| {
            AppError::CommandFailed(format!("State lock poisoned: {}", e))
        })?;
        m.current.clone()
    })
}

/// EM4305 write flow with detect + wipe-verify safety checks:
/// 1. detect EM4305 -> 2. wipe -> 3. verify wipe -> 4. clone with --em -> 5. done
async fn write_em4305_flow(
    app: &AppHandle,
    port: &str,
    card_type: &CardType,
    uid: &str,
    decoded: &std::collections::HashMap<String, String>,
    machine: &State<'_, Mutex<WizardMachine>>,
) -> Result<WizardState, AppError> {
    // Step 1: Detect EM4305 — verify the blank chip is present before wiping.
    // Mirrors the T5577 detect step to prevent wiping air / wrong chip.
    update_progress(app, machine, 0.1, Some(0), Some(EM4305_TOTAL_STEPS))?;

    let info_out =
        connection::run_command(app, port, command_builder::build_em4305_info()).await?;

    if !output_parser::parse_em4305_info(&info_out) {
        return report_error(
            machine,
            "EM4305 not detected on writer",
            "No EM4305 blank found. Place blank card on the reader.",
            true,
            Some(RecoveryAction::Retry),
        );
    }

    // Step 2: Wipe EM4305
    update_progress(app, machine, 0.3, Some(1), Some(EM4305_TOTAL_STEPS))?;

    connection::run_command(app, port, command_builder::build_em4305_wipe()).await?;

    // Step 3: Verify wipe — read word 0 and check it's zeroed.
    // PM3 can return exit code 0 even when wipe fails silently.
    // Proceeding to clone without this check risks corrupted data on the card.
    update_progress(app, machine, 0.5, Some(2), Some(EM4305_TOTAL_STEPS))?;

    let verify_out =
        connection::run_command(app, port, &command_builder::build_em4305_read_word(0)).await?;
    if let Some(word0) = output_parser::parse_em4305_word0(&verify_out) {
        if word0 != "00000000" {
            return report_error(
                machine,
                &format!(
                    "EM4305 wipe verification failed — word 0 is {} (expected 00000000)",
                    word0
                ),
                "Wipe verification failed. The card may not have been wiped correctly. \
                 Do not remove the card — try again or use a different blank.",
                true,
                Some(RecoveryAction::Retry),
            );
        }
    }
    // If parse_em4305_word0 returns None, we can't verify — proceed with caution.
    // This is acceptable: the clone step will fail if the card is in a bad state.

    // Step 4: Clone with --em flag
    update_progress(app, machine, 0.7, Some(3), Some(EM4305_TOTAL_STEPS))?;

    let base_clone_cmd = command_builder::build_clone_command(card_type, uid, decoded);
    match base_clone_cmd {
        Some(cmd) => {
            let em_cmd = command_builder::build_clone_for_em4305(&cmd);
            let clone_output = connection::run_command(app, port, &em_cmd).await?;
            // Check for failure indicators in PM3 output
            if clone_output.contains("[!!]")
                || clone_output.to_lowercase().contains("fail")
            {
                return report_error(
                    machine,
                    &format!("EM4305 clone may have failed: {}", clone_output.chars().take(200).collect::<String>()),
                    "Write may have failed. Do not remove the card — try again.",
                    true,
                    Some(RecoveryAction::Retry),
                );
            }
        }
        None => {
            return report_error(
                machine,
                &format!("No clone command for {:?}", card_type),
                "This card type cannot be cloned with the current method.",
                false,
                None,
            );
        }
    }

    // Step 5: Done -> Verifying
    update_progress(app, machine, 1.0, Some(4), Some(EM4305_TOTAL_STEPS))?;
    {
        let mut m = machine.lock().map_err(|e| {
            AppError::CommandFailed(format!("State lock poisoned: {}", e))
        })?;
        m.transition(WizardAction::WriteFinished)?;
    }

    Ok({
        let m = machine.lock().map_err(|e| {
            AppError::CommandFailed(format!("State lock poisoned: {}", e))
        })?;
        m.current.clone()
    })
}

/// Verify the clone by reading the written card and comparing fields.
/// Uses type-specific reader commands for more accurate verification.
///
/// `blank_type` is reserved for Phase 3 HF card verification where the blank
/// type determines the verification command. Currently unused for LF cards.
#[tauri::command]
pub async fn verify_clone(
    app: AppHandle,
    port: String,
    source_uid: String,
    source_card_type: CardType,
    source_decoded: Option<std::collections::HashMap<String, String>>,
    _blank_type: Option<BlankType>,
    machine: State<'_, Mutex<WizardMachine>>,
) -> Result<WizardState, AppError> {
    // Guard: must be in Verifying state before running any hardware commands.
    // Without this check, a call from the wrong state would waste a PM3
    // command before failing on the FSM transition.
    {
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
    }

    // Use generic `lf search` for verification — parse_lf_search is designed to parse
    // its output format. Type-specific readers (lf hid reader, etc.) produce different
    // output that parse_lf_search can't handle, causing false verification failures.
    let verify_output = connection::run_command(&app, &port, "lf search").await?;

    // Use detailed verification if decoded fields are available
    let (success, mismatched) = if let Some(ref decoded) = source_decoded {
        output_parser::verify_match_detailed(&source_card_type, decoded, &verify_output)
    } else {
        output_parser::verify_match(&source_uid, &verify_output)
    };

    let mut m = machine.lock().map_err(|e| {
        AppError::CommandFailed(format!("State lock poisoned: {}", e))
    })?;
    m.transition(WizardAction::VerificationResult {
        success,
        mismatched_blocks: mismatched.clone(),
    })?;

    // Note: VerificationComplete stores success/failure. The FINISH/MarkComplete
    // transition is guarded in both state.rs (line 272: `success: true` pattern match)
    // and wizardMachine.ts (guard: context.verifySuccess === true) to prevent
    // completing with failed verification. No additional guard needed here.
    if !success {
        log::warn!(
            "Verification failed: {} mismatched blocks",
            mismatched.len()
        );
    }

    Ok(m.current.clone())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn update_progress(
    app: &AppHandle,
    machine: &State<'_, Mutex<WizardMachine>>,
    progress: f32,
    current_block: Option<u16>,
    total_blocks: Option<u16>,
) -> Result<(), AppError> {
    let mut m = machine.lock().map_err(|e| {
        AppError::CommandFailed(format!("State lock poisoned: {}", e))
    })?;
    m.transition(WizardAction::UpdateWriteProgress {
        progress,
        current_block,
        total_blocks,
    })?;
    // Emit event to frontend for real-time progress updates
    if let Err(e) = app.emit(
        "write-progress",
        serde_json::json!({
            "progress": progress,
            "current_block": current_block,
            "total_blocks": total_blocks,
        }),
    ) {
        log::warn!("Failed to emit write-progress event: {}", e);
    }
    Ok(())
}

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
