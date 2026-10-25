//! Autonomous Research & System Inspection Tools for ZuuposOS Agent.
//!
//! Enables the agent to:
//! - Search online documentation and knowledge bases (`web_search`)
//! - Fetch and parse arbitrary web pages, APIs, and docs (`fetch_doc`)
//! - Inspect live system state (processes, disks, networks, env) (`inspect_system`)
//! - Store & recall curated long-term memories (`remember_fact`, `recall_memory`)

use super::Tool;
use crate::context::ContextEngine;
use log::{info, warn};
use std::sync::{Arc, Mutex};
use sysinfo::{Disks, Networks, System};

// ─── Web Search Tool ──────────────────────────────────────────────────────────

pub struct WebSearchTool {
    client: reqwest::Client,
}

impl WebSearchTool {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(8))
            .user_agent("ZuuposOS-Agent/0.6.0 (AI-Native OS; Research Engine)")
            .build()
            .unwrap_or_default();
        Self { client }
    }
}

impl Default for WebSearchTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl Tool for WebSearchTool {
    fn name(&self) -> &str {
        "web_search"
    }

    async fn execute(&self, args: &serde_json::Value) -> String {
        let query = args["query"].as_str().unwrap_or("").trim();
        if query.is_empty() {
            return "Error: Empty search query provided".to_string();
        }

        info!("[tool::web_search] Query: \"{query}\"");

        let url = format!(
            "https://api.duckduckgo.com/?q={}&format=json&no_html=1&skip_disambig=1",
            url_encode(query)
        );

        match self.client.get(&url).send().await {
            Ok(resp) => {
                if let Ok(json) = resp.json::<serde_json::Value>().await {
                    let mut results = Vec::new();

                    // Check Heading / Abstract
                    if let Some(heading) = json["Heading"].as_str() {
                        if !heading.is_empty() {
                            results.push(format!("### {}", heading));
                        }
                    }

                    if let Some(abs) = json["Abstract"].as_str() {
                        if !abs.is_empty() {
                            results.push(format!("Summary: {}", abs));
                        }
                    }

                    if let Some(source_url) = json["AbstractURL"].as_str() {
                        if !source_url.is_empty() {
                            results.push(format!("Source: {}", source_url));
                        }
                    }

                    // Check related topics
                    if let Some(topics) = json["RelatedTopics"].as_array() {
                        let mut count = 0;
                        for topic in topics {
                            if let Some(text) = topic["Text"].as_str() {
                                results.push(format!("- {}", text));
                                count += 1;
                                if count >= 4 {
                                    break;
                                }
                            }
                        }
                    }

                    if !results.is_empty() {
                        return results.join("\n");
                    }
                }
            }
            Err(e) => {
                warn!("[tool::web_search] DuckDuckGo request failed: {e}");
            }
        }

        // Fallback: Wikipedia search API
        let wiki_url = format!(
            "https://en.wikipedia.org/w/api.php?action=query&list=search&srsearch={}&format=json&utf8=1",
            url_encode(query)
        );

        if let Ok(resp) = self.client.get(&wiki_url).send().await {
            if let Ok(json) = resp.json::<serde_json::Value>().await {
                if let Some(search) = json["query"]["search"].as_array() {
                    let mut items = Vec::new();
                    for item in search.iter().take(3) {
                        let title = item["title"].as_str().unwrap_or("");
                        let snippet = clean_html(item["snippet"].as_str().unwrap_or(""));
                        items.push(format!("• {}: {}", title, snippet));
                    }
                    if !items.is_empty() {
                        return format!("Wikipedia Search Results:\n{}", items.join("\n"));
                    }
                }
            }
        }

        format!("No immediate search summary found for '{query}'. Try a more specific keyword or use fetch_doc.")
    }
}

// ─── Fetch Document / URL Tool ───────────────────────────────────────────────

pub struct FetchDocTool {
    client: reqwest::Client,
}

impl FetchDocTool {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .user_agent("ZuuposOS-Agent/0.6.0 (AI-Native OS; Research Engine)")
            .build()
            .unwrap_or_default();
        Self { client }
    }
}

impl Default for FetchDocTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl Tool for FetchDocTool {
    fn name(&self) -> &str {
        "fetch_doc"
    }

