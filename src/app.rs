use crate::api::*;
use crate::gdss::roc;
use crate::model::*;
use leptos::ev::SubmitEvent;
use leptos::prelude::*;
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{Redirect, Route, Router, Routes, A},
    StaticSegment,
};
use std::collections::BTreeMap;

const MEREK: &str = "Ruang Keputusan SIEM";

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="id">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                // Tema pilihan pengguna dipasang sebelum CSS dimuat agar tidak berkedip.
                <script>
                    "try{var t=localStorage.getItem('tema');if(t)document.documentElement.setAttribute('data-theme',t)}catch(e){}"
                </script>
                <link rel="icon" href="data:,"/>
                <link rel="preload" href="/fonts/ibm-plex-sans-latin-400-normal.woff2" r#as="font" r#type="font/woff2" crossorigin="anonymous"/>
                <AutoReload options=options.clone()/>
                <HydrationScripts options/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    view! {
        <Stylesheet id="leptos" href="/pkg/gdss-siem.css"/>
        <Title text=MEREK/>
        <Router>
            <Routes fallback=|| view! { <TidakDitemukan/> }>
                <Route path=StaticSegment("") view=HalamanBeranda/>
                <Route path=StaticSegment("login") view=HalamanMasuk/>
                <Route path=StaticSegment("penilaian") view=HalamanPenilaian/>
                <Route path=StaticSegment("hasil") view=HalamanHasil/>
            </Routes>
        </Router>
    }
}

/// Persentase bulat untuk tampilan ringkas (mis. 0.4567 -> "46%").
fn persen_bulat(x: f64) -> String {
    format!("{}%", (x * 100.0).round() as i64)
}

/// Skor indeks 0..100 bulat untuk tampilan ringkas SAW (mis. 0.9687 -> 97).
fn indeks_100(x: f64) -> i64 {
    (x * 100.0).round() as i64
}

// ------------------------------------------------------------------ kerangka

#[component]
fn Kerangka(user: Profil, children: Children) -> impl IntoView {
    let keluar = ServerAction::<Keluar>::new();
    view! {
        <a class="lewati" href="#isi">"Lewati ke konten utama"</a>
        <header class="kepala">
            <div class="kepala-kiri">
                <a href="/" class="merek">{MEREK}</a>
                <nav aria-label="Navigasi utama" class="nav">
                    <A href="/" exact=true>"Beranda"</A>
                    <A href="/penilaian">"Penilaian saya"</A>
                    <A href="/hasil">"Hasil"</A>
                </nav>
            </div>
            <div class="akun">
                <p class="akun-id">
                    <span class="akun-jabatan">{user.jabatan}</span>
                    <span class="akun-nama">{user.nama}</span>
                </p>
                <TombolTema/>
                <ActionForm action=keluar>
                    <button type="submit" class="btn-teks">"Keluar"</button>
                </ActionForm>
            </div>
        </header>
        <main id="isi" class="isi">{children()}</main>
        <footer class="kaki">
            <p>"GDSS Pemilihan SIEM Tim SOC · Praktikum Sistem Informasi"</p>
        </footer>
    }
}

#[component]
fn TombolTema() -> impl IntoView {
    let ganti = move |_| {
        #[cfg(feature = "hydrate")]
        {
            if let Some(el) = document().document_element() {
                let gelap = match el.get_attribute("data-theme").as_deref() {
                    Some("dark") => true,
                    Some("light") => false,
                    _ => window()
                        .match_media("(prefers-color-scheme: dark)")
                        .ok()
                        .flatten()
                        .map(|m| m.matches())
                        .unwrap_or(false),
                };
                let baru = if gelap { "light" } else { "dark" };
                let _ = el.set_attribute("data-theme", baru);
                if let Ok(Some(s)) = window().local_storage() {
                    let _ = s.set_item("tema", baru);
                }
            }
        }
    };
    view! {
        <button type="button" class="btn-teks" on:click=ganti>"Ganti tema"</button>
    }
}

#[component]
fn Memuat(teks: &'static str) -> impl IntoView {
    view! {
        <main id="isi" class="isi">
            <div class="panel memuat" role="status" aria-live="polite">
                <span class="lampu kedip" aria-hidden="true"></span>
                <span>{teks}</span>
            </div>
        </main>
    }
}

#[component]
fn Galat(pesan: String, ulang: Callback<()>) -> impl IntoView {
    view! {
        <main id="isi" class="isi">
            <div class="panel galat" role="alert">
                <h1>"Data gagal dimuat"</h1>
                <p>{pesan}</p>
                <button type="button" class="btn" on:click=move |_| ulang.run(())>
                    "Muat ulang data"
                </button>
            </div>
        </main>
    }
}

#[component]
fn TidakDitemukan() -> impl IntoView {
    view! {
        <Title text=format!("Halaman tidak ada · {MEREK}")/>
        <main id="isi" class="isi">
            <div class="panel">
                <h1>"Halaman tidak ditemukan"</h1>
                <p>"Alamat yang diminta tidak tersedia di aplikasi ini."</p>
                <a class="btn" href="/">"Kembali ke beranda"</a>
            </div>
        </main>
    }
}

// ------------------------------------------------------------------ masuk

