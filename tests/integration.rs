use minimble::data::*;
use minimble::util::MockEditor;
use minimble::*;

use std::fs::read_dir;
use std::path::Path;
use tempfile::tempdir;

fn files_in_dir(dir: &Path) -> Vec<String> {
    read_dir(dir)
        .unwrap()
        .map(|f| f.unwrap().file_name().to_string_lossy().into())
        .collect()
}

#[test]
fn edit_multiple_times() -> anyhow::Result<()> {
    let tmp = tempdir().unwrap();
    for _ in 0..3 {
        let opts = EditNoteOptions {
            name: Some("bob".into()),
        };
        edit(opts, tmp.path(), &mut NoteData::default(), MockEditor {}).expect("Edit failed");
    }
    assert_eq!(files_in_dir(&tmp.path()), vec!["bob.md"]);
    Ok(())
}

#[test]
fn edit_basic() -> anyhow::Result<()> {
    let tmp = tempdir().unwrap();
    edit(
        EditNoteOptions {
            name: Some("bob".into()),
        },
        tmp.path(),
        &mut NoteData::default(),
        MockEditor {},
    )?;
    assert_eq!(files_in_dir(&tmp.path()), vec!["bob.md"]);
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
        EditNoteOptions {
            name: Some("bob".into()),
        },
        tmpp,
        &mut NoteData::default(),
        MockEditor {},
    )?;
    // Add a dir tag
    tag_dir(
        TagDirOptions {
            name: Some("bob".into()),
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
            name: Some("bob".into()),
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
            name: Some("bob".into()),
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
