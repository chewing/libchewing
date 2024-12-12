use std::{
    collections::VecDeque,
    fmt::{Debug, Display, Write},
    iter,
    ops::{Mul, Neg},
};

use log::trace;

use crate::dictionary::{Dictionary, LookupStrategy, Phrase};

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
            let intervals = self.find_intervals(dict, comp);
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
                trace!("No best phrase for {:?} due to break point", symbols);
                return None;
            }
        }

        for selection in &com.selections {
            if selection.intersect_range(start, end) && !selection.is_contained_by(start, end) {
                // There's a conflicting partial intersecting selection.
                trace!(
                    "No best phrase for {:?} due to selection {:?}",
                    symbols,
                    selection
                );
                return None;
            }
        }

        if symbols.len() == 1 && symbols[0].is_char() {
            return Some(symbols[0].into());
        }

        if symbols.iter().any(|sym| sym.is_char()) {
            return None;
        }

        let mut max_freq = 0;
        let mut best_phrase = None;
        'next_phrase: for phrase in dict.lookup_all_phrases(&symbols, self.lookup_strategy) {
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

        if best_phrase.is_none() {
            // try to find if there's a forced selection
            for selection in &com.selections {
                if start == selection.start && end == selection.end {
                    best_phrase = Some(Phrase::new(selection.str.clone(), 0).into());
                    break;
                }
            }
        }

        trace!("best phrace for {:?} is {:?}", symbols, best_phrase);
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
        intervals: Vec<PossibleInterval>,
    ) -> Vec<PossiblePath> {
        let mut ksp = Vec::with_capacity(k);
        let mut candidates = vec![];
        let mut graph = vec![vec![]; len];
        let mut removed_edges = vec![false; len * len];

        for edge in intervals.into_iter() {
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
        ksp.into_iter()
            .map(|intervals| PossiblePath { intervals })
            .collect()
    }

    fn shortest_path(
        &self,
        graph: &[Vec<PossibleInterval>],
        removed_edges: &[bool],
        source: usize,
        len: usize,
    ) -> Option<Vec<PossibleInterval>> {
        let mut parent = vec![None; len + 1];
        let mut queue = VecDeque::new();
        queue.push_back(source);
        'bfs: while !queue.is_empty() {
            let node = queue.pop_front().unwrap();
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

#[derive(Clone, PartialEq, Eq)]
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
            .field("rule_largest_sum()", &self.rule_largest_sum().mul(1000))
            .field(
                "rule_largest_avgwordlen()",
                &self.rule_largest_avgwordlen().mul(1000),
            )
            .field(
                "rule_smallest_lenvariance()",
                &self.rule_smallest_lenvariance().mul(100),
            )
            .field("rule_largest_freqsum()", &self.rule_largest_freqsum())
            .field("total_score()", &self.score())
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
