use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::cards::types::CardSummary;
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

/// Actions that can be triggered directly by the frontend.
/// Internal-only actions (DeviceFound, CardFound, WriteFinished,
/// VerificationResult, UpdateWriteProgress, BlankReady, ReportError)
/// are restricted to backend-initiated transitions only.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "action", content = "payload")]
pub enum UserAction {
    Reset,
    Retry,
    ProceedToWrite,
    StartDetection,
    StartScan,
    StartWrite,
    MarkComplete {
        source: CardSummary,
        target: CardSummary,
    },
}

impl UserAction {
    fn into_wizard_action(self) -> WizardAction {
        match self {
            UserAction::Reset => WizardAction::Reset,
            UserAction::Retry => WizardAction::Retry,
            UserAction::ProceedToWrite => WizardAction::ProceedToWrite,
            UserAction::StartDetection => WizardAction::StartDetection,
            UserAction::StartScan => WizardAction::StartScan,
            UserAction::StartWrite => WizardAction::StartWrite,
            UserAction::MarkComplete { source, target } => {
                WizardAction::MarkComplete { source, target }
            }
        }
    }
}

#[tauri::command]
pub fn wizard_action(
    action: UserAction,
    machine: State<'_, Mutex<WizardMachine>>,
) -> Result<WizardState, AppError> {
    let mut machine = machine.lock().map_err(|e| {
        AppError::CommandFailed(format!("State lock poisoned: {}", e))
    })?;
    machine.transition(action.into_wizard_action())?;
    Ok(machine.current.clone())
}
