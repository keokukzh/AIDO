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
pub struct CompactionReport {
    pub compacted_source_entries: usize,
    pub created_semantic_entry_id: i64,
    pub category: String,
    pub digest_summary: String,
    pub saved_chars: usize,
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

            CREATE TRIGGER IF NOT EXISTS memory_au AFTER UPDATE ON memory_entries BEGIN
                INSERT INTO memory_fts(memory_fts, rowid, summary, details, category, layer)
                VALUES ('delete', old.id, old.summary, old.details, old.category, old.layer);
                INSERT INTO memory_fts(rowid, summary, details, category, layer)
                VALUES (new.id, new.summary, new.details, new.category, new.layer);
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

    pub fn list_memories_by_layer(
        &self,
        layer: Option<&str>,
        only_active: bool,
        limit: usize,
    ) -> Result<Vec<MemoryEntry>, MemoryError> {
        let conn = self.connect()?;
        let (sql, params_vec): (String, Vec<Box<dyn rusqlite::ToSql>>) = match (layer, only_active) {
            (Some(l), true) => (
                "SELECT id, created_at, layer, category, summary, details, source, validity
                 FROM memory_entries
                 WHERE layer = ?1 AND validity = 'active'
                 ORDER BY id DESC
                 LIMIT ?2"
                    .to_string(),
                vec![Box::new(l.to_string()), Box::new(limit as i64)],
            ),
            (Some(l), false) => (
                "SELECT id, created_at, layer, category, summary, details, source, validity
                 FROM memory_entries
                 WHERE layer = ?1
                 ORDER BY id DESC
                 LIMIT ?2"
                    .to_string(),
                vec![Box::new(l.to_string()), Box::new(limit as i64)],
            ),
            (None, true) => (
                "SELECT id, created_at, layer, category, summary, details, source, validity
                 FROM memory_entries
                 WHERE validity = 'active'
                 ORDER BY id DESC
                 LIMIT ?1"
                    .to_string(),
                vec![Box::new(limit as i64)],
            ),
            (None, false) => (
                "SELECT id, created_at, layer, category, summary, details, source, validity
                 FROM memory_entries
                 ORDER BY id DESC
                 LIMIT ?1"
                    .to_string(),
                vec![Box::new(limit as i64)],
            ),
        };

        let params_slice: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|b| b.as_ref()).collect();
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(&params_slice[..], |row| {
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

    pub fn compact_episodic_memories(
        &self,
        category_filter: Option<&str>,
        min_entries: usize,
    ) -> Result<Option<CompactionReport>, MemoryError> {
        let conn = self.connect()?;
        let (sql, params_vec): (String, Vec<Box<dyn rusqlite::ToSql>>) = match category_filter {
            Some(cat) => (
                "SELECT id, created_at, layer, category, summary, details, source, validity
                 FROM memory_entries
                 WHERE layer = 'EpisodicMemory' AND validity = 'active' AND category = ?1
                 ORDER BY id ASC"
                    .to_string(),
                vec![Box::new(cat.to_string())],
            ),
            None => (
                "SELECT id, created_at, layer, category, summary, details, source, validity
                 FROM memory_entries
                 WHERE layer = 'EpisodicMemory' AND validity = 'active'
                 ORDER BY id ASC"
                    .to_string(),
                vec![],
            ),
        };
        let params_slice: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|b| b.as_ref()).collect();
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(&params_slice[..], |row| {
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

        if entries.len() < min_entries || entries.is_empty() {
            return Ok(None);
        }

        let compacted_count = entries.len();
        let matched_category = category_filter
            .map(|c| c.to_string())
            .unwrap_or_else(|| entries[0].category.clone());

        let total_source_chars: usize = entries.iter().map(|e| e.summary.len() + e.details.len()).sum();

        let digest_summary = format!(
            "4-Stufen-Kompaktierung: {} episodische Aktivitäten konsolidiert in semantisches Wissen ({})",
            compacted_count, matched_category
        );

        let mut combined_details = format!(
            "Semantisch verdichtetes Wissen aus {} episodischen Einzelereignissen (Kategorie: {}):\n",
            compacted_count, matched_category
        );
        for e in &entries {
            combined_details.push_str(&format!(
                "- [{}] ID #{}: {}\n  Details: {}\n",
                e.created_at, e.id, e.summary, e.details
            ));
        }

        let new_chars = digest_summary.len() + combined_details.len();
        let saved_chars = if total_source_chars > new_chars {
            total_source_chars - new_chars
        } else {
            total_source_chars
        };

        // Insert new consolidated SemanticMemory
        let created_semantic_entry_id = self.store_memory(
            "SemanticMemory",
            &matched_category,
            &digest_summary,
            &combined_details,
            "MemoryEngine::compact_episodic_memories",
        )?;

        // Update the source episodic rows to 'compacted'
        let id_list: Vec<String> = entries.iter().map(|e| e.id.to_string()).collect();
        let in_clause = id_list.join(",");
        let update_sql = format!(
            "UPDATE memory_entries SET validity = 'compacted' WHERE id IN ({})",
            in_clause
        );
        conn.execute(&update_sql, [])?;

        Ok(Some(CompactionReport {
            compacted_source_entries: compacted_count,
            created_semantic_entry_id,
            category: matched_category,
            digest_summary,
            saved_chars,
        }))
    }

    pub fn forget_memory_entry(&self, entry_id: i64) -> Result<bool, MemoryError> {
        let conn = self.connect()?;
        let affected = conn.execute(
            "UPDATE memory_entries SET validity = 'forgotten_by_user' WHERE id = ?1",
            params![entry_id],
        )?;
        Ok(affected > 0)
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

    #[test]
    fn episodic_memories_compact_into_semantic_memory_and_preserve_traceability() {
        let temp_dir = std::env::temp_dir().join(format!("aido_mem_compact_{}", uuid::Uuid::new_v4()));
        let db_file = temp_dir.join("aido_memory.db");
        let engine = MemoryEngine::new(&db_file).unwrap();

        engine
            .store_memory(
                "EpisodicMemory",
                "FastCleanup",
                "Erstes Fast Cleanup ausgeführt",
                "3 Dateien entfernt, 448 KB bereinigt.",
                "test",
            )
            .unwrap();
        engine
            .store_memory(
                "EpisodicMemory",
                "FastCleanup",
                "Zweites Fast Cleanup ausgeführt",
                "2 Dateien entfernt, 256 KB bereinigt.",
                "test",
            )
            .unwrap();
        engine
            .store_memory(
                "EpisodicMemory",
                "FastCleanup",
                "Drittes Fast Cleanup ausgeführt",
                "5 Dateien entfernt, 1024 KB bereinigt.",
                "test",
            )
            .unwrap();

        let active_episodic = engine
            .list_memories_by_layer(Some("EpisodicMemory"), true, 10)
            .unwrap();
        assert_eq!(active_episodic.len(), 3);

        // Compact with min_entries = 2
        let report = engine
            .compact_episodic_memories(Some("FastCleanup"), 2)
            .unwrap();
        assert!(report.is_some());
        let rep = report.unwrap();
        assert_eq!(rep.compacted_source_entries, 3);
        assert_eq!(rep.category, "FastCleanup");

        // Active episodic memories should now be 0
        let active_after = engine
            .list_memories_by_layer(Some("EpisodicMemory"), true, 10)
            .unwrap();
        assert_eq!(active_after.len(), 0);

        // Inactive / all episodic entries should still exist as 'compacted'
        let all_episodic = engine
            .list_memories_by_layer(Some("EpisodicMemory"), false, 10)
            .unwrap();
        assert_eq!(all_episodic.len(), 3);
        assert!(all_episodic.iter().all(|e| e.validity == "compacted"));

        // SemanticMemory should now contain the compacted entry
        let semantic_entries = engine
            .list_memories_by_layer(Some("SemanticMemory"), true, 10)
            .unwrap();
        assert_eq!(semantic_entries.len(), 1);
        assert_eq!(semantic_entries[0].id, rep.created_semantic_entry_id);
        assert!(semantic_entries[0].summary.contains("4-Stufen-Kompaktierung"));

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn forget_memory_entry_marks_entry_forgotten_and_hides_from_active_layer_list() {
        let temp_dir = std::env::temp_dir().join(format!("aido_mem_forget_{}", uuid::Uuid::new_v4()));
        let db_file = temp_dir.join("aido_memory.db");
        let engine = MemoryEngine::new(&db_file).unwrap();

        let id = engine
            .store_memory(
                "WorkingMemory",
                "UserSession",
                "Temporäre Sitzungsdaten",
                "Details zur Benutzersitzung",
                "test",
            )
            .unwrap();

        let active = engine.list_memories_by_layer(None, true, 10).unwrap();
        assert!(active.iter().any(|e| e.id == id));

        let updated = engine.forget_memory_entry(id).unwrap();
        assert!(updated);

        let active_after = engine.list_memories_by_layer(None, true, 10).unwrap();
        assert!(!active_after.iter().any(|e| e.id == id));

        let all = engine.list_memories_by_layer(None, false, 10).unwrap();
        let forgotten = all.iter().find(|e| e.id == id).unwrap();
        assert_eq!(forgotten.validity, "forgotten_by_user");

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
