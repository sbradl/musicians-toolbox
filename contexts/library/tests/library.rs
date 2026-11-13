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
