//! TODO: doc

use std::cmp::min;

use crate::conversion::{Break, Composition, Glue, Interval, Symbol};

/// TODO
#[derive(Debug, Default, Clone)]
pub(crate) struct CompositionEditor {
    /// TODO
    cursor: usize,
    cursor_stack: Vec<usize>,
    /// TODO
    pub(crate) inner: Composition,
}

impl CompositionEditor {
    pub(crate) fn len(&self) -> usize {
        self.inner.buffer.len()
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
        self.cursor = min(self.cursor, self.inner.buffer.len());
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
    pub(crate) fn pop_front(&mut self, n: usize) {
        assert!(n < self.inner.buffer.len());
        let mut to_remove = vec![];
        for (i, selection) in self.inner.selections.iter().enumerate() {
            if selection.start < n {
                to_remove.push(i);
            }
        }
        for &i in &to_remove {
            self.inner.selections.swap_remove(i);
        }
        to_remove.clear();
        for (i, item) in self.inner.breaks.iter().enumerate() {
            if item.0 < n {
                to_remove.push(i);
            }
        }
        for &i in &to_remove {
            self.inner.breaks.swap_remove(i);
        }
        to_remove.clear();
        for (i, item) in self.inner.glues.iter().enumerate() {
            if item.0 < n {
                to_remove.push(i);
            }
        }
        for &i in &to_remove {
            self.inner.glues.swap_remove(i);
        }
        self.inner.buffer.splice(0..n, []);
        self.cursor = self.cursor.saturating_sub(n);
    }
    pub(crate) fn remove_after_cursor(&mut self) {
        if self.cursor == self.inner.buffer.len() {
            return;
        }
        self.remove_at(self.cursor);
        self.inner.buffer.remove(self.cursor);
    }
    pub(crate) fn remove_before_cursor(&mut self) {
        if self.cursor == 0 {
            return;
        }
        self.remove_at(self.cursor - 1);
        self.inner.buffer.remove(self.cursor - 1);
        self.cursor -= 1;
    }
    fn remove_at(&mut self, cursor: usize) {
        let mut to_remove = vec![];
        for (i, selection) in self.inner.selections.iter_mut().enumerate() {
            if selection.start < cursor && selection.end > cursor {
                to_remove.push(i);
            }
            if selection.start >= cursor {
                selection.start -= 1;
                selection.end -= 1;
            }
        }
        for &i in &to_remove {
            self.inner.selections.swap_remove(i);
        }
        for (i, item) in self.inner.breaks.iter().enumerate() {
            if item.0 >= cursor {
                to_remove.push(i);
            }
        }
        for &i in &to_remove {
            self.inner.breaks.swap_remove(i);
        }

        for (i, item) in self.inner.glues.iter().enumerate() {
            if item.0 >= cursor {
                to_remove.push(i);
            }
        }
        for &i in &to_remove {
            self.inner.glues.swap_remove(i);
        }
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
    pub(crate) fn insert(&mut self, s: Symbol) {
        let mut to_remove = vec![];
        for (i, selection) in self.inner.selections.iter_mut().enumerate() {
            if selection.start < self.cursor && selection.end > self.cursor {
                to_remove.push(i);
            }
            if selection.start >= self.cursor {
                selection.start += 1;
                selection.end += 1;
            }
        }
        for i in to_remove {
            self.inner.selections.swap_remove(i);
        }
        for item in self.inner.breaks.iter_mut() {
            if item.0 >= self.cursor {
                item.0 += 1;
            }
        }
        for item in self.inner.glues.iter_mut() {
            if item.0 >= self.cursor {
                item.0 += 1;
            }
        }
        self.inner.buffer.insert(self.cursor, s);
        self.cursor += 1;
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
        debug_assert!(!interval.phrase.is_empty());
        let mut to_remove = vec![];
        for (i, selection) in self.inner.selections.iter().enumerate() {
            if selection.intersect(&interval) {
                to_remove.push(i);
            }
        }
        for i in to_remove {
            self.inner.selections.swap_remove(i);
        }
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
