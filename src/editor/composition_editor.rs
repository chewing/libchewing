//! TODO: doc

use crate::conversion::{Composition, Symbol};

/// TODO
#[derive(Debug, Default)]
pub struct CompositionEditor {
    /// TODO
    cursor: usize,
    /// TODO
    inner: Composition,
}

impl CompositionEditor {
    pub(crate) fn cursor(&self) -> usize {
        self.cursor
    }
    pub(crate) fn slice(&self, start: usize, end: usize) -> &[Symbol] {
        &self.inner.buffer[start..end]
    }
    pub(crate) fn is_empty(&self) -> bool {
        self.inner.buffer.is_empty()
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
    pub(crate) fn move_cursor_left(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
    }
    pub(crate) fn rewind_cursor_to_break_point(&mut self) {
        loop {
            if self.cursor == 0 {
                break;
            }
            // TODO check break point
            if self.inner.buffer[self.cursor].is_syllable() {
                self.cursor -= 1;
            }
        }
    }
    pub(crate) fn push(&mut self, symbol: Symbol) {
        self.inner.buffer.push(symbol);
        self.cursor += 1;
    }
    pub(crate) fn is_cursor_on_syllable(&self) -> bool {
        self.inner.buffer[self.cursor].is_syllable()
    }
}

impl AsRef<Composition> for CompositionEditor {
    fn as_ref(&self) -> &Composition {
        &self.inner
    }
}

impl AsRef<CompositionEditor> for CompositionEditor {
    fn as_ref(&self) -> &CompositionEditor {
        self
    }
}
