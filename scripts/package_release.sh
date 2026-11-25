#!/usr/bin/env bash
# Mengemas hasil `cargo leptos build --release` menjadi arsip rilis mandiri
# yang dapat dijalankan tanpa Rust maupun cargo-leptos.
#
# Pemakaian: bash scripts/package_release.sh [versi]
set -euo pipefail

cd "$(dirname "$0")/.."
export PATH="$HOME/.cargo/bin:$PATH"

VERSION="${1:-v1.0.0}"
ARCH="$(uname -m)"
OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
NAME="gdss-siem-${VERSION}-${OS}-${ARCH}"
STAGE="dist/${NAME}"

if [ ! -x target/release/gdss-siem ] || [ ! -d target/site/pkg ]; then
  echo "→ Membangun target release..."
  cargo leptos build --release
fi

rm -rf "$STAGE" "dist/${NAME}.tar.gz" "dist/${NAME}.tar.gz.sha256"
mkdir -p "$STAGE/scripts"

cp target/release/gdss-siem "$STAGE/gdss-siem"
strip "$STAGE/gdss-siem" 2>/dev/null || true
chmod +x "$STAGE/gdss-siem"

cp -r target/site "$STAGE/site"
cp firebase.json .firebaserc firestore.rules "$STAGE/"
cp scripts/seed_emulator.py "$STAGE/scripts/"

cat > "$STAGE/start.sh" <<'EOF'
#!/usr/bin/env bash
# Menjalankan GDSS SIEM versi rilis (tanpa perlu Rust / kompilasi).
#
# Pemakaian:
#   ./start.sh            # jalankan (auto-seed jika kosong)
#   ./start.sh --reset    # reset & isi ulang akun + kriteria + alternatif
#   ./start.sh --contoh   # reset & isi ulang termasuk penilaian contoh 3 DM
set -euo pipefail

cd "$(dirname "$0")"
export PATH="$HOME/.local/bin:$PATH"

GREEN='\033[0;32m'; YELLOW='\033[1;33m'; CYAN='\033[0;36m'; RED='\033[0;31m'; NC='\033[0m'
ok()   { printf "${GREEN}✓${NC} %s\n" "$1"; }
info() { printf "${YELLOW}→${NC} %s\n" "$1"; }
fail() { printf "${RED}✗${NC} %s\n" "$1"; exit 1; }

RESET_FLAG=""
CONTOH_FLAG=""
for arg in "$@"; do
  case "$arg" in
    --reset)  RESET_FLAG="--reset" ;;
    --contoh) RESET_FLAG="--reset"; CONTOH_FLAG="--contoh" ;;
    -h|--help)
      sed -n '2,8p' "$0" | sed 's/^# \?//'
      exit 0
      ;;
    *) fail "Argumen tidak dikenal: $arg" ;;
  esac
done

command -v python3 &>/dev/null || fail "Python 3 belum terpasang (mis. sudo apt install python3)."
command -v java &>/dev/null    || fail "Java (OpenJDK >= 11) belum terpasang (mis. sudo apt install default-jre)."

if ! command -v firebase &>/dev/null; then
  command -v npm &>/dev/null || fail "Firebase CLI belum ada dan npm tidak ditemukan. Pasang Node.js dari https://nodejs.org/"
  info "Memasang Firebase CLI ke ~/.local..."
  mkdir -p "$HOME/.local"
  npm install -g --prefix "$HOME/.local" firebase-tools
fi

export FIREBASE_PROJECT_ID="${FIREBASE_PROJECT_ID:-demo-gdss-siem}"
export FIREBASE_API_KEY="${FIREBASE_API_KEY:-demo-key}"
export FIREBASE_AUTH_EMULATOR_HOST="${FIREBASE_AUTH_EMULATOR_HOST:-127.0.0.1:9099}"
export FIRESTORE_EMULATOR_HOST="${FIRESTORE_EMULATOR_HOST:-127.0.0.1:8080}"
export LEPTOS_ENV=PROD
export LEPTOS_SITE_ROOT=site
export LEPTOS_SITE_PKG_DIR=pkg
export LEPTOS_SITE_ADDR="${LEPTOS_SITE_ADDR:-127.0.0.1:3000}"

EMU_PID=""
cleanup() {
  if [ -n "$EMU_PID" ] && kill -0 "$EMU_PID" 2>/dev/null; then
    echo ""
    info "Menghentikan Firebase Emulator..."
    kill "$EMU_PID" 2>/dev/null || true
    wait "$EMU_PID" 2>/dev/null || true
  fi
}
trap cleanup INT TERM EXIT

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
  ok "Firebase Emulator sudah aktif"
else
  info "Menyalakan Firebase Emulator..."
  firebase emulators:start --project "$FIREBASE_PROJECT_ID" > .firebase-emulator.log 2>&1 &
  EMU_PID=$!
  for _ in $(seq 1 90); do
    if ! kill -0 "$EMU_PID" 2>/dev/null; then
      tail -25 .firebase-emulator.log || true
      fail "Firebase Emulator gagal menyala. Cek .firebase-emulator.log"
    fi
    port_siap 127.0.0.1 8080 && port_siap 127.0.0.1 9099 && break
    sleep 1
  done
  port_siap 127.0.0.1 8080 && port_siap 127.0.0.1 9099 || fail "Waktu tunggu emulator habis."
  ok "Firebase Emulator siap"
fi

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
  info "Mengisi data awal GDSS..."
  python3 scripts/seed_emulator.py $RESET_FLAG $CONTOH_FLAG
else
  ok "Data emulator sudah terisi (gunakan ./start.sh --reset untuk mereset)"
fi

echo ""
printf "${CYAN}============================================================${NC}\n"
printf "  GDSS SIEM siap di : ${GREEN}http://%s${NC}\n" "$LEPTOS_SITE_ADDR"
printf "  Emulator UI       : http://127.0.0.1:4000\n"
printf "  Akun Demo:\n"
printf "    • CISO          : ciso@gdss-siem.test     / ciso-gdss-2026\n"
printf "    • SOC Analyst   : soc@gdss-siem.test      / soc-gdss-2026\n"
printf "    • Network Admin : netadmin@gdss-siem.test / netadmin-gdss-2026\n"
printf "  Tekan Ctrl+C untuk berhenti.\n"
printf "${CYAN}============================================================${NC}\n\n"

./gdss-siem
EOF
chmod +x "$STAGE/start.sh"

(
  cd dist
  tar -czf "${NAME}.tar.gz" "${NAME}"
  sha256sum "${NAME}.tar.gz" > "${NAME}.tar.gz.sha256"
)

echo "✓ Paket rilis dibuat: dist/${NAME}.tar.gz ($(du -h "dist/${NAME}.tar.gz" | awk '{print $1}'))"
cat "dist/${NAME}.tar.gz.sha256"
