#!/usr/bin/env bash
# Setup otomatis seluruh dependensi GDSS SIEM.
# Jalankan: bash setup.sh
set -euo pipefail
RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'; NC='\033[0m'
ok()   { printf "${GREEN}✓${NC} %s\n" "$1"; }
info() { printf "${YELLOW}→${NC} %s\n" "$1"; }
fail() { printf "${RED}✗${NC} %s\n" "$1"; exit 1; }

cd "$(dirname "$0")"

# 1. Rust
if command -v rustc &>/dev/null; then
  ok "Rust $(rustc --version | awk '{print $2}') sudah terpasang"
else
  info "Memasang Rust via rustup..."
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal
  source "$HOME/.cargo/env"
  ok "Rust terpasang"
fi
source "$HOME/.cargo/env" 2>/dev/null || true

# 2. Target WASM
if rustup target list --installed | grep -q wasm32-unknown-unknown; then
  ok "Target wasm32-unknown-unknown sudah ada"
else
  info "Menambah target wasm32-unknown-unknown..."
  rustup target add wasm32-unknown-unknown
  ok "Target WASM ditambahkan"
fi

# 3. cargo-leptos
if command -v cargo-leptos &>/dev/null; then
  ok "cargo-leptos sudah terpasang"
else
  info "Memasang cargo-leptos (bisa 1-2 menit)..."
  if command -v cargo-binstall &>/dev/null; then
    cargo binstall -y cargo-leptos
  else
    cargo install cargo-leptos --locked
  fi
  ok "cargo-leptos terpasang"
fi

# 4. Node.js
if command -v node &>/dev/null; then
  ok "Node.js $(node --version) sudah terpasang"
else
  fail "Node.js belum terpasang. Pasang dari https://nodejs.org/"
fi

# 5. Firebase CLI
if command -v firebase &>/dev/null; then
  ok "Firebase CLI sudah terpasang"
else
  info "Memasang Firebase CLI via npm..."
  npm install -g firebase-tools
  ok "Firebase CLI terpasang"
fi

# 6. Python 3
if command -v python3 &>/dev/null; then
  ok "Python $(python3 --version | awk '{print $2}') sudah terpasang"
else
  fail "Python 3 belum terpasang. Pasang dari https://python.org/"
fi

# 7. Build proyek
info "Build proyek (kompilasi pertama bisa 5-10 menit)..."
cargo leptos build
ok "Build selesai"

echo ""
printf "${GREEN}Semua dependensi siap!${NC}\n"
echo ""
echo "Langkah selanjutnya:"
echo "  1. Terminal 1:  firebase emulators:start --project demo-gdss-siem"
echo "  2. Terminal 2:  python3 scripts/seed_emulator.py --reset"
echo "  3. Terminal 2:  bash scripts/dev.sh"
echo "  4. Buka http://127.0.0.1:3000"
