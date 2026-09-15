#!/usr/bin/env bash
# Drives the real binary with a scripted session so CI can capture its UI.
set -euo pipefail

ZONO_HOME="$(mktemp -d)"
export ZONO_HOME

BIN="${1:-./target/release/zono}"

printf 'a Ship zono v0.1.0\na Write the README\na Wire up the release pipeline\nd 1\nr 2\nq\n' | "$BIN"
