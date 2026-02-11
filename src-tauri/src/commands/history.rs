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
    // Validate field lengths to prevent oversized data from being stored in SQLite
    if record.source_type.len() > 100
        || record.source_uid.len() > 200
        || record.target_type.len() > 100
        || record.target_uid.len() > 200
        || record.port.len() > 50
        || record.timestamp.len() > 50
        || record.notes.as_ref().map_or(false, |n| n.len() > 1000)
    {
        return Err(AppError::CommandFailed(
            "Record fields too long".into(),
        ));
    }
    db.insert_record(&record)
}
