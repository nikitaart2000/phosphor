use std::sync::Mutex;
use tauri::{AppHandle, Emitter, State};

use crate::cards::types::{BlankType, CardType, RecoveryAction};
use crate::error::AppError;
use crate::pm3::{command_builder, connection, output_parser};
use crate::state::{WizardAction, WizardMachine, WizardState};

/// Stub that returns an error directing callers to write_clone_with_data.
/// Kept registered so the frontend gets a clear message if it calls without params.
#[tauri::command]
pub async fn write_clone(
    _app: AppHandle,
    machine: State<'_, Mutex<WizardMachine>>,
) -> Result<WizardState, AppError> {
    let m = machine.lock().map_err(|e| {
        AppError::CommandFailed(format!("State lock poisoned: {}", e))
    })?;
    match &m.current {
        WizardState::BlankDetected { .. } => {}
        _ => {
            return Err(AppError::InvalidTransition(
                "Must be in BlankDetected to write".to_string(),
            ));
        }
    }
    Err(AppError::CommandFailed(
        "Use write_clone_with_data and pass card_type, uid, decoded, port, blank_type".to_string(),
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
    // Guard: reject absurdly large decoded maps (prevents DoS via oversized IPC payload)
    if decoded.len() > 50 {
        return Err(AppError::CommandFailed(
            "Too many decoded fields".into(),
        ));
    }

    let blank = blank_type.unwrap_or_else(|| card_type.recommended_blank());

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
                    let _ = report_error(
                        &machine,
                        &e.to_string(),
                        "Write operation failed. Do not remove the card.",
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
                    let _ = report_error(
                        &machine,
                        &e.to_string(),
                        "Write operation failed. Do not remove the card.",
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

/// T5577 write flow with password safety:
/// 1. detect -> 2. check password -> 3. wipe (with password if needed) -> 4. verify wipe -> 5. clone
async fn write_t5577_flow(
    app: &AppHandle,
    port: &str,
    card_type: &CardType,
    uid: &str,
    decoded: &std::collections::HashMap<String, String>,
    machine: &State<'_, Mutex<WizardMachine>>,
) -> Result<WizardState, AppError> {
    // Step 1: Detect T5577
    update_progress(app, machine, 0.1, Some(0), Some(6))?;

    let detect_out =
        connection::run_command(app, port, command_builder::build_t5577_detect()).await?;
    let t5577_status = output_parser::parse_t5577_detect(&detect_out);

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
    update_progress(app, machine, 0.2, Some(1), Some(6))?;

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

    // Step 3: Wipe (with password if needed)
    update_progress(app, machine, 0.35, Some(2), Some(6))?;

    let wipe_cmd = command_builder::build_wipe_command(&BlankType::T5577, password.as_deref());
    connection::run_command(app, port, &wipe_cmd).await?;

    // Step 4: Verify wipe — ensure T5577 is detected and no longer password-protected.
    // PM3 can return exit code 0 even when a password-protected wipe fails silently.
    // Proceeding to clone without this check risks soft-bricking the card.
    update_progress(app, machine, 0.5, Some(3), Some(6))?;

    let verify_wipe_out =
        connection::run_command(app, port, command_builder::build_t5577_detect()).await?;
    let verify_status = output_parser::parse_t5577_detect(&verify_wipe_out);

    if !verify_status.detected || verify_status.password_set {
        return report_error(
            machine,
            "T5577 wipe verification failed — card may still be password-protected. Do not proceed with cloning.",
            "Wipe verification failed. The card may still be password-protected. \
             Do not remove the card — try again or use a different blank.",
            true,
            Some(RecoveryAction::Retry),
        );
    }

    // Step 5: Clone (with password if needed)
    update_progress(app, machine, 0.7, Some(4), Some(6))?;

    let base_clone_cmd = command_builder::build_clone_command(card_type, uid, decoded);
    match base_clone_cmd {
        Some(cmd) => {
            let final_cmd = match &password {
                Some(pw) => command_builder::build_clone_with_password(&cmd, pw),
                None => cmd,
            };
            connection::run_command(app, port, &final_cmd).await?;
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
    update_progress(app, machine, 1.0, Some(5), Some(6))?;
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

/// EM4305 write flow — skips T5577-specific steps (detect/chk/wipe).
/// Uses `lf em 4x05 wipe` and appends `--em` flag to clone.
async fn write_em4305_flow(
    app: &AppHandle,
    port: &str,
    card_type: &CardType,
    uid: &str,
    decoded: &std::collections::HashMap<String, String>,
    machine: &State<'_, Mutex<WizardMachine>>,
) -> Result<WizardState, AppError> {
    // Step 1: Wipe EM4305
    update_progress(app, machine, 0.2, Some(0), Some(3))?;

    connection::run_command(app, port, command_builder::build_em4305_wipe()).await?;

    // Step 2: Clone with --em flag
    update_progress(app, machine, 0.5, Some(1), Some(3))?;

    let base_clone_cmd = command_builder::build_clone_command(card_type, uid, decoded);
    match base_clone_cmd {
        Some(cmd) => {
            let em_cmd = command_builder::build_clone_for_em4305(&cmd);
            connection::run_command(app, port, &em_cmd).await?;
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

    // Step 3: Done -> Verifying
    update_progress(app, machine, 1.0, Some(2), Some(3))?;
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
    // Use type-specific reader command instead of generic lf search
    let verify_cmd = command_builder::build_verify_command(&source_card_type);
    let verify_output = connection::run_command(&app, &port, verify_cmd).await?;

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

    // Return VerificationComplete state to the frontend.
    // The frontend handles FINISH -> MarkComplete via wizard_action.
    // Previously this auto-advanced to Complete, causing the frontend
    // XState guard (which checks for VerificationComplete) to fail.
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
        eprintln!("Failed to emit write-progress event: {}", e);
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
