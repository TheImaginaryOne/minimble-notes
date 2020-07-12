mod data;

use anyhow::Context;
use clap::Clap;
use data::NoteData;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Clap)]
#[clap(about("A simple terminal notes app"))]
struct Options {
    #[clap(subcommand)]
    subcommand: SubCommand,
}
#[derive(Clap)]
enum SubCommand {
    Edit(EditNoteOptions),
    Show,
}
#[derive(Clap)]
struct EditNoteOptions {
    #[clap(index(1))]
    name: Option<String>,
    #[clap(short, long, alias="dir")]
    dir_tag: Option<PathBuf>,
    #[clap(long, alias="rm-dir")]
    remove_dir_tag: Option<PathBuf>,
}

fn main() {
    if let Err(e) = run(Options::parse()) {
        match e.source() {
            Some(s) => eprintln!("Error: {} (source: {})", e, s),
            None => eprintln!("Error: {}", e),
        }
    }
}
fn run(options: Options) -> anyhow::Result<()> {
    let notes_dir = std::env::var("NOTES_DIR").context("Notes directory not set")?;

    let data_path = Path::new(&notes_dir).join(Path::new("minimble_data.json"));
    let mut note_data = data::load_note_data(&data_path).with_context(|| {
        format!(
            "Failed to read note data from {}",
            data_path.to_string_lossy()
        )
    })?;
    match options.subcommand {
        SubCommand::Edit(o) => {
            edit(o, Path::new(&notes_dir), &mut note_data)?;
            data::save_note_data(&note_data, &data_path).with_context(|| {
                format!("Failed to write data to {}", data_path.to_string_lossy())
            })?;
        }
        SubCommand::Show => show_notes(Path::new(&notes_dir))?,
    }
    Ok(())
}
fn edit(
    options: EditNoteOptions,
    notes_dir: &Path,
    note_data: &mut NoteData,
) -> anyhow::Result<()> {
    let editor = std::env::var("EDITOR").context("Editor not set")?;
    // if a note name is specified in the command
    let note_name = if let Some(name) = options.name {
        name
    } else {
        // Otherwise...
        let current = std::env::current_dir().context("Failed to access current directory")?;
        let mut dir = current.into_boxed_path();

        // ...get note tagged to current dir
        if let Some(note_name) = note_data.get_dir_tag(dir.as_ref()) {
            note_name.clone()
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
            note_name_opt
                .cloned()
                .context("No note tagged to current or parent directories")?
        }
    };

    // Then form the file path name
    let note_path = notes_dir.join(Path::new(&note_name).with_extension("md"));

    let _ = Command::new(editor)
        .arg(note_path.clone())
        .status()
        .context("Failed to save note")?;

    if !note_path.exists() {
        return Err(anyhow::anyhow!(format!(
            "Did not save note at {}",
            note_path.to_string_lossy()
        )));
    }

    // Set the requested directory tags...
    if let Some(dir) = options.dir_tag {
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
fn show_notes(notes_dir: &Path) -> anyhow::Result<()> {
    let files = std::fs::read_dir(notes_dir)
        .with_context(|| format!("Failed to open directory {}", notes_dir.to_string_lossy()))?;
    for f in files {
        if let Ok(file) = f {
            let file_name = &file.file_name();
            let name_path = Path::new(file_name);
            if Some("md") == name_path.extension().and_then(|i| i.to_str()) {
                if let Some(note_name) = name_path.file_stem() {
                    println!("{}", note_name.to_string_lossy());
                }
            }
        }
    }

    Ok(())
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
