pub mod support;

use mt_music_kernel as _;
use quick_xml as _;
use std::fs;
use std::time::{Duration, Instant};

use mt_score::{LARGEST_SCORE_XML, LARGEST_TAB_FILE, ScoreHeader, Unreadable, read_header};
use support::{
    B_STANDARD_7, BASS_E_STANDARD, DROP_D, E_STANDARD, FileSystem, GuitarProFile, OPEN_G, ScoreXml,
    compressed_guitar_pro_6_file, guitar_pro_6_file, guitar_pro_6_file_of_compressed_bytes,
    guitar_pro_6_file_packing, guitar_pro_6_file_stating_length, guitar_pro_7_file,
    uncompressed_guitar_pro_6_file, with_local_header_broken, zip_holding, zip_shrinking,
    zip_storing,
};

const FIXTURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures");

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
fn a_guitar_pro_5_file_gives_its_title_artist_and_album() {
    let tab_file = GuitarProFile::version(5, 10)
        .titled("Wexmoor")
        .by("Corvid Lane")
        .on_album("Saltglass")
        .with_strings(&E_STANDARD)
        .bytes();

    let header = read_header(&tab_file);

    assert_names(
        header,
        [Some("Wexmoor"), Some("Corvid Lane"), Some("Saltglass")],
    );
}

#[test]
fn a_guitar_pro_3_file_gives_its_title_artist_and_album() {
    let tab_file = GuitarProFile::version(3, 0)
        .titled("Wexmoor")
        .by("Corvid Lane")
        .on_album("Saltglass")
        .with_strings(&E_STANDARD)
        .bytes();

    let header = read_header(&tab_file);

    assert_names(
        header,
        [Some("Wexmoor"), Some("Corvid Lane"), Some("Saltglass")],
    );
}

#[test]
fn an_empty_or_blank_name_in_the_file_is_not_stated() {
    let tab_file = GuitarProFile::version(4, 6)
        .by(" \t ")
        .with_strings(&E_STANDARD)
        .bytes();

    let header = read_header(&tab_file);

    assert_names(header, [None, None, None]);
}

#[test]
fn a_guitar_pro_7_file_gives_its_tracks_tunings() {
    let score = ScoreXml::guitar_pro(7)
        .with_strings(&E_STANDARD)
        .with_strings(&BASS_E_STANDARD)
        .with_drums()
        .with_strings(&B_STANDARD_7);
    let tab_file = guitar_pro_7_file(&score.xml());

    let header = read_header(&tab_file);

    assert_tunings(
        header,
        &["6 E Standard", "4 E Standard", "no Tuning", "7 B Standard"],
    );
}

#[test]
fn a_guitar_pro_6_file_gives_its_tracks_tunings() {
    let score = ScoreXml::guitar_pro(6)
        .with_strings(&B_STANDARD_7)
        .with_drums()
        .with_strings(&BASS_E_STANDARD);
    let tab_file = guitar_pro_6_file(&score.xml());

    let header = read_header(&tab_file);

    assert_tunings(header, &["7 B Standard", "no Tuning", "4 E Standard"]);
}

#[test]
fn an_uncompressed_guitar_pro_6_file_gives_its_tracks_tunings() {
    let score = ScoreXml::guitar_pro(6).with_strings(&DROP_D);
    let tab_file = uncompressed_guitar_pro_6_file(&score.xml());

    let header = read_header(&tab_file);

    assert_tunings(header, &["6 Drop D"]);
}

#[test]
fn a_guitar_pro_6_file_ending_before_its_stated_length_is_read() {
    let xml = ScoreXml::guitar_pro(6).with_strings(&DROP_D).xml();
    let one_byte_more = uncompressed_guitar_pro_6_file(&xml).len() + 1;
    let tab_file = guitar_pro_6_file_stating_length(&xml, one_byte_more);

    let header = read_header(&tab_file);

    assert_tunings(header, &["6 Drop D"]);
}

#[test]
fn what_follows_the_tracks_is_not_read() {
    let score = ScoreXml::guitar_pro(7)
        .with_strings(&DROP_D)
        .followed_by("<Bars><Bar>no end");
    let tab_file = guitar_pro_7_file(&score.xml());

    let header = read_header(&tab_file);

    assert_tunings(header, &["6 Drop D"]);
}

