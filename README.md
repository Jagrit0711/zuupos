# ZuuposOS 🧠

> An AI-Native Agentic Operating System built from scratch in Rust.  
> No windows. No icons. No menus. Just an AI that *is* the OS.

---

## Vision

ZuuposOS is not an OS with an AI assistant bolted on.  
The AI **is** the kernel's personality layer. It boots, perceives the hardware, understands context, and acts — autonomously — without a user telling it what to do.

Think Taby, but it *is* the entire operating system.

---

## Architecture Overview

```
┌────────────────────────────────────────────────────────────────┐
│                        HARDWARE (x86-64)                       │
└────────────────────────┬───────────────────────────────────────┘
                         │ UEFI Boot
┌────────────────────────▼───────────────────────────────────────┐
│                    BOOTLOADER (zuupos-boot)                     │
│          UEFI app written in Rust using uefi-rs crate          │
│   • Sets up page tables, enters long mode, loads kernel ELF    │
└────────────────────────┬───────────────────────────────────────┘
                         │
┌────────────────────────▼───────────────────────────────────────┐
│                    KERNEL (zuupos-kernel)                       │
│                      Written in Rust (no_std)                  │
│                                                                │
│  ┌──────────────┐  ┌──────────────┐  ┌─────────────────────┐  │
│  │  Memory Mgr  │  │ Interrupt/   │  │   Driver Registry   │  │
│  │  (buddy alloc│  │ Exception    │  │  (keyboard, display, │  │
│  │   + VMM)     │  │ Handler      │  │   PCI, NVMe, Net)   │  │
│  └──────────────┘  └──────────────┘  └─────────────────────┘  │
│                                                                │
│  ┌────────────────────────────────────────────────────────┐    │
│  │              CAPABILITY-BASED MICROKERNEL               │    │
│  │   Process Isolation │ IPC │ Scheduler (work-stealing)  │    │
│  └────────────────────────────────────────────────────────┘    │
└────────────────────────┬───────────────────────────────────────┘
                         │ Syscall Interface
┌────────────────────────▼───────────────────────────────────────┐
│                     AGENTIC RUNTIME                             │
│                    (zuupos-agentd)                              │
│                                                                │
│  ┌─────────────────┐  ┌──────────────────┐  ┌──────────────┐  │
│  │   LLM Engine    │  │   Tool Registry  │  │ Agent Loop   │  │
│  │ (Ollama HTTP /  │  │  • FS operations │  │ • Perceive   │  │
│  │  llama.cpp FFI) │  │  • Process spawn │  │ • Reason     │  │
│  │                 │  │  • Display write │  │ • Plan       │  │
│  │  Phi-4 / Llama  │  │  • Net requests  │  │ • Act        │  │
│  └─────────────────┘  └──────────────────┘  │ • Reflect    │  │
│                                             └──────────────┘  │
│                                                                │
│  ┌────────────────────────────────────────────────────────┐    │
│  │               CONTEXT ENGINE (Long Memory)              │    │
│  │  SQLite Episodic + Pattern Mining + Working Memory     │    │
│  └────────────────────────────────────────────────────────┘    │
└────────────────────────┬───────────────────────────────────────┘
                         │ Framebuffer / Display Protocol
┌────────────────────────▼───────────────────────────────────────┐
│                    AGENTIC SHELL (zuupos-shell)                 │
│              GPU-accelerated Compositor in Rust                │
│                                                                │
│  • NO traditional windows, icons, or menus                    │
│  • AI "Face" avatar (Taby-inspired) lives in a corner         │
│  • Floating "cards" rendered by AI decision: what to show     │
│  • Voice: whisper.cpp for STT, espeak-ng for TTS             │
│  • Dynamic layout: AI decides what surfaces to render         │
└────────────────────────────────────────────────────────────────┘
```

---

## Repository Layout

```
zuupos/
├── bootloader/          # UEFI bootloader (Rust, uefi-rs)
│   ├── src/main.rs
│   └── Cargo.toml
├── kernel/              # Microkernel (Rust, no_std)
│   ├── src/
│   │   ├── main.rs      # Kernel entry point
│   │   ├── arch/        # x86-64 specific (GDT, IDT, paging, PIT)
│   │   ├── mm/          # Memory management (PMM buddy, VMM 4-level)
│   │   ├── sched/       # Work-stealing scheduler
│   │   ├── ipc/         # Inter-process communication
│   │   └── drivers/     # Core drivers (VGA, UART, PS2, PCI, NVMe)
│   └── Cargo.toml
├── agentd/              # Agentic runtime daemon
│   ├── src/
│   │   ├── main.rs
│   │   ├── agent/       # Core ReAct loop
│   │   ├── llm/         # LLM backend (Ollama / llama.cpp FFI / fallback)
│   │   ├── planner/     # Multi-step task planner
│   │   ├── tools/       # Tool implementations
│   │   ├── context/     # SQLite episodic memory + pattern mining
│   │   ├── sensors/     # Hardware telemetry (CPU, RAM, load)
│   │   └── voice/       # whisper.cpp + TTS stubs
│   └── Cargo.toml
├── shell/               # Agentic visual shell
│   ├── src/
│   │   ├── main.rs
│   │   ├── compositor/  # Framebuffer compositor (winit 0.30 ApplicationHandler)
│   │   ├── renderer/    # Softbuffer 2D renderer
│   │   ├── avatar/      # AI face animations (Taby-style)
│   │   ├── cards/       # Dynamic UI card system
│   │   └── ipc/         # Unix domain socket IPC
│   └── Cargo.toml
├── sdk/                 # Developer SDK for building OS apps
│   ├── src/
│   │   ├── lib.rs
│   │   ├── capabilities.rs  # Capability tokens & CapabilitySet
│   │   ├── app.rs           # App manifest & lifecycle
│   │   ├── tools.rs         # Tool registration & JSON Schema
│   │   └── ui.rs            # Card/Widget primitives & ShellMessage
│   └── Cargo.toml
├── tools/               # Build tools, QEMU runner, disk image creator
│   ├── run.sh
│   ├── run-demo.sh
│   └── build-iso.sh
├── Cargo.toml           # Workspace root
└── README.md
```

