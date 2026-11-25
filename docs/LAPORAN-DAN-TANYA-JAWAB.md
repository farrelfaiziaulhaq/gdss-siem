# Laporan Teknis & Panduan Tanya Jawab Sidang/Presentasi Proyek Akhir

**Mata Kuliah**: Praktikum Sistem Informasi
**Dosen Pengampu**: Ibu Ashri Shabrina Afrah, M.T.
**Judul Proyek**: Group Decision Support System (GDSS) Pemilihan SIEM dan Alat Monitoring Jaringan untuk Tim SOC
**Repositori**: https://github.com/farrelfaiziaulhaq/gdss-siem

Dokumen ini disusun sebagai pendamping laporan utama ([`docs/RANCANGAN.md`](RANCANGAN.md)) untuk menjawab pertanyaan teknis maupun konseptual yang lazim diajukan dosen atau asisten praktikum saat demo/penilaian UAS, lengkap dengan **lokasi file dan fungsi di kode sumber sebagai bukti**.

---

## Daftar Isi

1. [Peta Cepat: Algoritma Apa, Dipakai di Mana, Buktinya di Mana](#1-peta-cepat-algoritma-apa-dipakai-di-mana-buktinya-di-mana)
2. [Penjelasan Lengkap Ketiga Algoritma & Contoh Hitung Manual](#2-penjelasan-lengkap-ketiga-algoritma--contoh-hitung-manual)
3. [Bukti Pemenuhan 5 Ketentuan Tugas Akhir (beserta Lokasi Kode)](#3-bukti-pemenuhan-5-ketentuan-tugas-akhir-beserta-lokasi-kode)
4. [Arsitektur Sistem, Database, dan Keamanan](#4-arsitektur-sistem-database-dan-keamanan)
5. [Daftar Tanya Jawab (Q&A) Saat Presentasi / Sidang](#5-daftar-tanya-jawab-qa-saat-presentasi--sidang)
6. [Panduan Skenario Demo Live di Depan Dosen](#6-panduan-skenario-demo-live-di-depan-dosen)

---

## 1. Peta Cepat: Algoritma Apa, Dipakai di Mana, Buktinya di Mana

Sistem ini memakai **tiga metode berurutan** yang masing-masing punya peran spesifik di dalam alur GDSS:

| Tahap di Sistem | Metode / Algoritma | Fungsi di Aplikasi | Lokasi Rumus (Kode) | Lokasi Pemanggilan (Server) | Lokasi Tampilan (UI) | Bukti Pengujian |
|---|---|---|---|---|---|---|
| **1. Pembobotan Kriteria Individual** | **ROC** (*Rank Order Centroid*) | Mengubah urutan prioritas kriteria (1 s.d. 5) pilihan tiap *decision maker* menjadi bobot desimal yang totalnya = `1,0000` | [`src/gdss.rs`](../src/gdss.rs) fungsi `roc()` & `bobot_dari_urutan()` | [`src/api.rs`](../src/api.rs) fungsi `kirim_penilaian()` | [`src/app.rs`](../src/app.rs) komponen `FormNilai` (label bobot di tiap baris prioritas) & `Hasil` | Unit test `gdss::tests::roc_lima_kriteria` |
| **2. Perangkingan Alternatif Individual** | **SAW** (*Simple Additive Weighting*) | Menormalisasi skor 1 s.d. 5 pada kriteria *benefit* dan *cost*, lalu menghitung skor preferensi tiap alternatif untuk masing-masing DM | [`src/gdss.rs`](../src/gdss.rs) fungsi `saw()` & `ranking()` | [`src/api.rs`](../src/api.rs) fungsi `kirim_penilaian()` | [`src/app.rs`](../src/app.rs) panel konfirmasi setelah kirim penilaian & kartu `Penilaian individual` di `/hasil` | Unit test `gdss::tests::saw_benefit_dan_cost` |
| **3. Konsensus Kelompok (Voting)** | **Borda Count** + *Tie-Break* Jabatan Tertinggi | Mengonversi peringkat 1 s.d. 5 dari ketiga DM menjadi poin voting (`4, 3, 2, 1, 0`), menjumlahkannya, dan memecahkan seri memakai ranking DM tertinggi (CISO) | [`src/gdss.rs`](../src/gdss.rs) fungsi `borda()` | [`src/api.rs`](../src/api.rs) fungsi `proses_konsensus()` (khusus level tertinggi) | [`src/app.rs`](../src/app.rs) komponen `Beranda` (panel Keputusan kelompok) & tabel `Rekap voting Borda` di `/hasil` | Unit test `gdss::tests::borda_dengan_seri` + uji E2E browser |

---

## 2. Penjelasan Lengkap Ketiga Algoritma & Contoh Hitung Manual

Bagian ini dapat dipakai langsung apabila dosen meminta pembuktian hitungan tangan/kalkulator dan mencocokkannya dengan angka di layar aplikasi.

### 2.1 Metode ROC (*Rank Order Centroid*) untuk Bobot Kriteria

Setiap DM menyusun urutan prioritas kriteria dari peringkat `1` (paling penting) sampai `n = 5` (paling rendah). Pendekatan ini menghindarkan pengguna dari tebakan angka desimal yang rawan tidak berjumlah satu. Rumus ROC menghitung bobot kriteria pada urutan ke-`k`:

```text
w_k = (1 / n) * Σ_{i = k..n} (1 / i)
```

Untuk `n = 5` kriteria:
- Urutan 1: `w_1 = (1 + 1/2 + 1/3 + 1/4 + 1/5) / 5 = 2,2833 / 5 = 0,4567`
- Urutan 2: `w_2 = (1/2 + 1/3 + 1/4 + 1/5) / 5     = 1,2833 / 5 = 0,2567`
- Urutan 3: `w_3 = (1/3 + 1/4 + 1/5) / 5           = 0,7833 / 5 = 0,1567`
- Urutan 4: `w_4 = (1/4 + 1/5) / 5                 = 0,4500 / 5 = 0,0900`
- Urutan 5: `w_5 = (1/5) / 5                       = 0,2000 / 5 = 0,0400`
- **Total bobot**: `0,4567 + 0,2567 + 0,1567 + 0,0900 + 0,0400 = 1,0000`.

### 2.2 Metode SAW (*Simple Additive Weighting*) untuk Penilaian Individual

Setiap DM mengisi matriks nilai `x_ij` berupa skor `1` sampai `5` untuk alternatif `i` pada kriteria `j`. Kriteria `C1` (Biaya lisensi) bersifat **Cost**: skor 1 berarti tanpa biaya lisensi dan skor 5 berarti sangat mahal. Empat kriteria lainnya (`C2` sampai `C5`) bersifat **Benefit** karena skor tinggi lebih diutamakan. Karena itu, normalisasi `r_ij` memisahkan keduanya:

```text
Jika kriteria j bertipe Benefit : r_ij = x_ij / max_i(x_ij)
Jika kriteria j bertipe Cost    : r_ij = min_i(x_ij) / x_ij
```

Setelah matriks ternormalisasi diperoleh, skor preferensi SAW untuk alternatif `i` adalah:

```text
V_i = Σ_{j = 1..5} (w_j * r_ij)
```

#### Contoh Hitung Lengkap pada Akun CISO (Data Simulasi `--contoh`)

Urutan prioritas CISO: `C2 > C1 > C3 > C5 > C4`, sehingga bobot per kolom adalah:
- `C1 (Cost) = 0,2567`
- `C2 (Benefit) = 0,4567`
- `C3 (Benefit) = 0,1567`
- `C4 (Benefit) = 0,0400`
- `C5 (Benefit) = 0,0900`

**Matriks Nilai Mentah (`x_ij`) yang Diisi CISO:**

| Kode | Alternatif | C1 (Cost) | C2 (Ben) | C3 (Ben) | C4 (Ben) | C5 (Ben) |
|:----:|------------|:---------:|:--------:|:--------:|:--------:|:--------:|
| A1 | Wazuh | 1 | 5 | 4 | 4 | 5 |
| A2 | Splunk ES | 5 | 5 | 5 | 2 | 4 |
| A3 | Elastic Security | 2 | 3 | 5 | 2 | 5 |
| A4 | Zabbix | 1 | 3 | 3 | 3 | 4 |
| A5 | PRTG | 3 | 3 | 3 | 4 | 3 |
| | **Acuan kolom** | `min = 1` | `max = 5` | `max = 5` | `max = 4` | `max = 5` |

**Matriks Ternormalisasi (`r_ij`) dan Skor Akhir SAW (`V_i`) CISO:**

- **A1 (Wazuh)**: `(0,2567 * 1/1) + (0,4567 * 5/5) + (0,1567 * 4/5) + (0,0400 * 4/4) + (0,0900 * 5/5)`
  `= 0,2567 + 0,4567 + 0,1253 + 0,0400 + 0,0900 = 0,9687` -> **Peringkat #1**
- **A2 (Splunk ES)**: `(0,2567 * 1/5) + (0,4567 * 5/5) + (0,1567 * 5/5) + (0,0400 * 2/4) + (0,0900 * 4/5)`
  `= 0,0513 + 0,4567 + 0,1567 + 0,0200 + 0,0720 = 0,7567` -> **Peringkat #2**
- **A4 (Zabbix)**: `(0,2567 * 1/1) + (0,4567 * 3/5) + (0,1567 * 3/5) + (0,0400 * 3/4) + (0,0900 * 4/5)`
  `= 0,2567 + 0,2740 + 0,0940 + 0,0300 + 0,0720 = 0,7267` -> **Peringkat #3**
- **A3 (Elastic)**: `(0,2567 * 1/2) + (0,4567 * 3/5) + (0,1567 * 5/5) + (0,0400 * 2/4) + (0,0900 * 5/5)`
  `= 0,1283 + 0,2740 + 0,1567 + 0,0200 + 0,0900 = 0,6690` -> **Peringkat #4**
- **A5 (PRTG)**: `(0,2567 * 1/3) + (0,4567 * 3/5) + (0,1567 * 3/5) + (0,0400 * 4/4) + (0,0900 * 3/5)`
  `= 0,0856 + 0,2740 + 0,0940 + 0,0400 + 0,0540 = 0,5476` -> **Peringkat #5**

Dengan cara yang sama, hasil SAW kedua DM lainnya adalah:
- **SOC Analyst** (`C2 > C3 > C4 > C5 > C1`): `A1 (0,9487) > A2 (0,9108) > A3 (0,8103) > A4 (0,6575) > A5 (0,6520)`
- **Network Admin** (`C3 > C4 > C5 > C2 > C1`): `A4 (0,8080) > A3 (0,7900) > A1 (0,7880) > A2 (0,7827) > A5 (0,7100)`

Perhatikan bahwa **Network Admin memiliki pemenang berbeda (`Zabbix`)** karena sudut pandang operasional jaringan menempatkan kemudahan dan pemantauan infrastruktur di atas korelasi ancaman. Perbedaan nyata antarjabatan inilah alasan diperlukannya konsensus GDSS.

### 2.3 Metode Voting *Borda Count* untuk Konsensus Kelompok

Pada *Borda count* dengan `m = 5` alternatif, alternatif yang menduduki peringkat ke-`r` (`r = 1..5`) dari seorang DM memperoleh poin sebesar `m - r`:
- Peringkat `#1` = `4 poin`
- Peringkat `#2` = `3 poin`
- Peringkat `#3` = `2 poin`
- Peringkat `#4` = `1 poin`
- Peringkat `#5` = `0 poin`

Penjumlahan suara dari ketiga DM:
- **Wazuh (A1)**: `4 (CISO) + 4 (SOC) + 2 (NetAdmin) = 10 poin` -> **Juara 1**
- **Splunk ES (A2)**: `3 (CISO) + 3 (SOC) + 1 (NetAdmin) = 7 poin` -> **Peringkat 2** *(menang tie-break)*
- **Zabbix (A4)**: `2 (CISO) + 1 (SOC) + 4 (NetAdmin) = 7 poin` -> **Peringkat 3**
- **Elastic Security (A3)**: `1 (CISO) + 2 (SOC) + 3 (NetAdmin) = 6 poin` -> **Peringkat 4**
- **PRTG (A5)**: `0 + 0 + 0 = 0 poin` -> **Peringkat 5**

**Aturan Pemecah Seri (*Tie-Breaker*).** `Splunk ES` dan `Zabbix` sama-sama mengumpulkan `7 poin`. Sesuai hierarki organisasi, sistem mengikuti peringkat individual DM tertinggi (`CISO`). Pada penilaian CISO, `Splunk ES` berada di `#2` dan `Zabbix` di `#3`. Hasilnya, `Splunk ES` naik ke peringkat 2 dan keterangan seri dicatat pada dokumen konsensus.

---

## 3. Bukti Pemenuhan 5 Ketentuan Tugas Akhir (beserta Lokasi Kode)

### Ketentuan 1: Tema bebas
Tema yang diangkat adalah pemilihan SIEM dan alat monitoring jaringan untuk tim SOC. Data vendor, insiden siber nasional, serta landasan regulasi dirinci bersama tautan sumber primer di [`docs/RANCANGAN.md`](RANCANGAN.md).

### Ketentuan 2: Minimal 3 decision maker
Terdapat 3 DM dengan atribut `jabatan` dan `level` numerik (`CISO = 3`, `SOC Analyst = 2`, `Network Admin = 1`) di koleksi `users` Firestore. Definisi datanya ada pada konstanta `AKUN` di [`scripts/seed_emulator.py`](../scripts/seed_emulator.py) dan struct `Profil` di [`src/model.rs`](../src/model.rs).

### Ketentuan 3: Website dengan 3 akun pengguna
Autentikasi memakai Firebase Auth REST API (`sign_in`, `lookup`, `refresh` di [`src/firebase.rs`](../src/firebase.rs)). Setiap DM masuk lewat halaman `/login` ([`src/app.rs`](../src/app.rs)) dengan sesi cookie tersendiri.

### Ketentuan 4: Penilaian individual lalu konsensus voting dari akun tertinggi
Penilaian individual ditangani fungsi `kirim_penilaian` di [`src/api.rs`](../src/api.rs). Dokumennya disimpan per UID pada `sesi/pemilihan-siem-2026/penilaian/{uid}`. Konsensus ditangani fungsi `proses_konsensus` di file yang sama melalui dua pemeriksaan server: `user.level == tertinggi.level` (menolak non-CISO) dan daftar `belum` harus kosong (seluruh DM sudah menilai).

### Ketentuan 5: Semua decision maker dapat melihat hasil konsensus
Begitu dokumen `konsensus/pemilihan-siem-2026` terbentuk, fungsi `ambil_dashboard` dan `ambil_hasil` di [`src/api.rs`](../src/api.rs) membuka hasil bagi semua akun. Baik `Network Admin` maupun `SOC Analyst` dapat melihat peringkat akhir di `/` dan `/hasil`.

---

## 4. Arsitektur Sistem, Database, dan Keamanan

### 4.1 Alur Arsitektur Full-Stack Rust + Firebase

```text
+-------------------------------------------------------------------+
| Browser Pengguna (HTML hasil SSR + Hidrasi WebAssembly Leptos)    |
|  - Rute: /login, /, /penilaian, /hasil                            |
|  - Memegang cookie sesi HttpOnly (gdss_id & gdss_rt)              |
|  - TIDAK memegang API key / akses langsung ke Cloud Firestore     |
+---------------------------------+---------------------------------+
                                  | HTTP POST / GET (Leptos #[server])
                                  v
+-------------------------------------------------------------------+
| Server Backend Rust (Axum + Tokio + Leptos SSR)                   |
|  - src/api.rs      : Validasi sesi, hak akses level, alur GDSS    |
|  - src/gdss.rs     : Mesin hitung ROC, SAW, dan Borda Count       |
|  - src/firebase.rs : Klien HTTP REST ke Auth & Cloud Firestore    |
+----------------+---------------------------------+----------------+
                 | Identity Toolkit REST           | Firestore REST v1
                 v                                 v
+--------------------------------+ +--------------------------------+
| Firebase Authentication        | | Cloud Firestore (NoSQL)        |
|  - Verifikasi email & sandi    | |  - users/{uid}                 |
|  - Penerbitan & refresh token  | |  - kriteria/{kode}             |
+--------------------------------+ |  - alternatif/{kode}           |
                                   |  - sesi/{id}/penilaian/{uid}   |
                                   |  - konsensus/{id}              |
                                   +--------------------------------+
```

### 4.2 Padanan Koleksi Firestore terhadap Model Relasional (ERD Konseptual)

Apabila dosen menanyakan bagaimana hubungan antar-entitas jika dipetakan ke konsep basis data Sistem Informasi:

- **`users` (`uid` PK)** berelasi *one-to-many* ke **`penilaian`** (satu pengguna mengirim maksimal satu penilaian per sesi keputusan).
- **`sesi` (`sesi_id` PK)** berelasi *one-to-many* ke **`penilaian`** (diimplementasikan sebagai *subcollection* `sesi/{sesi_id}/penilaian/{uid}`) dan *one-to-one* ke **`konsensus` (`konsensus/{sesi_id}`)**.
- **`kriteria` (`kode` PK: `C1..C5`)** dan **`alternatif` (`kode` PK: `A1..A5`)** menjadi data induk referensi yang kodenya dipakai sebagai kunci pada peta `bobot`, `nilai`, `skor`, `poin`, dan larik `ranking`.

---

## 5. Daftar Tanya Jawab (Q&A) Saat Presentasi / Sidang

### Q1: Apa perbedaan SPK (DSS) biasa dengan GDSS yang kalian buat?
**Jawab:** SPK biasa hanya melayani pengambil keputusan tunggal dengan satu set bobot. Sebaliknya, GDSS mewadahi tiga pengambil keputusan yang kepentingannya berbeda (`CISO`, `SOC Analyst`, `Network Admin`). Sistem menjaga independensi penilaian dengan menyembunyikan skor DM lain selama sesi berlangsung. Setelah ketiganya selesai, barulah preferensi mereka disatukan lewat voting *Borda count* oleh pimpinan tertinggi.

### Q2: Mengapa metode voting yang dipilih adalah *Borda Count*, bukan voting suara terbanyak (*plurality*) biasa?
**Jawab:** Pada *plurality voting*, tiap DM hanya memilih satu kandidat juara sehingga peringkat ke-2 sampai ke-5 dari SAW terbuang. Dengan 3 DM dan 5 alternatif, pola itu mudah buntu pada skor `1 - 1 - 1` saat pilihan pertama ketiganya berbeda. *Borda count* memberi poin berjenjang (`4, 3, 2, 1, 0`) untuk seluruh urutan. Hasilnya, kandidat yang stabil di papan atas menurut semua jabatan akan menang sebagai kompromi kelompok.

### Q3: Mengapa pembobotan kriteria memakai ROC, bukan AHP atau input persentase manual?
**Jawab:** Input persentase manual merepotkan pengguna karena jumlahnya wajib tepat 100%. Metode AHP pun menuntut `n(n-1)/2 = 10` perbandingan berpasangan per orang dan rawan gagal uji konsistensi (*Consistency Ratio* > 0,1). Lewat ROC, tiap DM cukup mengurutkan 5 kriteria dari yang terpenting. Rumus ROC langsung menghasilkan bobot yang konsisten dan berjumlah tepat `1,0000`.

### Q4: Bagaimana jika pengguna non-CISO mencoba memanggil endpoint voting secara paksa lewat Postman atau `curl`?
**Jawab:** Permintaan itu tetap ditolak server. Pembatasan tidak berhenti di tombol HTML, tetapi diperiksa ulang di fungsi `proses_konsensus()` pada [`src/api.rs`](../src/api.rs). Server memverifikasi cookie sesi, mengambil nilai `level` dari koleksi `users`, lalu mengecek `if user.level != tertinggi.level`. Akun selain CISO langsung menerima `Err` tanpa menyentuh koleksi `konsensus`.

### Q5: Mengapa isi file `firestore.rules` menolak semua akses (`allow read, write: if false`)?
**Jawab:** Aturan itu menerapkan pola *server-mediated access*. Browser tidak pernah menghubungi Firestore secara langsung; seluruh operasi dijalankan oleh backend Rust dengan kredensial *service account* (atau token `owner` di emulator). Pola ini mencegah dua celah sekaligus. Pertama, seorang DM tidak dapat mengintip dokumen `penilaian` rekannya lewat *DevTools* sebelum konsensus. Kedua, klien tidak bisa memalsukan skor SAW maupun dokumen `konsensus` karena seluruh hitungan dieksekusi di server.

### Q6: Apakah setelah konsensus ditetapkan, salah satu DM masih bisa mengubah nilainya?
**Jawab:** Tidak bisa. Fungsi `kirim_penilaian()` di [`src/api.rs`](../src/api.rs) memeriksa dokumen `konsensus/{sesi_id}` lebih dulu. Begitu dokumen konsensus tercatat, server menolak pembaruan nilai dan halaman `/penilaian` mengunci seluruh input dalam mode baca-saja.

### Q7: Mengapa Zabbix dan PRTG dimasukkan ke daftar alternatif padahal keduanya alat monitoring jaringan, bukan SIEM murni?
**Jawab:** Organisasi kecil-menengah kerap menimbang apakah cukup memakai alat monitoring berfitur penerima *syslog* (Zabbix, PRTG) atau langsung memasang SIEM penuh (Wazuh, Elastic Security, Splunk ES). Perbandingan ini justru memperlihatkan manfaat GDSS secara nyata. Dari sudut pandang `Network Admin`, `Zabbix` unggul di `#1` karena ringan dan mudah dirawat. Namun setelah digabung dengan kebutuhan deteksi ancaman dari `CISO` dan `SOC Analyst`, voting Borda memenangkan `Wazuh` yang berfitur SIEM/XDR penuh sekaligus bebas biaya lisensi.

---

## 6. Panduan Skenario Demo Live di Depan Dosen

### Skenario A: Demo Singkat 2 Menit (Langsung Tunjukkan Voting & Hasil)
1. Di Windows klik dua kali **`demo-siap-voting.bat`** (atau jalankan `.\run.ps1 -Contoh` / di Linux `./run.sh --contoh`).
2. Masuk sebagai `soc@gdss-siem.test` (`soc-gdss-2026`): tunjukkan bahwa meskipun `3 dari 3` penilaian sudah masuk, `SOC Analyst` **tidak memiliki** tombol proses voting, dan halaman `/hasil` masih tertutup ("Hasil belum tersedia").
3. Keluar, lalu masuk sebagai `ciso@gdss-siem.test` (`ciso-gdss-2026`): tunjukkan tombol **Proses voting Borda** muncul khusus untuk CISO. Klik tombol tersebut.
4. Klik **Buka rincian hasil** (`/hasil`): tunjukkan pemenang **Wazuh (10 poin)**, kasus seri `Splunk ES` vs `Zabbix` (sama-sama 7 poin) yang dipecahkan oleh ranking CISO, serta rincian bobot ROC dan skor SAW ketiga DM.
5. Keluar, masuk sebagai `netadmin@gdss-siem.test` (`netadmin-gdss-2026`): buka `/hasil` untuk membuktikan ketentuan ke-5 bahwa semua DM kini dapat melihat hasil konsensus, dan buka `/penilaian` untuk membuktikan form sudah terkunci.

### Skenario B: Demo Lengkap dari Nol (Bila Dosen Meminta Input Langsung)
1. Di Windows klik dua kali **`reset-data.bat`** (atau `.\run.ps1 -Reset` / di Linux `./run.sh --reset`).
2. Buka juga `http://127.0.0.1:4000/firestore` (Firebase Emulator UI) di tab sebelahnya agar dosen dapat melihat koleksi Firestore bertambah secara *real-time* setiap kali satu DM mengirim penilaian.
3. Masuk berurutan dengan ketiga akun, ubah urutan kriteria, isi ke-25 nilai, kirim penilaian, lalu eksekusi voting dari akun CISO.
