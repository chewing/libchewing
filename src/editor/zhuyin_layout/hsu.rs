//! Hsu keyboard layout

use crate::{
    input::KeyboardEvent,
    syl,
    zhuyin::{Bopomofo, BopomofoKind, Syllable},
};

use super::{KeyBehavior, SyllableEditor};

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

    /// tone key is hsu_end_key
    ///
    ///  S -> Bopomofo::TONE5
    ///  D -> Bopomofo::TONE2
    ///  F -> Bopomofo::TONE3
    ///  J -> Bopomofo::TONE4
    ///  Space -> Bopomofo::TONE1
    fn is_hsu_end_key(&self, key: KeyboardEvent) -> bool {
        // TODO allow customize end key mapping
        match key.ksym.to_unicode() {
            's' | 'd' | 'f' | 'j' | ' ' => !self.syllable.is_empty(),
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
    fn key_press(&mut self, key: KeyboardEvent) -> KeyBehavior {
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

            match key.ksym.to_unicode() {
                // KeyCode::Space => Some(Bopomofo::TONE1),
                'd' => self.syllable.update(Bopomofo::TONE2),
                'f' => self.syllable.update(Bopomofo::TONE3),
                'j' => self.syllable.update(Bopomofo::TONE4),
                's' => self.syllable.update(Bopomofo::TONE5),
                _ => {
                    self.syllable.remove_tone();
                }
            };
            KeyBehavior::Commit
        } else {
            let bopomofo = match key.ksym.to_unicode() {
                'a' => {
                    if self.has_initial_or_medial() {
                        Bopomofo::EI
                    } else {
                        Bopomofo::C
                    }
                }
                'b' => Bopomofo::B,
                'c' => Bopomofo::SH,
                'd' => Bopomofo::D,
                'e' => {
                    if self.syllable.has_medial() {
                        Bopomofo::EH
                    } else {
                        Bopomofo::I
                    }
                }
                'f' => Bopomofo::F,
                'g' => {
                    if self.has_initial_or_medial() {
                        Bopomofo::E
                    } else {
                        Bopomofo::G
                    }
                }
                'h' => {
                    if self.has_initial_or_medial() {
                        Bopomofo::O
                    } else {
                        Bopomofo::H
                    }
                }
                'i' => Bopomofo::AI,
                'j' => Bopomofo::ZH,
                'k' => {
                    if self.has_initial_or_medial() {
                        Bopomofo::ANG
                    } else {
                        Bopomofo::K
                    }
                }
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
                'o' => Bopomofo::OU,
                'p' => Bopomofo::P,
                'r' => Bopomofo::R,
                's' => Bopomofo::S,
                't' => Bopomofo::T,
                'u' => Bopomofo::IU,
                'v' => Bopomofo::CH,
                'w' => Bopomofo::AU,
                'x' => Bopomofo::U,
                'y' => Bopomofo::A,
                'z' => Bopomofo::Z,
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
        editor::zhuyin_layout::SyllableEditor,
        input::{
            KeyboardEvent,
            keysym::{self, Keysym},
        },
        zhuyin::Bopomofo,
    };

    use super::Hsu;

    fn map_key(ksym: Keysym) -> KeyboardEvent {
        KeyboardEvent::builder().ksym(ksym).build()
    }

    #[test]
    fn cen() {
        let mut hsu = Hsu::new();
        hsu.key_press(map_key(keysym::SYM_LOWER_C));
        hsu.key_press(map_key(keysym::SYM_LOWER_E));
        hsu.key_press(map_key(keysym::SYM_LOWER_N));
        hsu.key_press(map_key(keysym::SYM_SPACE));
        let result = hsu.read();
        assert_eq!(result.initial(), Some(Bopomofo::X));
        assert_eq!(result.medial(), Some(Bopomofo::I));
        assert_eq!(result.rime(), Some(Bopomofo::EN));
    }

    #[test]
    fn convert_n_to_en() {
        let mut hsu = Hsu::new();
        hsu.key_press(map_key(keysym::SYM_LOWER_N));
        hsu.key_press(map_key(keysym::SYM_LOWER_F));
        let result = hsu.read();
        assert_eq!(result.rime(), Some(Bopomofo::EN));
    }
}
