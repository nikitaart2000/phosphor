use std::sync::Mutex;
use tauri::{AppHandle, State};

use crate::cards::types::{BlankType, MagicGeneration, RecoveryAction};
use crate::error::AppError;
use crate::pm3::{command_builder, connection, output_parser};
use crate::state::{WizardAction, WizardMachine, WizardState};

/// Detect whether a blank card is present on the reader.
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
        BlankType::MagicMifareGen1a
        | BlankType::MagicMifareGen2
        | BlankType::MagicMifareGen3
        | BlankType::MagicMifareGen4GTU
        | BlankType::MagicMifareGen4GDM => {
            detect_magic_mifare(&app, &port, &machine, expected_blank).await
        }
        BlankType::MagicUltralight => detect_magic_ultralight(&app, &port, &machine).await,
        BlankType::IClassBlank => detect_iclass_blank(&app, &port, &machine).await,
    }
}

/// Run `lf t55xx detect` to confirm a T5577 is present, then `lf search` to
/// check if the card already has data written to it.
async fn detect_t5577(
    app: &AppHandle,
    port: &str,
    machine: &State<'_, Mutex<WizardMachine>>,
) -> Result<WizardState, AppError> {
    let output = connection::run_command(app, port, command_builder::build_t5577_detect()).await?;
    let status = output_parser::parse_t5577_detect(&output);

    if status.detected {
        // Check if the card already has data by running lf search
        let existing_data_type = match connection::run_command(app, port, "lf search").await {
            Ok(search_output) => {
                output_parser::parse_lf_search(&search_output)
                    .map(|(card_type, _)| format!("{:?}", card_type))
            }
            Err(_) => None,
        };

        let mut m = machine.lock().map_err(|e| {
            AppError::CommandFailed(format!("State lock poisoned: {}", e))
        })?;
        m.transition(WizardAction::BlankReady {
            blank_type: BlankType::T5577,
            existing_data_type,
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
        // Check if the card already has data
        let existing_data_type = match connection::run_command(app, port, "lf search").await {
            Ok(search_output) => {
                output_parser::parse_lf_search(&search_output)
                    .map(|(card_type, _)| format!("{:?}", card_type))
            }
            Err(_) => None,
        };

        let mut m = machine.lock().map_err(|e| {
            AppError::CommandFailed(format!("State lock poisoned: {}", e))
        })?;
        m.transition(WizardAction::BlankReady {
            blank_type: BlankType::EM4305,
            existing_data_type,
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

/// Map BlankType to the expected MagicGeneration for comparison.
fn expected_generation(blank: &BlankType) -> Option<MagicGeneration> {
    match blank {
        BlankType::MagicMifareGen1a => Some(MagicGeneration::Gen1a),
        BlankType::MagicMifareGen2 => Some(MagicGeneration::Gen2),
        BlankType::MagicMifareGen3 => Some(MagicGeneration::Gen3),
        BlankType::MagicMifareGen4GTU => Some(MagicGeneration::Gen4GTU),
        BlankType::MagicMifareGen4GDM => Some(MagicGeneration::Gen4GDM),
        _ => None,
    }
}

/// Map detected MagicGeneration back to the matching BlankType.
fn generation_to_blank(gen: &MagicGeneration) -> BlankType {
    match gen {
        MagicGeneration::Gen1a => BlankType::MagicMifareGen1a,
        MagicGeneration::Gen2 => BlankType::MagicMifareGen2,
        MagicGeneration::Gen3 => BlankType::MagicMifareGen3,
        MagicGeneration::Gen4GTU => BlankType::MagicMifareGen4GTU,
        MagicGeneration::Gen4GDM => BlankType::MagicMifareGen4GDM,
    }
}

/// Detect a MIFARE Classic magic card by running `hf 14a info` + `hf mf info`.
/// Checks that an ISO 14443-A card is present, then detects magic generation.
async fn detect_magic_mifare(
    app: &AppHandle,
    port: &str,
    machine: &State<'_, Mutex<WizardMachine>>,
    expected_blank: BlankType,
) -> Result<WizardState, AppError> {
    // Step 1: Check if any HF card is present via `hf 14a info`
    let card_present = match connection::run_command(app, port, command_builder::build_hf_14a_info())
        .await
    {
        Ok(output) => output_parser::is_hf_card_present(&output),
        Err(_) => false,
    };

    if !card_present {
        let mut m = machine.lock().map_err(|e| {
            AppError::CommandFailed(format!("State lock poisoned: {}", e))
        })?;
        m.transition(WizardAction::ReportError {
            message: "No HF card detected".to_string(),
            user_message: "No card found. Place the magic blank on the reader and try again."
                .to_string(),
            recoverable: true,
            recovery_action: Some(RecoveryAction::Retry),
        })?;
        return Ok(m.current.clone());
    }

    // Step 2: Detect magic generation via `hf mf info`
    let detected_gen = match connection::run_command(app, port, command_builder::build_hf_mf_info())
        .await
    {
        Ok(output) => output_parser::parse_magic_detection(&output),
        Err(_) => None,
    };

    let expected_gen = expected_generation(&expected_blank);

    match detected_gen {
        Some(ref gen) if Some(gen) == expected_gen.as_ref() => {
            // Perfect match — detected generation matches expected.
            // Check if card already has data written to it.
            let existing_data = check_mifare_data(app, port, gen).await;
            let mut m = machine.lock().map_err(|e| {
                AppError::CommandFailed(format!("State lock poisoned: {}", e))
            })?;
            m.transition(WizardAction::BlankReady {
                blank_type: expected_blank,
                existing_data_type: existing_data,
            })?;
            Ok(m.current.clone())
        }
        Some(ref gen) => {
            // Card present with magic capabilities, but different generation.
            // Accept the detected type instead — user placed a different magic card.
            let existing_data = check_mifare_data(app, port, gen).await;
            let actual_blank = generation_to_blank(gen);
            let mut m = machine.lock().map_err(|e| {
                AppError::CommandFailed(format!("State lock poisoned: {}", e))
            })?;
            let data_msg = match existing_data {
                Some(msg) => Some(format!("{} (detected {:?}, expected {:?})", msg, actual_blank, expected_blank)),
                None => Some(format!("Detected: {:?} (expected {:?})", actual_blank, expected_blank)),
            };
            m.transition(WizardAction::BlankReady {
                blank_type: actual_blank.clone(),
                existing_data_type: data_msg,
            })?;
            Ok(m.current.clone())
        }
        None => {
            // Card present but no magic detected — could be genuine MIFARE.
            // Still allow proceeding (some magic cards aren't detected by `hf mf info`).
            let mut m = machine.lock().map_err(|e| {
                AppError::CommandFailed(format!("State lock poisoned: {}", e))
            })?;
            m.transition(WizardAction::BlankReady {
                blank_type: expected_blank,
                existing_data_type: Some("No magic detected — card may be genuine".to_string()),
            })?;
            Ok(m.current.clone())
        }
    }
}

/// Check if a MIFARE Classic magic card has existing data by reading a data block.
/// For Gen1a/Gen4GDM: uses `hf mf cgetblk` (backdoor, no key needed).
/// For Gen2/Gen3/Gen4GTU: uses `hf mf rdbl` with default key.
/// Returns `Some("MIFARE Classic")` if data found, `None` if card appears blank.
async fn check_mifare_data(
    app: &AppHandle,
    port: &str,
    gen: &MagicGeneration,
) -> Option<String> {
    // Read block 4 (first data block of sector 1 — avoids manufacturer block 0)
    let cmd = match gen {
        MagicGeneration::Gen1a | MagicGeneration::Gen4GDM => {
            // Gen1a/GDM: backdoor read, no key needed
            command_builder::build_mf_cgetblk(4)
        }
        _ => {
            // Gen2/Gen3/Gen4GTU: try default key
            command_builder::build_mf_rdbl(4, "FFFFFFFFFFFF")
        }
    };

    match connection::run_command(app, port, &cmd).await {
        Ok(output) => {
            let clean = output_parser::strip_ansi(&output);
            // PM3 outputs block data as hex on a line with `[=]` or `[+]` marker
            // e.g. "[+]  4 | 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00"
            // Check if any hex data line contains non-zero bytes
            if has_nonzero_block_data(&clean) {
                Some("MIFARE Classic".to_string())
            } else {
                None // All zeros = blank
            }
        }
        Err(_) => {
            // Read failed — for Gen2+, this means keys were changed (non-default).
            // Changed keys = card has been written to.
            match gen {
                MagicGeneration::Gen1a | MagicGeneration::Gen4GDM => None, // Backdoor shouldn't fail
                _ => Some("MIFARE Classic (modified keys)".to_string()),
            }
        }
    }
}

/// Check if PM3 block read output contains non-zero data bytes.
/// Looks for hex data lines and checks if any byte is non-zero.
fn has_nonzero_block_data(output: &str) -> bool {
    for line in output.lines() {
        let trimmed = line.trim();
        // Look for lines with block data: "[+]  4 | AA BB CC ..." or just hex bytes
        // Skip lines that are status messages
        if trimmed.contains("[!!]") || trimmed.contains("[-]") {
            continue;
        }
        // Extract hex bytes after a pipe separator or from a data line
        let data_part = if let Some(idx) = trimmed.find('|') {
            &trimmed[idx + 1..]
        } else {
            continue
        };
        // Parse space-separated hex bytes
        let bytes: Vec<u8> = data_part
            .split_whitespace()
            .filter_map(|s| u8::from_str_radix(s, 16).ok())
            .collect();
        // If we got a full block (16 bytes for Classic) and any byte is non-zero → has data
        if bytes.len() >= 16 && bytes.iter().any(|&b| b != 0) {
            return true;
        }
    }
    false
}

/// Detect a magic Ultralight/NTAG card via `hf mfu info`.
async fn detect_magic_ultralight(
    app: &AppHandle,
    port: &str,
    machine: &State<'_, Mutex<WizardMachine>>,
) -> Result<WizardState, AppError> {
    let result = connection::run_command(app, port, command_builder::build_hf_mfu_info()).await;

    match result {
        Ok(output) => {
            // Check if any UL/NTAG card responded
            let clean = output_parser::strip_ansi(&output).to_lowercase();
            let card_found = clean.contains("uid") || clean.contains("ultralight") || clean.contains("ntag");

            if card_found {
                let existing_data_type = if output_parser::is_magic_ultralight(&output) {
                    None // Magic confirmed
                } else {
                    Some("No magic markers detected — card may be genuine".to_string())
                };

                let mut m = machine.lock().map_err(|e| {
                    AppError::CommandFailed(format!("State lock poisoned: {}", e))
                })?;
                m.transition(WizardAction::BlankReady {
                    blank_type: BlankType::MagicUltralight,
                    existing_data_type,
                })?;
                Ok(m.current.clone())
            } else {
                let mut m = machine.lock().map_err(|e| {
                    AppError::CommandFailed(format!("State lock poisoned: {}", e))
                })?;
                m.transition(WizardAction::ReportError {
                    message: "No Ultralight/NTAG card detected".to_string(),
                    user_message: "No Ultralight/NTAG card found. Place blank on the reader and try again."
                        .to_string(),
                    recoverable: true,
                    recovery_action: Some(RecoveryAction::Retry),
                })?;
                Ok(m.current.clone())
            }
        }
        Err(_) => {
            let mut m = machine.lock().map_err(|e| {
                AppError::CommandFailed(format!("State lock poisoned: {}", e))
            })?;
            m.transition(WizardAction::ReportError {
                message: "HF command failed".to_string(),
                user_message: "No Ultralight/NTAG card found. Place blank on the reader and try again."
                    .to_string(),
                recoverable: true,
                recovery_action: Some(RecoveryAction::Retry),
            })?;
            Ok(m.current.clone())
        }
    }
}

/// Detect an iCLASS/Picopass blank via `hf iclass info`.
async fn detect_iclass_blank(
    app: &AppHandle,
    port: &str,
    machine: &State<'_, Mutex<WizardMachine>>,
) -> Result<WizardState, AppError> {
    let result = connection::run_command(app, port, command_builder::build_hf_iclass_info()).await;

    let detected = match &result {
        Ok(output) => output_parser::is_iclass_present(output),
        Err(_) => false,
    };

    if detected {
        let mut m = machine.lock().map_err(|e| {
            AppError::CommandFailed(format!("State lock poisoned: {}", e))
        })?;
        m.transition(WizardAction::BlankReady {
            blank_type: BlankType::IClassBlank,
            existing_data_type: None,
        })?;
        Ok(m.current.clone())
    } else {
        let mut m = machine.lock().map_err(|e| {
            AppError::CommandFailed(format!("State lock poisoned: {}", e))
        })?;
        m.transition(WizardAction::ReportError {
            message: "No iCLASS card detected".to_string(),
            user_message: "No iCLASS card found. Place blank on the reader and try again."
                .to_string(),
            recoverable: true,
            recovery_action: Some(RecoveryAction::Retry),
        })?;
        Ok(m.current.clone())
    }
}
