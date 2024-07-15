use crate::dictionary::LookupStrategy;

use super::{ChewingEngine, ConversionEngine};

/// Same conversion method as Chewing but uses fuzzy phrase search.
#[derive(Debug, Default)]
pub struct FuzzyChewingEngine {
    inner: ChewingEngine,
}

impl FuzzyChewingEngine {
    /// Creates a new conversion engine.
    pub fn new() -> FuzzyChewingEngine {
        FuzzyChewingEngine {
            inner: ChewingEngine {
                lookup_strategy: LookupStrategy::FuzzyPartialPrefix,
            },
        }
    }
}

impl ConversionEngine for FuzzyChewingEngine {
    fn convert<'a>(
        &'a self,
        dict: &'a dyn crate::dictionary::Dictionary,
        comp: &'a super::Composition,
    ) -> Box<dyn Iterator<Item = Vec<super::Interval>> + 'a> {
        Box::new(ChewingEngine::convert(&self.inner, dict, comp))
    }
}