#[component]
fn HalamanMasuk() -> impl IntoView {
    let masuk = ServerAction::<Masuk>::new();
    let sudah = Resource::new(|| (), |_| saya());
    let email_val = RwSignal::new(String::new());
    let pass_val = RwSignal::new(String::new());

    let isi_demo = move |e: &'static str, p: &'static str| {
        email_val.set(e.to_string());
        pass_val.set(p.to_string());
    };

    let galat = move || match masuk.value().get() {
        Some(Err(e)) => Some(view! { <p class="pesan galat-teks" role="alert">{pesan(&e)}</p> }),
        _ => None,
    };

    view! {
        <Title text=format!("Masuk · {MEREK}")/>
        <Suspense fallback=|| ()>
            {move || Suspend::new(async move {
                match sudah.await {
                    Ok(Some(_)) => view! { <Redirect path="/"/> }.into_any(),
                    _ => ().into_any(),
                }
            })}
        </Suspense>
        <main id="isi" class="halaman-masuk">
            <div class="kartu-masuk">
                <div class="masuk-atas">
                    <p class="merek">{MEREK}</p>
                    <TombolTema/>
                </div>
                <h1>"Masuk sebagai decision maker"</h1>
                <p class="pengantar">
                    "Gunakan akun sesuai jabatan Anda. Penilaian bersifat independen dan baru dibuka setelah konsensus ditetapkan."
                </p>
                <ActionForm action=masuk>
                    <div class="form">
                        <label for="email">"Email"</label>
                        <input
                            id="email"
                            name="email"
                            type="email"
                            autocomplete="username"
                            required
                            prop:value=move || email_val.get()
                            on:input=move |ev| email_val.set(event_target_value(&ev))
                        />
                        <label for="password">"Kata sandi"</label>
                        <input
                            id="password"
                            name="password"
                            type="password"
                            autocomplete="current-password"
                            required
                            prop:value=move || pass_val.get()
                            on:input=move |ev| pass_val.set(event_target_value(&ev))
                        />
                        {galat}
                        <button type="submit" class="btn utama penuh" disabled=move || masuk.pending().get()>
                            {move || if masuk.pending().get() { "Memeriksa akun…" } else { "Masuk" }}
                        </button>
                    </div>
                </ActionForm>

                <details class="demo-akun">
                    <summary>"Akun demo untuk pengujian"</summary>
                    <p class="kecil redup">"Klik salah satu jabatan untuk mengisi kolom email dan sandi secara otomatis:"</p>
                    <div class="demo-daftar">
                        <button type="button" class="btn-demo" on:click=move |_| isi_demo("ciso@gdss-siem.test", "ciso-gdss-2026")>
                            <span>"CISO"</span>
                            <span class="redup kecil">"Jabatan tertinggi · voting"</span>
                        </button>
                        <button type="button" class="btn-demo" on:click=move |_| isi_demo("soc@gdss-siem.test", "soc-gdss-2026")>
                            <span>"SOC Analyst"</span>
                            <span class="redup kecil">"Operasional deteksi"</span>
                        </button>
                        <button type="button" class="btn-demo" on:click=move |_| isi_demo("netadmin@gdss-siem.test", "netadmin-gdss-2026")>
                            <span>"Network Admin"</span>
                            <span class="redup kecil">"Infrastruktur jaringan"</span>
                        </button>
                    </div>
                </details>
            </div>
        </main>
    }
}

// ------------------------------------------------------------------ beranda

#[component]
fn HalamanBeranda() -> impl IntoView {
    let proses = ServerAction::<ProsesKonsensus>::new();
    let data = Resource::new(move || proses.version().get(), |_| ambil_dashboard());
    view! {
        <Title text=format!("Beranda · {MEREK}")/>
        <Suspense fallback=|| view! { <Memuat teks="Memuat status sesi…"/> }>
            {move || Suspend::new(async move {
                match data.await {
                    Ok(Some(d)) => view! { <Beranda d=d proses=proses/> }.into_any(),
                    Ok(None) => view! { <Redirect path="/login"/> }.into_any(),
                    Err(e) => {
                        view! { <Galat pesan=pesan(&e) ulang=Callback::new(move |_| data.refetch())/> }
                            .into_any()
                    }
                }
            })}
        </Suspense>
    }
}

