#!/usr/bin/env bash
# ContentOS — one-shot setup for macOS
# © 2026 ZediaTech LLC (Zedia Labs). All rights reserved. Proprietary & source-available.
# Owner/operator: Victor Chaidez. Licensing: licensing@zedialabs.com · zedialabs.com
set -e

echo "▶ ContentOS setup"
echo

# 1. Rust
if ! command -v cargo >/dev/null 2>&1; then
  echo "→ Installing Rust (rustup)…"
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
  source "$HOME/.cargo/env"
else
  echo "✓ Rust present ($(cargo --version))"
fi

# 2. Node
if ! command -v node >/dev/null 2>&1; then
  echo "✗ Node.js not found. Install it first: https://nodejs.org (LTS), then re-run."
  exit 1
else
  echo "✓ Node present ($(node --version))"
fi

# 3. Xcode command line tools (needed to compile on macOS)
if ! xcode-select -p >/dev/null 2>&1; then
  echo "→ Triggering Xcode Command Line Tools install (follow the popup, then re-run this script)…"
  xcode-select --install || true
  exit 0
fi

# 4. JS deps + icons
echo "→ Installing CLI…"
npm install

echo "→ Generating app icons…"
npm run icon

echo
echo "✓ Setup complete."
echo "  Run the app in dev mode:   npm run dev"
echo "  Build a installable .app:  npm run build   (output in src-tauri/target/release/bundle)"
