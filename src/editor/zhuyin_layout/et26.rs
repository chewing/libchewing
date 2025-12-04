//! ET26 (倚天26鍵)

use super::{KeyBehavior, SyllableEditor};
use crate::{
    input::{
        KeyboardEvent,
        keysym::{Keysym, SYM_LOWER_D, SYM_LOWER_F, SYM_LOWER_J, SYM_LOWER_K, SYM_SPACE},
    },
    syl,
    zhuyin::{Bopomofo, BopomofoKind, Syllable},
};

/// TODO: docs
#[derive(Debug, Clone, Copy)]
pub struct Et26 {
    syllable: Syllable,
}

impl Et26 {
    /// TODO: docs
    pub fn new() -> Et26 {
        Et26 {
            syllable: Default::default(),
        }
    }
    fn is_end_key(&self, key: Keysym) -> bool {
        match key {
            SYM_LOWER_D | SYM_LOWER_F | SYM_LOWER_J | SYM_LOWER_K | SYM_SPACE => {
                !self.syllable.is_empty()
            }
            _ => false,
        }
    }
    fn has_initial_or_medial(&self) -> bool {
        self.syllable.has_initial() || self.syllable.has_medial()
    }

    const ALT_TABLE: &'static [(Syllable, &'static [Syllable])] = &[
        (syl![Bopomofo::OU], &[syl![Bopomofo::P]]),
        (syl![Bopomofo::ANG], &[syl![Bopomofo::T]]),
        (syl![Bopomofo::C], &[syl![Bopomofo::EH]]),
        (syl![Bopomofo::Z], &[syl![Bopomofo::EI]]),
        (syl![Bopomofo::ZH], &[syl![Bopomofo::J]]),
        (syl![Bopomofo::ER], &[syl![Bopomofo::H]]),
        (syl![Bopomofo::ENG], &[syl![Bopomofo::L]]),
        (syl![Bopomofo::SH], &[syl![Bopomofo::X]]),
        (syl![Bopomofo::G], &[syl![Bopomofo::Q]]),
        (syl![Bopomofo::EN], &[syl![Bopomofo::N]]),
        (syl![Bopomofo::AN], &[syl![Bopomofo::M]]),
        (syl![Bopomofo::D], &[syl![Bopomofo::TONE5]]),
        (syl![Bopomofo::F], &[syl![Bopomofo::TONE2]]),
        (syl![Bopomofo::R], &[syl![Bopomofo::TONE3]]),
        (syl![Bopomofo::K], &[syl![Bopomofo::TONE4]]),
    ];
}

impl Default for Et26 {
    fn default() -> Self {
        Self::new()
    }
}

