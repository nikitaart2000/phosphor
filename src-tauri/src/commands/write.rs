use std::sync::Mutex;
use tauri::{AppHandle, State};

use crate::cards::types::{CardSummary, CardType, RecoveryAction};
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
        "Use write_clone_with_data and pass card_type, uid, decoded, port".to_string(),
    ))
}

/// Write clone with explicit parameters from the frontend.
/// This is the preferred entry point.
#[tauri::command]
pub async fn write_clone_with_data(
    app: AppHandle,
    port: String,
    card_type: CardType,
    uid: String,
    decoded: std::collections::HashMap<String, String>,
    machine: State<'_, Mutex<WizardMachine>>,
) -> Result<WizardState, AppError> {
    // Transition: BlankDetected -> Writing
    {
        let mut m = machine.lock().map_err(|e| {
            AppError::CommandFailed(format!("State lock poisoned: {}", e))
        })?;
        m.transition(WizardAction::StartWrite)?;
    }

    // Step 1: Detect T5577
    {
        let mut m = machine.lock().map_err(|e| {
            AppError::CommandFailed(format!("State lock poisoned: {}", e))
        })?;
        m.transition(WizardAction::UpdateWriteProgress {
            progress: 0.1,
            current_block: Some(0),
            total_blocks: Some(4),
        })?;
    }

    let detect_out =
        connection::run_command(&app, &port, command_builder::build_t5577_detect()).await?;
    if !output_parser::is_t5577_detected(&detect_out) {
        let mut m = machine.lock().map_err(|e| {
            AppError::CommandFailed(format!("State lock poisoned: {}", e))
        })?;
        m.transition(WizardAction::ReportError {
            message: "T5577 not detected on writer".to_string(),
            user_message: "No T5577 blank found. Place blank card on the reader.".to_string(),
            recoverable: true,
            recovery_action: Some(RecoveryAction::Retry),
        })?;
        return Ok(m.current.clone());
    }

    // Step 2: Safety check
    {
        let mut m = machine.lock().map_err(|e| {
            AppError::CommandFailed(format!("State lock poisoned: {}", e))
        })?;
        m.transition(WizardAction::UpdateWriteProgress {
            progress: 0.25,
            current_block: Some(1),
            total_blocks: Some(4),
        })?;
    }
    // t55xx chk -- detect password protection. If it fails, card may be password-locked.
    let _chk_out = connection::run_command(&app, &port, command_builder::build_t5577_chk()).await;
    // We proceed regardless; wipe will fail if password-locked.

    // Step 3: Wipe
    {
        let mut m = machine.lock().map_err(|e| {
            AppError::CommandFailed(format!("State lock poisoned: {}", e))
        })?;
        m.transition(WizardAction::UpdateWriteProgress {
            progress: 0.5,
            current_block: Some(2),
            total_blocks: Some(4),
        })?;
    }
    connection::run_command(&app, &port, command_builder::build_t5577_wipe()).await?;

    // Step 4: Clone
    {
        let mut m = machine.lock().map_err(|e| {
            AppError::CommandFailed(format!("State lock poisoned: {}", e))
        })?;
        m.transition(WizardAction::UpdateWriteProgress {
            progress: 0.75,
            current_block: Some(3),
            total_blocks: Some(4),
        })?;
    }

    let clone_cmd = command_builder::build_clone_command(&card_type, &uid, &decoded);
    match clone_cmd {
        Some(cmd) => {
            connection::run_command(&app, &port, &cmd).await?;
        }
        None => {
            let mut m = machine.lock().map_err(|e| {
                AppError::CommandFailed(format!("State lock poisoned: {}", e))
            })?;
            m.transition(WizardAction::ReportError {
                message: format!("No clone command for {:?}", card_type),
                user_message: "This card type cannot be cloned with the current method.".to_string(),
                recoverable: false,
                recovery_action: None,
            })?;
            return Ok(m.current.clone());
        }
    }

    // Done writing -> Verifying transition
    {
        let mut m = machine.lock().map_err(|e| {
            AppError::CommandFailed(format!("State lock poisoned: {}", e))
        })?;
        m.transition(WizardAction::UpdateWriteProgress {
            progress: 1.0,
            current_block: Some(4),
            total_blocks: Some(4),
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

/// Verify the clone by reading the T5577 and comparing UIDs.
#[tauri::command]
pub async fn verify_clone(
    app: AppHandle,
    port: String,
    source_uid: String,
    source_card_type: String,
    machine: State<'_, Mutex<WizardMachine>>,
) -> Result<WizardState, AppError> {
    // Should be in Verifying state
    let verify_output =
        connection::run_command(&app, &port, command_builder::build_lf_search()).await?;

    let (success, mismatched) = output_parser::verify_match(&source_uid, &verify_output);

    let mut m = machine.lock().map_err(|e| {
        AppError::CommandFailed(format!("State lock poisoned: {}", e))
    })?;
    m.transition(WizardAction::VerificationResult {
        success,
        mismatched_blocks: mismatched.clone(),
    })?;

    if success {
        let target_uid = if let Some((_, cd)) = output_parser::parse_lf_search(&verify_output) {
            cd.uid
        } else {
            source_uid.clone()
        };

        m.transition(WizardAction::MarkComplete {
            source: CardSummary {
                card_type: source_card_type.clone(),
                uid: source_uid,
                display_name: format!("{} clone source", source_card_type),
            },
            target: CardSummary {
                card_type: "T5577".to_string(),
                uid: target_uid,
                display_name: "T5577 clone".to_string(),
            },
        })?;
    }

    Ok(m.current.clone())
}
