#!/usr/bin/env bash
# Build a UEFI-bootable disk image for ZuuposOS
set -euo pipefail

mkdir -p build/esp/EFI/BOOT
cp target/x86_64-unknown-uefi/release/zuupos-boot.efi build/esp/EFI/BOOT/BOOTX64.EFI
cp target/x86_64-unknown-none/release/zuupos-kernel build/esp/zuupos-kernel

dd if=/dev/zero of=build/zuupos.img bs=1M count=64
mkfs.vfat -F 32 build/zuupos.img
mcopy -i build/zuupos.img -s build/esp/* ::/
echo "[build-iso] build/zuupos.img created"
