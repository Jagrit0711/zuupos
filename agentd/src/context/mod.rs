//! Context Engine — Long-term memory for ZuuposOS.
//!
//! Three memory tiers:
//!   1. Working memory  — current in-RAM context window (LLM-facing)
//!   2. Episodic log    — SQLite-backed session history (persistent)
//!   3. Pattern store   — compressed behavioral summaries (semantic summary)
//!
//! SQLite DB location: ~/.zuupos/memory.db

use crate::tools::ToolResult;
use log::{debug, warn};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::path::PathBuf;

pub struct ContextEngine {
    context_window: usize,
    working_memory: VecDeque<Episode>,
    db: Option<Connection>,
    cycle_count: u64,
}

#[derive(Clone, Debug)]
pub struct Episode {
    pub timestamp: u64,
    pub summary: String,
    #[allow(dead_code)]
    pub actions_taken: Vec<String>,
    #[allow(dead_code)]
    pub outcomes: Vec<String>,
}

/// A distilled user behavioral pattern.
#[derive(Clone, Debug)]
pub struct Pattern {
    pub key: String,
    pub value: String,
    pub frequency: u32,
}

/// Curated long-term memory entry that persists across sessions.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CuratedNote {
    pub id: i64,
    pub timestamp: u64,
    pub category: String, // "user_preference", "system_fact", "project_context", "workflow_rule"
    pub title: String,
    pub content: String,
    pub importance: u8, // 1 to 10
}

// ─── Perception stub (re-exported from agent module) ─────────────────────────
pub use crate::agent::Perception;

impl ContextEngine {
    pub fn new(context_window: usize) -> Self {
        let db = Self::open_db();
        Self {
            context_window,
            working_memory: VecDeque::new(),
            db,
            cycle_count: 0,
        }
    }

