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
                // Tema yang dipilih pengguna dipasang sebelum CSS dimuat agar tidak berkedip.
                <script>
                    "try{var t=localStorage.getItem('tema');if(t)document.documentElement.setAttribute('data-theme',t)}catch(e){}"
                </script>
                // Belum ada ikon aplikasi; request favicon dimatikan sampai ikon disetujui.
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

// ------------------------------------------------------------------ kerangka

#[component]
fn Kerangka(user: Profil, children: Children) -> impl IntoView {
    let keluar = ServerAction::<Keluar>::new();
    view! {
        <a class="lewati" href="#isi">"Lewati ke konten"</a>
        <header class="kepala">
            <a href="/" class="merek">{MEREK}</a>
            <nav aria-label="Navigasi utama" class="nav">
                <A href="/" exact=true>"Beranda"</A>
                <A href="/penilaian">"Penilaian saya"</A>
                <A href="/hasil">"Hasil"</A>
            </nav>
            <div class="akun">
                <p class="akun-id">
                    <span class="akun-nama">{user.nama}</span>
                    <span class="akun-jabatan">{user.jabatan}</span>
                </p>
                <TombolTema/>
                <ActionForm action=keluar>
                    <button type="submit" class="btn-teks">"Keluar"</button>
                </ActionForm>
            </div>
        </header>
        <main id="isi" class="isi">{children()}</main>
        <footer class="kaki">
            <p>"Proyek akhir Praktikum Sistem Informasi. Data sesi disimpan di Cloud Firestore."</p>
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
        <div class="isi">
            <div class="panel memuat" role="status" aria-live="polite">
                <span class="lampu kedip" aria-hidden="true"></span>
                {teks}
            </div>
        </div>
    }
}

#[component]
fn Galat(pesan: String, ulang: Callback<()>) -> impl IntoView {
    view! {
        <div class="isi">
            <div class="panel galat" role="alert">
                <h1>"Data gagal dimuat"</h1>
                <p>{pesan}</p>
                <button type="button" class="btn" on:click=move |_| ulang.run(())>
                    "Muat ulang data"
                </button>
            </div>
        </div>
    }
}

