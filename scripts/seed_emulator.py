#!/usr/bin/env python3
"""Mengisi Firebase Emulator (Auth + Firestore) dengan data awal GDSS pemilihan SIEM.

Pemakaian (emulator harus sudah berjalan):
    python3 scripts/seed_emulator.py            # akun, kriteria, alternatif
    python3 scripts/seed_emulator.py --reset    # hapus semua data emulator dulu
    python3 scripts/seed_emulator.py --contoh   # ikut isi penilaian contoh untuk ketiga DM

Hanya untuk emulator. Pada proyek Firebase sungguhan, akun dibuat lewat Firebase Console
dan data awal diimpor oleh admin.
"""

import json
import os
import sys
import urllib.error
import urllib.request

if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")

PROJECT = os.environ.get("FIREBASE_PROJECT_ID", "demo-gdss-siem")
AUTH = os.environ.get("FIREBASE_AUTH_EMULATOR_HOST", "127.0.0.1:9099")
STORE = os.environ.get("FIRESTORE_EMULATOR_HOST", "127.0.0.1:8080")
DOCS = f"http://{STORE}/v1/projects/{PROJECT}/databases/(default)/documents"
SESI = "pemilihan-siem-2026"

# Nama memakai penanda tempat. Ganti dengan nama anggota kelompok sebelum demo.
AKUN = [
    {"email": "ciso@gdss-siem.test", "password": "ciso-gdss-2026", "nama": "Decision Maker 1", "jabatan": "CISO", "level": 3},
    {"email": "soc@gdss-siem.test", "password": "soc-gdss-2026", "nama": "Decision Maker 2", "jabatan": "SOC Analyst", "level": 2},
    {"email": "netadmin@gdss-siem.test", "password": "netadmin-gdss-2026", "nama": "Decision Maker 3", "jabatan": "Network Admin", "level": 1},
]

KRITERIA = [
    {
        "kode": "C1", "nama": "Biaya lisensi", "jenis": "cost", "urutan": 1,
        "deskripsi": "Biaya lisensi atau langganan untuk sekitar 50 endpoint dan perangkat jaringan.",
        "skala": [
            "Tanpa biaya lisensi untuk fitur inti",
            "Gratis, tetapi fitur penting perlu langganan",
            "Berbayar, di bawah US$300 per bulan",
            "Berbayar, US$300 sampai US$1.000 per bulan",
            "Di atas US$1.000 per bulan atau hanya lewat penawaran",
        ],
    },
    {
        "kode": "C2", "nama": "Fitur alerting", "jenis": "benefit", "urutan": 2,
        "deskripsi": "Kanal notifikasi, rule yang dapat dikustom, korelasi kejadian, dan eskalasi.",
        "skala": [
            "Tidak ada alert otomatis pada edisi yang dipakai",
            "Alert dasar lewat satu kanal",
            "Beberapa kanal dengan rule ambang sederhana",
            "Banyak kanal termasuk webhook atau chat, rule dapat dikustom",
            "Korelasi lintas sumber dan rule terpetakan ke MITRE ATT&CK",
        ],
    },
    {
        "kode": "C3", "nama": "Integrasi log", "jenis": "benefit", "urutan": 3,
        "deskripsi": "Ragam sumber log yang dapat diterima: syslog, Windows Event Log, agen, cloud, API.",
        "skala": [
            "Hanya metrik, tanpa pengumpulan log",
            "Syslog atau satu jenis sumber log",
            "Syslog dan Windows Event Log",
            "Agen multi-OS, syslog, Windows, dan API",
            "Ratusan integrasi siap pakai termasuk layanan cloud",
        ],
    },
    {
        "kode": "C4", "nama": "Kemudahan penggunaan", "jenis": "benefit", "urutan": 4,
        "deskripsi": "Usaha instalasi, kebutuhan hardware, dan kurva belajar bagi tim kecil.",
        "skala": [
            "Instalasi dan operasi butuh tim khusus",
            "Banyak komponen, kurva belajar tinggi",
            "Perlu konfigurasi manual yang cukup banyak",
            "Installer all-in-one dan dokumentasi jelas",
            "Dapat dipakai satu admin dalam satu hari",
        ],
    },
    {
        "kode": "C5", "nama": "Komunitas dan dukungan", "jenis": "benefit", "urutan": 5,
        "deskripsi": "Dokumentasi, forum, repositori terbuka, dan ketersediaan dukungan berbayar.",
        "skala": [
            "Dokumentasi minim, tanpa forum aktif",
            "Dokumentasi resmi saja",
            "Forum resmi dan dukungan berbayar",
            "Komunitas aktif dan dukungan berbayar",
            "Komunitas sangat besar, repositori terbuka, dukungan berbayar",
        ],
    },
]

