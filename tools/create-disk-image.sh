#!/bin/bash
# Create a FAT32 disk image with EFI bootloader, kernel, and initramfs
# Usage: ./create-disk-image.sh <output.img> <bootloader.efi> <kernel.elf> [initramfs.cpio]

set -euo pipefail

DISK_IMG="${1:?Disk image path required}"
BOOTLOADER="${2:?Bootloader EFI required}"
KERNEL="${3:?Kernel ELF required}"
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
INITRAMFS="${4:-$PROJECT_ROOT/target/initramfs.cpio}"

DISK_SIZE_MB=128

echo "Creating disk image: $DISK_IMG (${DISK_SIZE_MB}MB)"
mkdir -p "$(dirname "$DISK_IMG")"

populate_efi_volume() {
    local mnt="$1"
    mkdir -p "$mnt/EFI/BOOT"
    mkdir -p "$mnt/EFI/ZUUPOS"

    echo "Copying bootloader to $mnt/EFI/BOOT/BOOTX64.EFI..."
    cp "$BOOTLOADER" "$mnt/EFI/BOOT/BOOTX64.EFI"
    cp "$BOOTLOADER" "$mnt/EFI/ZUUPOS/BOOT.EFI"

    echo "Copying kernel to $mnt/EFI/ZUUPOS/KERNEL.ELF..."
    cp "$KERNEL" "$mnt/EFI/ZUUPOS/KERNEL.ELF"

    if [[ -f "$INITRAMFS" ]]; then
        echo "Copying initramfs to $mnt/EFI/ZUUPOS/INITRAMFS.CPIO..."
        cp "$INITRAMFS" "$mnt/EFI/ZUUPOS/INITRAMFS.CPIO"
    fi

    echo "Writing startup.nsh..."
    cat > "$mnt/startup.nsh" << 'EOF'
@echo -off
echo "ZuuposOS Boot Manager"
echo "Starting ZuuposOS Bootloader..."
fs0:\EFI\BOOT\BOOTX64.EFI
EOF
}

case "$(uname -s)" in
    Darwin)
        echo "Host detected: macOS (Darwin)"
        TMP_DMG="$(dirname "$DISK_IMG")/zuupos_tmp_$$.dmg"
        CDR_IMG="${TMP_DMG%.dmg}.cdr"
        rm -f "$TMP_DMG" "$CDR_IMG" "$DISK_IMG"

        # Create GPT partition table with FAT32 EFI partition
        hdiutil create -size "${DISK_SIZE_MB}m" \
            -fs "MS-DOS FAT32" \
            -volname "ZUUPOS" \
            -layout "GPTSPUD" \
            "$TMP_DMG" >/dev/null

        # Attach image without opening Finder
        ATTACH_OUT="$(hdiutil attach -nobrowse "$TMP_DMG")"
        DEV_NODE="$(echo "$ATTACH_OUT" | head -n 1 | awk '{print $1}')"
        MOUNT_POINT="$(echo "$ATTACH_OUT" | grep "/Volumes/" | awk '{print $NF}')"

        if [[ -z "$MOUNT_POINT" || ! -d "$MOUNT_POINT" ]]; then
            MOUNT_POINT="/Volumes/ZUUPOS"
        fi

        echo "Mounted at $MOUNT_POINT (device: $DEV_NODE)"
        populate_efi_volume "$MOUNT_POINT"

        sync
        hdiutil detach "$DEV_NODE" -force >/dev/null

        # Convert to raw sector disk image (.cdr -> .img)
        echo "Converting to raw disk image..."
        hdiutil convert "$TMP_DMG" -format UDTO -o "${TMP_DMG%.dmg}" >/dev/null
        mv "$CDR_IMG" "$DISK_IMG"
        rm -f "$TMP_DMG"
        ;;

    Linux)
        echo "Host detected: Linux"
        rm -f "$DISK_IMG"
        dd if=/dev/zero of="$DISK_IMG" bs=1M count="$DISK_SIZE_MB" status=none

        # Create GPT partition table with ESP
        parted -s "$DISK_IMG" \
            mklabel gpt \
            mkpart ESP fat32 1MiB "$((DISK_SIZE_MB - 1))MiB" \
            set 1 esp on \
            name 1 "EFI System Partition"

        LOOP_DEV=$(losetup --find --show --partscan "$DISK_IMG")
        ESP_PART="${LOOP_DEV}p1"
        sleep 1

        mkfs.fat -F32 -n "ZUUPOS" "$ESP_PART" >/dev/null

        MOUNT_POINT=$(mktemp -d)
        mount "$ESP_PART" "$MOUNT_POINT"

        populate_efi_volume "$MOUNT_POINT"

        sync
        umount "$MOUNT_POINT"
        rmdir "$MOUNT_POINT"
        losetup -d "$LOOP_DEV"
        ;;

    *)
        echo "Unsupported OS: $(uname -s)"
        exit 1
        ;;
esac

echo "Disk image successfully created: $DISK_IMG"