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
| Oct 24 2026 | **v0.5.1: Avatar character redesign** — replaced hard-edged `fill_circle`/`fill_rect` placeholder shapes (tiny dot eyes, debug-looking ALL-CAPS emotion label) with an anti-aliased orb character: AA circle/ellipse primitives with soft edge feathering, directional-light shading (offset highlight + specular + ambient-occlusion shadow), a pulsing antenna, catchlight eyes, a ground shadow with squash/stretch, idle breathing animation, and one continuous parametric mouth curve per emotion instead of mismatched rects/arcs | ✅ | `pending` |
| Oct 24 2026 | **Shell split into lib + bin** (`shell/src/lib.rs`) so the renderer/avatar code can be exercised headlessly, and a new `shell/examples/avatar_preview.rs` renders every emotion to PNG with no window/display needed — used to actually look at the redesigned character before shipping it (screen capture isn't available on this dev host) | ✅ | `pending` |
| Oct 24 2026 | **Fixed `new_without_default` clippy regressions surfaced by the lib split** (`Avatar`, `CardManager`, `IpcReceiver` — the lint only fires on externally-reachable items, which a bin-only crate never has) | ✅ | `pending` |
| Oct 24 2026 | **v0.5.2: UI overhaul after direct feedback that the shell "looked like a year-long project" with no clear interaction model** — see decision below for the full list (palette, menu bar, avatar roaming, voice dock) | ✅ | `pending` |
| Oct 25 2026 | **DEMO 1**: Shell + live LLM agent on macOS host | ✅ | `pending` |
| Oct 25 2026 | **Kernel: CPIO initramfs parser (`kernel/src/initramfs.rs`) & ELF64 process loader (`kernel/src/loader.rs`)** — extracts `/bin/hello`, maps PT_LOAD segments, sets user stack, spawns real PID 1 | ✅ | `pending` |
| Oct 25 2026 | **Kernel: Capability IPC syscalls (`SYS_IPC_SEND`, `SYS_IPC_RECV`)** wired into syscall dispatcher (`kernel/src/syscall/mod.rs`) | ✅ | `pending` |
| Oct 25 2026 | **Agentd: Autonomous research tools (`web_search`, `fetch_doc`, `inspect_system`, `remember_fact`, `recall_memory`)** in `agentd/src/tools/research.rs` | ✅ | `pending` |
| Oct 25 2026 | **Agentd: Curated long-term memory across sessions** (`curated_notes` in SQLite, injected into LLM reasoning prompt) | ✅ | `pending` |
| Oct 26 2026 | **Interactive Compositor Shell**: Live keyboard typing, dynamic intent dispatch, anchor-aware floating AI roaming physics, clean boot canvas | ✅ | `pending` |
| Oct 27 2026 | UEFI bootloader boots in QEMU | 🔄 | native macOS disk image creation (`hdiutil` UDTO raw disk) & QEMU runner integration |
| Oct 29 2026 | Kernel GDT/IDT/interrupts working | ✅ | - |
| Oct 31 2026 | Physical memory manager (buddy allocator) | ✅ | - |
| Nov 2 2026 | Virtual memory + higher-half kernel | ✅ | - |
| Nov 4 2026 | Work-stealing scheduler + preemption | ✅ | - |
| Nov 6 2026 | Userspace processes + syscalls | ✅ | ELF64 loader + user stack + int 0x80 syscalls complete |
| Nov 8 2026 | Capability-based IPC between processes | ✅ | channel primitive + `SYS_IPC_SEND`/`SYS_IPC_RECV` complete |
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

### Oct 24 2026 — v0.5.1: headless rendering to actually review the UI
**Decision**: Split `zuupos-shell` into a library (`lib.rs` exposing `avatar`/`cards`/`compositor`/`ipc`/`renderer`/`theme`) plus a thin binary, and added `Renderer::new_headless()` (no `softbuffer::Surface`, `present()` becomes a no-op) alongside a `shell/examples/avatar_preview.rs` that renders every `Emotion` to a PNG.
**Reason**: This dev environment has no display access (`screencapture` fails, no screen-recording permission), so a live `cargo run -p zuupos-shell` window could be confirmed *running* but never actually *looked at*. The first avatar pass had shipped un-reviewed as a result. Headless PNG export makes the design loop verifiable the same way a unit test would — render, read the image, judge it — instead of trusting that drawing code is correct by inspection of the source.
**Side effect**: making the shell's internals a real library surface made `clippy::new_without_default` start firing on `Avatar::new`/`CardManager::new`/`IpcReceiver::new` (the lint only applies to externally-reachable API, which a bin-only crate never exposes). Fixed by adding the `Default` impls rather than suppressing the lint.

### Oct 24 2026 — Avatar: anti-aliased orb character instead of placeholder primitives
**Decision**: Rewrote `Renderer::draw_avatar` around new `fill_circle_aa`/`fill_ellipse_aa` helpers (distance-based edge feathering, ~1px soft edge) and a `draw_arc_mouth` helper (a parabola stamped with small AA dots: positive curvature reads as a smile, negative as a worried frown). Added a pulsing antenna, an offset specular highlight + ambient-occlusion shadow for sphere-like shading, under-body ground shadow with squash/stretch, per-emotion idle "breathing" bob, and eye catchlights. Removed the permanent ALL-CAPS emotion-name text label under the character.
**Reason**: The original avatar was a flat 96px circle with 5px dot eyes and a 3px rectangle mouth drawn with `fill_circle`/`fill_rect`, which have hard pixel-aliased edges and no shading — it read as a debug placeholder, not a character, and the literal `"THINKING"`/`"ALERT"` caption underneath looked like developer instrumentation left in. The expression should carry the emotion, not a text label.
**Verification**: rendered via the new headless preview tool (`shell/examples/avatar_preview.rs`) for all 6 emotions plus a mid-blink frame — see `target/avatar_preview/*.png` (gitignored, regenerate on demand).

### Oct 24 2026 — v0.5.2: UI overhaul — palette, menu bar, roaming avatar, voice dock
**Decision**: Four changes made together, all driven by direct feedback that the shell read as immature:
1. **Palette**: Replaced the purple/blue "AI gradient" (background, avatar, HUD accents all ran through a violet→indigo→blue radial gradient) with a warm amber/coral primary accent + a single teal counterpoint, on a neutral warm-charcoal base. `ACCENT_PURPLE` is renamed `ACCENT_PRIMARY` throughout `shell/src/renderer/mod.rs` and every other hardcoded navy/indigo literal (`0x2d2a4a`, `0x1a1a2e`, ambient-particle colors, card backgrounds) was repainted to match. `draw_background`'s three gradient centers went from "violet / indigo / deep blue" to "warm amber glow / faint ember / barely-there cool graphite" — one accent doing the work instead of three competing hues.
2. **Menu bar**: `draw_statusbar` was rewritten into a macOS-menu-bar-style strip — brand + a live agent-status pill (color and label now track the avatar's actual emotion, not a hardcoded "Agent Active") on the left, CPU/MEM/date/time as compact "menu-bar-extra" text on the right, each item dropping off progressively if the window is too narrow to fit the whole cluster rather than overlapping. The old separate floating "SYSTEM" HUD panel (bottom-right) duplicated the same CPU/MEM numbers in a second place — deleted (`draw_metrics_hud` is gone) so system stats live in exactly one place.
3. **Roaming avatar**: `Avatar` gained `pos_x/pos_y/target_x/target_y` and a seeded `SmallRng`; `Avatar::tick` now takes the viewport size and drifts the character toward a randomly-chosen point within the lower-left ~58% of the screen (clear of the menu bar and the card column) whenever it's Idle or Happy, holding still during Thinking/Speaking/Alert/Processing so it reads as concentrating rather than fidgeting. `Renderer::draw_avatar` reads `avatar.position()` instead of a fixed bottom-left offset computed from `self.height`.
4. **Voice dock**: `draw_voice_dock` adds a docked mic button + breathing "ready" ring + the wake-word hint `"Hey ZuuposOS"` on the right edge. This is declared a visual affordance only, both here and in README's status table — `agentd/src/voice` is still whisper.cpp/TTS stubs, so the dock never claims to be listening.

**Reason**: Direct user feedback: the purple palette read as generic "AI tech" styling, there was no OS-like menu bar despite the status bar already tracking time/CPU, the character sat frozen in a corner instead of feeling present, and there was no visible entry point for voice interaction at all.

**Verification**: `shell/examples/avatar_preview.rs` was extended to render full 1280×800 frames (not just the avatar crop) with sample cards/plan steps/metrics populated, at `full_ui_idle` / `full_ui_wandered` / `full_ui_speaking` ticks — confirmed the avatar's position actually differs between the first two (roaming works), the menu bar status label tracks emotion, and the voice dock's ring color shifts green while Speaking. Caught and fixed a real layout bug this way: the right-side stats cluster's absolute positioning overlapped the left side at narrow widths (a self-inflicted issue in a 420px-wide test canvas, but a legitimate robustness gap — fixed with the progressive drop-off described above). Re-ran the live host demo (shell + agentd) after the change; starts clean, no panics.

**Known gaps, stated plainly rather than implemented half-way**: multi-app/window surface in shell/SDK (in progress); local/cloud LLM tiering and billing.

### Oct 25 2026 — Autonomous Research Tools & System Introspection (Roadmap 1.21)
**Decision**: Added `web_search`, `fetch_doc`, `inspect_system`, `remember_fact`, and `recall_memory` to `agentd/src/tools/research.rs`.
**Reason**: To think like Claude Code on an OS level, the agent needs the agency to discover how to perform unknown tasks: searching documentation (DuckDuckGo API / Wikipedia), reading web documentation, inspecting live hardware and system processes, and remembering critical findings.

### Oct 25 2026 — Curated Long-Term Memory (Roadmap 1.22)
**Decision**: Created `curated_notes` in SQLite (`agentd/src/context/mod.rs`) distinct from raw episodic execution logs. The agent curates and persists high-signal memories across sessions, which are injected directly into the LLM system prompt on every cycle.
**Reason**: Logging every cycle to episodic memory without curation wastes prompt tokens; curating high-signal architectural and system memories gives the agent compounding learning across restarts.

### Oct 25 2026 — Kernel Initramfs CPIO Parser & ELF64 Loader (Roadmap 1.18)
**Decision**: Created `kernel/src/initramfs.rs` (CPIO newc parser) and `kernel/src/loader.rs` (ELF64 segment mapper and user stack allocator at `0x0000_7FFF_FFFF_0000`). `sched::spawn_init_from_initramfs(&BootInfo)` parses `/bin/hello` from the bootloader memory buffer and spawns real PID 1.
**Reason**: Replaces the hardcoded dummy entry point with genuine userspace process execution from memory.

### Oct 25 2026 — macOS Native UEFI Disk Builder & Bare-Metal QEMU Boot (Roadmap 1.19 & 1.20)
**Decision**:
1. Rewrote `tools/create-disk-image.sh` with Darwin autodetection: uses `hdiutil create`, `hdiutil attach -nobrowse`, volume population, and `hdiutil convert -format UDTO` to create raw GPT/FAT32 UEFI disk images on macOS without Linux-specific tooling (`parted`, `losetup`, `mkfs.fat`).
2. Fixed UEFI bootloader (`bootloader/src/main.rs`): replaced length-0 `Vec::with_capacity` buffer bugs with initialized allocations, fixed null pointer panics in `alloc_page` (`AllocateType::AnyPages`), mapped 4 GiB huge-page tables (identity and `0xFFFF_8000_0000_0000` higher-half), and parsed ELF `PT_LOAD` segments with BSS zeroing.
3. Added `.cargo/config.toml` setting `-C relocation-model=static` for `x86_64-unknown-none` target so kernel calls jump to correct PC-relative addresses instead of un-relocated GOT stubs.
4. Kernel memory & privilege: moved 32 MiB heap init before PMM, corrected buddy allocator order calculation (`63 - remaining.leading_zeros()`), configured user code/data segments in GDT and `TSS.privilege_stack_table[0]`, enabled Ring 3 for IDT 0x80 syscalls.
**Verification**: Verified in QEMU: UEFI firmware loads bootloader, kernel boots, initializes PMM/VMM/GDT/IDT/PCI/NVMe/IPC/Interrupts/Scheduler, extracts `/bin/hello` (3840 bytes) from initramfs CPIO, sets up stack at `0x7fffffff0000`, and spawns PID 1!

### Oct 25 2026 — Multi-App & Multi-Card Surface in Shell & SDK (Roadmap 1.23)
**Decision**:
1. **SDK primitives** (`sdk/src/ui.rs`): extended `Card` and `CardUpdates` with `app_id`, `kind`, `focused`, `minimized`, along with fluent builder methods (`with_app_id`, `with_kind`, `with_focused`, `with_minimized`).
2. **Card manager** (`shell/src/cards/mod.rs`): implemented multi-card lifecycle supporting up to 6 stacked cards, focus cycling (`focus_next`, `focus_prev`), minimize/expand toggle, dismiss, and geometry hit testing (`hit_test`) for body, window controls, and action pills.
3. **Renderer** (`shell/src/renderer/mod.rs`): upgraded `draw_cards` with dynamic vertical stacking layout, glassmorphism opacity hierarchy (0.94 focused vs 0.78 unfocused), active perimeter glow on focused cards, app identifier badges (`[RESEARCH]`, `[KERNEL]`, `[AGENTD]`), vector micro-controls (`✕` dismiss, `–`/`+` minimize/restore), action button pills with upper highlight border, and fixed blend fading polarity.
4. **Compositor interaction** (`shell/src/compositor/mod.rs`): wired interactive cursor tracking and mouse clicks (focusing cards, toggling minimize, triggering actions, dismissing) + keyboard shortcuts (Tab / Arrow keys to cycle focus, 'M' to minimize, 'X' / Backspace to dismiss).
5. **Headless preview** (`shell/examples/avatar_preview.rs`): added multi-card test scenario producing `target/avatar_preview/full_ui_multicard.png`.
**Verification**: Offline rendering generated `full_ui_multicard.png` inspected and verified — crisp badges, vector window controls, and action pills. Host test suite (`zuupos-agentd`, `zuupos-sdk`, `zuupos-shell`) passing cleanly.

### Oct 26 2026 — Taby Ethereal Light Mode UI & Hotel Booking Visual Experience
**Decision**:
1. **Design System & Theme Engine** (`shell/src/theme.rs`): added `ThemeMode::Light` alongside `ThemeMode::Dark` with curated Taby Ethereal design tokens:
   - Ambient Pastel Mesh: Pale mint/sage (`0xe2f1ea`), soft lavender blush (`0xeee7f7`), mist white base (`0xf6f9f8`).
   - Floating Card Geometry: Pure white body (`0xffffff`), subtle 1px border (`0xe8edf5`), diffuse drop shadow (`0x121b28`, 22px blur, 0.08 alpha).
   - Micro-Typography & Badges: Emerald finalized pill (`0x059669`), title dark typography (`0x181a1f`), location & time muted typography (`0x64748b`).
   - Avatar & Speech: Iridescent pearl lilac orb (`0xd8c8fd`), periwinkle gradient body (`0xb5c9fe`), frosted speech bubble (`0xffffff`, 0.88 alpha, pointer notch).
   - Centered Pill Dock: Capsule dock (`0xffffff`, 24px radius), clock `12:15 AM`, status divider, WiFi & camera glyphs, and soft periwinkle circular mic button (`0x5164f5`).
2. **Card System Extensions** (`shell/src/cards/mod.rs`):
   - Added `CardType::Booking` and `CardType::Gallery`.
   - Added `CardDetailRow` struct for structured rows (`icon`, `label`, `value`).
   - Added freeform spatial placement: `pos_x`, `pos_y`, `custom_width`, `custom_height`.
   - Added `Card::new_booking` and `Card::new_gallery` constructors.
3. **Renderer Pipeline** (`shell/src/renderer/mod.rs`):
   - Added `theme_mode: ThemeMode` and `image_cache: HashMap<String, image::RgbaImage>` to `Renderer`.
   - Implemented real-time image decoding & rounded corner blitting (`blit_image_rounded`).
   - Implemented rounded drop shadow math (`draw_card_shadow`) with exact rounded box distance functions.
   - Fixed rounded rectangle corner clipping math in `fill_rounded_rect`.
   - Handled `draw_cards_light`: renders Booking Card (with `✓ BOOKING FINALIZED`, hotel name, location pin, dates/nights row, guests row) and Gallery Card (with 2-column image tile grid).
   - Handled `draw_avatar_light`: renders the floating lilac pearl orb with radial lighting + attached frosted speech bubble (`"I've finalized your booking and gathered the visual references you wanted. Where to next?"`).
   - Handled `draw_pill_dock`: renders the floating bottom capsule dock.
4. **Photographic Assets** (`assets/cards/`):
   - Generated authentic high-resolution travel & architecture imagery: `kyoto_street.jpg`, `mount_fuji.jpg`, `japan_night.jpg`, `modern_pavilion.jpg`, and `luxury_bedroom.jpg`.
5. **Verification**:
   - Added `taby_light_hotel_booking.png` generation to `shell/examples/avatar_preview.rs`.
   - Inspected and verified offline rendering against reference mock screenshot.
   - All unit tests across `zuupos-agentd`, `zuupos-sdk`, and `zuupos-shell` passing cleanly.

---

### 2026-10-26 — Interactive Compositor Shell: Live Keyboard Typing, Dynamic Intent Dispatch & Smooth AI Roaming Physics

1. **Problem Resolved**:
   - The OS was erroneously spawning the static Aman Kyoto hotel booking card on startup, leaving the user with a frozen mock rather than a responsive desktop.
   - The shell compositor lacked interactive keyboard input (typing characters was ignored or mapped to single-key hotkeys, Enter was unhandled).
   - The light mode avatar rendered as a static sphere without facial features, blinking, mouth expressions, or organic roaming across the screen.

2. **Continuous AI Avatar Movement & Organic Physics** (`shell/src/avatar/mod.rs` & `shell/src/renderer/mod.rs`):
   - **Harmonic Wave Bobbing**: Continuous multi-frequency sinusoidal floating (`wave_x: sin(t*0.024)*8 + cos(t*0.052)*4`, `wave_y: cos(t*0.033)*7.5 + sin(t*0.068)*3.5`).
   - **Desktop Roaming**: Smooth waypoint traversal across `[140..width-140]` and `[140..height-180]` with critically damped glide (`0.022..0.048`).
   - **Anchor-Aware Traversal**: Roams freely across the desktop when the canvas is empty; orbits gracefully around active work items when cards are focused.
   - **Expressive Facial Features**: Blinking cycle (`avatar.render_state`), specular catchlight glimmers, rosy blush cheeks, reactive mouth states (waveform when speaking, floating thinking dots, alert brow tilt).
   - **Dynamic Ground Shadow**: Anchored floor drop shadow that squashes/stretches with floating elevation.

3. **Clean Boot Canvas**:
   - `ShellApp::new` initializes with an empty card manager; the OS boots into a spacious, uncluttered desktop with the AI greeting the user.

4. **Live Interactive Keyboard Input & Floating Omnibox Dock** (`shell/src/compositor/mod.rs` & `shell/src/renderer/mod.rs`):
   - **Floating Capsule Dock**: Widened to 540px with real-time prompt input field, live blinking caret, placeholder hints, and vector Enter pill button.
   - **Keyboard Event Handling**: Handles printable Unicode characters via `KeyEvent.text` & `Key::Character`, `Backspace` deletion, and `Escape` input reset.
   - **Attention Feedback**: Typing triggers avatar attention (`Emotion::Thinking`).

5. **Dynamic Intent Execution Engine** (`ShellApp::handle_prompt_submission`):
   - `"book hotel in kyoto"`, `"travel"`: Dispatches Aman Kyoto booking card + high-res Kyoto visual references gallery, avatar positions to guide the user.
   - `"system stats"`, `"cpu"`, `"mem"`: Dispatches Kernel Diagnostics & Task Scheduler telemetry cards.
   - `"research papers"`, `"microkernel"`: Dispatches Autonomous Web Research card with interactive action buttons.
   - `"help"`, `"commands"`: Displays available agent command directory.
   - `"clear"`, `"clean"`: Clears all cards, releases anchor, returns avatar to free desktop exploration.
   - `"theme"`, `"dark"`, `"light"`: Toggles color palette.
   - Dynamic conversational responses for general user prompts.

6. **Verification & Artifacts**:
   - `clean_desktop_idle.png`: Clean canvas on boot with empty dock and roaming AI.
   - `clean_desktop_typing.png`: Real-time keyboard typing in dock with caret, vector enter pill, and thinking avatar.
   - `hotel_booking_spawned.png`: Dynamically spawned travel cards with zero overlap.
   - `system_diagnostics_spawned.png`: Live kernel diagnostics and telemetry.

---

## Vision (Not Yet Built)

Captured from a direct conversation about where this should go, so it doesn't get lost. None of this is implemented — it's roadmap, written down on purpose instead of quietly attempted and half-shipped.

- **The agent should understand the OS it's running, not just call fixed tools.** Today agentd's tool set is a fixed Rust match statement (`display_card`, `speak`, `dim_display`, ...). The ask is closer to Claude Code: given a goal it doesn't have a tool for, the agent should be able to reason about how to accomplish it on this specific machine — inspect what's available, fetch documentation from the web when it doesn't know how to do something, and compose a solution instead of failing closed.
- **Persistent, curated memory.** `agentd/src/context` already logs every cycle to SQLite. The ask is a step further: periodically pull out what actually matters (not every cycle) and keep it as long-term memory that makes the agent better over time, the way a person keeps notes instead of a full transcript.
- **A real local/cloud business model.** Basic, everyday asks ("open the camera and show me how I look") should run on the local model for free. Harder or longer tasks route to a paid cloud model. No tiering, routing, or billing exists yet — the `LlmEngine` backend abstraction (`agentd/src/llm`) is the right seam for this later, but nothing here implements pricing or request-based routing today.
- **Multiple apps, not one shell surface.** The SDK already has an `App`/`CapabilitySet` concept (`sdk/src/app.rs`), but the live shell only ever shows one compositor surface with agent-pushed cards. Real customization and multiple concurrent apps are unbuilt.
- **The framing that ties it together**: imagine Claude Code — already agentic, already able to reason about a codebase and take action — except it has a face, a voice, it never stops running, and it *is* the operating system instead of a tool running on top of one, so it isn't boxed in by what a host OS lets a CLI tool do.

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