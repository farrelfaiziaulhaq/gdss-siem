#!/usr/bin/env bash
# Menjalankan server Leptos terhubung ke Firebase Emulator lokal.
# Jalankan emulator lebih dulu di terminal lain:  firebase emulators:start
set -euo pipefail
cd "$(dirname "$0")/.."
export FIREBASE_PROJECT_ID="${FIREBASE_PROJECT_ID:-demo-gdss-siem}"
export FIREBASE_API_KEY="${FIREBASE_API_KEY:-demo-key}"
export FIREBASE_AUTH_EMULATOR_HOST="${FIREBASE_AUTH_EMULATOR_HOST:-127.0.0.1:9099}"
export FIRESTORE_EMULATOR_HOST="${FIRESTORE_EMULATOR_HOST:-127.0.0.1:8080}"
exec cargo leptos watch
