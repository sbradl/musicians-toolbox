pub mod support;

use support::{
    SongRow as _, TabFinder, because, listing, no_folder_chosen, no_folder_chosen_because, row,
    scanning, song,
};

#[test]
fn starting_without_a_remembered_library_location_shows_no_folder_chosen() {
    let tabfinder = TabFinder::without_folders().start();

    tabfinder.assert_library_screen(&no_folder_chosen());
}

#[test]
fn choose_folder_asks_the_platform_to_pick_a_folder_with_a_titled_dialog() {
    let tabfinder = TabFinder::without_folders().start();

    tabfinder.musician_clicks_choose_folder();

    tabfinder.assert_platform_was_asked_to_pick_a_folder_titled("Choose the Library folder");
}

#[test]
fn picking_a_folder_lists_its_songs() {
    let tabfinder = TabFinder::with_folder("/music", &["Wexmoor"]).start();

    tabfinder.musician_picks_folder("/music");

    tabfinder.assert_library_screen(&listing("1 Song", &["Wexmoor"]));
}

#[test]
fn songs_are_listed_by_title_ignoring_case_and_equal_titles_are_both_listed() {
    let tabfinder =
        TabFinder::with_folder("/music", &["Wexmoor", "amberline", "Wexmoor", "Brasswing"]).start();

    tabfinder.musician_picks_folder("/music");

    tabfinder.assert_status("4 Songs");
    tabfinder.assert_song_titles(&["amberline", "Brasswing", "Wexmoor", "Wexmoor"]);
}

#[test]
fn picking_a_folder_without_songs_shows_no_songs_found() {
    let tabfinder = TabFinder::with_folder("/music", &[]).start();

    tabfinder.musician_picks_folder("/music");

    tabfinder.assert_library_screen(&listing("No Songs found", &[]));
}

#[test]
fn picking_a_folder_shows_scanning_until_the_scan_is_done() {
    let tabfinder = TabFinder::with_folder("/music", &["Wexmoor"]).start();

    tabfinder.musician_picks_folder("/music");

    assert_eq!(
        tabfinder.library_screens_shown(),
        [
            no_folder_chosen(),
            scanning(),
            listing("1 Song", &["Wexmoor"])
        ]
    );
}

#[test]
fn cancelling_the_folder_pick_keeps_the_library() {
    let tabfinder = TabFinder::with_folder("/music", &["Wexmoor"]).start();
    tabfinder.musician_picks_folder("/music");
    let shown_before = tabfinder.library_screens_shown();

    tabfinder.musician_cancels_the_folder_pick();

    assert_eq!(tabfinder.library_screens_shown(), shown_before);
}

#[test]
fn without_a_folder_dialog_the_library_stays_and_the_screen_says_why() {
    let tabfinder = TabFinder::with_folder("/music", &["Wexmoor"]).start();
    tabfinder.musician_picks_folder("/music");

    tabfinder.musician_finds_no_folder_dialog();

    tabfinder.assert_library_screen(&because(
        listing("1 Song", &["Wexmoor"]),
        "The system folder dialog is not available.",
    ));
}

#[test]
fn picking_a_folder_that_is_gone_shows_no_folder_chosen_and_why() {
    let tabfinder = TabFinder::without_folders().start();

    tabfinder.musician_picks_folder("/gone");

    tabfinder.assert_library_screen(&no_folder_chosen_because(
        "The Library folder is gone. It may have been moved or deleted.",
    ));
}

#[test]
fn picking_a_folder_tabfinder_may_not_read_shows_no_folder_chosen_and_why() {
    let tabfinder = TabFinder::with_folder_it_may_not_read("/locked").start();

    tabfinder.musician_picks_folder("/locked");

    tabfinder.assert_library_screen(&no_folder_chosen_because(
        "TabFinder may not read the Library folder.",
    ));
}

#[test]
fn picking_another_folder_replaces_the_library() {
    let tabfinder = TabFinder::with_folder("/old", &["Amberline"])
        .with_folder("/new", &["Brasswing"])
        .start();
    tabfinder.musician_picks_folder("/old");
    let shown_before = tabfinder.library_screens_shown().len();

    tabfinder.musician_picks_folder("/new");

    assert_eq!(
        tabfinder.library_screens_shown()[shown_before..],
        [scanning(), listing("1 Song", &["Brasswing"])]
    );
}

