use tauri::AppHandle;

use crate::error::AppError;
use crate::pm3::connection;

#[tauri::command]
pub async fn run_raw_command(
    app: AppHandle,
    port: String,
    command: String,
) -> Result<String, AppError> {
    connection::run_command(&app, &port, &command).await
}
