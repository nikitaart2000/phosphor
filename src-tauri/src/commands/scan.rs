use std::sync::Mutex;
use tauri::{AppHandle, State};

use crate::cards::types::{CardType, RecoveryAction};
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

    // 1. Try LF search first (fast path for 125 kHz cards)
    let lf_result =
        connection::run_command(&app, &port, command_builder::build_lf_search()).await;

    if let Ok(ref output) = lf_result {
        if let Some((card_type, card_data)) = output_parser::parse_lf_search(output) {
            return finish_scan(&machine, card_type, card_data);
        }
    }

    // 2. LF found nothing → try HF search (13.56 MHz)
    let hf_result =
        connection::run_command(&app, &port, command_builder::build_hf_search()).await;

    match hf_result {
        Ok(output) => {
            if let Some((card_type, mut card_data)) = output_parser::parse_hf_search(&output)
            {
                // Enrich HF data with protocol-specific info commands
                enrich_hf_data(&app, &port, &card_type, &mut card_data).await;
                return finish_scan(&machine, card_type, card_data);
            }

            // Neither LF nor HF found a card
            let mut m = machine.lock().map_err(|e| {
                AppError::CommandFailed(format!("State lock poisoned: {}", e))
            })?;
            m.transition(WizardAction::ReportError {
                message: "No card detected".to_string(),
                user_message: "No card found. Place the card on the reader and try again."
                    .to_string(),
                recoverable: true,
                recovery_action: Some(RecoveryAction::Retry),
            })?;
            Ok(m.current.clone())
        }
        Err(_) => {
            // HF search also failed — check if LF had a connection error
            if let Err(e) = lf_result {
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
            } else {
                let mut m = machine.lock().map_err(|e| {
                    AppError::CommandFailed(format!("State lock poisoned: {}", e))
                })?;
                m.transition(WizardAction::ReportError {
                    message: "No card detected".to_string(),
                    user_message:
                        "No card found. Place the card on the reader and try again."
                            .to_string(),
                    recoverable: true,
                    recovery_action: Some(RecoveryAction::Retry),
                })?;
                Ok(m.current.clone())
            }
        }
    }
}

/// Enrich HF card data with protocol-specific info commands.
/// For MIFARE Classic: `hf 14a info` (PRNG) + `hf mf info` (magic detection).
/// For UL/NTAG: `hf mfu info` for subtype detection.
async fn enrich_hf_data(
    app: &AppHandle,
    port: &str,
    card_type: &CardType,
    card_data: &mut crate::cards::types::CardData,
) {
    match card_type {
        CardType::MifareClassic1K | CardType::MifareClassic4K => {
            // Get PRNG info if not already present
            if !card_data.decoded.contains_key("prng") {
                if let Ok(info_output) =
                    connection::run_command(app, port, command_builder::build_hf_14a_info())
                        .await
                {
                    let clean = output_parser::strip_ansi(&info_output);
                    if let Some(caps) =
                        regex::Regex::new(r"(?i)Prng\s+detection[\s.:]+(WEAK|HARD|STATIC)")
                            .ok()
                            .and_then(|re| re.captures(&clean))
                    {
                        card_data
                            .decoded
                            .insert("prng".to_string(), caps[1].to_uppercase());
                    }
                }
            }
            // Get magic card info
            if !card_data.decoded.contains_key("magic") {
                if let Ok(mf_output) =
                    connection::run_command(app, port, command_builder::build_hf_mf_info())
                        .await
                {
                    let clean = output_parser::strip_ansi(&mf_output);
                    if let Some(caps) = regex::Regex::new(r"(?i)(?:Magic|Gen(?:eration)?)\s*(?:capabilities)?[\s.:]*(?::[\s.]*)?(Gen\s*1[ab]?|CUID|USCUID|Gen\s*2|Gen\s*3|APDU|UFUID|GDM|Gen\s*4\s*(?:GTU|GDM)?|[Uu]ltimate)")
                        .ok()
                        .and_then(|re| re.captures(&clean))
                    {
                        card_data
                            .decoded
                            .insert("magic".to_string(), caps[1].to_string());
                    }
                }
            }
        }
        CardType::MifareUltralight | CardType::NTAG => {
            // Get UL/NTAG subtype info
            if let Ok(mfu_output) =
                connection::run_command(app, port, command_builder::build_hf_mfu_info()).await
            {
                let clean = output_parser::strip_ansi(&mfu_output);
                // Check for NTAG type
                if let Some(caps) = regex::Regex::new(r"(?i)NTAG\s*(\d{3})")
                    .ok()
                    .and_then(|re| re.captures(&clean))
                {
                    card_data
                        .decoded
                        .insert("ntag_type".to_string(), format!("NTAG{}", &caps[1]));
                }
                // Check for UL type
                if let Some(caps) =
                    regex::Regex::new(r"(?i)(?:MIFARE\s+)?Ultralight(?:\s+(EV1|C|Nano|AES))?")
                        .ok()
                        .and_then(|re| re.captures(&clean))
                {
                    if let Some(ul_variant) = caps.get(1) {
                        card_data.decoded.insert(
                            "ul_type".to_string(),
                            format!("Ultralight {}", ul_variant.as_str()),
                        );
                    }
                }
            }
        }
        _ => {}
    }
}

/// Common finish: transition FSM to CardFound with detected card info.
fn finish_scan(
    machine: &Mutex<WizardMachine>,
    card_type: CardType,
    card_data: crate::cards::types::CardData,
) -> Result<WizardState, AppError> {
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
}
