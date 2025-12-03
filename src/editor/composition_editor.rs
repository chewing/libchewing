//! TODO: doc

use std::cmp::min;

use tracing::warn;

use crate::conversion::{Composition, Gap, Interval, Symbol};

/// TODO
#[derive(Debug, Default, Clone)]
pub(crate) struct CompositionEditor {
    /// TODO
    cursor: usize,
    cursor_stack: Vec<usize>,
    /// TODO
    inner: Composition,
}

impl CompositionEditor {
    pub(crate) fn to_composition(&self) -> Composition {
        self.inner.clone()
    }
    pub(crate) fn len(&self) -> usize {
        self.inner.len()
    }
    pub(crate) fn cursor(&self) -> usize {
        self.cursor
    }
    pub(crate) fn push_cursor(&mut self) {
        self.cursor_stack.push(self.cursor)
    }
    pub(crate) fn pop_cursor(&mut self) {
        if let Some(cursor) = self.cursor_stack.pop() {
            self.cursor = cursor;
        }
        self.cursor = min(self.cursor, self.inner.len());
    }
    pub(crate) fn clamp_cursor(&mut self) {
        if self.cursor == self.inner.len() {
            self.cursor = self.cursor.saturating_sub(1);
        }
    }
    pub(crate) fn move_cursor(&mut self, cursor: usize) {
        self.cursor = min(cursor, self.inner.len());
    }
    /// Get the symbol under the cursor
    ///
    /// The cursor always indicates the gap between symbols.
    ///
    /// ```text
    /// |S0|S1|S2|S3|S4|S5|
    /// ^                 ^
    /// |                 `--- cursor 6 and end of buffer
    /// `--- cursor 0
    /// ```
    ///
    /// When returning the symbol under the cursor, we always
    /// return the symbol that has the same index with the cursor.
    /// So cursor 0 will return `Some(S0)` and cursor 6 will return
    /// `None`.
    ///
    /// When inserting a new symbol, it is always inserted to the
    /// gap indicated by the cursor. When cursor is at the end of the
    /// buffer, new symbols are appended.
    pub(crate) fn symbol(&self) -> Option<Symbol> {
        self.inner.symbol(self.cursor)
    }
    pub(crate) fn symbols(&self) -> &[Symbol] {
        self.inner.symbols()
    }
    pub(crate) fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
    pub(crate) fn is_beginning_of_buffer(&self) -> bool {
        0 == self.cursor
    }
    pub(crate) fn is_end_of_buffer(&self) -> bool {
        self.inner.len() == self.cursor
    }
    pub(crate) fn clear(&mut self) {
        self.inner.clear();
        self.cursor = 0;
    }
    pub(crate) fn remove_front(&mut self, n: usize) {
        self.inner.remove_front(n);
        self.cursor = self.cursor.saturating_sub(n);
    }
    pub(crate) fn remove_after_cursor(&mut self) {
        self.inner.remove(self.cursor);
    }
    pub(crate) fn remove_before_cursor(&mut self) {
        if self.cursor == 0 {
            return;
        }
        self.cursor -= 1;
        self.inner.remove(self.cursor);
    }
    pub(crate) fn move_cursor_to_end(&mut self) {
        self.cursor = self.inner.len();
    }
    pub(crate) fn move_cursor_to_beginning(&mut self) {
        self.cursor = 0;
    }
    pub(crate) fn move_cursor_left(&mut self, n: usize) {
        self.cursor = self.cursor.saturating_sub(n);
    }
    pub(crate) fn move_cursor_right(&mut self, n: usize) {
        self.cursor = self.cursor.saturating_add(n).min(self.inner.len());
    }
    pub(crate) fn insert(&mut self, sym: Symbol) {
        self.inner.insert(self.cursor, sym);
        self.cursor += 1;
    }
    pub(crate) fn insert_glue(&mut self) {
        if self.is_end_of_buffer() {
            warn!("cannot set glue at the end of buffer");
            return;
        }
        self.inner.set_gap(self.cursor, Gap::Glue);
    }
    pub(crate) fn insert_break(&mut self) {
        if self.is_end_of_buffer() {
            warn!("cannot set break at the end of buffer");
            return;
        }
        self.inner.set_gap(self.cursor, Gap::Break);
    }
    pub(crate) fn replace(&mut self, sym: Symbol) {
        self.inner.replace(self.cursor, sym);
    }
    pub(crate) fn symbol_for_select(&self) -> Option<Symbol> {
        let cursor = if self.is_end_of_buffer() {
            self.cursor.saturating_sub(1)
        } else {
            self.cursor
        };
        self.inner.symbol(cursor)
    }
    pub(crate) fn select(&mut self, interval: Interval) {
        assert!(!interval.text.is_empty());
        self.inner.push_selection(interval);
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
