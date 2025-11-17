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
    fn estimate(&self, phrase: &Phrase, max_freq: u32) -> u32;
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
const LONG_INCREASE_FREQ: u32 = 1;
const MAX_USER_FREQ: u32 = 9999999;

impl UserFreqEstimate for LaxUserFreqEstimate {
    fn tick(&mut self) {
        self.lifetime += 1;
    }

    fn now(&self) -> u64 {
        self.lifetime
    }

    fn estimate(&self, phrase: &Phrase, max_freq: u32) -> u32 {
        let last_used = phrase.last_used().unwrap_or(self.lifetime);
        let delta_time = if self.lifetime >= last_used {
            self.lifetime - last_used
        } else {
            u64::MAX
        };

        // # New phrase
        //
        // If the phrase is added to the user dictionary the first time,
        // always bump it to the highest frequency so new phrases are prefered.
        //
        // # Seen phrase
        //
        // If the phrase was seen recently, then bump the frequency faster. Otherwise
        // use smaller increase factor to avoid rarely used phrase suddenly become
        // more preferred.
        if delta_time == 0 {
            (max_freq + SHORT_INCREASE_FREQ).min(MAX_USER_FREQ)
        } else if delta_time < 4000 {
            (phrase.freq() + SHORT_INCREASE_FREQ).min(MAX_USER_FREQ)
        } else if delta_time < 50000 {
            (phrase.freq() + MEDIUM_INCREASE_FREQ).min(MAX_USER_FREQ)
        } else {
            (phrase.freq() + LONG_INCREASE_FREQ).min(MAX_USER_FREQ)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{dictionary::TrieBuf, editor::UserFreqEstimate, syl};

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

    #[test]
    fn estimate_first_use_phrase() {
        let est = LaxUserFreqEstimate::new(1000);
        let phrase = ("不", 0).into();
        assert_eq!(910, est.estimate(&phrase, 900));
    }

    #[test]
    fn estimate_second_use_phrase() {
        let est = LaxUserFreqEstimate::new(1000);
        let phrase = ("不", 50, 10).into();
        assert_eq!(60, est.estimate(&phrase, 900));
    }

    #[test]
    fn estimate_long_unused_phrase() {
        let est = LaxUserFreqEstimate::new(5000);
        let phrase = ("不", 50, 10).into();
        assert_eq!(55, est.estimate(&phrase, 900));
    }

    #[test]
    fn estimate_long_long_unused_phrase() {
        let est = LaxUserFreqEstimate::new(60000);
        let phrase = ("不", 50, 10).into();
        assert_eq!(51, est.estimate(&phrase, 900));
    }

    #[test]
    fn estimate_out_of_sync_lifetime() {
        let est = LaxUserFreqEstimate::new(100);
        let phrase = ("不", 50, 1000).into();
        assert_eq!(51, est.estimate(&phrase, 900));
    }
}
