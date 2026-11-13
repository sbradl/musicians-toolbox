use crate::{Cursor, Unreadable};

impl<'file> Cursor<'file> {
    pub(super) const fn over(tab_file: &'file [u8]) -> Self {
        Self {
            tab_file,
            position: 0,
        }
    }

    pub(super) fn bytes(&mut self, count: usize) -> Result<&'file [u8], Unreadable> {
        let end = self.position.checked_add(count).ok_or(Unreadable)?;
        let Some(bytes) = self.tab_file.get(self.position..end) else {
            return Err(Unreadable);
        };
        self.position = end;

        Ok(bytes)
    }

    pub(super) fn skip(&mut self, count: usize) -> Result<(), Unreadable> {
        _ = self.bytes(count)?;

        Ok(())
    }

    pub(super) fn byte(&mut self) -> Result<u8, Unreadable> {
        let [byte] = self.bytes(1)? else {
            return Err(Unreadable);
        };

        Ok(*byte)
    }

    pub(super) fn number(&mut self) -> Result<i32, Unreadable> {
        let Ok(little_endian) = self.bytes(4)?.try_into() else {
            return Err(Unreadable);
        };

        Ok(i32::from_le_bytes(little_endian))
    }

    pub(super) fn count_up_to(&mut self, most: usize) -> Result<usize, Unreadable> {
        let Ok(count) = usize::try_from(self.number()?) else {
            return Err(Unreadable);
        };

        if count > most {
            return Err(Unreadable);
        }

        Ok(count)
    }

    pub(super) fn skip_counted_bytes(&mut self) -> Result<(), Unreadable> {
        let count = self.count_up_to(usize::MAX)?;

        self.skip(count)
    }
}
