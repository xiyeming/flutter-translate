#!/bin/bash

set -e

echo "=== xym_ft Build Script ==="

PROJECT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
FLUTTER_DIR="$PROJECT_DIR/flutter"
NATIVE_DIR="$PROJECT_DIR/native"

# Build Rust library
echo "Building Rust library..."
cd "$NATIVE_DIR"
cargo build --release
echo "Rust library built: $NATIVE_DIR/target/release/libflutter_translate_native.so"

# Generate FFI bindings
echo "Generating FFI bindings..."
cd "$FLUTTER_DIR"
flutter_rust_bridge_codegen generate

# Copy native library to Flutter bundle
echo "Copying native library to Flutter bundle..."
BUNDLE_LIB_DIR="$FLUTTER_DIR/build/linux/x64/release/bundle/lib"
mkdir -p "$BUNDLE_LIB_DIR"
cp "$NATIVE_DIR/target/release/libflutter_translate_native.so" "$BUNDLE_LIB_DIR/"
echo "Native library copied to $BUNDLE_LIB_DIR/"

# Build Flutter app
echo "Building Flutter app..."
cd "$FLUTTER_DIR"
flutter build linux --release

# Re-copy native library (flutter build may recreate bundle)
echo "Re-copying native library after Flutter build..."
cp "$NATIVE_DIR/target/release/libflutter_translate_native.so" "$BUNDLE_LIB_DIR/"

# Create wrapper script for proper library loading
echo "Creating wrapper script..."
cat > "$FLUTTER_DIR/build/linux/x64/release/bundle/run.sh" << 'WRAPPER'
#!/bin/bash
SELF="$(readlink -f "$0")"
BUNDLE_DIR="$(dirname "$SELF")"
export LD_LIBRARY_PATH="$BUNDLE_DIR/lib:$LD_LIBRARY_PATH"
exec "$BUNDLE_DIR/xym_ft" "$@"
WRAPPER
chmod +x "$FLUTTER_DIR/build/linux/x64/release/bundle/run.sh"

echo "Build complete! Output: $FLUTTER_DIR/build/linux/x64/release/bundle/"
echo "Run with: cd $FLUTTER_DIR/build/linux/x64/release/bundle && ./run.sh"
