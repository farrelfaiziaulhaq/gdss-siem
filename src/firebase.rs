//! Klien Firebase sisi server: Firebase Auth (Identity Toolkit REST) dan Cloud Firestore (REST).
//!
//! Mode emulator aktif jika `FIREBASE_AUTH_EMULATOR_HOST` dan `FIRESTORE_EMULATOR_HOST` diisi.
//! Pada mode produksi, akses Firestore memakai service account lewat `gcp_auth`
//! (`GOOGLE_APPLICATION_CREDENTIALS`), sehingga Security Rules dapat menolak semua akses langsung
//! dari browser.

use serde_json::{json, Map, Value};
use std::sync::{Arc, OnceLock};

static FIREBASE: OnceLock<Firebase> = OnceLock::new();

pub struct Firebase {
    http: reqwest::Client,
    api_key: String,
    auth_base: String,
    token_base: String,
    docs_base: String,
    token_provider: Option<Arc<dyn gcp_auth::TokenProvider>>,
}

pub struct Sesi {
    pub id_token: String,
    pub refresh_token: String,
    pub uid: String,
}

fn env(k: &str, default: &str) -> String {
    std::env::var(k).unwrap_or_else(|_| default.to_string())
}

pub async fn init() {
    let project = env("FIREBASE_PROJECT_ID", "demo-gdss-siem");
    let api_key = env("FIREBASE_API_KEY", "demo-key");
    let auth_emu = std::env::var("FIREBASE_AUTH_EMULATOR_HOST").ok();
    let store_emu = std::env::var("FIRESTORE_EMULATOR_HOST").ok();

    let (auth_base, token_base) = match &auth_emu {
        Some(h) => (
            format!("http://{h}/identitytoolkit.googleapis.com/v1"),
            format!("http://{h}/securetoken.googleapis.com/v1"),
        ),
        None => (
            "https://identitytoolkit.googleapis.com/v1".to_string(),
            "https://securetoken.googleapis.com/v1".to_string(),
        ),
    };
    let store_root = match &store_emu {
        Some(h) => format!("http://{h}/v1"),
        None => "https://firestore.googleapis.com/v1".to_string(),
    };
    let token_provider = if store_emu.is_some() {
        None
    } else {
        Some(
            gcp_auth::provider()
                .await
                .expect("Service account tidak ditemukan. Isi GOOGLE_APPLICATION_CREDENTIALS."),
        )
    };
    let mode = if store_emu.is_some() {
        "emulator"
    } else {
        "produksi"
    };
    leptos::logging::log!("Firebase mode {mode}, proyek {project}");

    let fb = Firebase {
        http: reqwest::Client::new(),
        api_key,
        auth_base,
        token_base,
        docs_base: format!("{store_root}/projects/{project}/databases/(default)/documents"),
        token_provider,
    };
    let _ = FIREBASE.set(fb);
}

pub fn get() -> &'static Firebase {
    FIREBASE.get().expect("firebase::init() belum dipanggil")
}

impl Firebase {
    // ---------------------------------------------------------------- Auth

    pub async fn sign_in(&self, email: &str, password: &str) -> Result<Sesi, String> {
        let url = format!(
            "{}/accounts:signInWithPassword?key={}",
            self.auth_base, self.api_key
        );
        let body = json!({ "email": email, "password": password, "returnSecureToken": true });
        let v = self.post_json(&url, &body).await?;
        Ok(Sesi {
            id_token: str_of(&v, "idToken"),
            refresh_token: str_of(&v, "refreshToken"),
            uid: str_of(&v, "localId"),
        })
    }

    /// Memverifikasi ID token lewat Identity Toolkit dan mengembalikan UID.
    pub async fn lookup(&self, id_token: &str) -> Result<String, String> {
        let url = format!("{}/accounts:lookup?key={}", self.auth_base, self.api_key);
        let v = self
            .post_json(&url, &json!({ "idToken": id_token }))
            .await?;
        v["users"][0]["localId"]
            .as_str()
            .map(String::from)
            .ok_or_else(|| "USER_NOT_FOUND".to_string())
    }

    /// Menukar refresh token dengan ID token baru (ID token Firebase berlaku 1 jam).
    pub async fn refresh(&self, refresh_token: &str) -> Result<Sesi, String> {
        let url = format!("{}/token?key={}", self.token_base, self.api_key);
        let body = json!({ "grant_type": "refresh_token", "refresh_token": refresh_token });
        let v = self.post_json(&url, &body).await?;
        Ok(Sesi {
            id_token: str_of(&v, "id_token"),
            refresh_token: str_of(&v, "refresh_token"),
            uid: str_of(&v, "user_id"),
        })
    }

    async fn post_json(&self, url: &str, body: &Value) -> Result<Value, String> {
        let res = self
            .http
            .post(url)
            .json(body)
            .send()
            .await
            .map_err(|e| format!("NETWORK: {e}"))?;
        let ok = res.status().is_success();
        let v: Value = res.json().await.map_err(|e| format!("PARSE: {e}"))?;
        if ok {
            Ok(v)
        } else {
            Err(v["error"]["message"]
                .as_str()
                .unwrap_or("UNKNOWN")
                .to_string())
        }
    }

