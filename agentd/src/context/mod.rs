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
                    );",
                ) {
                    warn!("[context] DB schema error: {e}");
                    return None;
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
    pub async fn store_episode(
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
