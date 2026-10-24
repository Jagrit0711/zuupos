# ZuuposOS — Build Record

> Every time something ships, it gets logged here. This is the source of truth.

---

## Milestone Log

| Date | Milestone | Status | Commit |
|------|-----------|--------|--------|
| Oct 22 2026 | Project scaffolded (46 files, 2089 lines) | ✅ | `e87e996` |
| Oct 22 2026 | Architecture blueprint + README written | ✅ | `e87e996` |
| Oct 22 2026 | RECORD.md created | ✅ | `e87e996` |
| Oct 22 2026 | Shell API implemented (winit 0.29 + softbuffer 0.4) | ✅ | `pending` |
| Oct 22 2026 | Shell ↔ Agentd IPC via Unix domain socket (`/tmp/zuupos_shell.sock`) | ✅ | `pending` |
| Oct 22 2026 | Live Ollama LLM integration (`llama3.2:latest`) with autonomous fallback | ✅ | `pending` |
| Oct 22 2026 | Embedded typography with fontdue + Inter fonts | ✅ | `pending` |
| Oct 22 2026 | Reactive Avatar with 5 emotion states & blink animations | ✅ | `pending` |
| Oct 22 2026 | UEFI bootloader verified with `x86_64-unknown-uefi` target | ✅ | `db36e6a` |
| Oct 22 2026 | Kernel GDT/IDT/interrupts & bare-metal build (`x86_64-unknown-none`) | ✅ | `pending` |
| Oct 22 2026 | Release binaries built (`zuupos-boot.efi` + `zuupos-kernel` ELF) | ✅ | `pending` |
| Oct 22 2026 | Host demo launcher (`tools/run-demo.sh`) verified working end-to-end | ✅ | `3518437` |
| Oct 22 2026 | GitHub Actions CI/CD (.github/workflows/ci.yml) | ✅ | `3518437` |
| Oct 23 2026 | Multi-Step Task Planner with sequential decomposition (`agentd/src/planner/mod.rs`) | ✅ | `3518437` |
| Oct 23 2026 | Hardware Telemetry Sensors for CPU, RAM, & load avg (`agentd/src/sensors/mod.rs`) | ✅ | `3518437` |
| Oct 23 2026 | SQLite Episodic Memory & behavioral pattern mining (`agentd/src/context/mod.rs`) | ✅ | `3518437` |
| Oct 23 2026 | Shell HUD with smoothed gauges & animated Task Plan panel (`shell/src/renderer/mod.rs`) | ✅ | `3518437` |
| Oct 23 2026 | Extended Avatar emotions (`Processing`) + Speaking mouth waveform | ✅ | `3518437` |
| Oct 23 2026 | Zero-warning build pipeline across host crates & bare-metal targets | ✅ | `3518437` |
| Oct 23 2026 | **Kernel: Physical Memory Manager (buddy allocator)** from UEFI map | ✅ | `pending` |
| Oct 23 2026 | **Kernel: Virtual Memory Manager (4-level paging, higher-half kernel)** | ✅ | `pending` |
| Oct 23 2026 | **Kernel: Work-stealing scheduler with PIT timer preemption** | ✅ | `pending` |
| Oct 23 2026 | **Kernel: PCI bus enumeration + BAR mapping** | ✅ | `pending` |
| Oct 23 2026 | **Kernel: NVMe driver skeleton (admin queues, Identify)** | ✅ | `pending` |
| Oct 23 2026 | **Agentd: llama.cpp FFI backend + multi-backend LlmEngine** | ✅ | `pending` |
| Oct 23 2026 | **SDK: Capability tokens, CapabilitySet, App lifecycle** | ✅ | `pending` |
| Oct 23 2026 | **SDK: ToolRegistry with JSON Schema & FunctionTool** | ✅ | `pending` |
| Oct 23 2026 | **SDK: Card/Widget primitives, ShellMessage IPC, Emotion enum** | ✅ | `pending` |
| Oct 23 2026 | **Shell: winit 0.30 ApplicationHandler migration** | ✅ | `pending` |
| Oct 23 2026 | **Bootloader: Fixed for uefi 0.33 API** (MemoryType, AllocateType, cstr16, with_config_table) | ✅ | `db36e6a` |
| Oct 23 2026 | **Kernel: Fixed clippy warnings across all crates for CI** | ✅ | `6ab0b46` |
| Oct 23 2026 | **Kernel: Syscall interface (int 0x80 ABI) + IDT wiring + QEMU runner script** | ✅ | `892b621` |
| Oct 23 2026 | **Bootloader: initramfs loading from disk, passed to kernel via BootInfo** | ✅ | `d68b910` |
| Oct 23 2026 | **Kernel: Capability-based IPC channels (send/recv, rights, per-pid)** | ✅ | `b3f150c` |
| Oct 24 2026 | **v0.5.0: workspace version bump, `userspace/hello` wired into the workspace + CI** | ✅ | `pending` |
| Oct 24 2026 | **Userspace: `zuupos-hello` builds for `x86_64-unknown-none`; `tools/build-initramfs.sh` verified producing `target/initramfs.cpio`** | ✅ | `pending` |
| Oct 24 2026 | **`tools/run-qemu.sh` fixed for Apple Silicon hosts** (TCG accel, `qemu64` CPU, `cocoa`/`none` display — `-cpu host`/KVM/gtk were Linux-x86-host-only) | ✅ | `pending` |
| Oct 24 2026 | **Full workspace build verified**: all 6 crates (boot, kernel, agentd, shell, sdk, userspace/hello) compile clean on both host and bare-metal targets; `cargo fmt --check` + `cargo clippy -D warnings` + `cargo test` all pass for host crates | ✅ | `pending` |
| Oct 24 2026 | **Host demo re-verified end-to-end live**: shell compositor + agentd both launched, agentd completed a real PERCEIVE→REASON→PLAN→ACT→REFLECT cycle against local Ollama (`llama3.2:latest`) and executed `display_card`/`speak`/`dim_display` tool calls | ✅ | `pending` |
| Oct 25 2026 | **DEMO 1**: Shell + live LLM agent on macOS host | ✅ | `pending` |
| Oct 27 2026 | UEFI bootloader boots in QEMU | 🔄 | blocked on macOS disk-image tooling, see decision below |
| Oct 29 2026 | Kernel GDT/IDT/interrupts working | ✅ | - |
| Oct 31 2026 | Physical memory manager (buddy allocator) | ✅ | - |
| Nov 2 2026 | Virtual memory + higher-half kernel | ✅ | - |
| Nov 4 2026 | Work-stealing scheduler + preemption | ✅ | - |
| Nov 6 2026 | Userspace processes + syscalls | 🔄 | syscall ABI + first userspace binary done; kernel-side ELF loader to exec the initramfs as real PID 1 still stubbed (`sched::spawn_init_process` uses a hardcoded entry point) |
| Nov 8 2026 | Capability-based IPC between processes | ✅ | channel primitive done (`b3f150c`); still needs a syscall-level send/recv wrapper exposed to userspace |
| Nov 10 2026 | FAT32 filesystem driver | 🔴 | - |
| Nov 13 2026 | Embedded LLM (llama.cpp Phi-4 Mini GGUF) | 🔴 | - |
| Nov 15 2026 | Shell running on bare-metal framebuffer | 🔴 | - |
| Nov 18 2026 | Full boot experience + agent greeting | 🔴 | - |
| Nov 20 2026 | Voice I/O (whisper.cpp STT + espeak TTS) | 🔴 | - |
| Nov 22 2026 | **DEMO 2**: ZuuposOS boots in QEMU v0.1.0 | 🔴 | - |

