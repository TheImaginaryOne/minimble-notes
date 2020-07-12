use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::path::{Path, PathBuf};

/// Notes metadata
#[derive(Serialize, Deserialize, Default, PartialEq, Debug)]
pub struct NoteData {
    pub directory_tags: HashMap<PathBuf, String>,
}
impl NoteData {
    pub fn set_dir_tag(&mut self, name: Option<&str>, path: PathBuf) {
        if let Some(name) = name {
            self.directory_tags.insert(path, name.into());
        } else {
            self.directory_tags.remove_entry(&path);
        }
    }
    pub fn get_dir_tag(&self, path: &Path) -> Option<&String> {
        self.directory_tags.get(path)
    }
}

pub fn load_note_data(path: &Path) -> anyhow::Result<NoteData> {
    match File::open(path) {
        Ok(f) => Ok(serde_json::from_reader(f)?),
        Err(_) => Ok(Default::default()),
    }
}
pub fn save_note_data(note_data: &NoteData, path: &Path) -> anyhow::Result<()> {
    Ok(serde_json::to_writer(File::create(path)?, note_data)?)
}
