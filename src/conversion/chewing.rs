use std::{
    collections::BTreeMap,
    fmt::{Debug, Display, Write},
    iter,
    ops::Neg,
};

use log::{debug, log_enabled, trace, Level::Trace};

use crate::dictionary::{Dictionary, Phrase};

use super::{Composition, Gap, Interval, Symbol};

/// TODO: doc
#[derive(Debug, Default)]
pub struct ChewingEngine;

impl ChewingEngine {
    /// TODO: doc
    pub fn new() -> ChewingEngine {
        ChewingEngine
    }
    pub fn convert<'a>(
        &'a self,
        dict: &'a dyn Dictionary,
        comp: &'a Composition,
    ) -> impl Iterator<Item = Vec<Interval>> + Clone + 'a {
        let fast_dp = iter::once_with(|| {
            if comp.is_empty() {
                return vec![];
            }
            let intervals = self.find_intervals(dict, comp);
            self.find_best_path(comp.symbols.len(), intervals)
                .into_iter()
                .map(|interval| interval.into())
                .fold(vec![], |acc, interval| glue_fn(comp, acc, interval))
        });
        let slow_search = iter::once_with(move || {
            if comp.is_empty() {
                return vec![];
            }
            let mut graph = Graph::default();
            let paths = self.find_all_paths(dict, &mut graph, comp, 0, comp.len(), None);
            debug_assert!(!paths.is_empty());

            let mut trimmed_paths = self.trim_paths(paths);
            debug_assert!(!trimmed_paths.is_empty());

            trimmed_paths.sort_by(|a, b| b.cmp(a));
            trimmed_paths
        })
        .flatten()
        .skip(1)
        .map(|p| {
            p.intervals
                .into_iter()
                .map(|it| it.into())
                .fold(vec![], |acc, interval| glue_fn(comp, acc, interval))
        });
        fast_dp.chain(slow_search)
    }
}

fn glue_fn(com: &Composition, mut acc: Vec<Interval>, interval: Interval) -> Vec<Interval> {
    if acc.is_empty() {
        acc.push(interval);
        return acc;
    }
    let last = acc.last().expect("acc should have at least one item");
    if let Some(Gap::Glue) = com.gap(last.end) {
        let last = acc.pop().expect("acc should have at least one item");
        let mut phrase = last.str.into_string();
        phrase.push_str(&interval.str);
        acc.push(Interval {
            start: last.start,
            end: interval.end,
            is_phrase: true,
            str: phrase.into_boxed_str(),
        })
    } else {
        acc.push(interval);
    }
    acc
}

impl ChewingEngine {
    fn find_best_phrase<D: Dictionary + ?Sized>(
        &self,
        dict: &D,
        start: usize,
        symbols: &[Symbol],
        com: &Composition,
    ) -> Option<PossiblePhrase> {
        let end = start + symbols.len();

        for i in (start..end).skip(1) {
            if let Some(Gap::Break) = com.gap(i) {
                // There exists a break point that forbids connecting these
                // syllables.
                debug!("No best phrase for {:?} due to break point", symbols);
                return None;
            }
        }

        for selection in &com.selections {
            if selection.intersect_range(start, end) && !selection.is_contained_by(start, end) {
                // There's a conflicting partial intersecting selection.
                debug!(
                    "No best phrase for {:?} due to selection {:?}",
                    symbols, selection
                );
                return None;
            }
        }

        if symbols.len() == 1 && symbols[0].is_char() {
            return Some(symbols[0].into());
        }

        let syllables = symbols
            .iter()
            .take_while(|symbol| symbol.is_syllable())
            .map(|symbol| symbol.to_syllable().unwrap())
            .collect::<Vec<_>>();
        if syllables.len() != symbols.len() {
            return None;
        }

        let mut max_freq = 0;
        let mut best_phrase = None;
        'next_phrase: for phrase in dict.lookup_all_phrases(&syllables) {
            // If there exists a user selected interval which is a
            // sub-interval of this phrase but the substring is
            // different then we can skip this phrase.
            for selection in &com.selections {
                debug_assert!(!selection.str.is_empty());
                if start <= selection.start && end >= selection.end {
                    let offset = selection.start - start;
                    let len = selection.end - selection.start;
                    let substring: String =
                        phrase.as_str().chars().skip(offset).take(len).collect();
                    if substring != selection.str.as_ref() {
                        continue 'next_phrase;
                    }
                }
            }

            // If there are phrases that can satisfy all the constraints
            // then pick the one with highest frequency.
            if !(phrase.freq() > max_freq || best_phrase.is_none()) {
                continue;
            }
            max_freq = phrase.freq();
            best_phrase = Some(phrase.into());
        }

