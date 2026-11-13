use std::ffi::OsStr;
use std::fs::{self, Permissions};
use std::io;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::PermissionsExt;

use mt_tabfinder_application::{LibraryFolder, LibraryFolderError, LibraryLocation, Song};
use mt_tabfinder_fs::FsLibraryFolder;

pub mod common;

const TAB_FILE_WITH_DROP_D_AND_BASS: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../../../contexts/score/tests/fixtures/drop-d-and-bass.gp5"
);

use common::TempDir;

struct TempFolder(TempDir);

impl TempFolder {
    fn new(name: &str) -> io::Result<Self> {
        let dir = TempDir::new(name);
        fs::create_dir_all(dir.path())?;

        Ok(Self(dir))
    }

    fn with_file(self, relative: &str) -> io::Result<Self> {
        let file = self.0.path().join(relative);

        if let Some(parent) = file.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&file, b"")?;

        Ok(self)
    }

    fn with_tab_file(self, relative: &str) -> io::Result<Self> {
        let file = self.0.path().join(relative);

        if let Some(parent) = file.parent() {
            fs::create_dir_all(parent)?;
        }
        _ = fs::copy(TAB_FILE_WITH_DROP_D_AND_BASS, &file)?;

        Ok(self)
    }

    fn with_file_in_subfolder_named(self, raw_name: &[u8], file: &str) -> io::Result<Self> {
        let subfolder = self.0.path().join(OsStr::from_bytes(raw_name));
        fs::create_dir_all(&subfolder)?;
        fs::write(subfolder.join(file), b"")?;

        Ok(self)
    }

    fn with_link(self, relative: &str, target: &Self) -> io::Result<Self> {
        std::os::unix::fs::symlink(target.0.path(), self.0.path().join(relative))?;

        Ok(self)
    }

    fn with_link_to_file(self, relative: &str, target: &Self, file: &str) -> io::Result<Self> {
        std::os::unix::fs::symlink(target.0.path().join(file), self.0.path().join(relative))?;

        Ok(self)
    }

    fn lock_away(self) -> io::Result<Self> {
        fs::set_permissions(self.0.path(), Permissions::from_mode(0o000))?;

        Ok(self)
    }

    fn unlock(&self) {
        drop(fs::set_permissions(
            self.0.path(),
            Permissions::from_mode(0o755),
        ));
    }

    fn delete(self) {
        drop(self);
    }

    fn location(&self) -> LibraryLocation {
        self.location_of_subfolder("")
    }

    fn location_of_subfolder(&self, encoded_name: &str) -> LibraryLocation {
        let path = self.0.path().display();

        LibraryLocation::new(&format!("file://{path}/{encoded_name}"))
    }

    fn song_titles(&self) -> Vec<String> {
        song_titles_at(&self.location())
    }

    fn song_titled(&self, title: &str) -> Option<Song> {
        let songs = FsLibraryFolder.songs(&self.location()).unwrap_or_default();

        songs.into_iter().find(|song| song.title == title)
    }

    fn song(&self, title: &str) -> Song {
        self.song_titled(title)
            .unwrap_or_else(|| Song::titled("no such Song"))
    }

    #[track_caller]
    fn assert_artist_and_album(&self, title: &str, artist: Option<&str>, album: Option<&str>) {
        let song = self.song(title);

        assert_eq!(
            (song.artist.as_deref(), song.album.as_deref()),
            (artist, album)
        );
    }

    #[track_caller]
    fn assert_tunings(&self, title: &str, expected: &[&str]) {
        let song = self.song(title);
        let mut tunings = Vec::new();
        for tuning in song.tunings {
            tunings.push(format!("{} {}", tuning.strings(), tuning.name()));
        }

        assert_eq!(tunings, expected);
    }

    #[track_caller]
    fn assert_unreadable(&self, title: &str, expected: bool) {
        let song = self.song(title);

        assert_eq!(song.is_unreadable, expected);
    }
}

fn song_titles_at(location: &LibraryLocation) -> Vec<String> {
    let songs = FsLibraryFolder.songs(location).unwrap_or_default();
    let mut titles = Vec::new();
    for song in songs {
        titles.push(song.title);
    }
    titles.sort();

    titles
}

#[test]
fn finds_songs_in_subfolders_but_not_in_hidden_folders() -> io::Result<()> {
    let folder = TempFolder::new("recursive")?
        .with_file("Amberline.gp4")?
        .with_file("Some Artist/1968/Brasswing.GP5")?
        .with_file(".backup/Amberline.gp4")?
        .with_file("Some Artist/.cache/Brasswing.gp5")?;

    assert_eq!(folder.song_titles(), ["Amberline", "Brasswing"]);

    Ok(())
}

