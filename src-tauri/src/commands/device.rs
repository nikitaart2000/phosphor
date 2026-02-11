use std::sync::Mutex;
use tauri::{AppHandle, State};

use crate::error::AppError;
use crate::pm3::connection;
use crate::state::{WizardAction, WizardMachine, WizardState};

#[tauri::command]
pub async fn detect_device(
    app: AppHandle,
    machine: State<'_, Mutex<WizardMachine>>,
) -> Result<WizardState, AppError> {
    // Transition to DetectingDevice
    {
        let mut m = machine.lock().map_err(|e| {
            AppError::CommandFailed(format!("State lock poisoned: {}", e))
        })?;
        m.transition(WizardAction::StartDetection)?;
    }

    match connection::detect_device(&app).await {
        Ok((port, model, firmware)) => {
            let mut m = machine.lock().map_err(|e| {
                AppError::CommandFailed(format!("State lock poisoned: {}", e))
            })?;
            m.transition(WizardAction::DeviceFound {
                port,
                model,
                firmware,
            })?;
            Ok(m.current.clone())
        }
        Err(e) => {
            let err_msg = e.to_string();
            let user_message = if err_msg.contains("spawn")
                || err_msg.contains("not found")
                || err_msg.contains("No such file")
                || err_msg.contains("program not found")
            {
                "Proxmark3 binary not found. Ensure proxmark3 is installed and in your PATH."
                    .to_string()
            } else {
                "No Proxmark3 device found. Check your USB connection.".to_string()
            };
            let mut m = machine.lock().map_err(|e| {
                AppError::CommandFailed(format!("State lock poisoned: {}", e))
            })?;
            m.transition(WizardAction::ReportError {
                message: err_msg,
                user_message,
                recoverable: true,
                recovery_action: Some(crate::cards::types::RecoveryAction::Retry),
            })?;
            Ok(m.current.clone())
        }
    }
}
