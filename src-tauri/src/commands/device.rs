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
            let mut m = machine.lock().map_err(|e| {
                AppError::CommandFailed(format!("State lock poisoned: {}", e))
            })?;
            m.transition(WizardAction::ReportError {
                message: e.to_string(),
                user_message: "No Proxmark3 device found. Check USB connection and drivers."
                    .to_string(),
                recoverable: true,
                recovery_action: Some(crate::cards::types::RecoveryAction::Retry),
            })?;
            Ok(m.current.clone())
        }
    }
}
