
// src/storage/mod.rs

use rusqlite::{params, Connection, Result};
use std::time::{SystemTime, UNIX_EPOCH};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use crate::core::constants::*;

pub struct ClipboardDb {
    pub path: String,
    conn: Connection,
}

impl ClipboardDb {
    /// Open the database with optimized memory and caching configurations.
    pub fn open() -> Self {
        let db_path = crate::core::get_db_path();
        let conn = Connection::open(&db_path).expect("failed to open sqlite connection");

        if let Ok(metadata) = fs::metadata(&db_path) {
            let mut perms = metadata.permissions();
            if perms.mode() != 0o600 {
                perms.set_mode(0o600);
                let _ = fs::set_permissions(&db_path, perms);
            }
        }

        // 🚀 Optimization: Enhanced PRAGMA settings for high-throughput BLOB handling
        conn.busy_timeout(std::time::Duration::from_millis(SQLITE_TIMEOUT_MS)).ok();
        conn.execute_batch("
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = NORMAL;
            PRAGMA temp_store = MEMORY;
            PRAGMA mmap_size = 268435456;   -- 256MB memory map
            PRAGMA cache_size = -64000;     -- 64MB page cache
            PRAGMA page_size = 4096;        -- Standard page size for SSD alignment
        ").ok();

        conn.execute(
            "CREATE TABLE IF NOT EXISTS clipboard (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp INTEGER NOT NULL,
                mime TEXT NOT NULL,
                size INTEGER NOT NULL,
                preview TEXT,
                content BLOB NOT NULL,
                hash TEXT
            )", [],
        ).expect("failed to initialize schema");

        conn.execute("CREATE INDEX IF NOT EXISTS idx_ts ON clipboard(timestamp)", []).ok();
        conn.execute("CREATE INDEX IF NOT EXISTS idx_hash ON clipboard(hash)", []).ok();

        Self { path: db_path, conn }
    }

    /// Primary insertion logic. Accepts a pre-calculated hash to avoid redundant CPU cycles.
    pub fn insert_raw(&mut self, mime: &str, data: &[u8]) -> Result<()> {
        let hash = format!("{:x}", md5::compute(data));
        self.insert_with_hash(mime, data, &hash)
    }

    /// Internal logic for atomic persistence with pre-computed hash validation.
    /// 🚀 Optimization: Minimizes data traversal by using the hash provided by the caller.
    pub fn insert_with_hash(&mut self, mime: &str, data: &[u8], hash: &str) -> Result<()> {
        if data.is_empty() { return Ok(()); }

        if SENSITIVE_MIME_HINTS.iter().any(|&hint| mime.contains(hint)) {
            return Ok(());
        }

        let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as i64;
        let tx = self.conn.transaction()?;

        // Fast lookup via idx_hash
        let existing: Option<i64> = tx.query_row(
            "SELECT id FROM clipboard WHERE hash = ?1 LIMIT 1",
            params![hash],
            |row| row.get(0)
        ).ok();

        if let Some(id) = existing {
            tx.execute("UPDATE clipboard SET timestamp = ?1 WHERE id = ?2", params![ts, id])?;
        } else {
            let preview = if mime.contains("text") {
                let s = String::from_utf8_lossy(data);
                Some(s.chars().take(PREVIEW_CHARS).collect::<String>().replace('\n', " "))
            } else { None };

            tx.execute(
                "INSERT INTO clipboard (timestamp, mime, size, preview, content, hash) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![ts, mime, data.len() as i64, preview, data, hash],
            )?;
        }

        tx.execute(
            "DELETE FROM clipboard WHERE id NOT IN (SELECT id FROM clipboard ORDER BY timestamp DESC LIMIT ?1)",
            params![MAX_HISTORY as i64]
        )?;

        tx.commit()
    }

    // --- Query Methods ---

    pub fn search_metadata(&self, query: &str, limit: usize) -> Vec<(i64, i64, String, i64, Option<String>)> {
        let mut stmt = match self.conn.prepare(
            "SELECT id, timestamp, mime, size, preview FROM clipboard 
             WHERE (mime LIKE '%text%' OR mime LIKE '%UTF8%') 
             AND (preview LIKE ?1 OR (preview IS NULL AND CAST(content AS TEXT) LIKE ?1))
             ORDER BY timestamp DESC LIMIT ?2"
        ) {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };

        let query_param = format!("%{}%", query);
        let rows = stmt.query_map(params![query_param, limit as i64], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?))
        }).unwrap();

        rows.filter_map(|r| r.ok()).collect()
    }

    pub fn get_latest_data(&self) -> Option<Vec<u8>> {
        self.conn.query_row(
            "SELECT content FROM clipboard ORDER BY timestamp DESC LIMIT 1",
            [],
            |row| row.get(0)
        ).ok()
    }

    pub fn fetch_metadata(&self, limit: usize) -> Vec<(i64, i64, String, i64, Option<String>)> {
        let mut stmt = self.conn.prepare(
            "SELECT id, timestamp, mime, size, preview FROM clipboard ORDER BY timestamp DESC LIMIT ?1"
        ).unwrap();
        let rows = stmt.query_map(params![limit as i64], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?))
        }).unwrap();
        rows.filter_map(|r| r.ok()).collect()
    }

    pub fn get_content_by_id(&self, id: i64) -> Option<(String, Vec<u8>)> {
        self.conn.query_row(
            "SELECT mime, content FROM clipboard WHERE id = ?1",
            params![id],
            |row| Ok((row.get(0)?, row.get(1)?))
        ).ok()
    }

    pub fn update_timestamp(&self, id: i64) -> Result<()> {
        let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as i64;
        self.conn.execute("UPDATE clipboard SET timestamp = ?1 WHERE id = ?2", params![ts, id])?;
        Ok(())
    }

    pub fn delete_by_id(&self, id: i64) -> Result<bool> {
        let res = self.conn.execute("DELETE FROM clipboard WHERE id = ?1", params![id])?;
        Ok(res > 0)
    }

    pub fn wipe(&self) -> Result<()> {
        self.conn.execute("DELETE FROM clipboard", [])?;
        self.conn.execute("VACUUM", [])?;
        Ok(())
    }

    pub fn get_total_count(&self) -> usize {
        self.conn.query_row(
            "SELECT COUNT(*) FROM clipboard",
            [],
            |row| row.get::<_, i64>(0).map(|val| val as usize)
        ).unwrap_or(0)
    }
}
