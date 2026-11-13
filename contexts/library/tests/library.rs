use mt_library::{Library, Song};
use mt_music_kernel as _;

fn library_of(titles: &[&str]) -> Library {
    let mut songs = Vec::new();
    for title in titles {
        songs.push(Song::titled(title));
    }

    Library::of(songs)
}

fn titles_in(library: &Library) -> Vec<String> {
    let mut titles = Vec::new();
    for song in library.songs() {
        titles.push(song.title.clone());
    }

    titles
}

#[test]
fn songs_are_sorted_by_title_ignoring_case() {
    let library = library_of(&["Zephyrine", "amberline", "Lone.Pine", "Brasswing"]);

    assert_eq!(
        titles_in(&library),
        ["amberline", "Brasswing", "Lone.Pine", "Zephyrine"]
    );
}

#[test]
fn songs_with_the_same_title_are_both_kept() {
    let library = library_of(&["Amberline", "Amberline"]);

    assert_eq!(titles_in(&library), ["Amberline", "Amberline"]);
}

#[test]
fn an_artist_spelled_in_other_letter_case_takes_the_spelling_most_songs_use() {
    let library = Library::of(vec![
        song("Wexmoor", "Corvid Lane", "Saltglass"),
        song("Brasswing", "CORVID LANE", "Saltglass"),
        song("Ashgrove Reel", "Corvid Lane", "Saltglass"),
    ]);

    assert_eq!(
        artists_and_albums_in(&library),
        ["Corvid Lane - Saltglass"; 3]
    );
}

#[test]
fn an_album_spelled_in_other_letter_case_takes_the_spelling_most_songs_use() {
    let library = Library::of(vec![
        song("Wexmoor", "Corvid Lane", "Tidewater Hymns"),
        song("Brasswing", "Corvid Lane", "Tidewater hymns"),
        song("Ashgrove Reel", "Corvid Lane", "Tidewater hymns"),
    ]);

    assert_eq!(
        artists_and_albums_in(&library),
        ["Corvid Lane - Tidewater hymns"; 3]
    );
}

#[test]
fn albums_of_different_artists_keep_their_own_spelling() {
    let library = Library::of(vec![
        song("Ashgrove Reel", "Corvid Lane", "GREATEST HITS"),
        song("Brasswing", "Corvid Lane", "GREATEST HITS"),
        song("Wexmoor", "Marrow Bay", "Greatest Hits"),
    ]);

    assert_eq!(
        artists_and_albums_in(&library),
        [
            "Corvid Lane - GREATEST HITS",
            "Corvid Lane - GREATEST HITS",
            "Marrow Bay - Greatest Hits"
        ]
    );
}

#[test]
fn between_equally_used_spellings_the_one_with_fewer_capitals_wins() {
    let library = Library::of(vec![
        song("Wexmoor", "MARROW BAY", "Saltglass"),
        song("Brasswing", "Marrow Bay", "Saltglass"),
    ]);

    assert_eq!(
        artists_and_albums_in(&library),
        ["Marrow Bay - Saltglass"; 2]
    );
}

fn song(title: &str, artist: &str, album: &str) -> Song {
    let mut song = Song::titled(title);
    song.artist = Some(artist.to_owned());
    song.album = Some(album.to_owned());

    song
}

fn artists_and_albums_in(library: &Library) -> Vec<String> {
    let mut names = Vec::new();
    for song in library.songs() {
        let artist = song.artist.clone().unwrap_or_default();
        let album = song.album.clone().unwrap_or_default();
        names.push(format!("{artist} - {album}"));
    }

    names
}
