//! Dai Chien CP26

use super::{KeyBehavior, SyllableEditor};
use crate::input::keycode::*;
use crate::{
    input::KeyboardEvent,
    zhuyin::{Bopomofo, Syllable},
};

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
    fn is_end_key(&self, key: Keycode) -> bool {
        match key {
            KEY_E | KEY_R | KEY_D | KEY_Y | KEY_SPACE => !self.syllable.is_empty(),
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
    fn key_press(&mut self, key: KeyboardEvent) -> KeyBehavior {
        if self.is_end_key(key.code) {
            match key.code {
                // KeyIndex::K48 => Some(Bopomofo::TONE1),
                KEY_E => self.syllable.update(Bopomofo::TONE2),
                KEY_R => self.syllable.update(Bopomofo::TONE3),
                KEY_D => self.syllable.update(Bopomofo::TONE4),
                KEY_Y => self.syllable.update(Bopomofo::TONE5),
                _ => {
                    self.syllable.remove_tone();
                }
            };
            return KeyBehavior::Commit;
        }
        let bopomofo = match key.code {
            KEY_Q => default_or_alt(self.syllable.initial(), Bopomofo::B, Bopomofo::P),
            KEY_A => Bopomofo::M,
            KEY_Z => Bopomofo::F,
            KEY_W => default_or_alt(self.syllable.initial(), Bopomofo::D, Bopomofo::T),
            KEY_S => Bopomofo::N,
            KEY_X => Bopomofo::L,
            KEY_E => Bopomofo::G,
            KEY_D => Bopomofo::K,
            KEY_C => Bopomofo::H,
            KEY_R => Bopomofo::J,
            KEY_F => Bopomofo::Q,
            KEY_V => Bopomofo::X,
            KEY_T => default_or_alt(self.syllable.initial(), Bopomofo::ZH, Bopomofo::CH),
            KEY_G => Bopomofo::SH,
            KEY_B => {
                if self.has_initial_or_medial() {
                    Bopomofo::EH
                } else {
                    Bopomofo::R
                }
            }
            KEY_Y => Bopomofo::Z,
            KEY_H => Bopomofo::C,
            KEY_N => {
                if self.has_initial_or_medial() {
                    Bopomofo::ENG
                } else {
                    Bopomofo::S
                }
            }
            KEY_U => {
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
            KEY_J => Bopomofo::U,
            KEY_M => {
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
            KEY_I => default_or_alt(self.syllable.rime(), Bopomofo::O, Bopomofo::AI),
            KEY_K => Bopomofo::E,
            KEY_O => default_or_alt(self.syllable.rime(), Bopomofo::EI, Bopomofo::AN),
            KEY_L => default_or_alt(self.syllable.rime(), Bopomofo::AU, Bopomofo::ANG),
            KEY_P => default_or_alt(self.syllable.rime(), Bopomofo::EN, Bopomofo::ER),
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
