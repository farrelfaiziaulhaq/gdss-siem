# GDSS Pemilihan SIEM untuk Tim SOC

Group Decision Support System berbasis web untuk memilih Security Information and Event Management (SIEM) bagi tim Security Operation Center (SOC) skala kecil sampai menengah.

Proyek akhir mata kuliah Praktikum Sistem Informasi.

---

## Quick Start (3 langkah)

```bash
# 1. Clone & setup semua dependensi otomatis
git clone https://github.com/farrelfaiziaulhaq/gdss-siem.git
cd gdss-siem
bash setup.sh

# 2. Jalankan (buka 2 terminal)

# Terminal 1 — emulator Firebase
firebase emulators:start --project demo-gdss-siem

# Terminal 2 — isi data & jalankan server
python3 scripts/seed_emulator.py --reset
bash scripts/dev.sh

# 3. Buka browser
# http://127.0.0.1:3000
```

Masuk dengan salah satu akun di bawah, lalu ikuti alur di bagian [Cara Pakai](#cara-pakai).

## Akun Demo

| Jabatan | Email | Kata sandi |
|---------|-------|------------|
| CISO | `ciso@gdss-siem.test` | `ciso-gdss-2026` |
| SOC Analyst | `soc@gdss-siem.test` | `soc-gdss-2026` |
| Network Admin | `netadmin@gdss-siem.test` | `netadmin-gdss-2026` |

## Apa yang `setup.sh` Pasang

| Dependensi | Cara pasang | Catatan |
|------------|-------------|---------|
| Rust (stable) | [rustup.rs](https://rustup.rs/) | Otomatis jika belum ada |
| `wasm32-unknown-unknown` | `rustup target add` | Target kompilasi frontend |
| cargo-leptos | `cargo binstall` / `cargo install` | Build tool Leptos |
| Firebase CLI | `npm install -g firebase-tools` | Emulator Auth + Firestore |
| Node.js | **Harus sudah ada** | Prasyarat Firebase CLI |
| Python 3 | **Harus sudah ada** | Skrip seed data |

> Jika Node.js atau Python 3 belum ada, `setup.sh` akan memberitahu dan berhenti.

## Cara Pakai

1. Masuk dengan salah satu akun di atas
2. Buka **Penilaian saya**: atur prioritas kriteria, lalu beri nilai 1-5 untuk setiap alternatif
3. Ulangi langkah 1-2 untuk ketiga akun (logout, login akun lain)
4. Masuk sebagai **CISO**, tekan **Proses voting Borda** di beranda
5. Semua akun dapat melihat hasil di halaman **Hasil**

## Fitur

- **3 decision maker** dengan akun terpisah (CISO, SOC Analyst, Network Admin)
- **Penilaian individual**: prioritas kriteria (bobot ROC) + nilai alternatif (skala 1-5)
- **Perhitungan SAW** otomatis per decision maker
- **Voting Borda** untuk konsensus kelompok, diproses oleh jabatan tertinggi
- **Pemecah seri** berdasarkan ranking DM berjabatan tertinggi
- **Tema terang/gelap** dengan toggle, bertahan di localStorage
- **Aksesibel**: keyboard navigation, skip link, kontras WCAG AA, tap target 44px
- **Responsif**: tampilan mobile tanpa overflow horizontal

## Alternatif yang Dinilai

| Kode | Nama | Kategori |
|------|------|----------|
| A1 | Wazuh | SIEM dan XDR |
| A2 | Splunk Enterprise Security | SIEM komersial |
| A3 | Elastic Security (ELK) | SIEM |
| A4 | Zabbix | Monitoring infrastruktur |
| A5 | PRTG Network Monitor | Monitoring jaringan |

## Kriteria

| Kode | Kriteria | Jenis |
|------|----------|-------|
| C1 | Biaya lisensi | Cost |
| C2 | Fitur alerting | Benefit |
| C3 | Integrasi log | Benefit |
| C4 | Kemudahan penggunaan | Benefit |
| C5 | Komunitas dan dukungan | Benefit |

## Metode

1. **ROC (Rank Order Centroid)**: bobot dari urutan prioritas kriteria
2. **SAW (Simple Additive Weighting)**: skor preferensi per DM
3. **Borda count**: voting konsensus dari ranking ketiga DM

## Teknologi

| Komponen | Teknologi |
|----------|-----------|
| Backend | Rust (Leptos 0.8, Axum, Tokio) |
| Frontend | Leptos (Rust → WASM, hydration) |
| Database | Cloud Firestore (REST API) |
| Autentikasi | Firebase Auth (Identity Toolkit REST) |
| Emulator lokal | Firebase Emulator Suite |
| CI/CD | GitHub Actions |
| Font | IBM Plex Sans (self-hosted, SIL OFL) |

## Struktur Direktori

```
src/
├── main.rs         # entry point server Axum
├── lib.rs          # modul & hydration WASM
├── model.rs        # tipe data bersama
├── gdss.rs         # perhitungan ROC, SAW, Borda (+ unit test)
├── api.rs          # server function (login, penilaian, voting)
├── firebase.rs     # klien Firebase Auth & Firestore (REST)
└── app.rs          # komponen UI Leptos
style/main.css      # stylesheet tema terang/gelap
scripts/
├── setup.sh        # setup otomatis seluruh dependensi
├── seed_emulator.py  # data awal ke emulator
└── dev.sh            # jalankan server + emulator env
docs/RANCANGAN.md     # dokumen rancangan lengkap
```

## CI/CD

Setiap push ke `main` atau pull request menjalankan:

| Job | Isi |
|-----|-----|
| **check** | Kompilasi SSR + WASM, unit test, build lengkap |
| **lint** | `cargo fmt --check` + `cargo clippy -D warnings` |
| **seed-check** | Validasi sintaks skrip Python |

## Dokumen Rancangan

Dokumen lengkap (latar belakang, kriteria, alternatif, metode, database, halaman, progres): [docs/RANCANGAN.md](docs/RANCANGAN.md)

## Lisensi

MIT