#[test]
fn starting_with_a_remembered_library_location_lists_its_songs() {
    let tabfinder = TabFinder::with_folder("/music", &["Wexmoor"])
        .remembering("/music")
        .start();

    tabfinder.assert_library_screen(&listing("1 Song", &["Wexmoor"]));
}

#[test]
fn starting_with_a_remembered_library_location_goes_straight_to_scanning() {
    let tabfinder = TabFinder::with_folder("/music", &["Wexmoor"])
        .remembering("/music")
        .start();

    assert_eq!(tabfinder.statuses_shown(), ["Scanning…", "1 Song"]);
}

#[test]
fn a_picked_folder_becomes_the_remembered_library_location() {
    let tabfinder = TabFinder::with_folder("/music", &["Wexmoor"]).start();

    tabfinder.musician_picks_folder("/music");

    tabfinder.assert_remembers("/music");
}

#[test]
fn starting_with_a_remembered_library_location_that_is_gone_forgets_it() {
    let tabfinder = TabFinder::without_folders().remembering("/gone").start();

    tabfinder.assert_status("No folder chosen");
    tabfinder.assert_remembers_nothing();
}

#[test]
fn picking_a_folder_that_is_gone_keeps_the_remembered_library_location() {
    let tabfinder = TabFinder::with_folder("/music", &["Wexmoor"])
        .remembering("/music")
        .start();

    tabfinder.musician_picks_folder("/gone");

    tabfinder.assert_remembers("/music");
}

#[test]
fn picking_the_remembered_library_location_after_its_folder_is_deleted_forgets_it() {
    let tabfinder = TabFinder::with_folder("/music", &["Wexmoor"])
        .remembering("/music")
        .start();
    tabfinder.folder_is_deleted("/music");

    tabfinder.musician_picks_folder("/music");

    tabfinder.assert_status("No folder chosen");
    tabfinder.assert_remembers_nothing();
}

#[test]
fn rescan_lists_the_songs_now_in_the_library_folder() {
    let tabfinder = TabFinder::with_folder("/music", &["Wexmoor"]).start();
    tabfinder.musician_picks_folder("/music");
    tabfinder.folder_now_holds("/music", &["Wexmoor", "Brasswing"]);

    tabfinder.musician_clicks_rescan();

    tabfinder.assert_library_screen(&listing("2 Songs", &["Brasswing", "Wexmoor"]));
}

#[test]
fn rescan_is_offered_once_a_library_is_open_even_one_without_songs() {
    let tabfinder = TabFinder::with_folder("/music", &[]).start();
    tabfinder.assert_rescan_not_offered();

    tabfinder.musician_picks_folder("/music");

    tabfinder.assert_rescan_offered();
}

#[test]
fn after_a_rescan_a_message_tells_how_many_songs_and_unreadable_files_it_found() {
    let tabfinder = TabFinder::with_folder("/music", &["Wexmoor", "Brasswing"]).start();
    tabfinder.musician_picks_folder("/music");

    tabfinder.musician_clicks_rescan();

    tabfinder.assert_message_shown("2 Songs, 0 unreadable");
}

#[test]
fn after_a_rescan_of_one_song_the_message_says_song() {
    let tabfinder = TabFinder::with_folder("/music", &["Wexmoor"]).start();
    tabfinder.musician_picks_folder("/music");

    tabfinder.musician_clicks_rescan();

    tabfinder.assert_message_shown("1 Song, 0 unreadable");
}

#[test]
fn a_rescan_of_a_folder_that_is_gone_shows_why_and_no_message() {
    let tabfinder = TabFinder::with_folder("/music", &["Wexmoor"]).start();
    tabfinder.musician_picks_folder("/music");
    tabfinder.folder_is_deleted("/music");

    tabfinder.musician_clicks_rescan();

    tabfinder.assert_library_screen(&no_folder_chosen_because(
        "The Library folder is gone. It may have been moved or deleted.",
    ));
    tabfinder.assert_no_message_shown();
}

#[test]
fn each_song_row_shows_its_artist_and_album_below_the_title() {
    let tabfinder = TabFinder::with_folder_of_songs(
        "/music",
        &[
            song("Wexmoor").by("Corvid Lane").on("Saltglass"),
            song("Brasswing").by("Corvid Lane"),
            song("Amberline"),
        ],
    )
    .start();

    tabfinder.musician_picks_folder("/music");

    tabfinder.assert_song_rows(&[
        row("Amberline"),
        row("Brasswing").above("Corvid Lane"),
        row("Wexmoor").above("Corvid Lane · Saltglass"),
    ]);
}
