//! Planner — multi-step goal decomposition and task queue.
//!
//! The planner converts high-level agent reasoning into a concrete sequence
//! of steps, tracks their execution, and emits TaskPlan events to the shell.
//!
//! Architecture:
//!   Reasoner output → Planner::decompose() → Vec<PlanStep> (queued)
//!   Planner::tick() → pops next pending step → returns ToolCall to execute

use crate::ipc::{IpcClient, PlanStep, ShellEvent, StepStatus};
use crate::tools::ToolCall;
use log::{debug, info};
use std::collections::VecDeque;

pub struct Planner {
    active_plan: Vec<PlanStep>,
    step_queue: VecDeque<QueuedStep>,
    ipc: IpcClient,
    plan_id: u64,
}

struct QueuedStep {
    #[allow(dead_code)]
    plan_step: PlanStep,
    tool_call: ToolCall,
}

impl Planner {
    pub fn new(ipc: IpcClient) -> Self {
        Self {
            active_plan: Vec::new(),
            step_queue: VecDeque::new(),
            ipc,
            plan_id: 0,
        }
    }

    /// Parse LLM reasoning output and extract a structured multi-step plan.
    /// Falls back to single-step if no plan markers are found.
    pub fn decompose(&mut self, reasoning: &str) -> Vec<ToolCall> {
        // First, collect all JSON tool calls from the reasoning
        let raw_calls = extract_tool_calls(reasoning);

        if raw_calls.is_empty() {
            debug!("[planner] No tool calls in reasoning output");
            return vec![];
        }

        self.plan_id += 1;
        info!(
            "[planner] Decomposing plan #{} ({} steps)",
            self.plan_id,
            raw_calls.len()
        );

        // Build plan steps for display
        self.active_plan = raw_calls
            .iter()
            .enumerate()
            .map(|(i, call)| PlanStep {
                index: i as u32,
                description: format!("{}({})", call.tool, summarize_args(&call.args)),
                status: StepStatus::Pending,
            })
            .collect();

        // Emit plan to shell
        self.ipc
            .send(ShellEvent::TaskPlan(self.active_plan.clone()));

        // Queue all steps
        self.step_queue.clear();
        for (step, call) in self.active_plan.iter().cloned().zip(raw_calls.into_iter()) {
            self.step_queue.push_back(QueuedStep {
                plan_step: step,
                tool_call: call,
            });
        }

        // Return all tool calls for sequential execution
        self.step_queue
            .iter()
            .map(|s| ToolCall {
                tool: s.tool_call.tool.clone(),
                args: s.tool_call.args.clone(),
            })
            .collect()
    }

    /// Mark a step as currently executing.
    pub fn mark_active(&mut self, step_index: usize) {
        if let Some(step) = self.active_plan.get_mut(step_index) {
            step.status = StepStatus::Active;
        }
        if !self.active_plan.is_empty() {
            self.ipc
                .send(ShellEvent::TaskPlan(self.active_plan.clone()));
        }
    }

    /// Mark a step as done and update the shell plan display.
    pub fn mark_done(&mut self, step_index: usize) {
        if let Some(step) = self.active_plan.get_mut(step_index) {
            step.status = StepStatus::Done;
        }
        // Refresh plan display
        if !self.active_plan.is_empty() {
            self.ipc
                .send(ShellEvent::TaskPlan(self.active_plan.clone()));
        }
    }

    /// Mark a step as failed.
    pub fn mark_failed(&mut self, step_index: usize) {
        if let Some(step) = self.active_plan.get_mut(step_index) {
            step.status = StepStatus::Failed;
        }
        if !self.active_plan.is_empty() {
            self.ipc
                .send(ShellEvent::TaskPlan(self.active_plan.clone()));
        }
    }
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

/// Extract all JSON tool call objects from free-form LLM text.
pub fn extract_tool_calls(text: &str) -> Vec<ToolCall> {
    let mut calls = Vec::new();
    let mut pos = 0;

    while pos < text.len() {
        // Find next potential JSON object containing "tool"
        if let Some(rel) = text[pos..].find("{\"tool\"") {
            let abs = pos + rel;
            // Try to find matching closing brace
            let substr = &text[abs..];
            let mut depth = 0i32;
            let mut end = None;
            for (i, ch) in substr.char_indices() {
                match ch {
                    '{' => depth += 1,
                    '}' => {
                        depth -= 1;
                        if depth == 0 {
                            end = Some(i + 1);
                            break;
                        }
                    }
                    _ => {}
                }
            }
            if let Some(end) = end {
                let json_str = &substr[..end];
                if let Ok(call) = serde_json::from_str::<ToolCall>(json_str) {
                    calls.push(call);
                }
                pos = abs + end;
            } else {
                pos = abs + 1;
            }
        } else {
            break;
        }
    }
    calls
}

/// Create a short display summary of tool arguments.
fn summarize_args(args: &serde_json::Value) -> String {
    if let Some(obj) = args.as_object() {
        let parts: Vec<String> = obj
            .iter()
            .take(2)
            .map(|(k, v)| {
                let val_str = match v {
                    serde_json::Value::String(s) => {
                        if s.len() > 20 {
                            format!("\"{}…\"", &s[..20])
                        } else {
                            format!("\"{}\"", s)
                        }
                    }
                    _ => v.to_string(),
                };
                format!("{}={}", k, val_str)
            })
            .collect();
        parts.join(", ")
    } else {
        String::new()
    }
}
