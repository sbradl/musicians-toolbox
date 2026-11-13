use std::fs;
use std::io;

use mt_tabfinder_application::{LibraryLocation, RememberedLibraryLocation};
use mt_tabfinder_fs::FsRememberedLibraryLocation;

pub mod common;

use common::{TempDir, saving_is_blocked};

fn remembered_library_location(config: &TempDir) -> FsRememberedLibraryLocation {
    FsRememberedLibraryLocation::in_config_dir(config.path().join("tabfinder"))
}

fn tabs() -> LibraryLocation {
    LibraryLocation::new("/home/me/Tabs")
}

fn config_with_hand_edited_location(name: &str, contents: &str) -> io::Result<TempDir> {
    let config = TempDir::new(name);
    let tabfinder_dir = config.path().join("tabfinder");
    fs::create_dir_all(&tabfinder_dir)?;
    fs::write(tabfinder_dir.join("library-location"), contents)?;

    Ok(config)
}

#[test]
fn nothing_is_remembered_in_a_fresh_config_dir() {
    let config = TempDir::new("fresh");

    assert_eq!(remembered_library_location(&config).remembered(), None);
}

#[test]
fn a_remembered_library_location_survives_a_restart() {
    let config = TempDir::new("restart");

    remembered_library_location(&config).remember(&tabs());

    assert_eq!(
        remembered_library_location(&config).remembered(),
        Some(tabs())
    );
}

#[test]
fn a_forgotten_location_stays_forgotten_after_a_restart() {
    let config = TempDir::new("forget");
    remembered_library_location(&config).remember(&tabs());

    remembered_library_location(&config).forget();

    assert_eq!(remembered_library_location(&config).remembered(), None);
}

#[test]
fn a_hand_edited_location_ignores_its_trailing_line_break() -> io::Result<()> {
    let config = config_with_hand_edited_location("edited", "/home/me/Tabs\n")?;

    assert_eq!(
        remembered_library_location(&config).remembered(),
        Some(tabs())
    );

    Ok(())
}

#[test]
fn a_hand_edited_empty_location_is_nothing_remembered() -> io::Result<()> {
    let config = config_with_hand_edited_location("empty", "\n")?;

    assert_eq!(remembered_library_location(&config).remembered(), None);

    Ok(())
}

#[test]
fn a_location_that_could_not_be_remembered_leaves_no_outdated_location_behind() -> io::Result<()> {
    let config = TempDir::new("not-remembered");
    remembered_library_location(&config).remember(&tabs());
    saving_is_blocked(&config, "library-location")?;

    remembered_library_location(&config).remember(&LibraryLocation::new("/home/me/Other"));

    assert_eq!(remembered_library_location(&config).remembered(), None);

    Ok(())
}