#[test]
fn a_guitar_pro_7_file_whose_tracks_never_end_is_unreadable() {
    let mut xml = ScoreXml::guitar_pro(7).with_strings(&DROP_D).xml();
    xml.truncate(xml.len() / 2);
    let tab_file = guitar_pro_7_file(&xml);

    let header = read_header(&tab_file);

    assert_eq!(header, Err(Unreadable));
}

#[test]
fn a_guitar_pro_7_file_unpacking_too_much_before_its_tracks_is_unreadable() {
    let padding = " ".repeat(LARGEST_SCORE_XML);
    let score = ScoreXml::guitar_pro(7)
        .preceded_by(&padding)
        .with_strings(&DROP_D);
    let tab_file = guitar_pro_7_file(&score.xml());

    let header = read_header(&tab_file);

    assert_eq!(header, Err(Unreadable));
}

#[test]
fn a_guitar_pro_6_file_stating_more_than_the_largest_score_xml_is_unreadable() {
    let xml = ScoreXml::guitar_pro(6).with_strings(&DROP_D).xml();
    let tab_file = guitar_pro_6_file_stating_length(&xml, LARGEST_SCORE_XML + 1);

    let header = read_header(&tab_file);

    assert_eq!(header, Err(Unreadable));
}

#[test]
fn a_guitar_pro_6_score_xml_stating_more_than_the_largest_score_xml_is_unreadable() {
    let xml = ScoreXml::guitar_pro(6).with_strings(&DROP_D).xml();
    let file_system = FileSystem::default()
        .with_file_stating_size("score.gpif", LARGEST_SCORE_XML + 1, &xml)
        .bytes();
    let tab_file = compressed_guitar_pro_6_file(&file_system);

    let header = read_header(&tab_file);

    assert_eq!(header, Err(Unreadable));
}

#[test]
fn a_score_xml_nested_too_deep_is_unreadable() {
    let deep = format!("{}{}", "<Group>".repeat(100), "</Group>".repeat(100));
    let score = ScoreXml::guitar_pro(7)
        .preceded_by(&deep)
        .with_strings(&DROP_D);
    let tab_file = guitar_pro_7_file(&score.xml());

    let header = read_header(&tab_file);

    assert_eq!(header, Err(Unreadable));
}

#[test]
fn a_score_xml_with_too_many_elements_is_unreadable() {
    let many = "<Note/>".repeat(1_000_001);
    let score = ScoreXml::guitar_pro(7)
        .preceded_by(&many)
        .with_strings(&DROP_D);
    let tab_file = guitar_pro_7_file(&score.xml());

    let header = read_header(&tab_file);

    assert_eq!(header, Err(Unreadable));
}

