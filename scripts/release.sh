#!/bin/bash
set -e

PROJECT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
FLUTTER_DIR="$PROJECT_DIR/flutter"
NATIVE_DIR="$PROJECT_DIR/native"
BUILD_DIR="$PROJECT_DIR/build"
APP_NAME="xym_ft"
APP_ID="com.xym.ft"
VERSION=$(grep '^version' "$NATIVE_DIR/Cargo.toml" | head -1 | sed 's/.*"\(.*\)"/\1/')
OUTPUT="$BUILD_DIR/$APP_NAME-v$VERSION-x86_64.AppImage"

echo "=== $APP_NAME Release Build v$VERSION ==="

# 1. Build everything
echo "[1/4] Building..."
cd "$PROJECT_DIR"
./scripts/build.sh

# 2. Copy bundle
echo "[2/4] Copying bundle..."
rm -rf "$BUILD_DIR/AppDir"
mkdir -p "$BUILD_DIR/AppDir/usr/bin"
mkdir -p "$BUILD_DIR/AppDir/usr/share/applications"
mkdir -p "$BUILD_DIR/AppDir/usr/share/icons/hicolor/256x256/apps"
cp -r "$FLUTTER_DIR/build/linux/x64/release/bundle/"* "$BUILD_DIR/AppDir/usr/bin/"

# 3. Create desktop entry (must be in AppDir root for appimagetool)
echo "[3/4] Creating desktop entry..."
cat > "$BUILD_DIR/AppDir/$APP_ID.desktop" << EOF
[Desktop Entry]
Name=$APP_NAME
Comment=AI Translation Desktop Tool
Exec=$APP_NAME
Icon=$APP_NAME
Type=Application
Categories=Utility;
StartupNotify=true
Terminal=false
StartupWMClass=$APP_NAME
EOF

# Copy icon to AppDir root (appimagetool looks here)
if [ -f "$FLUTTER_DIR/assets/icons/tray_icon.png" ]; then
    cp "$FLUTTER_DIR/assets/icons/tray_icon.png" "$BUILD_DIR/AppDir/$APP_NAME.png"
fi

# Create AppRun
cat > "$BUILD_DIR/AppDir/AppRun" << 'EOF'
#!/bin/bash
HERE="$(dirname "$(readlink -f "$0")")"
export LD_LIBRARY_PATH="$HERE/usr/bin/lib:$LD_LIBRARY_PATH"
exec "$HERE/usr/bin/xym_ft" "$@"
EOF
chmod +x "$BUILD_DIR/AppDir/AppRun"

# 4. Package
echo "[4/4] Packaging AppImage..."
cd "$BUILD_DIR"
if [ ! -f appimagetool-x86_64.AppImage ]; then
    wget -q "https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-x86_64.AppImage" -O appimagetool-x86_64.AppImage
    chmod +x appimagetool-x86_64.AppImage
fi
ARCH=x86_64 ./appimagetool-x86_64.AppImage --appimage-extract-and-run AppDir "$(basename "$OUTPUT")"

echo ""
echo "=== Release complete ==="
echo "Output: $OUTPUT"
echo ""
echo "Install:"
echo "  chmod +x $OUTPUT"
echo "  ./$OUTPUT"
echo ""
echo "System deps needed on target machine:"
echo "  wl-clipboard grim slurp tesseract tesseract-data-eng tesseract-data-chi_sim"
