#!/usr/bin/env bash
# ZuuposOS — Run in QEMU (Phase 1 development launcher)
set -euo pipefail

COLOR_PURPLE='\033[0;35m'
COLOR_CYAN='\033[0;36m'
COLOR_YELLOW='\033[1;33m'
COLOR_RESET='\033[0m'

echo -e "${COLOR_PURPLE}"
echo "╔══════════════════════════════════════╗"
echo "║       Z U U P O S   O S             ║"
echo "║   AI-Native Agentic OS Runner       ║"
echo "╚══════════════════════════════════════╝"
echo -e "${COLOR_RESET}"

# Check dependencies
check_dep() {
    if ! command -v "$1" &>/dev/null; then
        echo -e "${COLOR_YELLOW}[!] Missing dependency: $1${COLOR_RESET}"
        echo "    Install with: $2"
        exit 1
    fi
}

check_dep qemu-system-x86_64 "brew install qemu"
check_dep cargo "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
check_dep cargo-make "cargo install cargo-make"

echo -e "${COLOR_CYAN}[*] Building bootloader...${COLOR_RESET}"
cd bootloader
cargo build --release --target x86_64-unknown-uefi
cd ..

echo -e "${COLOR_CYAN}[*] Building kernel...${COLOR_RESET}"
cd kernel
cargo +nightly build --release --target x86_64-unknown-none
cd ..

echo -e "${COLOR_CYAN}[*] Creating UEFI disk image...${COLOR_RESET}"
./tools/build-iso.sh

echo -e "${COLOR_CYAN}[*] Launching QEMU...${COLOR_RESET}"
qemu-system-x86_64 \
    -machine q35 \
    -m 4G \
    -smp 4 \
    -cpu host \
    -enable-kvm \
    -bios /usr/share/ovmf/OVMF.fd \
    -drive "format=raw,file=build/zuupos.img" \
    -serial stdio \
    -display sdl \
    -usb \
    -device usb-kbd \
    -device usb-mouse \
    -net nic,model=virtio \
    -net user \
    -no-reboot