    fn open_db() -> Option<Connection> {
        let mut path = PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string()));
        path.push(".zuupos");
        if let Err(e) = std::fs::create_dir_all(&path) {
            warn!("[context] Could not create ~/.zuupos: {e}");
            return None;
        }
        path.push("memory.db");

        match Connection::open(&path) {
            Ok(conn) => {
                // Create tables if first run
                if let Err(e) = conn.execute_batch(
                    "CREATE TABLE IF NOT EXISTS episodes (
                        id       INTEGER PRIMARY KEY,
                        ts       INTEGER NOT NULL,
                        summary  TEXT    NOT NULL,
                        actions  TEXT    NOT NULL,
                        outcomes TEXT    NOT NULL
                    );
                    CREATE TABLE IF NOT EXISTS patterns (
                        key       TEXT PRIMARY KEY,
                        value     TEXT NOT NULL,
                        frequency INTEGER NOT NULL DEFAULT 1
                    );
                    CREATE TABLE IF NOT EXISTS curated_notes (
                        id         INTEGER PRIMARY KEY AUTOINCREMENT,
                        ts         INTEGER NOT NULL,
                        category   TEXT NOT NULL,
                        title      TEXT NOT NULL,
                        content    TEXT NOT NULL,
                        importance INTEGER NOT NULL DEFAULT 5
                    );
                    CREATE INDEX IF NOT EXISTS idx_curated_imp ON curated_notes(importance DESC, ts DESC);",
                ) {
                    warn!("[context] DB schema error: {e}");
                    return None;
                }

                // Seed initial foundational knowledge if fresh
                let count: i64 = conn
                    .query_row("SELECT COUNT(*) FROM curated_notes", [], |row| row.get(0))
                    .unwrap_or(0);
                if count == 0 {
                    let now = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_secs())
                        .unwrap_or(0) as i64;
                    let _ = conn.execute(
                        "INSERT INTO curated_notes (ts, category, title, content, importance) VALUES
                         (?1, 'system_fact', 'OS Architecture', 'ZuuposOS is an AI-native OS written in Rust: UEFI bootloader -> no_std microkernel -> agentd (PID 1) -> softbuffer shell.', 10),
                         (?1, 'user_preference', 'UI Aesthetic', 'Dark mode obsidian, warm amber/coral accents, glassmorphic cards, roaming reactive AI avatar.', 9),
                         (?1, 'project_context', 'Execution Model', 'Supports bare-metal QEMU boot and live host preview with Ollama / llama.cpp LLM inference.', 8)",
                        params![now],
                    );
                }

                debug!("[context] SQLite memory store opened: {:?}", path);
                Some(conn)
            }
            Err(e) => {
                warn!("[context] Could not open SQLite DB: {e}");
                None
            }
        }
    }

    /// Return a brief text summary of recent context for the LLM prompt.
    pub fn get_summary(&self) -> String {
        let recent: Vec<&str> = self
            .working_memory
            .iter()
            .rev()
            .take(5)
            .map(|e| e.summary.as_str())
            .collect();

        if recent.is_empty() {
            "No prior context. Fresh session.".to_string()
        } else {
            recent.join(" | ")
        }
    }

    /// Retrieve top curated long-term memories ordered by importance.
    pub fn get_top_curated_memories(&self, limit: usize) -> Vec<CuratedNote> {
        match &self.db {
            None => vec![],
            Some(db) => {
                let mut stmt = match db.prepare(
                    "SELECT id, ts, category, title, content, importance
                     FROM curated_notes
                     ORDER BY importance DESC, ts DESC
                     LIMIT ?1",
                ) {
                    Ok(s) => s,
                    Err(e) => {
                        warn!("[context] Prepare query failed: {e}");
                        return vec![];
                    }
                };

                let rows = stmt.query_map(params![limit as i64], |row| {
                    Ok(CuratedNote {
                        id: row.get(0)?,
                        timestamp: row.get::<_, i64>(1)? as u64,
                        category: row.get(2)?,
                        title: row.get(3)?,
                        content: row.get(4)?,
                        importance: row.get::<_, u32>(5)? as u8,
                    })
                });

                rows.ok()
                    .map(|iter| iter.flatten().collect())
                    .unwrap_or_default()
            }
        }
    }

    /// Search curated memories by keyword in title, content, or category.
    pub fn query_curated_notes(&self, query: &str, limit: usize) -> Vec<CuratedNote> {
        match &self.db {
            None => vec![],
            Some(db) => {
                let search_term = format!("%{}%", query);
                let mut stmt = match db.prepare(
                    "SELECT id, ts, category, title, content, importance
                     FROM curated_notes
                     WHERE title LIKE ?1 OR content LIKE ?1 OR category LIKE ?1
                     ORDER BY importance DESC
                     LIMIT ?2",
                ) {
                    Ok(s) => s,
                    Err(e) => {
                        warn!("[context] Search prepare failed: {e}");
                        return vec![];
                    }
                };

                let rows = stmt.query_map(params![search_term, limit as i64], |row| {
                    Ok(CuratedNote {
                        id: row.get(0)?,
                        timestamp: row.get::<_, i64>(1)? as u64,
                        category: row.get(2)?,
                        title: row.get(3)?,
                        content: row.get(4)?,
                        importance: row.get::<_, u32>(5)? as u8,
                    })
                });

                rows.ok()
                    .map(|iter| iter.flatten().collect())
                    .unwrap_or_default()
            }
        }
    }

    /// Explicitly store a curated piece of long-term memory.
    pub fn store_curated_note(
        &self,
        category: &str,
        title: &str,
        content: &str,
        importance: u8,
    ) -> Result<i64, String> {
        match &self.db {
            None => Err("Database unavailable".to_string()),
            Some(db) => {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0) as i64;
                let imp = importance.clamp(1, 10) as i64;

                db.execute(
                    "INSERT INTO curated_notes (ts, category, title, content, importance)
                     VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![now, category, title, content, imp],
                )
                .map_err(|e| format!("DB insert error: {e}"))?;

                let id = db.last_insert_rowid();
                debug!("[context] Stored curated note #{id}: [{category}] {title}");
                Ok(id)
            }
        }
    }

    /// Formats curated long-term memories for injection into the agent's prompt.
    pub fn get_curated_summary(&self) -> String {
        let memories = self.get_top_curated_memories(6);
        if memories.is_empty() {
            "No curated long-term memories yet.".to_string()
        } else {
            memories
                .iter()
                .map(|m| format!("• [{}] {}: {} (imp: {})", m.category, m.title, m.content, m.importance))
                .collect::<Vec<_>>()
                .join("\n")
        }
    }

    /// Get frequently observed patterns for agent prompt enrichment.
    pub fn get_patterns(&self) -> Vec<Pattern> {
        match &self.db {
            None => vec![],
            Some(db) => {
                let mut stmt = db
                    .prepare("SELECT key, value, frequency FROM patterns ORDER BY frequency DESC LIMIT 5")
                    .unwrap_or_else(|_| panic!("DB prepare failed"));
                let rows = stmt.query_map([], |row| {
                    Ok(Pattern {
                        key: row.get(0)?,
                        value: row.get(1)?,
                        frequency: row.get(2)?,
                    })
                });
                rows.ok()
                    .map(|iter| iter.flatten().collect())
                    .unwrap_or_default()
            }
        }
    }

    /// Store an agent episode in memory after each loop iteration.
    pub fn store_episode(
        &mut self,
        perception: &Perception,
        _reasoning: &str,
        results: &[ToolResult],
    ) {
        self.cycle_count += 1;

        let actions: Vec<String> = results.iter().map(|r| r.tool.clone()).collect();
        let outcomes: Vec<String> = results.iter().map(|r| r.output.clone()).collect();

        let episode = Episode {
            timestamp: perception.timestamp,
            summary: format!(
                "[#{cycle}] t={ts}: {ev_count} events | Actions: {actions}",
                cycle = self.cycle_count,
                ts = perception.timestamp,
                ev_count = perception.events.len(),
                actions = actions.join(", "),
            ),
            actions_taken: actions.clone(),
            outcomes: outcomes.clone(),
        };

        debug!("[context] Storing episode: {}", episode.summary);
        self.working_memory.push_back(episode.clone());

        // Trim working memory
        while self.working_memory.len() > self.context_window {
            self.working_memory.pop_front();
        }

        // Persist to SQLite
        if let Some(db) = &self.db {
            let actions_json = serde_json::to_string(&actions).unwrap_or_default();
            let outcomes_json = serde_json::to_string(&outcomes).unwrap_or_default();
            if let Err(e) = db.execute(
                "INSERT INTO episodes (ts, summary, actions, outcomes) VALUES (?1, ?2, ?3, ?4)",
                params![
                    episode.timestamp as i64,
                    episode.summary,
                    actions_json,
                    outcomes_json
                ],
            ) {
                warn!("[context] Failed to persist episode: {e}");
            }
        }

        // Extract and update patterns from tool usage
        for action in &actions {
            self.update_pattern(action, "tool_used");
        }

        // Keep only 1000 most recent episodes in DB
        if self.cycle_count % 100 == 0 {
            self.prune_old_episodes();
        }
    }

    fn update_pattern(&self, key: &str, value: &str) {
        if let Some(db) = &self.db {
            let _ = db.execute(
                "INSERT INTO patterns (key, value, frequency)
                 VALUES (?1, ?2, 1)
                 ON CONFLICT(key) DO UPDATE SET frequency = frequency + 1",
                params![key, value],
            );
        }
    }

    fn prune_old_episodes(&self) {
        if let Some(db) = &self.db {
            let _ = db.execute(
                "DELETE FROM episodes WHERE id NOT IN (
                    SELECT id FROM episodes ORDER BY ts DESC LIMIT 1000
                )",
                [],
            );
        }
    }

    /// How many total cycles have been recorded.
    pub fn cycle_count(&self) -> u64 {
        self.cycle_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_curated_memory_persistence() {
        let ctx = ContextEngine::new(10);
        let id1 = ctx
            .store_curated_note(
                "user_preference",
                "Theme Preference",
                "User strongly prefers dark mode obsidian with warm amber accents",
                9,
            )
            .expect("Failed to store note 1");
        assert!(id1 > 0);

        let id2 = ctx
            .store_curated_note(
                "system_fact",
                "Kernel ABI",
                "Syscalls use int 0x80 ABI with RAX as syscall number",
                8,
            )
            .expect("Failed to store note 2");
        assert!(id2 > 0);

        let top = ctx.get_top_curated_memories(5);
        assert!(!top.is_empty());
        assert!(top[0].importance >= top[top.len() - 1].importance);

        let search = ctx.query_curated_notes("obsidian", 5);
        assert!(!search.is_empty());
        assert_eq!(search[0].category, "user_preference");

        let summary = ctx.get_curated_summary();
        assert!(summary.contains("Theme Preference"));
    }
}

