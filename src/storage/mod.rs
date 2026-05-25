
// src/storage/main.rs

use rusqlite::{params, Connection, Result};
use std::time::{SystemTime, UNIX_EPOCH};
use std::fs;
use std::os::unix::fs::PermissionsExt; // For permission configurations
use crate::core::constants::*;

pub struct ClipboardDb {
    pub path: String,
    conn: Connection,
}

impl ClipboardDb {
    /// Open the database safely and execute initial runtime configuration.
    pub fn open() -> Self {
        // 1. Resolve secure file path via core module utilities
        let db_path = crate::core::get_db_path();

        // 2. Establish database connection context
        let conn = Connection::open(&db_path).expect("failed to open sqlite connection");

        // 3. Security: Enforce owner-only (600) filesystem permissions to eliminate multi-user leakage risks
        if let Ok(metadata) = fs::metadata(&db_path) {
            let mut perms = metadata.permissions();
            if perms.mode() != 0o600 {
                perms.set_mode(0o600);
                let _ = fs::set_permissions(&db_path, perms);
            }
        }

        // 4. Performance & Stability: Inject global optimization PRAGMA instructions
        conn.busy_timeout(std::time::Duration::from_millis(SQLITE_TIMEOUT_MS)).ok();
        conn.execute_batch("
            PRAGMA journal_mode = WAL;      -- Concurrent read/write operational boost
            PRAGMA synchronous = NORMAL;    -- Disks synchronization profile optimized for SSDs
            PRAGMA temp_store = MEMORY;     -- Direct transient records processing to RAM
            PRAGMA mmap_size = 268435456;   -- Memory-map limit up to 256MB for handling large images
        ").ok();

        // 5. Initialize layout schemas
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

        // Indexes: Accelerate recent retrieval operations and keyword filtering passes
        conn.execute("CREATE INDEX IF NOT EXISTS idx_ts ON clipboard(timestamp)", []).ok();
        conn.execute("CREATE INDEX IF NOT EXISTS idx_hash ON clipboard(hash)", []).ok();

        Self { path: db_path, conn }
    }

    /// Insert raw data payloads using transactional atomic guarantees.
    pub fn insert_raw(&mut self, mime: &str, data: &[u8]) -> rusqlite::Result<()> {
        if data.is_empty() { return Ok(()); }

        // Privacy compliance filters
        if SENSITIVE_MIME_HINTS.iter().any(|&hint| mime.contains(hint)) {
            return Ok(());
        }

        let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as i64;
        let hash = format!("{:x}", md5::compute(data));

        // Initialize write transaction scope
        let tx = self.conn.transaction()?;

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

    /// Query: High-speed, sanitization-safe metadata scanning profile.
    pub fn search_metadata(&self, query: &str, limit: usize) -> Vec<(i64, i64, String, i64, Option<String>)> {
        let mut stmt = match self.conn.prepare(
            "SELECT id, timestamp, mime, size, preview FROM clipboard 
             WHERE (mime LIKE '%text%' OR mime LIKE '%UTF8%') 
             AND (preview LIKE ?1 OR CAST(content AS TEXT) LIKE ?1)
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

    /// Retrieve latest entry: Used primarily by daemons to prevent duplication.
    pub fn get_latest_data(&self) -> Option<Vec<u8>> {
        self.conn.query_row(
            "SELECT content FROM clipboard ORDER BY timestamp DESC LIMIT 1",
            [],

            |row| row.get(0)
        ).ok()
    }

    /// Fetch all metadata columns: Tailored for standard list layout rendering.
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

    pub fn delete_by_index(&self, idx: usize) -> Result<bool> {
        let res = self.conn.execute(
            "DELETE FROM clipboard WHERE id = (
                SELECT id FROM clipboard ORDER BY timestamp DESC LIMIT 1 OFFSET ?1
            )",
            params![idx as i64],
        )?;
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
