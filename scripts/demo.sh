#!/usr/bin/env bash
# Launches the real compiled binary, captures its window, and proves what
# rendered by running OCR over the result.
#
# Outputs:
#   dist/zono-ui-linux-x86_64.png   the screenshot (published with the release)
#   docs/CI_CAPTURE_LOG.md          diagnostics + OCR text, committed to main
#
# Capture is best-effort: this script always exits 0.

set -uo pipefail

BIN="${1:-./target/release/zono}"
OS="$(uname -s)"

mkdir -p dist docs
LOG="docs/CI_CAPTURE_LOG.md"

say() { echo "$*" | tee -a "$LOG"; }

printf '# CI capture log\n\n```\n' > "$LOG"

say "== zono UI capture =="
say "date   : $(date -u)"
say "binary : $BIN"
say "os     : $OS"

finish() {
  printf '```\n' >> "$LOG"
  if [ "$OS" = "Linux" ]; then
    git config user.name "github-actions[bot]" || true
    git config user.email "41898282+github-actions[bot]@users.noreply.github.com" || true
    git add "$LOG" || true
    git commit -m "ci: publish capture log and OCR result [skip" -m "ci]" || true
    git pull --rebase --autostash origin main || true
    git push origin HEAD:main || say "push failed"
  fi
  exit 0
}

if [ ! -x "$BIN" ]; then
  say "FAIL: binary missing or not executable"
  finish
fi

say "size   : $(wc -c < "$BIN") bytes"

# Seed a data file so the window shows real rows, not the empty state.
ZONO_HOME="$(mktemp -d)"
export ZONO_HOME
mkdir -p "$ZONO_HOME/.zono"
printf '1\t1\tShip zono v0.2.1\n2\t0\tWrite the README\n3\t0\tWire up the release pipeline\n' > "$ZONO_HOME/.zono/tasks.tsv"
say "seeded 3 tasks (1 done)"

if [ "$OS" != "Linux" ]; then
  say "SKIP: unattended capture only runs on Linux runners"
  finish
fi

say "-- installing capture tools --"
sudo apt-get update -qq >/dev/null 2>&1 || true
sudo DEBIAN_FRONTEND=noninteractive apt-get install -y -qq xvfb imagemagick xdotool x11-utils tesseract-ocr libegl1 libgl1 libgl1-mesa-dri libxkbcommon-x11-0 >/dev/null 2>&1 || true
for t in Xvfb import xdotool tesseract; do
  command -v "$t" >/dev/null && say "tool ok   : $t" || say "tool MISS : $t"
done

# A runner has no GPU. iced ships a software rasteriser, use it.
export ICED_BACKEND=tiny-skia
export LIBGL_ALWAYS_SOFTWARE=1
export WINIT_UNIX_BACKEND=x11
export RUST_BACKTRACE=1
export DISPLAY=:99
say "renderer: ICED_BACKEND=$ICED_BACKEND"

say "-- starting Xvfb --"
Xvfb :99 -screen 0 1100x760x24 >/tmp/xvfb.log 2>&1 &
XVFB_PID=$!
sleep 3

say "-- launching zono --"
"$BIN" >/tmp/zono.log 2>&1 &
APP_PID=$!

WID=""
for i in $(seq 1 20); do
  sleep 2
  if ! kill -0 "$APP_PID" 2>/dev/null; then
    say "app exited after ~$((i * 2))s"
    break
  fi
  WID="$(xdotool search --name zono 2>/dev/null | head -n1)"
  if [ -n "$WID" ]; then
    say "window found after ~$((i * 2))s : id=$WID"
    break
  fi
done

sleep 4

SHOT="dist/zono-ui-linux-x86_64.png"
if [ -n "$WID" ]; then
  xdotool windowactivate "$WID" 2>/dev/null || true
  xdotool windowraise "$WID" 2>/dev/null || true
  sleep 2
  import -window "$WID" "$SHOT" 2>/dev/null || import -window root "$SHOT" 2>/dev/null || true
else
  say "no window matched, grabbing root"
  import -window root "$SHOT" 2>/dev/null || true
fi

if [ -f "$SHOT" ]; then
  say "captured : $(wc -c < "$SHOT") bytes"
  say "identify : $(identify "$SHOT" 2>/dev/null)"
  say "colours  : $(identify -format '%k unique colours' "$SHOT" 2>/dev/null)"
  say "stats    : $(identify -format 'mean=%[mean] sd=%[standard-deviation]' "$SHOT" 2>/dev/null)"

  say "-- OCR of the captured window --"
  # Upscale first, the UI font is small and OCR does better on a 2x image.
  convert "$SHOT" -resize 200% -colorspace gray -normalize /tmp/ocr.png 2>/dev/null || cp "$SHOT" /tmp/ocr.png
  tesseract /tmp/ocr.png stdout 2>/dev/null | sed '/^[[:space:]]*$/d' | head -n 30 | tee -a "$LOG" || say "OCR unavailable"
else
  say "WARN: no screenshot produced"
fi

say "-- app stdout/stderr --"
head -n 40 /tmp/zono.log 2>/dev/null | tee -a "$LOG" || true

kill "$APP_PID" 2>/dev/null || true
kill "$XVFB_PID" 2>/dev/null || true

say "== capture finished =="
finish
