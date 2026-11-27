#!/bin/sh
# Builds the Rust CPA Editor in release mode and wraps it in a macOS app bundle: rust/dist/CPA Editor.app.
# Double-click the app to start it, or drag a CPA file onto it.
set -eu

cd "$(dirname "$0")/.."
[ -f "$HOME/.cargo/env" ] && . "$HOME/.cargo/env"

# UNIVERSAL=1: one binary for Apple Silicon and Intel (needs the x86_64-apple-darwin and aarch64-apple-darwin targets)
if [ "${UNIVERSAL:-0}" = 1 ]; then
    # The libxml2 of the macOS SDK serves both architectures
    export PKG_CONFIG_ALLOW_CROSS=1
    cargo build --release -p cpa-editor --target aarch64-apple-darwin
    cargo build --release -p cpa-editor --target x86_64-apple-darwin
    mkdir -p target/universal
    lipo -create -output target/universal/cpa-editor \
        target/aarch64-apple-darwin/release/cpa-editor target/x86_64-apple-darwin/release/cpa-editor
    binary=target/universal/cpa-editor
else
    cargo build --release -p cpa-editor
    binary=target/release/cpa-editor
fi

version=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n 1)
app="dist/CPA Editor.app"
rm -rf "$app"
mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources"
cp "$binary" "$app/Contents/MacOS/cpa-editor"

cat > "$app/Contents/Info.plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleName</key><string>CPA Editor</string>
    <key>CFBundleDisplayName</key><string>CPA Editor</string>
    <key>CFBundleIdentifier</key><string>nl.cpaeditor.rust</string>
    <key>CFBundleExecutable</key><string>cpa-editor</string>
    <key>CFBundlePackageType</key><string>APPL</string>
    <key>CFBundleShortVersionString</key><string>${version:-0.0.0}</string>
    <key>CFBundleVersion</key><string>${version:-0.0.0}</string>
    <key>LSMinimumSystemVersion</key><string>11.0</string>
    <key>NSHighResolutionCapable</key><true/>
    <key>CFBundleDocumentTypes</key>
    <array>
        <dict>
            <key>CFBundleTypeName</key><string>CPA file</string>
            <key>CFBundleTypeRole</key><string>Editor</string>
            <key>LSItemContentTypes</key><array><string>public.xml</string></array>
        </dict>
    </array>
</dict>
</plist>
EOF

# Ad-hoc signature, so macOS starts the app on this machine without a "damaged" warning
codesign --force --sign - "$app" >/dev/null 2>&1 || true

echo "Built $app"