ALTERNATIF = [
    {
        "kode": "A1", "nama": "Wazuh", "kategori": "SIEM dan XDR",
        "lisensi": "Open source, tanpa biaya lisensi",
        "ringkasan": "Agen untuk Linux, Windows, dan macOS dengan rule yang dipetakan ke MITRE ATT&CK, FIM, deteksi kerentanan, serta modul kepatuhan PCI DSS dan GDPR. Tersedia installer all-in-one.",
    },
    {
        "kode": "A2", "nama": "Splunk Enterprise Security", "kategori": "SIEM komersial",
        "lisensi": "Proprietary, harga lewat penawaran",
        "ringkasan": "Platform log dengan Enterprise Security dan Risk-Based Alerting. Edisi gratis dibatasi 500 MB per hari dan tidak memiliki alerting. Referensi hardware single-instance memakai 12 GB RAM.",
    },
    {
        "kode": "A3", "nama": "Elastic Security (ELK)", "kategori": "SIEM",
        "lisensi": "Tier Basic gratis, konektor notifikasi berbayar",
        "ringkasan": "Lebih dari 400 integrasi siap pakai dan detection engine dengan pemetaan MITRE ATT&CK. Pada tier Basic, notifikasi email, webhook, dan Slack belum tersedia.",
    },
    {
        "kode": "A4", "nama": "Zabbix", "kategori": "Monitoring infrastruktur",
        "lisensi": "AGPLv3, tanpa biaya lisensi",
        "ringkasan": "Pemantauan ketersediaan dan kinerja lewat agen, SNMP, dan log file, dengan notifikasi email, SMS, webhook, dan eskalasi. Tidak memiliki korelasi ancaman keamanan seperti SIEM.",
    },
    {
        "kode": "A5", "nama": "PRTG Network Monitor", "kategori": "Monitoring jaringan",
        "lisensi": "Langganan per sensor, mulai US$200 per bulan",
        "ringkasan": "Pemantauan jaringan berbasis sensor (SNMP, flow, syslog, Windows Event Log) yang hanya berjalan di Windows. Notifikasi ke email, Teams, Slack, dan webhook, tanpa fitur deteksi ancaman.",
    },
]

# Penilaian contoh (opsi --contoh). Nilai acuan diturunkan dari dokumentasi resmi vendor,
# lalu tiap DM menggeser beberapa nilai sesuai sudut pandang jabatannya.
CONTOH = {
    "CISO": {
        "urutan": ["C2", "C1", "C3", "C5", "C4"],
        "nilai": {
            "A1": [1, 5, 4, 4, 5], "A2": [5, 5, 5, 2, 4], "A3": [2, 3, 5, 2, 5],
            "A4": [1, 3, 3, 3, 4], "A5": [3, 3, 3, 4, 3],
        },
    },
    "SOC Analyst": {
        "urutan": ["C2", "C3", "C4", "C5", "C1"],
        "nilai": {
            "A1": [1, 5, 4, 4, 5], "A2": [5, 5, 5, 3, 4], "A3": [2, 4, 5, 2, 5],
            "A4": [1, 3, 3, 3, 4], "A5": [3, 3, 3, 4, 3],
        },
    },
    "Network Admin": {
        "urutan": ["C3", "C4", "C5", "C2", "C1"],
        "nilai": {
            "A1": [1, 4, 4, 3, 5], "A2": [5, 5, 5, 2, 4], "A3": [2, 3, 5, 2, 5],
            "A4": [1, 4, 4, 4, 4], "A5": [3, 4, 3, 5, 3],
        },
    },
}


