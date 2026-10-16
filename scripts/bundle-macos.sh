#!/bin/sh
# Build a release rjot.app (with icon) and a zip for distribution in dist/.
set -e
cd "$(dirname "$0")/.."

version=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)
# Info.plist versions must be numeric: 0.0.1-alpha -> 0.0.1
short=${version%%-*}
arch=$(uname -m)

cargo build --release -p rjot

app=dist/rjot.app
rm -rf "$app"
mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources"
cp target/release/rjot "$app/Contents/MacOS/rjot"
cp assets/rjot.icns "$app/Contents/Resources/rjot.icns"
cat > "$app/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleName</key><string>rjot</string>
  <key>CFBundleDisplayName</key><string>rjot</string>
  <key>CFBundleIdentifier</key><string>dev.rjot.rjot</string>
  <key>CFBundleExecutable</key><string>rjot</string>
  <key>CFBundleIconFile</key><string>rjot</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleShortVersionString</key><string>$short</string>
  <key>CFBundleVersion</key><string>$version</string>
  <key>LSMinimumSystemVersion</key><string>11.0</string>
  <key>NSHighResolutionCapable</key><true/>
  <key>LSApplicationCategoryType</key><string>public.app-category.productivity</string>
</dict>
</plist>
PLIST

# Ad-hoc signature (not notarized): required to run on Apple silicon.
codesign --force --deep --sign - "$app"

zip="dist/rjot-$version-macos-$arch.zip"
rm -f "$zip"
ditto -c -k --keepParent "$app" "$zip"
echo "$zip"
