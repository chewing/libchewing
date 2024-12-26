//! Hsu keyboard layout

use crate::{
    editor::keyboard::KeyCode,
    syl,
    zhuyin::{Bopomofo, BopomofoKind, Syllable},
};

use super::{KeyBehavior, KeyEvent, SyllableEditor};

/// TODO: docs
#[derive(Debug, Clone, Copy)]
pub struct Hsu {
    syllable: Syllable,
}

impl Hsu {
    /// TODO: docs
    pub fn new() -> Hsu {
        Hsu {
            syllable: Default::default(),
        }
    }

    ///
    /// tone key is hsu_end_key
    ///  KeyCode::S -> Bopomofo::TONE5
    ///  KeyCode::D -> Bopomofo::TONE2
    ///  KeyCode::F -> Bopomofo::TONE3
    ///  KeyCode::J -> Bopomofo::TONE4
    ///  KeyCode::Space -> Bopomofo::TONE1
    ///
    fn is_hsu_end_key(&self, key: KeyEvent) -> bool {
        // TODO allow customize end key mapping
        match key.code {
            KeyCode::S | KeyCode::D | KeyCode::F | KeyCode::J | KeyCode::Space => {
                !self.syllable.is_empty()
            }
            _ => false,
        }
    }
    fn has_initial_or_medial(&self) -> bool {
        self.syllable.has_initial() || self.syllable.has_medial()
    }

    const ALT_TABLE: &'static [(Syllable, &'static [Syllable])] = &[
        (syl![Bopomofo::C], &[syl![Bopomofo::EI]]),
        (syl![Bopomofo::I], &[syl![Bopomofo::EH]]),
        (syl![Bopomofo::S], &[syl![Bopomofo::TONE5]]),
        (syl![Bopomofo::D], &[syl![Bopomofo::TONE2]]),
        (syl![Bopomofo::F], &[syl![Bopomofo::TONE3]]),
        (syl![Bopomofo::E], &[syl![Bopomofo::G]]),
        (syl![Bopomofo::O], &[syl![Bopomofo::H]]),
        (
            syl![Bopomofo::ZH],
            &[syl![Bopomofo::J], syl![Bopomofo::TONE4]],
        ),
        (syl![Bopomofo::ANG], &[syl![Bopomofo::K]]),
        (
            syl![Bopomofo::ER],
            &[syl![Bopomofo::L], syl![Bopomofo::ENG]],
        ),
        (syl![Bopomofo::SH], &[syl![Bopomofo::X]]),
        (syl![Bopomofo::CH], &[syl![Bopomofo::Q]]),
        (syl![Bopomofo::EN], &[syl![Bopomofo::N]]),
        (syl![Bopomofo::AN], &[syl![Bopomofo::M]]),
    ];
}

impl Default for Hsu {
    fn default() -> Self {
        Self::new()
    }
}

