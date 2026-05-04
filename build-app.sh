#!/bin/zsh
set -euo pipefail

SRC="$(cd "$(dirname "$0")" && pwd)"
APP_NAME="${APP_NAME:-Mac Health Monitor Rust}"
BUNDLE_ID="${BUNDLE_ID:-dev.alexandrezeller.MacHealthMonitorRust}"
TARGET_DIR="${CARGO_TARGET_DIR:-/tmp/mac-health-monitor-rust-target}"
OUT_DIR="${OUT_DIR:-$SRC/dist}"
APP="$OUT_DIR/$APP_NAME.app"
CONTENTS="$APP/Contents"
MACOS="$CONTENTS/MacOS"
RESOURCES="$CONTENTS/Resources"
RESOURCE_APP="$RESOURCES/app"
ICONSET="/tmp/mac-health-monitor-rust-AppIcon.iconset"
ICON_PNG="$SRC/assets/AppIcon.png"
ICON_TIFF="/tmp/mac-health-monitor-rust-AppIcon.tiff"

cd "$SRC"
CARGO_TARGET_DIR="$TARGET_DIR" cargo build --release

rm -rf "$APP"
mkdir -p "$MACOS" "$RESOURCE_APP"

cp "$TARGET_DIR/release/mac-health-monitor-rust" "$MACOS/MacHealthMonitorRust"
cp -R "$SRC/public" "$RESOURCE_APP/public"
chmod +x "$MACOS/MacHealthMonitorRust"

cat > "$CONTENTS/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleDevelopmentRegion</key>
  <string>en</string>
  <key>CFBundleDisplayName</key>
  <string>$APP_NAME</string>
  <key>CFBundleExecutable</key>
  <string>MacHealthMonitorRust</string>
  <key>CFBundleIconFile</key>
  <string>AppIcon</string>
  <key>CFBundleIdentifier</key>
  <string>$BUNDLE_ID</string>
  <key>CFBundleInfoDictionaryVersion</key>
  <string>6.0</string>
  <key>CFBundleName</key>
  <string>$APP_NAME</string>
  <key>CFBundlePackageType</key>
  <string>APPL</string>
  <key>CFBundleShortVersionString</key>
  <string>0.1.0</string>
  <key>CFBundleVersion</key>
  <string>1</string>
  <key>LSMinimumSystemVersion</key>
  <string>13.0</string>
  <key>NSHighResolutionCapable</key>
  <true/>
</dict>
</plist>
PLIST

printf "APPL????" > "$CONTENTS/PkgInfo"

if [[ -f "$ICON_PNG" ]]; then
  cp "$ICON_PNG" "$RESOURCES/AppIcon.png"
fi

if [[ -f "$ICON_PNG" ]] && command -v magick >/dev/null 2>&1 && command -v tiff2icns >/dev/null 2>&1; then
  magick "$ICON_PNG" "$ICON_TIFF"
  tiff2icns "$ICON_TIFF" "$RESOURCES/AppIcon.icns"
elif [[ -f "$ICON_PNG" ]] && command -v sips >/dev/null 2>&1 && command -v iconutil >/dev/null 2>&1; then
  rm -rf "$ICONSET"
  mkdir -p "$ICONSET"
  sips -z 16 16 "$ICON_PNG" --out "$ICONSET/icon_16x16.png" >/dev/null
  sips -z 32 32 "$ICON_PNG" --out "$ICONSET/icon_16x16@2x.png" >/dev/null
  sips -z 32 32 "$ICON_PNG" --out "$ICONSET/icon_32x32.png" >/dev/null
  sips -z 64 64 "$ICON_PNG" --out "$ICONSET/icon_32x32@2x.png" >/dev/null
  sips -z 128 128 "$ICON_PNG" --out "$ICONSET/icon_128x128.png" >/dev/null
  sips -z 256 256 "$ICON_PNG" --out "$ICONSET/icon_128x128@2x.png" >/dev/null
  sips -z 256 256 "$ICON_PNG" --out "$ICONSET/icon_256x256.png" >/dev/null
  sips -z 512 512 "$ICON_PNG" --out "$ICONSET/icon_256x256@2x.png" >/dev/null
  sips -z 512 512 "$ICON_PNG" --out "$ICONSET/icon_512x512.png" >/dev/null
  sips -z 1024 1024 "$ICON_PNG" --out "$ICONSET/icon_512x512@2x.png" >/dev/null
  if ! iconutil -c icns "$ICONSET" -o "$RESOURCES/AppIcon.icns"; then
    echo "Icon skipped: iconutil rejected the generated iconset." >&2
  fi
else
  echo "Icon skipped: assets/AppIcon.png, sips, or iconutil is missing." >&2
fi

echo "Built: $APP"