        debug!("best phrace for {:?} is {:?}", symbols, best_phrase);
        best_phrase
    }
    fn find_intervals<D: Dictionary + ?Sized>(
        &self,
        dict: &D,
        com: &Composition,
    ) -> Vec<PossibleInterval> {
        let mut intervals = vec![];
        for begin in 0..com.symbols.len() {
            for end in begin..=com.symbols.len() {
                if let Some(phrase) =
                    self.find_best_phrase(dict, begin, &com.symbols[begin..end], com)
                {
                    intervals.push(PossibleInterval {
                        start: begin,
                        end,
                        phrase,
                    });
                }
            }
        }
        intervals
    }
    /// Calculate the best path with dynamic programming.
    ///
    /// Assume P(x,y) is the highest score phrasing result from x to y. The
    /// following is formula for P(x,y):
    ///
    /// P(x,y) = MAX( P(x,y-1)+P(y-1,y), P(x,y-2)+P(y-2,y), ... )
    ///
    /// While P(x,y-1) is stored in highest_score array, and P(y-1,y) is
    /// interval end at y. In this formula, x is always 0.
    ///
    /// The format of highest_score array is described as following:
    ///
    /// highest_score[0] = P(0,0)
    /// highest_score[1] = P(0,1)
    /// ...
    /// highest_score[y-1] = P(0,y-1)
    fn find_best_path(
        &self,
        len: usize,
        mut intervals: Vec<PossibleInterval>,
    ) -> Vec<PossibleInterval> {
        let mut highest_score = vec![PossiblePath::default(); len + 1];

        // The interval shall be sorted by the increase order of end.
        intervals.sort_by(|a, b| a.end.cmp(&b.end));

        for interval in intervals.into_iter() {
            let start = interval.start;
            let end = interval.end;

            let mut candidate_path = highest_score[start].clone();
            candidate_path.intervals.push(interval);

            if highest_score[end].score() < candidate_path.score() {
                highest_score[end] = candidate_path;
            }
        }

        highest_score
            .pop()
            .expect("highest_score has at least one element")
            .intervals
    }

    fn find_all_paths<'g, D: Dictionary + ?Sized>(
        &'g self,
        dict: &D,
        graph: &mut Graph<'g>,
        com: &Composition,
        start: usize,
        target: usize,
        prefix: Option<PossiblePath>,
    ) -> Vec<PossiblePath> {
        if start == target {
            return vec![prefix.expect("should have prefix")];
        }
        let mut result = vec![];
        for end in (start + 1)..=target {
            let entry = graph.entry((start, end));
            if let Some(phrase) = entry.or_insert_with(|| {
                self.find_best_phrase(dict, start, &com.symbols[start..end], com)
            }) {
                let mut prefix = prefix.clone().unwrap_or_default();
                prefix.intervals.push(PossibleInterval {
                    start,
                    end,
                    phrase: phrase.clone(),
                });
                result.append(&mut self.find_all_paths(
                    dict,
                    graph,
                    com,
                    end,
                    target,
                    Some(prefix),
                ));
            }
        }
        result
    }

    /// Trim some paths that were part of other paths
    ///
    /// Ported from original C implementation, but the original algorithm seems wrong.
    fn trim_paths(&self, paths: Vec<PossiblePath>) -> Vec<PossiblePath> {
        let mut trimmed_paths: Vec<PossiblePath> = vec![];
        for candidate in paths.into_iter() {
            if log_enabled!(Trace) {
                trace!("Trim check {}", candidate);
            }
            let mut drop_candidate = false;
            let mut keeper = vec![];
            for p in trimmed_paths.into_iter() {
                if drop_candidate || p.contains(&candidate) {
                    drop_candidate = true;
                    if log_enabled!(Trace) {
                        trace!("  Keep {}", p);
                    }
                    keeper.push(p);
                    continue;
                }
                if candidate.contains(&p) {
                    if log_enabled!(Trace) {
                        trace!("  Drop {}", p);
                    }
                    continue;
                }
                if log_enabled!(Trace) {
                    trace!("  Keep {}", p);
                }
                keeper.push(p);
            }
            if !drop_candidate {
                if log_enabled!(Trace) {
                    trace!("  Keep {}", candidate);
                }
                keeper.push(candidate);
            }
            trimmed_paths = keeper;
        }
        trimmed_paths
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum PossiblePhrase {
    Symbol(Symbol),
    Phrase(Phrase),
}

impl PossiblePhrase {
    fn freq(&self) -> u32 {
        match self {
            PossiblePhrase::Symbol(_) => 0,
            PossiblePhrase::Phrase(phrase) => phrase.freq(),
        }
    }
}

impl Display for PossiblePhrase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PossiblePhrase::Symbol(sym) => f.write_char(sym.to_char().unwrap()),
            PossiblePhrase::Phrase(phrase) => f.write_str(phrase.as_str()),
        }
    }
}

