use fluent_bundle::{FluentArgs, FluentResource, concurrent::FluentBundle};

use crate::Texts;
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

    pub(super) fn songs_count(&self, count: usize) -> String {
        let mut args = FluentArgs::new();
        args.set("count", count);

        self.format("songs-count", Some(&args))
    }

    pub(super) fn choose_folder(&self) -> String {
        self.format("choose-folder", None)
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
