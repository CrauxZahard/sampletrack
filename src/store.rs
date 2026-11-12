//! Where the workspace lives. The app talks to [`Repository`], never to a file or a socket, so
//! this local store can be swapped for a REST client without touching a single view.

#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;

use crate::model::Data;

/// The persistence boundary: load the workspace, save it. A server-backed implementation would
/// keep the same methods and map them to GET/PUT requests.
pub trait Repository {
    fn load(&self) -> Option<Data>;
    fn save(&self, data: &Data) -> Result<(), String>;
    /// A short path or URL, shown in the app's storage note.
    fn location(&self) -> String;
}

/// One JSON document: a file in the user's application data folder natively, and localStorage
/// in the browser.
#[cfg(not(target_arch = "wasm32"))]
pub struct LocalJson {
    path: PathBuf,
}

#[cfg(not(target_arch = "wasm32"))]
impl LocalJson {
    pub fn open() -> LocalJson {
        LocalJson { path: data_path() }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl Repository for LocalJson {
    fn load(&self) -> Option<Data> {
        let text = std::fs::read_to_string(&self.path).ok()?;
        serde_json::from_str(&text).ok()
    }

    fn save(&self, data: &Data) -> Result<(), String> {
        if let Some(dir) = self.path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| format!("folder data: {e}"))?;
        }
        let text = serde_json::to_string_pretty(data).map_err(|e| format!("serialisasi: {e}"))?;
        std::fs::write(&self.path, text).map_err(|e| format!("menulis data: {e}"))
    }

    fn location(&self) -> String {
        self.path.display().to_string()
    }
}

#[cfg(target_arch = "wasm32")]
pub struct LocalJson;

#[cfg(target_arch = "wasm32")]
impl LocalJson {
    pub fn open() -> LocalJson {
        LocalJson
    }
}

#[cfg(target_arch = "wasm32")]
impl LocalJson {
    fn storage() -> Option<web_sys::Storage> {
        web_sys::window()?.local_storage().ok().flatten()
    }

    const KEY: &'static str = "sample-track-workspace";
}

#[cfg(target_arch = "wasm32")]
impl Repository for LocalJson {
    fn load(&self) -> Option<Data> {
        let text = LocalJson::storage()?
            .get_item(LocalJson::KEY)
            .ok()
            .flatten()?;
        serde_json::from_str(&text).ok()
    }

    fn save(&self, data: &Data) -> Result<(), String> {
        let text = serde_json::to_string(data).map_err(|e| format!("serialisasi: {e}"))?;
        let store = LocalJson::storage().ok_or("localStorage tidak tersedia")?;
        store
            .set_item(LocalJson::KEY, &text)
            .map_err(|_| "localStorage penuh".to_owned())
    }

    fn location(&self) -> String {
        "localStorage · sample-track-workspace".into()
    }
}

/// An in-memory repository for tests: nothing is read and saves are discarded.
#[cfg(test)]
pub struct Memory;

#[cfg(test)]
impl Repository for Memory {
    fn load(&self) -> Option<Data> {
        None
    }

    fn save(&self, _data: &Data) -> Result<(), String> {
        Ok(())
    }

    fn location(&self) -> String {
        "memori".into()
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn data_path() -> PathBuf {
    if let Ok(dir) = std::env::var("APPDATA") {
        return PathBuf::from(dir)
            .join("SampleTrack")
            .join("workspace.json");
    }
    if let Ok(dir) = std::env::var("XDG_DATA_HOME") {
        return PathBuf::from(dir)
            .join("sample-track")
            .join("workspace.json");
    }
    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home).join(".local/share/sample-track/workspace.json");
    }
    PathBuf::from("sample-track-workspace.json")
}