impl From<Phrase> for PossiblePhrase {
    fn from(value: Phrase) -> Self {
        PossiblePhrase::Phrase(value)
    }
}

impl From<Symbol> for PossiblePhrase {
    fn from(value: Symbol) -> Self {
        PossiblePhrase::Symbol(value)
    }
}

impl From<PossiblePhrase> for Box<str> {
    fn from(value: PossiblePhrase) -> Self {
        match value {
            PossiblePhrase::Symbol(sym) => sym.to_char().unwrap().to_string().into_boxed_str(),
            PossiblePhrase::Phrase(phrase) => phrase.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PossibleInterval {
    start: usize,
    end: usize,
    phrase: PossiblePhrase,
}

impl PossibleInterval {
    fn contains(&self, other: &PossibleInterval) -> bool {
        self.start <= other.start && self.end >= other.end
    }
    fn len(&self) -> usize {
        self.end - self.start
    }
}

impl From<PossibleInterval> for Interval {
    fn from(value: PossibleInterval) -> Self {
        Interval {
            start: value.start,
            end: value.end,
            is_phrase: match value.phrase {
                PossiblePhrase::Symbol(_) => false,
                PossiblePhrase::Phrase(_) => true,
            },
            str: value.phrase.into(),
        }
    }
}

#[derive(Default, Clone, Eq)]
struct PossiblePath {
    intervals: Vec<PossibleInterval>,
}

impl Debug for PossiblePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PossiblePath")
            .field("score()", &self.score())
            .field("intervals", &self.intervals)
            .finish()
    }
}

impl PossiblePath {
    fn score(&self) -> i32 {
        let mut score = 0;
        score += 1000 * self.rule_largest_sum();
        score += 1000 * self.rule_largest_avgwordlen();
        score += 100 * self.rule_smallest_lenvariance();
        score += self.rule_largest_freqsum();
        score
    }

    /// Copied from IsRecContain to trim some paths
    fn contains(&self, other: &Self) -> bool {
        let mut big = 0;
        for sml in 0..other.intervals.len() {
            loop {
                if big < self.intervals.len()
                    && self.intervals[big].start < other.intervals[sml].end
                {
                    if self.intervals[big].contains(&other.intervals[sml]) {
                        break;
                    }
                } else {
                    return false;
                }
                big += 1;
            }
        }
        true
    }

    fn rule_largest_sum(&self) -> i32 {
        let mut score = 0;
        for interval in &self.intervals {
            score += interval.end - interval.start;
        }
        score as i32
    }

    fn rule_largest_avgwordlen(&self) -> i32 {
        if self.intervals.is_empty() {
            return 0;
        }
        // Constant factor 6=1*2*3, to keep value as integer
        6 * self.rule_largest_sum()
            / i32::try_from(self.intervals.len()).expect("number of intervals should be small")
    }

    fn rule_smallest_lenvariance(&self) -> i32 {
        let len = self.intervals.len();
        let mut score = 0;
        // kcwu: heuristic? why variance no square function?
        for i in 0..len {
            for j in i + 1..len {
                let interval_1 = &self.intervals[i];
                let interval_2 = &self.intervals[j];
                score += interval_1.len().abs_diff(interval_2.len());
            }
        }
        i32::try_from(score).expect("score should fit in i32").neg()
    }