#[component]
fn TidakDitemukan() -> impl IntoView {
    view! {
        <Title text=format!("Halaman tidak ada · {MEREK}")/>
        <main id="isi" class="isi">
            <div class="panel">
                <h1>"Halaman tidak ditemukan"</h1>
                <p>"Alamat ini tidak ada di aplikasi."</p>
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
                <p class="merek">{MEREK}</p>
                <h1>"Masuk sebagai decision maker"</h1>
                <p class="pengantar">
                    "Setiap jabatan memakai akunnya sendiri. Penilaian Anda tidak terlihat oleh decision maker lain sampai konsensus diproses."
                </p>
                <ActionForm action=masuk>
                    <div class="form">
                        <label for="email">"Email"</label>
                        <input id="email" name="email" type="email" autocomplete="username" required/>
                        <label for="password">"Kata sandi"</label>
                        <input id="password" name="password" type="password" autocomplete="current-password" required/>
                        {galat}
                        <button type="submit" class="btn utama" disabled=move || masuk.pending().get()>
                            {move || if masuk.pending().get() { "Memeriksa akun…" } else { "Masuk" }}
                        </button>
                    </div>
                </ActionForm>
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

    let (status, aksi) = if let Some(k) = &d.konsensus {
        (
            format!(
                "Konsensus selesai. {} terpilih lewat voting Borda yang diproses oleh {} pada {}.",
                nama_alt(&alternatif, &k.pemenang),
                k.jabatan_pemroses,
                waktu_tampil(&k.diproses_pada)
            ),
            view! { <a class="btn utama" href="/hasil">"Buka rincian hasil"</a> }.into_any(),
        )
    } else if !d.saya_selesai {
        (
            format!("Anda belum mengirim penilaian. {masuk} dari {jumlah} decision maker sudah menilai."),
            view! { <a class="btn utama" href="/penilaian">"Isi penilaian saya"</a> }.into_any(),
        )
    } else if d.semua_selesai && d.saya_tertinggi {
        (
            format!(
                "Semua penilaian sudah masuk. Sebagai {}, Anda memproses voting Borda untuk menetapkan keputusan kelompok.",
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
            format!("Semua penilaian sudah masuk. Menunggu {tertinggi} memproses voting Borda."),
            view! { <a class="btn" href="/penilaian">"Ubah penilaian saya"</a> }.into_any(),
        )
    } else {
        (
            format!(
                "Penilaian Anda sudah tersimpan. Menunggu {} decision maker lagi sebelum voting dapat diproses.",
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
            <p class="kosong">"Belum ada decision maker terdaftar. Jalankan skrip seed untuk membuat tiga akun."</p>
        }
        .into_any()
    } else {
        let uid_saya = d.user.uid.clone();
        view! {
            <ul class="daftar-dm">
                {d.dms.iter().map(|s| {
                    let waktu = match (&s.selesai, &s.dikirim_pada) {
                        (true, Some(w)) => format!("Terkirim {}", waktu_tampil(w)),
                        _ => "Belum menilai".to_string(),
                    };
                    let peran = if s.level == level_maks { " · memproses voting" } else { "" };
                    let anda = if s.uid == uid_saya { " (Anda)" } else { "" };
                    view! {
                        <li>
                            <span class="lampu" class:nyala=s.selesai aria-hidden="true"></span>
                            <div class="dm-id">
                                <p class="dm-nama">{s.nama.clone()}{anda}</p>
                                <p class="dm-jabatan">{s.jabatan.clone()}{peran}</p>
                            </div>
                            <p class="dm-waktu" class:redup=!s.selesai>{waktu}</p>
                        </li>
                    }
                }).collect_view()}
            </ul>
        }
        .into_any()
    };

    let panel_hasil = match &d.konsensus {
        Some(k) => {
            let alt = alternatif.clone();
            view! {
                <ol class="peringkat">
                    {k.ranking.iter().enumerate().map(|(i, kode)| {
                        let poin = k.poin.get(kode).copied().unwrap_or(0);
                        view! {
                            <li class:juara=i == 0>
                                <span class="no">{i + 1}</span>
                                <span class="alt">{nama_alt(&alt, kode)}</span>
                                <span class="poin">{poin}" poin"</span>
                            </li>
                        }
                    }).collect_view()}
                </ol>
                <p class="catatan">{k.catatan.clone()}</p>
            }
            .into_any()
        }
        None => view! {
            <p class="kosong">
                "Belum ada keputusan kelompok. Voting Borda diproses oleh "{tertinggi.clone()}
                " setelah semua decision maker mengirim penilaian."
            </p>
        }
        .into_any(),
    };

    let tabel_alt = if d.alternatif.is_empty() {
        view! { <p class="kosong">"Belum ada alternatif di Firestore."</p> }.into_any()
    } else {
        view! {
            <div class="tabel-wrap">
                <table>
                    <thead>
                        <tr><th scope="col">"Kode"</th><th scope="col">"Alternatif"</th><th scope="col">"Kategori"</th><th scope="col">"Lisensi"</th></tr>
                    </thead>
                    <tbody>
                        {d.alternatif.iter().map(|a| view! {
                            <tr>
                                <td class="kode">{a.kode.clone()}</td>
                                <th scope="row">{a.nama.clone()}</th>
                                <td>{a.kategori.clone()}</td>
                                <td>{a.lisensi.clone()}</td>
                            </tr>
                        }).collect_view()}
                    </tbody>
                </table>
            </div>
        }
        .into_any()
    };

    let daftar_kriteria = d
        .kriteria
        .iter()
        .map(|k| {
            let jenis = match k.jenis {
                Jenis::Benefit => "benefit",
                Jenis::Cost => "cost",
            };
            view! {
                <li>
                    <span class="kode">{k.kode.clone()}</span>
                    <span>{k.nama.clone()}</span>
                    <span class="jenis">{jenis}</span>
                </li>
            }
        })
        .collect_view();

    view! {
        <Kerangka user=d.user.clone()>
            <section class="panel fokus" aria-labelledby="judul-sesi">
                <p class="label-sesi">{format!("Sesi {SESI_ID}")}</p>
                <h1 id="judul-sesi">"Pemilihan SIEM untuk tim SOC"</h1>
                <p class="status">{status}</p>
                <div class="aksi">{aksi}</div>
                {galat_proses}
            </section>

            <div class="grid-dua">
                <section class="panel" aria-labelledby="judul-dm">
                    <div class="kepala-panel">
                        <h2 id="judul-dm">"Penilaian decision maker"</h2>
                        <p class="hitung">{masuk}" dari "{jumlah}" masuk"</p>
                    </div>
                    <div class="meter" aria-hidden="true">{lampu_meter}</div>
                    {daftar_dm}
                </section>

                <section class="panel" aria-labelledby="judul-hasil">
                    <div class="kepala-panel">
                        <h2 id="judul-hasil">"Keputusan kelompok"</h2>
                    </div>
                    {panel_hasil}
                </section>
            </div>

            <section class="panel" aria-labelledby="judul-objek">
                <h2 id="judul-objek">"Yang diputuskan"</h2>
                <div class="grid-objek">
                    <div>
                        <h3>"Alternatif"</h3>
                        {tabel_alt}
                    </div>
                    <div>
                        <h3>"Kriteria"</h3>
                        <ul class="daftar-kriteria">{daftar_kriteria}</ul>
                    </div>
                </div>
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
        <Suspense fallback=|| view! { <Memuat teks="Memuat formulir penilaian…"/> }>
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
    let nilai = RwSignal::new(awal.clone());
    let pesan_lokal = RwSignal::new(None::<String>);

    let jumlah_sel = f.kriteria.len() * f.alternatif.len();
    let terisi = move || {
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
        let kosong = jumlah_sel - terisi();
        if kosong > 0 {
            pesan_lokal.set(Some(format!(
                "Masih ada {kosong} nilai yang belum dipilih. Semua sel wajib diisi sebelum dikirim."
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

    let bobot = roc(f.kriteria.len());
    let daftar_prioritas = move || {
        let n = urutan.with(|u| u.len());
        urutan
            .get()
            .into_iter()
            .enumerate()
            .map(|(i, kode)| {
                let k = kriteria
                    .with_value(|ks| ks.iter().find(|k| k.kode == kode).cloned())
                    .expect("kriteria ada");
                let w = desimal(bobot[i], 4);
                let nama_naik = format!("Naikkan prioritas {}", k.nama);
                let nama_turun = format!("Turunkan prioritas {}", k.nama);
                let jenis = match k.jenis {
                    Jenis::Benefit => "benefit, makin tinggi makin baik",
                    Jenis::Cost => "cost, makin tinggi makin mahal",
                };
                view! {
                    <li class="prioritas">
                        <span class="no">{i + 1}</span>
                        <div class="prioritas-isi">
                            <p class="prioritas-nama">{k.nama.clone()}<span class="jenis">{jenis}</span></p>
                            <p class="prioritas-desk">{k.deskripsi.clone()}</p>
                            <details>
                                <summary>"Rubrik skala 1 sampai 5"</summary>
                                <ol class="rubrik">
                                    {k.skala.iter().map(|s| view! { <li>{s.clone()}</li> }).collect_view()}
                                </ol>
                            </details>
                        </div>
                        <p class="bobot"><span class="redup">"bobot "</span>{w}</p>
                        <div class="geser">
                            <button type="button" class="btn-kecil" aria-label=nama_naik
                                disabled=terkunci || i == 0 on:click=move |_| geser(i, true)>"Naik"</button>
                            <button type="button" class="btn-kecil" aria-label=nama_turun
                                disabled=terkunci || i + 1 == n on:click=move |_| geser(i, false)>"Turun"</button>
                        </div>
                    </li>
                }
            })
            .collect_view()
    };

    let isian = alternatif.with_value(|alts| {
        alts.iter()
            .map(|a| {
                let sel = kriteria.with_value(|ks| {
                    ks.iter()
                        .map(|k| {
                            let id = format!("n-{}-{}", a.kode, k.kode);
                            let ak = a.kode.clone();
                            let kk = k.kode.clone();
                            let awal_v = awal.get(&a.kode).and_then(|r| r.get(&k.kode)).copied().unwrap_or(0);
                            let opsi = k
                                .skala
                                .iter()
                                .enumerate()
                                .map(|(i, s)| {
                                    let n = (i + 1) as u8;
                                    view! { <option value=n.to_string() selected=awal_v == n>{format!("{n}. {s}")}</option> }
                                })
                                .collect_view();
                            view! {
                                <div class="sel">
                                    <label for=id.clone()>{k.nama.clone()}</label>
                                    <select id=id disabled=terkunci on:change=move |ev| {
                                        let v: u8 = event_target_value(&ev).parse().unwrap_or(0);
                                        nilai.update(|m| {
                                            m.entry(ak.clone()).or_default().insert(kk.clone(), v);
                                        });
                                    }>
                                        <option value="0" selected=awal_v == 0>"Pilih nilai"</option>
                                        {opsi}
                                    </select>
                                </div>
                            }
                        })
                        .collect_view()
                });
                view! {
                    <fieldset class="alt-nilai">
                        <legend>{a.nama.clone()}<span class="redup">{format!(" · {}", a.kategori)}</span></legend>
                        <p class="alt-ringkas">{a.ringkasan.clone()}</p>
                        <div class="grid-sel">{sel}</div>
                    </fieldset>
                }
            })
            .collect_view()
    });

    let hasil_kirim = move || match kirim.value().get() {
        Some(Ok(p)) => {
            let urut = alternatif.with_value(|alts| {
                p.ranking
                    .iter()
                    .map(|kode| {
                        let s = p.skor.get(kode).copied().unwrap_or(0.0);
                        view! { <li>{nama_alt(alts, kode)}<span class="poin">{desimal(s, 4)}</span></li> }
                    })
                    .collect_view()
            });
            Some(
                view! {
                    <div class="panel sukses" role="status">
                        <h2>"Penilaian tersimpan"</h2>
                        <p>{format!("Terkirim {}. Urutan SAW versi Anda:", waktu_tampil(&p.dikirim_pada))}</p>
                        <ol class="peringkat-ringkas">{urut}</ol>
                        <a class="btn" href="/">"Kembali ke beranda"</a>
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
            <h1 id="judul-form">"Penilaian individual"</h1>
            <p class="status">
                {if terkunci {
                    "Konsensus sudah diproses, jadi penilaian ini dikunci dan hanya dapat dibaca.".to_string()
                } else {
                    format!(
                        "Atur prioritas kriteria menurut sudut pandang {}, lalu beri nilai 1 sampai 5 untuk setiap alternatif. Penilaian dapat diubah sampai voting diproses.",
                        f.user.jabatan
                    )
                }}
            </p>
        </section>

        <form on:submit=on_submit class="form-nilai" novalidate>
            <section class="panel" aria-labelledby="judul-prioritas">
                <h2 id="judul-prioritas">"1. Prioritas kriteria"</h2>
                <p class="bantu">"Urutan teratas mendapat bobot terbesar. Bobot dihitung dengan Rank Order Centroid (ROC)."</p>
                <ol class="daftar-prioritas">{daftar_prioritas}</ol>
            </section>

            <section class="panel" aria-labelledby="judul-isian">
                <div class="kepala-panel">
                    <h2 id="judul-isian">"2. Nilai alternatif"</h2>
                    <p class="hitung" aria-live="polite">{move || format!("{} dari {jumlah_sel} terisi", terisi())}</p>
                </div>
                {isian}
            </section>

            <div class="kirim">
                {move || pesan_lokal.get().map(|m| view! { <p class="pesan galat-teks" role="alert">{m}</p> })}
                {hasil_kirim}
                <button type="submit" class="btn utama" disabled=move || terkunci || kirim.pending().get()>
                    {move || {
                        if kirim.pending().get() {
                            "Menyimpan…"
                        } else if ada_tersimpan || matches!(kirim.value().get(), Some(Ok(_))) {
                            "Simpan perubahan"
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
                <h1 id="judul-hasil-kosong">"Hasil belum tersedia"</h1>
                <p class="status">
                    {format!(
                        "Voting Borda diproses oleh decision maker berjabatan tertinggi setelah {} penilaian masuk. Penilaian individual juga baru dibuka setelah itu, supaya tidak saling memengaruhi.",
                        h.jumlah_dm
                    )}
                </p>
                <div class="aksi"><a class="btn" href="/">"Kembali ke beranda"</a></div>
            </section>
        }
        .into_any();
    };

    let m = h.alternatif.len();
    let kepala_dm = h
        .penilaian
        .iter()
        .map(|p| view! { <th scope="col">{p.jabatan.clone()}</th> })
        .collect_view();
    let baris = k
        .ranking
        .iter()
        .enumerate()
        .map(|(i, kode)| {
            let sel = h
                .penilaian
                .iter()
                .map(|p| {
                    let pos = p.ranking.iter().position(|x| x == kode).map(|x| x + 1).unwrap_or(0);
                    let poin = m.saturating_sub(pos);
                    view! { <td>{format!("#{pos}")}<span class="redup">{format!(" · {poin} poin")}</span></td> }
                })
                .collect_view();
            view! {
                <tr class:juara=i == 0>
                    <td class="kode">{i + 1}</td>
                    <th scope="row">{h.nama_alternatif(kode)}</th>
                    {sel}
                    <td class="total">{k.poin.get(kode).copied().unwrap_or(0)}</td>
                </tr>
            }
        })
        .collect_view();

    let individu = h
        .penilaian
        .iter()
        .map(|p| {
            let bobot = p
                .urutan_kriteria
                .iter()
                .map(|kode| {
                    let nama = h
                        .kriteria
                        .iter()
                        .find(|x| &x.kode == kode)
                        .map(|x| x.nama.clone())
                        .unwrap_or_else(|| kode.clone());
                    let w = p.bobot.get(kode).copied().unwrap_or(0.0);
                    view! { <li>{nama}<span class="poin">{desimal(w, 4)}</span></li> }
                })
                .collect_view();
            let skor = p
                .ranking
                .iter()
                .map(|kode| {
                    let s = p.skor.get(kode).copied().unwrap_or(0.0);
                    view! { <li>{h.nama_alternatif(kode)}<span class="poin">{desimal(s, 4)}</span></li> }
                })
                .collect_view();
            view! {
                <article class="panel individu">
                    <h3>{p.nama.clone()}<span class="redup">{format!(" · {}", p.jabatan)}</span></h3>
                    <p class="redup kecil">{format!("Terkirim {}", waktu_tampil(&p.dikirim_pada))}</p>
                    <h4>"Bobot kriteria (ROC)"</h4>
                    <ol class="peringkat-ringkas">{bobot}</ol>
                    <h4>"Skor SAW"</h4>
                    <ol class="peringkat-ringkas">{skor}</ol>
                </article>
            }
        })
        .collect_view();

    view! {
        <section class="panel fokus" aria-labelledby="judul-pemenang">
            <p class="label-sesi">"Keputusan kelompok"</p>
            <h1 id="judul-pemenang">{h.nama_alternatif(&k.pemenang)}</h1>
            <p class="status">
                {format!(
                    "Ditetapkan dengan {} oleh {} ({}) pada {}. {}",
                    k.metode,
                    k.diproses_oleh,
                    k.jabatan_pemroses,
                    waktu_tampil(&k.diproses_pada),
                    k.catatan
                )}
            </p>
        </section>

        <section class="panel" aria-labelledby="judul-rekap">
            <h2 id="judul-rekap">"Rekap voting Borda"</h2>
            <p class="bantu">
                {format!(
                    "Setiap peringkat individual diubah menjadi poin: peringkat 1 dari {m} alternatif mendapat {} poin, peringkat terakhir 0 poin.",
                    m.saturating_sub(1)
                )}
            </p>
            <div class="tabel-wrap">
                <table>
                    <thead>
                        <tr>
                            <th scope="col">"Akhir"</th>
                            <th scope="col">"Alternatif"</th>
                            {kepala_dm}
                            <th scope="col">"Total poin"</th>
                        </tr>
                    </thead>
                    <tbody>{baris}</tbody>
                </table>
            </div>
        </section>

        <h2 class="judul-lepas">"Penilaian individual"</h2>
        <div class="grid-individu">{individu}</div>
    }
    .into_any()
}
