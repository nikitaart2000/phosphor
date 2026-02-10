use std::sync::Mutex;
use tauri::{AppHandle, State};

use crate::cards::types::RecoveryAction;
use crate::error::AppError;
use crate::pm3::{command_builder, connection, output_parser};
use crate::state::{WizardAction, WizardMachine, WizardState};

#[tauri::command]
pub async fn scan_card(
    app: AppHandle,
    machine: State<'_, Mutex<WizardMachine>>,
) -> Result<WizardState, AppError> {
    // Get the port from current state, then transition to ScanningCard
    let port = {
        let mut m = machine.lock().map_err(|e| {
            AppError::CommandFailed(format!("State lock poisoned: {}", e))
        })?;
        let port = match &m.current {
            WizardState::DeviceConnected { port, .. } => port.clone(),
            _ => {
                return Err(AppError::InvalidTransition(
                    "Must be in DeviceConnected to scan".to_string(),
                ));
            }
        };
        m.transition(WizardAction::StartScan)?;
        port
    };

    // Run LF search
    let lf_output =
        connection::run_command(&app, &port, command_builder::build_lf_search()).await;

    match lf_output {
        Ok(output) => {
            if let Some((card_type, card_data)) = output_parser::parse_lf_search(&output) {
                let frequency = card_type.frequency();
                let cloneable = card_type.is_cloneable();
                let recommended_blank = card_type.recommended_blank();

                let mut m = machine.lock().map_err(|e| {
                    AppError::CommandFailed(format!("State lock poisoned: {}", e))
                })?;
                m.transition(WizardAction::CardFound {
                    frequency,
                    card_type,
                    card_data,
                    cloneable,
                    recommended_blank,
                })?;
                Ok(m.current.clone())
            } else {
                let mut m = machine.lock().map_err(|e| {
                    AppError::CommandFailed(format!("State lock poisoned: {}", e))
                })?;
                m.transition(WizardAction::ReportError {
                    message: "No LF card detected".to_string(),
                    user_message: "No card found. Place the card on the reader and try again."
                        .to_string(),
                    recoverable: true,
                    recovery_action: Some(RecoveryAction::Retry),
                })?;
                Ok(m.current.clone())
            }
        }
        Err(e) => {
            let mut m = machine.lock().map_err(|e| {
                AppError::CommandFailed(format!("State lock poisoned: {}", e))
            })?;
            m.transition(WizardAction::ReportError {
                message: e.to_string(),
                user_message: "Scan failed. Check device connection.".to_string(),
                recoverable: true,
                recovery_action: Some(RecoveryAction::Reconnect),
            })?;
            Ok(m.current.clone())
        }
    }
}