#[test]
fn every_tab_extension_in_any_case_is_a_song_and_other_files_are_not() -> io::Result<()> {
    let folder = TempFolder::new("extensions")?
        .with_file("a.GP3")?
        .with_file("b.gp4")?
        .with_file("c.Gp5")?
        .with_file("d.gpx")?
        .with_file("e.GP")?
        .with_file("f.tg")?
        .with_file("g.PTB")?
        .with_file("h.gtp")?
        .with_file("i.zip")?
        .with_file("j.crdownload")?
        .with_file("notes.txt")?
        .with_file("gp5")?
        .with_file("k.gp5x")?;

    assert_eq!(
        folder.song_titles(),
        ["a", "b", "c", "d", "e", "f", "g", "h", "i", "j"]
    );

    Ok(())
}

#[test]
fn a_song_is_titled_by_its_file_name_without_the_extension() -> io::Result<()> {
    let folder = TempFolder::new("titles")?.with_file("Lone.Pine.gp5")?;

    assert_eq!(folder.song_titles(), ["Lone.Pine"]);

    Ok(())
}

#[test]
fn a_folder_that_is_gone_is_reported() -> io::Result<()> {
    let folder = TempFolder::new("gone")?;
    let location = folder.location();
    folder.delete();

    assert_eq!(
        FsLibraryFolder.songs(&location),
        Err(LibraryFolderError::Gone)
    );

    Ok(())
}

#[test]
fn hidden_files_are_no_songs() -> io::Result<()> {
    let folder = TempFolder::new("hidden-files")?
        .with_file("Amberline.gp4")?
        .with_file(".Brasswing.gp5")?;

    assert_eq!(folder.song_titles(), ["Amberline"]);

    Ok(())
}

#[test]
fn linked_folders_are_not_followed() -> io::Result<()> {
    let elsewhere = TempFolder::new("link-target")?.with_file("Brasswing.gp5")?;
    let folder = TempFolder::new("links")?
        .with_file("Amberline.gp4")?
        .with_link("Linked Artist", &elsewhere)?;

    assert_eq!(folder.song_titles(), ["Amberline"]);

    Ok(())
}

#[test]
fn a_folder_tabfinder_may_not_read_is_reported() -> io::Result<()> {
    let folder = TempFolder::new("locked")?.lock_away()?;

    let songs = FsLibraryFolder.songs(&folder.location());

    folder.unlock();
    assert_eq!(songs, Err(LibraryFolderError::NoAccess));

    Ok(())
}

#[test]
fn linked_files_are_no_songs() -> io::Result<()> {
    let elsewhere = TempFolder::new("file-link-target")?.with_file("Brasswing.gp5")?;
    let folder = TempFolder::new("file-links")?
        .with_file("Amberline.gp4")?
        .with_link_to_file("Brasswing.gp5", &elsewhere, "Brasswing.gp5")?;

    assert_eq!(folder.song_titles(), ["Amberline"]);

    Ok(())
}

#[test]
fn finds_songs_in_a_folder_whose_name_is_not_utf8() -> io::Result<()> {
    let folder =
        TempFolder::new("not-utf8")?.with_file_in_subfolder_named(b"Caf\xe9", "Amberline.gp4")?;

    let titles = song_titles_at(&folder.location_of_subfolder("Caf%E9"));

    assert_eq!(titles, ["Amberline"]);

    Ok(())
}

#[test]
fn the_first_folder_names_the_artist_and_the_second_the_album() -> io::Result<()> {
    let folder = TempFolder::new("artist-album")?
        .with_file("Corvid Lane/Saltglass/Wexmoor.gp5")?
        .with_file("Corvid Lane/Brasswing.gp5")?
        .with_file("Amberline.gp4")?
        .with_file("Corvid Lane/Saltglass/Disc 2/Lone Pine.gp5")?;

    folder.assert_artist_and_album("Wexmoor", Some("Corvid Lane"), Some("Saltglass"));
    folder.assert_artist_and_album("Brasswing", Some("Corvid Lane"), None);
    folder.assert_artist_and_album("Amberline", None, None);
    folder.assert_artist_and_album("Lone Pine", Some("Corvid Lane"), Some("Saltglass"));

    Ok(())
}

#[test]
fn a_songs_tunings_come_from_its_tab_file_once_each() -> io::Result<()> {
    let folder = TempFolder::new("tunings")?.with_tab_file("Corvid Lane/Wexmoor.gp5")?;

    folder.assert_tunings("Wexmoor", &["6 Drop D", "4 E Standard"]);
    folder.assert_unreadable("Wexmoor", false);

    Ok(())
}

#[test]
fn a_tab_file_is_read_by_its_contents_whatever_its_name() -> io::Result<()> {
    let folder = TempFolder::new("renamed")?
        .with_tab_file("Wexmoor.gp3")?
        .with_tab_file("Brasswing.crdownload")?;

    folder.assert_tunings("Wexmoor", &["6 Drop D", "4 E Standard"]);
    folder.assert_tunings("Brasswing", &["6 Drop D", "4 E Standard"]);

    Ok(())
}

#[test]
fn a_tab_file_that_cannot_be_read_is_an_unreadable_song() -> io::Result<()> {
    let folder = TempFolder::new("unreadable")?.with_file("Corvid Lane/Lone Pine.gp5")?;

    folder.assert_unreadable("Lone Pine", true);
    folder.assert_artist_and_album("Lone Pine", Some("Corvid Lane"), None);

    Ok(())
}
