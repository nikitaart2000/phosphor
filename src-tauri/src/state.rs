use serde::{Deserialize, Serialize};

use crate::cards::types::{
    BlankType, CardData, CardSummary, CardType, Frequency, RecoveryAction,
};
use crate::error::AppError;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "step", content = "data")]
pub enum WizardState {
    Idle,
    DetectingDevice,
    DeviceConnected {
        port: String,
        model: String,
        firmware: String,
    },
    ScanningCard,
    CardIdentified {
        frequency: Frequency,
        card_type: CardType,
        card_data: CardData,
        cloneable: bool,
        recommended_blank: BlankType,
    },
    WaitingForBlank {
        expected_blank: BlankType,
    },
    BlankDetected {
        blank_type: BlankType,
        ready_to_write: bool,
    },
    Writing {
        progress: f32,
        current_block: Option<u16>,
        total_blocks: Option<u16>,
    },
    Verifying,
    VerificationComplete {
        success: bool,
        mismatched_blocks: Vec<u16>,
    },
    Complete {
        source: CardSummary,
        target: CardSummary,
        timestamp: String,
    },
    Error {
        message: String,
        user_message: String,
        recoverable: bool,
        recovery_action: Option<RecoveryAction>,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "action", content = "payload")]
pub enum WizardAction {
    StartDetection,
    DeviceFound {
        port: String,
        model: String,
        firmware: String,
    },
    StartScan,
    CardFound {
        frequency: Frequency,
        card_type: CardType,
        card_data: CardData,
        cloneable: bool,
        recommended_blank: BlankType,
    },
    ProceedToWrite {
        blank_type: BlankType,
    },
    BlankReady {
        blank_type: BlankType,
    },
    StartWrite,
    UpdateWriteProgress {
        progress: f32,
        current_block: Option<u16>,
        total_blocks: Option<u16>,
    },
    WriteFinished,
    VerificationResult {
        success: bool,
        mismatched_blocks: Vec<u16>,
    },
    MarkComplete {
        source: CardSummary,
        target: CardSummary,
    },
    ReportError {
        message: String,
        user_message: String,
        recoverable: bool,
        recovery_action: Option<RecoveryAction>,
    },
    Retry,
    Reset,
}

fn state_name(s: &WizardState) -> &str {
    match s {
        WizardState::Idle => "Idle",
        WizardState::DetectingDevice => "DetectingDevice",
        WizardState::DeviceConnected { .. } => "DeviceConnected",
        WizardState::ScanningCard => "ScanningCard",
        WizardState::CardIdentified { .. } => "CardIdentified",
        WizardState::WaitingForBlank { .. } => "WaitingForBlank",
        WizardState::BlankDetected { .. } => "BlankDetected",
        WizardState::Writing { .. } => "Writing",
        WizardState::Verifying => "Verifying",
        WizardState::VerificationComplete { .. } => "VerificationComplete",
        WizardState::Complete { .. } => "Complete",
        WizardState::Error { .. } => "Error",
    }
}

fn action_name(a: &WizardAction) -> &str {
    match a {
        WizardAction::StartDetection => "StartDetection",
        WizardAction::DeviceFound { .. } => "DeviceFound",
        WizardAction::StartScan => "StartScan",
        WizardAction::CardFound { .. } => "CardFound",
        WizardAction::ProceedToWrite { .. } => "ProceedToWrite",
        WizardAction::BlankReady { .. } => "BlankReady",
        WizardAction::StartWrite => "StartWrite",
        WizardAction::UpdateWriteProgress { .. } => "UpdateWriteProgress",
        WizardAction::WriteFinished => "WriteFinished",
        WizardAction::VerificationResult { .. } => "VerificationResult",
        WizardAction::MarkComplete { .. } => "MarkComplete",
        WizardAction::ReportError { .. } => "ReportError",
        WizardAction::Retry => "Retry",
        WizardAction::Reset => "Reset",
    }
}

pub struct WizardMachine {
    pub current: WizardState,
}

impl WizardMachine {
    pub fn new() -> Self {
        WizardMachine {
            current: WizardState::Idle,
        }
    }

