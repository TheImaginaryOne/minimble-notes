pub mod data;
pub mod util;

use anyhow::Context;
use clap::Clap;
use std::borrow::Cow;
use std::path::{Path, PathBuf};

use data::NoteData;
use util::EditorTrait;

#[derive(Clap)]
pub struct TagDirOptions {
    #[clap(index(1))]
    pub name: String,
    #[clap(short, long, alias = "add-dir")]
    pub add_dir_tag: Option<PathBuf>,
    #[clap(long, alias = "rm-dir")]
    pub remove_dir_tag: Option<PathBuf>,
}
#[derive(Clap)]
pub struct EditNoteOptions {
    #[clap(index(1))]
    pub name: String,
}
#[derive(Clap)]
pub struct RemoveNoteOptions {
    #[clap(index(1))]
    pub name: String,
}
#[derive(Clap)]
pub struct ShowNoteOptions {
    #[clap(index(1))]
    pub name: Option<String>,
}
#[derive(Clap)]
pub struct RenameNoteOptions {
    #[clap(index(1))]
    pub name: String,
    #[clap(index(2))]
    pub new_name: String,
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

pub fn remove(
    options: RemoveNoteOptions,
    notes_dir: &Path,
    note_data: &mut NoteData,
) -> anyhow::Result<()> {
    let note_name = get_note_file_name(&options.name, note_data)?;
    let note_path = get_full_path(notes_dir, &note_name, "md");

    std::fs::remove_file(note_path.clone())
        .with_context(|| format!("Failed to delete {}", note_path.to_string_lossy()))?;

    note_data.remove_note(&note_name);

    println!("Removed note at {}", note_path.to_string_lossy());
    Ok(())
}

pub fn rename(
    options: RenameNoteOptions,
    notes_dir: &Path,
    note_data: &mut NoteData,
) -> anyhow::Result<()> {
    let note_name = get_note_file_name(&options.name, note_data)?;
    let note_path = get_full_path(notes_dir, &note_name, "md");
    let new_note_path = get_full_path(notes_dir, &options.new_name, "md");

    if new_note_path.exists() {
        anyhow::bail!(format!(
            "Note exists at {}",
            new_note_path.to_string_lossy()
        ));
    }

    std::fs::rename(note_path.clone(), new_note_path)
        .with_context(|| format!("Failed to rename {}", note_path.to_string_lossy()))?;

    // Update note data as required
    note_data.rename_note(&note_name, &options.new_name);

    println!("Renamed note at {}", note_path.to_string_lossy());
    Ok(())
}

pub fn show(
    options: ShowNoteOptions,
    notes_dir: &Path,
    note_data: &mut NoteData,
) -> anyhow::Result<()> {
    if let Some(name) = options.name {
        let note_name = get_note_file_name(&name, note_data)?;
        let note_path = get_full_path(notes_dir, &note_name, "md");

        let mut file = std::fs::File::open(note_path.clone())
            .with_context(|| format!("Failed to open file at {}", note_path.to_string_lossy()))?;
        let mut stdout = std::io::stdout();

        std::io::copy(&mut file, &mut stdout)?;
    } else {
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
                        let dir_list = note_data
                            .directory_tags
                            .iter()
                            .filter_map(|(key, val)| {
                                if &val[..] == note_name {
                                    Some(key.to_string_lossy())
                                } else {
                                    None
                                }
                            })
                            .collect::<Vec<_>>();

                        if dir_list.len() > 0 {
                            println!("{} ({})", note_name.to_string_lossy(), dir_list.join(", "));
                        } else {
                            println!("{}", note_name.to_string_lossy());
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

pub fn get_note_file_name<'a>(name: &'a str, note_data: &NoteData) -> anyhow::Result<Cow<'a, str>> {
    // @ means the note tagged to the current or parent directory
    if name == "@" {
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
    } else {
        Ok(Cow::Borrowed(name))
    }
}

fn get_full_path(directory: &Path, name: &str, extension: &str) -> PathBuf {
    directory.join(Path::new(name).with_extension(extension))
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
