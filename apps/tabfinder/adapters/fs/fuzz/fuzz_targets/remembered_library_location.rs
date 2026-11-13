#![no_main]

use libfuzzer_sys::fuzz_target;
use mt_tabfinder_application::RememberedLibraryLocation as _;
use mt_tabfinder_fs::FsRememberedLibraryLocation;
use mt_tabfinder_fs_fuzz::config_dir_with;

fuzz_target!(|bytes: &[u8]| {
    let config_dir = config_dir_with("library-location", bytes);

    drop(FsRememberedLibraryLocation::in_config_dir(config_dir).remembered());
});
