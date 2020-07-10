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
    #[clap(short, long, parse(from_os_str))]
    filename: Option<OsString>,
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
    let base_dirs = BaseDirs::new().context("No base directory found")?;
    let data_dir = base_dirs.data_dir();

    let data_path = data_dir.join(Path::new("minimble_data.json"));
    let mut note_data = data::load_note_data(&data_path).with_context(|| {
        format!(
            "Failed to read note data from {}",
            data_path.to_string_lossy()
        )
    })?;
    match options.subcommand {
        SubCommand::Edit(o) => edit(o, &note_data)?,
        SubCommand::New(o) => {
            new(o, &mut note_data)?;
            data::save_note_data(&note_data, &data_path).with_context(|| {
                format!("Failed to write data to {}", data_path.to_string_lossy())
            })?;
        }
    }
    Ok(())
}
fn new(options: NewNoteOptions, note_data: &mut NoteData) -> anyhow::Result<()> {
    if note_data.has_note(&options.name) {
        return Err(anyhow::anyhow!(
            "Note named \"{}\" already exists",
            options.name
        ));
    }
    let editor = std::env::var("EDITOR").context("Editor not set")?;
    let filename = options
        .filename
        .unwrap_or(push_os_string(OsString::from(options.name.clone()), ".md"));

    let note_path =
        Path::new(&options.directory.expect("TODO add default dir")).join(Path::new(&filename));

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

    note_data.add_note(
        options.name,
        absolute_path(&note_path).with_context(|| {
            format!(
                "Failed to convert {} into an absolute path",
                note_path.to_string_lossy()
            )
        })?,
    );
    println!("Saved note");
    Ok(())
}
fn edit(options: EditNoteOptions, note_data: &NoteData) -> anyhow::Result<()> {
    let editor = std::env::var("EDITOR").context("Editor not set")?;

    let path = note_data.get_note(&options.name)
        .context("Note with this name does not exist")?;

    let _ = Command::new(editor)
        .arg(path)
        .status()
        .context("Failed to save note")?;
    println!("Saved note");
    Ok(())
}

fn push_os_string(mut s: OsString, ext: &str) -> OsString {
    s.push(ext);
    s
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
