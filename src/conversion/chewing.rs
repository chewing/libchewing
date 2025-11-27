use std::{
    collections::VecDeque,
    fmt::{Debug, Display, Write},
    iter,
    ops::Not,
};

use log::trace;

use crate::{
    dictionary::{Dictionary, LookupStrategy, Phrase},
    zhuyin::Syllable,
};

use super::{Composition, ConversionEngine, Gap, Interval, Symbol};

/// The default Chewing conversion method.
#[derive(Debug, Default)]
pub struct ChewingEngine {
    pub(crate) lookup_strategy: LookupStrategy,
}

impl ChewingEngine {
    const MAX_OUT_PATHS: usize = 100;
    /// Creates a new conversion engine.
    pub fn new() -> ChewingEngine {
        ChewingEngine {
            lookup_strategy: LookupStrategy::Standard,
        }
    }
    pub(crate) fn convert<'a>(
        &'a self,
        dict: &'a dyn Dictionary,
        comp: &'a Composition,
    ) -> impl Iterator<Item = Vec<Interval>> + Clone + 'a {
        iter::once_with(move || {
            if comp.is_empty() {
                return vec![PossiblePath::default()];
            }
            let intervals = self.find_edges(dict, comp);
            if intervals.is_empty() {
                return vec![PossiblePath::default()];
            }
            let paths = self.find_k_paths(Self::MAX_OUT_PATHS, comp.len(), intervals);
            trace!("paths: {:#?}", paths);
            debug_assert!(!paths.is_empty());

            let mut trimmed_paths = self.trim_paths(paths);
            debug_assert!(!trimmed_paths.is_empty());

            trimmed_paths.sort_by(|a, b| b.cmp(a));
            trimmed_paths
        })
        .flatten()
        .map(|p| {
            p.intervals
                .into_iter()
                .map(|it| it.into())
                .fold(vec![], |acc, interval| glue_fn(comp, acc, interval))
        })
    }
}

impl ConversionEngine for ChewingEngine {
    fn convert<'a>(
        &'a self,
        dict: &'a dyn Dictionary,
        comp: &'a Composition,
    ) -> Box<dyn Iterator<Item = Vec<Interval>> + 'a> {
        Box::new(ChewingEngine::convert(self, dict, comp))
    }
}

