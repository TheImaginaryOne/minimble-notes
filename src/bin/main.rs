use anyhow::Context;
use clap::Clap;
use minimble::{data, edit, rename, show_notes, tag_dir, util::Editor, RemoveNoteOptions, remove, EditNoteOptions, RenameNoteOptions, TagDirOptions};
use std::path::Path;

#[derive(Clap)]
#[clap(about("A simple terminal notes app"))]
struct Options {
    #[clap(subcommand)]
    subcommand: SubCommand,
}
#[derive(Clap)]
enum SubCommand {
    Edit(EditNoteOptions),
    #[clap(alias("rm"))]
    Remove(RemoveNoteOptions),
    Rename(RenameNoteOptions),
    TagDir(TagDirOptions),
    Show,
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

    // Grabs metadata, in same place as the notes directory
    let data_path = Path::new(&notes_dir).join(Path::new("minimble_data.json"));
    let mut note_data = data::load_note_data(&data_path).with_context(|| {
        format!(
            "Failed to read note data from {}",
            data_path.to_string_lossy()
        )
    })?;
    match options.subcommand {
        SubCommand::Edit(o) => {
            let command = std::env::var("EDITOR").context("Editor not set")?;
            edit(o, Path::new(&notes_dir), &mut note_data, Editor { command })?;
        }
        SubCommand::TagDir(opts) => {
            tag_dir(opts, Path::new(&notes_dir), &mut note_data)?;
        }
        SubCommand::Show => show_notes(Path::new(&notes_dir))?,
        SubCommand::Remove(opts) => remove(opts, Path::new(&notes_dir), &mut note_data)?,
        SubCommand::Rename(opts) => rename(opts, Path::new(&notes_dir), &mut note_data)?,
    }
    data::save_note_data(&note_data, &data_path).with_context(|| {
        format!("Failed to write data to {}", data_path.to_string_lossy())
    })?;
    Ok(())
}
