#!/usr/bin/env bash
# Setup otomatis seluruh dependensi GDSS SIEM (tanpa sudo).
# Jalankan: ./setup.sh          (mode lengkap: termasuk Rust + cargo-leptos)
#           ./setup.sh --runtime (mode rilis: cukup Java, Node, Firebase CLI, Python 3)
set -euo pipefail

MODE="${1:-full}"
RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'; NC='\033[0m'
ok()   { printf "${GREEN}✓${NC} %s\n" "$1"; }
info() { printf "${YELLOW}→${NC} %s\n" "$1"; }
fail() { printf "${RED}✗${NC} %s\n" "$1"; exit 1; }

cd "$(dirname "$0")"
export PATH="$HOME/.cargo/bin:$HOME/.local/bin:$PATH"

# 1. Python 3 (untuk seed_emulator.py)
if command -v python3 &>/dev/null; then
  ok "Python $(python3 --version | awk '{print $2}') tersedia"
else
  fail "Python 3 belum terpasang. Pasang terlebih dahulu (mis. apt install python3)."
fi

# 2. Java >= 11 (wajib untuk Cloud Firestore Emulator)
if command -v java &>/dev/null; then
  ok "Java tersedia ($(java -version 2>&1 | head -1))"
else
  fail "Java (OpenJDK >= 11, disarankan 21) belum terpasang. Pasang terlebih dahulu (mis. sudo apt install default-jre)."
fi

# 3. Node.js & npm (wajib untuk Firebase CLI)
if command -v node &>/dev/null && command -v npm &>/dev/null; then
  ok "Node.js $(node --version) & npm $(npm --version) tersedia"
else
  fail "Node.js & npm belum terpasang. Pasang dari https://nodejs.org/"
fi

# 4. Firebase CLI (pasang ke ~/.local jika prefix global butuh root)
if command -v firebase &>/dev/null; then
  ok "Firebase CLI $(firebase --version) tersedia"
else
  info "Memasang Firebase CLI..."
  NPM_PREFIX="$(npm config get prefix 2>/dev/null || echo /usr/local)"
  if [ -w "$NPM_PREFIX/lib" ] && [ -w "$NPM_PREFIX/bin" ]; then
    npm install -g firebase-tools
  else
    mkdir -p "$HOME/.local"
    npm install -g --prefix "$HOME/.local" firebase-tools
  fi
  ok "Firebase CLI terpasang di $(command -v firebase)"
fi

if [ "$MODE" = "--runtime" ]; then
  echo ""
  ok "Dependensi runtime siap!"
  exit 0
fi

# 5. Rust (stable)
if command -v rustc &>/dev/null && command -v cargo &>/dev/null; then
  ok "Rust $(rustc --version | awk '{print $2}') tersedia"
else
  info "Memasang Rust (rustup, level user)..."
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal
  source "$HOME/.cargo/env"
  ok "Rust terpasang"
fi

# 6. Target WASM
if rustup target list --installed | grep -q wasm32-unknown-unknown; then
  ok "Target wasm32-unknown-unknown tersedia"
else
  info "Menambahkan target wasm32-unknown-unknown..."
  rustup target add wasm32-unknown-unknown
  ok "Target wasm32-unknown-unknown ditambahkan"
fi

# 7. cargo-leptos (prioritaskan biner pra-kompilasi via cargo-binstall agar cepat)
if command -v cargo-leptos &>/dev/null; then
  ok "cargo-leptos tersedia"
else
  info "Memasang cargo-leptos..."
  if ! command -v cargo-binstall &>/dev/null; then
    curl -L --proto '=https' --tlsv1.2 -sSf \
      https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash || true
  fi
  if command -v cargo-binstall &>/dev/null; then
    cargo binstall -y cargo-leptos
  else
    cargo install cargo-leptos --locked
  fi
  ok "cargo-leptos terpasang"
fi

echo ""
printf "${GREEN}Semua dependensi siap!${NC} Jalankan aplikasi dengan satu perintah:\n"
echo "  ./run.sh"
