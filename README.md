# ZuuposOS 🧠

> An AI-Native Agentic Operating System built from scratch in Rust.  
> No windows. No icons. No menus. Just an AI that *is* the OS.

**Current version: v0.5.2** — see [RECORD.md](RECORD.md) for the full build log.

---

## Status at a Glance (v0.5.2)

| Layer | State |
|-------|-------|
| Bootloader (`zuupos-boot`) | ✅ Builds for `x86_64-unknown-uefi`, reads kernel + initramfs from disk |
| Kernel (`zuupos-kernel`) | ✅ Builds for `x86_64-unknown-none` — PMM/VMM, work-stealing scheduler, syscall ABI, capability IPC channels, CPIO initramfs extraction, and ELF64 loader mapping real PID 1 |
| Userspace (`userspace/hello`) | ✅ First freestanding userspace binary, builds + packs into `target/initramfs.cpio` via `tools/build-initramfs.sh` |
| Agentd (`zuupos-agentd`) | ✅ Full ReAct loop verified live against Ollama — reasoning → plan → tool-call cycles. Equipped with autonomous web search, doc fetcher, system introspection, and persistent curated memory |
| Shell (`zuupos-shell`) | ✅ Dual-Palette UI Engine: Taby Ethereal Light Mode (pastel mint/lavender ambient mesh, pure white floating cards with diffuse shadows, pearl orb avatar, frosted speech bubbles, centered capsule island dock) and Warm Charcoal Dark Mode (menu bar, system stats, amber accents) |
| Avatar character | ✅ Dual persona: soft iridescent lilac-periwinkle gradient pearl orb with attached speech bubbles (Light Mode) and expressive anti-aliased roaming creature with antenna & eyelids (Dark Mode) — see `shell/examples/avatar_preview.rs` |
| Voice dock | 🔄 Visual affordance: centered floating capsule dock (`12:15 AM | ᯤ 📹 (🎙)`) in Light Mode; right-docked voice button with breathing ring in Dark Mode |
| SDK (`zuupos-sdk`) | ✅ Builds clean, capability tokens + tool registry + UI primitives |
| Multi-app / spatial cards | ✅ Freeform spatial canvas & stacked surface: Booking Cards (`✓ BOOKING FINALIZED`, dates, guests), Visual Reference Gallery Cards (2-column photo tile grids), vector window controls, and focus cycling |
| QEMU boot | ✅ Bare-metal UEFI boot + kernel initialization in QEMU via native macOS disk builder (`hdiutil`/FAT32), higher-half paging, GDT/TSS, and PID 1 ELF load from initramfs |

