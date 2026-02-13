use tauri::State;

use crate::db::models::SavedCard;
use crate::db::Database;
use crate::error::AppError;

#[tauri::command]
pub fn save_card(db: State<'_, Database>, card: SavedCard) -> Result<i64, AppError> {
    db.insert_saved_card(&card)
}

#[tauri::command]
pub fn get_saved_cards(db: State<'_, Database>) -> Result<Vec<SavedCard>, AppError> {
    db.get_saved_cards()
}

#[tauri::command]
pub fn delete_saved_card(db: State<'_, Database>, id: i64) -> Result<(), AppError> {
    db.delete_saved_card(id)
}
