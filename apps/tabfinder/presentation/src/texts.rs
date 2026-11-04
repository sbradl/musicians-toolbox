use fluent_bundle::{FluentResource, concurrent::FluentBundle};
use unic_langid::LanguageIdentifier;

pub struct Texts(FluentBundle<FluentResource>);

impl Texts {
    pub fn english() -> Self {
        let english: LanguageIdentifier = "en".parse().expect("en is a valid language identifier");
        let resource = FluentResource::try_new(include_str!("../locales/en.ftl").to_owned())
            .expect("en.ftl is valid Fluent");
        let mut bundle = FluentBundle::new_concurrent(vec![english]);
        bundle
            .add_resource(resource)
            .expect("en.ftl defines each message once");
        Self(bundle)
    }

    pub fn get(&self, id: &str) -> String {
        let pattern = self
            .0
            .get_message(id)
            .and_then(|message| message.value())
            .unwrap_or_else(|| panic!("en.ftl defines {id}"));
        let mut errors = vec![];
        let text = self.0.format_pattern(pattern, None, &mut errors);
        assert!(errors.is_empty(), "{id} formats without errors: {errors:?}");
        text.into_owned()
    }
}
