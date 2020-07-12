use minimble::data::*;
use minimble::util::EditorTrait;
use minimble::*;

use std::fs::read_dir;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use tempfile::tempdir;

fn files_in_dir(dir: &Path) -> Vec<String> {
    read_dir(dir)
        .unwrap()
        .map(|f| f.unwrap().file_name().to_string_lossy().into())
        .collect()
}
pub struct MockEditor {}
impl EditorTrait for MockEditor {
    fn open(&self, path: &Path) -> std::io::Result<bool> {
        let mut f = OpenOptions::new().append(true).create(true).open(path)?;
        f.write(b"38")?;
        Ok(true)
    }
}

#[test]
fn edit_multiple_times() -> anyhow::Result<()> {
    let tmp = tempdir().unwrap();
    for _ in 0..3 {
        let opts = EditNoteOptions { name: "bob".into() };
        edit(opts, tmp.path(), &mut NoteData::default(), MockEditor {}).expect("Edit failed");
    }
    assert_eq!(files_in_dir(&tmp.path()), vec!["bob.md"]);
    Ok(())
}

#[test]
fn edit_basic() -> anyhow::Result<()> {
    let tmp = tempdir().unwrap();
    edit(
        EditNoteOptions { name: "bob".into() },
        tmp.path(),
        &mut NoteData::default(),
        MockEditor {},
    )?;
    assert_eq!(files_in_dir(&tmp.path()), vec!["bob.md"]);
    Ok(())
}

#[test]
fn remove_basic() -> anyhow::Result<()> {
    let tmp = tempdir().unwrap();
    let tmpp = tmp.path();
    let mut note_data = NoteData::default();
    edit(
        EditNoteOptions { name: "bob".into() },
        tmp.path(),
        &mut note_data,
        MockEditor {},
    )?;
    edit(
        EditNoteOptions { name: "bob2".into() },
        tmp.path(),
        &mut note_data,
        MockEditor {},
    )?;
    // Add a dir tag
    tag_dir(
        TagDirOptions {
            name: "bob".into(),
            add_dir_tag: Some(tmpp.into()),
            remove_dir_tag: None,
        },
        tmpp,
        &mut note_data,
    )?;
    // Add another dir tag
    tag_dir(
        TagDirOptions {
            name: "bob2".into(),
            add_dir_tag: Some(tmpp.into()),
            remove_dir_tag: None,
        },
        tmpp,
        &mut note_data,
    )?;
    remove(
        RemoveNoteOptions { name: "bob".into() },
        tmp.path(),
        &mut note_data,
    )?;
    let mut b = NoteData::default();
    b.set_dir_tag(Some("bob2"), tmpp.into());

    assert_eq!(b, note_data);
    assert_eq!(files_in_dir(&tmp.path()), vec!["bob2.md"]);
    Ok(())
}

#[test]
fn tag_dir_basic() -> anyhow::Result<()> {
    let tmp = tempdir().unwrap();
    let tmpp = tmp.path();
    let subp = tmpp.join(&Path::new("/sub"));
    let mut note_data = NoteData::default();
    // Add the file
    edit(
        EditNoteOptions { name: "bob".into() },
        tmpp,
        &mut NoteData::default(),
        MockEditor {},
    )?;
    // Add a dir tag
    tag_dir(
        TagDirOptions {
            name: "bob".into(),
            add_dir_tag: Some(tmpp.into()),
            remove_dir_tag: None,
        },
        tmpp,
        &mut note_data,
    )?;
    let mut b = NoteData::default();
    b.set_dir_tag(Some("bob"), tmpp.into());
    // Check
    assert_eq!(note_data, b);
    // Add another dir tag
    tag_dir(
        TagDirOptions {
            name: "bob".into(),
            add_dir_tag: Some(subp.clone()),
            remove_dir_tag: None,
        },
        tmpp,
        &mut note_data,
    )?;
    b.set_dir_tag(Some("bob"), subp);
    // Check
    assert_eq!(note_data, b);

    // Now remove the dir tag
    tag_dir(
        TagDirOptions {
            name: "bob".into(),
            add_dir_tag: None,
            remove_dir_tag: Some(tmpp.into()),
        },
        tmpp,
        &mut note_data,
    )?;
    b.set_dir_tag(None, tmpp.into());
    // Check
    assert_eq!(note_data, b);

    Ok(())
}

#[test]
fn tag_dir_current() -> anyhow::Result<()> {
    let tmp = tempdir().unwrap();
    let tmpp = tmp.path();
    let mut note_data = NoteData::default();
    // Add the file
    edit(
        EditNoteOptions { name: "bob".into() },
        tmpp,
        &mut note_data,
        MockEditor {},
    )?;
    // Add a dir tag
    tag_dir(
        TagDirOptions {
            name: "bob".into(),
            add_dir_tag: Some(tmpp.into()),
            remove_dir_tag: None,
        },
        tmpp,
        &mut note_data,
    )?;
    std::env::set_current_dir(tmpp)?;
    edit(
        EditNoteOptions { name: "@".into() },
        tmpp,
        &mut note_data,
        MockEditor {},
    )?;
    assert_eq!(files_in_dir(&tmpp), vec!["bob.md"]);
    assert_eq!(
        std::fs::read_to_string(tmpp.join(Path::new("bob.md")))?,
        "3838"
    );
    Ok(())
}
