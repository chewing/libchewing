//! TODO: doc

use std::cmp::min;

use crate::conversion::{Break, Composition, Glue, Interval, Symbol};

/// TODO
#[derive(Debug, Default, Clone)]
pub struct CompositionEditor {
    /// TODO
    cursor: usize,
    cursor_stack: Vec<usize>,
    /// TODO
    pub(crate) inner: Composition,
}

impl CompositionEditor {
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
    }
    pub(crate) fn clamp_cursor(&mut self) {
        if self.cursor == self.inner.buffer.len() {
            self.cursor = self.cursor.saturating_sub(1);
        }
    }
    pub(crate) fn move_cursor(&mut self, cursor: usize) {
        self.cursor = min(cursor, self.inner.buffer.len());
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
    pub(crate) fn is_end_of_buffer(&self) -> bool {
        self.inner.buffer.len() == self.cursor
    }
    pub(crate) fn clear(&mut self) {
        self.inner.buffer.clear();
        self.inner.selections.clear();
        self.inner.breaks.clear();
        self.cursor = 0;
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
        self.cursor += 1;
        // FIXME shift selections and breaks
    }
    pub(crate) fn insert_glue(&mut self) {
        let break_idx = self
            .inner
            .breaks
            .iter()
            .position(|&b| b == Break(self.cursor));
        let glue_idx = self
            .inner
            .glues
            .iter()
            .position(|&b| b == Glue(self.cursor));
        if glue_idx.is_none() {
            self.inner.glues.push(Glue(self.cursor));
        }
        if let Some(break_idx) = break_idx {
            self.inner.breaks.swap_remove(break_idx);
        }
    }
    pub(crate) fn insert_break(&mut self) {
        let break_idx = self
            .inner
            .breaks
            .iter()
            .position(|&b| b == Break(self.cursor));
        let glue_idx = self
            .inner
            .glues
            .iter()
            .position(|&b| b == Glue(self.cursor));
        if break_idx.is_none() {
            self.inner.breaks.push(Break(self.cursor));
        }
        if let Some(glue_idx) = glue_idx {
            self.inner.glues.swap_remove(glue_idx);
        }
    }
    pub(crate) fn replace(&mut self, s: Symbol) {
        self.inner.buffer[self.cursor] = s;
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
