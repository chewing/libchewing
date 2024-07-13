//! Dai Chien CP26

use crate::{
    editor::keyboard::{KeyEvent, KeyIndex},
    zhuyin::{Bopomofo, Syllable},
};

use super::{KeyBehavior, SyllableEditor};

/// TODO: docs
#[derive(Debug, Clone, Copy)]
pub struct DaiChien26 {
    syllable: Syllable,
}

impl DaiChien26 {
    /// TODO: docs
    pub fn new() -> DaiChien26 {
        DaiChien26 {
            syllable: Default::default(),
        }
    }
    fn is_end_key(&self, key: KeyIndex) -> bool {
        match key {
            KeyIndex::K17 | KeyIndex::K18 | KeyIndex::K29 | KeyIndex::K20 | KeyIndex::K48 => {
                !self.syllable.is_empty()
            }
            _ => false,
        }
    }
    fn has_initial_or_medial(&self) -> bool {
        self.syllable.has_initial() || self.syllable.has_medial()
    }
}

impl Default for DaiChien26 {
    fn default() -> Self {
        Self::new()
    }
}

fn default_or_alt(source: Option<Bopomofo>, default: Bopomofo, alt: Bopomofo) -> Bopomofo {
    match source {
        None => default,
        Some(src) => {
            if src == default {
                alt
            } else {
                default
            }
        }
    }
}

impl SyllableEditor for DaiChien26 {
    fn key_press(&mut self, key: KeyEvent) -> KeyBehavior {
        if self.is_end_key(key.index) {
            match key.index {
                // KeyIndex::K48 => Some(Bopomofo::TONE1),
                KeyIndex::K17 => self.syllable.update(Bopomofo::TONE2),
                KeyIndex::K18 => self.syllable.update(Bopomofo::TONE3),
                KeyIndex::K29 => self.syllable.update(Bopomofo::TONE4),
                KeyIndex::K20 => self.syllable.update(Bopomofo::TONE5),
                _ => {
                    self.syllable.remove_tone();
                }
            };
            return KeyBehavior::Commit;
        }
        let bopomofo = match key.index {
            KeyIndex::K15 => default_or_alt(self.syllable.initial(), Bopomofo::B, Bopomofo::P),
            KeyIndex::K27 => Bopomofo::M,
            KeyIndex::K38 => Bopomofo::F,
            KeyIndex::K16 => default_or_alt(self.syllable.initial(), Bopomofo::D, Bopomofo::T),
            KeyIndex::K28 => Bopomofo::N,
            KeyIndex::K39 => Bopomofo::L,
            KeyIndex::K17 => Bopomofo::G,
            KeyIndex::K29 => Bopomofo::K,
            KeyIndex::K40 => Bopomofo::H,
            KeyIndex::K18 => Bopomofo::J,
            KeyIndex::K30 => Bopomofo::Q,
            KeyIndex::K41 => Bopomofo::X,
            KeyIndex::K19 => default_or_alt(self.syllable.initial(), Bopomofo::ZH, Bopomofo::CH),
            KeyIndex::K31 => Bopomofo::SH,
            KeyIndex::K42 => {
                if self.has_initial_or_medial() {
                    Bopomofo::EH
                } else {
                    Bopomofo::R
                }
            }
            KeyIndex::K20 => Bopomofo::Z,
            KeyIndex::K32 => Bopomofo::C,
            KeyIndex::K43 => {
                if self.has_initial_or_medial() {
                    Bopomofo::ENG
                } else {
                    Bopomofo::S
                }
            }
            KeyIndex::K21 => {
                match (self.syllable.medial(), self.syllable.rime()) {
                    (Some(Bopomofo::I), Some(Bopomofo::A)) => {
                        self.syllable.remove_medial();
                        self.syllable.remove_rime();
                        return KeyBehavior::Absorb;
                    }
                    (_, Some(Bopomofo::A)) => {
                        self.syllable.update(Bopomofo::I);
                        return KeyBehavior::Absorb;
                    }
                    (Some(Bopomofo::I), _) => {
                        self.syllable.remove_medial();
                        self.syllable.update(Bopomofo::A);
                        return KeyBehavior::Absorb;
                    }
                    (Some(_), _) => {
                        self.syllable.update(Bopomofo::A);
                        return KeyBehavior::Absorb;
                    }
                    _ => (),
                }
                Bopomofo::I
            }
            KeyIndex::K33 => Bopomofo::U,
            KeyIndex::K44 => {
                match (self.syllable.medial(), self.syllable.rime()) {
                    (Some(Bopomofo::IU), None) => {
                        self.syllable.remove_medial();
                        self.syllable.update(Bopomofo::OU);
                        return KeyBehavior::Absorb;
                    }
                    (Some(Bopomofo::IU), Some(f)) if f != Bopomofo::OU => {
                        self.syllable.remove_medial();
                        self.syllable.update(Bopomofo::OU);
                        return KeyBehavior::Absorb;
                    }
                    (None, Some(Bopomofo::OU)) => {
                        self.syllable.update(Bopomofo::IU);
                        self.syllable.remove_rime();
                        return KeyBehavior::Absorb;
                    }
                    (Some(f), Some(Bopomofo::OU)) if f != Bopomofo::IU => {
                        self.syllable.update(Bopomofo::IU);
                        self.syllable.remove_rime();
                        return KeyBehavior::Absorb;
                    }
                    (Some(_), _) => {
                        self.syllable.update(Bopomofo::OU);
                        return KeyBehavior::Absorb;
                    }
                    _ => (),
                }
                Bopomofo::IU
            }
            KeyIndex::K22 => default_or_alt(self.syllable.rime(), Bopomofo::O, Bopomofo::AI),
            KeyIndex::K34 => Bopomofo::E,
            KeyIndex::K23 => default_or_alt(self.syllable.rime(), Bopomofo::EI, Bopomofo::AN),
            KeyIndex::K35 => default_or_alt(self.syllable.rime(), Bopomofo::AU, Bopomofo::ANG),
            KeyIndex::K24 => default_or_alt(self.syllable.rime(), Bopomofo::EN, Bopomofo::ER),
            _ => return KeyBehavior::KeyError,
        };

        self.syllable.update(bopomofo);
        KeyBehavior::Absorb
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

    fn key_seq(&self) -> Option<String> {
        None
    }

    fn clone(&self) -> Box<dyn SyllableEditor> {
        Box::new(Clone::clone(self))
    }
}
