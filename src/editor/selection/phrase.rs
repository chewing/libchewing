use std::cmp::min;

use crate::{
    conversion::{Composition, Gap, Interval},
    dictionary::{Dictionary, Layered, LookupStrategy},
    editor::{EditorError, SharedState},
};

#[derive(Debug)]
pub(crate) struct PhraseSelector {
    begin: usize,
    end: usize,
    forward_select: bool,
    orig: usize,
    lookup_strategy: LookupStrategy,
    com: Composition,
}

impl PhraseSelector {
    pub(crate) fn new(
        forward_select: bool,
        lookup_strategy: LookupStrategy,
        com: Composition,
    ) -> PhraseSelector {
        PhraseSelector {
            begin: 0,
            end: com.len(),
            forward_select,
            orig: 0,
            lookup_strategy,
            com,
        }
    }

    pub(crate) fn init<D: Dictionary>(&mut self, cursor: usize, dict: &D) {
        self.orig = cursor;
        if self.forward_select {
            self.begin = if cursor == self.com.len() {
                cursor - 1
            } else {
                cursor
            };
            self.end = self.next_break_point(cursor);
        } else {
            self.end = min(cursor + 1, self.com.len());
            self.begin = self.after_previous_break_point(cursor);
        }
        loop {
            let syllables = &self.com.symbols()[self.begin..self.end];
            debug_assert!(
                !syllables.is_empty(),
                "should not enter here if there's no syllable in range"
            );
            if dict
                .lookup_first_phrase(&syllables, self.lookup_strategy)
                .is_some()
            {
                break;
            }
            if self.forward_select {
                self.end -= 1;
            } else {
                self.begin += 1;
            }
        }
    }

    pub(crate) fn init_single_word(&mut self, cursor: usize) {
        self.orig = cursor;
        self.end = min(cursor, self.com.len());
        self.begin = self.end - 1;
    }

    pub(crate) fn begin(&self) -> usize {
        self.begin
    }

