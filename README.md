# Minimble, a note taking app

```
minimble 
A simple terminal notes app

USAGE:
    minimble-bin <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    edit       
    help       Prints this message or the help of the given subcommand(s)
    remove     
    rename     
    show       
    tag-dir
```

## Instructions
The `NOTES_DIR` and `EDITOR` envvars must be configured. All notes will be placed in `NOTES_DIR`.

The commands are

* `minimble edit my_note`: open a note or create a new one if it is non-existent.
* `rename my_note new_name`: rename a note
* `remove my_note`: remove a note
* `tag-dir [--add-dir-tag home/j/dir] [--remove-dir-tag /bob]`
  Basically a directory can be associated to a note such that
  if I run `edit @` (or `show @` etc), then it will automatically open the note associated with the current working directory, or the closest parent directory
* `show` subcommand to show all names of notes (as a list),
  and `show my_note` to print a note to the terminal

## Note

This is personalised for my own use so some 'common' features are not implemented.
