use std::iter;

use crate::{
    conversion::Outcome,
    dictionary::{Dictionary, LookupStrategy},
};

use super::{Composition, ConversionEngine, Interval};

/// Simple engine does not perform any intelligent conversion.
#[derive(Debug, Default)]
pub struct SimpleEngine;

impl SimpleEngine {
    pub fn new() -> SimpleEngine {
        SimpleEngine
    }
    pub fn convert<'a>(&'a self, dict: &'a dyn Dictionary, comp: &'a Composition) -> Vec<Outcome> {
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
                    text: sym.to_char().unwrap().to_string().into_boxed_str(),
                });
            } else {
                let phrase = dict
                    .lookup(&[sym.to_syllable().unwrap()], LookupStrategy::Standard)
                    .first()
                    .cloned();
                let phrase_str = phrase.map_or_else(
                    || sym.to_syllable().unwrap().to_string(),
                    |phrase| phrase.to_string(),
                );
                intervals.push(Interval {
                    start: i,
                    end: i + 1,
                    is_phrase: true,
                    text: phrase_str.into_boxed_str(),
                })
            }
        }
        intervals.extend_from_slice(comp.selections());
        intervals.sort_by_key(|int| int.start);
        vec![Outcome {
            intervals,
            log_prob: 0.0,
        }]
    }
}

impl ConversionEngine for SimpleEngine {
    fn convert<'a>(&'a self, dict: &'a dyn Dictionary, comp: &'a Composition) -> Vec<Outcome> {
        SimpleEngine::convert(self, dict, comp)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        conversion::{Composition, Interval, Outcome, Symbol},
        dictionary::{Dictionary, TrieBuf},
        syl,
        zhuyin::Bopomofo::*,
    };

    use super::SimpleEngine;

    fn test_dictionary() -> impl Dictionary {
        TrieBuf::from([
            (vec![syl![G, U, O, TONE2]], vec![("國", 1)]),
            (vec![syl![M, I, EN, TONE2]], vec![("民", 1)]),
            (vec![syl![D, A, TONE4]], vec![("大", 1)]),
            (vec![syl![H, U, EI, TONE4]], vec![("會", 1)]),
            (vec![syl![D, AI, TONE4]], vec![("代", 1)]),
            (vec![syl![B, I, AU, TONE3]], vec![("表", 1), ("錶", 1)]),
            (vec![syl![C, E, TONE4]], vec![("測", 18)]),
            (vec![syl![SH, TONE4]], vec![("試", 18)]),
            (
                vec![syl![G, U, O, TONE2], syl![M, I, EN, TONE2]],
                vec![("國民", 200)],
            ),
            (
                vec![syl![D, A, TONE4], syl![H, U, EI, TONE4]],
                vec![("大會", 200)],
            ),
            (
                vec![syl![D, AI, TONE4], syl![B, I, AU, TONE3]],
                vec![("代表", 200), ("戴錶", 100)],
            ),
            (vec![syl![X, I, EN]], vec![("心", 1)]),
            (vec![syl![K, U, TONE4], syl![I, EN]], vec![("庫音", 300)]),
            (
                vec![syl![X, I, EN], syl![K, U, TONE4], syl![I, EN]],
                vec![("新酷音", 200)],
            ),
            (
                vec![syl![C, E, TONE4], syl![SH, TONE4], syl![I, TONE2]],
                vec![("測試儀", 42)],
            ),
            (
                vec![syl![C, E, TONE4], syl![SH, TONE4]],
                vec![("測試", 9318)],
            ),
            (
                vec![syl![I, TONE2], syl![X, I, A, TONE4]],
                vec![("一下", 10576)],
            ),
            (vec![syl![X, I, A, TONE4]], vec![("下", 10576)]),
            (vec![syl![H, A]], vec![("哈", 1)]),
            (vec![syl![H, A], syl![H, A]], vec![("哈哈", 1)]),
        ])
    }

    #[test]
    fn convert_empty_composition() {
        let dict = test_dictionary();
        let engine = SimpleEngine::new();
        let composition = Composition::new();
        assert_eq!(
            vec![Outcome {
                intervals: vec![],
                log_prob: 0.0
            }],
            engine.convert(&dict, &composition)
        );
    }

    // Some corrupted user dictionary may contain empty length syllables
    #[test]
    fn convert_zero_length_entry() {
        let mut dict = test_dictionary();
        let dict_mut = dict.as_dict_mut().unwrap();
        dict_mut.add_phrase(&[], ("", 0).into()).unwrap();
        let engine = SimpleEngine::new();
        let mut composition = Composition::new();
        for sym in [
            Symbol::from(syl![C, E, TONE4]),
            Symbol::from(syl![SH, TONE4]),
        ] {
            composition.push(sym);
        }
        assert_eq!(
            vec![Outcome {
                intervals: vec![
                    Interval {
                        start: 0,
                        end: 1,
                        is_phrase: true,
                        text: "測".into()
                    },
                    Interval {
                        start: 1,
                        end: 2,
                        is_phrase: true,
                        text: "試".into()
                    },
                ],
                log_prob: 0.0
            }],
            engine.convert(&dict, &composition)
        );
    }

    #[test]
    fn convert_simple_chinese_composition() {
        let dict = test_dictionary();
        let engine = SimpleEngine::new();
        let mut composition = Composition::new();
        for sym in [
            Symbol::from(syl![G, U, O, TONE2]),
            Symbol::from(syl![M, I, EN, TONE2]),
            Symbol::from(syl![D, A, TONE4]),
            Symbol::from(syl![H, U, EI, TONE4]),
            Symbol::from(syl![D, AI, TONE4]),
            Symbol::from(syl![B, I, AU, TONE3]),
        ] {
            composition.push(sym);
        }
        assert_eq!(
            vec![Outcome {
                intervals: vec![
                    Interval {
                        start: 0,
                        end: 1,
                        is_phrase: true,
                        text: "國".into()
                    },
                    Interval {
                        start: 1,
                        end: 2,
                        is_phrase: true,
                        text: "民".into()
                    },
                    Interval {
                        start: 2,
                        end: 3,
                        is_phrase: true,
                        text: "大".into()
                    },
                    Interval {
                        start: 3,
                        end: 4,
                        is_phrase: true,
                        text: "會".into()
                    },
                    Interval {
                        start: 4,
                        end: 5,
                        is_phrase: true,
                        text: "代".into()
                    },
                    Interval {
                        start: 5,
                        end: 6,
                        is_phrase: true,
                        text: "表".into()
                    },
                ],
                log_prob: 0.0
            }],
            engine.convert(&dict, &composition)
        );
    }
}
