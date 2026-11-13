#![no_main]

use libfuzzer_sys::fuzz_target;
use mt_tabfinder_application::{Song, song_read_from_tab_file};
use mt_tabfinder_fs as _;
use mt_tabfinder_fs_fuzz as _;

fuzz_target!(|bytes: &[u8]| {
    any_tab_file_is_read_without_crashing(bytes);
});

fn any_tab_file_is_read_without_crashing(bytes: &[u8]) {
    drop(song_read_from_tab_file(Song::titled("Fuzzed"), Some(bytes)));
}
