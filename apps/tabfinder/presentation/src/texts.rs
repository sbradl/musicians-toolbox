use fluent_bundle::{FluentArgs, FluentResource, concurrent::FluentBundle};

use crate::Texts;
use mt_tabfinder_application::{LibraryFolderError, Tuning};
use unic_langid::LanguageIdentifier;

impl Texts {
    pub(super) fn english() -> Self {
        let english: LanguageIdentifier = "en".parse().unwrap_or_default();
        let resource = FluentResource::try_new(include_str!("../locales/en.ftl").to_owned())
            .unwrap_or_else(|(partially_parsed, _)| partially_parsed);
        let mut bundle = FluentBundle::new_concurrent(vec![english]);
        bundle.set_use_isolating(false);
        bundle.add_resource_overriding(resource);

        Self(bundle)
    }

    pub(super) fn no_folder_chosen(&self) -> String {
        self.format("no-folder-chosen", None)
    }

    pub(super) fn scanning(&self) -> String {
        self.format("scanning", None)
    }

    pub(super) fn no_songs_found(&self) -> String {
        self.format("no-songs-found", None)
    }

    pub(super) fn library_folder_problem(&self, error: &LibraryFolderError) -> String {
        match error {
            LibraryFolderError::Gone => self.format("library-folder-gone", None),
            LibraryFolderError::NoAccess => self.format("library-folder-no-access", None),
        }
    }

    pub(super) fn folder_picker_unavailable(&self) -> String {
        self.format("folder-picker-unavailable", None)
    }

    pub(super) fn songs_count(&self, count: usize) -> String {
        let mut args = FluentArgs::new();
        args.set("count", count);

        self.format("songs-count", Some(&args))
    }

    pub(super) fn rescan_finished(&self, songs: usize, unreadable: usize) -> String {
        let mut args = FluentArgs::new();
        args.set("songs", songs);
        args.set("unreadable", unreadable);

        self.format("rescan-finished", Some(&args))
    }

    pub(super) fn songs_matching(&self, matching: usize, all: usize) -> String {
        let mut args = FluentArgs::new();
        args.set("matching", matching);
        args.set("all", all);

        self.format("songs-matching", Some(&args))
    }

    pub(super) fn artist_and_album(&self, artist: &str, album: &str) -> String {
        let mut args = FluentArgs::new();
        args.set("artist", artist);
        args.set("album", album);

        self.format("artist-and-album", Some(&args))
    }

    pub(super) fn tuning_badge(&self, tuning: &Tuning) -> String {
        let mut args = FluentArgs::new();
        args.set("strings", tuning.strings());
        args.set("tuning", tuning.name().to_string());

        self.format("tuning-badge", Some(&args))
    }

    pub(super) fn unreadable_file(&self) -> String {
        self.format("unreadable-file", None)
    }

    pub(super) fn artist_filter(&self) -> String {
        self.format("artist-filter", None)
    }

    pub(super) fn clear_artist_filter(&self) -> String {
        self.format("clear-artist-filter", None)
    }

    pub(super) fn song_filter(&self) -> String {
        self.format("song-filter", None)
    }

    pub(super) fn clear_song_filter(&self) -> String {
        self.format("clear-song-filter", None)
    }

    pub(super) fn no_songs_match(&self) -> String {
        self.format("no-songs-match", None)
    }

    pub(super) fn clear_filters(&self) -> String {
        self.format("clear-filters", None)
    }

    pub(super) fn choose_folder(&self) -> String {
        self.format("choose-folder", None)
    }

    pub(super) fn rescan(&self) -> String {
        self.format("rescan", None)
    }

    pub(super) fn library_folder_dialog_title(&self) -> String {
        self.format("library-folder-dialog-title", None)
    }

    fn format(&self, id: &str, args: Option<&FluentArgs<'_>>) -> String {
        let Some(pattern) = self.0.get_message(id).and_then(|message| message.value()) else {
            return id.to_owned();
        };
        let mut errors = vec![];

        self.0
            .format_pattern(pattern, args, &mut errors)
            .into_owned()
    }
}