The fastest way to see it working today is the **host demo** below — it runs the real shell + real agent + real local LLM on your machine right now.

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
│   │   ├── ipc/         # Capability-based IPC channels
│   │   ├── syscall/     # Syscall ABI (exit, yield, getpid, write, mmap, spawn, ipc, fb)
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
│   │   ├── main.rs      # Thin binary entry point
│   │   ├── lib.rs       # Library surface (so examples/tests can render headlessly)
│   │   ├── compositor/  # Framebuffer compositor (winit 0.30 ApplicationHandler)
│   │   ├── renderer/    # Softbuffer 2D renderer — AA primitives, the avatar character
│   │   ├── avatar/      # AI buddy character state (emotion, blink, breathing)
│   │   ├── cards/       # Dynamic UI card system
│   │   └── ipc/         # Unix domain socket IPC
│   ├── examples/
│   │   └── avatar_preview.rs  # Headless PNG export of the character/HUD for review
│   └── Cargo.toml
├── sdk/                 # Developer SDK for building OS apps
│   ├── src/
│   │   ├── lib.rs
│   │   ├── capabilities.rs  # Capability tokens & CapabilitySet
│   │   ├── app.rs           # App manifest & lifecycle
│   │   ├── tools.rs         # Tool registration & JSON Schema
│   │   └── ui.rs            # Card/Widget primitives & ShellMessage
│   └── Cargo.toml
├── userspace/           # Freestanding userspace programs (no_std, raw syscalls)
│   └── hello/           # First userspace binary — packed into the initramfs
│       ├── src/main.rs
│       └── Cargo.toml
├── tools/               # Build tools, QEMU runner, disk image creator
│   ├── run.sh
│   ├── run-demo.sh      # Host demo launcher (shell + agentd, no QEMU needed)
│   ├── run-qemu.sh      # Builds kernel+bootloader, boots the disk image in QEMU
│   ├── create-disk-image.sh
│   └── build-initramfs.sh  # Packs userspace/ binaries into target/initramfs.cpio
├── Cargo.toml           # Workspace root
└── README.md
```

---

## Core Design Principles

### 1. AI-Native, Not AI-First
"AI-first" still implies a product that could exist without the AI and just chose to lead with it. ZuuposOS can't — there is no shell, no process manager, no UX layer that exists independent of the agent. The LLM *is* those things. Every user-facing decision is made by the agent, not routed through it.

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
| **1.14** | Kernel: Syscall ABI (int 0x80) + IDT wiring | ✅ Done |
| **1.15** | Kernel: Capability-based IPC channels (send/recv, rights) | ✅ Done |
| **1.16** | Bootloader: initramfs loading from disk into memory | ✅ Done |
| **1.17** | Userspace: first `no_std` program (`zuupos-hello`) + initramfs packer | ✅ Done |
| **1.18** | Kernel: load & exec the initramfs ELF as the real PID 1 | ✅ Done (CPIO parser + ELF64 segment loader + user stack setup in `kernel/src/sched/mod.rs`) |
| **1.19** | Shell: anti-aliased avatar character redesign + headless PNG preview tool | ✅ Done |
| **1.20** | Shell: menu bar (time/stats/live agent status), roaming avatar, voice-input dock, warm palette | ✅ Done |
| **1.21** | Agentd: tool to search the web / fetch docs when it doesn't know how to do something | ✅ Done (`web_search`, `fetch_doc`, `inspect_system` in `agentd/src/tools/research.rs`) |
| **1.22** | Agentd: curate and persist "important" memories across sessions, not just log every cycle | ✅ Done (SQLite `curated_notes` table, persistent recall, injected into prompt context) |
| **1.23** | Shell/SDK: a real multi-app surface (more than one agent-spawned app/window at once) | 🔄 In progress |

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

### Previewing the Avatar / HUD Without a Window

The shell's drawing code is also exposed as a library (`zuupos_shell`), so the
same `draw_avatar`/`draw_*` pipeline can render to an offscreen buffer and be
dumped to PNG — no window, display server, or screen access required. Useful
for iterating on the character/HUD design headlessly:

```bash
cargo run -p zuupos-shell --example avatar_preview
# -> target/avatar_preview/avatar_{idle,thinking,speaking,alert,happy,processing}.png
```

### Building the Bare-Metal Kernel

```bash
# Build kernel for x86_64 bare metal
cargo build -p zuupos-kernel --target x86_64-unknown-none

# Build UEFI bootloader
cargo build -p zuupos-boot --target x86_64-unknown-uefi

# Build the first userspace program and pack it into an initramfs
cargo build -p zuupos-hello --target x86_64-unknown-none --release
./tools/build-initramfs.sh
```

### Booting in QEMU

```bash
./tools/run-qemu.sh
```

`run-qemu.sh` auto-detects the host: on Linux it uses KVM when `/dev/kvm` is
available, on macOS it falls back to pure TCG emulation (Apple Silicon hosts
cannot hardware-accelerate an x86_64 guest). Disk-image creation
(`tools/create-disk-image.sh`) currently relies on Linux-only tooling
(`parted`, `losetup`, `mkfs.fat`); run it on Linux or in a Linux container
until a macOS-native path (`hdiutil`/`diskutil`) is added.

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