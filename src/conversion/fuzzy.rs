use crate::dictionary::LookupStrategy;

use super::{ChewingEngine, ConversionEngine};

/// TODO: doc
#[derive(Debug, Default)]
pub struct FuzzyChewingEngine {
    inner: ChewingEngine,
}

impl FuzzyChewingEngine {
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
