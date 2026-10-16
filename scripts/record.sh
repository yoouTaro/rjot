#!/bin/sh
# Record the README / site clips: rjot plays a scripted demo (debug build,
# RJOT_DEMO_STEPS) on throwaway demo notes while its window is recorded,
# then ffmpeg makes an MP4 (site) and a GIF (README). The captures in the
# script arrive the way real ones do (note, source, page link, glow, toast);
# the key caps drawn at the bottom show which keys a real run would press.
# The window must stay on top while recording: don't touch anything.
# Usage: scripts/record.sh [clip…]   (default: all clips)
set -e
cd "$(dirname "$0")/.."
out=${RJOT_CLIPS_OUT:-docs/clips}
mkdir -p "$out" target/demo
cargo build -q -p rjot
swiftc -O scripts/winid.swift -o target/winid 2>/dev/null
only=" $* "
now=$(date +%s)
ago() { echo $((now - $1)); }

seed() {
  cat <<JSON
{"settings": {"theme":"Copper","appearance":"dark","sensitive":false,"sound":false},
 "next_id": 20,
 "items": [
  {"id":1,"kind":"prompt","text":"Rewrite the onboarding email so it's half as long and keeps the pricing link.","created":$(ago 90000),"done":true,"color":"purple"},
  {"id":2,"kind":"note","text":"CREATE INDEX CONCURRENTLY can't run inside a transaction block, so a migration that uses it has to turn off the wrapping transaction.","source":"Claude","created":$(ago 26000),"color":"blue"},
  {"id":3,"kind":"prompt","text":"Given this stack trace, explain the root cause before proposing a fix. Keep it under ten lines.","created":$(ago 9000),"color":"red"},
  {"id":4,"kind":"note","text":"Good defaults beat options. Every setting is a question you didn't answer for the user.","source":"Safari","created":$(ago 7400)},
  {"id":5,"kind":"prompt","text":"Review this diff for race conditions between the event tap thread and the UI.","created":$(ago 4100)},
  {"id":7,"kind":"prompt","text":"Summarize the thread below as three bullet points for a status update.","created":$(ago 1900),"color":"orange"},
  {"id":9,"kind":"note","text":"Meeting moved to Thursday 10:30, same room.","source":"Messages","created":$(ago 420)}
 ],
 "scratch": ""}
JSON
}

clip() { # name, seconds, then env assignments
  name=$1; secs=$2; shift 2
  [ "$only" = "  " ] || case "$only" in *" $name "*) ;; *) return 0 ;; esac
  data="target/demo/clip-$name.json"
  seed > "$data"
  ready=$(mktemp -u); done_file=$(mktemp -u); mov=$(mktemp -u).mov
  env RJOT_DATA="$data" RJOT_DEMO_TRUSTED=1 RJOT_DEMO_READY="$ready" RJOT_DEMO_DONE="$done_file" "$@" target/debug/rjot >/dev/null 2>&1 &
  pid=$!
  i=0
  while [ ! -f "$ready" ] && [ $i -lt 300 ]; do sleep 0.1; i=$((i + 1)); done
  set -- $(target/winid $pid)
  # Keep the pointer out of the clip: the far corner of the main display.
  target/winid --park 4 4
  screencapture -x -v -V "$secs" -R"$2,$3,$4,$5" "$mov"
  kill $pid 2>/dev/null || true
  wait $pid 2>/dev/null || true
  [ -f "$done_file" ] || echo "clip $name: script didn't finish in ${secs}s" >&2
  ffmpeg -loglevel error -y -i "$mov" -vf "fps=30,scale=880:-2:flags=lanczos" -c:v libx264 -preset slow -crf 24 -pix_fmt yuv420p -movflags +faststart -an "$out/$name.mp4"
  ffmpeg -loglevel error -y -i "$mov" -vf "fps=15,scale=440:-1:flags=lanczos,split[a][b];[a]palettegen=max_colors=128:stats_mode=diff[p];[b][p]paletteuse=dither=bayer:bayer_scale=4:diff_mode=rectangle" "$out/$name.gif"
  rm -f "$mov" "$ready" "$done_file"
  ls -la "$out/$name.mp4" "$out/$name.gif"
}

# Two captures land (the second from a browser, with its page), then ⌥⇧V
# pastes a queued prompt into Cursor and checks it off.
clip capture 15 RJOT_DEMO_STEPS="wait:900;keys:⇧ ⇧;wait:420;capture:Safari|The Accessibility API returns the focused element's AXSelectedText; browsers often leave it empty, so fall back to a copy and restore the clipboard.|https://developer.apple.com/documentation/applicationservices/axuielement_h|AXUIElement.h | Apple Developer Documentation;wait:2300;keys:⇧ ⇧;wait:420;capture:Claude|Capture now, sort later.|https://claude.ai/chat/3f2a|Designing a capture app;wait:2400;keys:⌥ ⇧ V;wait:500;picker:Cursor;wait:1100;down;wait:800;down;wait:900;choose;wait:2600;done"
