// -- https://github.com/StormWorld0/smflog
// -- https://pypi.org/project/smflog
// -- GPLv2 License
// -- Author: zxelzy
use std::fs;
use pyo3::prelude::*;
use rusqlite::Connection;
use crate::errors::PrintResult;
use crate::path::log_dir;

pub fn get_db_connection(_py: Python<'_>) -> PrintResult<Connection> {
    // Extract Path log
    let log_path = log_dir("smflog")?;
    let file_path = log_path.join("log.db");

    // Ensure Directory is Available (Pre-flight check)
    if !log_path.exists() {
        fs::create_dir_all(&log_path)?;
    }
    
    // Open SQLite Connection (Automatically create log.db file if it doesn't exist)
    // ? operator here will be thrown as SqliteError to PrintResult
    let conn = Connection::open(file_path)?;
    
    // High-Performance Database Engine Configuration
    conn.execute_batch(
        "PRAGMA journal_mode = WAL;
         PRAGMA synchronous = NORMAL;
         PRAGMA temp_store = MEMORY;
         PRAGMA busy_timeout = 5000;
         
         CREATE TABLE IF NOT EXISTS system_logs (
             id INTEGER PRIMARY KEY AUTOINCREMENT,
             timestamp REAL,
             level TEXT,
             label TEXT,
             payload TEXT,
             caller TEXT,
             location TEXT,
             traceback TEXT
         );
         CREATE INDEX IF NOT EXISTS idx_system_logs_timestamp ON system_logs(timestamp);
         ",
    )?;
    
    Ok(conn)
}

pub fn insert_log(
    conn: &Connection, 
    timestamp: f64, 
    level: &str, 
    label: &str, 
    payload: &str, 
    caller: &str,
    location: &str,
    traceback: &str
) -> PrintResult<()> {
    let tx = conn.unchecked_transaction()?;
    
    tx.execute(
        "INSERT INTO system_logs (timestamp, level, label, payload, caller, location, traceback) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        (timestamp, level, label, payload, caller, location, traceback),
    )?;

    let should_cleanup = rand::random::<u8>() < 3; // Probability ~1%

    if should_cleanup {
        let retention_seconds = (3 * 24 * 60 * 60) as f64; // 3 Day
        let cutoff_timestamp = timestamp - retention_seconds;
        
        // Delete by time
        tx.execute(
            "DELETE FROM system_logs
             WHERE timestamp < ?1",
            [cutoff_timestamp],
        )?;

        // Delete based on maximum limit of 10,000 rows
        // Using OFFSET on Primary Key index (O(1)) 
        // much faster than calculating COUNT(*)
        tx.execute(
            "DELETE FROM system_logs 
             WHERE id <= (
                 SELECT id FROM system_logs 
                 ORDER BY id DESC 
                 LIMIT 1 OFFSET 10000
             )",
            [],
        )?;
    }

    tx.commit()?;
    Ok(())
}
