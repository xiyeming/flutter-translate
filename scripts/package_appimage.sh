#!/bin/bash

set -e

ARCH="${ARCH:-x86_64}"
PROJECT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
APP_NAME="Waylex"
BUILD_DIR="$PROJECT_DIR/build"
APPDIR="$BUILD_DIR/AppDir"

if [ "$ARCH" = "aarch64" ]; then
    FLUTTER_BUILD_ARCH="arm64"
else
    FLUTTER_BUILD_ARCH="x64"
fi

echo "=== Waylex AppImage Builder ($ARCH) ==="

echo "Cleaning previous build..."
rm -rf "$BUILD_DIR"
mkdir -p "$APPDIR/usr/bin"
mkdir -p "$APPDIR/usr/share/icons/hicolor/256x256/apps"
mkdir -p "$APPDIR/usr/share/applications"

echo "Building Flutter Linux app ($FLUTTER_BUILD_ARCH)..."
cd "$PROJECT_DIR/flutter"
flutter build linux --release || {
    echo "ERROR: flutter build linux failed"
    exit 1
}

BUNDLE_DIR="$PROJECT_DIR/flutter/build/linux/$FLUTTER_BUILD_ARCH/release/bundle"
echo "Bundle dir: $BUNDLE_DIR"
ls -la "$BUNDLE_DIR/" || true
ls -la "$BUNDLE_DIR/lib/" || true

echo "Copying binaries..."
cp -r "$BUNDLE_DIR/"* "$APPDIR/usr/bin/"

echo "Creating desktop entry..."
cat > "$APPDIR/usr/share/applications/$APP_NAME.desktop" << EOF
[Desktop Entry]
Name=Waylex
Comment=AI Translation Desktop Tool
Exec=$APP_NAME
Icon=$APP_NAME
Type=Application
Categories=Utility;Translation;
StartupNotify=true
Terminal=false
EOF

echo "Creating AppRun..."
cat > "$APPDIR/AppRun" << 'EOF'
#!/bin/bash
SELF="$(readlink -f "$0")"
HERE="${SELF%/*}"
export PATH="$HERE/usr/bin:$PATH"
exec "$HERE/usr/bin/Waylex" "$@"
EOF
chmod +x "$APPDIR/AppRun"

echo "Copying icon..."
ICON_PATH="$PROJECT_DIR/flutter/assets/icons/tray_icon.png"
if [ ! -f "$ICON_PATH" ]; then
    echo "Warning: No icon found, skipping icon copy"
else
    cp "$ICON_PATH" "$APPDIR/usr/share/icons/hicolor/256x256/apps/$APP_NAME.png"
    cp "$ICON_PATH" "$APPDIR/$APP_NAME.png"
fi

echo "Packaging AppImage ($ARCH)..."
cd "$BUILD_DIR"

APPIMAGETOOL="appimagetool-${ARCH}.AppImage"
if [ ! -f "$APPIMAGETOOL" ]; then
    echo "Downloading appimagetool for $ARCH..."
    if ! wget -q --show-progress "https://github.com/AppImage/AppImageKit/releases/download/continuous/${APPIMAGETOOL}" -O "$APPIMAGETOOL"; then
        echo "Warning: Failed to download appimagetool for $ARCH. Skipping AppImage."
        exit 0
    fi
    chmod +x "$APPIMAGETOOL"
fi

# Verify downloaded file is not a small error page
FILE_SIZE=$(stat -c%s "$APPIMAGETOOL" 2>/dev/null || stat -f%z "$APPIMAGETOOL" 2>/dev/null || echo "0")
echo "appimagetool size: $FILE_SIZE bytes"
if [ "$FILE_SIZE" -lt 100000 ]; then
    echo "Warning: appimagetool download seems invalid (too small). Skipping AppImage."
    exit 0
fi

# CI containers lack FUSE; extract and run appimagetool directly
echo "Extracting appimagetool..."
if ./"$APPIMAGETOOL" --appimage-extract >/dev/null 2>&1; then
    echo "Running extracted appimagetool..."
    ./squashfs-root/AppRun "$APPDIR" "$APP_NAME-${ARCH}.AppImage"
    rm -rf squashfs-root
else
    echo "FUSE-less extraction failed, trying direct run with ARCH=$ARCH..."
    ARCH=$ARCH ./"$APPIMAGETOOL" "$APPDIR" "$APP_NAME-${ARCH}.AppImage" || {
        echo "Warning: AppImage packaging failed. Skipping."
        exit 0
    }
fi

if [ -f "$BUILD_DIR/$APP_NAME-${ARCH}.AppImage" ]; then
    echo "Build complete: $BUILD_DIR/$APP_NAME-${ARCH}.AppImage"
else
    echo "Warning: AppImage file not found after build. Skipping."
    exit 0
fi
