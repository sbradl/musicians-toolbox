use libfuzzer_sys as _;
use mt_tabfinder_application as _;
use mt_tabfinder_fs as _;
use std::env;
use std::fs;
use std::path::PathBuf;

#[must_use]
pub fn config_dir_with(file_name: &str, contents: &[u8]) -> PathBuf {
    let config_dir = config_dir();
    drop(fs::create_dir_all(&config_dir));
    drop(fs::write(config_dir.join(file_name), contents));

    config_dir
}

#[must_use]
pub fn config_dir() -> PathBuf {
    private_dir().join(format!("mt-tabfinder-fs-fuzz-{}", std::process::id()))
}

fn private_dir() -> PathBuf {
    let Some(runtime_dir) = env::var_os("XDG_RUNTIME_DIR") else {
        return PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target");
    };

    PathBuf::from(runtime_dir)
}
