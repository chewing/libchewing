use std::{
    collections::{BinaryHeap, HashSet},
    fmt::{Debug, Display, Write},
};

use tracing::trace;

use super::{Composition, ConversionEngine, Gap, Interval, Outcome, Symbol};
use crate::{
    dictionary::{Dictionary, LookupStrategy, Phrase},
    zhuyin::Syllable,
};

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
    ) -> Vec<Outcome> {
        let paths = {
            if comp.is_empty() {
                return vec![Outcome::default()];
            }
            let (edges, phrases) = self.find_edges(dict, comp);
            if edges.is_empty() {
                return vec![Outcome::default()];
            }
            let mut paths = self.find_k_paths(Self::MAX_OUT_PATHS, comp.len(), edges, &phrases);
            trace!("paths: {:#?}", paths);
            debug_assert!(!paths.is_empty());

            // TODO: Reranking
            paths.sort_by(|a, b| b.cmp(a));
            paths
        };
        paths
            .into_iter()
            .map(|p| {
                let log_prob = p.total_probability();
                let intervals = p
                    .intervals
                    .into_iter()
                    .map(|it| it.into())
                    .fold(vec![], |acc, interval| glue_fn(comp, acc, interval));
                Outcome {
                    intervals,
                    log_prob,
                }
            })
            .collect()
    }
}