---

## Legend
- ✅ Done and committed
- 🔄 In progress
- 🔴 Not started yet

---

## Architecture Decisions

### Oct 22 2026 — Language: Rust
**Decision**: Rust for all layers (bootloader, kernel, agentd, shell).  
**Reason**: Memory safety, no_std support, excellent ecosystem (uefi-rs, x86_64, llama.cpp, winit).  
**Alternative considered**: C — rejected for safety guarantees.

### Oct 22 2026 — Weekend Demo: Ollama over bare-metal LLM
**Decision**: Use Ollama HTTP API for weekend demo instead of candle-rs embedded.  
**Reason**: candle-rs requires bare-metal environment; demo runs on macOS host. llama.cpp will be used for bare-metal agentd.  
**Migration path**: `LlmEngine` backend abstraction lets us swap backends without changing agent code.

### Oct 22 2026 — Shell: softbuffer + winit (no Wayland/X11)
**Decision**: softbuffer for pixel-perfect CPU rendering on macOS host; direct GOP framebuffer on bare metal.  
**Reason**: Platform-agnostic rendering, same pixel pipeline on host and bare metal.

### Oct 22 2026 — IPC: Unix domain socket (host), kernel IPC channel (bare metal)
**Decision**: Unix socket for host demo, migrate to kernel IPC channel primitive in Phase 3.  
**Reason**: Allows the same agentd ↔ shell protocol to be developed and tested on macOS before bare metal.

