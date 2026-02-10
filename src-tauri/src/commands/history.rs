use tauri::State;

use crate::db::models::CloneRecord;
use crate::db::Database;
use crate::error::AppError;

#[tauri::command]
pub fn get_history(
    db: State<'_, Database>,
    limit: Option<u32>,
) -> Result<Vec<CloneRecord>, AppError> {
    db.get_history(limit.unwrap_or(50))
}

#[tauri::command]
pub fn save_clone_record(
    db: State<'_, Database>,
    record: CloneRecord,
) -> Result<i64, AppError> {
    db.insert_record(&record)
}
