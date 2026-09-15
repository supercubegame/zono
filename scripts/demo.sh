#!/usr/bin/env bash
# Launches the real compiled binary and captures its window as a PNG.
#
# Screenshots land in dist/, which the workflow upload-artifact step
# publishes together with the binary. Capture is best-effort only:
# this script always exits 0 so it can never fail a build.

set -uo pipefail

BIN="${1:-./target/release/zono}"
OS="$(uname -s)"

echo "== zono UI capture =="
echo "binary : $BIN"
echo "os     : $OS"

if [ ! -x "$BIN" ]; then
  echo "FAIL: binary missing or not executable"
  exit 0
fi

mkdir -p dist

# Seed a data file so the window shows real rows, not the empty state.
ZONO_HOME="$(mktemp -d)"
export ZONO_HOME
mkdir -p "$ZONO_HOME/.zono"
printf '1\t1\tShip zono v0.2.1\n2\t0\tWrite the README\n3\t0\tWire up the release pipeline\n' > "$ZONO_HOME/.zono/tasks.tsv"
echo "seeded 3 tasks (1 done)"

if [ "$OS" != "Linux" ]; then
  echo "SKIP: unattended capture only runs on Linux runners"
  exit 0
fi

echo "-- installing capture tools --"
sudo apt-get update -qq || true
sudo DEBIAN_FRONTEND=noninteractive apt-get install -y -qq xvfb imagemagick libgl1-mesa-dri mesa-vulkan-drivers libxkbcommon-x11-0 || true

# No GPU on a runner, so force a software rasteriser.
export LIBGL_ALWAYS_SOFTWARE=1
export WGPU_BACKEND=gl
export DISPLAY=:99

echo "-- starting Xvfb --"
Xvfb :99 -screen 0 1100x760x24 +extension GLX +extension RENDER >/tmp/xvfb.log 2>&1 &
XVFB_PID=$!
sleep 3

echo "-- launching zono --"
"$BIN" >/tmp/zono.log 2>&1 &
APP_PID=$!

# Give wgpu time to pick an adapter and present the first frame.
sleep 12

SHOT="dist/zono-ui-linux-x86_64.png"
if kill -0 "$APP_PID" 2>/dev/null; then
  echo "app is alive, capturing"
else
  echo "WARN: app exited early, capturing screen anyway"
fi
import -window root "$SHOT" 2>/dev/null || true

if [ -f "$SHOT" ]; then
  echo "OK: captured $SHOT"
  identify "$SHOT" || true
else
  echo "WARN: no screenshot produced"
fi

echo "-- app log --"
cat /tmp/zono.log || true
echo "-- xvfb log --"
tail -n 20 /tmp/xvfb.log || true

kill "$APP_PID" 2>/dev/null || true
kill "$XVFB_PID" 2>/dev/null || true

exit 0
