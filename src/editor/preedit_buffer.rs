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
pub(crate) enum Char {
    Chinese(Syllable),
    Other(char),
}

pub(crate) enum Segment {
    Chinese(Vec<Syllable>),
    Other(Vec<char>),
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
    pub(crate) fn push(&mut self, char_: Char) {
        self.buffer.push(char_);
        self.cursor += 1;
    }
    pub(crate) fn segments(&self) -> Vec<Segment> {
        let mut chinese_segment = vec![];
        let mut other_segment = vec![];
        let mut segments = vec![];
        for char_ in self.buffer.iter() {
            match char_ {
                Char::Chinese(syllable) => {
                    if !other_segment.is_empty() {
                        segments.push(Segment::Other(other_segment));
                        other_segment = vec![];
                    }
                    chinese_segment.push(*syllable);
                }
                Char::Other(char_) => {
                    if !chinese_segment.is_empty() {
                        segments.push(Segment::Chinese(chinese_segment));
                        chinese_segment = vec![];
                    }
                    other_segment.push(*char_);
                }
            }
        }
        if !other_segment.is_empty() {
            segments.push(Segment::Other(other_segment));
        }
        if !chinese_segment.is_empty() {
            segments.push(Segment::Chinese(chinese_segment));
        }
        segments
    }
}