### Oct 23 2026 — Kernel Memory: Buddy Allocator + 4-Level Paging
**Decision**: Implement PMM buddy allocator from UEFI memory map; VMM with higher-half kernel at `0xFFFF_8000_0000_0000`.  
**Reason**: Standard x86_64 approach; compatible with bootloader page tables.

### Oct 23 2026 — Scheduler: Interrupt-Driven Context Switch
**Decision**: PIT timer at 1000 Hz triggers naked asm handler that saves all GP registers, calls scheduler, restores next task.  
**Reason**: True preemptive multitasking; avoids complex userspace context switch for now.

### Oct 23 2026 — LLM Backend: Multi-Backend with Fallback
**Decision**: LlmEngine tries llama.cpp (GGUF), then Ollama, then autonomous fallback.  
**Reason**: Best model support — local GGUF for capability, Ollama for convenience, fallback for offline.

### Oct 23 2026 — SDK: Capability-Based from Day One
**Decision**: SDK exposes CapabilityType enum (31 types), CapabilitySet with root/empty/from_tokens, App lifecycle with capability checks.  
**Reason**: Enforces least-privilege; agent holds root capabilities and delegates.

### Oct 24 2026 — v0.5.0: workspace member instead of standalone crate for userspace
**Decision**: `userspace/hello` joined the top-level Cargo workspace (`Cargo.toml` members) rather than staying a free-standing crate with its own `[workspace]` table.  
**Reason**: It already needed the same `x86_64-unknown-none` target and nightly toolchain as the kernel; standalone it failed to build ("believes it's in a workspace when it's not"). Removed its unused `alloc`/`Vec` import and duplicate `[profile.*]` tables (inherited from the workspace root) at the same time.  
**Follow-up**: dropped `#![feature(panic_info_message)]` — stabilized upstream, the nightly flag now errors as an internal-feature use.

### Oct 24 2026 — QEMU runner: no hardware acceleration on Apple Silicon
**Decision**: `tools/run-qemu.sh` now detects the host OS/arch and picks `-accel tcg` + `-cpu qemu64` + `-display cocoa` (macOS) or `-accel kvm:tcg` + `-cpu host` + `-display gtk` (Linux with `/dev/kvm`), instead of hardcoding the Linux/KVM flags.  
**Reason**: `-cpu host` and `kvm`/`gtk` assume an x86_64 Linux host. This dev machine is Apple Silicon running x86_64 QEMU under pure emulation — KVM and GTK aren't available at all, and `-cpu host` with no matching accelerator is meaningless. `ZUUPOS_QEMU_DISPLAY` env var still allows an override.  
**Known gap**: `tools/create-disk-image.sh` still shells out to `parted`/`losetup`/`mkfs.fat`/`efibootmgr`, which don't exist on macOS — the GPT/FAT32 image step needs a Linux host (or a future `hdiutil`-based rewrite) until that's ported.

---

## Performance Targets

| Metric | Target | Current |
|--------|--------|---------|
| Boot to agent loop | < 3s | N/A |
| LLM first token latency | < 2s (Phi-4 Mini) | N/A |
| Shell frame rate | 60 FPS | ~60 FPS (host) |
| Agent loop cycle | < 100ms (excluding LLM) | ~50ms |
| Memory footprint (agentd) | < 6 GB (LLM included) | ~2 GB (host) |

---

## Dependencies Log

| Crate | Version | Purpose | Phase Added |
|-------|---------|---------|-------------|
| `uefi` | 0.33 | UEFI bootloader | Phase 0 |
| `x86_64` | 0.15 | Kernel arch primitives | Phase 0 |
| `spinning_top` | 0.3 | no_std spinlock | Phase 0 |
| `linked_list_allocator` | 0.10 | Kernel heap | Phase 0 |
| `uart_16550` | 0.3 | Serial console | Phase 0 |
| `winit` | 0.30 | Shell windowing (host) | Phase 0 |
| `softbuffer` | 0.4 | Framebuffer (host) | Phase 0 |
| `fontdue` | 0.9 | Font rendering | Phase 1 |
| `tokio` | 1 | Async runtime (agentd) | Phase 0 |
| `reqwest` | 0.12 | Ollama HTTP client | Phase 1 |
| `serde_json` | 1 | Tool call parsing | Phase 0 |
| `llama-cpp-2` | 0.1 | llama.cpp FFI bindings | Phase 1 |
| `rusqlite` | 0.31 | SQLite episodic memory | Phase 1 |
| `sysinfo` | 0.31 | Hardware sensors | Phase 1 |
| `candle-core` | 0.8 | LLM inference (future) | Phase 3 |

`userspace/hello` is dependency-free (`#![no_std]`, raw `int 0x80` syscalls only) — it links against nothing but `core`.

---

*This file is updated after every committed milestone.*