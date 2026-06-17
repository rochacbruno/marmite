use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Credential {
    pub pds_url: String,
    pub identifier: String,
    pub password: String,
}

type CredentialsStore = HashMap<String, Credential>;

fn credentials_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".config").join("marmite")
}

pub fn credentials_path() -> PathBuf {
    credentials_dir().join("credentials.json")
}

fn load_store() -> CredentialsStore {
    let path = credentials_path();
    if !path.exists() {
        return HashMap::new();
    }
    let content = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return HashMap::new(),
    };
    serde_json::from_str(&content).unwrap_or_default()
}

fn save_store(store: &CredentialsStore) -> Result<(), Box<dyn std::error::Error>> {
    let dir = credentials_dir();
    fs::create_dir_all(&dir)?;
    let path = credentials_path();
    let json = serde_json::to_string_pretty(store)?;
    fs::write(&path, json)?;
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600))?;
    Ok(())
}

pub fn save(cred: &Credential) -> Result<(), Box<dyn std::error::Error>> {
    let mut store = load_store();
    store.insert(cred.identifier.clone(), cred.clone());
    save_store(&store)
}

pub fn load(identifier: &str) -> Option<Credential> {
    let mut store = load_store();
    store.remove(identifier)
}

pub fn load_any() -> Option<Credential> {
    let store = load_store();
    if store.len() == 1 {
        store.into_values().next()
    } else {
        None
    }
}
