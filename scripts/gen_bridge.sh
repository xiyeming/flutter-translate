#!/bin/bash

set -e

echo "=== Generating FFI Bridge Code ==="

PROJECT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$PROJECT_DIR/flutter"

flutter_rust_bridge_codegen generate

echo "FFI code generation complete!"
