pub mod models;

use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::Mutex;

use crate::error::AppError;

pub struct Database {
    pub conn: Mutex<Connection>,
}

impl Database {
    pub fn open(app_data_dir: PathBuf) -> Result<Self, AppError> {
        std::fs::create_dir_all(&app_data_dir).map_err(|e| {
            AppError::DatabaseError(format!("Cannot create data dir: {}", e))
        })?;

        let db_path = app_data_dir.join("phosphor.db");
        let conn = Connection::open(&db_path)?;

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS clone_log (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                source_type TEXT NOT NULL,
                source_uid  TEXT NOT NULL,
                target_type TEXT NOT NULL,
                target_uid  TEXT NOT NULL,
                port        TEXT NOT NULL,
                success     INTEGER NOT NULL DEFAULT 0,
                timestamp   TEXT NOT NULL,
                notes       TEXT
            );",
        )?;

        Ok(Database {
            conn: Mutex::new(conn),
        })
    }
}