impl SyllableEditor for Hsu {
    fn key_press(&mut self, key: KeyEvent) -> KeyBehavior {
        if self.is_hsu_end_key(key) {
            if !self.syllable.has_medial() && !self.syllable.has_rime() {
                if let Some(key) = self.syllable.initial() {
                    match key {
                        Bopomofo::J => {
                            self.syllable.update(Bopomofo::ZH);
                        }
                        Bopomofo::Q => {
                            self.syllable.update(Bopomofo::CH);
                        }
                        Bopomofo::X => {
                            self.syllable.update(Bopomofo::SH);
                        }
                        Bopomofo::H => {
                            self.syllable.remove_initial();
                            self.syllable.update(Bopomofo::O);
                        }
                        Bopomofo::G => {
                            self.syllable.remove_initial();
                            self.syllable.update(Bopomofo::E);
                        }
                        Bopomofo::M => {
                            self.syllable.remove_initial();
                            self.syllable.update(Bopomofo::AN);
                        }
                        Bopomofo::N => {
                            self.syllable.remove_initial();
                            self.syllable.update(Bopomofo::EN);
                        }
                        Bopomofo::K => {
                            self.syllable.remove_initial();
                            self.syllable.update(Bopomofo::ANG);
                        }
                        Bopomofo::L => {
                            self.syllable.remove_initial();
                            self.syllable.update(Bopomofo::ER);
                        }
                        _ => (),
                    }
                }
            }

            // fuzzy ㄍㄧ to ㄐㄧ and ㄍㄩ to ㄐㄩ
            match (self.syllable.initial(), self.syllable.medial()) {
                (Some(Bopomofo::G), Some(Bopomofo::I))
                | (Some(Bopomofo::G), Some(Bopomofo::IU)) => {
                    self.syllable.update(Bopomofo::J);
                }
                _ => (),
            }

            match key.code {
                // KeyCode::Space => Some(Bopomofo::TONE1),
                KeyCode::D => self.syllable.update(Bopomofo::TONE2),
                KeyCode::F => self.syllable.update(Bopomofo::TONE3),
                KeyCode::J => self.syllable.update(Bopomofo::TONE4),
                KeyCode::S => self.syllable.update(Bopomofo::TONE5),
                _ => {
                    self.syllable.remove_tone();
                }
            };
            KeyBehavior::Commit
        } else {
            let bopomofo = match key.code {
                KeyCode::A => {
                    if self.has_initial_or_medial() {
                        Bopomofo::EI
                    } else {
                        Bopomofo::C
                    }
                }
                KeyCode::B => Bopomofo::B,
                KeyCode::C => Bopomofo::SH,
                KeyCode::D => Bopomofo::D,
                KeyCode::E => {
                    if self.syllable.has_medial() {
                        Bopomofo::EH
                    } else {
                        Bopomofo::I
                    }
                }
                KeyCode::F => Bopomofo::F,
                KeyCode::G => {
                    if self.has_initial_or_medial() {
                        Bopomofo::E
                    } else {
                        Bopomofo::G
                    }
                }
                KeyCode::H => {
                    if self.has_initial_or_medial() {
                        Bopomofo::O
                    } else {
                        Bopomofo::H
                    }
                }
                KeyCode::I => Bopomofo::AI,
                KeyCode::J => Bopomofo::ZH,
                KeyCode::K => {
                    if self.has_initial_or_medial() {
                        Bopomofo::ANG
                    } else {
                        Bopomofo::K
                    }
                }
                KeyCode::L => {
                    if self.has_initial_or_medial() {
                        Bopomofo::ENG
                    } else {
                        Bopomofo::L
                    }
                }
                KeyCode::M => {
                    if self.has_initial_or_medial() {
                        Bopomofo::AN
                    } else {
                        Bopomofo::M
                    }
                }
                KeyCode::N => {
                    if self.has_initial_or_medial() {
                        Bopomofo::EN
                    } else {
                        Bopomofo::N
                    }
                }
                KeyCode::O => Bopomofo::OU,
                KeyCode::P => Bopomofo::P,
                KeyCode::R => Bopomofo::R,
                KeyCode::S => Bopomofo::S,
                KeyCode::T => Bopomofo::T,
                KeyCode::U => Bopomofo::IU,
                KeyCode::V => Bopomofo::CH,
                KeyCode::W => Bopomofo::AU,
                KeyCode::X => Bopomofo::U,
                KeyCode::Y => Bopomofo::A,
                KeyCode::Z => Bopomofo::Z,
                _ => return KeyBehavior::NoWord,
            };
            let kind = bopomofo.kind();

            // fuzzy ㄍㄧ to ㄐㄧ and ㄍㄩ to ㄐㄩ
            match (self.syllable.initial(), self.syllable.medial()) {
                (Some(Bopomofo::G), Some(Bopomofo::I))
                | (Some(Bopomofo::G), Some(Bopomofo::IU)) => {
                    self.syllable.update(Bopomofo::J);
                }
                _ => (),
            }

            // ㄐㄑㄒ must be followed by ㄧ or ㄩ. If not, convert them to ㄓㄔㄕ
            if (kind == BopomofoKind::Medial && bopomofo == Bopomofo::U)
                || (kind == BopomofoKind::Rime && self.syllable.medial().is_none())
            {
                match self.syllable.initial() {
                    Some(Bopomofo::J) => {
                        self.syllable.update(Bopomofo::ZH);
                    }
                    Some(Bopomofo::Q) => {
                        self.syllable.update(Bopomofo::CH);
                    }
                    Some(Bopomofo::X) => {
                        self.syllable.update(Bopomofo::SH);
                    }
                    _ => (),
                }
            }

            // Likeweise, when ㄓㄔㄕ is followed by ㄧ or ㄩ, convert them to ㄐㄑㄒ
            if bopomofo == Bopomofo::I || bopomofo == Bopomofo::IU {
                match self.syllable.initial() {
                    Some(Bopomofo::ZH) => {
                        self.syllable.update(Bopomofo::J);
                    }
                    Some(Bopomofo::CH) => {
                        self.syllable.update(Bopomofo::Q);
                    }
                    Some(Bopomofo::SH) => {
                        self.syllable.update(Bopomofo::X);
                    }
                    _ => (),
                }
            }

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

#[cfg(test)]
mod test {

    use crate::{
        editor::{
            keyboard::{KeyCode, KeyboardLayout, Qwerty},
            zhuyin_layout::SyllableEditor,
        },
        zhuyin::Bopomofo,
    };

    use super::Hsu;

    #[test]
    fn cen() {
        let mut hsu = Hsu::new();
        let keyboard = Qwerty;
        hsu.key_press(keyboard.map(KeyCode::C));
        hsu.key_press(keyboard.map(KeyCode::E));
        hsu.key_press(keyboard.map(KeyCode::N));
        hsu.key_press(keyboard.map(KeyCode::Space));
        let result = hsu.read();
        assert_eq!(result.initial(), Some(Bopomofo::X));
        assert_eq!(result.medial(), Some(Bopomofo::I));
        assert_eq!(result.rime(), Some(Bopomofo::EN));
    }

    #[test]
    fn convert_n_to_en() {
        let mut hsu = Hsu::new();
        let keyboard = Qwerty;
        hsu.key_press(keyboard.map(KeyCode::N));
        hsu.key_press(keyboard.map(KeyCode::F));
        let result = hsu.read();
        assert_eq!(result.rime(), Some(Bopomofo::EN));
    }
}