impl SyllableEditor for Et26 {
    fn key_press(&mut self, key: KeyboardEvent) -> KeyBehavior {
        if self.is_end_key(key.ksym) {
            if !self.syllable.has_medial() && !self.syllable.has_rime() {
                match self.syllable.initial() {
                    Some(Bopomofo::J) => {
                        self.syllable.update(Bopomofo::ZH);
                    }
                    Some(Bopomofo::X) => {
                        self.syllable.update(Bopomofo::SH);
                    }
                    Some(Bopomofo::P) => {
                        self.syllable.remove_initial();
                        self.syllable.update(Bopomofo::OU);
                    }
                    Some(Bopomofo::M) => {
                        self.syllable.remove_initial();
                        self.syllable.update(Bopomofo::AN);
                    }
                    Some(Bopomofo::N) => {
                        self.syllable.remove_initial();
                        self.syllable.update(Bopomofo::EN);
                    }
                    Some(Bopomofo::T) => {
                        self.syllable.remove_initial();
                        self.syllable.update(Bopomofo::ANG);
                    }
                    Some(Bopomofo::L) => {
                        self.syllable.remove_initial();
                        self.syllable.update(Bopomofo::ENG);
                    }
                    Some(Bopomofo::H) => {
                        self.syllable.remove_initial();
                        self.syllable.update(Bopomofo::ER);
                    }
                    _ => (),
                }
            }
            match key.ksym.to_unicode() {
                // KeyCode::Space => Some(Bopomofo::TONE1),
                'f' => self.syllable.update(Bopomofo::TONE2),
                'j' => self.syllable.update(Bopomofo::TONE3),
                'k' => self.syllable.update(Bopomofo::TONE4),
                'd' => self.syllable.update(Bopomofo::TONE5),
                _ => {
                    self.syllable.remove_tone();
                }
            };
            KeyBehavior::Commit
        } else {
            let bopomofo = match key.ksym.to_unicode() {
                'a' => Bopomofo::A,
                'b' => Bopomofo::B,
                'c' => Bopomofo::X,
                'd' => Bopomofo::D,
                'e' => Bopomofo::I,
                'f' => Bopomofo::F,
                'g' => Bopomofo::J,
                'h' => {
                    if self.has_initial_or_medial() {
                        Bopomofo::ER
                    } else {
                        Bopomofo::H
                    }
                }
                'i' => Bopomofo::AI,
                'j' => Bopomofo::R,
                'k' => Bopomofo::K,
                'l' => {
                    if self.has_initial_or_medial() {
                        Bopomofo::ENG
                    } else {
                        Bopomofo::L
                    }
                }
                'm' => {
                    if self.has_initial_or_medial() {
                        Bopomofo::AN
                    } else {
                        Bopomofo::M
                    }
                }
                'n' => {
                    if self.has_initial_or_medial() {
                        Bopomofo::EN
                    } else {
                        Bopomofo::N
                    }
                }
                'o' => Bopomofo::O,
                'p' => {
                    if self.has_initial_or_medial() {
                        Bopomofo::OU
                    } else {
                        Bopomofo::P
                    }
                }
                'q' => {
                    if self.has_initial_or_medial() {
                        Bopomofo::EI
                    } else {
                        Bopomofo::Z
                    }
                }
                'r' => Bopomofo::E,
                's' => Bopomofo::S,
                't' => {
                    if self.has_initial_or_medial() {
                        Bopomofo::ANG
                    } else {
                        Bopomofo::T
                    }
                }
                'u' => Bopomofo::IU,
                'v' => Bopomofo::G,
                'w' => {
                    if self.has_initial_or_medial() {
                        Bopomofo::EH
                    } else {
                        Bopomofo::C
                    }
                }
                'x' => Bopomofo::U,
                'y' => Bopomofo::CH,
                'z' => Bopomofo::AU,
                _ => return KeyBehavior::NoWord,
            };

            match bopomofo.kind() {
                BopomofoKind::Medial => {
                    if bopomofo == Bopomofo::U {
                        match self.syllable.initial() {
                            Some(Bopomofo::J) => {
                                self.syllable.update(Bopomofo::ZH);
                            }
                            Some(Bopomofo::X) => {
                                self.syllable.update(Bopomofo::SH);
                            }
                            _ => (),
                        }
                    } else if let Some(Bopomofo::G) = self.syllable.initial() {
                        self.syllable.update(Bopomofo::Q);
                    }
                }
                BopomofoKind::Rime if !self.syllable.has_medial() => {
                    match self.syllable.initial() {
                        Some(Bopomofo::J) => {
                            self.syllable.update(Bopomofo::ZH);
                        }
                        Some(Bopomofo::X) => {
                            self.syllable.update(Bopomofo::SH);
                        }
                        _ => (),
                    };
                }
                _ => (),
            };

            self.syllable.update(bopomofo);
            KeyBehavior::Absorb
        }
    }

    fn is_empty(&self) -> bool {
        self.syllable.is_empty()
    }

    fn remove_last(&mut self) {
        self.syllable.pop();
    }

    fn clear(&mut self) {
        self.syllable.clear();
    }

    fn read(&self) -> Syllable {
        self.syllable
    }

    fn alt_syllables(&self, syl: Syllable) -> &[Syllable] {
        for entry in Self::ALT_TABLE {
            if entry.0 == syl {
                return entry.1;
            }
        }
        &[]
    }

    fn clone(&self) -> Box<dyn SyllableEditor> {
        Box::new(Clone::clone(self))
    }
}
