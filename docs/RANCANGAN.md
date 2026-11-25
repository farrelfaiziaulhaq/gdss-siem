# Rancangan Proyek Akhir: Group Decision Support System Pemilihan SIEM untuk Tim SOC

Mata kuliah: Praktikum Sistem Informasi
Dosen pengampu: Ashri Shabrina Afrah, M.T.

---

## 1. Latar Belakang

Badan Siber dan Sandi Negara (BSSN) mencatat lebih dari 5,5 miliar anomali trafik pada tahun 2025, dengan rata-rata lebih dari 174 anomali per detik. Angka itu melonjak sekitar tujuh kali dibandingkan rata-rata periode 2020 sampai 2024 yang sekitar 24 per detik, walaupun BSSN sendiri mencatat bahwa sebagian lonjakan itu dipengaruhi perbaikan sistem monitoring dan optimalisasi aliran trafik ke sensor deteksi (Laporan Kinerja BSSN 2025, hlm. 21, https://www.bssn.go.id/media/uploads/2026/06/1.-Laporan-Kinerja-LKj-BSSN-2025.pdf).

Pada Juni 2024, insiden ransomware Brain Cipher mengenai Pusat Data Nasional Sementara (PDNS) 2 di Surabaya. Aktivitas berbahaya pertama terdeteksi pada 17 Juni 2024, disusul percobaan penonaktifan Windows Defender sekitar pukul 23.15 WIB. Dampaknya meluas ke 239 instansi pemerintah, yang terdiri dari 30 kementerian/lembaga, 15 provinsi, 148 kabupaten, dan 48 kota. Pelaku meminta tebusan sebesar 8 juta dolar AS atau sekitar 131 miliar rupiah (Antara, 27 Juni 2024, https://www.antaranews.com/berita/4171167/menkominfo-jelaskan-kronologi-serangan-siber-pdns-2). Laporan Kinerja BSSN 2024 (hlm. 172) turut menyebut insiden ini sebagai kejadian yang berdampak pada layanan publik, termasuk keimigrasian (https://www.bssn.go.id/media/uploads/2025/12/Publish-LKJ-BSSN-2024-Pemuktahiran-LAMPIRAN.pdf).

Dari sisi regulasi, Indonesia sudah memiliki UU No. 27 Tahun 2022 tentang Pelindungan Data Pribadi (https://peraturan.bpk.go.id/Details/229798/uu-no-27-tahun-2022), Perpres No. 82 Tahun 2022 tentang Pelindungan Infrastruktur Informasi Vital (https://peraturan.bpk.go.id/Details/211029), dan Perpres No. 47 Tahun 2023 tentang Strategi Keamanan Siber Nasional (https://peraturan.bpk.go.id/Details/255542). Peraturan BSSN No. 1 Tahun 2024 mengatur pengelolaan insiden siber dan Tim Tanggap Insiden Siber (CSIRT). Pada 22 Mei 2026, Peraturan BSSN No. 2 Tahun 2026 tentang KKNI bidang Security Operation Center diterbitkan (https://peraturan.bpk.go.id/Details/353767).

Security Information and Event Management (SIEM) menjadi komponen inti SOC karena fungsinya mengumpulkan log dari berbagai sumber, menormalisasikannya, lalu mengorelasikan kejadian untuk mendeteksi ancaman. Pemilihan SIEM tidak dapat diputuskan oleh satu orang saja. CISO memprioritaskan kemampuan deteksi dan kepatuhan regulasi, SOC Analyst memperhatikan kualitas alerting dan integrasi log untuk operasi sehari-hari, sedangkan Network Admin menekankan kemudahan instalasi dan kompatibilitas dengan perangkat jaringan yang sudah berjalan. Perbedaan sudut pandang inilah yang membuat pemilihan SIEM cocok diselesaikan dengan Group Decision Support System (GDSS), yaitu sistem yang memfasilitasi pengambilan keputusan kelompok secara terstruktur.

Proyek ini membangun GDSS berbasis web untuk memilih SIEM bagi tim SOC skala kecil sampai menengah (sekitar 50 endpoint). Tiga decision maker menilai lima alternatif berdasarkan lima kriteria secara individual, lalu hasilnya digabung dengan voting Borda.

---

## 2. Kriteria Pengambilan Keputusan dan Bobot

### 2.1 Daftar Kriteria

| Kode | Kriteria | Jenis | Deskripsi |
|------|----------|-------|-----------|
| C1 | Biaya lisensi | Cost | Biaya lisensi atau langganan untuk sekitar 50 endpoint dan perangkat jaringan. Makin tinggi nilainya, makin mahal biayanya. |
| C2 | Fitur alerting | Benefit | Kanal notifikasi (email, Slack, webhook), rule yang dapat dikustom, korelasi kejadian lintas sumber, dan eskalasi. |
| C3 | Integrasi log | Benefit | Ragam sumber log yang dapat diterima: syslog, Windows Event Log, agen multi-OS, cloud, dan API. |
| C4 | Kemudahan penggunaan | Benefit | Usaha instalasi, kebutuhan hardware minimum, dan kurva belajar bagi tim kecil. |
| C5 | Komunitas dan dukungan | Benefit | Dokumentasi resmi, forum, repositori terbuka, dan ketersediaan dukungan teknis berbayar. |

### 2.2 Rubrik Penilaian (Skala 1 sampai 5)

#### C1: Biaya lisensi (cost)

| Skor | Deskripsi |
|------|-----------|
| 1 | Tanpa biaya lisensi untuk fitur inti |
| 2 | Gratis, tetapi fitur penting perlu langganan |
| 3 | Berbayar, di bawah US$300 per bulan |
| 4 | Berbayar, US$300 sampai US$1.000 per bulan |
| 5 | Di atas US$1.000 per bulan atau hanya lewat penawaran |

#### C2: Fitur alerting (benefit)

| Skor | Deskripsi |
|------|-----------|
| 1 | Tidak ada alert otomatis pada edisi yang dipakai |
| 2 | Alert dasar lewat satu kanal |
| 3 | Beberapa kanal dengan rule ambang sederhana |
| 4 | Banyak kanal termasuk webhook atau chat, rule dapat dikustom |
| 5 | Korelasi lintas sumber dan rule terpetakan ke MITRE ATT&CK |

#### C3: Integrasi log (benefit)

| Skor | Deskripsi |
|------|-----------|
| 1 | Hanya metrik, tanpa pengumpulan log |
| 2 | Syslog atau satu jenis sumber log |
| 3 | Syslog dan Windows Event Log |
| 4 | Agen multi-OS, syslog, Windows, dan API |
| 5 | Ratusan integrasi siap pakai termasuk layanan cloud |

#### C4: Kemudahan penggunaan (benefit)

| Skor | Deskripsi |
|------|-----------|
| 1 | Instalasi dan operasi butuh tim khusus |
| 2 | Banyak komponen, kurva belajar tinggi |
| 3 | Perlu konfigurasi manual yang cukup banyak |
| 4 | Installer all-in-one dan dokumentasi jelas |
| 5 | Dapat dipakai satu admin dalam satu hari |

#### C5: Komunitas dan dukungan (benefit)

| Skor | Deskripsi |
|------|-----------|
| 1 | Dokumentasi minim, tanpa forum aktif |
| 2 | Dokumentasi resmi saja |
| 3 | Forum resmi dan dukungan berbayar |
| 4 | Komunitas aktif dan dukungan berbayar |
| 5 | Komunitas sangat besar, repositori terbuka, dukungan berbayar |

### 2.3 Pembobotan: Rank Order Centroid (ROC)

Setiap decision maker mengurutkan kriteria dari yang paling diprioritaskan. Bobot dihitung dengan rumus ROC:

```
w_k = (1/n) * Σ(i=k sampai n) (1/i)
```

Untuk 5 kriteria, bobot ROC-nya:

| Prioritas ke- | Bobot |
|---------------|-------|
| 1 | 0,4567 |
| 2 | 0,2567 |
| 3 | 0,1567 |
| 4 | 0,0900 |
| 5 | 0,0400 |

Jumlah bobot = 1,0000.

---

## 3. Alternatif

### A1: Wazuh (SIEM dan XDR)

Open source dan tanpa biaya lisensi, dengan agen yang tersedia untuk Linux, Windows, dan macOS. Rule deteksi dipetakan ke MITRE ATT&CK, dilengkapi File Integrity Monitoring (FIM), deteksi kerentanan, serta modul kepatuhan PCI DSS dan GDPR. Instalasinya cukup satu perintah `curl` dan `bash` (all-in-one installer), dengan kebutuhan hardware minimum 4 vCPU, 8 GiB RAM, dan 50 GB penyimpanan untuk 25 agen. Repositori GitHub mencatat 17.894 bintang per 25 November 2026, sedangkan Wazuh Cloud (layanan terkelola) mulai dari US$571 per bulan untuk 100 agen.

Sumber: https://wazuh.com/, https://documentation.wazuh.com/current/quickstart.html

### A2: Splunk Enterprise Security (SIEM komersial)

Proprietary, dengan harga yang hanya tersedia lewat penawaran. Edisi gratis dibatasi 500 MB per hari dan tidak memiliki fitur alerting, role pengguna, maupun clustering. Enterprise Security menyediakan Risk-Based Alerting (RBA) dan korelasi lintas sumber, sedangkan referensi hardware single-instance memakai 12 core fisik, 12 GB RAM, dan disk dengan 800 IOPS. Repositori `splunk/security_content` di GitHub berisi 2.229 deteksi dan 376 analytic story (versi 6.11.1), dengan 1.738 bintang.

Sumber: https://www.splunk.com/en_us/products/pricing.html, https://docs.splunk.com/Documentation/Splunk/latest/Capacity/Referencehardware

### A3: Elastic Security / ELK Stack (SIEM)

Tier Basic gratis dengan detection engine dan rule yang dipetakan ke MITRE ATT&CK, tetapi konektor notifikasi (email, Slack, webhook) baru tersedia mulai tier Platinum. Lebih dari 400 integrasi siap pakai yang dinormalisasi ke Elastic Common Schema (ECS). Terdiri dari beberapa komponen (Elasticsearch, Kibana, Fleet/Agent), sehingga kurva belajarnya sedang sampai tinggi. GitHub: Elasticsearch 78.768, Kibana 21.395, Beats 12.656, dan detection-rules 2.998 bintang. Mendukung FIM pada tier Basic.

Sumber: https://www.elastic.co/subscriptions, https://www.elastic.co/security/siem

### A4: Zabbix (Monitoring infrastruktur)

AGPLv3 dan tanpa biaya lisensi, dengan fungsi utama pemantauan ketersediaan dan kinerja infrastruktur lewat agen, SNMP, dan log file. Notifikasi mendukung email, SMS, webhook, dan eskalasi, sedangkan kebutuhan hardware untuk 1.000 metrik hanya 2 CPU dan 8 GiB RAM. GitHub mencatat 6.611 bintang, dan subscription support mulai US$325 per bulan. **Catatan**: Zabbix bukan SIEM karena tidak memiliki korelasi ancaman keamanan, threat intelligence, maupun pemetaan MITRE ATT&CK.

Sumber: https://www.zabbix.com/features, https://www.zabbix.com/support

### A5: PRTG Network Monitor (Monitoring jaringan)

Proprietary dengan model langganan per sensor, mulai US$200 per bulan untuk PRTG 500 (sekitar 50 perangkat), dan freeware 100 sensor juga tersedia. Aplikasi ini hanya berjalan di Windows (Server 2016 sampai 2025, Windows 10/11), dengan notifikasi ke email, Microsoft Teams, Slack, dan webhook. Sensor mencakup SNMP, flow, syslog receiver, dan Windows Event Log. **Catatan**: PRTG adalah alat monitoring jaringan, bukan SIEM, sehingga tidak memiliki fitur deteksi ancaman, korelasi keamanan, maupun kepatuhan.

Sumber: https://www.paessler.com/prtg/pricing, https://www.paessler.com/manuals/prtg/notifications

### Catatan tentang Kategori Alternatif

Wazuh, Splunk ES, dan Elastic Security termasuk kategori SIEM. Zabbix dan PRTG termasuk kategori monitoring infrastruktur/jaringan (NMS). Keduanya memang dapat menerima syslog dan Windows Event Log, tetapi tanpa korelasi keamanan dan threat intelligence. Perbedaan kategori ini menyebabkan Zabbix dan PRTG wajar mendapat skor rendah pada kriteria fitur alerting (C2), karena rubriknya menilai kemampuan korelasi ancaman.

---

## 4. Metode yang Digunakan

### 4.1 Arsitektur GDSS

Sistem ini menggabungkan tiga komponen:

1. **Penilaian individual (ROC + SAW)**: Setiap decision maker menentukan prioritas kriteria (bobot ROC) lalu menilai alternatif (skor 1 sampai 5). Skor preferensi dihitung dengan Simple Additive Weighting (SAW), menghasilkan ranking per DM.
2. **Konsensus kelompok (Borda count)**: Ranking individual ketiga DM digabung dengan voting Borda. Alternatif di peringkat *r* dari *m* alternatif mendapat *m - r* poin.
3. **Pemecah seri**: Jika dua alternatif memperoleh poin Borda sama, urutannya ditentukan oleh ranking individual decision maker berjabatan tertinggi (CISO).

### 4.2 Simple Additive Weighting (SAW)

Normalisasi matriks keputusan:
- Kriteria benefit: r = x / max(x)
- Kriteria cost: r = min(x) / x

Skor preferensi: V = Σ (w × r)

### 4.3 Borda Count

Setiap DM menghasilkan ranking dari SAW. Untuk *m* = 5 alternatif:
- Peringkat 1 mendapat 4 poin
- Peringkat 2 mendapat 3 poin
- Peringkat 3 mendapat 2 poin
- Peringkat 4 mendapat 1 poin
- Peringkat 5 mendapat 0 poin

Poin dari ketiga DM dijumlah. Alternatif dengan total poin tertinggi terpilih.

### 4.4 Contoh Perhitungan

#### Urutan Prioritas Tiap DM

| DM | Prioritas 1 | 2 | 3 | 4 | 5 |
|----|-------------|---|---|---|---|
| CISO | C2 (0,4567) | C1 (0,2567) | C3 (0,1567) | C5 (0,0900) | C4 (0,0400) |
| SOC Analyst | C2 (0,4567) | C3 (0,2567) | C4 (0,1567) | C5 (0,0900) | C1 (0,0400) |
| Network Admin | C3 (0,4567) | C4 (0,2567) | C5 (0,1567) | C2 (0,0900) | C1 (0,0400) |

CISO dan SOC Analyst sama-sama memprioritaskan fitur alerting (C2), tetapi berbeda pada prioritas kedua: CISO menekankan biaya (C1), SOC Analyst menekankan integrasi log (C3). Network Admin memprioritaskan integrasi log (C3) dan kemudahan penggunaan (C4).

#### Skor SAW dan Ranking per DM

| Alternatif | CISO | SOC Analyst | Network Admin |
|------------|------|-------------|---------------|
| Wazuh | 0,9687 (#1) | 0,9487 (#1) | 0,7880 (#3) |
| Splunk ES | 0,7567 (#2) | 0,9108 (#2) | 0,7827 (#4) |
| Elastic Security | 0,6690 (#4) | 0,8103 (#3) | 0,7900 (#2) |
| Zabbix | 0,7267 (#3) | 0,6575 (#4) | 0,8080 (#1) |
| PRTG | 0,5476 (#5) | 0,6520 (#5) | 0,7100 (#5) |

Wazuh menempati peringkat pertama pada dua DM (CISO dan SOC Analyst) karena skor tinggi pada C2 (alerting) dan biaya nol pada C1. Network Admin menempatkan Zabbix di peringkat pertama karena bobot terbesar diberikan pada integrasi log dan kemudahan penggunaan, di mana Zabbix memperoleh skor 4 untuk keduanya.

#### Voting Borda

| Alternatif | CISO | SOC | Net Admin | Total | Peringkat |
|------------|------|-----|-----------|-------|-----------|
| Wazuh | 4 | 4 | 2 | **10** | **1** |
| Splunk ES | 3 | 3 | 1 | 7 | 2 |
| Zabbix | 2 | 1 | 4 | 7 | 3 |
| Elastic Security | 1 | 2 | 3 | 6 | 4 |
| PRTG | 0 | 0 | 0 | 0 | 5 |

Splunk ES dan Zabbix sama-sama memperoleh 7 poin. Seri ini dipecahkan oleh ranking individual CISO: Splunk ES berada di peringkat 2 dan Zabbix di peringkat 3, sehingga Splunk ES ditempatkan di atas Zabbix. **Wazuh terpilih** dengan 10 poin.

### 4.5 Referensi Metode

1. Sari, T.N., Astriratma, R., Mulyana, S. (2025). "GDSS Model on Software Engineer Selection: Integration of AHP, SAW, TOPSIS, and BORDA." *JIKO (Jurnal Informatika dan Komputer)*. DOI: 10.26798/jiko.v9i1.1495
2. Sonatha, Y., Azmi, M., Rahmayuni, I. (2021). "GDSS Using AHP, TOPSIS and Borda Methods for Loan Determination in Cooperatives." *JOIV*. DOI: 10.30630/joiv.5.4.640
3. Budhi, M.A., Wardoyo, R. (2017). "GDSS Determination of Best Employee Using TOPSIS and Borda." *IJCCS (UGM)*. DOI: 10.22146/ijccs.22773
4. Saputri, I., Puspaningrum, A.S. (2024). "SPK Pemilihan Sales Provider Terbaik Menggunakan Kombinasi ROC dan TOPSIS." *JOSH*. DOI: 10.47065/josh.v6i1.5842
5. Manzoor, J., dkk. (2024). "Cybersecurity on a budget: Evaluating security and performance of open-source SIEM solutions for SMEs." *PLoS ONE*. DOI: 10.1371/journal.pone.0301183

---

## 5. Database Aplikasi

### 5.1 Teknologi Penyimpanan

Aplikasi ini memakai **Cloud Firestore** (Firebase) sebagai database NoSQL berbasis dokumen. Pada tahap pengembangan, Firebase Emulator dijalankan secara lokal sehingga tidak memerlukan akun atau proyek Firebase sungguhan.

Alasan pemilihan Firestore: (1) Aplikasi ini dibangun dengan Rust (Leptos full-stack), yang tidak memiliki driver MySQL bawaan dari framework, sedangkan Firestore tersedia lewat REST API. (2) Tidak perlu migrasi skema. (3) Firebase Auth menangani autentikasi tanpa harus membangun sistem login dari awal.

### 5.2 Struktur Koleksi Firestore

```
users/{uid}
├── uid: string
├── nama: string
├── email: string
├── jabatan: string        ("CISO" | "SOC Analyst" | "Network Admin")
└── level: number          (3 = tertinggi, 1 = terendah)

kriteria/{kode}
├── kode: string           ("C1" ... "C5")
├── nama: string
├── jenis: string          ("cost" | "benefit")
├── deskripsi: string
├── urutan: number
└── skala: array<string>   (rubrik skor 1 sampai 5)

alternatif/{kode}
├── kode: string           ("A1" ... "A5")
├── nama: string
├── kategori: string
├── lisensi: string
└── ringkasan: string

sesi/{sesi_id}
├── judul: string
└── status: string

sesi/{sesi_id}/penilaian/{uid}
├── uid, nama, jabatan, level
├── urutan_kriteria: array<string>
├── bobot: map<string, number>     (kode kriteria -> bobot ROC)
├── nilai: map<string, map<string, number>>  (kode_alt -> kode_krit -> skor 1..5)
├── skor: map<string, number>      (kode_alt -> skor SAW)
├── ranking: array<string>         (kode_alt, urut dari tertinggi)
└── dikirim_pada: string           (ISO 8601 WIB)

konsensus/{sesi_id}
├── metode: string
├── poin: map<string, number>
├── ranking: array<string>
├── pemenang: string
├── diproses_oleh: string
├── jabatan_pemroses: string
├── diproses_pada: string
└── catatan: string
```

### 5.3 Alur Data

1. Data master (users, kriteria, alternatif) diisi lewat skrip seed (`scripts/seed_emulator.py`).
2. Setiap DM masuk dengan email dan kata sandi lewat Firebase Auth.
3. Server Rust menyimpan sesi sebagai cookie HttpOnly (ID token dan refresh token). Browser tidak pernah mengakses Firestore secara langsung.
4. DM mengirim penilaian: server menghitung bobot ROC dan skor SAW, lalu menyimpan dokumen `penilaian/{uid}`.
5. CISO memproses voting: server menghitung Borda count dari ketiga ranking, lalu menyimpan dokumen `konsensus/{sesi_id}`.
6. Penilaian individual baru terbuka untuk DM lain setelah konsensus diproses, agar tidak saling memengaruhi.

### 5.4 Keamanan

- Firestore Security Rules menolak semua akses langsung dari browser (`allow read, write: if false`).
- Seluruh operasi Firestore dilakukan oleh server Rust dengan service account (atau token `owner` pada emulator).
- Cookie sesi bersifat HttpOnly dan SameSite=Lax, sehingga tidak dapat dibaca oleh JavaScript di browser.

---

## 6. Halaman Aplikasi

### 6.1 Halaman Login (`/login`)

Formulir email dan kata sandi. Validasi: pesan galat ditampilkan jika email atau sandi salah ("Email atau kata sandi salah."), terlalu banyak percobaan, atau emulator tidak berjalan. Pengguna yang sudah masuk langsung diarahkan ke beranda.

### 6.2 Halaman Beranda (`/`)

Halaman utama setelah masuk. Terdiri dari empat bagian:

1. **Panel fokus** (latar gelap): judul sesi, status terkini, dan tombol aksi utama yang berubah sesuai kondisi:
   - "Isi penilaian saya" jika DM belum menilai
   - "Proses voting Borda" jika semua DM sudah menilai dan pengguna adalah DM berjabatan tertinggi
   - "Buka rincian hasil" jika konsensus sudah diproses
2. **Penilaian decision maker**: daftar tiga DM dengan lampu status (hijau = sudah mengirim), waktu pengiriman, dan meter lampu di atas daftar.
3. **Keputusan kelompok**: peringkat akhir Borda beserta poin, atau pesan menunggu jika belum diproses.
4. **Yang diputuskan**: tabel alternatif (kode, nama, kategori, lisensi) dan daftar kriteria (kode, nama, jenis).

### 6.3 Halaman Penilaian (`/penilaian`)

Formulir dua langkah:

1. **Prioritas kriteria**: daftar lima kriteria yang dapat diurutkan dengan tombol "Naik" dan "Turun". Bobot ROC ditampilkan di samping setiap kriteria. Rubrik skala tersedia lewat elemen `<details>` yang dapat dibuka.
2. **Nilai alternatif**: setiap alternatif memiliki lima dropdown (satu per kriteria). Penghitung isian menampilkan "X dari 25 terisi" secara langsung.

Validasi: tombol "Kirim penilaian" menampilkan pesan galat jika ada sel kosong. Setelah berhasil, panel konfirmasi menampilkan ranking SAW versi DM tersebut. Penilaian dapat diubah sampai konsensus diproses, dan dikunci setelahnya.

### 6.4 Halaman Hasil (`/hasil`)

Tersedia setelah konsensus diproses. Tiga bagian:

1. **Panel fokus**: nama alternatif pemenang, metode, waktu pemrosesan, dan catatan seri jika ada.
2. **Rekap voting Borda**: tabel berisi peringkat akhir, ranking individual tiap DM beserta konversi poinnya, dan total poin.
3. **Penilaian individual**: tiga panel yang masing-masing menampilkan bobot kriteria (ROC) dan skor SAW per alternatif.

### 6.5 Halaman Tidak Ditemukan (404)

Pesan "Halaman tidak ditemukan" dengan tombol kembali ke beranda.

### 6.6 Fitur Lintas Halaman

- **Toggle tema terang/gelap**: bertahan lewat `localStorage` dan dimuat sebelum CSS untuk menghindari kedipan.
- **Skip link "Lewati ke konten"**: muncul saat Tab pertama ditekan.
- **Navigasi**: tiga tautan (Beranda, Penilaian saya, Hasil) dengan indikator halaman aktif.
- **Keluar**: menghapus cookie sesi dan mengarahkan ke halaman login.

---

## 7. Progres dan Hal Teknis

### 7.1 Teknologi

| Komponen | Teknologi |
|----------|-----------|
| Backend | Rust (Leptos 0.8, Axum, Tokio) |
| Frontend | Leptos (Rust dikompilasi ke WASM, hydration) |
| Database | Cloud Firestore (REST API) |
| Autentikasi | Firebase Auth (Identity Toolkit REST) |
| Emulator lokal | Firebase Emulator Suite |
| CSS | Custom, tanpa framework |
| Font | IBM Plex Sans (self-hosted, SIL OFL) |

### 7.2 Struktur Direktori

```
gdss-siem/
├── Cargo.toml                  # dependensi dan metadata cargo-leptos
├── firebase.json               # konfigurasi emulator Firebase
├── firestore.rules             # aturan keamanan Firestore
├── .firebaserc                 # proyek default
├── src/
│   ├── main.rs                 # entry point server Axum
│   ├── lib.rs                  # modul dan entry point hydration WASM
│   ├── model.rs                # tipe data bersama (Profil, Kriteria, dll.)
│   ├── gdss.rs                 # perhitungan ROC, SAW, Borda (dengan unit test)
│   ├── api.rs                  # server function Leptos (login, penilaian, voting)
│   ├── firebase.rs             # klien Firebase Auth dan Firestore (REST)
│   └── app.rs                  # komponen UI Leptos (halaman, kerangka, form)
├── style/
│   └── main.css                # stylesheet dengan tema terang/gelap
├── public/
│   └── fonts/                  # IBM Plex Sans woff2
├── scripts/
│   ├── seed_emulator.py        # mengisi data awal ke emulator
│   └── dev.sh                  # menjalankan server dengan variabel emulator
└── docs/
    └── RANCANGAN.md            # dokumen ini
```

### 7.3 Cara Menjalankan

**Prasyarat**: Rust (rustup), cargo-leptos, wasm32-unknown-unknown target, Firebase CLI, Node.js.

```bash
# Terminal 1: jalankan emulator Firebase
cd gdss-siem
firebase emulators:start --project demo-gdss-siem

# Terminal 2: isi data awal
python3 scripts/seed_emulator.py --reset

# Terminal 2: jalankan server Leptos
bash scripts/dev.sh
```

Akun demo:

| Jabatan | Email | Kata sandi |
|---------|-------|------------|
| CISO | ciso@gdss-siem.test | ciso-gdss-2026 |
| SOC Analyst | soc@gdss-siem.test | soc-gdss-2026 |
| Network Admin | netadmin@gdss-siem.test | netadmin-gdss-2026 |

Aplikasi berjalan di http://127.0.0.1:3000. Emulator Firebase UI di http://127.0.0.1:4000.

### 7.4 Unit Test

Modul `gdss.rs` memiliki tiga unit test yang sudah lulus:

```
test gdss::tests::roc_lima_kriteria ... ok
test gdss::tests::saw_benefit_dan_cost ... ok
test gdss::tests::borda_dengan_seri ... ok
```

### 7.5 Uji End-to-End (Playwright/Chromium)

Semua alur telah diuji:

| Uji | Hasil |
|-----|-------|
| Redirect ke `/login` saat belum masuk | Lulus |
| Pesan galat sandi salah | Lulus |
| Login, beranda menampilkan status | Lulus |
| Toggle tema terang/gelap, bertahan setelah reload | Lulus |
| Skip link dan focus indicator | Lulus |
| Halaman hasil sebelum konsensus menampilkan pesan menunggu | Lulus |
| Validasi form kosong sebelum kirim | Lulus |
| Kirim penilaian 3 DM berurutan | Lulus |
| Tombol voting hanya muncul untuk DM tertinggi | Lulus |
| Proses voting Borda, seri dipecahkan | Lulus |
| Penilaian dikunci setelah konsensus | Lulus |
| DM lain dapat melihat hasil | Lulus |
| Halaman 404 | Lulus |
| Tidak ada overflow horizontal di mobile (375px) | Lulus |
| Semua kontrol berukuran minimum 44px | Lulus |
| Kontras warna WCAG AA (seluruh pasangan warna) | Lulus |
| `prefers-reduced-motion` menghentikan animasi | Lulus |

### 7.6 Yang Belum Dikerjakan

- Halaman admin untuk mengelola kriteria dan alternatif lewat UI (saat ini lewat skrip seed).
- Ekspor hasil ke PDF.
- Deployment ke Firebase Hosting atau server publik.
- Nama decision maker masih penanda tempat ("Decision Maker 1/2/3"), perlu diganti nama anggota kelompok sebelum demo.
