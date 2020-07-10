use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize)]
pub struct NoteData {
    notes: HashMap<String, PathBuf>,
}
impl NoteData {
    pub fn add_note<T: Into<String>>(&mut self, name: T, path: PathBuf) {
        self.notes.insert(name.into(), path);
    }
    pub fn has_note(&mut self, name: &str) -> bool {
        self.notes.contains_key(name)
    }
}

pub fn load_note_data(path: &Path) -> anyhow::Result<NoteData> {
    match File::open(path) {
        Ok(f) => Ok(serde_json::from_reader(f)?),
        Err(_) => Ok(NoteData {
            notes: HashMap::new(),
        }),
    }
}
pub fn save_note_data(note_data: &NoteData, path: &Path) -> anyhow::Result<()> {
    Ok(serde_json::to_writer(File::create(path)?, note_data)?)
}
