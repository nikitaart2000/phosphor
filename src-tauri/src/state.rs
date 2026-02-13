use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::cards::types::{
    BlankType, CardData, CardSummary, CardType, Frequency, ProcessPhase, RecoveryAction,
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
    HfProcessing {
        phase: ProcessPhase,
        keys_found: u32,
        keys_total: u32,
        elapsed_secs: u32,
    },
    HfDumpReady {
        dump_info: String,
    },
    WaitingForBlank {
        expected_blank: BlankType,
    },
    BlankDetected {
        blank_type: BlankType,
        ready_to_write: bool,
        existing_data_type: Option<String>,
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
    StartHfProcess,
    UpdateHfProgress {
        phase: ProcessPhase,
        keys_found: u32,
        keys_total: u32,
        elapsed_secs: u32,
    },
    HfProcessComplete {
        dump_info: String,
    },
    CancelHfProcess,
    ProceedToWrite {
        blank_type: BlankType,
    },
    BlankReady {
        blank_type: BlankType,
        existing_data_type: Option<String>,
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

fn state_name(s: &WizardState) -> &str {
    match s {
        WizardState::Idle => "Idle",
        WizardState::DetectingDevice => "DetectingDevice",
        WizardState::DeviceConnected { .. } => "DeviceConnected",
        WizardState::ScanningCard => "ScanningCard",
        WizardState::CardIdentified { .. } => "CardIdentified",
        WizardState::HfProcessing { .. } => "HfProcessing",
        WizardState::HfDumpReady { .. } => "HfDumpReady",
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
        WizardAction::StartHfProcess => "StartHfProcess",
        WizardAction::UpdateHfProgress { .. } => "UpdateHfProgress",
        WizardAction::HfProcessComplete { .. } => "HfProcessComplete",
        WizardAction::CancelHfProcess => "CancelHfProcess",
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
        WizardAction::BackToScan => "BackToScan",
        WizardAction::SoftReset => "SoftReset",
        WizardAction::Disconnect => "Disconnect",
        WizardAction::ReDetectBlank => "ReDetectBlank",
        WizardAction::LoadSavedCard { .. } => "LoadSavedCard",
    }
}

pub struct WizardMachine {
    pub current: WizardState,
    pub port: Option<String>,
    pub model: Option<String>,
    pub firmware: Option<String>,
}

impl WizardMachine {
    pub fn new() -> Self {
        WizardMachine {
            current: WizardState::Idle,
            port: None,
            model: None,
            firmware: None,
        }
    }

    pub fn transition(&mut self, action: WizardAction) -> Result<&WizardState, AppError> {
        // Reset is always valid from any state — full reset to idle
        if matches!(action, WizardAction::Reset) {
            self.current = WizardState::Idle;
            self.port = None;
            self.model = None;
            self.firmware = None;
            return Ok(&self.current);
        }

        // Disconnect is valid from any connected state — clears persistent fields
        if matches!(action, WizardAction::Disconnect) {
            self.current = WizardState::Idle;
            self.port = None;
            self.model = None;
            self.firmware = None;
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

            // DetectingDevice -> DeviceConnected (also stores persistent device info)
            (
                WizardState::DetectingDevice,
                WizardAction::DeviceFound {
                    port,
                    model,
                    firmware,
                },
            ) => {
                self.port = Some(port.clone());
                self.model = Some(model.clone());
                self.firmware = Some(firmware.clone());
                WizardState::DeviceConnected {
                    port: port.clone(),
                    model: model.clone(),
                    firmware: firmware.clone(),
                }
            }

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

            // CardIdentified -> HfProcessing (start key recovery)
            (WizardState::CardIdentified { .. }, WizardAction::StartHfProcess) => {
                WizardState::HfProcessing {
                    phase: ProcessPhase::KeyCheck,
                    keys_found: 0,
                    keys_total: 0,
                    elapsed_secs: 0,
                }
            }

            // HfProcessing -> HfProcessing (progress update)
            (
                WizardState::HfProcessing { .. },
                WizardAction::UpdateHfProgress {
                    phase,
                    keys_found,
                    keys_total,
                    elapsed_secs,
                },
            ) => WizardState::HfProcessing {
                phase: phase.clone(),
                keys_found: *keys_found,
                keys_total: *keys_total,
                elapsed_secs: *elapsed_secs,
            },

            // HfProcessing -> HfDumpReady (key recovery + dump complete)
            (
                WizardState::HfProcessing { .. },
                WizardAction::HfProcessComplete { dump_info },
            ) => WizardState::HfDumpReady {
                dump_info: dump_info.clone(),
            },

            // HfProcessing -> DeviceConnected (user cancelled)
            (WizardState::HfProcessing { .. }, WizardAction::CancelHfProcess) => {
                match (&self.port, &self.model, &self.firmware) {
                    (Some(p), Some(m), Some(f)) => WizardState::DeviceConnected {
                        port: p.clone(),
                        model: m.clone(),
                        firmware: f.clone(),
                    },
                    _ => {
                        return Err(AppError::InvalidTransition(
                            "CancelHfProcess requires persistent device info".to_string(),
                        ));
                    }
                }
            }

            // HfDumpReady -> WaitingForBlank (proceed to write)
            (
                WizardState::HfDumpReady { .. },
                WizardAction::ProceedToWrite { blank_type },
            ) => WizardState::WaitingForBlank {
                expected_blank: blank_type.clone(),
            },

            // HfDumpReady -> DeviceConnected (back to scan)
            (WizardState::HfDumpReady { .. }, WizardAction::BackToScan) => {
                match (&self.port, &self.model, &self.firmware) {
                    (Some(p), Some(m), Some(f)) => WizardState::DeviceConnected {
                        port: p.clone(),
                        model: m.clone(),
                        firmware: f.clone(),
                    },
                    _ => {
                        return Err(AppError::InvalidTransition(
                            "BackToScan requires persistent device info".to_string(),
                        ));
                    }
                }
            }

            // CardIdentified -> WaitingForBlank (LF cards skip HF processing)
            (
                WizardState::CardIdentified { .. },
                WizardAction::ProceedToWrite { blank_type },
            ) => WizardState::WaitingForBlank {
                expected_blank: blank_type.clone(),
            },

            // WaitingForBlank -> BlankDetected
            (WizardState::WaitingForBlank { .. }, WizardAction::BlankReady { blank_type, existing_data_type }) => {
                WizardState::BlankDetected {
                    blank_type: blank_type.clone(),
                    ready_to_write: true,
                    existing_data_type: existing_data_type.clone(),
                }
            }

            // BlankDetected -> WaitingForBlank (re-detect after erase)
            (WizardState::BlankDetected { .. }, WizardAction::ReDetectBlank) => {
                // Extract expected blank from current state
                WizardState::WaitingForBlank {
                    expected_blank: match &self.current {
                        WizardState::BlankDetected { blank_type, .. } => blank_type.clone(),
                        _ => unreachable!(),
                    },
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
                timestamp: chrono::Local::now().to_rfc3339(),
            },

            // Error + Retry -> Idle (user can restart the flow)
            (WizardState::Error { recoverable: true, .. }, WizardAction::Retry) => {
                WizardState::Idle
            }

            // Complete -> Idle (start over)
            (WizardState::Complete { .. }, WizardAction::StartDetection) => {
                WizardState::DetectingDevice
            }

            // BackToScan: post-scan states -> DeviceConnected using persistent device info
            (WizardState::CardIdentified { .. }, WizardAction::BackToScan)
            | (WizardState::WaitingForBlank { .. }, WizardAction::BackToScan)
            | (WizardState::HfProcessing { .. }, WizardAction::BackToScan) => {
                match (&self.port, &self.model, &self.firmware) {
                    (Some(p), Some(m), Some(f)) => WizardState::DeviceConnected {
                        port: p.clone(),
                        model: m.clone(),
                        firmware: f.clone(),
                    },
                    _ => {
                        return Err(AppError::InvalidTransition(
                            "BackToScan requires persistent device info".to_string(),
                        ));
                    }
                }
            }

            // SoftReset: Complete/Error -> DeviceConnected using persistent device info
            (WizardState::Complete { .. }, WizardAction::SoftReset)
            | (WizardState::Error { .. }, WizardAction::SoftReset) => {
                match (&self.port, &self.model, &self.firmware) {
                    (Some(p), Some(m), Some(f)) => WizardState::DeviceConnected {
                        port: p.clone(),
                        model: m.clone(),
                        firmware: f.clone(),
                    },
                    _ => {
                        return Err(AppError::InvalidTransition(
                            "SoftReset requires persistent device info".to_string(),
                        ));
                    }
                }
            }

            // LoadSavedCard: DeviceConnected -> CardIdentified with provided card data
            (
                WizardState::DeviceConnected { .. },
                WizardAction::LoadSavedCard {
                    frequency,
                    card_type,
                    uid,
                    raw,
                    decoded,
                    cloneable,
                    recommended_blank,
                },
            ) => WizardState::CardIdentified {
                frequency: frequency.clone(),
                card_type: card_type.clone(),
                card_data: CardData {
                    uid: uid.clone(),
                    raw: raw.clone(),
                    decoded: decoded.clone(),
                },
                cloneable: *cloneable,
                recommended_blank: recommended_blank.clone(),
            },

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