#[component]
fn Beranda(d: Dashboard, proses: ServerAction<ProsesKonsensus>) -> impl IntoView {
    let jumlah = d.dms.len();
    let masuk = d.dms.iter().filter(|s| s.selesai).count();
    let tertinggi = d.dms.first().map(|s| s.jabatan.clone()).unwrap_or_default();
    let level_maks = d.dms.first().map(|s| s.level).unwrap_or(0);
    let alternatif = d.alternatif.clone();
    let tab_info = RwSignal::new(0u8); // 0 = kandidat, 1 = kriteria

    let (tahap, status, aksi) = if let Some(k) = &d.konsensus {
        (
            "Keputusan ditetapkan",
            format!(
                "{} terpilih sebagai solusi utama melalui voting Borda oleh {}.",
                nama_alt(&alternatif, &k.pemenang),
                k.jabatan_pemroses
            ),
            view! { <a class="btn utama" href="/hasil">"Buka rincian hasil"</a> }.into_any(),
        )
    } else if !d.saya_selesai {
        (
            "Tahap 1 dari 2 · Penilaian individual",
            format!("Anda belum mengirim penilaian ({masuk} dari {jumlah} penilai sudah selesai)."),
            view! { <a class="btn utama" href="/penilaian">"Isi penilaian saya"</a> }.into_any(),
        )
    } else if d.semua_selesai && d.saya_tertinggi {
        (
            "Tahap 2 dari 2 · Siap konsensus",
            format!(
                "Seluruh penilaian sudah lengkap. Sebagai {}, Anda dapat menetapkan hasil lewat voting Borda.",
                d.user.jabatan
            ),
            view! {
                <ActionForm action=proses>
                    <button type="submit" class="btn utama" disabled=move || proses.pending().get()>
                        {move || if proses.pending().get() { "Memproses voting…" } else { "Proses voting Borda" }}
                    </button>
                </ActionForm>
                <a class="btn" href="/penilaian">"Tinjau penilaian saya"</a>
            }
            .into_any(),
        )
    } else if d.semua_selesai {
        (
            "Tahap 2 dari 2 · Menunggu penetapan",
            format!(
                "Seluruh penilaian sudah lengkap. Menunggu {tertinggi} memproses voting Borda."
            ),
            view! { <a class="btn" href="/penilaian">"Ubah penilaian saya"</a> }.into_any(),
        )
    } else {
        (
            "Tahap 1 dari 2 · Penilaian individual",
            format!(
                "Penilaian Anda sudah tersimpan. Menunggu {} penilai lagi sebelum voting dibuka.",
                jumlah - masuk
            ),
            view! { <a class="btn" href="/penilaian">"Ubah penilaian saya"</a> }.into_any(),
        )
    };

    let galat_proses = move || match proses.value().get() {
        Some(Err(e)) => Some(view! { <p class="pesan galat-teks" role="alert">{pesan(&e)}</p> }),
        _ => None,
    };

    let lampu_meter = d
        .dms
        .iter()
        .map(|s| view! { <span class="lampu besar" class:nyala=s.selesai></span> })
        .collect_view();

    let daftar_dm = if d.dms.is_empty() {
        view! {
            <p class="kosong">"Belum ada penilai terdaftar. Jalankan skrip seed untuk mengisi data awal."</p>
        }
        .into_any()
    } else {
        let uid_saya = d.user.uid.clone();
        view! {
            <ul class="daftar-dm">
                {d.dms.iter().map(|s| {
                    let ket_status = if s.selesai { "Selesai menilai" } else { "Belum menilai" };
                    let waktu = s.dikirim_pada.as_deref().map(waktu_tampil);
                    let peran = if s.level == level_maks { "Penentu voting" } else { "Penilai" };
                    let anda = if s.uid == uid_saya { " · Anda" } else { "" };
                    view! {
                        <li>
                            <span class="lampu" class:nyala=s.selesai aria-hidden="true"></span>
                            <div class="dm-id">
                                <p class="dm-nama">{s.jabatan.clone()}<span class="redup normal">{anda}</span></p>
                                <p class="dm-jabatan">{format!("{} · {peran}", s.nama)}</p>
                            </div>
                            <div class="dm-kanan">
                                <p class="dm-status" class:redup=!s.selesai>{ket_status}</p>
                                {waktu.map(|w| view! { <p class="dm-waktu redup kecil">{w}</p> })}
                            </div>
                        </li>
                    }
                }).collect_view()}
            </ul>
        }
        .into_any()
    };

    let poin_maks = ((d.alternatif.len().saturating_sub(1)) * jumlah).max(1) as f64;
    let panel_hasil = match &d.konsensus {
        Some(k) => {
            let alt = alternatif.clone();
            view! {
                <ol class="bar-daftar">
                    {k.ranking.iter().enumerate().map(|(i, kode)| {
                        let poin = k.poin.get(kode).copied().unwrap_or(0);
                        let lebar = format!("width: {:.1}%", (poin as f64 / poin_maks) * 100.0);
                        view! {
                            <li class="bar-item" class:juara=i == 0>
                                <div class="bar-atas">
                                    <span class="bar-judul">
                                        <span class="no">{i + 1}</span>
                                        <span class="alt">{nama_alt(&alt, kode)}</span>
                                    </span>
                                    <span class="poin">{format!("{poin} poin")}</span>
                                </div>
                                <div class="bar-rel" aria-hidden="true">
                                    <span class="bar-isi" style=lebar></span>
                                </div>
                            </li>
                        }
                    }).collect_view()}
                </ol>
                <p class="catatan">{k.catatan.clone()}</p>
            }
            .into_any()
        }
        None => view! {
            <div class="kosong-kotak">
                <p class="kosong">
                    "Hasil konsensus akan tampil di sini setelah ketiga penilai selesai dan "
                    {tertinggi.clone()}
                    " memproses voting Borda."
                </p>
            </div>
        }
        .into_any(),
    };

    let jml_alt = d.alternatif.len();
    let jml_krit = d.kriteria.len();
    let alt_list = d.alternatif.clone();
    let krit_list = d.kriteria.clone();

    view! {
        <Kerangka user=d.user.clone()>
            <section class="panel fokus" aria-labelledby="judul-sesi">
                <p class="label-sesi">{tahap}</p>
                <h1 id="judul-sesi">"Pemilihan SIEM untuk Tim SOC"</h1>
                <p class="status">{status}</p>
                <div class="aksi">{aksi}</div>
                {galat_proses}
            </section>

            <div class="grid-dua">
                <section class="panel" aria-labelledby="judul-dm">
                    <div class="kepala-panel">
                        <h2 id="judul-dm">"Progres penilai"</h2>
                        <p class="hitung">{format!("{masuk} dari {jumlah} selesai")}</p>
                    </div>
                    <div class="meter" aria-hidden="true">{lampu_meter}</div>
                    {daftar_dm}
                </section>

                <section class="panel" aria-labelledby="judul-hasil">
                    <div class="kepala-panel">
                        <h2 id="judul-hasil">"Keputusan kelompok"</h2>
                        {d.konsensus.as_ref().map(|_| view! { <a class="tautan-kecil" href="/hasil">"Rincian lengkap"</a> })}
                    </div>
                    {panel_hasil}
                </section>
            </div>

            <section class="panel" aria-labelledby="judul-referensi">
                <div class="kepala-tab">
                    <div>
                        <h2 id="judul-referensi">"Daftar kandidat & kriteria"</h2>
                        <p class="kecil redup tanpa-margin">"Klik baris mana pun untuk membuka spesifikasi atau rubrik penilaian."</p>
                    </div>
                    <div class="tab-bar" role="tablist" aria-label="Data acuan sesi">
                        <button
                            type="button"
                            role="tab"
                            id="tab-kandidat"
                            aria-controls="panel-kandidat"
                            aria-selected=move || (tab_info.get() == 0).to_string()
                            class="tab-btn"
                            class:aktif=move || tab_info.get() == 0
                            on:click=move |_| tab_info.set(0)
                        >
                            {format!("Kandidat ({jml_alt})")}
                        </button>
                        <button
                            type="button"
                            role="tab"
                            id="tab-kriteria"
                            aria-controls="panel-kriteria"
                            aria-selected=move || (tab_info.get() == 1).to_string()
                            class="tab-btn"
                            class:aktif=move || tab_info.get() == 1
                            on:click=move |_| tab_info.set(1)
                        >
                            {format!("Kriteria ({jml_krit})")}
                        </button>
                    </div>
                </div>

                {move || if tab_info.get() == 0 {
                    if alt_list.is_empty() {
                        view! {
                            <div id="panel-kandidat" role="tabpanel" aria-labelledby="tab-kandidat">
                                <p class="kosong">"Belum ada kandidat di database."</p>
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <div id="panel-kandidat" role="tabpanel" aria-labelledby="tab-kandidat" class="akordeon-daftar">
                                {alt_list.iter().map(|a| view! {
                                    <details class="akordeon">
                                        <summary>
                                            <span class="kode">{a.kode.clone()}</span>
                                            <span class="akordeon-utama">{a.nama.clone()}</span>
                                            <span class="akordeon-meta">{format!("{} · {}", a.kategori, a.lisensi)}</span>
                                        </summary>
                                        <p class="akordeon-isi">{a.ringkasan.clone()}</p>
                                    </details>
                                }).collect_view()}
                            </div>
                        }.into_any()
                    }
                } else {
                    view! {
                        <div id="panel-kriteria" role="tabpanel" aria-labelledby="tab-kriteria" class="akordeon-daftar">
                            {krit_list.iter().map(|k| {
                                let sifat = match k.jenis {
                                    Jenis::Benefit => "Manfaat · skor tinggi lebih baik",
                                    Jenis::Cost => "Biaya · skor rendah lebih hemat",
                                };
                                view! {
                                    <details class="akordeon">
                                        <summary>
                                            <span class="kode">{k.kode.clone()}</span>
                                            <span class="akordeon-utama">{k.nama.clone()}</span>
                                            <span class="akordeon-meta">{sifat}</span>
                                        </summary>
                                        <div class="akordeon-isi">
                                            <p>{k.deskripsi.clone()}</p>
                                            <ol class="rubrik">
                                                {k.skala.iter().map(|s| view! { <li>{s.clone()}</li> }).collect_view()}
                                            </ol>
                                        </div>
                                    </details>
                                }
                            }).collect_view()}
                        </div>
                    }.into_any()
                }}
            </section>
        </Kerangka>
    }
}

