use mt_library::{Library, Song};

fn scanned(files: &[&str]) -> Vec<Song> {
    Library::scanned(
        files
            .iter()
            .map(|file| (format!("/music/{file}"), (*file).to_owned())),
    )
    .songs()
    .to_vec()
}

fn song(file: &str, title: &str) -> Song {
    Song {
        id: format!("/music/{file}"),
        title: title.to_owned(),
    }
}

#[test]
fn every_tab_file_extension_in_any_case_becomes_a_song_and_other_files_do_not() {
    assert_eq!(
        scanned(&[
            "a.GP3", "b.gp4", "c.Gp5", "d.gpx", "e.GP", "f.tg", "g.PTB", "h.txt", "gp5", "i.gp5x",
        ]),
        vec![
            song("a.GP3", "a"),
            song("b.gp4", "b"),
            song("c.Gp5", "c"),
            song("d.gpx", "d"),
            song("e.GP", "e"),
            song("f.tg", "f"),
            song("g.PTB", "g"),
        ]
    );
}

#[test]
fn songs_are_titled_by_file_name_and_sorted_ignoring_case() {
    assert_eq!(
        scanned(&["Zombie.gp5", "angie.gp4", "Live.Wire.gp5", "Blackbird.tg"]),
        vec![
            song("angie.gp4", "angie"),
            song("Blackbird.tg", "Blackbird"),
            song("Live.Wire.gp5", "Live.Wire"),
            song("Zombie.gp5", "Zombie"),
        ]
    );
}

#[test]
fn equal_titles_are_both_kept() {
    let songs = Library::scanned([
        ("/music/b/Angie.gp4".to_owned(), "Angie.gp4".to_owned()),
        ("/music/a/Angie.gp4".to_owned(), "Angie.gp4".to_owned()),
    ]);

    assert_eq!(
        songs.songs(),
        [
            Song {
                id: "/music/a/Angie.gp4".to_owned(),
                title: "Angie".to_owned(),
            },
            Song {
                id: "/music/b/Angie.gp4".to_owned(),
                title: "Angie".to_owned(),
            },
        ]
    );
}
