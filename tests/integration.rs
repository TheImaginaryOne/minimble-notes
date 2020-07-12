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
fn edit_multiple_times() {
    let tmp = tempdir().unwrap();
    for _ in 0..3 {
        let opts = EditNoteOptions {
            name: Some("bob".into()),
            dir_tag: None,
            remove_dir_tag: None,
        };
        edit(opts, tmp.path(), &mut NoteData::default(), MockEditor {}).expect("Edit failed");
    }
    assert_eq!(files_in_dir(&tmp.path()), vec!["bob.md"]);
}
#[test]
fn edit_basic() {
    let tmp = tempdir().unwrap();
    edit(
        EditNoteOptions {
            name: Some("bob".into()),
            dir_tag: None,
            remove_dir_tag: None,
        },
        tmp.path(),
        &mut NoteData::default(),
        MockEditor {},
    )
    .expect("Edit failed");
    assert_eq!(files_in_dir(&tmp.path()), vec!["bob.md"]);
}
