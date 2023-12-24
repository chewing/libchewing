use std::cmp::min;

use crate::{
    conversion::{Break, Composition, Interval, Symbol},
    dictionary::{Dictionary, Phrases},
};

#[derive(Debug)]
pub(crate) struct PhraseSelector {
    begin: usize,
    end: usize,
    forward_select: bool,
    buffer: Vec<Symbol>,
    selections: Vec<Interval>,
    breaks: Vec<Break>,
}

impl PhraseSelector {
    pub(crate) fn new(forward_select: bool, com: Composition) -> PhraseSelector {
        PhraseSelector {
            begin: 0,
            end: com.buffer.len(),
            forward_select,
            buffer: com.buffer,
            selections: com.selections,
            breaks: com.breaks,
        }
    }

    pub(crate) fn init<D: Dictionary>(&mut self, cursor: usize, dict: &D) {
        if self.forward_select {
            self.begin = cursor;
            self.end = self.next_break_point(cursor);
        } else {
            self.end = min(cursor + 1, self.buffer.len());
            self.begin = self.previous_break_point(cursor);
        }
        loop {
            let syllables = &self.buffer[self.begin..self.end];
            if dict.lookup_phrase(syllables).next().is_some() {
                break;
            }
            if self.forward_select {
                self.end -= 1;
            } else {
                self.begin += 1;
            }
        }
    }

    pub(crate) fn next<D: Dictionary>(&mut self, dict: &D) {
        loop {
            if self.forward_select {
                self.end -= 1;
                if self.begin == self.end {
                    self.end = self.next_break_point(self.begin);
                }
            } else {
                self.begin += 1;
                if self.begin == self.end {
                    self.begin = self.previous_break_point(self.begin);
                }
            }
            let syllables = &self.buffer[self.begin..self.end];
            if dict.lookup_phrase(syllables).next().is_some() {
                break;
            }
        }
    }

    fn next_break_point(&self, mut cursor: usize) -> usize {
        loop {
            if self.buffer.len() == cursor {
                break;
            }
            if !self.buffer[cursor].is_syllable() {
                break;
            }
            cursor += 1;
        }
        cursor
    }

    fn previous_break_point(&self, mut cursor: usize) -> usize {
        let selection_ends: Vec<_> = self.selections.iter().map(|sel| sel.end).collect();
        loop {
            if cursor == 0 {
                return 0;
            }
            if self.buffer.len() == cursor {
                cursor -= 1;
            }
            if selection_ends.binary_search(&cursor).is_ok() {
                break;
            }
            if self.breaks.binary_search(&Break(cursor)).is_ok() {
                break;
            }
            cursor -= 1;
            if !self.buffer[cursor].is_syllable() {
                cursor += 1;
                break;
            }
        }
        cursor
    }

    pub(crate) fn candidates<D: Dictionary>(&self, dict: &D) -> Vec<String> {
        dict.lookup_phrase(&self.buffer[self.begin..self.end])
            .map(|phrase| phrase.into())
            .collect()
    }

    pub(crate) fn interval(&self, phrase: String) -> Interval {
        Interval {
            start: self.begin,
            end: self.end,
            phrase,
        }
    }
}
