#![no_main]

use libfuzzer_sys::arbitrary::{Arbitrary as _, Unstructured};
use libfuzzer_sys::fuzz_target;
use mt_tabfinder_application::{LastScan, LibraryLocation, Song};
use mt_tabfinder_fs::FsLastScan;
use mt_tabfinder_fs_fuzz::{config_dir, config_dir_with};

fuzz_target!(|bytes: &[u8]| {
    any_last_scan_file_is_read_without_crashing(bytes);
    kept_songs_come_back_unchanged(bytes);
});

fn any_last_scan_file_is_read_without_crashing(bytes: &[u8]) {
    let last_scan = FsLastScan::in_config_dir(config_dir_with("last-scan", bytes));

    drop(last_scan.songs_of(&LibraryLocation::new("file:///home/me/Tabs")));
}

fn kept_songs_come_back_unchanged(bytes: &[u8]) {
    let Some((location, songs)) = location_and_songs(bytes) else {
        return;
    };
    let last_scan = FsLastScan::in_config_dir(config_dir());

    last_scan.keep(&location, &songs);

    assert_eq!(last_scan.songs_of(&location), Some(songs));
}

fn location_and_songs(bytes: &[u8]) -> Option<(LibraryLocation, Vec<Song>)> {
    let mut input = Unstructured::new(bytes);
    let location = String::arbitrary(&mut input).ok()?;
    let fields = Vec::<(String, Option<String>, Option<String>)>::arbitrary(&mut input).ok()?;
    let songs = fields.into_iter().map(song_as_found_in_a_folder).collect();

    Some((LibraryLocation::new(&location), songs))
}

fn song_as_found_in_a_folder(
    (title, artist, album): (String, Option<String>, Option<String>),
) -> Song {
    Song {
        title,
        artist: artist.filter(|name| !name.is_empty()),
        album: album.filter(|name| !name.is_empty()),
    }
}
