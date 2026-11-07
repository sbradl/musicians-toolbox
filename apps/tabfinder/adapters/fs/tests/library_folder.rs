use std::fs::{self, Permissions};
use std::io;
use std::os::unix::fs::PermissionsExt;

use mt_tabfinder_application::{LibraryFolder, LibraryFolderError, LibraryLocation};
use mt_tabfinder_fs::FsLibraryFolder;

pub mod common;

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
        let path = self.0.path().to_string_lossy();

        LibraryLocation::new(&path)
    }

    fn song_titles(&self) -> Vec<String> {
        let songs = FsLibraryFolder.songs(&self.location()).unwrap_or_default();
        let mut titles = Vec::new();
        for song in songs {
            titles.push(song.title);
        }
        titles.sort();

        titles
    }
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
        .with_file("notes.txt")?
        .with_file("gp5")?
        .with_file("h.gp5x")?;

    assert_eq!(folder.song_titles(), ["a", "b", "c", "d", "e", "f", "g"]);

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