    pub fn transition(&mut self, action: WizardAction) -> Result<&WizardState, AppError> {
        // Reset is always valid from any state
        if matches!(action, WizardAction::Reset) {
            self.current = WizardState::Idle;
            return Ok(&self.current);
        }

        // ReportError is always valid from any state
        if let WizardAction::ReportError {
            message,
            user_message,
            recoverable,
            recovery_action,
        } = &action
        {
            self.current = WizardState::Error {
                message: message.clone(),
                user_message: user_message.clone(),
                recoverable: *recoverable,
                recovery_action: recovery_action.clone(),
            };
            return Ok(&self.current);
        }

        let next = match (&self.current, &action) {
            // Idle -> DetectingDevice
            (WizardState::Idle, WizardAction::StartDetection) => WizardState::DetectingDevice,

            // DetectingDevice -> DeviceConnected
            (
                WizardState::DetectingDevice,
                WizardAction::DeviceFound {
                    port,
                    model,
                    firmware,
                },
            ) => WizardState::DeviceConnected {
                port: port.clone(),
                model: model.clone(),
                firmware: firmware.clone(),
            },

            // DeviceConnected -> ScanningCard
            (WizardState::DeviceConnected { .. }, WizardAction::StartScan) => {
                WizardState::ScanningCard
            }

            // ScanningCard -> CardIdentified
            (
                WizardState::ScanningCard,
                WizardAction::CardFound {
                    frequency,
                    card_type,
                    card_data,
                    cloneable,
                    recommended_blank,
                },
            ) => WizardState::CardIdentified {
                frequency: frequency.clone(),
                card_type: card_type.clone(),
                card_data: card_data.clone(),
                cloneable: *cloneable,
                recommended_blank: recommended_blank.clone(),
            },

            // CardIdentified -> WaitingForBlank
            (
                WizardState::CardIdentified { .. },
                WizardAction::ProceedToWrite { blank_type },
            ) => WizardState::WaitingForBlank {
                expected_blank: blank_type.clone(),
            },

            // WaitingForBlank -> BlankDetected
            (WizardState::WaitingForBlank { .. }, WizardAction::BlankReady { blank_type }) => {
                WizardState::BlankDetected {
                    blank_type: blank_type.clone(),
                    ready_to_write: true,
                }
            }

            // BlankDetected -> Writing
            (WizardState::BlankDetected { .. }, WizardAction::StartWrite) => {
                WizardState::Writing {
                    progress: 0.0,
                    current_block: None,
                    total_blocks: None,
                }
            }

            // Writing -> Writing (progress update)
            (
                WizardState::Writing { .. },
                WizardAction::UpdateWriteProgress {
                    progress,
                    current_block,
                    total_blocks,
                },
            ) => WizardState::Writing {
                progress: *progress,
                current_block: *current_block,
                total_blocks: *total_blocks,
            },

            // Writing -> Verifying
            (WizardState::Writing { .. }, WizardAction::WriteFinished) => WizardState::Verifying,

            // Verifying -> VerificationComplete
            (
                WizardState::Verifying,
                WizardAction::VerificationResult {
                    success,
                    mismatched_blocks,
                },
            ) => WizardState::VerificationComplete {
                success: *success,
                mismatched_blocks: mismatched_blocks.clone(),
            },

            // VerificationComplete -> Complete
            (
                WizardState::VerificationComplete { success: true, .. },
                WizardAction::MarkComplete { source, target },
            ) => WizardState::Complete {
                source: source.clone(),
                target: target.clone(),
                timestamp: chrono::Utc::now().to_rfc3339(),
            },

            // Error + Retry -> Idle (user can restart the flow)
            (WizardState::Error { recoverable: true, .. }, WizardAction::Retry) => {
                WizardState::Idle
            }

            // Complete -> Idle (start over)
            (WizardState::Complete { .. }, WizardAction::StartDetection) => {
                WizardState::DetectingDevice
            }

            _ => {
                return Err(AppError::InvalidTransition(format!(
                    "{} is not valid from {}",
                    action_name(&action),
                    state_name(&self.current)
                )));
            }
        };

        self.current = next;
        Ok(&self.current)
    }
}
