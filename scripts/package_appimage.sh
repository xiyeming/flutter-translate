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
flutter build linux --release

echo "Copying binaries..."
cp -r "$PROJECT_DIR/flutter/build/linux/$FLUTTER_BUILD_ARCH/release/bundle/"* "$APPDIR/usr/bin/"

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
    if ! wget -q "https://github.com/AppImage/AppImageKit/releases/download/continuous/${APPIMAGETOOL}" -O "$APPIMAGETOOL"; then
        echo "Warning: Failed to download appimagetool for $ARCH. Skipping AppImage."
        exit 0
    fi
    chmod +x "$APPIMAGETOOL"
fi

# CI containers lack FUSE; extract and run appimagetool directly
if ./"$APPIMAGETOOL" --appimage-extract >/dev/null 2>&1; then
    ./squashfs-root/AppRun "$APPDIR" "$APP_NAME-${ARCH}.AppImage"
    rm -rf squashfs-root
else
    ARCH=$ARCH ./"$APPIMAGETOOL" "$APPDIR" "$APP_NAME-${ARCH}.AppImage"
fi

echo "Build complete: $BUILD_DIR/$APP_NAME-${ARCH}.AppImage"
