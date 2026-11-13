use std::fs;
use std::io;

use mt_tabfinder_application::{LastScan, LibraryLocation, Song};
use mt_tabfinder_fs::FsLastScan;

pub mod common;

use common::TempDir;

fn last_scan(config: &TempDir) -> FsLastScan {
    FsLastScan::in_config_dir(config.path().join("tabfinder"))
}

fn tabs() -> LibraryLocation {
    LibraryLocation::new("file:///home/me/Tabs")
}

fn config_with_hand_edited_last_scan(name: &str, contents: &str) -> io::Result<TempDir> {
    let config = TempDir::new(name);
    let tabfinder_dir = config.path().join("tabfinder");
    fs::create_dir_all(&tabfinder_dir)?;
    fs::write(tabfinder_dir.join("last-scan"), contents)?;

    Ok(config)
}

fn saving_is_blocked(config: &TempDir) -> io::Result<()> {
    fs::create_dir_all(config.path().join("tabfinder").join("last-scan.unfinished"))
}

fn song(title: &str, artist: Option<&str>, album: Option<&str>) -> Song {
    Song {
        title: title.to_owned(),
        artist: artist.map(str::to_owned),
        album: album.map(str::to_owned),
    }
}

#[test]
fn nothing_is_kept_in_a_fresh_config_dir() {
    let config = TempDir::new("scan-fresh");

    assert_eq!(last_scan(&config).songs_of(&tabs()), None);
}

#[test]
fn a_kept_scan_survives_a_restart() {
    let config = TempDir::new("scan-restart");
    let songs = [
        song("Wexmoor", Some("Corvid Lane"), Some("Saltglass")),
        song("Brasswing", Some("Corvid Lane"), None),
        song("Lone Ember", None, None),
    ];

    last_scan(&config).keep(&tabs(), &songs);

    assert_eq!(last_scan(&config).songs_of(&tabs()), Some(songs.to_vec()));
}

#[test]
fn a_scan_without_songs_is_kept_too() {
    let config = TempDir::new("scan-empty");

    last_scan(&config).keep(&tabs(), &[]);

    assert_eq!(last_scan(&config).songs_of(&tabs()), Some(vec![]));
}

#[test]
fn names_with_tabs_line_breaks_and_percent_signs_survive_a_restart() {
    let config = TempDir::new("scan-odd-names");
    let songs = [song("Wex\tmoor\n100%", Some("Corvid\r\nLane"), Some("%41"))];

    last_scan(&config).keep(&tabs(), &songs);

    assert_eq!(last_scan(&config).songs_of(&tabs()), Some(songs.to_vec()));
}

#[test]
fn the_scan_of_another_library_location_is_not_used() {
    let config = TempDir::new("scan-other");
    let other = LibraryLocation::new("file:///home/me/Other");

    last_scan(&config).keep(&tabs(), &[song("Wexmoor", None, None)]);

    assert_eq!(last_scan(&config).songs_of(&other), None);
}

#[test]
fn keeping_a_scan_replaces_the_one_before() {
    let config = TempDir::new("scan-replace");
    last_scan(&config).keep(&tabs(), &[song("Wexmoor", None, None)]);

    last_scan(&config).keep(&tabs(), &[song("Brasswing", None, None)]);

    let brasswing_only = vec![song("Brasswing", None, None)];
    assert_eq!(last_scan(&config).songs_of(&tabs()), Some(brasswing_only));
}

#[test]
fn a_last_scan_tabfinder_cannot_read_is_no_scan() -> io::Result<()> {
    let config = config_with_hand_edited_last_scan(
        "scan-unreadable",
        "file:///home/me/Tabs\nonly a title, no artist or album fields\n",
    )?;

    assert_eq!(last_scan(&config).songs_of(&tabs()), None);

    Ok(())
}

#[test]
fn a_scan_that_could_not_be_kept_leaves_no_outdated_scan_behind() -> io::Result<()> {
    let config = TempDir::new("scan-not-kept");
    last_scan(&config).keep(&tabs(), &[song("Wexmoor", None, None)]);
    saving_is_blocked(&config)?;

    last_scan(&config).keep(&tabs(), &[song("Brasswing", None, None)]);

    assert_eq!(last_scan(&config).songs_of(&tabs()), None);

    Ok(())
}
