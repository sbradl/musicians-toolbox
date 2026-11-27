pub mod support;

use miniz_oxide as _;
use mt_music_kernel as _;
use quick_xml as _;
use rawzip as _;

use mt_score::{ScoreHeader, Track, Unreadable, read_header};
use support::{
    BASS_E_STANDARD, DROP_D, NameSize, PowerTabFile, Release, TuxGuitar1File, TuxGuitar2Xml,
    tux_guitar_2_file, zip_holding, zip_storing,
};

#[test]
fn a_tux_guitar_1_file_gives_its_title_artist_and_album_and_no_tunings() {
    let tab_file = TuxGuitar1File::default()
        .titled("Wexmoor")
        .by("Corvid Lane")
        .on_album("Saltglass")
        .bytes();

    let header = read_header(&tab_file);

    assert_names_without_tunings(
        header,
        [Some("Wexmoor"), Some("Corvid Lane"), Some("Saltglass")],
    );
}

#[test]
fn a_tux_guitar_1_file_with_blank_names_states_none() {
    let tab_file = TuxGuitar1File::default().by("  ").bytes();

    let header = read_header(&tab_file);

    assert_names_without_tunings(header, [None, None, None]);
}

#[test]
fn a_tux_guitar_1_file_ending_inside_its_names_is_unreadable() {
    let mut tab_file = TuxGuitar1File::default().titled("Wexmoor").bytes();
    let format_and_part_of_title = 1 + 2 * 27 + 1 + 2;
    tab_file.truncate(format_and_part_of_title);

    assert_eq!(read_header(&tab_file), Err(Unreadable));
}

#[test]
fn a_power_tab_file_gives_its_title_artist_and_album_and_no_tunings() {
    let tab_file = PowerTabFile::version(4)
        .titled("Wexmoor")
        .by("Corvid Lane")
        .on_album("Saltglass", Release::PublicAudio)
        .bytes();

    let header = read_header(&tab_file);

    assert_names_without_tunings(
        header,
        [Some("Wexmoor"), Some("Corvid Lane"), Some("Saltglass")],
    );
}

#[test]
fn a_power_tab_1_5_file_released_as_a_video_gives_its_album() {
    let tab_file = PowerTabFile::version(3)
        .titled("Wexmoor")
        .on_album("Saltglass", Release::PublicVideo)
        .bytes();

    let header = read_header(&tab_file);

    assert_names_without_tunings(header, [Some("Wexmoor"), None, Some("Saltglass")]);
}

#[test]
fn an_unreleased_power_tab_song_has_no_album() {
    let tab_file = PowerTabFile::version(4)
        .titled("Wexmoor")
        .on_album("Saltglass", Release::Unreleased)
        .bytes();

    let header = read_header(&tab_file);

    assert_names_without_tunings(header, [Some("Wexmoor"), None, None]);
}

#[test]
fn a_power_tab_lesson_gives_its_title() {
    let tab_file = PowerTabFile::version(4)
        .titled("Wexmoor Scales")
        .as_lesson()
        .bytes();

    let header = read_header(&tab_file);

    assert_names_without_tunings(header, [Some("Wexmoor Scales"), None, None]);
}

#[test]
fn power_tab_names_longer_than_254_letters_are_read() {
    let long_title = "Wexmoor ".repeat(40);
    let tab_file = PowerTabFile::version(4).titled(&long_title).bytes();

    let header = read_header(&tab_file);

    assert_names_without_tunings(header, [Some(long_title.trim()), None, None]);
}

#[test]
fn power_tab_names_sized_in_four_bytes_are_read() {
    let tab_file = PowerTabFile::version(4)
        .titled("Wexmoor")
        .with_names_sized(NameSize::FourBytes)
        .bytes();

    let header = read_header(&tab_file);

    assert_names_without_tunings(header, [Some("Wexmoor"), None, None]);
}

#[test]
fn a_power_tab_file_with_unicode_names_is_unreadable() {
    let tab_file = PowerTabFile::version(4)
        .titled("Wexmoor")
        .with_names_sized(NameSize::Unicode)
        .bytes();

    assert_eq!(read_header(&tab_file), Err(Unreadable));
}

#[test]
fn a_power_tab_1_0_file_is_unreadable() {
    let tab_file = PowerTabFile::version(2).titled("Wexmoor").bytes();

    assert_eq!(read_header(&tab_file), Err(Unreadable));
}

#[test]
fn a_power_tab_file_ending_inside_its_names_is_unreadable() {
    let mut tab_file = PowerTabFile::version(4).titled("Wexmoor").bytes();
    let magic_version_song_content_and_part_of_title = 4 + 2 + 1 + 1 + 1 + 3;
    tab_file.truncate(magic_version_song_content_and_part_of_title);

    assert_eq!(read_header(&tab_file), Err(Unreadable));
}

#[test]
fn a_tux_guitar_2_file_gives_its_names_and_its_tracks_tunings() {
    let content = TuxGuitar2Xml::default()
        .titled("Wexmoor &amp; Tide")
        .by("Corvid &#x4C;ane")
        .with_strings(&DROP_D)
        .with_drums()
        .with_strings(&BASS_E_STANDARD);

    let header = read_header(&tux_guitar_2_file(&content.xml()));

    assert_names_and_tunings(
        header,
        [
            Some("Wexmoor & Tide"),
            Some("Corvid Lane"),
            Some("Saltglass"),
        ],
        &["6 Drop D", "no Tuning", "4 E Standard"],
    );
}

#[test]
fn a_tux_guitar_2_file_storing_its_content_uncompressed_is_read() {
    let content = TuxGuitar2Xml::default().with_strings(&DROP_D);
    let tab_file = zip_storing(&[("content.xml", &content.xml())]);

    let header = read_header(&tab_file);

    assert_names_and_tunings(header, [None, None, Some("Saltglass")], &["6 Drop D"]);
}

