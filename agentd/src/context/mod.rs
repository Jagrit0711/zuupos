//! Context Engine — Long-term memory for ZuuposOS.
//!
//! Three memory tiers:
//! 1. Working memory   — current context window (LLM-facing, short-lived)
//! 2. Episodic memory  — session logs (what happened, when, outcome)
//! 3. Semantic memory  — HNSW vector store of compressed user patterns

use crate::agent::{Perception, SystemEvent};
use crate::tools::ToolResult;
use log::debug;
use std::collections::VecDeque;

pub struct ContextEngine {
    context_window: usize,
    working_memory: VecDeque<Episode>,
    episodic_log: Vec<Episode>,
    // Phase 1.6: add HNSW vector store for semantic search
}

#[derive(Clone, Debug)]
pub struct Episode {
    pub timestamp: u64,
    pub summary: String,
    pub actions_taken: Vec<String>,
    pub outcomes: Vec<String>,
}

impl ContextEngine {
    pub fn new(context_window: usize) -> Self {
        Self {
            context_window,
            working_memory: VecDeque::new(),
            episodic_log: Vec::new(),
        }
    }

    /// Return a brief text summary of recent context for the LLM prompt.
    pub fn get_summary(&self) -> String {
        if self.working_memory.is_empty() {
            return "No prior context. First boot.".to_string();
        }
        self.working_memory
            .iter()
            .rev()
            .take(5)
            .map(|e| e.summary.as_str())
            .collect::<Vec<_>>()
            .join(" | ")
    }

    /// Store an agent episode in memory after each loop iteration.
    pub async fn store_episode(
        &mut self,
        perception: &Perception,
        reasoning: &str,
        results: &[ToolResult],
    ) {
        let episode = Episode {
            timestamp: perception.timestamp,
            summary: format!(
                "t={}: {} events processed. Actions: {}",
                perception.timestamp,
                perception.events.len(),
                results.iter().map(|r| r.tool.as_str()).collect::<Vec<_>>().join(", ")
            ),
            actions_taken: results.iter().map(|r| r.tool.clone()).collect(),
            outcomes: results.iter().map(|r| r.output.clone()).collect(),
        };

        debug!("[context] Storing episode: {}", episode.summary);
        self.working_memory.push_back(episode.clone());
        self.episodic_log.push(episode);

        // Trim working memory to fit context window
        while self.working_memory.len() > 50 {
            self.working_memory.pop_front();
        }
    }
}
