//! TODO: doc

use crate::zhuyin::Syllable;

/// TODO
#[derive(Debug, Default)]
pub struct PreeditBuffer {
    /// TODO
    cursor: usize,
    /// TODO
    buffer: Vec<Char>,
}

#[derive(Debug, Copy, Clone)]
enum Char {
    Chinese(Syllable),
    Other(char),
}

impl PreeditBuffer {
    pub(crate) fn is_empty(&self) -> bool {
        todo!()
    }
    pub(crate) fn remove_last(&mut self) {
        todo!()
    }
    pub(crate) fn remove_after_cursor(&mut self) {
        todo!()
    }
    pub(crate) fn remove_before_cursor(&mut self) {
        todo!()
    }
    pub(crate) fn move_cursor_to_end(&mut self) {
        todo!()
    }
    pub(crate) fn move_cursor_to_beginning(&mut self) {
        todo!()
    }
    pub(crate) fn insert(&mut self, syl: Syllable) {
        self.buffer.insert(self.cursor, Char::Chinese(syl));
    }
    pub(crate) fn syllables(&self) -> Vec<Syllable> {
        self.buffer
            .iter()
            .filter_map(|ch| match ch {
                Char::Chinese(syl) => Some(*syl),
                Char::Other(_) => None,
            })
            .collect()
    }
}
