//! Server function Leptos. Semua akses Firebase terjadi di server; browser tidak pernah
//! memegang kredensial Firestore.

use crate::model::*;
use leptos::prelude::*;
use leptos::server_fn::codec::Json;

#[cfg(feature = "ssr")]
pub mod srv {
    use crate::firebase;
    use crate::model::*;
    use axum::http::{header, HeaderMap, HeaderValue};
    use leptos::prelude::*;
    use serde::de::DeserializeOwned;

    const C_TOKEN: &str = "gdss_id";
    const C_REFRESH: &str = "gdss_rt";

    pub fn err(m: impl Into<String>) -> ServerFnError {
        ServerFnError::new(m.into())
    }

    fn fb_err(e: String) -> ServerFnError {
        leptos::logging::error!("firebase: {e}");
        err("Layanan data tidak dapat dihubungi. Pastikan Firebase Emulator berjalan, lalu coba lagi.")
    }

    fn cookie(headers: &HeaderMap, name: &str) -> Option<String> {
        headers
            .get_all(header::COOKIE)
            .iter()
            .filter_map(|v| v.to_str().ok())
            .flat_map(|s| s.split(';'))
            .filter_map(|p| {
                let (k, v) = p.trim().split_once('=')?;
                (k == name && !v.is_empty()).then(|| v.to_string())
            })
            .next()
    }

    fn tulis_cookie(nama: &str, isi: &str, umur: u32) {
        let secure = if std::env::var("COOKIE_SECURE").is_ok() {
            "; Secure"
        } else {
            ""
        };
        let c = format!("{nama}={isi}; Path=/; HttpOnly; SameSite=Lax; Max-Age={umur}{secure}");
        if let (Some(resp), Ok(v)) = (
            use_context::<leptos_axum::ResponseOptions>(),
            HeaderValue::from_str(&c),
        ) {
            resp.append_header(header::SET_COOKIE, v);
        }
    }

    pub fn simpan_sesi(id_token: &str, refresh_token: &str) {
        tulis_cookie(C_TOKEN, id_token, 3600);
        tulis_cookie(C_REFRESH, refresh_token, 60 * 60 * 24 * 7);
    }

    pub fn hapus_sesi() {
        tulis_cookie(C_TOKEN, "", 0);
        tulis_cookie(C_REFRESH, "", 0);
    }

    fn parse<T: DeserializeOwned>(v: serde_json::Value) -> Result<T, ServerFnError> {
        serde_json::from_value(v).map_err(|e| {
            leptos::logging::error!("data tidak sesuai skema: {e}");
            err("Data di Firestore tidak sesuai skema. Jalankan ulang skrip seed.")
        })
    }

    /// Pengguna yang sedang masuk, atau `None` jika belum masuk / sesi habis.
    pub async fn pengguna() -> Result<Option<Profil>, ServerFnError> {
        let headers: HeaderMap = leptos_axum::extract().await?;
        let fb = firebase::get();
        let mut uid = None;
        if let Some(t) = cookie(&headers, C_TOKEN) {
            uid = fb.lookup(&t).await.ok();
        }
        if uid.is_none() {
            if let Some(rt) = cookie(&headers, C_REFRESH) {
                if let Ok(s) = fb.refresh(&rt).await {
                    simpan_sesi(&s.id_token, &s.refresh_token);
                    uid = Some(s.uid);
                }
            }
        }
        let Some(uid) = uid else { return Ok(None) };
        match fb.get_doc(&format!("users/{uid}")).await.map_err(fb_err)? {
            Some(v) => Ok(Some(parse(v)?)),
            None => Ok(None),
        }
    }

    pub async fn wajib_masuk() -> Result<Profil, ServerFnError> {
        pengguna()
            .await?
            .ok_or_else(|| err("Sesi Anda sudah berakhir. Silakan masuk kembali."))
    }

    pub async fn semua_dm() -> Result<Vec<Profil>, ServerFnError> {
        let mut v: Vec<Profil> = firebase::get()
            .list("users")
            .await
            .map_err(fb_err)?
            .into_iter()
            .map(|(_, d)| parse(d))
            .collect::<Result<_, _>>()?;
        v.sort_by_key(|a| std::cmp::Reverse(a.level));
        Ok(v)
    }

    pub async fn kriteria() -> Result<Vec<Kriteria>, ServerFnError> {
        let mut v: Vec<Kriteria> = firebase::get()
            .list("kriteria")
            .await
            .map_err(fb_err)?
            .into_iter()
            .map(|(_, d)| parse(d))
            .collect::<Result<_, _>>()?;
        v.sort_by_key(|k| k.urutan);
        Ok(v)
    }

    pub async fn alternatif() -> Result<Vec<Alternatif>, ServerFnError> {
        let mut v: Vec<Alternatif> = firebase::get()
            .list("alternatif")
            .await
            .map_err(fb_err)?
            .into_iter()
            .map(|(_, d)| parse(d))
            .collect::<Result<_, _>>()?;
        v.sort_by(|a, b| a.kode.cmp(&b.kode));
        Ok(v)
    }