fn glue_fn(com: &Composition, mut acc: Vec<Interval>, interval: Interval) -> Vec<Interval> {
    if acc.is_empty() {
        acc.push(interval);
        return acc;
    }
    let last = acc.last().expect("acc should have at least one item");
    if !last.is_phrase || !interval.is_phrase {
        acc.push(interval);
        return acc;
    }
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
    fn find_best_phrases<D: Dictionary + ?Sized>(
        &self,
        dict: &D,
        start: usize,
        symbols: &[Symbol],
        com: &Composition,
    ) -> Vec<PossiblePhrase> {
        let end = start + symbols.len();

        for i in (start..end).skip(1) {
            if let Some(Gap::Break) = com.gap(i) {
                // There exists a break point that forbids connecting these
                // syllables.
                trace!("No best phrase for {:?} due to break point", symbols);
                return vec![];
            }
        }

        for selection in &com.selections {
            if selection.intersect_range(start, end) && !selection.is_contained_by(start, end) {
                // There's a conflicting partial intersecting selection.
                trace!(
                    "No best phrase for {:?} due to selection {:?}",
                    symbols, selection
                );
                return vec![];
            }
        }

        if symbols.len() == 1 && symbols[0].is_char() {
            return vec![PossiblePhrase::Symbol(symbols[0])];
        }

        if symbols.iter().any(|sym| sym.is_char()) {
            return vec![];
        }

        let syllables: Vec<Syllable> = symbols
            .iter()
            .map(|s| s.to_syllable().unwrap_or_default())
            .collect();

        let max_phrases_count = 10;
        // Approximate value. We only use this global for scaling for now, so we can
        // use any value.
        let global_total: f64 = 1_000_000_000.0;
        let mut phrases = dict
            .lookup(&syllables, self.lookup_strategy)
            .into_iter()
            .filter(|phrase| {
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
                            return false;
                        }
                    }
                }
                true
            })
            .map(|phrase| {
                let log_prob = (phrase.freq() as f64 / global_total).ln();
                PossiblePhrase::Phrase(phrase, log_prob)
            })
            .collect::<Vec<_>>();
        phrases.sort_by(|a, b| f64::total_cmp(&-a.log_prob(), &-b.log_prob()));
        phrases.truncate(max_phrases_count);
        // 'next_phrase: for phrase in dict.lookup(&syllables, self.lookup_strategy) {
        //     // If there exists a user selected interval which is a
        //     // sub-interval of this phrase but the substring is
        //     // different then we can skip this phrase.
        //     for selection in &com.selections {
        //         debug_assert!(!selection.str.is_empty());
        //         if start <= selection.start && end >= selection.end {
        //             let offset = selection.start - start;
        //             let len = selection.end - selection.start;
        //             let substring: String =
        //                 phrase.as_str().chars().skip(offset).take(len).collect();
        //             if substring != selection.str.as_ref() {
        //                 continue 'next_phrase;
        //             }
        //         }
        //     }

        //     // If there are phrases that can satisfy all the constraints
        //     // then pick the one with highest frequency.
        //     if !(phrase.freq() > max_freq || best_phrase.is_none()) {
        //         continue;
        //     }
        //     // Calculate conditional probability:
        //     //     P(phrase|bopomofo) = count(bopomofo, phrase) / count(bopomofo)
        //     max_freq = phrase.freq();
        //     let log_prob = match global_total {
        //         0.0 => -23.025850929940457, //1e-10_f64.ln()
        //         _ => (max_freq as f64 / global_total).ln(),
        //     };
        //     best_phrase = Some(PossiblePhrase::Phrase(phrase, log_prob));
        // }

        if phrases.is_empty() {
            // try to find if there's a forced selection
            for selection in &com.selections {
                if start == selection.start && end == selection.end {
                    phrases = vec![PossiblePhrase::Phrase(
                        Phrase::new(selection.str.clone(), 0),
                        0.0,
                    )];
                    break;
                }
            }
        }

        trace!("best phraces for {:?} is {:?}", symbols, phrases);
        phrases
    }
    fn find_edges<D: Dictionary + ?Sized>(&self, dict: &D, com: &Composition) -> Vec<Edge> {
        let mut intervals = vec![];
        for start in 0..com.symbols.len() {
            for end in (start + 1)..=com.symbols.len() {
                let phrases = self.find_best_phrases(dict, start, &com.symbols[start..end], com);
                if phrases.is_empty().not() {
                    intervals.push(Edge {
                        start,
                        end,
                        phrases,
                    });
                }
            }
        }
        intervals
    }

    /// Find K possible best alternative paths.
    ///
    /// This method is modified from [Yen's algorithm][1] to run on a single
    /// source and single sink DAG.
    ///
    /// [1]: https://en.wikipedia.org/wiki/Yen%27s_algorithm
    fn find_k_paths(&self, k: usize, len: usize, edges: Vec<Edge>) -> Vec<PossiblePath> {
        let mut ksp = Vec::with_capacity(k);
        let mut candidates = vec![];
        let mut graph = vec![vec![]; len];
        let mut removed_edges = vec![false; len * len];

        for edge in edges.into_iter() {
            graph[edge.start].push(edge);
        }
        for segment in &mut graph {
            segment.sort_by(|a, b| {
                f64::total_cmp(&-a.phrases[0].log_prob(), &-b.phrases[0].log_prob())
            });
        }
        ksp.push(self.shortest_path(&graph, &removed_edges, 0, len).unwrap());

        for kth in 1..k {
            let prev = kth - 1;
            for i in 0..ksp[prev].len() {
                let spur_node = &ksp[prev][i].start;
                let root_path = &ksp[prev][0..i];

                for p in &ksp {
                    if i < p.len() {
                        removed_edges[p[i].start * len + p[i].end - 1] = true;
                    }
                }

                if let Some(spur_path) = self.shortest_path(&graph, &removed_edges, *spur_node, len)
                {
                    let mut total_path = root_path.to_vec();
                    total_path.extend(spur_path);
                    if !ksp.contains(&total_path) {
                        candidates.push(total_path);
                    }
                }
            }
            if candidates.is_empty() {
                break;
            }
            candidates.sort_unstable_by_key(|k| k.len());
            ksp.push(candidates.swap_remove(0));
        }
        // TODO: Reranking
        ksp.into_iter()
            .map(|edges| {
                edges
                    .into_iter()
                    .map(|edge| {
                        let Edge {
                            start,
                            end,
                            phrases,
                        } = edge;
                        let phrase = phrases[0].clone();
                        PossibleInterval { start, end, phrase }
                    })
                    .collect()
            })
            .map(|intervals| PossiblePath { intervals })
            .collect()
    }

    fn shortest_path(
        &self,
        graph: &[Vec<Edge>],
        removed_edges: &[bool],
        source: usize,
        len: usize,
    ) -> Option<Vec<Edge>> {
        let mut parent = vec![None; len + 1];
        let mut queue = VecDeque::new();
        queue.push_back(source);
        'bfs: loop {
            let Some(node) = queue.pop_front() else {
                break;
            };
            if let Some(next_edges) = graph.get(node) {
                for edge in next_edges {
                    if removed_edges[edge.start * len + edge.end - 1] {
                        continue;
                    }
                    if parent[edge.end].is_none() {
                        parent[edge.end] = Some(edge);
                        queue.push_back(edge.end);
                    }
                    if edge.end == len {
                        break 'bfs;
                    }
                }
            }
        }
        let mut path = vec![];
        let mut node = len;
        while node != source {
            let interval = parent[node]?;
            node = interval.start;
            path.push(interval.clone());
        }
        path.reverse();
        Some(path)
    }

    /// Trim some paths that were part of other paths
    ///
    /// Ported from original C implementation, but the original algorithm seems wrong.
    fn trim_paths(&self, paths: Vec<PossiblePath>) -> Vec<PossiblePath> {
        let mut trimmed_paths: Vec<PossiblePath> = vec![];
        for candidate in paths.into_iter() {
            trace!("Trim check {}", candidate);
            let mut drop_candidate = false;
            let mut keeper = vec![];
            for p in trimmed_paths.into_iter() {
                if drop_candidate || p.contains(&candidate) {
                    drop_candidate = true;
                    trace!("  Keep {}", p);
                    keeper.push(p);
                    continue;
                }
                if candidate.contains(&p) {
                    trace!("  Drop {}", p);
                    continue;
                }
                trace!("  Keep {}", p);
                keeper.push(p);
            }
            if !drop_candidate {
                trace!("  Keep {}", candidate);
                keeper.push(candidate);
            }
            trimmed_paths = keeper;
        }
        trimmed_paths
    }
}

