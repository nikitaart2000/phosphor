use std::sync::Mutex;
use tauri::{AppHandle, State};

use crate::cards::types::{BlankType, RecoveryAction};
use crate::error::AppError;
use crate::pm3::{command_builder, connection, output_parser};
use crate::state::{WizardAction, WizardMachine, WizardState};

/// Detect whether a blank card (T5577 or EM4305) is present on the reader.
///
/// The FSM must be in `WaitingForBlank` state. On success, transitions to
/// `BlankDetected`; on failure, transitions to a recoverable `Error` with
/// `RecoveryAction::Retry` so the user can re-place the blank and try again.
///
/// The `port` parameter is supplied by the frontend from its XState context
/// (originally received during the `DeviceFound` event).
#[tauri::command]
pub async fn detect_blank(
    app: AppHandle,
    port: String,
    machine: State<'_, Mutex<WizardMachine>>,
) -> Result<WizardState, AppError> {
    // Validate we're in WaitingForBlank and extract expected blank type
    let expected_blank = {
        let m = machine.lock().map_err(|e| {
            AppError::CommandFailed(format!("State lock poisoned: {}", e))
        })?;
        match &m.current {
            WizardState::WaitingForBlank { expected_blank } => expected_blank.clone(),
            _ => {
                return Err(AppError::InvalidTransition(
                    "Must be in WaitingForBlank to detect blank".to_string(),
                ));
            }
        }
    };

    // Detect based on expected blank type
    match expected_blank {
        BlankType::T5577 => detect_t5577(&app, &port, &machine).await,
        BlankType::EM4305 => detect_em4305(&app, &port, &machine).await,
        // HF blank types — accept without hardware check (Phase 3 will add real detection)
        other => {
            let mut m = machine.lock().map_err(|e| {
                AppError::CommandFailed(format!("State lock poisoned: {}", e))
            })?;
            m.transition(WizardAction::BlankReady { blank_type: other })?;
            Ok(m.current.clone())
        }
    }
}

/// Run `lf t55xx detect` and parse the result to confirm a T5577 blank is present.
async fn detect_t5577(
    app: &AppHandle,
    port: &str,
    machine: &State<'_, Mutex<WizardMachine>>,
) -> Result<WizardState, AppError> {
    let output = connection::run_command(app, port, command_builder::build_t5577_detect()).await?;
    let status = output_parser::parse_t5577_detect(&output);

    if status.detected {
        let mut m = machine.lock().map_err(|e| {
            AppError::CommandFailed(format!("State lock poisoned: {}", e))
        })?;
        m.transition(WizardAction::BlankReady {
            blank_type: BlankType::T5577,
        })?;
        Ok(m.current.clone())
    } else {
        let mut m = machine.lock().map_err(|e| {
            AppError::CommandFailed(format!("State lock poisoned: {}", e))
        })?;
        m.transition(WizardAction::ReportError {
            message: "T5577 blank not detected".to_string(),
            user_message: "No T5577 blank found. Place blank card on the reader and try again."
                .to_string(),
            recoverable: true,
            recovery_action: Some(RecoveryAction::Retry),
        })?;
        Ok(m.current.clone())
    }
}

/// Detect an EM4305 blank by running `lf em 4x05 info` and parsing the output.
/// Checks for EM4x05-specific strings in the output to confirm the chip is present,
/// rather than relying solely on the exit code.
async fn detect_em4305(
    app: &AppHandle,
    port: &str,
    machine: &State<'_, Mutex<WizardMachine>>,
) -> Result<WizardState, AppError> {
    let result = connection::run_command(app, port, "lf em 4x05 info").await;

    let detected = match &result {
        Ok(output) => {
            let clean = output.to_lowercase();
            // Confirm EM4x05/EM4305 chip by checking for known markers in the output.
            // Proxmark3 `lf em 4x05 info` outputs lines like:
            //   "EM4x05/EM4x69"  or  "Chip Type:   EM4305"  or  "UID:"
            clean.contains("em4x05")
                || clean.contains("em4x69")
                || clean.contains("em4305")
                || clean.contains("em4469")
                || (clean.contains("chip") && clean.contains("uid"))
        }
        Err(_) => false,
    };

    if detected {
        let mut m = machine.lock().map_err(|e| {
            AppError::CommandFailed(format!("State lock poisoned: {}", e))
        })?;
        m.transition(WizardAction::BlankReady {
            blank_type: BlankType::EM4305,
        })?;
        Ok(m.current.clone())
    } else {
        let mut m = machine.lock().map_err(|e| {
            AppError::CommandFailed(format!("State lock poisoned: {}", e))
        })?;
        m.transition(WizardAction::ReportError {
            message: "EM4305 blank not detected".to_string(),
            user_message:
                "No EM4305 blank found. Place blank card on the reader and try again."
                    .to_string(),
            recoverable: true,
            recovery_action: Some(RecoveryAction::Retry),
        })?;
        Ok(m.current.clone())
    }
}
