use std::path::PathBuf;

use mt_tabfinder_application as _;
use mt_tabfinder_fs::desktop_config_dir;

fn env(vars: &[(&'static str, &'static str)]) -> impl Fn(&str) -> Option<String> {
    move |name| {
        vars.iter()
            .find(|(known, _)| *known == name)
            .map(|(_, value)| (*value).to_owned())
    }
}

#[test]
fn the_config_dir_is_tabfinder_under_xdg_config_home() {
    assert_eq!(
        desktop_config_dir(env(&[("XDG_CONFIG_HOME", "/xdg"), ("HOME", "/home/me")])),
        Some(PathBuf::from("/xdg/tabfinder"))
    );
}

#[test]
fn without_xdg_config_home_the_config_dir_is_under_home_dot_config() {
    assert_eq!(
        desktop_config_dir(env(&[("HOME", "/home/me")])),
        Some(PathBuf::from("/home/me/.config/tabfinder"))
    );
}

#[test]
fn a_relative_or_empty_xdg_config_home_is_ignored() {
    assert_eq!(
        [
            desktop_config_dir(env(&[("XDG_CONFIG_HOME", "xdg"), ("HOME", "/home/me")])),
            desktop_config_dir(env(&[("XDG_CONFIG_HOME", ""), ("HOME", "/home/me")])),
        ],
        [
            Some(PathBuf::from("/home/me/.config/tabfinder")),
            Some(PathBuf::from("/home/me/.config/tabfinder")),
        ]
    );
}

#[test]
fn without_any_home_there_is_no_config_dir() {
    assert_eq!(desktop_config_dir(env(&[])), None);
}