    pub(crate) fn next_selection_point<D: Dictionary>(&self, dict: &D) -> Option<(usize, usize)> {
        let (mut begin, mut end) = (self.begin, self.end);
        loop {
            if self.forward_select {
                end -= 1;
                if begin == end {
                    return None;
                }
            } else {
                begin += 1;
                if begin == end {
                    return None;
                }
            }
            let syllables = &self.com.symbols()[begin..end];
            if dict
                .lookup_first_phrase(&syllables, self.lookup_strategy)
                .is_some()
            {
                return Some((begin, end));
            }
        }
    }
    pub(crate) fn prev_selection_point<D: Dictionary>(&self, dict: &D) -> Option<(usize, usize)> {
        let (mut begin, mut end) = (self.begin, self.end);
        loop {
            if self.forward_select {
                if end == self.com.len() {
                    return None;
                }
                end += 1;
                if end > self.next_break_point(self.orig) {
                    return None;
                }
            } else {
                if begin == 0 {
                    return None;
                }
                begin -= 1;
                if begin < self.after_previous_break_point(self.orig) {
                    return None;
                }
            }
            let syllables = &self.com.symbols()[begin..end];
            if dict
                .lookup_first_phrase(&syllables, self.lookup_strategy)
                .is_some()
            {
                return Some((begin, end));
            }
        }
    }
    pub(crate) fn jump_to_next_selection_point<D: Dictionary>(
        &mut self,
        dict: &D,
    ) -> Result<(), EditorError> {
        if let Some((begin, end)) = self.next_selection_point(dict) {
            self.begin = begin;
            self.end = end;
            Ok(())
        } else {
            Err(EditorError::Impossible)
        }
    }
    pub(crate) fn jump_to_prev_selection_point<D: Dictionary>(
        &mut self,
        dict: &D,
    ) -> Result<(), EditorError> {
        if let Some((begin, end)) = self.prev_selection_point(dict) {
            self.begin = begin;
            self.end = end;
            Ok(())
        } else {
            Err(EditorError::Impossible)
        }
    }
    pub(crate) fn jump_to_first_selection_point<D: Dictionary>(&mut self, dict: &D) {
        self.init(self.orig, dict);
    }
    pub(crate) fn jump_to_last_selection_point<D: Dictionary>(&mut self, dict: &D) {
        while self.next_selection_point(dict).is_some() {
            let _ = self.jump_to_next_selection_point(dict);
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
                    self.begin -= 1;
                    self.begin = self.after_previous_break_point(self.begin);
                }
            }
            let syllables = &self.com.symbols()[self.begin..self.end];
            if dict
                .lookup_first_phrase(&syllables, self.lookup_strategy)
                .is_some()
            {
                break;
            }
        }
    }

    fn next_break_point(&self, mut cursor: usize) -> usize {
        loop {
            if self.com.len() == cursor {
                break;
            }
            if let Some(sym) = self.com.symbol(cursor) {
                if !sym.is_syllable() {
                    break;
                }
            }
            cursor += 1;
        }
        cursor
    }

    fn after_previous_break_point(&self, mut cursor: usize) -> usize {
        let selection_ends: Vec<_> = self.com.selections().iter().map(|sel| sel.end).collect();
        loop {
            if cursor == 0 {
                return 0;
            }
            if selection_ends.contains(&cursor) {
                break;
            }
            if let Some(Gap::Break) = self.com.gap(cursor) {
                break;
            }
            if let Some(sym) = self.com.symbol(cursor - 1) {
                if !sym.is_syllable() {
                    break;
                }
            }
            cursor -= 1;
        }
        cursor
    }

    pub(crate) fn candidates(&self, editor: &SharedState, dict: &Layered) -> Vec<String> {
        let mut candidates = dict
            .lookup_all_phrases(
                &&self.com.symbols()[self.begin..self.end],
                self.lookup_strategy,
            )
            .into_iter()
            .map(|phrase| phrase.into())
            .collect::<Vec<String>>();
        if self.end - self.begin == 1 {
            let alt = editor
                .syl
                .alt_syllables(self.com.symbol(self.begin).unwrap().to_syllable().unwrap());
            for &syl in alt {
                candidates.extend(
                    dict.lookup_all_phrases(&[syl], self.lookup_strategy)
                        .into_iter()
                        .map(|ph| ph.into()),
                )
            }
        }
        candidates
    }

    pub(crate) fn interval(&self, phrase: impl Into<Box<str>>) -> Interval {
        Interval {
            start: self.begin,
            end: self.end,
            is_phrase: true,
            str: phrase.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        conversion::{Composition, Interval, Symbol},
        dictionary::{LookupStrategy, TrieBuf},
        syl,
        zhuyin::Bopomofo::*,
    };

    use super::PhraseSelector;

    #[test]
    fn init_when_cursor_end_of_buffer_syllable() {
        let mut com = Composition::new();
        com.push(Symbol::from(syl![C, E, TONE4]));
        let mut sel = PhraseSelector {
            begin: 0,
            end: 1,
            forward_select: false,
            orig: 0,
            lookup_strategy: LookupStrategy::Standard,
            com,
        };
        let dict = TrieBuf::from([(vec![syl![C, E, TONE4]], vec![("測", 100)])]);
        sel.init(1, &dict);

        assert_eq!(0, sel.begin);
        assert_eq!(1, sel.end);
    }

    #[test]
    #[should_panic]
    fn init_when_cursor_end_of_buffer_not_syllable() {
        let mut com = Composition::new();
        com.push(Symbol::from(','));
        let mut sel = PhraseSelector {
            begin: 0,
            end: 1,
            forward_select: false,
            orig: 0,
            lookup_strategy: LookupStrategy::Standard,
            com,
        };
        let dict = TrieBuf::from([(vec![syl![C, E, TONE4]], vec![("測", 100)])]);
        sel.init(1, &dict);
    }

    #[test]
    fn init_forward_select_when_cursor_end_of_buffer_syllable() {
        let mut com = Composition::new();
        com.push(Symbol::from(syl![C, E, TONE4]));
        let mut sel = PhraseSelector {
            begin: 0,
            end: 1,
            forward_select: true,
            orig: 0,
            lookup_strategy: LookupStrategy::Standard,
            com,
        };
        let dict = TrieBuf::from([(vec![syl![C, E, TONE4]], vec![("測", 100)])]);
        sel.init(1, &dict);

        assert_eq!(0, sel.begin);
        assert_eq!(1, sel.end);
    }

    #[test]
    #[should_panic]
    fn init_forward_select_when_cursor_end_of_buffer_not_syllable() {
        let mut com = Composition::new();
        com.push(Symbol::from(','));
        let mut sel = PhraseSelector {
            begin: 0,
            end: 1,
            forward_select: true,
            orig: 0,
            lookup_strategy: LookupStrategy::Standard,
            com,
        };
        let dict = TrieBuf::from([(vec![syl![C, E, TONE4]], vec![("測", 100)])]);
        sel.init(1, &dict);
    }

    #[test]
    fn should_stop_at_left_boundary() {
        let mut com = Composition::new();
        for sym in [
            Symbol::from(syl![C, E, TONE4]),
            Symbol::from(syl![C, E, TONE4]),
        ] {
            com.push(sym);
        }
        let sel = PhraseSelector {
            begin: 0,
            end: 2,
            forward_select: false,
            orig: 0,
            lookup_strategy: LookupStrategy::Standard,
            com,
        };

        assert_eq!(0, sel.after_previous_break_point(0));
        assert_eq!(0, sel.after_previous_break_point(1));
        assert_eq!(0, sel.after_previous_break_point(2));
    }

    #[test]
    fn should_stop_after_first_non_syllable() {
        let mut com = Composition::new();
        for sym in [Symbol::from(','), Symbol::from(syl![C, E, TONE4])] {
            com.push(sym);
        }
        let sel = PhraseSelector {
            begin: 0,
            end: 2,
            forward_select: false,
            orig: 0,
            lookup_strategy: LookupStrategy::Standard,
            com,
        };

        assert_eq!(0, sel.after_previous_break_point(0));
        assert_eq!(1, sel.after_previous_break_point(1));
        assert_eq!(1, sel.after_previous_break_point(2));
    }

    #[test]
    fn should_stop_after_first_selection() {
        let mut com = Composition::new();
        for sym in [
            Symbol::from(syl![C, E, TONE4]),
            Symbol::from(syl![C, E, TONE4]),
        ] {
            com.push(sym);
        }
        com.push_selection(Interval {
            start: 0,
            end: 1,
            is_phrase: true,
            str: "冊".into(),
        });
        let sel = PhraseSelector {
            begin: 0,
            end: 2,
            forward_select: false,
            orig: 0,
            lookup_strategy: LookupStrategy::Standard,
            com,
        };

        assert_eq!(0, sel.after_previous_break_point(0));
        assert_eq!(1, sel.after_previous_break_point(1));
        assert_eq!(1, sel.after_previous_break_point(2));
    }
}