    async fn execute(&self, args: &serde_json::Value) -> String {
        let url = args["url"].as_str().unwrap_or("").trim();
        let max_len = args["max_len"].as_u64().unwrap_or(2000) as usize;

        if url.is_empty() || (!url.starts_with("http://") && !url.starts_with("https://")) {
            return "Error: Invalid URL. Must start with http:// or https://".to_string();
        }

        info!("[tool::fetch_doc] Fetching: {url}");

        match self.client.get(url).send().await {
            Ok(resp) => {
                if !resp.status().is_success() {
                    return format!("HTTP Error: {}", resp.status());
                }
                match resp.text().await {
                    Ok(body) => {
                        let clean = clean_html(&body);
                        let truncated = if clean.len() > max_len {
                            format!("{}... [truncated]", &clean[..max_len])
                        } else {
                            clean
                        };
                        format!("Fetched from {url} ({} chars):\n{}", truncated.len(), truncated)
                    }
                    Err(e) => format!("Failed to read response body: {e}"),
                }
            }
            Err(e) => format!("Request failed: {e}"),
        }
    }
}

// ─── Inspect System Tool ─────────────────────────────────────────────────────

pub struct InspectSystemTool;

#[async_trait::async_trait]
impl Tool for InspectSystemTool {
    fn name(&self) -> &str {
        "inspect_system"
    }

    async fn execute(&self, args: &serde_json::Value) -> String {
        let target = args["target"].as_str().unwrap_or("summary");
        info!("[tool::inspect_system] Target: {target}");

        let mut sys = System::new_all();
        sys.refresh_all();

        match target {
            "processes" => {
                let mut procs: Vec<_> = sys.processes().values().collect();
                procs.sort_by(|a, b| b.cpu_usage().partial_cmp(&a.cpu_usage()).unwrap());
                let top: Vec<String> = procs
                    .iter()
                    .take(8)
                    .map(|p| {
                        format!(
                            "PID {}: {:<16} | CPU: {:>4.1}% | MEM: {:>5} MB",
                            p.pid(),
                            p.name().to_string_lossy(),
                            p.cpu_usage(),
                            p.memory() / 1_048_576
                        )
                    })
                    .collect();
                format!("Top Running Processes:\n{}", top.join("\n"))
            }
            "disks" => {
                let disks = Disks::new_with_refreshed_list();
                let list: Vec<String> = disks
                    .iter()
                    .map(|d| {
                        format!(
                            "{:?}: Free {} GB / Total {} GB ({})",
                            d.mount_point(),
                            d.available_space() / 1_073_741_824,
                            d.total_space() / 1_073_741_824,
                            d.file_system().to_string_lossy()
                        )
                    })
                    .collect();
                if list.is_empty() {
                    "No mounted disk partitions discovered.".to_string()
                } else {
                    format!("Disk Storage:\n{}", list.join("\n"))
                }
            }
            "network" => {
                let nets = Networks::new_with_refreshed_list();
                let list: Vec<String> = nets
                    .iter()
                    .map(|(name, data)| {
                        format!(
                            "• {}: Received {} KB, Transmitted {} KB",
                            name,
                            data.total_received() / 1024,
                            data.total_transmitted() / 1024
                        )
                    })
                    .collect();
                format!("Network Interfaces:\n{}", list.join("\n"))
            }
            "env" => {
                let os_name = System::name().unwrap_or_else(|| "Unknown OS".to_string());
                let os_version = System::os_version().unwrap_or_else(|| "Unknown".to_string());
                let host_name = System::host_name().unwrap_or_else(|| "localhost".to_string());
                let cpus = sys.cpus().len();
                let user = std::env::var("USER").unwrap_or_else(|_| "user".to_string());
                let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());

                format!(
                    "System Environment:\n• Host: {} (User: {})\n• OS: {} v{}\n• CPU Cores: {}\n• Home: {}",
                    host_name, user, os_name, os_version, cpus, home
                )
            }
            _ => {
                let total_mem = sys.total_memory() / 1_048_576;
                let used_mem = sys.used_memory() / 1_048_576;
                let cpu_usage = sys.global_cpu_usage();
                let procs_count = sys.processes().len();
                let os_name = System::name().unwrap_or_else(|| "Unknown OS".to_string());

                format!(
                    "System Summary:\n• OS: {}\n• CPU: {:.1}%\n• Memory: {}/{} MB ({:.1}%)\n• Processes: {}",
                    os_name,
                    cpu_usage,
                    used_mem,
                    total_mem,
                    (used_mem as f32 / total_mem.max(1) as f32) * 100.0,
                    procs_count
                )
            }
        }
    }
}

