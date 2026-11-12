use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemoryLayer {
    SoulMemory,
    UserMemory,
    SystemMemory,
    ProjectMemory,
    EpisodicMemory,
    SemanticMemory,
    ProceduralMemory,
}

impl MemoryLayer {
    pub fn as_str(&self) -> &'static str {
        match self {
            MemoryLayer::SoulMemory => "SoulMemory",
            MemoryLayer::UserMemory => "UserMemory",
            MemoryLayer::SystemMemory => "SystemMemory",
            MemoryLayer::ProjectMemory => "ProjectMemory",
            MemoryLayer::EpisodicMemory => "EpisodicMemory",
            MemoryLayer::SemanticMemory => "SemanticMemory",
            MemoryLayer::ProceduralMemory => "ProceduralMemory",
        }
    }

    pub fn all() -> [MemoryLayer; 7] {
        [
            MemoryLayer::SoulMemory,
            MemoryLayer::UserMemory,
            MemoryLayer::SystemMemory,
            MemoryLayer::ProjectMemory,
            MemoryLayer::EpisodicMemory,
            MemoryLayer::SemanticMemory,
            MemoryLayer::ProceduralMemory,
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProceduralSkill {
    pub skill_id: String,
    pub title_de: String,
    pub trigger_pattern: String,
    pub steps_json: String,
    pub version: u32,
    pub validity_status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryQuotaReport {
    pub db_size_bytes: u64,
    pub quota_limit_bytes: u64,
    pub usage_percent: f64,
    pub counts_by_layer: Vec<(String, usize)>,
    pub procedural_skills_count: usize,
    pub quota_healthy: bool,
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

            CREATE TABLE IF NOT EXISTS procedural_skills (
                skill_id TEXT PRIMARY KEY,
                title_de TEXT NOT NULL,
                trigger_pattern TEXT NOT NULL,
                steps_json TEXT NOT NULL,
                version INTEGER NOT NULL DEFAULT 1,
                validity_status TEXT NOT NULL DEFAULT 'active',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            ",
        )?;

        let soul_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM memory_entries WHERE layer = 'SoulMemory'",
            [],
            |r| r.get(0),
        )?;
        if soul_count == 0 {
            let now = Utc::now().to_rfc3339();
            conn.execute(
                "INSERT INTO memory_entries (created_at, layer, category, summary, details, source, validity)
                 VALUES (?1, 'SoulMemory', 'CoreIdentity', ?2, ?3, 'Masterplan::SOUL', 'active')",
                params![
                    now,
                    "AIDO (Agentic Intelligent Developing Operator) — Ruhiger, ehrlicher lokaler Begleiter",
                    "AIDO denkt für dich, nicht du für AIDO. Keine Aktion ohne verständliche Freigabe. Private Daten bleiben lokal.",
                ],
            )?;
        }

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
             WHERE memory_fts MATCH ?1 AND m.validity = 'active'
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

    pub fn store_typed_memory(
        &self,
        layer: MemoryLayer,
        category: &str,
        summary: &str,
        details: &str,
        source: &str,
    ) -> Result<i64, MemoryError> {
        self.store_memory(layer.as_str(), category, summary, details, source)
    }

    pub fn register_or_update_procedural_skill(
        &self,
        skill_id: &str,
        title_de: &str,
        trigger_pattern: &str,
        steps_json: &str,
    ) -> Result<ProceduralSkill, MemoryError> {
        let conn = self.connect()?;
        let now = Utc::now().to_rfc3339();

        let existing: Option<(u32, String)> = conn
            .query_row(
                "SELECT version, created_at FROM procedural_skills WHERE skill_id = ?1",
                params![skill_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?;

        let (version, created_at) = match existing {
            Some((v, c)) => {
                let next_v = v + 1;
                conn.execute(
                    "UPDATE procedural_skills
                     SET title_de = ?1, trigger_pattern = ?2, steps_json = ?3, version = ?4, validity_status = 'active', updated_at = ?5
                     WHERE skill_id = ?6",
                    params![title_de, trigger_pattern, steps_json, next_v, now, skill_id],
                )?;
                (next_v, c)
            }
            None => {
                conn.execute(
                    "INSERT INTO procedural_skills (skill_id, title_de, trigger_pattern, steps_json, version, validity_status, created_at, updated_at)
                     VALUES (?1, ?2, ?3, ?4, 1, 'active', ?5, ?5)",
                    params![skill_id, title_de, trigger_pattern, steps_json, now],
                )?;
                (1, now.clone())
            }
        };

        // Also store corresponding ProceduralMemory entry in memory_entries
        self.store_typed_memory(
            MemoryLayer::ProceduralMemory,
            "ProceduralSkill",
            &format!("Prozeduraler Skill '{}' (v{}): {}", skill_id, version, title_de),
            steps_json,
            "MemoryEngine::register_or_update_procedural_skill",
        )?;

        Ok(ProceduralSkill {
            skill_id: skill_id.to_string(),
            title_de: title_de.to_string(),
            trigger_pattern: trigger_pattern.to_string(),
            steps_json: steps_json.to_string(),
            version,
            validity_status: "active".to_string(),
            created_at,
            updated_at: now,
        })
    }

    pub fn mark_skill_validity(&self, skill_id: &str, validity_status: &str) -> Result<bool, MemoryError> {
        let conn = self.connect()?;
        let now = Utc::now().to_rfc3339();
        let affected = conn.execute(
            "UPDATE procedural_skills SET validity_status = ?1, updated_at = ?2 WHERE skill_id = ?3",
            params![validity_status, now, skill_id],
        )?;
        Ok(affected > 0)
    }

    pub fn list_procedural_skills(&self, only_active: bool) -> Result<Vec<ProceduralSkill>, MemoryError> {
        let conn = self.connect()?;
        let sql = if only_active {
            "SELECT skill_id, title_de, trigger_pattern, steps_json, version, validity_status, created_at, updated_at
             FROM procedural_skills
             WHERE validity_status = 'active'
             ORDER BY skill_id ASC"
        } else {
            "SELECT skill_id, title_de, trigger_pattern, steps_json, version, validity_status, created_at, updated_at
             FROM procedural_skills
             ORDER BY skill_id ASC"
        };
        let mut stmt = conn.prepare(sql)?;
        let rows = stmt.query_map([], |row| {
            Ok(ProceduralSkill {
                skill_id: row.get(0)?,
                title_de: row.get(1)?,
                trigger_pattern: row.get(2)?,
                steps_json: row.get(3)?,
                version: row.get(4)?,
                validity_status: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    pub fn inspect_storage_quota(&self, quota_limit_bytes: u64) -> Result<MemoryQuotaReport, MemoryError> {
        let db_size_bytes = std::fs::metadata(&self.db_path)
            .map(|m| m.len())
            .unwrap_or(0);
        let conn = self.connect()?;

        let mut counts_by_layer = Vec::with_capacity(7);
        for layer in MemoryLayer::all() {
            let count: i64 = conn.query_row(
                "SELECT COUNT(*) FROM memory_entries WHERE layer = ?1 AND validity = 'active'",
                params![layer.as_str()],
                |row| row.get(0),
            )?;
            counts_by_layer.push((layer.as_str().to_string(), count as usize));
        }

        let procedural_skills_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM procedural_skills",
            [],
            |row| row.get(0),
        )?;

        let usage_percent = if quota_limit_bytes > 0 {
            (db_size_bytes as f64 / quota_limit_bytes as f64) * 100.0
        } else {
            0.0
        };

        let quota_healthy = db_size_bytes <= quota_limit_bytes;

        Ok(MemoryQuotaReport {
            db_size_bytes,
            quota_limit_bytes,
            usage_percent,
            counts_by_layer,
            procedural_skills_count: procedural_skills_count as usize,
            quota_healthy,
        })
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

    #[test]
    fn seven_memory_layers_soul_seed_and_storage_quota_report() {
        let temp_dir = std::env::temp_dir().join(format!("aido_mem_7layers_{}", uuid::Uuid::new_v4()));
        let db_file = temp_dir.join("aido_memory.db");
        let engine = MemoryEngine::new(&db_file).unwrap();

        // Verify SoulMemory is seeded on init
        let soul_entries = engine
            .list_memories_by_layer(Some("SoulMemory"), true, 10)
            .unwrap();
        assert_eq!(soul_entries.len(), 1);
        assert_eq!(soul_entries[0].category, "CoreIdentity");
        assert!(soul_entries[0].summary.contains("AIDO"));
        assert_eq!(soul_entries[0].source, "Masterplan::SOUL");

        // Add entries to other layers using store_typed_memory
        engine
            .store_typed_memory(
                MemoryLayer::UserMemory,
                "Preference",
                "Nutzer bevorzugt deutsche Ausgaben",
                "Spracheinstellung: DE",
                "UserInput",
            )
            .unwrap();

        engine
            .store_typed_memory(
                MemoryLayer::ProjectMemory,
                "Status",
                "AIDO Projekt in Phase 10",
                "Enterprise Core Completion",
                "TaskRunner",
            )
            .unwrap();

        let quota_limit = 50 * 1024 * 1024; // 50 MB
        let report = engine.inspect_storage_quota(quota_limit).unwrap();
        assert!(report.db_size_bytes > 0);
        assert_eq!(report.quota_limit_bytes, quota_limit);
        assert!(report.quota_healthy);
        assert!(report.usage_percent < 5.0);
        assert_eq!(report.counts_by_layer.len(), 7);

        // Verify layer counts
        let soul_tuple = report.counts_by_layer.iter().find(|(l, _)| l == "SoulMemory").unwrap();
        assert_eq!(soul_tuple.1, 1);
        let user_tuple = report.counts_by_layer.iter().find(|(l, _)| l == "UserMemory").unwrap();
        assert_eq!(user_tuple.1, 1);
        let project_tuple = report.counts_by_layer.iter().find(|(l, _)| l == "ProjectMemory").unwrap();
        assert_eq!(project_tuple.1, 1);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn procedural_memory_versions_learned_skills_and_tracks_validity() {
        let temp_dir = std::env::temp_dir().join(format!("aido_mem_procedural_{}", uuid::Uuid::new_v4()));
        let db_file = temp_dir.join("aido_memory.db");
        let engine = MemoryEngine::new(&db_file).unwrap();

        let skill_id = "clean_build_artifacts";
        let skill_v1 = engine
            .register_or_update_procedural_skill(
                skill_id,
                "Build-Artefakte aufräumen",
                "cleanup build",
                r#"{"steps":["cargo clean","del target"]}"#,
            )
            .unwrap();
        assert_eq!(skill_v1.version, 1);
        assert_eq!(skill_v1.validity_status, "active");

        // Check ProceduralMemory layer was populated
        let proc_memories = engine
            .list_memories_by_layer(Some("ProceduralMemory"), true, 10)
            .unwrap();
        assert_eq!(proc_memories.len(), 1);
        assert!(proc_memories[0].summary.contains("clean_build_artifacts"));

        // Update skill to v2
        let skill_v2 = engine
            .register_or_update_procedural_skill(
                skill_id,
                "Build-Artefakte aufräumen v2",
                "cleanup build",
                r#"{"steps":["cargo clean --release","del target"]}"#,
            )
            .unwrap();
        assert_eq!(skill_v2.version, 2);
        assert_eq!(skill_v2.validity_status, "active");

        // Mark outdated
        let marked = engine.mark_skill_validity(skill_id, "outdated").unwrap();
        assert!(marked);

        let active_skills = engine.list_procedural_skills(true).unwrap();
        assert!(active_skills.is_empty());

        let all_skills = engine.list_procedural_skills(false).unwrap();
        assert_eq!(all_skills.len(), 1);
        assert_eq!(all_skills[0].validity_status, "outdated");
        assert_eq!(all_skills[0].version, 2);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