    fn rule_largest_freqsum(&self) -> i32 {
        let mut score = 0;
        for interval in &self.intervals {
            let reduction_factor = if interval.len() == 1 { 512 } else { 1 };
            score += interval.phrase.freq() / reduction_factor;
        }
        i32::try_from(score).expect("score should fit in i32")
    }
}

impl PartialEq for PossiblePath {
    fn eq(&self, other: &Self) -> bool {
        self.score() == other.score()
    }
}

impl PartialOrd for PossiblePath {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PossiblePath {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.score().cmp(&other.score())
    }
}

impl Display for PossiblePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#PossiblePath({}", self.score())?;
        for interval in &self.intervals {
            write!(
                f,
                " ({} {} '{})",
                interval.start, interval.end, interval.phrase
            )?;
        }
        write!(f, ")")?;
        Ok(())
    }
}

type Graph<'a> = BTreeMap<(usize, usize), Option<PossiblePhrase>>;

#[cfg(test)]
mod tests {
    use crate::{
        conversion::{Composition, Gap, Interval, Symbol},
        dictionary::{Dictionary, Phrase, TrieBuf},
        syl,
        zhuyin::Bopomofo::*,
    };

    use super::{ChewingEngine, PossibleInterval, PossiblePath};

    fn test_dictionary() -> impl Dictionary {
        TrieBuf::from([
            (vec![syl![G, U, O, TONE2]], vec![("國", 1)]),
            (vec![syl![M, I, EN, TONE2]], vec![("民", 1)]),
            (vec![syl![D, A, TONE4]], vec![("大", 1)]),
            (vec![syl![H, U, EI, TONE4]], vec![("會", 1)]),
            (vec![syl![D, AI, TONE4]], vec![("代", 1)]),
            (vec![syl![B, I, AU, TONE3]], vec![("表", 1), ("錶", 1)]),
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
        ])
    }

    #[test]
    fn convert_empty_composition() {
        let dict = test_dictionary();
        let engine = ChewingEngine::new();
        let composition = Composition::new();
        assert_eq!(
            Some(Vec::<Interval>::new()),
            engine.convert(&dict, &composition).next()
        );
    }

