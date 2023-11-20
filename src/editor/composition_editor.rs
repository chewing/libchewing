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
    pub(crate) fn push(&mut self, symbol: Symbol) {
        self.inner.buffer.push(symbol);
        self.cursor += 1;
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
