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
mkdir -p "$APPDIR/usr/share/icons/hicolor/48x48/apps"
mkdir -p "$APPDIR/usr/share/applications"
mkdir -p "$APPDIR/usr/share/metainfo"

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

# Copy native Rust library into bundle before packaging
NATIVE_SO="$PROJECT_DIR/native/target/release/libflutter_translate_native.so"
if [ -f "$NATIVE_SO" ]; then
    echo "Copying native Rust library..."
    mkdir -p "$BUNDLE_DIR/lib"
    cp "$NATIVE_SO" "$BUNDLE_DIR/lib/"
else
    echo "Warning: Native library not found at $NATIVE_SO"
fi

echo "Copying binaries..."
cp -r "$BUNDLE_DIR/"* "$APPDIR/usr/bin/"

echo "Creating desktop entry..."
cat > "$APPDIR/usr/share/applications/$APP_NAME.desktop" << EOF
[Desktop Entry]
Name=Waylex
Comment=AI Translation Desktop Tool
Exec=$APP_NAME
Icon=$APP_NAME
StartupWMClass=com.xym.ft.Waylex
Type=Application
Categories=Utility;Translation;
StartupNotify=true
Terminal=false
EOF

# appimagetool also requires desktop file at AppDir root
cp "$APPDIR/usr/share/applications/$APP_NAME.desktop" "$APPDIR/$APP_NAME.desktop"

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
# Prefer 256x256 icon if available (for correct hicolor sizing), fallback to 48x48 tray icon
ICON_256="$PROJECT_DIR/flatpak/com.xym.ft.Waylex.png"
ICON_48="$PROJECT_DIR/flutter/assets/icons/tray_icon.png"
if [ -f "$ICON_256" ]; then
    cp "$ICON_256" "$APPDIR/usr/share/icons/hicolor/256x256/apps/$APP_NAME.png"
    cp "$ICON_256" "$APPDIR/$APP_NAME.png"
    echo "Using 256x256 icon"
elif [ -f "$ICON_48" ]; then
    cp "$ICON_48" "$APPDIR/usr/share/icons/hicolor/48x48/apps/$APP_NAME.png"
    cp "$ICON_48" "$APPDIR/$APP_NAME.png"
    echo "Using 48x48 icon (installing to hicolor/48x48)"
else
    echo "Warning: No icon found, skipping icon copy"
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
