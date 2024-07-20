use std::iter;

use crate::dictionary::Dictionary;

use super::{Composition, ConversionEngine, Interval};

/// Simple engine does not perform any intelligent conversion.
#[derive(Debug, Default)]
pub struct SimpleEngine;

impl SimpleEngine {
    pub fn new() -> SimpleEngine {
        SimpleEngine
    }
    pub fn convert<'a>(
        &'a self,
        dict: &'a dyn Dictionary,
        comp: &'a Composition,
    ) -> impl Iterator<Item = Vec<Interval>> + Clone + 'a {
        let mut intervals = vec![];

        for (i, sym) in comp.symbols().iter().enumerate() {
            if comp
                .selections()
                .iter()
                .any(|selection| selection.intersect_range(i, i + 1))
            {
                continue;
            }
            if sym.is_char() {
                intervals.push(Interval {
                    start: i,
                    end: i + 1,
                    is_phrase: false,
                    str: sym.to_char().unwrap().to_string().into_boxed_str(),
                });
            } else {
                let phrase = dict.lookup_first_phrase(
                    &[sym.to_syllable().unwrap()],
                    crate::dictionary::LookupStrategy::Standard,
                );
                let phrase_str = phrase.map_or_else(
                    || sym.to_syllable().unwrap().to_string(),
                    |phrase| phrase.to_string(),
                );
                intervals.push(Interval {
                    start: i,
                    end: i + 1,
                    is_phrase: true,
                    str: phrase_str.into_boxed_str(),
                })
            }
        }
        intervals.extend_from_slice(comp.selections());
        intervals.sort_by_key(|int| int.start);
        iter::once(intervals)
    }
}

impl ConversionEngine for SimpleEngine {
    fn convert<'a>(
        &'a self,
        dict: &'a dyn Dictionary,
        comp: &'a Composition,
    ) -> Box<dyn Iterator<Item = Vec<Interval>> + 'a> {
        Box::new(SimpleEngine::convert(self, dict, comp))
    }
}
