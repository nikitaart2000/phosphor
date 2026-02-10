use std::sync::Mutex;
use tauri::State;

use crate::error::AppError;
use crate::state::{WizardAction, WizardMachine, WizardState};

#[tauri::command]
pub fn get_wizard_state(
    machine: State<'_, Mutex<WizardMachine>>,
) -> Result<WizardState, AppError> {
    let machine = machine.lock().map_err(|e| {
        AppError::CommandFailed(format!("State lock poisoned: {}", e))
    })?;
    Ok(machine.current.clone())
}

#[tauri::command]
pub fn wizard_action(
    action: WizardAction,
    machine: State<'_, Mutex<WizardMachine>>,
) -> Result<WizardState, AppError> {
    let mut machine = machine.lock().map_err(|e| {
        AppError::CommandFailed(format!("State lock poisoned: {}", e))
    })?;
    machine.transition(action)?;
    Ok(machine.current.clone())
}
