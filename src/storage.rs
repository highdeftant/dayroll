use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use crate::model::Todo;

pub trait TodoStore {
    fn load(&self) -> Result<Vec<Todo>, String>;
    fn save(&self, todos: &[Todo]) -> Result<(), String>;
}

enum Backend {
    File(PathBuf),
    Memory(Arc<Mutex<Vec<Todo>>>),
}

pub struct Store {
    backend: Backend,
}

impl Store {
    pub fn new_in_memory() -> Self {
        Self {
            backend: Backend::Memory(Arc::new(Mutex::new(Vec::new()))),
        }
    }

    pub fn new_file(path: impl Into<PathBuf>) -> Self {
        Self {
            backend: Backend::File(path.into()),
        }
    }

    pub fn default_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        Path::new(&home).join(".dayroll").join("todos.json")
    }
}

impl TodoStore for Store {
    fn load(&self) -> Result<Vec<Todo>, String> {
        match &self.backend {
            Backend::Memory(shared) => shared
                .lock()
                .map(|todos| todos.clone())
                .map_err(|_| "memory store lock poisoned".to_string()),
            Backend::File(path) => {
                if !path.exists() {
                    return Ok(Vec::new());
                }

                let raw = fs::read_to_string(path)
                    .map_err(|error| format!("failed reading store: {error}"))?;
                serde_json::from_str::<Vec<Todo>>(&raw)
                    .map_err(|error| format!("failed parsing store: {error}"))
            }
        }
    }

    fn save(&self, todos: &[Todo]) -> Result<(), String> {
        match &self.backend {
            Backend::Memory(shared) => shared
                .lock()
                .map(|mut current| {
                    *current = todos.to_vec();
                })
                .map_err(|_| "memory store lock poisoned".to_string()),
            Backend::File(path) => {
                if let Some(parent) = path.parent() {
                    fs::create_dir_all(parent)
                        .map_err(|error| format!("failed creating store directory: {error}"))?;
                }

                let encoded = serde_json::to_string_pretty(todos)
                    .map_err(|error| format!("failed encoding store: {error}"))?;

                let tmp_path = path.with_extension("json.tmp");
                fs::write(&tmp_path, encoded)
                    .map_err(|error| format!("failed writing temp store: {error}"))?;
                fs::rename(&tmp_path, path)
                    .map_err(|error| format!("failed replacing store: {error}"))
            }
        }
    }
}