#[test]
fn a_tux_guitar_2_track_without_strings_has_no_tuning() {
    let content = TuxGuitar2Xml::default().with_strings_written(&[], 0);

    let header = read_header(&tux_guitar_2_file(&content.xml()));

    assert_names_and_tunings(header, [None, None, Some("Saltglass")], &["no Tuning"]);
}

#[test]
fn a_tux_guitar_2_track_on_a_channel_the_song_does_not_list_keeps_its_tuning() {
    let content = b"<TuxGuitarFile><TGSong><TGTrack><channelId>9</channelId>\
        <TGString>64</TGString><TGString>59</TGString><TGString>55</TGString>\
        <TGString>50</TGString><TGString>45</TGString><TGString>38</TGString>\
        </TGTrack></TGSong></TuxGuitarFile>";

    let header = read_header(&tux_guitar_2_file(content));

    assert_names_and_tunings(header, [None, None, None], &["6 Drop D"]);
}

#[test]
fn a_tux_guitar_2_name_with_an_unknown_entity_is_unreadable() {
    let content = TuxGuitar2Xml::default()
        .titled("Wexmoor &nbsp; Tide")
        .with_strings(&DROP_D);

    assert_eq!(
        read_header(&tux_guitar_2_file(&content.xml())),
        Err(Unreadable)
    );
}

#[test]
fn a_tux_guitar_2_empty_string_entry_is_skipped() {
    let strings = ["64", "", "59", "55", "50", "45", "38"].map(str::to_owned);
    let content = TuxGuitar2Xml::default().with_strings_written(&strings, 0);

    let header = read_header(&tux_guitar_2_file(&content.xml()));

    assert_names_and_tunings(header, [None, None, Some("Saltglass")], &["6 Drop D"]);
}

#[test]
fn a_tux_guitar_2_name_written_as_cdata_is_read() {
    let content = TuxGuitar2Xml::default()
        .titled("<![CDATA[Wexmoor & Tide]]>")
        .with_strings(&DROP_D);

    let header = read_header(&tux_guitar_2_file(&content.xml()));

    assert_names_and_tunings(
        header,
        [Some("Wexmoor & Tide"), None, Some("Saltglass")],
        &["6 Drop D"],
    );
}

#[test]
fn a_zip_holding_some_other_content_xml_before_its_tab_file_reads_the_tab_file() {
    let tab_file = PowerTabFile::version(4).titled("Wexmoor").bytes();
    let zip = zip_holding(&[
        ("content.xml", b"<Export><Notes>not a tab</Notes></Export>"),
        ("Invented Song.ptb", &tab_file),
    ]);

    let header = read_header(&zip);

    assert_names_without_tunings(header, [Some("Wexmoor"), None, None]);
}

#[test]
fn a_tux_guitar_2_string_of_words_is_unreadable() {
    let content = TuxGuitar2Xml::default().with_strings_written(&["low E".to_owned()], 0);

    assert_eq!(
        read_header(&tux_guitar_2_file(&content.xml())),
        Err(Unreadable)
    );
}

#[test]
fn a_tux_guitar_2_track_of_more_than_16_strings_is_unreadable() {
    let content = TuxGuitar2Xml::default().with_strings(&[40; 17]);

    assert_eq!(
        read_header(&tux_guitar_2_file(&content.xml())),
        Err(Unreadable)
    );
}

#[test]
fn a_tux_guitar_2_file_of_more_than_128_tracks_is_unreadable() {
    let mut content = TuxGuitar2Xml::default();
    for _ in 0..129 {
        content = content.with_strings(&DROP_D);
    }

    assert_eq!(
        read_header(&tux_guitar_2_file(&content.xml())),
        Err(Unreadable)
    );
}

#[test]
fn a_tux_guitar_2_file_ending_inside_its_song_is_unreadable() {
    let tab_file = tux_guitar_2_file(b"<TuxGuitarFile><TGSong><name>Wexmoor</name>");

    assert_eq!(read_header(&tab_file), Err(Unreadable));
}

#[test]
fn a_tux_guitar_2_file_of_broken_xml_is_unreadable() {
    let tab_file = tux_guitar_2_file(b"<TuxGuitarFile><TGSong></Wrong>");

    assert_eq!(read_header(&tab_file), Err(Unreadable));
}

#[track_caller]
fn assert_names_and_tunings(
    header: Result<ScoreHeader, Unreadable>,
    names: [Option<&str>; 3],
    tunings: &[&str],
) {
    let read = header.map(|header| {
        (
            [header.title, header.artist, header.album],
            tunings_in(header.tracks),
        )
    });
    let expected_tunings = tunings.iter().map(|&tuning| tuning.to_owned()).collect();

    assert_eq!(
        read,
        Ok((names.map(|name| name.map(str::to_owned)), expected_tunings))
    );
}

fn tunings_in(tracks: Vec<Track>) -> Vec<String> {
    let mut tunings = Vec::new();
    for track in tracks {
        match track.tuning {
            Some(tuning) => tunings.push(format!("{} {}", tuning.strings(), tuning.name())),
            None => tunings.push("no Tuning".to_owned()),
        }
    }

    tunings
}

#[track_caller]
fn assert_names_without_tunings(
    header: Result<ScoreHeader, Unreadable>,
    expected: [Option<&str>; 3],
) {
    let names = header.map(|header| ([header.title, header.artist, header.album], header.tracks));

    assert_eq!(
        names,
        Ok((expected.map(|name| name.map(str::to_owned)), vec![]))
    );
}
