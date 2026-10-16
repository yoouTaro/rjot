#!/bin/sh
# Capture the README screenshots from scripted scenes (debug build), each
# with its own throwaway note file under target/. Only rjot's own window is
# captured (by PID), at 2x on Retina.
# Usage: scripts/screenshots.sh [scene…]   (default: all scenes)
set -e
only=" $* "
cd "$(dirname "$0")/.."
out=${RJOT_SHOTS_OUT:-docs/screenshots}
mkdir -p "$out" target/demo
cargo build -q -p rjot
swiftc -O scripts/winid.swift -o target/winid 2>/dev/null

now=$(date +%s)
ago() { echo $((now - $1)); }

# Demo notes; $1 is the settings object.
notes() {
  cat <<JSON
{"settings": $1,
 "next_id": 12,
 "items": [
  {"id":1,"kind":"prompt","text":"Rewrite the onboarding email so it's half as long and keeps the pricing link.","created":$(ago 90000),"done":true,"color":"purple"},
  {"id":2,"kind":"note","text":"CREATE INDEX CONCURRENTLY can't run inside a transaction block, so a migration that uses it has to turn off the wrapping transaction.","source":"Claude","created":$(ago 26000),"color":"blue"},
  {"id":3,"kind":"prompt","text":"Given this stack trace, explain the root cause before proposing a fix. Keep it under ten lines.","created":$(ago 9000),"color":"red"},
  {"id":4,"kind":"note","text":"Good defaults beat options. Every setting is a question you didn't answer for the user.","source":"Safari","created":$(ago 7400)},
  {"id":5,"kind":"prompt","text":"Review this diff for race conditions between the event tap thread and the UI.","created":$(ago 4100)},
  {"id":6,"kind":"note","text":"brew install --cask rjot  # someday","source":"Terminal","created":$(ago 3000),"color":"green"},
  {"id":7,"kind":"prompt","text":"Summarize the thread below as three bullet points for a status update.","created":$(ago 1900),"color":"orange"},
  {"id":8,"kind":"note","text":"The Accessibility API returns the focused element's AXSelectedText; browsers often leave it empty, so fall back to a copy and restore the clipboard.","source":"Notes","created":$(ago 1300),"color":"orange"},
  {"id":9,"kind":"note","text":"Meeting moved to Thursday 10:30, same room.","source":"Messages","created":$(ago 420)},
  {"id":10,"kind":"note","text":"Capture now, sort later.","source":"Claude","created":$(ago 25),"color":"purple"}
 ],
 "scratch": "rjot 0.0.1\\n- double-tap shift capture\\n- queue + scratch\\n- color labels\\n- themes, sensitive mode\\n\\nnext: menu bar icon?"}
JSON
}

dark='{"theme":"Copper","appearance":"dark","sensitive":false,"sound":true}'

scene() { # name, settings, then env assignments
  name=$1; settings=$2; shift 2
  [ "$only" = "  " ] || case "$only" in *" $name "*) ;; *) return 0 ;; esac
  data="target/demo/$name.json"
  notes "$settings" > "$data"
  ready=$(mktemp -u)
  env RJOT_DATA="$data" RJOT_DEMO_TRUSTED=1 RJOT_DEMO_READY="$ready" "$@" target/debug/rjot >/dev/null 2>&1 &
  pid=$!
  i=0
  while [ ! -f "$ready" ] && [ $i -lt 300 ]; do sleep 0.1; i=$((i + 1)); done
  sleep 0.3
  if [ -f "$ready" ] && wid=$(target/winid $pid | cut -d" " -f1); then
    screencapture -x -o -l "$wid" "$out/$name.png"
    echo "$out/$name.png"
  else
    echo "scene $name: not ready" >&2
  fi
  kill $pid 2>/dev/null || true
  wait $pid 2>/dev/null || true
  rm -f "$ready"
}

# Keep the pointer off the window so nothing looks hovered by accident.
target/winid --park 4000 4000 || true

scene notes "$dark" RJOT_DEMO_HOVER=8
scene queue "$dark" RJOT_DEMO_TAB=queue
scene colors "$dark" RJOT_DEMO_PICK=8
scene settings "$dark" RJOT_DEMO_SETTINGS=1
scene light '{"theme":"Graphite","appearance":"light","sensitive":false,"sound":true}'
scene sage '{"theme":"Sage","appearance":"dark","sensitive":false,"sound":true}' RJOT_DEMO_TAB=queue
scene sensitive '{"theme":"Copper","appearance":"dark","sensitive":true,"sound":true}' RJOT_DEMO_CAPTURABLE=1
scene scratch "$dark" RJOT_DEMO_TAB=scratch
