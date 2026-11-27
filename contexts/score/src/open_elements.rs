use crate::{MOST_XML_DEPTH, MOST_XML_ELEMENTS, OpenElements, Unreadable};

impl OpenElements {
    pub(super) fn open(&mut self, name: String) -> Result<(), Unreadable> {
        self.seen += 1;
        self.names.push(name);

        if self.names.len() > MOST_XML_DEPTH || self.seen > MOST_XML_ELEMENTS {
            return Err(Unreadable);
        }

        Ok(())
    }

    pub(super) fn close(&mut self) {
        drop(self.names.pop());
    }

    pub(super) fn are(&self, path: &[&str]) -> bool {
        self.names == path
    }

    pub(super) fn are_inside(&self, path: &[&str]) -> bool {
        let open = &self.names;

        open.len() > path.len() && open[..path.len()] == *path
    }

    pub(super) fn end_with(&self, path: &[&str]) -> bool {
        let open = &self.names;

        open.len() >= path.len() && open[open.len() - path.len()..] == *path
    }
}
