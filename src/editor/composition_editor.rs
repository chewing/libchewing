//! TODO: doc

use std::cmp::min;

use crate::conversion::{Composition, Interval, Symbol};

/// TODO
#[derive(Debug, Default, Clone)]
pub struct CompositionEditor {
    /// TODO
    cursor: usize,
    /// TODO
    pub(crate) inner: Composition,
}

impl CompositionEditor {
    pub(crate) fn cursor(&self) -> usize {
        self.cursor
    }
    /// Get the current symbol under the cursor
    ///
    /// The cursor always indicates the gap between symbols.
    ///
    /// ```text
    /// |S|S|S|S|S|S|
    /// ^           ^
    /// |           `--- cursor 6 and end of buffer
    /// `--- cursor 0
    /// ```
    ///
    /// When returning the symbol under the cursor, we always
    /// return the symbol that has the same index with the cursor.
    /// So cursor 0 will return `Some(S)` and cursor 6 will return
    /// `None`.
    ///
    /// When inserting a new symbol, it is always inserted to the
    /// gap indicated by the cursor. When cursor is at the end of the
    /// buffer, new symbols are appended.
    pub(crate) fn symbol(&self) -> Option<&Symbol> {
        if self.cursor == self.inner.buffer.len() {
            return None;
        }
        Some(&self.inner.buffer[self.cursor])
    }
    pub(crate) fn is_empty(&self) -> bool {
        self.inner.buffer.is_empty()
    }
    pub(crate) fn remove_after_cursor(&mut self) {
        if self.cursor == self.inner.buffer.len() {
            return;
        }
        self.inner.buffer.splice(self.cursor..self.cursor + 1, []);
    }
    pub(crate) fn remove_before_cursor(&mut self) {
        if self.cursor == 0 {
            return;
        }
        self.inner.buffer.splice(self.cursor - 1..self.cursor, []);
    }
    pub(crate) fn move_cursor_to_end(&mut self) {
        self.cursor = self.inner.buffer.len();
    }
    pub(crate) fn move_cursor_to_beginning(&mut self) {
        self.cursor = 0;
    }
    pub(crate) fn move_cursor_left(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
    }
    pub(crate) fn move_cursor_right(&mut self) {
        self.cursor = min(self.cursor + 1, self.inner.buffer.len());
    }
    pub(crate) fn push(&mut self, symbol: Symbol) {
        self.inner.buffer.push(symbol);
        self.cursor += 1;
    }
    pub(crate) fn insert(&mut self, s: Symbol) {
        self.inner.buffer.insert(self.cursor, s);
        // FIXME shift selections and breaks
    }
    pub(crate) fn symbol_for_select(&self) -> Option<Symbol> {
        let cursor = if self.cursor == self.inner.buffer.len() {
            self.cursor.saturating_sub(1)
        } else {
            self.cursor
        };
        if cursor == self.inner.buffer.len() {
            None
        } else {
            Some(self.inner.buffer[cursor])
        }
    }
    pub(crate) fn select(&mut self, interval: Interval) {
        self.inner.selections.push(interval);
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