#[derive(Debug, Clone, PartialEq)]
enum PossiblePhrase {
    Symbol(Symbol),
    Phrase(Phrase, f64),
}

impl PossiblePhrase {
    fn log_prob(&self) -> f64 {
        match self {
            PossiblePhrase::Symbol(_) => 0.0,
            PossiblePhrase::Phrase(_, log_prob) => *log_prob,
        }
    }
}

impl Display for PossiblePhrase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PossiblePhrase::Symbol(sym) => f.write_char(sym.to_char().unwrap()),
            PossiblePhrase::Phrase(phrase, _) => f.write_str(phrase.as_str()),
        }
    }
}

impl From<PossiblePhrase> for Box<str> {
    fn from(value: PossiblePhrase) -> Self {
        match value {
            PossiblePhrase::Symbol(sym) => sym.to_char().unwrap().to_string().into_boxed_str(),
            PossiblePhrase::Phrase(phrase, _) => phrase.into(),
        }
    }
}

#[derive(Clone, PartialEq)]
struct Edge {
    start: usize,
    end: usize,
    phrases: Vec<PossiblePhrase>,
}

#[derive(Clone, PartialEq)]
struct PossibleInterval {
    start: usize,
    end: usize,
    phrase: PossiblePhrase,
}

impl Debug for PossibleInterval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("I")
            .field(&(self.start..self.end))
            .field(&self.phrase)
            .finish()
    }
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
                PossiblePhrase::Phrase(_, _) => true,
            },
            str: value.phrase.into(),
        }
    }
}