    pub async fn semua_penilaian() -> Result<Vec<Penilaian>, ServerFnError> {
        let mut v: Vec<Penilaian> = firebase::get()
            .list(&format!("sesi/{SESI_ID}/penilaian"))
            .await
            .map_err(fb_err)?
            .into_iter()
            .map(|(_, d)| parse(d))
            .collect::<Result<_, _>>()?;
        v.sort_by_key(|a| std::cmp::Reverse(a.level));
        Ok(v)
    }

    pub async fn konsensus() -> Result<Option<Konsensus>, ServerFnError> {
        match firebase::get()
            .get_doc(&format!("konsensus/{SESI_ID}"))
            .await
            .map_err(fb_err)?
        {
            Some(v) => Ok(Some(parse(v)?)),
            None => Ok(None),
        }
    }

    pub async fn simpan<T: serde::Serialize>(path: &str, data: &T) -> Result<(), ServerFnError> {
        let v = serde_json::to_value(data).map_err(|e| err(e.to_string()))?;
        firebase::get().set_doc(path, &v).await.map_err(fb_err)
    }

    pub fn sekarang() -> String {
        let wib = chrono::FixedOffset::east_opt(7 * 3600).unwrap();
        chrono::Utc::now()
            .with_timezone(&wib)
            .to_rfc3339_opts(chrono::SecondsFormat::Secs, false)
    }
}

/// Pesan galat yang ramah ditampilkan, tanpa awalan teknis dari server_fn.
pub fn pesan(e: &ServerFnError) -> String {
    match e {
        ServerFnError::ServerError(m) => m.clone(),
        _ => "Permintaan gagal dikirim. Periksa koneksi lalu coba lagi.".to_string(),
    }
}

#[server]
pub async fn masuk(email: String, password: String) -> Result<(), ServerFnError> {
    use srv::*;
    let email = email.trim().to_lowercase();
    if email.is_empty() || password.is_empty() {
        return Err(err("Email dan kata sandi wajib diisi."));
    }
    let s = crate::firebase::get()
        .sign_in(&email, &password)
        .await
        .map_err(|e| match e.as_str() {
            "INVALID_LOGIN_CREDENTIALS"
            | "EMAIL_NOT_FOUND"
            | "INVALID_PASSWORD"
            | "INVALID_EMAIL" => err("Email atau kata sandi salah."),
            "USER_DISABLED" => err("Akun ini dinonaktifkan."),
            other if other.starts_with("TOO_MANY_ATTEMPTS") => {
                err("Terlalu banyak percobaan masuk. Tunggu beberapa menit.")
            }
            other if other.starts_with("NETWORK") => {
                err("Firebase Auth tidak dapat dihubungi. Pastikan emulator berjalan.")
            }
            _ => err("Gagal masuk. Coba lagi."),
        })?;
    simpan_sesi(&s.id_token, &s.refresh_token);
    leptos_axum::redirect("/");
    Ok(())
}

#[server]
pub async fn keluar() -> Result<(), ServerFnError> {
    srv::hapus_sesi();
    leptos_axum::redirect("/login");
    Ok(())
}

#[server]
pub async fn saya() -> Result<Option<Profil>, ServerFnError> {
    srv::pengguna().await
}

#[server]
pub async fn ambil_dashboard() -> Result<Option<Dashboard>, ServerFnError> {
    use srv::*;
    let Some(user) = pengguna().await? else {
        return Ok(None);
    };
    let dms = semua_dm().await?;
    let penilaian = semua_penilaian().await?;
    let konsensus = konsensus().await?;
    let level_maks = dms.iter().map(|d| d.level).max().unwrap_or(0);
    let status: Vec<StatusDm> = dms
        .iter()
        .map(|d| {
            let p = penilaian.iter().find(|p| p.uid == d.uid);
            StatusDm {
                uid: d.uid.clone(),
                nama: d.nama.clone(),
                jabatan: d.jabatan.clone(),
                level: d.level,
                selesai: p.is_some(),
                dikirim_pada: p.map(|p| p.dikirim_pada.clone()),
            }
        })
        .collect();
    Ok(Some(Dashboard {
        semua_selesai: !status.is_empty() && status.iter().all(|s| s.selesai),
        saya_tertinggi: user.level == level_maks,
        saya_selesai: status.iter().any(|s| s.uid == user.uid && s.selesai),
        dms: status,
        konsensus,
        alternatif: alternatif().await?,
        kriteria: kriteria().await?,
        user,
    }))
}

#[server]
pub async fn ambil_form_penilaian() -> Result<Option<FormPenilaian>, ServerFnError> {
    use srv::*;
    let Some(user) = pengguna().await? else {
        return Ok(None);
    };
    let tersimpan = semua_penilaian()
        .await?
        .into_iter()
        .find(|p| p.uid == user.uid);
    Ok(Some(FormPenilaian {
        terkunci: konsensus().await?.is_some(),
        kriteria: kriteria().await?,
        alternatif: alternatif().await?,
        tersimpan,
        user,
    }))
}