// ------------------------------------------------------------------ penilaian

#[component]
fn HalamanPenilaian() -> impl IntoView {
    let data = Resource::new(|| (), |_| ambil_form_penilaian());
    view! {
        <Title text=format!("Penilaian saya · {MEREK}")/>
        <Suspense fallback=|| view! { <Memuat teks="Memuat lembar penilaian…"/> }>
            {move || Suspend::new(async move {
                match data.await {
                    Ok(Some(f)) => {
                        let user = f.user.clone();
                        view! { <Kerangka user=user><FormNilai f=f/></Kerangka> }.into_any()
                    }
                    Ok(None) => view! { <Redirect path="/login"/> }.into_any(),
                    Err(e) => {
                        view! { <Galat pesan=pesan(&e) ulang=Callback::new(move |_| data.refetch())/> }
                            .into_any()
                    }
                }
            })}
        </Suspense>
    }
}

#[component]
fn FormNilai(f: FormPenilaian) -> impl IntoView {
    let kirim = ServerAction::<KirimPenilaian>::new();
    let kriteria = StoredValue::new(f.kriteria.clone());
    let alternatif = StoredValue::new(f.alternatif.clone());
    let terkunci = f.terkunci;
    let ada_tersimpan = f.tersimpan.is_some();

    let urutan = RwSignal::new(
        f.tersimpan
            .as_ref()
            .map(|p| p.urutan_kriteria.clone())
            .unwrap_or_else(|| f.kriteria.iter().map(|k| k.kode.clone()).collect()),
    );
    let awal: BTreeMap<String, BTreeMap<String, u8>> = f
        .tersimpan
        .as_ref()
        .map(|p| p.nilai.clone())
        .unwrap_or_default();
    let nilai = RwSignal::new(awal);
    let tab_alt = RwSignal::new(0usize);
    let pesan_lokal = RwSignal::new(None::<String>);

    let jml_krit = f.kriteria.len();
    let jml_alt = f.alternatif.len();
    let jumlah_sel = jml_krit * jml_alt;

    let terisi_total = move || {
        nilai.with(|m| {
            m.values()
                .map(|r| r.values().filter(|v| (1..=5).contains(*v)).count())
                .sum::<usize>()
        })
    };

    let geser = move |i: usize, naik: bool| {
        urutan.update(|u| {
            let j = if naik { i - 1 } else { i + 1 };
            u.swap(i, j);
        })
    };

    let on_submit = move |ev: SubmitEvent| {
        ev.prevent_default();
        // Cari kandidat pertama yang belum lengkap, lalu arahkan tab langsung ke sana.
        let belum = alternatif.with_value(|alts| {
            kriteria.with_value(|ks| {
                alts.iter().enumerate().find_map(|(idx, a)| {
                    let kurang = nilai.with(|m| {
                        ks.iter()
                            .filter(|k| {
                                !m.get(&a.kode)
                                    .and_then(|r| r.get(&k.kode))
                                    .map(|v| (1..=5).contains(v))
                                    .unwrap_or(false)
                            })
                            .count()
                    });
                    (kurang > 0).then(|| (idx, a.nama.clone(), kurang))
                })
            })
        });

        if let Some((idx, nama_kandidat, kurang)) = belum {
            tab_alt.set(idx);
            let sisa = jumlah_sel - terisi_total();
            pesan_lokal.set(Some(format!(
                "Masih ada {sisa} skor kosong. Lengkapi {kurang} kriteria pada kandidat {nama_kandidat}."
            )));
            return;
        }
        pesan_lokal.set(None);
        kirim.dispatch(KirimPenilaian {
            input: PenilaianInput {
                urutan_kriteria: urutan.get(),
                nilai: nilai.get(),
            },
        });
    };

    let bobot = StoredValue::new(roc(jml_krit));
    let bobot_maks = bobot.with_value(|b| b.first().copied().unwrap_or(1.0).max(0.01));

    let daftar_prioritas = move || {
        let n = urutan.with(|u| u.len());
        urutan
            .get()
            .into_iter()
            .enumerate()
            .map(|(i, kode)| {
                let k = kriteria
                    .with_value(|ks| ks.iter().find(|k| k.kode == kode).cloned())
                    .expect("kriteria tersedia");
                let w = bobot.with_value(|b| b[i]);
                let w_persen = persen_bulat(w);
                let lebar = format!("width: {:.1}%", (w / bobot_maks) * 100.0);
                let nama_naik = format!("Naikkan prioritas {}", k.nama);
                let nama_turun = format!("Turunkan prioritas {}", k.nama);
                let sifat = match k.jenis {
                    Jenis::Benefit => "Manfaat",
                    Jenis::Cost => "Biaya",
                };
                view! {
                    <li class="prioritas-baris">
                        <span class="no">{i + 1}</span>
                        <div class="prioritas-utama">
                            <div class="prioritas-judul">
                                <span class="prioritas-nama">{k.nama.clone()}</span>
                                <span class="tag-sifat">{sifat}</span>
                            </div>
                            <div class="bobot-mini" aria-hidden="true">
                                <span class="bobot-isi" style=lebar></span>
                            </div>
                            <details class="detail-mini">
                                <summary>"Penjelasan & skala 1 sampai 5"</summary>
                                <p class="kecil redup">{k.deskripsi.clone()}</p>
                                <ol class="rubrik">
                                    {k.skala.iter().map(|s| view! { <li>{s.clone()}</li> }).collect_view()}
                                </ol>
                            </details>
                        </div>
                        <p class="bobot-angka" title=format!("Bobot desimal ROC: {}", desimal(w, 4))>
                            {w_persen}
                        </p>
                        <div class="geser">
                            <button
                                type="button"
                                class="btn-kecil"
                                aria-label=nama_naik
                                disabled=terkunci || i == 0
                                on:click=move |_| geser(i, true)
                            >
                                "Naik"
                            </button>
                            <button
                                type="button"
                                class="btn-kecil"
                                aria-label=nama_turun
                                disabled=terkunci || i + 1 == n
                                on:click=move |_| geser(i, false)
                            >
                                "Turun"
                            </button>
                        </div>
                    </li>
                }
            })
            .collect_view()
    };

    let bobot_rinci = move || {
        urutan
            .get()
            .into_iter()
            .enumerate()
            .map(|(i, kode)| {
                let nama = kriteria.with_value(|ks| {
                    ks.iter()
                        .find(|k| k.kode == kode)
                        .map(|k| k.nama.clone())
                        .unwrap_or(kode.clone())
                });
                let w = bobot.with_value(|b| b[i]);
                view! {
                    <tr>
                        <td class="kode">{i + 1}</td>
                        <td>{format!("{kode} · {nama}")}</td>
                        <td class="angka-kanan">{desimal(w, 4)}</td>
                    </tr>
                }
            })
            .collect_view()
    };

    // Tab pemilih kandidat di Langkah 2
    let pemilih_kandidat = move || {
        alternatif.with_value(|alts| {
            alts.iter()
                .enumerate()
                .map(|(idx, a)| {
                    let ak = a.kode.clone();
                    let jml_terisi = Memo::new(move |_| {
                        nilai.with(|m| {
                            m.get(&ak)
                                .map(|r| r.values().filter(|v| (1..=5).contains(*v)).count())
                                .unwrap_or(0)
                        })
                    });
                    let lengkap = move || jml_terisi.get() == jml_krit;
                    let nama = a.nama.clone();
                    view! {
                        <button
                            type="button"
                            role="tab"
                            id=format!("tab-alt-{idx}")
                            aria-controls="panel-skor-aktif"
                            aria-selected=move || (tab_alt.get() == idx).to_string()
                            class="kandidat-tab"
                            class:aktif=move || tab_alt.get() == idx
                            on:click=move |_| tab_alt.set(idx)
                        >
                            <span class="lampu" class:nyala=lengkap aria-hidden="true"></span>
                            <span class="kandidat-tab-nama">{nama}</span>
                            <span class="kandidat-tab-hitung">{move || format!("{}/{jml_krit}", jml_terisi.get())}</span>
                        </button>
                    }
                })
                .collect_view()
        })
    };

    // Kartu kandidat yang sedang dipilih (hanya 5 baris kriteria, bebas tumpukan dropdown)
    let kartu_aktif = move || {
        let idx = tab_alt.get();
        let Some(a) = alternatif.with_value(|alts| alts.get(idx).cloned()) else {
            return ().into_any();
        };
        let nama_sebelum = (idx > 0)
            .then(|| alternatif.with_value(|alts| alts.get(idx - 1).map(|x| x.nama.clone())))
            .flatten();
        let nama_lanjut = alternatif.with_value(|alts| alts.get(idx + 1).map(|x| x.nama.clone()));

        let baris_kriteria = kriteria.with_value(|ks| {
            ks.iter()
                .map(|k| {
                    let ak = a.kode.clone();
                    let kk = k.kode.clone();
                    let skala = k.skala.clone();
                    let sifat = match k.jenis {
                        Jenis::Benefit => "Manfaat",
                        Jenis::Cost => "Biaya · skor 1 paling murah, 5 paling mahal",
                    };
                    let ak_ket = ak.clone();
                    let kk_ket = kk.clone();
                    let ket_pilihan = move || {
                        let skor = nilai.with(|m| m.get(&ak_ket).and_then(|r| r.get(&kk_ket)).copied().unwrap_or(0));
                        if (1..=5).contains(&skor) {
                            let teks = skala.get((skor - 1) as usize).cloned().unwrap_or_default();
                            format!("Skor {skor}: {teks}")
                        } else {
                            "Belum dipilih. Klik angka 1 sampai 5.".to_string()
                        }
                    };
                    let ak_cek = ak.clone();
                    let kk_cek = kk.clone();
                    let sudah_pilih = move || {
                        nilai.with(|m| {
                            m.get(&ak_cek)
                                .and_then(|r| r.get(&kk_cek))
                                .map(|v| (1..=5).contains(v))
                                .unwrap_or(false)
                        })
                    };

                    let tombol_skor = (1u8..=5)
                        .map(|n| {
                            let ak_klik = ak.clone();
                            let kk_klik = kk.clone();
                            let ak_aktif = ak.clone();
                            let kk_aktif = kk.clone();
                            let ket_n = k.skala.get((n - 1) as usize).cloned().unwrap_or_default();
                            let label_aria = format!("Skor {n} untuk {} pada {}: {ket_n}", k.nama, a.nama);
                            let terpilih = Memo::new(move |_| {
                                nilai.with(|m| m.get(&ak_aktif).and_then(|r| r.get(&kk_aktif)).copied() == Some(n))
                            });
                            view! {
                                <button
                                    type="button"
                                    class="skor-btn"
                                    class:terpilih=move || terpilih.get()
                                    aria-pressed=move || terpilih.get().to_string()
                                    aria-label=label_aria
                                    title=format!("{n}. {ket_n}")
                                    disabled=terkunci
                                    on:click=move |_| {
                                        pesan_lokal.set(None);
                                        nilai.update(|m| {
                                            m.entry(ak_klik.clone()).or_default().insert(kk_klik.clone(), n);
                                        });
                                    }
                                >
                                    {n}
                                </button>
                            }
                        })
                        .collect_view();

                    view! {
                        <div class="baris-skor" role="group" aria-label=format!("{} untuk {}", k.nama, a.nama)>
                            <div class="baris-skor-info">
                                <p class="baris-skor-judul">
                                    <span class="kode">{k.kode.clone()}</span>
                                    <span>{k.nama.clone()}</span>
                                    <span class="tag-sifat">{sifat}</span>
                                </p>
                                <p class="baris-skor-ket" class:redup=move || !sudah_pilih()>{ket_pilihan}</p>
                            </div>
                            <div class="skor-grup">{tombol_skor}</div>
                        </div>
                    }
                })
                .collect_view()
        });

        view! {
            <div id="panel-skor-aktif" role="tabpanel" aria-labelledby=format!("tab-alt-{idx}") class="kartu-kandidat">
                <div class="kandidat-kepala">
                    <div>
                        <p class="kecil redup tanpa-margin">{format!("Kandidat {} dari {jml_alt} · {} · {}", idx + 1, a.kategori, a.lisensi)}</p>
                        <h3 class="kandidat-judul">{a.nama.clone()}</h3>
                    </div>
                </div>
                <p class="alt-ringkas">{a.ringkasan.clone()}</p>

                <div class="daftar-baris-skor">{baris_kriteria}</div>

                <div class="navigasi-kandidat">
                    {match nama_sebelum {
                        Some(nm) => view! {
                            <button type="button" class="btn" on:click=move |_| tab_alt.set(idx - 1)>
                                {format!("Sebelumnya: {nm}")}
                            </button>
                        }.into_any(),
                        None => view! { <span></span> }.into_any(),
                    }}
                    {match nama_lanjut {
                        Some(nm) => view! {
                            <button type="button" class="btn" on:click=move |_| tab_alt.set(idx + 1)>
                                {format!("Berikutnya: {nm}")}
                            </button>
                        }.into_any(),
                        None => view! { <span class="kecil redup">"Kandidat terakhir. Periksa lalu kirim penilaian di bawah."</span> }.into_any(),
                    }}
                </div>
            </div>
        }
        .into_any()
    };

    // Ringkasan matriks 5x5 opsional
    let matriks_ringkas = move || {
        let kepala = kriteria.with_value(|ks| {
            ks.iter()
                .map(|k| view! { <th scope="col" class="angka-tengah">{k.kode.clone()}</th> })
                .collect_view()
        });
        let baris = alternatif.with_value(|alts| {
            alts.iter()
                .enumerate()
                .map(|(idx, a)| {
                    let sel = kriteria.with_value(|ks| {
                        ks.iter()
                            .map(|k| {
                                let ak = a.kode.clone();
                                let kk = k.kode.clone();
                                let isi = move || {
                                    nilai.with(|m| {
                                        m.get(&ak)
                                            .and_then(|r| r.get(&kk))
                                            .copied()
                                            .filter(|v| (1..=5).contains(v))
                                            .map(|v| v.to_string())
                                            .unwrap_or_else(|| "-".to_string())
                                    })
                                };
                                view! { <td class="angka-tengah">{isi}</td> }
                            })
                            .collect_view()
                    });
                    let nm = a.nama.clone();
                    view! {
                        <tr>
                            <th scope="row">
                                <button type="button" class="tautan-baris" on:click=move |_| tab_alt.set(idx)>
                                    {nm}
                                </button>
                            </th>
                            {sel}
                        </tr>
                    }
                })
                .collect_view()
        });
        view! {
            <div class="tabel-wrap">
                <table>
                    <thead>
                        <tr>
                            <th scope="col">"Kandidat"</th>
                            {kepala}
                        </tr>
                    </thead>
                    <tbody>{baris}</tbody>
                </table>
            </div>
        }
    };

    let hasil_kirim = move || match kirim.value().get() {
        Some(Ok(p)) => {
            let urut = alternatif.with_value(|alts| {
                p.ranking
                    .iter()
                    .enumerate()
                    .map(|(i, kode)| {
                        let s = p.skor.get(kode).copied().unwrap_or(0.0);
                        let lebar = format!("width: {:.1}%", (s * 100.0).clamp(0.0, 100.0));
                        view! {
                            <li class="bar-item" class:juara=i == 0>
                                <div class="bar-atas">
                                    <span class="bar-judul">
                                        <span class="no">{i + 1}</span>
                                        <span class="alt">{nama_alt(alts, kode)}</span>
                                    </span>
                                    <span class="poin" title=format!("Skor SAW: {}", desimal(s, 4))>
                                        {format!("{} / 100", indeks_100(s))}
                                    </span>
                                </div>
                                <div class="bar-rel" aria-hidden="true">
                                    <span class="bar-isi" style=lebar></span>
                                </div>
                            </li>
                        }
                    })
                    .collect_view()
            });
            Some(
                view! {
                    <div class="panel sukses" role="status">
                        <h2>"Penilaian tersimpan"</h2>
                        <p class="redup">{format!("Tercatat {}. Urutan preferensi SAW versi Anda:", waktu_tampil(&p.dikirim_pada))}</p>
                        <ol class="bar-daftar">{urut}</ol>
                        <div class="aksi">
                            <a class="btn utama" href="/">"Kembali ke beranda"</a>
                        </div>
                    </div>
                }
                .into_any(),
            )
        }
        Some(Err(e)) => {
            Some(view! { <p class="pesan galat-teks" role="alert">{pesan(&e)}</p> }.into_any())
        }
        None => None,
    };

    view! {
        <section class="panel fokus" aria-labelledby="judul-form">
            <p class="label-sesi">{format!("Sudut pandang {}", f.user.jabatan)}</p>
            <h1 id="judul-form">"Penilaian Individual"</h1>
            <p class="status">
                {if terkunci {
                    "Konsensus sudah ditetapkan, sehingga lembar penilaian ini dikunci dan hanya dapat ditinjau.".to_string()
                } else {
                    "Susun urutan prioritas kriteria, lalu pilih skor 1 sampai 5 untuk kelima kandidat.".to_string()
                }}
            </p>
        </section>

        <form on:submit=on_submit class="form-nilai" novalidate>
            <section class="panel" aria-labelledby="judul-prioritas">
                <div class="kepala-panel">
                    <h2 id="judul-prioritas">"1. Urutan prioritas kriteria"</h2>
                    <span class="hitung">"Bobot otomatis (ROC)"</span>
                </div>
                <p class="bantu">"Kriteria di urutan teratas memperoleh porsi bobot terbesar."</p>
                <ol class="daftar-prioritas">{daftar_prioritas}</ol>
                <details class="rincian-bawah">
                    <summary>"Lihat angka desimal bobot ROC (4 desimal)"</summary>
                    <div class="tabel-wrap">
                        <table>
                            <thead>
                                <tr>
                                    <th scope="col">"Urutan"</th>
                                    <th scope="col">"Kriteria"</th>
                                    <th scope="col" class="angka-kanan">"Bobot ROC"</th>
                                </tr>
                            </thead>
                            <tbody>{bobot_rinci}</tbody>
                        </table>
                    </div>
                </details>
            </section>

            <section class="panel" aria-labelledby="judul-isian">
                <div class="kepala-panel">
                    <h2 id="judul-isian">"2. Skor kandidat SIEM"</h2>
                    <p class="hitung" aria-live="polite">{move || format!("{} dari {jumlah_sel} skor terisi", terisi_total())}</p>
                </div>
                <div class="bar-rel progres-isian" aria-hidden="true">
                    <span
                        class="bar-isi ok-isi"
                        style=move || format!("width: {:.1}%", (terisi_total() as f64 / jumlah_sel.max(1) as f64) * 100.0)
                    ></span>
                </div>

                <div class="kandidat-tab-bar" role="tablist" aria-label="Pilih kandidat yang dinilai">
                    {pemilih_kandidat}
                </div>

                {kartu_aktif}

                <details class="rincian-bawah">
                    <summary>"Lihat ringkasan matriks 5×5 seluruh skor Anda"</summary>
                    {matriks_ringkas}
                </details>
            </section>

            <div class="kirim">
                {move || pesan_lokal.get().map(|m| view! { <p class="pesan galat-teks" role="alert">{m}</p> })}
                {hasil_kirim}
                <button type="submit" class="btn utama" disabled=move || terkunci || kirim.pending().get()>
                    {move || {
                        if kirim.pending().get() {
                            "Menyimpan penilaian…"
                        } else if ada_tersimpan || matches!(kirim.value().get(), Some(Ok(_))) {
                            "Simpan perubahan penilaian"
                        } else {
                            "Kirim penilaian"
                        }
                    }}
                </button>
            </div>
        </form>
    }
}