#[test]
fn a_guitar_pro_6_file_of_nothing_but_zeros_is_unreadable_at_once() {
    let zero_bits = vec![0; LARGEST_TAB_FILE - 8];
    let tab_file = guitar_pro_6_file_of_compressed_bytes(LARGEST_SCORE_XML, &zero_bits);

    let header = read_header_timed(&tab_file);

    assert_unreadable_at_once(header);
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

fn read_header_timed(tab_file: &[u8]) -> (Result<ScoreHeader, Unreadable>, Duration) {
    let started = Instant::now();
    let header = read_header(tab_file);

    (header, started.elapsed())
}

#[track_caller]
fn assert_unreadable_at_once((header, took): (Result<ScoreHeader, Unreadable>, Duration)) {
    assert_eq!(header, Err(Unreadable));
    assert!(took < Duration::from_millis(100), "took {took:?}");
}

#[track_caller]
fn assert_tunings(header: Result<ScoreHeader, Unreadable>, expected: &[&str]) {
    assert_eq!(tunings_in(header), expected);
}

#[track_caller]
fn assert_names(header: Result<ScoreHeader, Unreadable>, expected: [Option<&str>; 3]) {
    let names = header.map(|header| [header.title, header.artist, header.album]);

    assert_eq!(names, Ok(expected.map(|name| name.map(str::to_owned))));
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
fn the_fixtures_for_the_apps_tests_are_what_the_builder_writes() {
    let drop_d_and_bass = GuitarProFile::version(5, 0)
        .by(" ")
        .with_strings(&DROP_D)
        .with_drums()
        .with_strings(&BASS_E_STANDARD)
        .with_strings(&DROP_D);
    let named = GuitarProFile::version(5, 0)
        .titled("Ashgrove Reel")
        .by("Marrow Bay")
        .on_album("Tidewater")
        .with_strings(&DROP_D);

    assert_fixture("drop-d-and-bass.gp5", &drop_d_and_bass);
    assert_fixture("named-drop-d.gp5", &named);
}

#[track_caller]
fn assert_fixture(name: &str, builder: &GuitarProFile) {
    let path = format!("{FIXTURES}/{name}");
    let built = builder.bytes();

    if std::env::var_os("WRITE_FIXTURES").is_some() {
        drop(fs::write(&path, &built));
    }
    let fixture = fs::read(&path).unwrap_or_default();

    assert!(
        fixture == built,
        "{path} differs from the builder; write it anew with: WRITE_FIXTURES=1 cargo test -p mt-score"
    );
}

#[test]
fn a_guitar_pro_file_whose_version_has_no_v_is_unreadable() {
    let tab_file = GuitarProFile::version(5, 0)
        .stating_version("FICHIER GUITAR PRO")
        .with_strings(&E_STANDARD)
        .bytes();

    assert_eq!(read_header(&tab_file), Err(Unreadable));
}

#[test]
fn a_guitar_pro_file_whose_version_has_no_minor_number_is_unreadable() {
    let tab_file = GuitarProFile::version(5, 0)
        .stating_version("FICHIER GUITAR PRO v5")
        .with_strings(&E_STANDARD)
        .bytes();

    assert_eq!(read_header(&tab_file), Err(Unreadable));
}

#[test]
fn a_guitar_pro_file_whose_version_is_no_number_is_unreadable() {
    let tab_file = GuitarProFile::version(5, 0)
        .stating_version("FICHIER GUITAR PRO vX.YZ")
        .with_strings(&E_STANDARD)
        .bytes();

    assert_eq!(read_header(&tab_file), Err(Unreadable));
}

#[test]
fn names_of_negative_size_are_not_stated_and_the_tunings_still_read() {
    let tab_file = GuitarProFile::version(5, 0)
        .with_name_fields_sized(-1)
        .with_strings(&DROP_D)
        .bytes();

    let names = read_header(&tab_file);
    let tunings = read_header(&tab_file);

    assert_names(names, [None, None, None]);
    assert_tunings(tunings, &["6 Drop D"]);
}

#[test]
fn names_of_size_zero_are_not_stated() {
    let tab_file = GuitarProFile::version(5, 0)
        .with_name_fields_sized(0)
        .with_strings(&DROP_D)
        .bytes();

    let header = read_header(&tab_file);

    assert_names(header, [None, None, None]);
}

#[test]
fn a_title_in_latin_1_is_read_with_its_accents() {
    let tab_file = GuitarProFile::version(5, 0)
        .titled_in_bytes(b"Caf\xe9 Wexmoor")
        .with_strings(&DROP_D)
        .bytes();

    let header = read_header(&tab_file);

    assert_names(header, [Some("Café Wexmoor"), None, None]);
}

#[test]
fn a_bar_changing_key_is_skipped() {
    let tab_file = GuitarProFile::version(5, 0)
        .changing_key()
        .with_strings(&DROP_D)
        .bytes();

    let header = read_header(&tab_file);

    assert_tunings(header, &["6 Drop D"]);
}

#[test]
fn a_track_without_strings_is_unreadable() {
    let tab_file = GuitarProFile::version(5, 0).with_strings(&[]).bytes();

    assert_eq!(read_header(&tab_file), Err(Unreadable));
}

#[test]
fn a_guitar_pro_file_stating_a_negative_number_of_bars_is_unreadable() {
    let tab_file = GuitarProFile::version(5, 0)
        .stating_bars(-1)
        .with_strings(&DROP_D)
        .bytes();

    assert_eq!(read_header(&tab_file), Err(Unreadable));
}

#[test]
fn a_guitar_pro_file_stating_more_than_10_000_bars_is_unreadable() {
    let tab_file = GuitarProFile::version(5, 0)
        .stating_bars(10_001)
        .with_strings(&DROP_D)
        .bytes();

    assert_eq!(read_header(&tab_file), Err(Unreadable));
}

#[test]
fn a_score_xml_tuning_of_negative_pitches_is_unreadable() {
    let score = ScoreXml::guitar_pro(7).with_pitches_written("-1 45 50 55 59 64");

    assert_eq!(
        read_header(&guitar_pro_7_file(&score.xml())),
        Err(Unreadable)
    );
}

#[test]
fn a_score_xml_tuning_above_the_highest_pitch_is_unreadable() {
    let score = ScoreXml::guitar_pro(7).with_pitches_written("38 45 50 55 59 128");

    assert_eq!(
        read_header(&guitar_pro_7_file(&score.xml())),
        Err(Unreadable)
    );
}

#[test]
fn a_score_xml_tuning_of_words_is_unreadable() {
    let score = ScoreXml::guitar_pro(7).with_pitches_written("low E");

    assert_eq!(
        read_header(&guitar_pro_7_file(&score.xml())),
        Err(Unreadable)
    );
}

#[test]
fn a_score_xml_tuning_of_blanks_is_unreadable() {
    let score = ScoreXml::guitar_pro(7).with_pitches_written("  ");

    assert_eq!(
        read_header(&guitar_pro_7_file(&score.xml())),
        Err(Unreadable)
    );
}

#[test]
fn a_score_xml_tuning_of_more_than_16_strings_is_unreadable() {
    let score = ScoreXml::guitar_pro(7).with_pitches_written(&"40 ".repeat(17));

    assert_eq!(
        read_header(&guitar_pro_7_file(&score.xml())),
        Err(Unreadable)
    );
}

#[test]
fn a_score_xml_property_without_a_name_is_skipped() {
    let score = ScoreXml::guitar_pro(7)
        .preceded_by("<Property><Fret>0</Fret></Property>")
        .with_strings(&DROP_D);

    let header = read_header(&guitar_pro_7_file(&score.xml()));

    assert_tunings(header, &["6 Drop D"]);
}

#[test]
fn a_score_xml_of_more_than_128_tracks_is_unreadable() {
    let mut score = ScoreXml::guitar_pro(7);
    for _ in 0..129 {
        score = score.with_strings(&DROP_D);
    }

    assert_eq!(
        read_header(&guitar_pro_7_file(&score.xml())),
        Err(Unreadable)
    );
}

#[test]
fn a_score_xml_ending_inside_its_tracks_is_unreadable() {
    let tab_file = guitar_pro_7_file(b"<GPIF><Tracks>");

    assert_eq!(read_header(&tab_file), Err(Unreadable));
}

#[test]
fn a_guitar_pro_7_file_storing_its_score_xml_uncompressed_is_read() {
    let score = ScoreXml::guitar_pro(7).with_strings(&DROP_D);
    let tab_file = zip_storing(&[("Content/score.gpif", &score.xml())]);

    assert_tunings(read_header(&tab_file), &["6 Drop D"]);
}

#[test]
fn a_guitar_pro_7_file_packing_its_score_xml_in_an_unknown_way_is_unreadable() {
    let score = ScoreXml::guitar_pro(7).with_strings(&DROP_D);
    let tab_file = zip_shrinking(&[("Content/score.gpif", &score.xml())]);

    assert_eq!(read_header(&tab_file), Err(Unreadable));
}

#[test]
fn a_guitar_pro_7_file_whose_score_xml_entry_is_broken_is_unreadable() {
    let score = ScoreXml::guitar_pro(7).with_strings(&DROP_D);
    let tab_file = with_local_header_broken(guitar_pro_7_file(&score.xml()), "Content/score.gpif");

    assert_eq!(read_header(&tab_file), Err(Unreadable));
}

#[test]
fn a_zip_storing_its_tab_file_uncompressed_is_read() {
    let tab_file = GuitarProFile::version(5, 0).with_strings(&DROP_D).bytes();
    let zip = zip_storing(&[("Invented Song.gp5", &tab_file)]);

    assert_tunings(read_header(&zip), &["6 Drop D"]);
}

#[test]
fn a_zip_packing_its_tab_file_in_an_unknown_way_is_unreadable() {
    let tab_file = GuitarProFile::version(5, 0).with_strings(&DROP_D).bytes();
    let zip = zip_shrinking(&[("Invented Song.gp5", &tab_file)]);

    assert_eq!(read_header(&zip), Err(Unreadable));
}

#[test]
fn a_zip_whose_tab_file_entry_is_broken_is_unreadable() {
    let tab_file = GuitarProFile::version(5, 0).with_strings(&DROP_D).bytes();
    let zip = with_local_header_broken(
        zip_holding(&[
            ("readme.txt", b"not a tab"),
            ("Invented Song.gp5", &tab_file),
        ]),
        "Invented Song.gp5",
    );

    assert_eq!(read_header(&zip), Err(Unreadable));
}

#[test]
fn a_broken_zip_is_unreadable() {
    assert_eq!(
        read_header(b"PK\x03\x04 and then nothing a zip needs"),
        Err(Unreadable)
    );
}

#[test]
fn a_zip_of_more_than_4096_entries_before_a_tab_file_is_unreadable() {
    let mut names = Vec::new();
    for number in 0..4097 {
        names.push(format!("note {number}.txt"));
    }
    let mut files: Vec<(&str, &[u8])> = Vec::new();
    for name in &names {
        files.push((name, b"not a tab"));
    }

    assert_eq!(read_header(&zip_holding(&files)), Err(Unreadable));
}

#[test]
fn a_guitar_pro_6_file_too_short_to_state_its_length_is_unreadable() {
    assert_eq!(read_header(b"BCFZ\x01\x02"), Err(Unreadable));
}

#[test]
fn a_guitar_pro_6_file_unpacking_to_no_file_system_is_unreadable() {
    let tab_file = guitar_pro_6_file_packing(b"NOPE and more bytes");

    assert_eq!(read_header(&tab_file), Err(Unreadable));
}

#[test]
fn a_guitar_pro_6_file_without_a_score_xml_is_unreadable() {
    let file_system = FileSystem::default().with_file("misc.xml", b"<Misc/>");
    let tab_file = compressed_guitar_pro_6_file(&file_system.bytes());

    assert_eq!(read_header(&tab_file), Err(Unreadable));
}

#[test]
fn a_guitar_pro_6_file_system_ending_inside_an_entry_is_unreadable() {
    let file_system = [FileSystem::default().bytes(), vec![2, 0]].concat();
    let tab_file = compressed_guitar_pro_6_file(&file_system);

    assert_eq!(read_header(&tab_file), Err(Unreadable));
}

#[test]
fn a_guitar_pro_6_score_xml_ending_inside_its_tracks_is_unreadable() {
    let tab_file = guitar_pro_6_file(b"<GPIF><Tracks>");

    assert_eq!(read_header(&tab_file), Err(Unreadable));
}

#[test]
fn a_guitar_pro_6_score_xml_shorter_than_its_stated_size_is_unreadable() {
    let file_system =
        FileSystem::default().with_file_stating_size("score.gpif", 10_000, b"<GPIF><Tracks>");
    let tab_file = compressed_guitar_pro_6_file(&file_system.bytes());

    assert_eq!(read_header(&tab_file), Err(Unreadable));
}

#[test]
fn a_guitar_pro_6_score_xml_stored_past_the_end_is_unreadable() {
    let file_system = FileSystem::default().with_file_past_the_end("score.gpif", 100);
    let tab_file = compressed_guitar_pro_6_file(&file_system.bytes());

    assert_eq!(read_header(&tab_file), Err(Unreadable));
}

#[test]
fn a_guitar_pro_file_ending_inside_a_number_is_unreadable() {
    let mut tab_file = GuitarProFile::version(5, 0).with_strings(&DROP_D).bytes();
    let version_field = 1 + 30;
    tab_file.truncate(version_field + 2);

    assert_eq!(read_header(&tab_file), Err(Unreadable));
}
