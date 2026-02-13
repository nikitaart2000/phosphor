use std::collections::HashMap;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::cards::types::{BlankType, CardSummary, CardType, Frequency};
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
    ProceedToWrite {
        blank_type: BlankType,
    },
    StartDetection,
    StartScan,
    StartWrite,
    MarkComplete {
        source: CardSummary,
        target: CardSummary,
    },
    BackToScan,
    SoftReset,
    Disconnect,
    ReDetectBlank,
    LoadSavedCard {
        frequency: Frequency,
        card_type: CardType,
        uid: String,
        raw: String,
        decoded: HashMap<String, String>,
        cloneable: bool,
        recommended_blank: BlankType,
    },
}

impl UserAction {
    fn into_wizard_action(self) -> WizardAction {
        match self {
            UserAction::Reset => WizardAction::Reset,
            UserAction::Retry => WizardAction::Retry,
            UserAction::ProceedToWrite { blank_type } => {
                WizardAction::ProceedToWrite { blank_type }
            }
            UserAction::StartDetection => WizardAction::StartDetection,
            UserAction::StartScan => WizardAction::StartScan,
            UserAction::StartWrite => WizardAction::StartWrite,
            UserAction::MarkComplete { source, target } => {
                WizardAction::MarkComplete { source, target }
            }
            UserAction::BackToScan => WizardAction::BackToScan,
            UserAction::SoftReset => WizardAction::SoftReset,
            UserAction::Disconnect => WizardAction::Disconnect,
            UserAction::ReDetectBlank => WizardAction::ReDetectBlank,
            UserAction::LoadSavedCard {
                frequency,
                card_type,
                uid,
                raw,
                decoded,
                cloneable,
                recommended_blank,
            } => WizardAction::LoadSavedCard {
                frequency,
                card_type,
                uid,
                raw,
                decoded,
                cloneable,
                recommended_blank,
            },
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
