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
    New(NewNoteOptions),
}
#[derive(Clap)]
struct EditNoteOptions {
    #[clap(short, long)]
    name: String,
}
#[derive(Clap)]
struct NewNoteOptions {
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
        SubCommand::Edit(o) => edit(o, Path::new(&notes_dir), &note_data)?,
        SubCommand::New(o) => {
            new(o, Path::new(&notes_dir), &mut note_data)?;
            data::save_note_data(&note_data, &data_path).with_context(|| {
                format!("Failed to write data to {}", data_path.to_string_lossy())
            })?;
        }
    }
    Ok(())
}
fn new(options: NewNoteOptions, notes_dir: &Path, note_data: &mut NoteData) -> anyhow::Result<()> {
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
fn edit(options: EditNoteOptions, notes_dir: &Path, note_data: &NoteData) -> anyhow::Result<()> {
    let editor = std::env::var("EDITOR").context("Editor not set")?;

    let path = notes_dir.join(Path::new(&(options.name + ".md")));

    let _ = Command::new(editor)
        .arg(path)
        .status()
        .context("Failed to save note")?;
    println!("Saved note");
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
