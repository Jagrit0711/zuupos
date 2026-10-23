//! App lifecycle, registration, and process management.
//!
//! Apps in ZuuposOS are capability-bound processes. They register with the
//! agent runtime, declare their required capabilities, and communicate
//! via IPC with the agent.

use crate::capabilities::{Capability, CapabilitySet, CapabilityType};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppManifest {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub required_capabilities: Vec<CapabilityType>,
    pub entry_point: String,
    pub icon: Option<String>,
}

impl AppManifest {
    pub fn new(name: &str, version: &str) -> Self {
        Self {
            name: name.to_string(),
            version: version.to_string(),
            description: String::new(),
            author: String::new(),
            required_capabilities: Vec::new(),
            entry_point: "main".to_string(),
            icon: None,
        }
    }

    pub fn with_description(mut self, desc: &str) -> Self {
        self.description = desc.to_string();
        self
    }

    pub fn with_author(mut self, author: &str) -> Self {
        self.author = author.to_string();
        self
    }

    pub fn require_capability(mut self, cap: CapabilityType) -> Self {
        self.required_capabilities.push(cap);
        self
    }

    pub fn entry_point(mut self, entry: &str) -> Self {
        self.entry_point = entry.to_string();
        self
    }

    pub fn icon(mut self, icon: &str) -> Self {
        self.icon = Some(icon.to_string());
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct App {
    pub manifest: AppManifest,
    pub capabilities: CapabilitySet,
    pub state: AppState,
    pub pid: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AppState {
    Unregistered,
    Registered,
    Running,
    Suspended,
    Terminated,
    Crashed,
}

impl App {
    pub fn new(manifest: AppManifest) -> Self {
        let mut caps = CapabilitySet::empty();
        for cap in &manifest.required_capabilities {
            caps.grant(*cap);
        }

        Self {
            manifest,
            capabilities: caps,
            state: AppState::Unregistered,
            pid: None,
        }
    }

    pub fn register(&mut self) -> Result<(), AppError> {
        if self.state != AppState::Unregistered {
            return Err(AppError::AlreadyRegistered);
        }

        // In real implementation, send registration to agentd via IPC
        // For now, just update state
        self.state = AppState::Registered;
        Ok(())
    }

    pub fn spawn(&mut self) -> Result<u32, AppError> {
        if self.state != AppState::Registered {
            return Err(AppError::NotRegistered);
        }

        // In real implementation, send spawn request to kernel via agentd
        // For now, simulate
        self.pid = Some(1000); // Would come from kernel
        self.state = AppState::Running;
        Ok(self.pid.unwrap())
    }

    pub fn suspend(&mut self) -> Result<(), AppError> {
        if self.state != AppState::Running {
            return Err(AppError::NotRunning);
        }
        self.state = AppState::Suspended;
        Ok(())
    }

    pub fn resume(&mut self) -> Result<(), AppError> {
        if self.state != AppState::Suspended {
            return Err(AppError::NotSuspended);
        }
        self.state = AppState::Running;
        Ok(())
    }

    pub fn terminate(&mut self) -> Result<(), AppError> {
        if self.state == AppState::Terminated {
            return Err(AppError::AlreadyTerminated);
        }
        self.state = AppState::Terminated;
        self.pid = None;
        Ok(())
    }

    pub fn has_capability(&self, cap: CapabilityType) -> bool {
        self.capabilities.has(cap)
    }

    pub fn get_capability_tokens(&self) -> Vec<Capability> {
        self.capabilities.to_tokens()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub state: ProcessState,
    pub memory_mb: u64,
    pub cpu_percent: f32,
    pub capabilities: Vec<Capability>,
    pub parent_pid: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProcessState {
    Ready,
    Running,
    Blocked,
    Terminated,
}

/// IPC message for process management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProcessMessage {
    Spawn {
        manifest: AppManifest,
        capabilities: Vec<Capability>,
    },
    Kill {
        pid: u32,
    },
    Suspend {
        pid: u32,
    },
    Resume {
        pid: u32,
    },
    GetInfo {
        pid: u32,
    },
    ListAll,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProcessResponse {
    Spawned { pid: u32 },
    Killed,
    Suspended,
    Resumed,
    Info(ProcessInfo),
    List(Vec<ProcessInfo>),
    Error(String),
}

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("App already registered")]
    AlreadyRegistered,
    #[error("App not registered")]
    NotRegistered,
    #[error("App not running")]
    NotRunning,
    #[error("App not suspended")]
    NotSuspended,
    #[error("App already terminated")]
    AlreadyTerminated,
    #[error("Capability denied: {0:?}")]
    CapabilityDenied(CapabilityType),
    #[error("IPC error: {0}")]
    IpcError(String),
}
