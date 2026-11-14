use mt_music_kernel::Tuning;

fn name_of(lowest_string_first: &[u8]) -> String {
    Tuning::of_midi_numbers(lowest_string_first)
        .name()
        .to_string()
}

#[test]
fn a_five_string_bass_in_fourths_is_a_standard_tuning() {
    assert_eq!(name_of(&[23, 28, 33, 38, 43]), "B Standard");
}

#[test]
fn eight_strings_have_no_standard_to_compare_with_and_are_custom() {
    assert_eq!(name_of(&[30, 35, 40, 45, 50, 55, 59, 64]), "Custom");
}

#[test]
fn a_tuning_without_strings_is_custom() {
    assert_eq!(name_of(&[]), "Custom");
}
