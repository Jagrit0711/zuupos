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
| Oct 22 2026 | UEFI bootloader verified with `x86_64-unknown-uefi` target | ✅ | `pending` |
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
| Oct 25 2026 | **DEMO 1**: Shell + live LLM agent on macOS host | ✅ | `pending` |
| Oct 27 2026 | UEFI bootloader boots in QEMU | 🔄 | in progress |
| Oct 29 2026 | Kernel GDT/IDT/interrupts working | 🔴 | - |
| Oct 31 2026 | Physical memory manager (buddy allocator) | 🔴 | - |
| Nov 2 2026 | Virtual memory + higher-half kernel | 🔴 | - |
| Nov 4 2026 | Round-robin scheduler + preemption | 🔴 | - |
| Nov 6 2026 | Userspace processes + syscalls | 🔴 | - |
| Nov 8 2026 | Capability-based IPC between processes | 🔴 | - |
| Nov 10 2026 | FAT32 filesystem driver | 🔴 | - |
| Nov 13 2026 | Embedded LLM (candle-rs Phi-4 Mini) | 🔴 | - |
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
**Reason**: Memory safety, no_std support, excellent ecosystem (uefi-rs, x86_64, candle, winit).  
**Alternative considered**: C — rejected for safety guarantees.

### Oct 22 2026 — Weekend Demo: Ollama over bare-metal LLM
**Decision**: Use Ollama HTTP API for weekend demo instead of candle-rs embedded.  
**Reason**: candle-rs requires bare-metal environment; demo runs on macOS host. candle-rs will be used in Phase 3 (bare-metal agentd).  
**Migration path**: `LlmEngine` trait abstraction lets us swap backends without changing agent code.

### Oct 22 2026 — Shell: softbuffer + winit (no Wayland/X11)
**Decision**: softbuffer for pixel-perfect CPU rendering on macOS host; direct GOP framebuffer on bare metal.  
**Reason**: Platform-agnostic rendering, same pixel pipeline on host and bare metal.

### Oct 22 2026 — IPC: Unix domain socket (host), kernel IPC channel (bare metal)
**Decision**: Unix socket for host demo, migrate to kernel IPC channel primitive in Phase 3.  
**Reason**: Allows the same agentd ↔ shell protocol to be developed and tested on macOS before bare metal.

---

## Performance Targets

| Metric | Target | Current |
|--------|--------|---------|
| Boot to agent loop | < 3s | N/A |
| LLM first token latency | < 2s (Phi-4 Mini) | N/A |
| Shell frame rate | 60 FPS | N/A |
| Agent loop cycle | < 100ms (excluding LLM) | N/A |
| Memory footprint (agentd) | < 6 GB (LLM included) | N/A |

---

## Dependencies Log

| Crate | Version | Purpose | Phase Added |
|-------|---------|---------|-------------|
| `uefi` | 0.33 | UEFI bootloader | Phase 0 |
| `x86_64` | 0.15 | Kernel arch primitives | Phase 0 |
| `spinning_top` | 0.3 | no_std spinlock | Phase 0 |
| `linked_list_allocator` | 0.10 | Kernel heap | Phase 0 |
| `uart_16550` | 0.3 | Serial console | Phase 0 |
| `winit` | 0.29 | Shell windowing (host) | Phase 0 |
| `softbuffer` | 0.4 | Framebuffer (host) | Phase 0 |
| `fontdue` | 0.9 | Font rendering | Phase 1 |
| `tokio` | 1 | Async runtime (agentd) | Phase 0 |
| `reqwest` | 0.12 | Ollama HTTP client | Phase 1 |
| `serde_json` | 1 | Tool call parsing | Phase 0 |
| `candle-core` | 0.8 | LLM inference (bare metal) | Phase 3 |

---

*This file is updated after every committed milestone.*