---

## Core Design Principles

### 1. AI-First, Not AI-Added
The LLM is not a feature. It's the process manager, the shell, and the UX layer. Every user-facing decision is made by the agent.

### 2. Capability-Based Security
No ambient authority. Every process holds explicit capability tokens to access resources. The AI agent holds the root capability set and delegates to child agents.

### 3. Event-Driven Agent Loop
```
PERCEIVE → REASON → PLAN → ACT → REFLECT → (loop)
```
The agent daemon receives a continuous stream of system events (hardware interrupts, timer ticks, user input, network events) and decides autonomously what to do.

### 4. Local, Private AI
No cloud dependency. The LLM runs on-device using:
- `llama.cpp` via FFI for larger GGUF models (Llama 3 8B, Mistral 7B)
- `Ollama` HTTP API for development/demo
- Autonomous time-aware fallback when offline

### 5. Persistent Context
The OS maintains a long-horizon memory:
- **Episodic memory**: SQLite-backed session history
- **Semantic memory**: Behavioral pattern mining from tool usage
- **Working memory**: Current task stack + active context window

---

## Phase 1 Roadmap (Current Focus)

| Phase | Goal | Status |
|-------|------|--------|
| **1.1** | Bootloader: UEFI app compiled for `x86_64-unknown-uefi` | ✅ Done |
| **1.2** | Kernel: GDT/IDT/interrupts, PMM (buddy), VMM (4-level paging) | ✅ Done |
| **1.3** | Kernel: Work-stealing scheduler + PIT timer + preemption | ✅ Done |
| **1.4** | Kernel: PCI enumeration, NVMe skeleton, PS/2 keyboard, GOP framebuffer | ✅ Done |
| **1.5** | Agentd: ReAct loop + Ollama + llama.cpp FFI + fallback | ✅ Done |
| **1.6** | Agentd: Multi-step Task Planner | ✅ Done |
| **1.7** | Agentd: Hardware Telemetry Sensors | ✅ Done |
| **1.8** | Agentd: SQLite Episodic Memory & pattern mining | ✅ Done |
| **1.9** | Shell: Softbuffer compositor + avatar + fontdue | ✅ Done |
| **1.10** | Shell: Live CPU/RAM HUD + animated Task Plan panel | ✅ Done |
| **1.11** | IPC: Unix domain socket bridge | ✅ Done |
| **1.12** | Host Demo: Full end-to-end macOS agentic OS preview | ✅ Done |
| **1.13** | SDK: Capability-based IPC, tool calling framework, UI primitives | ✅ Done |

---

## Running the Live Demo

You can run the full host-native demo directly on your macOS machine right now:

```bash
# Ensure local Ollama is running (optional — autonomous fallback active if offline)
ollama run llama3.2:latest

# Run the complete ZuuposOS host preview (shell compositor + agent daemon)
./tools/run-demo.sh
```

Or run the components independently:

```bash
# Terminal 1: Run the compositor shell
cargo run -p zuupos-shell

# Terminal 2: Run the agent runtime daemon (PID 1)
cargo run -p zuupos-agentd
```

### Building the Bare-Metal Kernel

```bash
# Build kernel for x86_64 bare metal
cargo build -p zuupos-kernel --target x86_64-unknown-none

# Build UEFI bootloader
cargo build -p zuupos-boot --target x86_64-unknown-uefi
```

---

## SDK Usage

Add `zuupos-sdk` to your `Cargo.toml` and build capability-bound apps:

```rust
use zuupos_sdk::prelude::*;

let manifest = AppManifest::new("my-app", "1.0.0")
    .with_description("An AI-native app")
    .require_capability(CapabilityType::CallLlm)
    .require_capability(CapabilityType::ShowCard);

let mut app = App::new(manifest);
app.register()?;
let pid = app.spawn()?;

// Register tools the agent can call
let mut tools = ToolRegistry::new();
tools.register_function(
    "greet",
    "Say hello to the user",
    tool_params! {
        "name" => { type: "string", desc: "User's name", required: true }
    },
    |args| Ok(json!({ "message": format!("Hello, {}!", args["name"]) }))
);
```

---

## Inspiration

- **[Redox OS](https://www.redox-os.org/)** — Rust microkernel reference
- **[Taby](https://www.heytaby.com/)** — AI personality & agentic UX inspiration
- **[SerenityOS](https://serenityos.org/)** — ground-up OS design philosophy
- **[Theseus OS](https://www.theseus-os.com/)** — intra-lingual OS (Rust only) research
- **[Phil-Opp's Blog OS](https://os.phil-opp.com/)** — best Rust bare-metal tutorial

---

*ZuuposOS — The OS that thinks for itself.*