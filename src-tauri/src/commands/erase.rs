use serde::Serialize;
use tauri::AppHandle;

use crate::error::AppError;
use crate::pm3::{command_builder, connection, output_parser};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DetectChipResult {
    pub chip_type: String,
    pub password_protected: bool,
    pub details: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WipeResult {
    pub success: bool,
    pub message: String,
}

/// Detect the underlying chip type on the reader (T5577 or EM4305).
/// Independent of the wizard FSM — can be called at any time.
#[tauri::command]
pub async fn detect_chip(app: AppHandle, port: String) -> Result<DetectChipResult, AppError> {
    // Validate port
    if port.is_empty() || port.len() > 32 {
        return Err(AppError::CommandFailed("Invalid port".into()));
    }

    // Try T5577 first (most common LF blank)
    let t5577_output =
        connection::run_command(&app, &port, command_builder::build_t5577_detect()).await?;
    let t5577_status = output_parser::parse_t5577_detect(&t5577_output);

    if t5577_status.detected {
        let details = if t5577_status.password_set {
            "T5577 chip detected (password protected)".to_string()
        } else {
            "T5577 chip detected".to_string()
        };

        return Ok(DetectChipResult {
            chip_type: "T5577".to_string(),
            password_protected: t5577_status.password_set,
            details,
        });
    }

    // Try EM4305
    let em_output = connection::run_command(&app, &port, command_builder::build_em4305_info())
        .await
        .unwrap_or_default();
    if output_parser::parse_em4305_info(&em_output) {
        return Ok(DetectChipResult {
            chip_type: "EM4305".to_string(),
            password_protected: false,
            details: "EM4305 chip detected".to_string(),
        });
    }

    Err(AppError::CommandFailed(
        "No erasable chip detected. Place a T5577 or EM4305 card on the reader.".into(),
    ))
}

/// Wipe a chip that was previously detected by `detect_chip`.
/// Independent of the wizard FSM.
#[tauri::command]
pub async fn wipe_chip(
    app: AppHandle,
    port: String,
    chip_type: String,
) -> Result<WipeResult, AppError> {
    // Validate port
    if port.is_empty() || port.len() > 32 {
        return Err(AppError::CommandFailed("Invalid port".into()));
    }

    let wipe_cmd = match chip_type.as_str() {
        "T5577" => {
            // Re-detect to check for password (card might have been swapped)
            let output =
                connection::run_command(&app, &port, command_builder::build_t5577_detect()).await?;
            let status = output_parser::parse_t5577_detect(&output);

            if !status.detected {
                return Ok(WipeResult {
                    success: false,
                    message: "T5577 no longer detected. Do not remove card during erase.".into(),
                });
            }

            // T5577 wipe — works for unprotected cards.
            // For password-protected cards, PM3 `lf t55xx wipe` tries default passwords.
            command_builder::build_t5577_wipe().to_string()
        }
        "EM4305" => command_builder::build_em4305_wipe().to_string(),
        other => {
            return Err(AppError::CommandFailed(format!(
                "Unsupported chip type for wipe: {}",
                other
            )));
        }
    };

    let wipe_output = connection::run_command(&app, &port, &wipe_cmd).await?;

    // Check for errors in output
    if wipe_output.contains("[!!]") || wipe_output.to_lowercase().contains("error") {
        return Ok(WipeResult {
            success: false,
            message: format!(
                "Wipe may have failed: {}",
                wipe_output
                    .lines()
                    .find(|l| l.contains("[!!]") || l.to_lowercase().contains("error"))
                    .unwrap_or("unknown error")
                    .trim()
            ),
        });
    }

    Ok(WipeResult {
        success: true,
        message: format!("{} erased successfully", chip_type),
    })
}
