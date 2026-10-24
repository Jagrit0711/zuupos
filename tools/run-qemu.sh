#!/bin/bash
# Build and run ZuuposOS in QEMU
# Usage: ./run-qemu.sh [debug|release]

set -euo pipefail

MODE="${1:-debug}"
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BUILD_DIR="$PROJECT_ROOT/target"
DISK_IMG="$BUILD_DIR/zuupos.img"
KERNEL_ELF="$BUILD_DIR/x86_64-unknown-none/$MODE/zuupos-kernel"
BOOTLOADER_EFI="$BUILD_DIR/x86_64-unknown-uefi/$MODE/zuupos-boot.efi"
OVMF_CODE="/usr/share/edk2/ovmf/OVMF_CODE.fd"
OVMF_VARS="/usr/share/edk2/ovmf/OVMF_VARS.fd"

echo "=== ZuuposOS QEMU Runner ==="
echo "Mode: $MODE"
echo "Project: $PROJECT_ROOT"

# Build kernel
echo "Building kernel..."
cd "$PROJECT_ROOT"
cargo build -p zuupos-kernel --target x86_64-unknown-none --$MODE 2>&1 | tail -20

# Build bootloader
echo "Building bootloader..."
cargo build -p zuupos-boot --target x86_64-unknown-uefi --$MODE 2>&1 | tail -10

# Verify artifacts exist
if [[ ! -f "$KERNEL_ELF" ]]; then
    echo "ERROR: Kernel not found at $KERNEL_ELF"
    exit 1
fi
if [[ ! -f "$BOOTLOADER_EFI" ]]; then
    echo "ERROR: Bootloader not found at $BOOTLOADER_EFI"
    exit 1
fi

# Create disk image
echo "Creating disk image..."
"$PROJECT_ROOT/tools/create-disk-image.sh" "$DISK_IMG" "$BOOTLOADER_EFI" "$KERNEL_ELF"

# Check for OVMF
if [[ ! -f "$OVMF_CODE" ]]; then
    # Try alternative locations
    for path in \
        "/usr/share/edk2/ovmf/OVMF_CODE.fd" \
        "/usr/share/OVMF/OVMF_CODE.fd" \
        "/usr/share/qemu/OVMF_CODE.fd" \
        "/opt/homebrew/share/qemu/edk2-aarch64-code.fd" \
        "/usr/local/share/qemu/edk2-x86_64-code.fd"; do
        if [[ -f "$path" ]]; then
            OVMF_CODE="$path"
            OVMF_VARS="${path/CODE/VARS}"
            break
        fi
    done
fi

if [[ ! -f "$OVMF_CODE" ]]; then
    echo "ERROR: OVMF_CODE.fd not found. Install edk2-ovmf package."
    echo "  Ubuntu/Debian: apt install ovmf"
    echo "  Fedora: dnf install edk2-ovmf"
    echo "  Arch: pacman -S edk2-ovmf"
    echo "  macOS: brew install qemu (includes edk2)"
    exit 1
fi

# Pick an accelerator/CPU/display that actually exists on this host.
# Apple Silicon hosts run x86_64 QEMU purely under TCG (no KVM/HVF across
# architectures), so "-cpu host" and "-accel kvm" are Linux/x86-host-only.
ACCEL="tcg"
CPU="qemu64"
DISPLAY_BACKEND="${ZUUPOS_QEMU_DISPLAY:-none}"
case "$(uname -s)" in
    Linux)
        if [[ -e /dev/kvm ]]; then ACCEL="kvm:tcg"; CPU="host"; fi
        DISPLAY_BACKEND="${ZUUPOS_QEMU_DISPLAY:-gtk}"
        ;;
    Darwin)
        if [[ "$(uname -m)" == "x86_64" ]]; then ACCEL="hvf:tcg"; CPU="host"; fi
        DISPLAY_BACKEND="${ZUUPOS_QEMU_DISPLAY:-cocoa}"
        ;;
esac

echo "Starting QEMU..."
echo "  Disk: $DISK_IMG"
echo "  Kernel: $KERNEL_ELF"
echo "  Bootloader: $BOOTLOADER_EFI"
echo "  OVMF: $OVMF_CODE"
echo "  Accel: $ACCEL | CPU: $CPU | Display: $DISPLAY_BACKEND"

# Run QEMU
exec qemu-system-x86_64 \
    -machine q35,accel="$ACCEL" \
    -cpu "$CPU" \
    -m 2G \
    -smp 4 \
    -drive if=pflash,format=raw,readonly=on,file="$OVMF_CODE" \
    -drive if=pflash,format=raw,file="$OVMF_VARS" \
    -drive if=virtio,format=raw,file="$DISK_IMG" \
    -serial stdio \
    -display "$DISPLAY_BACKEND" \
    -device virtio-gpu-pci \
    -device virtio-keyboard-pci \
    -device virtio-mouse-pci \
    -netdev user,id=net0 \
    -device virtio-net-pci,netdev=net0 \
    -monitor telnet:127.0.0.1:5555,server,nowait \
    -no-reboot \
    -d int,cpu_reset \
    -D "$BUILD_DIR/qemu.log"