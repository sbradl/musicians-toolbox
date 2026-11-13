pub mod support;

use mt_music_kernel as _;
use std::fs;

use mt_score::{LARGEST_TAB_FILE, ScoreHeader, Unreadable, read_header};
use support::{BASS_E_STANDARD, DROP_D, E_STANDARD, GuitarProFile, OPEN_G, zip_holding};

const FIXTURE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/drop-d-and-bass.gp5"
);

#[test]
fn a_guitar_pro_5_file_gives_its_tracks_tunings() {
    let tab_file = GuitarProFile::version(5, 0)
        .with_strings(&E_STANDARD)
        .bytes();

    let header = read_header(&tab_file);

    assert_tunings(header, &["6 E Standard"]);
}

#[test]
fn a_guitar_pro_5_10_file_gives_its_tracks_tunings() {
    let tab_file = GuitarProFile::version(5, 10)
        .with_strings(&DROP_D)
        .with_strings(&BASS_E_STANDARD)
        .bytes();

    let header = read_header(&tab_file);

    assert_tunings(header, &["6 Drop D", "4 E Standard"]);
}

#[test]
fn a_guitar_pro_4_file_gives_its_tracks_tunings() {
    let tab_file = GuitarProFile::version(4, 6).with_strings(&OPEN_G).bytes();

    let header = read_header(&tab_file);

    assert_tunings(header, &["6 Custom"]);
}

#[test]
fn a_guitar_pro_3_file_gives_its_tracks_tunings() {
    let tab_file = GuitarProFile::version(3, 0)
        .with_strings(&E_STANDARD)
        .with_strings(&DROP_D)
        .bytes();

    let header = read_header(&tab_file);

    assert_tunings(header, &["6 E Standard", "6 Drop D"]);
}

#[test]
fn a_drum_track_has_no_tuning() {
    let tab_file = GuitarProFile::version(5, 0)
        .with_drums()
        .with_strings(&E_STANDARD)
        .bytes();

    let header = read_header(&tab_file);

    assert_tunings(header, &["no Tuning", "6 E Standard"]);
}

#[test]
fn a_zip_holding_one_tab_file_is_read() {
    let tab_file = GuitarProFile::version(5, 0).with_strings(&DROP_D).bytes();
    let zip = zip_holding(&[
        ("readme.txt", b"not a tab"),
        ("Invented Song.gp5", &tab_file),
    ]);

    let header = read_header(&zip);

    assert_tunings(header, &["6 Drop D"]);
}

#[test]
fn a_zip_without_a_tab_file_is_unreadable() {
    let zip = zip_holding(&[("readme.txt", b"not a tab")]);

    let header = read_header(&zip);

    assert_eq!(header, Err(Unreadable));
}

#[test]
fn a_zip_inside_a_zip_is_unreadable() {
    let tab_file = GuitarProFile::version(5, 0).with_strings(&DROP_D).bytes();
    let inner = zip_holding(&[("Invented Song.gp5", &tab_file)]);
    let zip = zip_holding(&[("Invented Song.zip", &inner)]);

    let header = read_header(&zip);

    assert_eq!(header, Err(Unreadable));
}

#[test]
fn a_truncated_guitar_pro_5_file_is_unreadable() {
    let mut tab_file = GuitarProFile::version(5, 0)
        .with_strings(&E_STANDARD)
        .bytes();
    tab_file.truncate(tab_file.len() / 2);

    let header = read_header(&tab_file);

    assert_eq!(header, Err(Unreadable));
}

#[test]
fn a_guitar_pro_2_file_is_unreadable() {
    let tab_file = GuitarProFile::version(2, 21)
        .with_strings(&E_STANDARD)
        .bytes();

    let header = read_header(&tab_file);

    assert_eq!(header, Err(Unreadable));
}

#[test]
fn a_guitar_pro_file_without_tracks_is_unreadable() {
    let tab_file = GuitarProFile::version(5, 0).bytes();

    let header = read_header(&tab_file);

    assert_eq!(header, Err(Unreadable));
}

#[test]
fn a_file_of_no_known_format_is_unreadable() {
    let header = read_header(b"Just some text that is no tab at all.");

    assert_eq!(header, Err(Unreadable));
}

#[track_caller]
fn assert_tunings(header: Result<ScoreHeader, Unreadable>, expected: &[&str]) {
    assert_eq!(tunings_in(header), expected);
}

fn tunings_in(header: Result<ScoreHeader, Unreadable>) -> Vec<String> {
    let Ok(header) = header else {
        return vec!["Unreadable".to_owned()];
    };
    let mut tunings = Vec::new();
    for track in header.tracks {
        match track.tuning {
            Some(tuning) => tunings.push(format!("{} {}", tuning.strings(), tuning.name())),
            None => tunings.push("no Tuning".to_owned()),
        }
    }

    tunings
}

#[test]
fn a_file_larger_than_the_largest_tab_file_is_unreadable() {
    let mut tab_file = GuitarProFile::version(5, 0)
        .with_strings(&E_STANDARD)
        .bytes();
    tab_file.resize(LARGEST_TAB_FILE + 1, 0);

    let header = read_header(&tab_file);

    assert_eq!(header, Err(Unreadable));
}

#[test]
fn the_fixture_for_the_apps_tests_is_what_the_builder_writes() {
    let built = GuitarProFile::version(5, 0)
        .with_strings(&DROP_D)
        .with_drums()
        .with_strings(&BASS_E_STANDARD)
        .with_strings(&DROP_D)
        .bytes();

    if std::env::var_os("WRITE_FIXTURES").is_some() {
        drop(fs::write(FIXTURE, &built));
    }

    let fixture = fs::read(FIXTURE).unwrap_or_default();

    assert!(
        fixture == built,
        "{FIXTURE} differs from the builder; write it anew with: WRITE_FIXTURES=1 cargo test -p mt-score"
    );
}
