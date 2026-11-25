//! Perhitungan inti GDSS: bobot ROC, SAW per decision maker, dan voting Borda.
//! Modul ini murni (tanpa I/O) sehingga dapat diuji dengan `cargo test`.

use crate::model::{Jenis, Kriteria};
use std::collections::BTreeMap;

/// Bobot Rank Order Centroid untuk `n` kriteria yang sudah diurutkan.
/// w_k = (1/n) * sum_{i=k}^{n} (1/i), dengan k = 1 untuk prioritas tertinggi.
pub fn roc(n: usize) -> Vec<f64> {
    (1..=n)
        .map(|k| (k..=n).map(|i| 1.0 / i as f64).sum::<f64>() / n as f64)
        .collect()
}

/// Memetakan urutan prioritas kriteria ke bobot ROC.
pub fn bobot_dari_urutan(urutan: &[String]) -> BTreeMap<String, f64> {
    urutan.iter().cloned().zip(roc(urutan.len())).collect()
}

/// Simple Additive Weighting.
/// Benefit: r = x / max(x). Cost: r = min(x) / x. V = sum(w * r).
pub fn saw(
    kriteria: &[Kriteria],
    nilai: &BTreeMap<String, BTreeMap<String, u8>>,
    bobot: &BTreeMap<String, f64>,
) -> BTreeMap<String, f64> {
    let mut skor: BTreeMap<String, f64> = nilai.keys().map(|a| (a.clone(), 0.0)).collect();
    for k in kriteria {
        let kolom: Vec<f64> = nilai
            .values()
            .map(|baris| *baris.get(&k.kode).unwrap_or(&1) as f64)
            .collect();
        let maks = kolom.iter().cloned().fold(f64::MIN, f64::max);
        let min = kolom.iter().cloned().fold(f64::MAX, f64::min);
        let w = *bobot.get(&k.kode).unwrap_or(&0.0);
        for (alt, baris) in nilai {
            let x = *baris.get(&k.kode).unwrap_or(&1) as f64;
            let r = match k.jenis {
                Jenis::Benefit => x / maks,
                Jenis::Cost => min / x,
            };
            *skor.get_mut(alt).unwrap() += w * r;
        }
    }
    skor
}

/// Mengurutkan alternatif dari skor tertinggi. Skor sama diurutkan menurut kode.
pub fn ranking(skor: &BTreeMap<String, f64>) -> Vec<String> {
    let mut v: Vec<(&String, &f64)> = skor.iter().collect();
    v.sort_by(|a, b| {
        b.1.partial_cmp(a.1)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.0.cmp(b.0))
    });
    v.into_iter().map(|(k, _)| k.clone()).collect()
}

/// Hasil voting Borda.
pub struct HasilBorda {
    pub poin: BTreeMap<String, u32>,
    pub ranking: Vec<String>,
    pub ada_seri: bool,
}

/// Borda count: alternatif di posisi r (mulai 1) dari m alternatif mendapat m - r poin.
/// Poin seri dipecahkan dengan ranking individual decision maker berjabatan tertinggi.
pub fn borda(rankings: &[Vec<String>], ranking_tertinggi: &[String]) -> HasilBorda {
    let mut poin: BTreeMap<String, u32> = BTreeMap::new();
    for r in rankings {
        let m = r.len() as u32;
        for (i, alt) in r.iter().enumerate() {
            *poin.entry(alt.clone()).or_insert(0) += m - 1 - i as u32;
        }
    }
    let posisi = |a: &String| {
        ranking_tertinggi
            .iter()
            .position(|x| x == a)
            .unwrap_or(usize::MAX)
    };
    let mut urut: Vec<String> = poin.keys().cloned().collect();
    urut.sort_by(|a, b| {
        poin[b]
            .cmp(&poin[a])
            .then_with(|| posisi(a).cmp(&posisi(b)))
    });
    let mut nilai: Vec<u32> = poin.values().cloned().collect();
    nilai.sort_unstable();
    let ada_seri = nilai.windows(2).any(|w| w[0] == w[1]);
    HasilBorda {
        poin,
        ranking: urut,
        ada_seri,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn k(kode: &str, jenis: Jenis) -> Kriteria {
        Kriteria {
            kode: kode.into(),
            nama: kode.into(),
            jenis,
            deskripsi: String::new(),
            urutan: 0,
            skala: vec![],
        }
    }

    #[test]
    fn roc_lima_kriteria() {
        let w = roc(5);
        let harap = [0.4567, 0.2567, 0.1567, 0.0900, 0.0400];
        for (a, b) in w.iter().zip(harap) {
            assert!((a - b).abs() < 1e-4, "{a} != {b}");
        }
        assert!((w.iter().sum::<f64>() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn saw_benefit_dan_cost() {
        let kriteria = vec![k("C1", Jenis::Cost), k("C2", Jenis::Benefit)];
        let mut nilai = BTreeMap::new();
        nilai.insert(
            "A1".into(),
            BTreeMap::from([("C1".into(), 1u8), ("C2".into(), 4u8)]),
        );
        nilai.insert(
            "A2".into(),
            BTreeMap::from([("C1".into(), 4u8), ("C2".into(), 5u8)]),
        );
        let bobot = BTreeMap::from([("C1".into(), 0.5), ("C2".into(), 0.5)]);
        let s = saw(&kriteria, &nilai, &bobot);
        // A1: 0.5*(1/1) + 0.5*(4/5) = 0.9 ; A2: 0.5*(1/4) + 0.5*(5/5) = 0.625
        assert!((s["A1"] - 0.9).abs() < 1e-9);
        assert!((s["A2"] - 0.625).abs() < 1e-9);
        assert_eq!(ranking(&s), vec!["A1", "A2"]);
    }

    #[test]
    fn borda_dengan_seri() {
        let r1: Vec<String> = ["A", "B", "C"].map(String::from).to_vec();
        let r2: Vec<String> = ["B", "A", "C"].map(String::from).to_vec();
        let h = borda(&[r1.clone(), r2], &r1);
        assert_eq!(h.poin["A"], 3);
        assert_eq!(h.poin["B"], 3);
        assert!(h.ada_seri);
        // seri dipecahkan oleh ranking jabatan tertinggi (r1): A di atas B
        assert_eq!(h.ranking, vec!["A", "B", "C"]);
    }
}
