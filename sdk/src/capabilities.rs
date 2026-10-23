//! Capability tokens — explicit permission grants for OS resources.
//!
//! Every operation in ZuuposOS requires a capability token. The agent holds
//! the root capability set and delegates specific capabilities to child processes.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Capability(pub u64);

impl Capability {
    pub const fn new(id: u64) -> Self {
        Self(id)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CapabilityType {
    // Process capabilities
    SpawnProcess,
    KillProcess,
    InspectProcess,

    // Memory capabilities
    AllocateMemory,
    MapMemory,
    ShareMemory,

    // Filesystem capabilities
    ReadFile,
    WriteFile,
    ListDirectory,
    CreateFile,
    DeleteFile,

    // Network capabilities
    TcpConnect,
    TcpListen,
    UdpBind,
    DnsResolve,

    // Display/UI capabilities
    CreateWindow,
    DrawFramebuffer,
    ShowCard,
    UpdateAvatar,

    // Hardware capabilities
    GpioAccess,
    I2cAccess,
    SpiAccess,
    PciConfig,

    // Agent capabilities
    RegisterTool,
    CallLlm,
    StoreMemory,
    QueryPatterns,

    // System capabilities
    Reboot,
    Shutdown,
    ReadSensors,
    SetTimer,
}

impl CapabilityType {
    pub fn to_id(&self) -> u64 {
        match self {
            CapabilityType::SpawnProcess => 0x01,
            CapabilityType::KillProcess => 0x02,
            CapabilityType::InspectProcess => 0x03,
            CapabilityType::AllocateMemory => 0x10,
            CapabilityType::MapMemory => 0x11,
            CapabilityType::ShareMemory => 0x12,
            CapabilityType::ReadFile => 0x20,
            CapabilityType::WriteFile => 0x21,
            CapabilityType::ListDirectory => 0x22,
            CapabilityType::CreateFile => 0x23,
            CapabilityType::DeleteFile => 0x24,
            CapabilityType::TcpConnect => 0x30,
            CapabilityType::TcpListen => 0x31,
            CapabilityType::UdpBind => 0x32,
            CapabilityType::DnsResolve => 0x33,
            CapabilityType::CreateWindow => 0x40,
            CapabilityType::DrawFramebuffer => 0x41,
            CapabilityType::ShowCard => 0x42,
            CapabilityType::UpdateAvatar => 0x43,
            CapabilityType::GpioAccess => 0x50,
            CapabilityType::I2cAccess => 0x51,
            CapabilityType::SpiAccess => 0x52,
            CapabilityType::PciConfig => 0x53,
            CapabilityType::RegisterTool => 0x60,
            CapabilityType::CallLlm => 0x61,
            CapabilityType::StoreMemory => 0x62,
            CapabilityType::QueryPatterns => 0x63,
            CapabilityType::Reboot => 0x70,
            CapabilityType::Shutdown => 0x71,
            CapabilityType::ReadSensors => 0x72,
            CapabilityType::SetTimer => 0x73,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilitySet {
    capabilities: HashSet<CapabilityType>,
}

impl CapabilitySet {
    pub fn new() -> Self {
        Self {
            capabilities: HashSet::new(),
        }
    }

    pub fn root() -> Self {
        let mut set = Self::new();
        // Root has all capabilities
        set.capabilities.extend([
            CapabilityType::SpawnProcess,
            CapabilityType::KillProcess,
            CapabilityType::InspectProcess,
            CapabilityType::AllocateMemory,
            CapabilityType::MapMemory,
            CapabilityType::ShareMemory,
            CapabilityType::ReadFile,
            CapabilityType::WriteFile,
            CapabilityType::ListDirectory,
            CapabilityType::CreateFile,
            CapabilityType::DeleteFile,
            CapabilityType::TcpConnect,
            CapabilityType::TcpListen,
            CapabilityType::UdpBind,
            CapabilityType::DnsResolve,
            CapabilityType::CreateWindow,
            CapabilityType::DrawFramebuffer,
            CapabilityType::ShowCard,
            CapabilityType::UpdateAvatar,
            CapabilityType::GpioAccess,
            CapabilityType::I2cAccess,
            CapabilityType::SpiAccess,
            CapabilityType::PciConfig,
            CapabilityType::RegisterTool,
            CapabilityType::CallLlm,
            CapabilityType::StoreMemory,
            CapabilityType::QueryPatterns,
            CapabilityType::Reboot,
            CapabilityType::Shutdown,
            CapabilityType::ReadSensors,
            CapabilityType::SetTimer,
        ]);
        set
    }

    pub fn empty() -> Self {
        Self::new()
    }

    pub fn grant(&mut self, cap: CapabilityType) {
        self.capabilities.insert(cap);
    }

    pub fn revoke(&mut self, cap: CapabilityType) {
        self.capabilities.remove(&cap);
    }

    pub fn has(&self, cap: CapabilityType) -> bool {
        self.capabilities.contains(&cap)
    }

    pub fn to_tokens(&self) -> Vec<Capability> {
        self.capabilities.iter().map(|c| Capability(c.to_id())).collect()
    }

    pub fn from_tokens(tokens: &[Capability]) -> Self {
        let mut set = Self::new();
        for token in tokens {
            if let Some(cap) = Self::id_to_type(token.0) {
                set.capabilities.insert(cap);
            }
        }
        set
    }

    fn id_to_type(id: u64) -> Option<CapabilityType> {
        match id {
            0x01 => Some(CapabilityType::SpawnProcess),
            0x02 => Some(CapabilityType::KillProcess),
            0x03 => Some(CapabilityType::InspectProcess),
            0x10 => Some(CapabilityType::AllocateMemory),
            0x11 => Some(CapabilityType::MapMemory),
            0x12 => Some(CapabilityType::ShareMemory),
            0x20 => Some(CapabilityType::ReadFile),
            0x21 => Some(CapabilityType::WriteFile),
            0x22 => Some(CapabilityType::ListDirectory),
            0x23 => Some(CapabilityType::CreateFile),
            0x24 => Some(CapabilityType::DeleteFile),
            0x30 => Some(CapabilityType::TcpConnect),
            0x31 => Some(CapabilityType::TcpListen),
            0x32 => Some(CapabilityType::UdpBind),
            0x33 => Some(CapabilityType::DnsResolve),
            0x40 => Some(CapabilityType::CreateWindow),
            0x41 => Some(CapabilityType::DrawFramebuffer),
            0x42 => Some(CapabilityType::ShowCard),
            0x43 => Some(CapabilityType::UpdateAvatar),
            0x50 => Some(CapabilityType::GpioAccess),
            0x51 => Some(CapabilityType::I2cAccess),
            0x52 => Some(CapabilityType::SpiAccess),
            0x53 => Some(CapabilityType::PciConfig),
            0x60 => Some(CapabilityType::RegisterTool),
            0x61 => Some(CapabilityType::CallLlm),
            0x62 => Some(CapabilityType::StoreMemory),
            0x63 => Some(CapabilityType::QueryPatterns),
            0x70 => Some(CapabilityType::Reboot),
            0x71 => Some(CapabilityType::Shutdown),
            0x72 => Some(CapabilityType::ReadSensors),
            0x73 => Some(CapabilityType::SetTimer),
            _ => None,
        }
    }
}