    #[test]
    fn convert_simple_chinese_composition() {
        let dict = test_dictionary();
        let engine = ChewingEngine::new();
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
            Some(vec![
                Interval {
                    start: 0,
                    end: 2,
                    is_phrase: true,
                    str: "國民".into()
                },
                Interval {
                    start: 2,
                    end: 4,
                    is_phrase: true,
                    str: "大會".into()
                },
                Interval {
                    start: 4,
                    end: 6,
                    is_phrase: true,
                    str: "代表".into()
                },
            ]),
            engine.convert(&dict, &composition).next()
        );
    }

    #[test]
    fn convert_chinese_composition_with_breaks() {
        let dict = test_dictionary();
        let engine = ChewingEngine::new();
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
        composition.set_gap(1, Gap::Break);
        composition.set_gap(5, Gap::Break);
        assert_eq!(
            Some(vec![
                Interval {
                    start: 0,
                    end: 1,
                    is_phrase: true,
                    str: "國".into()
                },
                Interval {
                    start: 1,
                    end: 2,
                    is_phrase: true,
                    str: "民".into()
                },
                Interval {
                    start: 2,
                    end: 4,
                    is_phrase: true,
                    str: "大會".into()
                },
                Interval {
                    start: 4,
                    end: 5,
                    is_phrase: true,
                    str: "代".into()
                },
                Interval {
                    start: 5,
                    end: 6,
                    is_phrase: true,
                    str: "表".into()
                },
            ]),
            engine.convert(&dict, &composition).next()
        );
    }

    #[test]
    fn convert_chinese_composition_with_good_selection() {
        let dict = test_dictionary();
        let engine = ChewingEngine::new();
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
        composition.push_selection(Interval {
            start: 4,
            end: 6,
            is_phrase: true,
            str: "戴錶".into(),
        });
        assert_eq!(
            Some(vec![
                Interval {
                    start: 0,
                    end: 2,
                    is_phrase: true,
                    str: "國民".into()
                },
                Interval {
                    start: 2,
                    end: 4,
                    is_phrase: true,
                    str: "大會".into()
                },
                Interval {
                    start: 4,
                    end: 6,
                    is_phrase: true,
                    str: "戴錶".into()
                },
            ]),
            engine.convert(&dict, &composition).next()
        );
    }

    #[test]
    fn convert_chinese_composition_with_substring_selection() {
        let dict = test_dictionary();
        let engine = ChewingEngine::new();
        let mut composition = Composition::new();
        for sym in [
            Symbol::from(syl![X, I, EN]),
            Symbol::from(syl![K, U, TONE4]),
            Symbol::from(syl![I, EN]),
        ] {
            composition.push(sym);
        }
        composition.push_selection(Interval {
            start: 1,
            end: 3,
            is_phrase: true,
            str: "酷音".into(),
        });
        assert_eq!(
            Some(vec![Interval {
                start: 0,
                end: 3,
                is_phrase: true,
                str: "新酷音".into()
            },]),
            engine.convert(&dict, &composition).next()
        );
    }

    #[test]
    fn multiple_single_word_selection() {
        let dict = test_dictionary();
        let engine = ChewingEngine::new();
        let mut composition = Composition::new();
        for sym in [
            Symbol::from(syl![D, AI, TONE4]),
            Symbol::from(syl![B, I, AU, TONE3]),
        ] {
            composition.push(sym);
        }
        for interval in [
            Interval {
                start: 0,
                end: 1,
                is_phrase: true,
                str: "代".into(),
            },
            Interval {
                start: 1,
                end: 2,
                is_phrase: true,
                str: "錶".into(),
            },
        ] {
            composition.push_selection(interval);
        }
        assert_eq!(
            Some(vec![
                Interval {
                    start: 0,
                    end: 1,
                    is_phrase: true,
                    str: "代".into()
                },
                Interval {
                    start: 1,
                    end: 2,
                    is_phrase: true,
                    str: "錶".into()
                }
            ]),
            engine.convert(&dict, &composition).next()
        );
    }

    #[test]
    fn convert_cycle_alternatives() {
        let dict = test_dictionary();
        let engine = ChewingEngine::new();
        let mut composition = Composition::new();
        for sym in [
            Symbol::from(syl![C, E, TONE4]),
            Symbol::from(syl![SH, TONE4]),
            Symbol::from(syl![I, TONE2]),
            Symbol::from(syl![X, I, A, TONE4]),
        ] {
            composition.push(sym);
        }
        assert_eq!(
            Some(vec![
                Interval {
                    start: 0,
                    end: 2,
                    is_phrase: true,
                    str: "測試".into()
                },
                Interval {
                    start: 2,
                    end: 4,
                    is_phrase: true,
                    str: "一下".into()
                }
            ]),
            engine.convert(&dict, &composition).next()
        );
        assert_eq!(
            Some(vec![
                Interval {
                    start: 0,
                    end: 3,
                    is_phrase: true,
                    str: "測試儀".into()
                },
                Interval {
                    start: 3,
                    end: 4,
                    is_phrase: true,
                    str: "下".into()
                }
            ]),
            engine.convert(&dict, &composition).nth(1)
        );
        assert_eq!(
            Some(vec![
                Interval {
                    start: 0,
                    end: 2,
                    is_phrase: true,
                    str: "測試".into()
                },
                Interval {
                    start: 2,
                    end: 4,
                    is_phrase: true,
                    str: "一下".into()
                }
            ]),
            engine.convert(&dict, &composition).cycle().nth(2)
        );
    }

    #[test]
    fn possible_path_contains() {
        let path_1 = PossiblePath {
            intervals: vec![
                PossibleInterval {
                    start: 0,
                    end: 2,
                    phrase: Phrase::new("測試", 0).into(),
                },
                PossibleInterval {
                    start: 2,
                    end: 4,
                    phrase: Phrase::new("一下", 0).into(),
                },
            ],
        };
        let path_2 = PossiblePath {
            intervals: vec![
                PossibleInterval {
                    start: 0,
                    end: 2,
                    phrase: Phrase::new("測試", 0).into(),
                },
                PossibleInterval {
                    start: 2,
                    end: 3,
                    phrase: Phrase::new("遺", 0).into(),
                },
                PossibleInterval {
                    start: 3,
                    end: 4,
                    phrase: Phrase::new("下", 0).into(),
                },
            ],
        };
        assert!(path_1.contains(&path_2));
    }
}
