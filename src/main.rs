mod data;

use anyhow::Context;
use clap::Clap;
use data::NoteData;
use directories::BaseDirs;
use std::ffi::OsString;
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
    #[clap(short, long)]
    name: String,
    #[clap(short, long)]
    filename: Option<String>,
    #[clap(short, long)]
    directory: Option<PathBuf>,
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
    if note_data.has_note(&options.name) {
        return Err(anyhow::anyhow!(
            "Note named \"{}\" already exists",
            options.name
        ));
    }
    let editor = std::env::var("EDITOR").context("Editor not set")?;
    let filename = options.name.clone() + ".md";

    let note_path = notes_dir.join(Path::new(&filename));

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
    /*
    note_data.add_note(
        options.name,
        absolute_path(&note_path).with_context(|| {
            format!(
                "Failed to convert {} into an absolute path",
                note_path.to_string_lossy()
            )
        })?,
    );*/
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
            if let Some(note_name) = name_path.file_stem() {
                if Some("md") == name_path.extension().and_then(|i| i.to_str()) {
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
