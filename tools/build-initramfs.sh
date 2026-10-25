#!/bin/bash
# Build initramfs.cpio for ZuuposOS
# Usage: ./build-initramfs.sh <output.cpio>

set -euo pipefail

export PATH="$HOME/.cargo/bin:$PATH"

OUTPUT="${1:-/Users/zylon/Documents/zuupos/target/initramfs.cpio}"
BUILD_DIR="/Users/zylon/Documents/zuupos/userspace"
HELLO_BIN="/Users/zylon/Documents/zuupos/target/x86_64-unknown-none/release/zuupos-hello"

echo "=== Building ZuuposOS initramfs ==="

# Build userspace programs
echo "Building hello program..."
cd "$BUILD_DIR/hello"
cargo build --release --target x86_64-unknown-none 2>&1 | tail -5

# Create initramfs directory structure
INITRAMFS_DIR=$(mktemp -d)
mkdir -p "$INITRAMFS_DIR/bin"
mkdir -p "$INITRAMFS_DIR/etc"
mkdir -p "$INITRAMFS_DIR/lib"

# Copy hello binary
cp "$HELLO_BIN" "$INITRAMFS_DIR/bin/hello"

# Create a simple init script
cat > "$INITRAMFS_DIR/init" << 'EOF'
#!/bin/sh
echo "ZuuposOS initramfs init starting..."
exec /bin/hello
EOF
chmod +x "$INITRAMFS_DIR/init"

# Create cpio archive
echo "Creating cpio archive..."
cd "$INITRAMFS_DIR"
find . -print0 | cpio --null -ov --format=newc > "$OUTPUT"

echo "Initramfs created: $OUTPUT"
ls -la "$OUTPUT"

# Cleanup
rm -rf "$INITRAMFS_DIR"