#[server(input = Json)]
pub async fn kirim_penilaian(input: PenilaianInput) -> Result<Penilaian, ServerFnError> {
    use srv::*;
    let user = wajib_masuk().await?;
    if konsensus().await?.is_some() {
        return Err(err(
            "Konsensus sudah diproses, penilaian tidak dapat diubah lagi.",
        ));
    }
    let kriteria = kriteria().await?;
    let alternatif = alternatif().await?;

    let mut urut = input.urutan_kriteria.clone();
    urut.sort();
    let mut kode_k: Vec<String> = kriteria.iter().map(|k| k.kode.clone()).collect();
    kode_k.sort();
    if urut != kode_k {
        return Err(err(
            "Urutan prioritas harus memuat setiap kriteria tepat satu kali.",
        ));
    }
    for a in &alternatif {
        let Some(baris) = input.nilai.get(&a.kode) else {
            return Err(err(format!("Nilai untuk {} belum diisi.", a.nama)));
        };
        for k in &kriteria {
            match baris.get(&k.kode) {
                Some(n) if (1..=5).contains(n) => {}
                _ => {
                    return Err(err(format!(
                        "Nilai {} untuk {} belum diisi.",
                        k.nama, a.nama
                    )))
                }
            }
        }
    }
    if input.nilai.len() != alternatif.len() {
        return Err(err("Data penilaian memuat alternatif yang tidak dikenal."));
    }

    let bobot = crate::gdss::bobot_dari_urutan(&input.urutan_kriteria);
    let skor = crate::gdss::saw(&kriteria, &input.nilai, &bobot);
    let ranking = crate::gdss::ranking(&skor);
    let p = Penilaian {
        uid: user.uid.clone(),
        nama: user.nama.clone(),
        jabatan: user.jabatan.clone(),
        level: user.level,
        urutan_kriteria: input.urutan_kriteria,
        bobot,
        nilai: input.nilai,
        skor,
        ranking,
        dikirim_pada: sekarang(),
    };
    simpan(&format!("sesi/{SESI_ID}/penilaian/{}", user.uid), &p).await?;
    Ok(p)
}

#[server]
pub async fn proses_konsensus() -> Result<Konsensus, ServerFnError> {
    use srv::*;
    let user = wajib_masuk().await?;
    let dms = semua_dm().await?;
    let tertinggi = dms
        .first()
        .cloned()
        .ok_or_else(|| err("Belum ada decision maker terdaftar."))?;
    if user.level != tertinggi.level {
        return Err(err(format!(
            "Voting hanya dapat diproses oleh {} sebagai jabatan tertinggi.",
            tertinggi.jabatan
        )));
    }
    if konsensus().await?.is_some() {
        return Err(err("Konsensus untuk sesi ini sudah diproses."));
    }
    let penilaian = semua_penilaian().await?;
    let belum: Vec<String> = dms
        .iter()
        .filter(|d| !penilaian.iter().any(|p| p.uid == d.uid))
        .map(|d| d.jabatan.clone())
        .collect();
    if !belum.is_empty() {
        return Err(err(format!(
            "Masih menunggu penilaian dari {}.",
            belum.join(", ")
        )));
    }
    let ranking_tertinggi = penilaian
        .iter()
        .find(|p| p.uid == user.uid)
        .map(|p| p.ranking.clone())
        .unwrap_or_default();
    let rankings: Vec<Vec<String>> = penilaian.iter().map(|p| p.ranking.clone()).collect();
    let h = crate::gdss::borda(&rankings, &ranking_tertinggi);
    let catatan = if h.ada_seri {
        format!(
            "Ada alternatif dengan poin sama. Urutannya ditentukan oleh ranking individual {}.",
            user.jabatan
        )
    } else {
        "Tidak ada poin seri.".to_string()
    };
    let k = Konsensus {
        metode: "Borda count".to_string(),
        pemenang: h.ranking.first().cloned().unwrap_or_default(),
        poin: h.poin,
        ranking: h.ranking,
        diproses_oleh: user.nama.clone(),
        jabatan_pemroses: user.jabatan.clone(),
        diproses_pada: sekarang(),
        catatan,
    };
    simpan(&format!("konsensus/{SESI_ID}"), &k).await?;
    Ok(k)
}

#[server]
pub async fn ambil_hasil() -> Result<Option<HasilData>, ServerFnError> {
    use srv::*;
    let Some(user) = pengguna().await? else {
        return Ok(None);
    };
    let konsensus = konsensus().await?;
    // Penilaian individual baru dibuka setelah konsensus agar tidak memengaruhi DM lain.
    let penilaian = if konsensus.is_some() {
        semua_penilaian().await?
    } else {
        vec![]
    };
    Ok(Some(HasilData {
        jumlah_dm: semua_dm().await?.len(),
        konsensus,
        penilaian,
        alternatif: alternatif().await?,
        kriteria: kriteria().await?,
        user,
    }))
}