impl ConversionEngine for ChewingEngine {
    fn convert<'a>(&'a self, dict: &'a dyn Dictionary, comp: &'a Composition) -> Vec<Outcome> {
        ChewingEngine::convert(self, dict, comp)
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
        let mut phrase = last.text.into_string();
        phrase.push_str(&interval.text);
        acc.push(Interval {
            start: last.start,
            end: interval.end,
            is_phrase: true,
            text: phrase.into_boxed_str(),
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

        if symbols.len() == 1
            && let Some(sym) = symbols[0].to_char()
        {
            return vec![PossiblePhrase::Symbol(sym)];
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
                    debug_assert!(!selection.text.is_empty());
                    if start <= selection.start && end >= selection.end {
                        let offset = selection.start - start;
                        let len = selection.end - selection.start;
                        let substring: String =
                            phrase.as_str().chars().skip(offset).take(len).collect();
                        if substring != selection.text.as_ref() {
                            return false;
                        }
                    }
                }
                true
            })
            .map(|phrase| {
                let log_phrase_prob = (phrase.freq().clamp(1, 9999999) as f64 / global_total).ln();
                let log_length_prob: f64 = match syllables.len() {
                    // log probability of phrase lenght calculated from tsi.src
                    1 => -1.520439227173415,
                    2 => -0.4236568120124837,
                    3 => -1.455835986003893,
                    4 => -1.6178072894679227,
                    5 => -4.425765184802149,
                    _ => -4.787357595622411,
                };
                let log_prob = log_phrase_prob + log_length_prob;
                debug_assert!(log_prob.is_normal());
                PossiblePhrase::Phrase(phrase, log_prob)
            })
            .collect::<Vec<_>>();
        phrases.sort_by(|a, b| f64::total_cmp(&-a.log_prob(), &-b.log_prob()));
        phrases.truncate(max_phrases_count);

        if phrases.is_empty() {
            // try to find if there's a forced selection
            for selection in &com.selections {
                if start == selection.start && end == selection.end {
                    phrases = vec![PossiblePhrase::Phrase(
                        Phrase::new(selection.text.clone(), 0),
                        0.0,
                    )];
                    break;
                }
            }
        }

        trace!("best phraces for {:?} is {:?}", symbols, phrases);
        phrases
    }
    fn find_edges<D: Dictionary + ?Sized>(
        &self,
        dict: &D,
        com: &Composition,
    ) -> (Vec<Edge>, Vec<PossiblePhrase>) {
        let mut sn = 0;
        let mut edges = vec![];
        let mut phrases = vec![];
        for start in 0..com.symbols.len() {
            for end in (start + 1)..=com.symbols.len() {
                for phrase in self.find_best_phrases(dict, start, &com.symbols[start..end], com) {
                    edges.push(Edge {
                        start,
                        end,
                        sn,
                        cost: -phrase.log_prob(),
                    });
                    phrases.push(phrase);
                    sn += 1;
                }
            }
        }
        (edges, phrases)
    }

    /// Find K possible best alternative paths.
    ///
    /// This method is modified from [Yen's algorithm][1] to run on a single
    /// source and single sink DAG.
    ///
    /// [1]: https://en.wikipedia.org/wiki/Yen%27s_algorithm
    fn find_k_paths(
        &self,
        k: usize,
        len: usize,
        edges: Vec<Edge>,
        phrases: &[PossiblePhrase],
    ) -> Vec<PossiblePath> {
        let mut ksp = Vec::with_capacity(k);
        let mut candidates = vec![];
        let mut graph = vec![vec![]; len];
        let mut removed_edges = HashSet::new();

        for edge in edges.into_iter() {
            graph[edge.start].push(edge);
        }
        ksp.push(self.shortest_path(&graph, &removed_edges, 0, len).unwrap());

        for kth in 1..k {
            let prev = kth - 1;
            for i in 0..ksp[prev].len() {
                let spur_node = &ksp[prev][i].start;
                let root_path = &ksp[prev][0..i];

                for p in &ksp {
                    if i < p.len() {
                        removed_edges.insert(p[i].sn);
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
        ksp.into_iter()
            .map(|edges| {
                edges
                    .into_iter()
                    .map(|edge| {
                        let Edge {
                            start,
                            end,
                            sn,
                            cost: _,
                        } = edge;
                        let phrase = phrases[sn].clone();
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
        removed_edges: &HashSet<usize>,
        source: usize,
        goal: usize,
    ) -> Option<Vec<Edge>> {
        let mut back_link: Vec<Option<Edge>> = vec![None; goal + 1];
        let mut frontier = BinaryHeap::new();
        frontier.push(FrontierNode {
            position: source,
            cost: 0.0,
        });
        while let Some(FrontierNode { position, cost }) = frontier.pop() {
            if position == goal {
                break;
            }
            if back_link[position].is_some_and(|prev| cost > prev.cost) {
                continue;
            }
            if let Some(neighbor_edges) = graph.get(position) {
                for edge in neighbor_edges {
                    if removed_edges.contains(&edge.sn) {
                        continue;
                    }
                    let alt = FrontierNode {
                        position: edge.end,
                        cost: cost + edge.cost,
                    };
                    if back_link[alt.position].is_none_or(|prev| alt.cost < prev.cost) {
                        back_link[alt.position] = Some(Edge {
                            cost: alt.cost,
                            ..*edge
                        });
                        frontier.push(alt);
                    }
                }
            }
        }
        let mut path = vec![];
        let mut node = goal;
        while node != source {
            let edge = back_link[node]?;
            node = edge.start;
            path.push(edge);
        }
        path.reverse();
        Some(path)
    }
}

#[derive(Debug, Clone, PartialEq)]
enum PossiblePhrase {
    Symbol(char),
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
            PossiblePhrase::Symbol(sym) => f.write_char(*sym),
            PossiblePhrase::Phrase(phrase, _) => f.write_str(phrase.as_str()),
        }
    }
}

impl From<PossiblePhrase> for Box<str> {
    fn from(value: PossiblePhrase) -> Self {
        match value {
            PossiblePhrase::Symbol(sym) => sym.to_string().into_boxed_str(),
            PossiblePhrase::Phrase(phrase, _) => phrase.into(),
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct FrontierNode {
    position: usize,
    cost: f64,
}

impl Ord for FrontierNode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.cost.total_cmp(&self.cost)
    }
}

impl PartialOrd for FrontierNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for FrontierNode {}

impl PartialEq for FrontierNode {
    fn eq(&self, other: &Self) -> bool {
        self.position == other.position
    }
}

#[derive(Debug, Copy, Clone)]
struct Edge {
    start: usize,
    end: usize,
    sn: usize,
    cost: f64,
}

impl PartialEq for Edge {
    fn eq(&self, other: &Self) -> bool {
        self.sn == other.sn
    }
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

impl From<PossibleInterval> for Interval {
    fn from(value: PossibleInterval) -> Self {
        Interval {
            start: value.start,
            end: value.end,
            is_phrase: match value.phrase {
                PossiblePhrase::Symbol(_) => false,
                PossiblePhrase::Phrase(_, _) => true,
            },
            text: value.phrase.into(),
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
            .field("total_probability()", &self.total_probability())
            .field("intervals", &self.intervals)
            .finish()
    }
}

impl PossiblePath {
    fn total_probability(&self) -> f64 {
        let prob = self.phrase_log_probability();
        debug_assert!(!prob.is_nan());
        prob
    }
    fn phrase_log_probability(&self) -> f64 {
        self.intervals.iter().map(|it| it.phrase.log_prob()).sum()
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
    use std::collections::HashSet;

    use super::ChewingEngine;
    use crate::{
        conversion::{Composition, Gap, Interval, Outcome, Symbol, chewing::Edge},
        dictionary::{Dictionary, TrieBuf},
        syl,
        zhuyin::Bopomofo::*,
    };

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
    fn simple_shortest_path() {
        let engine = ChewingEngine::new();
        let graph = vec![
            vec![
                Edge {
                    start: 0,
                    end: 1,
                    sn: 0,
                    cost: 1.0,
                },
                Edge {
                    start: 0,
                    end: 2,
                    sn: 2,
                    cost: 3.0,
                },
            ],
            vec![Edge {
                start: 1,
                end: 2,
                sn: 1,
                cost: 1.0,
            }],
        ];
        let removed_edges = HashSet::new();

        assert_eq!(
            Some(vec![
                Edge {
                    start: 0,
                    end: 1,
                    sn: 0,
                    cost: 1.0,
                },
                Edge {
                    start: 1,
                    end: 2,
                    sn: 1,
                    cost: 1.0,
                }
            ]),
            engine.shortest_path(&graph, &removed_edges, 0, 2)
        );
    }

    #[test]
    fn multi_edge_shortest_path() {
        let engine = ChewingEngine::new();
        let graph = vec![
            vec![
                Edge {
                    start: 0,
                    end: 1,
                    sn: 0,
                    cost: 1.0,
                },
                Edge {
                    start: 0,
                    end: 1,
                    sn: 3,
                    cost: 2.0,
                },
                Edge {
                    start: 0,
                    end: 2,
                    sn: 2,
                    cost: 3.0,
                },
            ],
            vec![Edge {
                start: 1,
                end: 2,
                sn: 1,
                cost: 1.0,
            }],
        ];
        let removed_edges = HashSet::new();

        assert_eq!(
            Some(vec![
                Edge {
                    start: 0,
                    end: 1,
                    sn: 0,
                    cost: 1.0,
                },
                Edge {
                    start: 1,
                    end: 2,
                    sn: 1,
                    cost: 1.0,
                }
            ]),
            engine.shortest_path(&graph, &removed_edges, 0, 2)
        );
    }

    #[test]
    fn convert_empty_composition() {
        let dict = test_dictionary();
        let engine = ChewingEngine::new();
        let composition = Composition::new();
        assert_eq!(
            vec![Outcome::default()],
            engine.convert(&dict, &composition)
        );
    }

    // Some corrupted user dictionary may contain empty length syllables
    #[test]
    fn convert_zero_length_entry() {
        let mut dict = test_dictionary();
        dict.add_phrase(&[], ("", 0).into()).unwrap();
        let engine = ChewingEngine::new();
        let mut composition = Composition::new();
        for sym in [
            Symbol::from(syl![C, E, TONE4]),
            Symbol::from(syl![SH, TONE4]),
        ] {
            composition.push(sym);
        }
        assert_eq!(
            vec![Interval {
                start: 0,
                end: 2,
                is_phrase: true,
                text: "測試".into()
            }],
            engine.convert(&dict, &composition)[0].intervals
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
            vec![
                Interval {
                    start: 0,
                    end: 2,
                    is_phrase: true,
                    text: "國民".into()
                },
                Interval {
                    start: 2,
                    end: 4,
                    is_phrase: true,
                    text: "大會".into()
                },
                Interval {
                    start: 4,
                    end: 6,
                    is_phrase: true,
                    text: "代表".into()
                },
            ],
            engine.convert(&dict, &composition)[0].intervals
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
            vec![
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
                    end: 4,
                    is_phrase: true,
                    text: "大會".into()
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
            engine.convert(&dict, &composition)[0].intervals
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
            text: "戴錶".into(),
        });
        assert_eq!(
            vec![
                Interval {
                    start: 0,
                    end: 2,
                    is_phrase: true,
                    text: "國民".into()
                },
                Interval {
                    start: 2,
                    end: 4,
                    is_phrase: true,
                    text: "大會".into()
                },
                Interval {
                    start: 4,
                    end: 6,
                    is_phrase: true,
                    text: "戴錶".into()
                },
            ],
            engine.convert(&dict, &composition)[0].intervals
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
            text: "酷音".into(),
        });
        assert_eq!(
            vec![Interval {
                start: 0,
                end: 3,
                is_phrase: true,
                text: "新酷音".into()
            }],
            engine.convert(&dict, &composition)[0].intervals
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
                text: "代".into(),
            },
            Interval {
                start: 1,
                end: 2,
                is_phrase: true,
                text: "錶".into(),
            },
        ] {
            composition.push_selection(interval);
        }
        assert_eq!(
            vec![
                Interval {
                    start: 0,
                    end: 1,
                    is_phrase: true,
                    text: "代".into()
                },
                Interval {
                    start: 1,
                    end: 2,
                    is_phrase: true,
                    text: "錶".into()
                }
            ],
            engine.convert(&dict, &composition)[0].intervals
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
            vec![
                Interval {
                    start: 0,
                    end: 2,
                    is_phrase: true,
                    text: "測試".into()
                },
                Interval {
                    start: 2,
                    end: 4,
                    is_phrase: true,
                    text: "一下".into()
                }
            ],
            engine.convert(&dict, &composition)[0].intervals
        );
        assert_eq!(
            vec![
                Interval {
                    start: 0,
                    end: 3,
                    is_phrase: true,
                    text: "測試儀".into()
                },
                Interval {
                    start: 3,
                    end: 4,
                    is_phrase: true,
                    text: "下".into()
                }
            ],
            engine.convert(&dict, &composition)[1].intervals
        );
        assert_eq!(
            Some(vec![
                Interval {
                    start: 0,
                    end: 2,
                    is_phrase: true,
                    text: "測試".into()
                },
                Interval {
                    start: 2,
                    end: 4,
                    is_phrase: true,
                    text: "一下".into()
                }
            ]),
            engine
                .convert(&dict, &composition)
                .into_iter()
                .cycle()
                .nth(2)
                .map(|p| p.intervals)
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
        assert_eq!(40, engine.convert(&dict, &composition)[0].intervals.len());
        assert_eq!(41, engine.convert(&dict, &composition)[1].intervals.len());
        assert_eq!(41, engine.convert(&dict, &composition)[2].intervals.len());
    }
}
