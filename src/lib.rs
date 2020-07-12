pub mod data;
pub mod util;

use anyhow::Context;
use clap::Clap;
use std::path::{Path, PathBuf};
use std::borrow::Cow;

use data::NoteData;
use util::EditorTrait;

#[derive(Clap)]
pub struct TagDirOptions {
    #[clap(index(1))]
    pub name: Option<String>,
    #[clap(short, long, alias = "add-dir")]
    pub add_dir_tag: Option<PathBuf>,
    #[clap(long, alias = "rm-dir")]
    pub remove_dir_tag: Option<PathBuf>,
}
#[derive(Clap)]
pub struct EditNoteOptions {
    #[clap(index(1))]
    pub name: Option<String>,
}

pub fn tag_dir(
    options: TagDirOptions,
    notes_dir: &Path,
    note_data: &mut NoteData,
) -> anyhow::Result<()> {
    // if a note name is specified in the command
    let note_name = get_note_file_name(&options.name, note_data)?;
    // Then form the file path name
    let note_path = get_full_path(notes_dir, &note_name, "md");
    // Set the requested directory tags...
    if let Some(dir) = options.add_dir_tag {
        note_data.set_dir_tag(
            Some(&note_name),
            absolute_path(&dir).with_context(|| {
                format!(
                    "Failed to convert {} into an absolute path",
                    note_path.to_string_lossy()
                )
            })?,
        );
    }
    // ...and unset the requested directory tags
    if let Some(dir) = options.remove_dir_tag {
        note_data.set_dir_tag(
            None,
            absolute_path(&dir).with_context(|| {
                format!(
                    "Failed to convert {} into an absolute path",
                    note_path.to_string_lossy()
                )
            })?,
        );
    }
    println!("Saved note");
    Ok(())
}

pub fn edit(
    options: EditNoteOptions,
    notes_dir: &Path,
    note_data: &mut NoteData,
    editor: impl EditorTrait,
) -> anyhow::Result<()> {
    let note_name = get_note_file_name(&options.name, note_data)?;
    let note_path = get_full_path(notes_dir, &note_name, "md");

    // Open the text editor
    if !editor
        .open(&note_path)
        .context("Failed to run editor subprocess successfully")?
    {
        anyhow::bail!("Editor exited with error");
    }

    if !note_path.exists() {
        return Err(anyhow::anyhow!(format!(
            "Did not save note at {}",
            note_path.to_string_lossy()
        )));
    }
    println!("Saved note to {}", note_path.to_string_lossy());
    Ok(())
}

pub fn show_notes(notes_dir: &Path) -> anyhow::Result<()> {
    let files = std::fs::read_dir(notes_dir)
        .with_context(|| format!("Failed to open directory {}", notes_dir.to_string_lossy()))?;
    // Walk the directory
    for f in files {
        if let Ok(file) = f {
            let file_name = &file.file_name();
            let name_path = Path::new(file_name);
            // Check if it is a .md file
            if Some("md") == name_path.extension().and_then(|i| i.to_str()) {
                // Does it have a file stem?
                if let Some(note_name) = name_path.file_stem() {
                    println!("{}", note_name.to_string_lossy());
                }
            }
        }
    }

    Ok(())
}

pub fn get_note_file_name<'a>(name: &'a Option<String>, note_data: &NoteData) -> anyhow::Result<Cow<'a, str>> {
    if let Some(name) = name {
        Ok(Cow::Borrowed(name))
    } else {
        // Otherwise...
        let current = std::env::current_dir().context("Failed to access current directory")?;
        let mut dir = current.into_boxed_path();

        // ...get note tagged to current dir
        if let Some(note_name) = note_data.get_dir_tag(dir.as_ref()) {
            Ok(note_name.clone().into())
        } else {
            let mut note_name_opt = None;
            // Or traverse parents until successful (or not)
            while let Some(d) = dir.parent() {
                dir = Box::from(d);
                if let Some(note_name) = note_data.get_dir_tag(dir.as_ref()) {
                    note_name_opt = Some(note_name);
                    break;
                }
            }
            Ok(note_name_opt
                .cloned()
                .context("No note tagged to current or parent directories")?
                .into())
        }
    }
}

fn get_full_path(directory: &Path, name: &Cow<str>, extension: &str) -> PathBuf {
    directory.join(Path::new(name.as_ref()).with_extension(extension))
}

use path_clean::PathClean;

pub fn absolute_path(path: &Path) -> std::io::Result<PathBuf> {
    let absolute_path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?.join(path)
    }
    .clean();

    Ok(absolute_path)
}