#[derive(Default, Clone)]
struct PossiblePath {
    intervals: Vec<PossibleInterval>,
}

impl Debug for PossiblePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PossiblePath")
            .field("phrase_log_probability()", &self.phrase_log_probability())
            .field("length_log_probability()", &self.length_log_probability())
            .field("total_probability()", &self.total_probability())
            .field("intervals", &self.intervals)
            .finish()
    }
}

impl PossiblePath {
    fn total_probability(&self) -> f64 {
        let prob = self.phrase_log_probability() + self.length_log_probability();
        debug_assert!(!prob.is_nan());
        prob
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

    fn phrase_log_probability(&self) -> f64 {
        self.intervals.iter().map(|it| it.phrase.log_prob()).sum()
    }
    fn length_log_probability(&self) -> f64 {
        self.intervals
            .iter()
            .map(|it| match it.len() {
                // log probability of phrase lenght calculated from tsi.src
                1 => -1.520439227173415,
                2 => -0.4236568120124837,
                3 => -1.455835986003893,
                4 => -1.6178072894679227,
                5 => -4.425765184802149,
                _ => -4.787357595622411,
            })
            .sum()
    }
}

impl PartialEq for PossiblePath {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other).is_eq()
    }
}

impl PartialOrd for PossiblePath {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PossiblePath {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.total_probability()
            .total_cmp(&other.total_probability())
    }
}

impl Eq for PossiblePath {}

impl Display for PossiblePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#PossiblePath({}", self.total_probability())?;
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

#[cfg(test)]
mod tests {
    use crate::{
        conversion::{Composition, Gap, Interval, Symbol, chewing::PossiblePhrase},
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
            (vec![syl![H, A]], vec![("哈", 1)]),
            (vec![syl![H, A], syl![H, A]], vec![("哈哈", 1)]),
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

    // Some corrupted user dictionary may contain empty length syllables
    #[test]
    fn convert_zero_length_entry() {
        let mut dict = test_dictionary();
        let dict_mut = dict.as_dict_mut().unwrap();
        dict_mut.add_phrase(&[], ("", 0).into()).unwrap();
        let engine = ChewingEngine::new();
        let mut composition = Composition::new();
        for sym in [
            Symbol::from(syl![C, E, TONE4]),
            Symbol::from(syl![SH, TONE4]),
        ] {
            composition.push(sym);
        }
        assert_eq!(
            Some(vec![Interval {
                start: 0,
                end: 2,
                is_phrase: true,
                str: "測試".into()
            },]),
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
    fn convert_pathological_case() {
        let dict = test_dictionary();
        let engine = ChewingEngine::new();
        let mut composition = Composition::new();
        for _ in 0..80 {
            composition.push(Symbol::from(syl![H, A]));
        }
        assert_eq!(
            40,
            engine.convert(&dict, &composition).next().unwrap().len()
        );
        assert_eq!(
            41,
            engine.convert(&dict, &composition).nth(1).unwrap().len()
        );
        assert_eq!(
            41,
            engine.convert(&dict, &composition).nth(2).unwrap().len()
        );
    }

    #[test]
    fn possible_path_contains() {
        let path_1 = PossiblePath {
            intervals: vec![
                PossibleInterval {
                    start: 0,
                    end: 2,
                    phrase: PossiblePhrase::Phrase(Phrase::new("測試", 0), 0.0),
                },
                PossibleInterval {
                    start: 2,
                    end: 4,
                    phrase: PossiblePhrase::Phrase(Phrase::new("一下", 0), 0.0),
                },
            ],
        };
        let path_2 = PossiblePath {
            intervals: vec![
                PossibleInterval {
                    start: 0,
                    end: 2,
                    phrase: PossiblePhrase::Phrase(Phrase::new("測試", 0), 0.0),
                },
                PossibleInterval {
                    start: 2,
                    end: 3,
                    phrase: PossiblePhrase::Phrase(Phrase::new("遺", 0), 0.0),
                },
                PossibleInterval {
                    start: 3,
                    end: 4,
                    phrase: PossiblePhrase::Phrase(Phrase::new("下", 0), 0.0),
                },
            ],
        };
        assert!(path_1.contains(&path_2));
    }
}
