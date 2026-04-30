#!/bin/bash

set -e

echo "=== Flutter Translate AppImage Builder ==="

PROJECT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
APP_NAME="flutter-translate"
BUILD_DIR="$PROJECT_DIR/build"
APPDIR="$BUILD_DIR/AppDir"

echo "Cleaning previous build..."
rm -rf "$BUILD_DIR"
mkdir -p "$APPDIR/usr/bin"
mkdir -p "$APPDIR/usr/share/icons/hicolor/256x256/apps"
mkdir -p "$APPDIR/usr/share/applications"

echo "Building Flutter Linux app..."
cd "$PROJECT_DIR/flutter"
flutter build linux --release

echo "Copying binaries..."
cp -r "$PROJECT_DIR/flutter/build/linux/x64/release/bundle/"* "$APPDIR/usr/bin/"

echo "Creating desktop entry..."
cat > "$APPDIR/usr/share/applications/$APP_NAME.desktop" << EOF
[Desktop Entry]
Name=Flutter Translate
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
exec "$HERE/usr/bin/flutter_translate" "$@"
EOF
chmod +x "$APPDIR/AppRun"

echo "Copying icon..."
# Generate a placeholder icon if none exists
if [ ! -f "$PROJECT_DIR/flutter/assets/icon.png" ]; then
    echo "Warning: No icon found, skipping icon copy"
else
    cp "$PROJECT_DIR/flutter/assets/icon.png" "$APPDIR/usr/share/icons/hicolor/256x256/apps/$APP_NAME.png"
    cp "$PROJECT_DIR/flutter/assets/icon.png" "$APPDIR/$APP_NAME.png"
fi

echo "Packaging AppImage..."
cd "$BUILD_DIR"

# Download appimagetool if not present
if [ ! -f appimagetool-x86_64.AppImage ]; then
    echo "Downloading appimagetool..."
    wget -q "https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-x86_64.AppImage" -O appimagetool-x86_64.AppImage
    chmod +x appimagetool-x86_64.AppImage
fi

ARCH=x86_64 ./appimagetool-x86_64.AppImage "$APPDIR" "$APP_NAME-x86_64.AppImage"

echo "Build complete: $BUILD_DIR/$APP_NAME-x86_64.AppImage"
