//! ET26 (倚天26鍵)

use crate::{
    editor::keyboard::{Keysym, KeyEvent},
    syl,
    zhuyin::{Bopomofo, BopomofoKind, Syllable},
};

use super::{KeyBehavior, SyllableEditor};

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
            Keysym::D | Keysym::F | Keysym::J | Keysym::K | Keysym::Space => {
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
    fn key_press(&mut self, key: KeyEvent) -> KeyBehavior {
        if self.is_end_key(key.key) {
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
            match key.key {
                // KeyCode::Space => Some(Bopomofo::TONE1),
                Keysym::F => self.syllable.update(Bopomofo::TONE2),
                Keysym::J => self.syllable.update(Bopomofo::TONE3),
                Keysym::K => self.syllable.update(Bopomofo::TONE4),
                Keysym::D => self.syllable.update(Bopomofo::TONE5),
                _ => {
                    self.syllable.remove_tone();
                }
            };
            KeyBehavior::Commit
        } else {
            let bopomofo = match key.key {
                Keysym::A => Bopomofo::A,
                Keysym::B => Bopomofo::B,
                Keysym::C => Bopomofo::X,
                Keysym::D => Bopomofo::D,
                Keysym::E => Bopomofo::I,
                Keysym::F => Bopomofo::F,
                Keysym::G => Bopomofo::J,
                Keysym::H => {
                    if self.has_initial_or_medial() {
                        Bopomofo::ER
                    } else {
                        Bopomofo::H
                    }
                }
                Keysym::I => Bopomofo::AI,
                Keysym::J => Bopomofo::R,
                Keysym::K => Bopomofo::K,
                Keysym::L => {
                    if self.has_initial_or_medial() {
                        Bopomofo::ENG
                    } else {
                        Bopomofo::L
                    }
                }
                Keysym::M => {
                    if self.has_initial_or_medial() {
                        Bopomofo::AN
                    } else {
                        Bopomofo::M
                    }
                }
                Keysym::N => {
                    if self.has_initial_or_medial() {
                        Bopomofo::EN
                    } else {
                        Bopomofo::N
                    }
                }
                Keysym::O => Bopomofo::O,
                Keysym::P => {
                    if self.has_initial_or_medial() {
                        Bopomofo::OU
                    } else {
                        Bopomofo::P
                    }
                }
                Keysym::Q => {
                    if self.has_initial_or_medial() {
                        Bopomofo::EI
                    } else {
                        Bopomofo::Z
                    }
                }
                Keysym::R => Bopomofo::E,
                Keysym::S => Bopomofo::S,
                Keysym::T => {
                    if self.has_initial_or_medial() {
                        Bopomofo::ANG
                    } else {
                        Bopomofo::T
                    }
                }
                Keysym::U => Bopomofo::IU,
                Keysym::V => Bopomofo::G,
                Keysym::W => {
                    if self.has_initial_or_medial() {
                        Bopomofo::EH
                    } else {
                        Bopomofo::C
                    }
                }
                Keysym::X => Bopomofo::U,
                Keysym::Y => Bopomofo::CH,
                Keysym::Z => Bopomofo::AU,
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
