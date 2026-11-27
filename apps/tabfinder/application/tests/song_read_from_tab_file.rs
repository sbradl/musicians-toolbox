use mt_library as _;
use mt_music_kernel as _;
use mt_score as _;

use mt_tabfinder_application::{Song, WrittenTempo, song_read_from_tab_file};

#[test]
fn a_tab_file_that_could_not_be_read_leaves_its_song_unreadable_under_its_file_name() {
    let found = Song::titled("Wexmoor");

    let song = song_read_from_tab_file(found, None);

    let mut unreadable = Song::titled("Wexmoor");
    unreadable.is_unreadable = true;
    assert_eq!(song, unreadable);
}

#[test]
fn a_tab_file_gives_its_song_the_written_tempos_it_states() {
    let tab_file = include_bytes!("../../../../contexts/score/tests/fixtures/drop-d-and-bass.gp5");

    let song = song_read_from_tab_file(Song::titled("Wexmoor"), Some(tab_file));

    assert_eq!(
        song.written_tempos,
        [
            WrittenTempo::beats_per_minute(120),
            WrittenTempo::beats_per_minute(140)
        ]
    );
}
