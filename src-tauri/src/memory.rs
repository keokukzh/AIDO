use chrono::Utc;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MemoryError {
    #[error("SQLite-Fehler: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("Dateisystem-Fehler: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub id: i64,
    pub created_at: String,
    pub layer: String,
    pub category: String,
    pub summary: String,
    pub details: String,
    pub source: String,
    pub validity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub id: i64,
    pub timestamp: String,
    pub action: String,
    pub access_mode: String,
    pub privacy_mode: String,
    pub approved: bool,
    pub summary: String,
}

pub struct MemoryEngine {
    db_path: PathBuf,
}

impl MemoryEngine {
    pub fn new(db_path: impl AsRef<Path>) -> Result<Self, MemoryError> {
        let db_path = db_path.as_ref().to_path_buf();
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let engine = Self { db_path };
        engine.init_schema()?;
        Ok(engine)
    }

    fn connect(&self) -> Result<Connection, MemoryError> {
        Ok(Connection::open(&self.db_path)?)
    }

    fn init_schema(&self) -> Result<(), MemoryError> {
        let conn = self.connect()?;
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS memory_entries (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                created_at TEXT NOT NULL,
                layer TEXT NOT NULL,
                category TEXT NOT NULL,
                summary TEXT NOT NULL,
                details TEXT NOT NULL,
                source TEXT NOT NULL,
                validity TEXT NOT NULL DEFAULT 'active'
            );

            CREATE VIRTUAL TABLE IF NOT EXISTS memory_fts USING fts5(
                summary,
                details,
                category,
                layer,
                content='memory_entries',
                content_rowid='id'
            );

            CREATE TRIGGER IF NOT EXISTS memory_ai AFTER INSERT ON memory_entries BEGIN
                INSERT INTO memory_fts(rowid, summary, details, category, layer)
                VALUES (new.id, new.summary, new.details, new.category, new.layer);
            END;

            CREATE TRIGGER IF NOT EXISTS memory_ad AFTER DELETE ON memory_entries BEGIN
                INSERT INTO memory_fts(memory_fts, rowid, summary, details, category, layer)
                VALUES ('delete', old.id, old.summary, old.details, old.category, old.layer);
            END;

            CREATE TABLE IF NOT EXISTS audit_log (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp TEXT NOT NULL,
                action TEXT NOT NULL,
                access_mode TEXT NOT NULL,
                privacy_mode TEXT NOT NULL,
                approved INTEGER NOT NULL,
                summary TEXT NOT NULL
            );
            ",
        )?;
        Ok(())
    }

    pub fn store_memory(
        &self,
        layer: &str,
        category: &str,
        summary: &str,
        details: &str,
        source: &str,
    ) -> Result<i64, MemoryError> {
        let conn = self.connect()?;
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO memory_entries (created_at, layer, category, summary, details, source, validity)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'active')",
            params![now, layer, category, summary, details, source],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn search_memory(&self, query: &str, limit: usize) -> Result<Vec<MemoryEntry>, MemoryError> {
        let conn = self.connect()?;
        let mut stmt = conn.prepare(
            "SELECT m.id, m.created_at, m.layer, m.category, m.summary, m.details, m.source, m.validity
             FROM memory_fts f
             JOIN memory_entries m ON m.id = f.rowid
             WHERE memory_fts MATCH ?1
             ORDER BY rank
             LIMIT ?2",
        )?;
        let rows = stmt.query_map(params![query, limit as i64], |row| {
            Ok(MemoryEntry {
                id: row.get(0)?,
                created_at: row.get(1)?,
                layer: row.get(2)?,
                category: row.get(3)?,
                summary: row.get(4)?,
                details: row.get(5)?,
                source: row.get(6)?,
                validity: row.get(7)?,
            })
        })?;
        let mut entries = Vec::new();
        for r in rows {
            entries.push(r?);
        }
        Ok(entries)
    }

    pub fn list_recent_memories(&self, limit: usize) -> Result<Vec<MemoryEntry>, MemoryError> {
        let conn = self.connect()?;
        let mut stmt = conn.prepare(
            "SELECT id, created_at, layer, category, summary, details, source, validity
             FROM memory_entries
             ORDER BY id DESC
             LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit as i64], |row| {
            Ok(MemoryEntry {
                id: row.get(0)?,
                created_at: row.get(1)?,
                layer: row.get(2)?,
                category: row.get(3)?,
                summary: row.get(4)?,
                details: row.get(5)?,
                source: row.get(6)?,
                validity: row.get(7)?,
            })
        })?;
        let mut entries = Vec::new();
        for r in rows {
            entries.push(r?);
        }
        Ok(entries)
    }

    pub fn record_audit(
        &self,
        action: &str,
        access_mode: &str,
        privacy_mode: &str,
        approved: bool,
        summary: &str,
    ) -> Result<i64, MemoryError> {
        let conn = self.connect()?;
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO audit_log (timestamp, action, access_mode, privacy_mode, approved, summary)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                now,
                action,
                access_mode,
                privacy_mode,
                if approved { 1 } else { 0 },
                summary
            ],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn list_audit_log(&self, limit: usize) -> Result<Vec<AuditLogEntry>, MemoryError> {
        let conn = self.connect()?;
        let mut stmt = conn.prepare(
            "SELECT id, timestamp, action, access_mode, privacy_mode, approved, summary
             FROM audit_log
             ORDER BY id DESC
             LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit as i64], |row| {
            let approved_int: i64 = row.get(5)?;
            Ok(AuditLogEntry {
                id: row.get(0)?,
                timestamp: row.get(1)?,
                action: row.get(2)?,
                access_mode: row.get(3)?,
                privacy_mode: row.get(4)?,
                approved: approved_int != 0,
                summary: row.get(6)?,
            })
        })?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sqlite_fts5_stores_and_searches_memories_and_audit() {
        let temp_dir = std::env::temp_dir().join(format!("aido_mem_test_{}", uuid::Uuid::new_v4()));
        let db_file = temp_dir.join("aido_memory.db");
        let engine = MemoryEngine::new(&db_file).unwrap();

        engine
            .store_memory(
                "SystemMemory",
                "FastCleanup",
                "RTX 5090 Systemcheck & Fast Cleanup abgeschlossen",
                "VRAM vor Cleanup: 25544 MiB, nach Sleep/Cleanup: 2830 MiB. Temporäre Cache-Dateien bereinigt.",
                "WindowsBridge::FastCleanupExecute",
            )
            .unwrap();

        let results = engine.search_memory("RTX AND Cleanup", 5).unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].summary.contains("RTX 5090"));

        engine
            .record_audit(
                "FastCleanupExecute",
                "StandardAccess",
                "PrivateOffline",
                true,
                "Nutzer hat Fast Cleanup genehmigt und ausgeführt.",
            )
            .unwrap();

        let logs = engine.list_audit_log(5).unwrap();
        assert_eq!(logs.len(), 1);
        assert!(logs[0].approved);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
