use std::fs;
use std::io;

use mt_tabfinder_application::RememberedLibraryLocation;
use mt_tabfinder_fs::FsRememberedLibraryLocation;

pub mod common;

use common::TempDir;

fn remembered_library_location(config: &TempDir) -> FsRememberedLibraryLocation {
    FsRememberedLibraryLocation::in_config_dir(config.path().join("tabfinder"))
}

#[test]
fn nothing_is_remembered_in_a_fresh_config_dir() {
    let config = TempDir::new("fresh");

    assert_eq!(remembered_library_location(&config).remembered(), None);
}

#[test]
fn a_remembered_library_location_survives_a_restart() {
    let config = TempDir::new("restart");

    remembered_library_location(&config).remember("/home/me/Tabs");

    assert_eq!(
        remembered_library_location(&config).remembered(),
        Some("/home/me/Tabs".to_owned())
    );
}

#[test]
fn a_forgotten_location_stays_forgotten_after_a_restart() {
    let config = TempDir::new("forget");
    remembered_library_location(&config).remember("/home/me/Tabs");

    remembered_library_location(&config).forget();

    assert_eq!(remembered_library_location(&config).remembered(), None);
}

#[test]
fn a_hand_edited_location_ignores_the_trailing_line_break_and_an_empty_one_is_none()
-> io::Result<()> {
    let edited = TempDir::new("edited");
    let empty = TempDir::new("empty");
    for (config, contents) in [(&edited, "/home/me/Tabs\n"), (&empty, "\n")] {
        fs::create_dir_all(config.path().join("tabfinder"))?;
        fs::write(
            config.path().join("tabfinder").join("library-location"),
            contents,
        )?;
    }

    assert_eq!(
        (
            remembered_library_location(&edited).remembered(),
            remembered_library_location(&empty).remembered()
        ),
        (Some("/home/me/Tabs".to_owned()), None)
    );
    Ok(())
}
