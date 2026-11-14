use mt_library as _;
use mt_music_kernel as _;
use mt_score as _;

use mt_tabfinder_application::{Song, song_read_from_tab_file};

#[test]
fn a_tab_file_that_could_not_be_read_leaves_its_song_unreadable_under_its_file_name() {
    let found = Song::titled("Wexmoor");

    let song = song_read_from_tab_file(found, None);

    let mut unreadable = Song::titled("Wexmoor");
    unreadable.is_unreadable = true;
    assert_eq!(song, unreadable);
}
