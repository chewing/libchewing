use crate::dictionary::{Dictionary, Phrase};

/// Estimates new user phrase frequency.
///
/// Use UserFreqEstimate to keep track of the time passed and use the original
/// frequency and time to calculate the new frequency of user phrases.
pub trait UserFreqEstimate {
    /// Increments the time passed.
    ///
    /// This should be called for every user interaction.
    fn tick(&mut self);
    /// Returns the current time in ticks.
    fn now(&self) -> u64;
    /// Returns the estimated new user phrase frequency.
    fn estimate(&self, phrase: &Phrase, orig_freq: u32, max_freq: u32) -> u32;
}

/// Loosely tracks time without persisting to disk.
#[derive(Debug)]
pub struct LaxUserFreqEstimate {
    lifetime: u64,
}

impl LaxUserFreqEstimate {
    /// Initialize with the last time value from the user dictionary.
    pub fn max_from(user_dict: &dyn Dictionary) -> LaxUserFreqEstimate {
        let lifetime = user_dict
            .entries()
            .map(|it| it.1.last_used().unwrap_or_default())
            .max()
            .unwrap_or_default();
        LaxUserFreqEstimate { lifetime }
    }

    /// Creates a new LaxUserFreqEstimate from a initial epoch.
    pub fn new(initial_lifetime: u64) -> LaxUserFreqEstimate {
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
    fn tick(&mut self) {
        self.lifetime += 1;
    }

    fn now(&self) -> u64 {
        self.lifetime
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
    use crate::{dictionary::TrieBuf, syl};

    use super::LaxUserFreqEstimate;

    #[test]
    fn load_from_dictionary() {
        let user_dict = TrieBuf::from([
            (
                vec![syl![crate::zhuyin::Bopomofo::A]],
                vec![("A", 1, 1), ("B", 1, 2), ("C", 1, 99), ("D", 1, 3)],
            ),
            (
                vec![syl![crate::zhuyin::Bopomofo::I]],
                vec![("I", 1, 5), ("J", 1, 100), ("K", 1, 4), ("L", 1, 3)],
            ),
        ]);
        let estimate = LaxUserFreqEstimate::max_from(&user_dict);

        assert_eq!(100, estimate.lifetime);
    }
}
