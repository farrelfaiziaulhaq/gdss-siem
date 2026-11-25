//! Tipe data yang dipakai bersama oleh server (SSR) dan klien (WASM).

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Identitas sesi keputusan yang aktif. Satu proyek ini hanya memakai satu sesi.
pub const SESI_ID: &str = "pemilihan-siem-2026";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Profil {
    pub uid: String,
    pub nama: String,
    pub email: String,
    pub jabatan: String,
    /// Semakin besar, semakin tinggi jabatannya. CISO = 3.
    pub level: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Jenis {
    Benefit,
    Cost,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Kriteria {
    pub kode: String,
    pub nama: String,
    pub jenis: Jenis,
    pub deskripsi: String,
    pub urutan: u8,
    /// Rubrik skala 1 sampai 5, indeks 0 = skor 1.
    pub skala: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Alternatif {
    pub kode: String,
    pub nama: String,
    pub kategori: String,
    pub lisensi: String,
    pub ringkasan: String,
}

/// Penilaian individual satu decision maker.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Penilaian {
    pub uid: String,
    pub nama: String,
    pub jabatan: String,
    pub level: u8,
    /// Kode kriteria, urut dari prioritas tertinggi.
    pub urutan_kriteria: Vec<String>,
    /// Bobot ROC hasil urutan prioritas.
    pub bobot: BTreeMap<String, f64>,
    /// nilai[kode_alternatif][kode_kriteria] = skor 1..5
    pub nilai: BTreeMap<String, BTreeMap<String, u8>>,
    /// Skor preferensi SAW per alternatif.
    pub skor: BTreeMap<String, f64>,
    /// Kode alternatif, urut dari skor tertinggi.
    pub ranking: Vec<String>,
    pub dikirim_pada: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Konsensus {
    pub metode: String,
    pub poin: BTreeMap<String, u32>,
    pub ranking: Vec<String>,
    pub pemenang: String,
    pub diproses_oleh: String,
    pub jabatan_pemroses: String,
    pub diproses_pada: String,
    pub catatan: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StatusDm {
    pub uid: String,
    pub nama: String,
    pub jabatan: String,
    pub level: u8,
    pub selesai: bool,
    pub dikirim_pada: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Dashboard {
    pub user: Profil,
    pub dms: Vec<StatusDm>,
    pub semua_selesai: bool,
    pub saya_tertinggi: bool,
    pub saya_selesai: bool,
    pub konsensus: Option<Konsensus>,
    pub alternatif: Vec<Alternatif>,
    pub kriteria: Vec<Kriteria>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FormPenilaian {
    pub user: Profil,
    pub kriteria: Vec<Kriteria>,
    pub alternatif: Vec<Alternatif>,
    pub tersimpan: Option<Penilaian>,
    /// true jika konsensus sudah diproses sehingga penilaian tidak dapat diubah.
    pub terkunci: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PenilaianInput {
    pub urutan_kriteria: Vec<String>,
    pub nilai: BTreeMap<String, BTreeMap<String, u8>>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HasilData {
    pub user: Profil,
    pub konsensus: Option<Konsensus>,
    pub penilaian: Vec<Penilaian>,
    pub alternatif: Vec<Alternatif>,
    pub kriteria: Vec<Kriteria>,
    pub jumlah_dm: usize,
}

impl HasilData {
    pub fn nama_alternatif(&self, kode: &str) -> String {
        nama_alt(&self.alternatif, kode)
    }
}

/// "2026-11-25T14:05:33+07:00" menjadi "25 Nov 2026, 14.05 WIB".
pub fn waktu_tampil(iso: &str) -> String {
    const BULAN: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "Mei", "Jun", "Jul", "Agu", "Sep", "Okt", "Nov", "Des",
    ];
    let parse = || -> Option<String> {
        let (tgl, jam) = iso.split_once('T')?;
        let mut t = tgl.split('-');
        let (y, m, d) = (t.next()?, t.next()?.parse::<usize>().ok()?, t.next()?);
        let hh = jam.get(0..2)?;
        let mm = jam.get(3..5)?;
        let d = d.trim_start_matches('0');
        Some(format!(
            "{d} {} {y}, {hh}.{mm} WIB",
            BULAN.get(m.checked_sub(1)?)?
        ))
    };
    parse().unwrap_or_else(|| iso.to_string())
}

/// Angka desimal dengan koma, sesuai kebiasaan penulisan Indonesia.
pub fn desimal(x: f64, digit: usize) -> String {
    format!("{x:.digit$}").replace('.', ",")
}

pub fn nama_alt(list: &[Alternatif], kode: &str) -> String {
    list.iter()
        .find(|a| a.kode == kode)
        .map(|a| a.nama.clone())
        .unwrap_or_else(|| kode.to_string())
}