    // ----------------------------------------------------------- Firestore

    async fn bearer(&self) -> Result<String, String> {
        match &self.token_provider {
            None => Ok("Bearer owner".to_string()),
            Some(p) => p
                .token(&["https://www.googleapis.com/auth/datastore"])
                .await
                .map(|t| format!("Bearer {}", t.as_str()))
                .map_err(|e| format!("TOKEN: {e}")),
        }
    }

    /// Mengambil satu dokumen sebagai JSON biasa. `None` jika tidak ada.
    pub async fn get_doc(&self, path: &str) -> Result<Option<Value>, String> {
        let res = self
            .http
            .get(format!("{}/{path}", self.docs_base))
            .header("Authorization", self.bearer().await?)
            .send()
            .await
            .map_err(|e| format!("NETWORK: {e}"))?;
        if res.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(None);
        }
        if !res.status().is_success() {
            return Err(format!("FIRESTORE {}", res.status()));
        }
        let v: Value = res.json().await.map_err(|e| format!("PARSE: {e}"))?;
        Ok(Some(dari_fields(&v["fields"])))
    }

    /// Membuat atau menimpa dokumen.
    pub async fn set_doc(&self, path: &str, data: &Value) -> Result<(), String> {
        let res = self
            .http
            .patch(format!("{}/{path}", self.docs_base))
            .header("Authorization", self.bearer().await?)
            .json(&json!({ "fields": ke_fields(data) }))
            .send()
            .await
            .map_err(|e| format!("NETWORK: {e}"))?;
        if res.status().is_success() {
            Ok(())
        } else {
            Err(format!("FIRESTORE {}", res.status()))
        }
    }

    /// Mengambil semua dokumen dalam satu koleksi: (id, data).
    pub async fn list(&self, collection: &str) -> Result<Vec<(String, Value)>, String> {
        let res = self
            .http
            .get(format!("{}/{collection}?pageSize=300", self.docs_base))
            .header("Authorization", self.bearer().await?)
            .send()
            .await
            .map_err(|e| format!("NETWORK: {e}"))?;
        if !res.status().is_success() {
            return Err(format!("FIRESTORE {}", res.status()));
        }
        let v: Value = res.json().await.map_err(|e| format!("PARSE: {e}"))?;
        let docs = v["documents"].as_array().cloned().unwrap_or_default();
        Ok(docs
            .iter()
            .map(|d| {
                let name = d["name"].as_str().unwrap_or_default();
                let id = name.rsplit('/').next().unwrap_or_default().to_string();
                (id, dari_fields(&d["fields"]))
            })
            .collect())
    }
}

fn str_of(v: &Value, k: &str) -> String {
    v[k].as_str().unwrap_or_default().to_string()
}

// ------------------------------------------- konversi JSON <-> Firestore Value

fn ke_value(v: &Value) -> Value {
    match v {
        Value::Null => json!({ "nullValue": null }),
        Value::Bool(b) => json!({ "booleanValue": b }),
        Value::Number(n) if n.is_i64() || n.is_u64() => json!({ "integerValue": n.to_string() }),
        Value::Number(n) => json!({ "doubleValue": n.as_f64() }),
        Value::String(s) => json!({ "stringValue": s }),
        Value::Array(a) => {
            json!({ "arrayValue": { "values": a.iter().map(ke_value).collect::<Vec<_>>() } })
        }
        Value::Object(_) => json!({ "mapValue": { "fields": ke_fields(v) } }),
    }
}

fn ke_fields(v: &Value) -> Value {
    let mut m = Map::new();
    if let Value::Object(o) = v {
        for (k, val) in o {
            m.insert(k.clone(), ke_value(val));
        }
    }
    Value::Object(m)
}

fn dari_value(v: &Value) -> Value {
    let Some(o) = v.as_object() else {
        return Value::Null;
    };
    let Some((tipe, isi)) = o.iter().next() else {
        return Value::Null;
    };
    match tipe.as_str() {
        "integerValue" => isi
            .as_str()
            .and_then(|s| s.parse::<i64>().ok())
            .map(Value::from)
            .unwrap_or(Value::Null),
        "doubleValue" => isi.clone(),
        "booleanValue" | "stringValue" | "timestampValue" | "referenceValue" => isi.clone(),
        "arrayValue" => Value::Array(
            isi["values"]
                .as_array()
                .map(|a| a.iter().map(dari_value).collect())
                .unwrap_or_default(),
        ),
        "mapValue" => dari_fields(&isi["fields"]),
        _ => Value::Null,
    }
}

fn dari_fields(f: &Value) -> Value {
    let mut m = Map::new();
    if let Some(o) = f.as_object() {
        for (k, v) in o {
            m.insert(k.clone(), dari_value(v));
        }
    }
    Value::Object(m)
}
