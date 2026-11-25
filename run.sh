#!/usr/bin/env bash
# Runner satu perintah untuk GDSS SIEM (Emulator + Seed + Server Web).
#
# Pemakaian:
#   ./run.sh              # jalankan aplikasi (auto-setup & auto-seed jika kosong)
#   ./run.sh --reset      # bersihkan & isi ulang akun + kriteria + alternatif
#   ./run.sh --contoh     # bersihkan & isi ulang termasuk penilaian contoh 3 DM
#   ./run.sh --release    # jalankan biner release yang teroptimasi
set -euo pipefail

cd "$(dirname "$0")"
export PATH="$HOME/.cargo/bin:$HOME/.local/bin:$PATH"

GREEN='\033[0;32m'; YELLOW='\033[1;33m'; CYAN='\033[0;36m'; RED='\033[0;31m'; NC='\033[0m'
ok()   { printf "${GREEN}✓${NC} %s\n" "$1"; }
info() { printf "${YELLOW}→${NC} %s\n" "$1"; }
fail() { printf "${RED}✗${NC} %s\n" "$1"; exit 1; }

RESET_FLAG=""
CONTOH_FLAG=""
RELEASE_MODE=0
for arg in "$@"; do
  case "$arg" in
    --reset)   RESET_FLAG="--reset" ;;
    --contoh)  RESET_FLAG="--reset"; CONTOH_FLAG="--contoh" ;;
    --release) RELEASE_MODE=1 ;;
    -h|--help)
      sed -n '2,9p' "$0" | sed 's/^# \?//'
      exit 0
      ;;
    *) fail "Argumen tidak dikenal: $arg (gunakan --help)" ;;
  esac
done

# 1. Pastikan semua dependensi tersedia (jika belum, jalankan setup.sh otomatis)
if ! command -v firebase &>/dev/null || ! command -v python3 &>/dev/null || ! command -v java &>/dev/null; then
  info "Dependensi belum lengkap, menjalankan setup.sh..."
  bash ./setup.sh
fi
if [ "$RELEASE_MODE" -eq 0 ] && ! command -v cargo-leptos &>/dev/null; then
  info "cargo-leptos belum tersedia, menjalankan setup.sh..."
  bash ./setup.sh
fi

export FIREBASE_PROJECT_ID="${FIREBASE_PROJECT_ID:-demo-gdss-siem}"
export FIREBASE_API_KEY="${FIREBASE_API_KEY:-demo-key}"
export FIREBASE_AUTH_EMULATOR_HOST="${FIREBASE_AUTH_EMULATOR_HOST:-127.0.0.1:9099}"
export FIRESTORE_EMULATOR_HOST="${FIRESTORE_EMULATOR_HOST:-127.0.0.1:8080}"
export LEPTOS_SITE_ADDR="${LEPTOS_SITE_ADDR:-127.0.0.1:3000}"

EMU_PID=""
cleanup() {
  if [ -n "$EMU_PID" ] && kill -0 "$EMU_PID" 2>/dev/null; then
    echo ""
    info "Menghentikan Firebase Emulator (PID $EMU_PID)..."
    kill "$EMU_PID" 2>/dev/null || true
    wait "$EMU_PID" 2>/dev/null || true
  fi
}
trap cleanup INT TERM EXIT

# 2. Nyalakan Firebase Emulator jika belum aktif
port_siap() {
  python3 - "$1" "$2" <<'PY'
import socket, sys
s = socket.socket()
s.settimeout(0.5)
try:
    s.connect((sys.argv[1], int(sys.argv[2])))
    sys.exit(0)
except OSError:
    sys.exit(1)
PY
}

if port_siap 127.0.0.1 8080 && port_siap 127.0.0.1 9099; then
  ok "Firebase Emulator sudah aktif di 127.0.0.1:8080 & 127.0.0.1:9099"
else
  info "Menyalakan Firebase Emulator (Auth :9099, Firestore :8080, UI :4000)..."
  firebase emulators:start --project "$FIREBASE_PROJECT_ID" > .firebase-emulator.log 2>&1 &
  EMU_PID=$!
  for _ in $(seq 1 90); do
    if ! kill -0 "$EMU_PID" 2>/dev/null; then
      tail -25 .firebase-emulator.log || true
      fail "Firebase Emulator berhenti sebelum siap. Periksa .firebase-emulator.log"
    fi
    if port_siap 127.0.0.1 8080 && port_siap 127.0.0.1 9099; then
      break
    fi
    sleep 1
  done
  if ! port_siap 127.0.0.1 8080 || ! port_siap 127.0.0.1 9099; then
    tail -25 .firebase-emulator.log || true
    fail "Waktu tunggu Firebase Emulator habis."
  fi
  ok "Firebase Emulator siap"
fi

# 3. Seed otomatis jika koleksi users masih kosong atau diminta --reset/--contoh
SUDAH_ADA="$(python3 - <<'PY'
import json, os, urllib.request
p = os.environ["FIREBASE_PROJECT_ID"]
h = os.environ["FIRESTORE_EMULATOR_HOST"]
url = f"http://{h}/v1/projects/{p}/databases/(default)/documents/users?pageSize=1"
req = urllib.request.Request(url, headers={"Authorization": "Bearer owner"})
try:
    with urllib.request.urlopen(req, timeout=3) as r:
        d = json.loads(r.read() or b"{}")
        print("1" if d.get("documents") else "0")
except Exception:
    print("0")
PY
)"

if [ -n "$RESET_FLAG" ] || [ "$SUDAH_ADA" = "0" ]; then
  info "Mengisi data awal GDSS ke emulator..."
  python3 scripts/seed_emulator.py $RESET_FLAG $CONTOH_FLAG
else
  ok "Data di emulator sudah terisi (gunakan ./run.sh --reset untuk mereset)"
fi

echo ""
printf "${CYAN}============================================================${NC}\n"
printf "  Aplikasi Web GDSS : ${GREEN}http://%s${NC}\n" "$LEPTOS_SITE_ADDR"
printf "  Emulator UI       : http://127.0.0.1:4000\n"
printf "  Akun Demo:\n"
printf "    • CISO          : ciso@gdss-siem.test     / ciso-gdss-2026\n"
printf "    • SOC Analyst   : soc@gdss-siem.test      / soc-gdss-2026\n"
printf "    • Network Admin : netadmin@gdss-siem.test / netadmin-gdss-2026\n"
printf "  Tekan Ctrl+C untuk menghentikan server & emulator.\n"
printf "${CYAN}============================================================${NC}\n\n"

# 4. Jalankan server
if [ "$RELEASE_MODE" -eq 1 ]; then
  if [ ! -x target/release/gdss-siem ] || [ ! -d target/site/pkg ]; then
    info "Membangun versi release..."
    cargo leptos build --release
  fi
  LEPTOS_ENV=PROD LEPTOS_SITE_ROOT=target/site ./target/release/gdss-siem
else
  cargo leptos serve
fi
