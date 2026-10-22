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
│  │ (candle-rs /    │  │  • FS operations │  │ • Perceive   │  │
│  │  llama.cpp FFI) │  │  • Process spawn │  │ • Reason     │  │
│  │                 │  │  • Display write │  │ • Act        │  │
│  │  Phi-4 / Llama  │  │  • Net requests  │  │ • Reflect    │  │
│  └─────────────────┘  └──────────────────┘  └──────────────┘  │
│                                                                │
│  ┌────────────────────────────────────────────────────────┐    │
│  │               CONTEXT ENGINE (Long Memory)              │    │
│  │  Vector store (custom HNSW) + Event stream + Episodic  │    │
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
│   │   ├── arch/        # x86-64 specific (GDT, IDT, paging)
│   │   ├── mm/          # Memory management
│   │   ├── sched/       # Scheduler
│   │   ├── ipc/         # Inter-process communication
│   │   └── drivers/     # Core drivers (VGA, UART, PS2, PCI)
│   └── Cargo.toml
├── agentd/              # Agentic runtime daemon
│   ├── src/
│   │   ├── main.rs
│   │   ├── agent/       # Core ReAct loop
│   │   ├── llm/         # LLM backend (candle-rs / llama.cpp FFI)
│   │   ├── tools/       # Tool implementations
│   │   ├── context/     # Vector store + memory
│   │   └── voice/       # whisper.cpp + TTS
│   └── Cargo.toml
├── shell/               # Agentic visual shell
│   ├── src/
│   │   ├── main.rs
│   │   ├── compositor/  # Framebuffer compositor
│   │   ├── renderer/    # GPU-accelerated 2D (softbuffer/wgpu)
│   │   ├── avatar/      # AI face animations (Taby-style)
│   │   └── cards/       # Dynamic UI card system
│   └── Cargo.toml
├── sdk/                 # Developer SDK for building OS apps
├── tools/               # Build tools, QEMU runner, disk image creator
│   ├── run.sh
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
PERCEIVE → REASON → ACT → REFLECT → (loop)
```
The agent daemon receives a continuous stream of system events (hardware interrupts, timer ticks, user input, network events) and decides autonomously what to do.

### 4. Local, Private AI
No cloud dependency. The LLM runs on-device using:
- `candle` (pure Rust ML framework by HuggingFace) for smaller models (Phi-4, Gemma)
- `llama.cpp` via FFI for larger GGUF models (Llama 3 8B, Mistral 7B)

### 5. Persistent Context
The OS maintains a long-horizon memory:
- **Episodic memory**: what happened in the last N sessions
- **Semantic memory**: HNSW vector store of user patterns
- **Working memory**: current task stack + active context window

---

## Phase 1 Roadmap (Current Focus)

| Phase | Goal | Status |
|-------|------|--------|
| **1.1** | Bootloader: UEFI app that enters long mode | 🔴 TODO |
| **1.2** | Kernel: GDT/IDT setup, physical memory manager | 🔴 TODO |
| **1.3** | Kernel: Basic scheduler + process isolation | 🔴 TODO |
| **1.4** | Kernel: UART + framebuffer output | 🔴 TODO |
| **1.5** | Agentd: LLM running in userspace on candle-rs | 🔴 TODO |
| **1.6** | Agentd: Basic tool loop (read/write FS, exec) | 🔴 TODO |
| **1.7** | Shell: Framebuffer compositor + avatar renderer | 🔴 TODO |
| **1.8** | Shell: Voice I/O via whisper.cpp + espeak | 🔴 TODO |
| **1.9** | Integration: AI-driven boot experience | 🔴 TODO |

---

## Tech Stack

| Layer | Technology |
|-------|-----------|
| **Language** | Rust (primary), x86-64 Assembly (minimal, for boot/interrupts) |
| **Boot** | UEFI via `uefi` crate |
| **Kernel** | `no_std` Rust, custom allocator, custom scheduler |
| **AI Runtime** | `candle` (HuggingFace, pure Rust) + `llama.cpp` FFI |
| **GPU/Display** | Framebuffer (UEFI GOP) → wgpu when drivers mature |
| **Voice STT** | whisper.cpp via FFI |
| **Voice TTS** | espeak-ng or kokoro-rs |
| **Build** | `cargo`, `cargo-make`, QEMU for emulation |
| **Target** | x86-64-unknown-none (bare metal) |

---

## Build & Run

```bash
# Prerequisites
cargo install cargo-make
# Install QEMU: brew install qemu

# Run in QEMU (full emulation)
cargo make run

# Build disk image only
cargo make build-iso
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
# zuupos
