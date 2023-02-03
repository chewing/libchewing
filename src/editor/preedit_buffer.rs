//! TODO: doc

/// TODO
#[derive(Debug)]
pub struct PreeditBuffer {
    /// TODO
    cursor: usize,
    /// TODO
    buffer: Vec<char>,
    /// TODO
    char_type: Vec<CharType>,
}

/// TODO
#[derive(Debug)]
enum CharType {
    Wide,
    Narrow,
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
    pub(crate) fn move_cursor_to_end(&mut self) {
        todo!()
    }
    pub(crate) fn move_cursor_to_beginning(&mut self) {
        todo!()
    }
}
