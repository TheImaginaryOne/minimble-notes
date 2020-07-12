use std::io::Result as IoResult;
use std::path::Path;
use std::process::Command;

pub trait EditorTrait {
    fn open(&self, path: &Path) -> IoResult<bool>;
}
pub struct Editor {
    pub command: String,
}
impl EditorTrait for Editor {
    fn open(&self, path: &Path) -> IoResult<bool> {
        Command::new(self.command.clone())
            .arg(path.clone())
            .status()
            .map(|s| s.success())
    }
}
