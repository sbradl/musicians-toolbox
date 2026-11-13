pub struct Fuzzed;

impl Saved for Fuzzed {
    fn keep(&self) {
        save_whole_or_nothing(&self.file, "");
    }
}

pub struct NotFuzzed;

impl NotFuzzed {
    fn keep(&self) {
        save_whole_or_nothing(&self.file, "");
    }
}

pub struct NeverSaved;

impl Saved for NeverSaved {}

fn save_whole_or_nothing(file: &Path, contents: &str) {}