// ------------------------------------------------------------------ hasil

#[component]
fn HalamanHasil() -> impl IntoView {
    let data = Resource::new(|| (), |_| ambil_hasil());
    view! {
        <Title text=format!("Hasil · {MEREK}")/>
        <Suspense fallback=|| view! { <Memuat teks="Memuat hasil konsensus…"/> }>
            {move || Suspend::new(async move {
                match data.await {
                    Ok(Some(h)) => {
                        let user = h.user.clone();
                        view! { <Kerangka user=user><Hasil h=h/></Kerangka> }.into_any()
                    }
                    Ok(None) => view! { <Redirect path="/login"/> }.into_any(),
                    Err(e) => {
                        view! { <Galat pesan=pesan(&e) ulang=Callback::new(move |_| data.refetch())/> }
                            .into_any()
                    }
                }
            })}
        </Suspense>
    }
}

#[component]
fn Hasil(h: HasilData) -> impl IntoView {
    let Some(k) = h.konsensus.clone() else {
        return view! {
            <section class="panel fokus" aria-labelledby="judul-hasil-kosong">
                <p class="label-sesi">"Menunggu konsensus"</p>
                <h1 id="judul-hasil-kosong">"Hasil belum tersedia"</h1>
                <p class="status">
                    {format!(
                        "Keputusan kelompok beserta rincian penilaian tiap jabatan akan dibuka setelah seluruh {} penilai selesai dan voting Borda ditetapkan.",
                        h.jumlah_dm
                    )}
                </p>
                <div class="aksi"><a class="btn" href="/">"Kembali ke beranda"</a></div>
            </section>
        }
        .into_any();
    };

    let m = h.alternatif.len();
    let poin_maks_kandidat = m.saturating_sub(1) * h.penilaian.len().max(1);
    let poin_juara = k.poin.get(&k.pemenang).copied().unwrap_or(0);

    let kepala_dm = h
        .penilaian
        .iter()
        .map(|p| view! { <th scope="col" class="angka-tengah">{p.jabatan.clone()}</th> })
        .collect_view();

    let baris_borda = k
        .ranking
        .iter()
        .enumerate()
        .map(|(i, kode)| {
            let poin = k.poin.get(kode).copied().unwrap_or(0);
            let lebar = format!(
                "width: {:.1}%",
                (poin as f64 / poin_maks_kandidat.max(1) as f64) * 100.0
            );
            let sel_dm = h
                .penilaian
                .iter()
                .map(|p| {
                    let pos = p.ranking.iter().position(|x| x == kode).map(|x| x + 1).unwrap_or(0);
                    let p_dm = m.saturating_sub(pos);
                    view! {
                        <td class="angka-tengah" title=format!("Peringkat #{pos} ({p_dm} poin)")>
                            <span class="chip-peringkat" class:chip-satu=pos == 1>{format!("#{pos}")}</span>
                        </td>
                    }
                })
                .collect_view();
            view! {
                <tr class:juara=i == 0>
                    <td class="kode">{i + 1}</td>
                    <th scope="row">
                        <div class="sel-bar">
                            <span>{h.nama_alternatif(kode)}</span>
                            <div class="bar-rel kecil-bar" aria-hidden="true">
                                <span class="bar-isi" style=lebar></span>
                            </div>
                        </div>
                    </th>
                    {sel_dm}
                    <td class="total angka-kanan">{format!("{poin} poin")}</td>
                </tr>
            }
        })
        .collect_view();

    // Kartu ringkas per Decision Maker (mudah dipindai, tanpa 30 angka 4-desimal bertumpuk)
    let kartu_individu = h
        .penilaian
        .iter()
        .map(|p| {
            let juara_dm = p
                .ranking
                .first()
                .map(|kode| h.nama_alternatif(kode))
                .unwrap_or_default();
            let fokus_utama = p
                .urutan_kriteria
                .first()
                .map(|kode| {
                    let nm = h
                        .kriteria
                        .iter()
                        .find(|x| &x.kode == kode)
                        .map(|x| x.nama.clone())
                        .unwrap_or_else(|| kode.clone());
                    let w = p.bobot.get(kode).copied().unwrap_or(0.0);
                    format!("{nm} ({})", persen_bulat(w))
                })
                .unwrap_or_default();

            let daftar_skor = p
                .ranking
                .iter()
                .enumerate()
                .map(|(idx, kode)| {
                    let s = p.skor.get(kode).copied().unwrap_or(0.0);
                    let lebar = format!("width: {:.1}%", (s * 100.0).clamp(0.0, 100.0));
                    view! {
                        <li class="bar-item padat" class:juara=idx == 0>
                            <div class="bar-atas">
                                <span class="bar-judul">
                                    <span class="no">{idx + 1}</span>
                                    <span class="alt">{h.nama_alternatif(kode)}</span>
                                </span>
                                <span class="poin" title=format!("Skor SAW: {}", desimal(s, 4))>
                                    {indeks_100(s)}
                                </span>
                            </div>
                            <div class="bar-rel tipis" aria-hidden="true">
                                <span class="bar-isi" style=lebar></span>
                            </div>
                        </li>
                    }
                })
                .collect_view();

            view! {
                <article class="panel individu">
                    <div class="individu-kepala">
                        <h3>{p.jabatan.clone()}</h3>
                        <p class="redup kecil tanpa-margin">{p.nama.clone()}</p>
                    </div>
                    <dl class="ringkasan-dm">
                        <div>
                            <dt>"Pilihan #1"</dt>
                            <dd>{juara_dm}</dd>
                        </div>
                        <div>
                            <dt>"Prioritas utama"</dt>
                            <dd>{fokus_utama}</dd>
                        </div>
                    </dl>
                    <h4>"Indeks preferensi SAW (0 sampai 100)"</h4>
                    <ol class="bar-daftar">{daftar_skor}</ol>
                </article>
            }
        })
        .collect_view();

    // Tabel rincian teknis lengkap (4 desimal ROC + SAW + matriks nilai mentah) di dalam <details>
    let tabel_teknis_dm = h
        .penilaian
        .iter()
        .map(|p| {
            let baris_roc = p
                .urutan_kriteria
                .iter()
                .enumerate()
                .map(|(i, kode)| {
                    let nama = h
                        .kriteria
                        .iter()
                        .find(|x| &x.kode == kode)
                        .map(|x| x.nama.clone())
                        .unwrap_or_else(|| kode.clone());
                    let w = p.bobot.get(kode).copied().unwrap_or(0.0);
                    view! {
                        <tr>
                            <td class="kode">{i + 1}</td>
                            <td>{format!("{kode} · {nama}")}</td>
                            <td class="angka-kanan">{desimal(w, 4)}</td>
                        </tr>
                    }
                })
                .collect_view();

            let kepala_k = h
                .kriteria
                .iter()
                .map(|kr| view! { <th scope="col" class="angka-tengah">{kr.kode.clone()}</th> })
                .collect_view();

            let baris_saw = p
                .ranking
                .iter()
                .enumerate()
                .map(|(i, kode)| {
                    let sel_mentah = h
                        .kriteria
                        .iter()
                        .map(|kr| {
                            let v = p
                                .nilai
                                .get(kode)
                                .and_then(|r| r.get(&kr.kode))
                                .copied()
                                .unwrap_or(0);
                            view! { <td class="angka-tengah">{v}</td> }
                        })
                        .collect_view();
                    let s = p.skor.get(kode).copied().unwrap_or(0.0);
                    view! {
                        <tr>
                            <td class="kode">{i + 1}</td>
                            <th scope="row">{h.nama_alternatif(kode)}</th>
                            {sel_mentah}
                            <td class="angka-kanan total">{desimal(s, 4)}</td>
                        </tr>
                    }
                })
                .collect_view();

            view! {
                <div class="blok-teknis">
                    <h3>{format!("{} ({})", p.jabatan, p.nama)}</h3>
                    <p class="kecil redup">{format!("Dikirim pada {}", waktu_tampil(&p.dikirim_pada))}</p>
                    <div class="grid-dua">
                        <div>
                            <h4>"Bobot ROC (4 desimal)"</h4>
                            <div class="tabel-wrap">
                                <table>
                                    <thead>
                                        <tr><th scope="col">"#"</th><th scope="col">"Kriteria"</th><th scope="col" class="angka-kanan">"Bobot"</th></tr>
                                    </thead>
                                    <tbody>{baris_roc}</tbody>
                                </table>
                            </div>
                        </div>
                        <div>
                            <h4>"Matriks skor mentah (1 sampai 5) & hasil SAW"</h4>
                            <div class="tabel-wrap">
                                <table>
                                    <thead>
                                        <tr>
                                            <th scope="col">"#"</th>
                                            <th scope="col">"Kandidat"</th>
                                            {kepala_k}
                                            <th scope="col" class="angka-kanan">"SAW"</th>
                                        </tr>
                                    </thead>
                                    <tbody>{baris_saw}</tbody>
                                </table>
                            </div>
                        </div>
                    </div>
                </div>
            }
        })
        .collect_view();

    view! {
        <section class="panel fokus" aria-labelledby="judul-pemenang">
            <p class="label-sesi">"Keputusan konsensus kelompok"</p>
            <h1 id="judul-pemenang">{h.nama_alternatif(&k.pemenang)}</h1>
            <p class="status">
                {format!(
                    "Unggul dengan {poin_juara} dari {poin_maks_kandidat} poin maksimal Borda. Ditetapkan oleh {} ({}) pada {}.",
                    k.diproses_oleh,
                    k.jabatan_pemroses,
                    waktu_tampil(&k.diproses_pada)
                )}
            </p>
            <p class="catatan-konsol">{k.catatan.clone()}</p>
        </section>

        <section class="panel" aria-labelledby="judul-rekap">
            <div class="kepala-panel">
                <h2 id="judul-rekap">"Perolehan poin voting Borda"</h2>
                <span class="hitung">
                    {format!("#1 = {} poin · #{} = 0 poin", m.saturating_sub(1), m)}
                </span>
            </div>
            <div class="tabel-wrap">
                <table>
                    <thead>
                        <tr>
                            <th scope="col">"#"</th>
                            <th scope="col">"Kandidat"</th>
                            {kepala_dm}
                            <th scope="col" class="angka-kanan">"Total"</th>
                        </tr>
                    </thead>
                    <tbody>{baris_borda}</tbody>
                </table>
            </div>
        </section>

        <section aria-labelledby="judul-sudut-pandang">
            <div class="judul-bagian">
                <h2 id="judul-sudut-pandang">"Perbandingan sudut pandang penilai"</h2>
                <p class="kecil redup tanpa-margin">"Ringkasan pilihan utama dan indeks SAW (skala 0 sampai 100) dari tiap jabatan."</p>
            </div>
            <div class="grid-individu">{kartu_individu}</div>
        </section>

        <details class="panel rincian-teknis">
            <summary>"Buka rincian perhitungan matematis lengkap (Bobot ROC, matriks nilai 1 sampai 5, dan skor SAW 4 desimal)"</summary>
            <div class="rincian-teknis-isi">
                {tabel_teknis_dm}
            </div>
        </details>
    }
    .into_any()
}
