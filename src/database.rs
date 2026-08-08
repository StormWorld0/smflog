// -- https://github.com/StormWorld0/smflog
// -- GPLv2 License
// -- Author: zxelzy

use pyo3::prelude::*;
use rusqlite::Connection;
use std::env;
use std::fs;

#[cfg(unix)]
use std::os::unix::fs::DirBuilderExt;
use crate::errors::PrintResult;

pub fn get_db_connection(_py: Python<'_>) -> PrintResult<Connection> {
    // Ekstrak Path tmp
    // Linux/macOS: /tmp/ atau $TMPDIR
    // Windows: C:\Users\<User>\AppData\Local\Temp\
    let tmp_path = env::temp_dir();

    // tmp/smflog/
    let output_dir = tmp_path.join("smflog");

    // tmp/smflog/smflog.db    
    let file_path = output_dir.join("smflog.db");

    // Pre-flight check & Secure Directory Creation
    if !output_dir.exists() {
        let mut builder = fs::DirBuilder::new();
        builder.recursive(true);
        
        // Permission (rwx------)
        #[cfg(unix)]
        builder.mode(0o700); 
        
        builder.create(&output_dir)?;
    }
    // Buka Koneksi SQLite (Otomatis membuat file log.db jika belum ada)
    // ? operator di sini akan dilempar sebagai SqliteError ke PrintResult
    let conn = Connection::open(file_path)?;
    
    // Konfigurasi Mesin Database High-Performance
    conn.execute_batch(
        "PRAGMA journal_mode = WAL;
         PRAGMA synchronous = NORMAL;
         PRAGMA temp_store = MEMORY;

         CREATE TABLE IF NOT EXISTS system_logs (
             id INTEGER PRIMARY KEY AUTOINCREMENT,
             timestamp REAL,
             level TEXT,
             label TEXT,
             payload TEXT,
             traceback TEXT,
             caller_info TEXT
         );",
    )?;
    
    Ok(conn)
}

pub fn insert_log(
    conn: &Connection, 
    timestamp: f64, 
    level: &str, 
    label: &str, 
    payload: &str, 
    traceback: &str, 
    caller_info: &str
) -> PrintResult<()> {
    conn.execute(
        "INSERT INTO system_logs (timestamp, level, label, payload, traceback, caller_info) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        (timestamp, level, label, payload, traceback, caller_info),
    )?;

    let should_cleanup = rand::random::<u8>() < 3; // Probabilitas ~1% (membutuhkan crate `rand`)

    if should_cleanup {
        let retention_period = 7 * 24 * 60 * 60; // 7 Hari
        
        // Hapus berdasarkan waktu
        conn.execute(
            "DELETE FROM system_logs
             WHERE timestamp < (strftime('%s','now') - ?1)",
            [retention_period],
        )?;

        // Hapus berdasarkan batas maksimal 10.000 row
        // Menggunakan OFFSET pada Primary Key index (O(1)) jauh lebih cepat daripada menghitung COUNT(*)
        conn.execute(
            "DELETE FROM system_logs 
             WHERE id <= (
                 SELECT id FROM system_logs 
                 ORDER BY id DESC 
                 LIMIT 1 OFFSET 10000
             )",
            [],
        )?;
    }

    Ok(())
}
