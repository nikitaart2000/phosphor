use rusqlite::params;
use serde::{Deserialize, Serialize};

use crate::db::Database;
use crate::error::AppError;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CloneRecord {
    pub id: Option<i64>,
    pub source_type: String,
    pub source_uid: String,
    pub target_type: String,
    pub target_uid: String,
    pub port: String,
    pub success: bool,
    pub timestamp: String,
    pub notes: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SavedCard {
    pub id: Option<i64>,
    pub name: String,
    pub card_type: String,
    pub frequency: String,
    pub uid: String,
    pub raw: String,
    pub decoded: String,
    pub cloneable: bool,
    pub recommended_blank: String,
    pub created_at: String,
}

impl Database {
    pub fn insert_record(&self, record: &CloneRecord) -> Result<i64, AppError> {
        let conn = self.conn.lock().map_err(|e| {
            AppError::DatabaseError(format!("Lock poisoned: {}", e))
        })?;
        conn.execute(
            "INSERT INTO clone_log (source_type, source_uid, target_type, target_uid, port, success, timestamp, notes)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                record.source_type,
                record.source_uid,
                record.target_type,
                record.target_uid,
                record.port,
                record.success as i32,
                record.timestamp,
                record.notes,
            ],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn get_history(&self, limit: u32) -> Result<Vec<CloneRecord>, AppError> {
        let conn = self.conn.lock().map_err(|e| {
            AppError::DatabaseError(format!("Lock poisoned: {}", e))
        })?;
        let mut stmt = conn.prepare(
            "SELECT id, source_type, source_uid, target_type, target_uid, port, success, timestamp, notes
             FROM clone_log ORDER BY id DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit], |row| {
            Ok(CloneRecord {
                id: row.get(0)?,
                source_type: row.get(1)?,
                source_uid: row.get(2)?,
                target_type: row.get(3)?,
                target_uid: row.get(4)?,
                port: row.get(5)?,
                success: row.get::<_, i32>(6)? != 0,
                timestamp: row.get(7)?,
                notes: row.get(8)?,
            })
        })?;

        let mut records = Vec::new();
        for row in rows {
            records.push(row?);
        }
        Ok(records)
    }

    pub fn insert_saved_card(&self, card: &SavedCard) -> Result<i64, AppError> {
        let conn = self.conn.lock().map_err(|e| {
            AppError::DatabaseError(format!("Lock poisoned: {}", e))
        })?;
        conn.execute(
            "INSERT INTO saved_cards (name, card_type, frequency, uid, raw, decoded, cloneable, recommended_blank, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                card.name,
                card.card_type,
                card.frequency,
                card.uid,
                card.raw,
                card.decoded,
                card.cloneable as i32,
                card.recommended_blank,
                card.created_at,
            ],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn get_saved_cards(&self) -> Result<Vec<SavedCard>, AppError> {
        let conn = self.conn.lock().map_err(|e| {
            AppError::DatabaseError(format!("Lock poisoned: {}", e))
        })?;
        let mut stmt = conn.prepare(
            "SELECT id, name, card_type, frequency, uid, raw, decoded, cloneable, recommended_blank, created_at
             FROM saved_cards ORDER BY created_at DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(SavedCard {
                id: row.get(0)?,
                name: row.get(1)?,
                card_type: row.get(2)?,
                frequency: row.get(3)?,
                uid: row.get(4)?,
                raw: row.get(5)?,
                decoded: row.get(6)?,
                cloneable: row.get::<_, i32>(7)? != 0,
                recommended_blank: row.get(8)?,
                created_at: row.get(9)?,
            })
        })?;

        let mut cards = Vec::new();
        for row in rows {
            cards.push(row?);
        }
        Ok(cards)
    }

    pub fn delete_saved_card(&self, id: i64) -> Result<(), AppError> {
        let conn = self.conn.lock().map_err(|e| {
            AppError::DatabaseError(format!("Lock poisoned: {}", e))
        })?;
        conn.execute("DELETE FROM saved_cards WHERE id = ?1", params![id])?;
        Ok(())
    }
}