// ─── Curated Memory Tools ────────────────────────────────────────────────────

pub struct RememberFactTool {
    context: Arc<Mutex<ContextEngine>>,
}

impl RememberFactTool {
    pub fn new(context: Arc<Mutex<ContextEngine>>) -> Self {
        Self { context }
    }
}

#[async_trait::async_trait]
impl Tool for RememberFactTool {
    fn name(&self) -> &str {
        "remember_fact"
    }

    async fn execute(&self, args: &serde_json::Value) -> String {
        let category = args["category"].as_str().unwrap_or("system_fact");
        let title = args["title"].as_str().unwrap_or("Note");
        let content = args["content"].as_str().unwrap_or("");
        let importance = args["importance"].as_u64().unwrap_or(7) as u8;

        if content.is_empty() {
            return "Error: Missing content for remembered fact".to_string();
        }

        let ctx = self.context.lock().unwrap();
        match ctx.store_curated_note(category, title, content, importance) {
            Ok(id) => {
                info!("[tool::remember_fact] #{id} [{category}] {title}: {content}");
                format!("Memory #{id} curated and saved permanently under [{category}] '{title}'")
            }
            Err(e) => format!("Failed to store fact: {e}"),
        }
    }
}

pub struct RecallMemoryTool {
    context: Arc<Mutex<ContextEngine>>,
}

impl RecallMemoryTool {
    pub fn new(context: Arc<Mutex<ContextEngine>>) -> Self {
        Self { context }
    }
}

#[async_trait::async_trait]
impl Tool for RecallMemoryTool {
    fn name(&self) -> &str {
        "recall_memory"
    }

    async fn execute(&self, args: &serde_json::Value) -> String {
        let query = args["query"].as_str().unwrap_or("");
        let limit = args["limit"].as_u64().unwrap_or(5) as usize;

        let ctx = self.context.lock().unwrap();
        let notes = if query.trim().is_empty() {
            ctx.get_top_curated_memories(limit)
        } else {
            ctx.query_curated_notes(query, limit)
        };

        if notes.is_empty() {
            format!("No curated memories found matching query '{query}'.")
        } else {
            let items: Vec<String> = notes
                .iter()
                .map(|n| format!("• [ID #{}] [{}] {}: {} (imp: {})", n.id, n.category, n.title, n.content, n.importance))
                .collect();
            format!("Recalled {} memories:\n{}", items.len(), items.join("\n"))
        }
    }
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn clean_html(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut in_tag = false;

    for ch in input.chars() {
        match ch {
            '<' => {
                in_tag = true;
                out.push(' ');
            }
            '>' => in_tag = false,
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }

    let replaced = out
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&nbsp;", " ");

    replaced
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .replace(" .", ".")
        .replace(" ,", ",")
        .replace(" !", "!")
        .replace(" ?", "?")
}

fn url_encode(input: &str) -> String {
    let mut encoded = String::with_capacity(input.len());
    for byte in input.bytes() {
        match byte {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(byte as char);
            }
            b' ' => encoded.push('+'),
            _ => {
                encoded.push_str(&format!("%{:02X}", byte));
            }
        }
    }
    encoded
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_html() {
        let raw = "<div class='content'><h1>Title &amp; Subtitle</h1><p>Paragraph with &quot;quotes&quot; and <a href='#'>links</a>.</p></div>";
        let cleaned = clean_html(raw);
        assert_eq!(cleaned, "Title & Subtitle Paragraph with \"quotes\" and links.");
    }

    #[test]
    fn test_url_encode() {
        assert_eq!(url_encode("hello world"), "hello+world");
        assert_eq!(url_encode("rust & c++"), "rust+%26+c%2B%2B");
    }

    #[tokio::test]
    async fn test_inspect_system() {
        let tool = InspectSystemTool;
        let summary = tool.execute(&serde_json::json!({"target": "summary"})).await;
        assert!(summary.contains("System Summary"));
        assert!(summary.contains("CPU:"));

        let env = tool.execute(&serde_json::json!({"target": "env"})).await;
        assert!(env.contains("System Environment"));
    }
}

