use mt_tab_library::Song;
use mt_tabfinder_application::{FolderFile, LibraryFolder, LibraryFolderGone, scan};

struct OneFileFolder;

impl LibraryFolder for OneFileFolder {
    fn files(&self, location: &str) -> Result<Vec<FolderFile>, LibraryFolderGone> {
        if location == "/music" {
            Ok(vec![FolderFile {
                id: "/music/Angie.gp4".to_owned(),
                file_name: "Angie.gp4".to_owned(),
            }])
        } else {
            Err(LibraryFolderGone)
        }
    }
}

#[test]
fn a_scan_turns_the_library_folder_files_into_songs() {
    assert_eq!(
        scan(&OneFileFolder, "/music").map(|library| library.songs().to_vec()),
        Ok(vec![Song {
            id: "/music/Angie.gp4".to_owned(),
            title: "Angie".to_owned(),
        }])
    );
}

#[test]
fn a_scan_of_a_folder_that_is_gone_reports_it() {
    assert_eq!(
        scan(&OneFileFolder, "/gone").map(|library| library.songs().len()),
        Err(LibraryFolderGone)
    );
}
