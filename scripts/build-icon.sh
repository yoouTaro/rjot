#!/bin/sh
# Render assets/logo.svg into the PNG and .icns files used by the app.
# Rendering uses headless Chrome/Brave/Helium for a transparent background
# (Quick Look fills it with white); resizing uses sips and iconutil.
set -e
cd "$(dirname "$0")/.."
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

browser=""
for app in "Google Chrome" "Brave Browser" "Helium" "Chromium"; do
  bin="/Applications/$app.app/Contents/MacOS/$app"
  [ -x "$bin" ] && browser="$bin" && break
done
[ -n "$browser" ] || { echo "Needs Chrome, Brave, Helium or Chromium in /Applications" >&2; exit 1; }

cat > "$tmp/icon.html" <<HTML
<!doctype html><html><head><style>html,body{margin:0;background:transparent}img{display:block;width:1024px;height:1024px}</style></head>
<body><img src="file://$PWD/assets/logo.svg"></body></html>
HTML
master="$tmp/master.png"
"$browser" --headless=new --disable-gpu --hide-scrollbars --no-first-run \
  --user-data-dir="$tmp/profile" --allow-file-access-from-files \
  --default-background-color=00000000 --force-device-scale-factor=1 \
  --window-size=1024,1024 --screenshot="$master" "file://$tmp/icon.html" 2>/dev/null

iconset="$tmp/rjot.iconset"
mkdir -p "$iconset"
for size in 16 32 128 256 512; do
  sips -z $size $size "$master" --out "$iconset/icon_${size}x${size}.png" >/dev/null
  double=$((size * 2))
  sips -z $double $double "$master" --out "$iconset/icon_${size}x${size}@2x.png" >/dev/null
done
iconutil -c icns "$iconset" -o assets/rjot.icns
# Window/Dock icon embedded in the binary.
sips -z 512 512 "$master" --out assets/icon-512.png >/dev/null
echo "assets/rjot.icns and assets/icon-512.png updated"
