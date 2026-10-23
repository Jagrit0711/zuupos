#!/bin/bash
# Create a FAT32 disk image with EFI bootloader and kernel
# Usage: ./create-disk-image.sh <output.img> <bootloader.efi> <kernel.elf>

set -euo pipefail

DISK_IMG="${1:?Disk image path required}"
BOOTLOADER="${2:?Bootloader EFI required}"
KERNEL="${3:?Kernel ELF required}"

DISK_SIZE_MB=256
ESP_SIZE_MB=128

echo "Creating disk image: $DISK_IMG (${DISK_SIZE_MB}MB)"

# Create empty disk image
dd if=/dev/zero of="$DISK_IMG" bs=1M count="$DISK_SIZE_MB" status=progress

# Create GPT partition table with ESP
parted -s "$DISK_IMG" \
    mklabel gpt \
    mkpart ESP fat32 1MiB "$((ESP_SIZE_MB + 1))MiB" \
    set 1 esp on \
    name 1 "EFI System Partition"

# Setup loop device
LOOP_DEV=$(losetup --find --show --partscan "$DISK_IMG")
echo "Using loop device: $LOOP_DEV"

ESP_PART="${LOOP_DEV}p1"

# Wait for partition to appear
sleep 1

# Format ESP as FAT32
mkfs.fat -F32 -n "ZUUPOS" "$ESP_PART"

# Mount ESP
MOUNT_POINT=$(mktemp -d)
mount "$ESP_PART" "$MOUNT_POINT"

# Create EFI directory structure
mkdir -p "$MOUNT_POINT/EFI/BOOT"
mkdir -p "$MOUNT_POINT/EFI/ZUUPOS"

# Copy bootloader as fallback (BOOTX64.EFI)
cp "$BOOTLOADER" "$MOUNT_POINT/EFI/BOOT/BOOTX64.EFI"

# Copy bootloader and kernel to Zuupos directory
cp "$BOOTLOADER" "$MOUNT_POINT/EFI/ZUUPOS/BOOT.EFI"
cp "$KERNEL" "$MOUNT_POINT/EFI/ZUUPOS/KERNEL.ELF"

# Create startup.nsh for automatic boot
cat > "$MOUNT_POINT/startup.nsh" << 'EOF'
@echo -off
echo "ZuuposOS Boot Manager"
echo "Loading kernel..."
fs0:\EFI\ZUUPOS\BOOT.EFI
EOF

# Create boot entry
efibootmgr --create --disk "$LOOP_DEV" --part 1 --label "ZuuposOS" --loader '\EFI\ZUUPOS\BOOT.EFI' 2>/dev/null || true

# Sync and unmount
sync
umount "$MOUNT_POINT"
rmdir "$MOUNT_POINT"

# Detach loop device
losetup -d "$LOOP_DEV"

echo "Disk image created: $DISK_IMG"