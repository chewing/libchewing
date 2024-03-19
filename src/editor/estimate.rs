use std::{error::Error, fmt::Display};

use crate::dictionary::{Dictionary, Phrase};

/// TODO: doc
/// TODO: change to enum?
#[derive(Debug)]
pub struct EstimateError {
    source: Box<dyn Error>,
}

impl Display for EstimateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "update estimate error")
    }
}

impl Error for EstimateError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.source.as_ref())
    }
}

/// TODO: doc
pub trait UserFreqEstimate {
    /// TODO: doc
    fn tick(&mut self) -> Result<(), EstimateError>;
    /// TODO: doc
    fn now(&self) -> Result<u64, EstimateError>;
    /// TODO: doc
    fn estimate(&self, phrase: &Phrase, orig_freq: u32, max_freq: u32) -> u32;
}

/// TODO: doc
#[derive(Debug)]
pub struct LaxUserFreqEstimate {
    lifetime: u64,
}

impl LaxUserFreqEstimate {
    /// TODO: doc
    pub fn open(user_dict: &dyn Dictionary) -> Result<LaxUserFreqEstimate, EstimateError> {
        let lifetime = user_dict
            .entries()
            .map(|it| it.1.last_used().unwrap_or_default())
            .max()
            .unwrap_or_default();
        Ok(LaxUserFreqEstimate { lifetime })
    }

    pub fn open_in_memory(initial_lifetime: u64) -> LaxUserFreqEstimate {
        LaxUserFreqEstimate {
            lifetime: initial_lifetime,
        }
    }
}

const SHORT_INCREASE_FREQ: u32 = 10;
const MEDIUM_INCREASE_FREQ: u32 = 5;
const LONG_DECREASE_FREQ: u32 = 10;
const MAX_USER_FREQ: u32 = 99999999;

impl UserFreqEstimate for LaxUserFreqEstimate {
    fn tick(&mut self) -> Result<(), EstimateError> {
        self.lifetime += 1;
        Ok(())
    }

    fn now(&self) -> Result<u64, EstimateError> {
        Ok(self.lifetime)
    }

    fn estimate(&self, phrase: &Phrase, orig_freq: u32, max_freq: u32) -> u32 {
        let delta_time = self.lifetime - phrase.last_used().unwrap_or(self.lifetime);

        if delta_time < 4000 {
            let delta = if phrase.freq() >= max_freq {
                ((max_freq - orig_freq) / 5 + 1).min(SHORT_INCREASE_FREQ)
            } else {
                ((max_freq - orig_freq) / 5 + 1).max(SHORT_INCREASE_FREQ)
            };
            (phrase.freq() + delta).min(MAX_USER_FREQ)
        } else if delta_time < 50000 {
            let delta = if phrase.freq() >= max_freq {
                ((max_freq - orig_freq) / 10 + 1).min(MEDIUM_INCREASE_FREQ)
            } else {
                ((max_freq - orig_freq) / 10 + 1).max(MEDIUM_INCREASE_FREQ)
            };
            (phrase.freq() + delta).min(MAX_USER_FREQ)
        } else {
            let delta = ((phrase.freq() - orig_freq) / 5).max(LONG_DECREASE_FREQ);
            (phrase.freq() - delta).max(orig_freq)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{dictionary::KVDictionary, syl};

    use super::LaxUserFreqEstimate;

    #[test]
    fn load_from_dictionary() {
        let user_dict = KVDictionary::from([
            (
                vec![syl![crate::zhuyin::Bopomofo::A]],
                vec![
                    ("A", 1, 1).into(),
                    ("B", 1, 2).into(),
                    ("C", 1, 99).into(),
                    ("D", 1, 3).into(),
                ],
            ),
            (
                vec![syl![crate::zhuyin::Bopomofo::I]],
                vec![
                    ("I", 1, 5).into(),
                    ("J", 1, 100).into(),
                    ("K", 1, 4).into(),
                    ("L", 1, 3).into(),
                ],
            ),
        ]);
        let estimate = LaxUserFreqEstimate::open(&user_dict).unwrap();

        assert_eq!(100, estimate.lifetime);
    }
}