def http(method, url, body=None, auth=True):
    data = json.dumps(body).encode() if body is not None else None
    req = urllib.request.Request(url, data=data, method=method)
    req.add_header("Content-Type", "application/json")
    if auth:
        req.add_header("Authorization", "Bearer owner")
    try:
        with urllib.request.urlopen(req) as r:
            raw = r.read()
            return json.loads(raw) if raw else {}
    except urllib.error.HTTPError as e:
        return {"error": json.loads(e.read() or b"{}").get("error", {"message": str(e)})}


def ke_value(v):
    if v is None:
        return {"nullValue": None}
    if isinstance(v, bool):
        return {"booleanValue": v}
    if isinstance(v, int):
        return {"integerValue": str(v)}
    if isinstance(v, float):
        return {"doubleValue": v}
    if isinstance(v, str):
        return {"stringValue": v}
    if isinstance(v, list):
        return {"arrayValue": {"values": [ke_value(x) for x in v]}}
    if isinstance(v, dict):
        return {"mapValue": {"fields": {k: ke_value(x) for k, x in v.items()}}}
    raise TypeError(type(v))


def set_doc(path, data):
    r = http("PATCH", f"{DOCS}/{path}", {"fields": {k: ke_value(x) for k, x in data.items()}})
    if "error" in r:
        sys.exit(f"Gagal menulis {path}: {r['error']}")


def buat_akun(a):
    base = f"http://{AUTH}/identitytoolkit.googleapis.com/v1"
    body = {"email": a["email"], "password": a["password"], "returnSecureToken": True}
    r = http("POST", f"{base}/accounts:signUp?key=demo-key", body, auth=False)
    if "error" in r:
        r = http("POST", f"{base}/accounts:signInWithPassword?key=demo-key", body, auth=False)
    if "error" in r:
        sys.exit(f"Gagal membuat akun {a['email']}: {r['error']}")
    return r["localId"]


def roc(n):
    return [sum(1 / i for i in range(k, n + 1)) / n for k in range(1, n + 1)]


def saw(nilai, bobot):
    skor = {a: 0.0 for a in nilai}
    for k in KRITERIA:
        kol = [nilai[a][k["kode"]] for a in nilai]
        for a in nilai:
            x = nilai[a][k["kode"]]
            r = min(kol) / x if k["jenis"] == "cost" else x / max(kol)
            skor[a] += bobot[k["kode"]] * r
    return skor


def main():
    if "--reset" in sys.argv:
        http("DELETE", f"http://{STORE}/emulator/v1/projects/{PROJECT}/databases/(default)/documents")
        http("DELETE", f"http://{AUTH}/emulator/v1/projects/{PROJECT}/accounts", auth=False)
        print("Data emulator dihapus.")

    profil = []
    for a in AKUN:
        uid = buat_akun(a)
        p = {"uid": uid, "nama": a["nama"], "email": a["email"], "jabatan": a["jabatan"], "level": a["level"]}
        set_doc(f"users/{uid}", p)
        profil.append(p)
        print(f"Akun {a['jabatan']:<14} {a['email']:<26} sandi: {a['password']}")

    for k in KRITERIA:
        set_doc(f"kriteria/{k['kode']}", k)
    for a in ALTERNATIF:
        set_doc(f"alternatif/{a['kode']}", a)
    set_doc(f"sesi/{SESI}", {"judul": "Pemilihan SIEM untuk tim SOC", "status": "penilaian"})
    print(f"{len(KRITERIA)} kriteria dan {len(ALTERNATIF)} alternatif tersimpan.")

    if "--contoh" in sys.argv:
        kode_k = [k["kode"] for k in KRITERIA]
        for p in profil:
            c = CONTOH[p["jabatan"]]
            nilai = {a: dict(zip(kode_k, v)) for a, v in c["nilai"].items()}
            bobot = dict(zip(c["urutan"], roc(len(c["urutan"]))))
            skor = saw(nilai, bobot)
            ranking = sorted(skor, key=lambda a: (-skor[a], a))
            set_doc(f"sesi/{SESI}/penilaian/{p['uid']}", {
                "uid": p["uid"], "nama": p["nama"], "jabatan": p["jabatan"], "level": p["level"],
                "urutan_kriteria": c["urutan"], "bobot": bobot, "nilai": nilai, "skor": skor,
                "ranking": ranking, "dikirim_pada": "2026-11-25T09:00:00+07:00",
            })
            print(f"Penilaian contoh {p['jabatan']}: " + " > ".join(ranking))


if __name__ == "__main__":
    main()
