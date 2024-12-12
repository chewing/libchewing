//! Pinyin

use crate::{
    editor::keyboard::{KeyCode, KeyEvent},
    zhuyin::{Bopomofo, Syllable},
};

use super::{KeyBehavior, SyllableEditor};

const MAX_PINYIN_LEN: usize = 10;

/// TODO: docs
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum PinyinVariant {
    /// TODO: docs
    #[default]
    HanyuPinyin,
    /// TODO: docs
    ThlPinyin,
    /// TODO: docs
    Mps2Pinyin,
}

/// TODO: docs
#[derive(Default, Debug, Clone)]
pub struct Pinyin {
    key_seq: String,
    syllable: Syllable,
    syllable_alt: Syllable,
    variant: PinyinVariant,
}

impl Pinyin {
    /// TODO: docs
    pub fn new() -> Pinyin {
        Default::default()
    }
    /// TODO: docs
    /// TODO: refactor this to const variable
    pub fn hanyu() -> Pinyin {
        Pinyin {
            variant: PinyinVariant::HanyuPinyin,
            ..Default::default()
        }
    }
    /// TODO: docs
    /// TODO: refactor this to const variable
    pub fn thl() -> Pinyin {
        Pinyin {
            variant: PinyinVariant::ThlPinyin,
            ..Default::default()
        }
    }
    /// TODO: docs
    /// TODO: refactor this to const variable
    pub fn mps2() -> Pinyin {
        Pinyin {
            variant: PinyinVariant::Mps2Pinyin,
            ..Default::default()
        }
    }
    /// TODO: docs
    pub fn alt(&self) -> Syllable {
        self.syllable_alt
    }
    /// TODO: docs
    pub fn key_seq(&self) -> &String {
        &self.key_seq
    }
}

impl SyllableEditor for Pinyin {
    fn key_press(&mut self, key: KeyEvent) -> KeyBehavior {
        if self.key_seq.is_empty() && !key.code.is_atoz() {
            return KeyBehavior::KeyError;
        }
        if ![
            KeyCode::Space,
            KeyCode::N1,
            KeyCode::N2,
            KeyCode::N3,
            KeyCode::N4,
            KeyCode::N5,
        ]
        .contains(&key.code)
        {
            if self.key_seq.len() == MAX_PINYIN_LEN {
                // buffer is full, ignore this keystroke
                return KeyBehavior::NoWord;
            }
            if !key.unicode.is_ascii_alphabetic() {
                return KeyBehavior::KeyError;
            }
            self.key_seq.push(key.unicode);
            return KeyBehavior::Absorb;
        }

        let tone = match key.code {
            // KeyCode::Space | KeyCode::N1 => Some(Bopomofo::TONE1),
            KeyCode::N2 => Some(Bopomofo::TONE2),
            KeyCode::N3 => Some(Bopomofo::TONE3),
            KeyCode::N4 => Some(Bopomofo::TONE4),
            KeyCode::N5 => Some(Bopomofo::TONE5),
            _ => None,
        };

        if let Some(entry) = match self.variant {
            PinyinVariant::HanyuPinyin => table::HANYU_PINYIN_MAPPING.iter(),
            PinyinVariant::ThlPinyin => table::THL_PINYIN_MAPPING.iter(),
            PinyinVariant::Mps2Pinyin => table::MPS2_PINYIN_MAPPING.iter(),
        }
        .find(|entry| entry.pinyin == self.key_seq)
        {
            self.key_seq.clear();
            self.syllable = entry.primary;
            self.syllable_alt = entry.alt;
            if let Some(tone) = tone {
                self.syllable.update(tone);
                self.syllable_alt.update(tone);
            }
            return KeyBehavior::Commit;
        }

        if let Some(entry) = table::COMMON_MAPPING
            .iter()
            .find(|entry| entry.pinyin == self.key_seq)
        {
            self.key_seq.clear();
            self.syllable = entry.primary;
            self.syllable_alt = entry.alt;
            if let Some(tone) = tone {
                self.syllable.update(tone);
                self.syllable_alt.update(tone);
            }
            return KeyBehavior::Commit;
        }

        let initial = table::INITIAL_MAPPING
            .iter()
            .find(|entry| self.key_seq.starts_with(entry.pinyin));

        let final_seq = match initial {
            Some(entry) => self.key_seq.trim_start_matches(entry.pinyin),
            None => &self.key_seq,
        };

        let fina = table::FINAL_MAPPING
            .iter()
            .find(|entry| final_seq == entry.pinyin);

        if initial.is_none() && fina.is_none() {
            self.key_seq.clear();
            return KeyBehavior::Absorb;
        }

        let mut initial = initial.map(|i| i.initial);
        let mut medial = fina.and_then(|f| f.medial);
        let mut rime = fina.and_then(|f| f.rime);

        /* Hanyu empty rime
         * ㄓ/ㄔ/ㄕ/ㄖ/ㄗ/ㄘ/ㄙ + -i, -i is empty rime, not ㄧ
         * */
        if self.variant == PinyinVariant::HanyuPinyin
            && matches!(
                (medial, rime),
                (Some(Bopomofo::I), None) | (None, Some(Bopomofo::I))
            )
        {
            match initial {
                Some(Bopomofo::ZH) | Some(Bopomofo::CH) | Some(Bopomofo::SH)
                | Some(Bopomofo::R) | Some(Bopomofo::Z) | Some(Bopomofo::C) | Some(Bopomofo::S) => {
                    medial.take();
                    rime.take();
                }
                _ => (),
            }
        }

        /* Hanyu uan/un/u :
         * ㄐ/ㄑ/ㄒ + -uan, -uan is ㄩㄢ, not ㄨㄢ
         * ㄐ/ㄑ/ㄒ + -un,  -un is ㄩㄣ, not ㄨㄣ
         * ㄐ/ㄑ/ㄒ + -u,   -u is ㄧ, not ㄨ
         */
        if self.variant == PinyinVariant::HanyuPinyin {
            match initial {
                Some(Bopomofo::J) | Some(Bopomofo::Q) | Some(Bopomofo::X) => {
                    match (medial, rime) {
                        (Some(Bopomofo::U), Some(Bopomofo::AN))
                        | (Some(Bopomofo::U), Some(Bopomofo::EN))
                        | (Some(Bopomofo::U), None) => {
                            medial.replace(Bopomofo::IU);
                        }
                        _ => (),
                    };
                }
                _ => (),
            }
        }

        /* THL/MPS2 s/sh/c/ch/j :
         * s-  + ー/ㄩ, s-  is ㄒ, not ㄙ (THL/Tongyong)
         * sh- + ー/ㄩ, sh- is ㄒ, not ㄕ (MPS2)
         * c-  + ー/ㄩ, c-  is ㄑ, not ㄘ (Tongyong)
         * ch- + ㄧ/ㄩ, ch- is ㄑ, not ㄔ (THL)
         * j-  + other than ー/ㄩ, j-  is ㄓ, not ㄐ (MPS2)
         */
        match self.variant {
            PinyinVariant::ThlPinyin | PinyinVariant::Mps2Pinyin => match medial {
                Some(Bopomofo::I) | Some(Bopomofo::IU) => {
                    match initial {
                        Some(Bopomofo::S) | Some(Bopomofo::SH) => {
                            initial.replace(Bopomofo::X);
                        }
                        Some(Bopomofo::C) | Some(Bopomofo::CH) => {
                            initial.replace(Bopomofo::Q);
                        }
                        _ => (),
                    };
                }
                _ => {
                    if initial == Some(Bopomofo::J) {
                        initial.replace(Bopomofo::ZH);
                    }
                }
            },
            PinyinVariant::HanyuPinyin => {}
        }

        /* THL supplemental set
         * ㄅ/ㄆ/ㄇ/ㄈ + -ㄨㄥ, -ㄨㄥ is another reading of -ㄥ
         * ㄅ/ㄆ/ㄇ/ㄈ + -ㄨㄛ, -ㄨㄛ is another reading of -ㄛ
         */
        match self.variant {
            PinyinVariant::ThlPinyin | PinyinVariant::Mps2Pinyin => match initial {
                Some(Bopomofo::B) | Some(Bopomofo::P) | Some(Bopomofo::M) | Some(Bopomofo::F) => {
                    match (medial, rime) {
                        (Some(Bopomofo::U), Some(Bopomofo::ENG))
                        | (Some(Bopomofo::U), Some(Bopomofo::O)) => {
                            medial.take();
                        }
                        _ => (),
                    };
                }
                _ => (),
            },
            _ => {}
        }

        self.key_seq.clear();
        let mut builder = Syllable::builder();
        if let Some(initial) = initial {
            builder = builder.insert(initial).unwrap();
        }
        if let Some(medial) = medial {
            builder = builder.insert(medial).unwrap();
        }
        if let Some(rime) = rime {
            builder = builder.insert(rime).unwrap();
        }
        if let Some(tone) = tone {
            builder = builder.insert(tone).unwrap();
        }
        self.syllable = builder.build();
        self.syllable_alt = self.syllable;
        KeyBehavior::Commit
    }

    fn fuzzy_key_press(&mut self, key: KeyEvent) -> KeyBehavior {
        self.key_press(key)
    }

    fn is_empty(&self) -> bool {
        self.key_seq.is_empty()
    }

    fn remove_last(&mut self) {
        self.key_seq.pop();
    }

    fn clear(&mut self) {
        self.key_seq.clear();
        self.syllable.clear();
        self.syllable_alt.clear();
    }

    fn read(&self) -> Syllable {
        self.syllable
    }

    fn key_seq(&self) -> Option<String> {
        Some(self.key_seq.clone())
    }

    fn clone(&self) -> Box<dyn SyllableEditor> {
        Box::new(Clone::clone(self))
    }
}

struct AmbiguousMapEntry {
    pinyin: &'static str,
    primary: Syllable,
    alt: Syllable,
}

macro_rules! amb {
    ($pinyin:expr, $primary:expr, $alt:expr ) => {
        AmbiguousMapEntry {
            pinyin: $pinyin,
            primary: $primary,
            alt: $alt,
        }
    };
}

struct InitialMapEntry {
    pinyin: &'static str,
    initial: Bopomofo,
}

macro_rules! ini {
    ($pinyin:expr, $bopomofo:expr) => {
        InitialMapEntry {
            pinyin: $pinyin,
            initial: $bopomofo,
        }
    };
}

struct FinalMapEntry {
    pinyin: &'static str,
    medial: Option<Bopomofo>,
    rime: Option<Bopomofo>,
}

macro_rules! fin {
    ($pinyin:expr, $medial:expr, $rime:expr) => {
        FinalMapEntry {
            pinyin: $pinyin,
            medial: $medial,
            rime: $rime,
        }
    };
}

mod table {

    use crate::{syl, zhuyin::Bopomofo::*};

    use super::{AmbiguousMapEntry, FinalMapEntry, InitialMapEntry};

    pub(super) const COMMON_MAPPING: [AmbiguousMapEntry; 18] = [
        // Special cases for WG
        amb!("tzu", syl![Z], syl![Z, U]),
        amb!("ssu", syl![S], syl![S, U]),
        amb!("szu", syl![S], syl![S, U]),
        // Common multiple mapping
        amb!("e", syl![E], syl![EH]),
        amb!("ch", syl![CH], syl![Q]),
        amb!("sh", syl![SH], syl![X]),
        amb!("c", syl![C], syl![Q]),
        amb!("s", syl![S], syl![X]),
        amb!("nu", syl![N, U], syl![N, IU]),
        amb!("lu", syl![L, U], syl![L, IU]),
        amb!("luan", syl![L, U, AN], syl![L, IU, AN]),
        amb!("niu", syl![N, I, OU], syl![N, IU]),
        amb!("liu", syl![L, I, OU], syl![L, IU]),
        amb!("jiu", syl![J, I, OU], syl![J, IU]),
        amb!("chiu", syl![Q, I, OU], syl![Q, IU]),
        amb!("shiu", syl![X, I, OU], syl![X, IU]),
        amb!("ju", syl![J, IU], syl![ZH, U]),
        amb!("juan", syl![J, IU, AN], syl![ZH, U, AN]),
    ];

    pub(super) const HANYU_PINYIN_MAPPING: [AmbiguousMapEntry; 4] = [
        amb!("chi", syl![CH], syl![Q, I]),
        amb!("shi", syl![SH], syl![X, I]),
        amb!("ci", syl![C], syl![Q, I]),
        amb!("si", syl![S], syl![X, I]),
    ];

    pub(super) const THL_PINYIN_MAPPING: [AmbiguousMapEntry; 4] = [
        amb!("chi", syl![Q, I], syl![CH]),
        amb!("shi", syl![X, I], syl![SH]),
        amb!("ci", syl![Q, I], syl![C]),
        amb!("si", syl![X, I], syl![S]),
    ];

    pub(super) const MPS2_PINYIN_MAPPING: [AmbiguousMapEntry; 13] = [
        amb!("chi", syl![Q, I], syl![CH]),
        amb!("shi", syl![X, I], syl![SH]),
        amb!("ci", syl![Q, I], syl![C]),
        amb!("si", syl![X, I], syl![S]),
        amb!("niu", syl![N, IU], syl![N, I, OU]),
        amb!("liu", syl![L, IU], syl![L, I, OU]),
        amb!("jiu", syl![J, IU], syl![J, I, OU]),
        amb!("chiu", syl![Q, IU], syl![Q, I, OU]),
        amb!("shiu", syl![X, IU], syl![X, I, OU]),
        amb!("ju", syl![ZH, U], syl![J, IU]),
        amb!("juan", syl![ZH, U, AN], syl![J, IU, AN]),
        amb!("juen", syl![ZH, U, EN], syl![J, IU, EN]),
        amb!("tzu", syl![Z, U], syl![Z]),
    ];

    pub(super) const INITIAL_MAPPING: [InitialMapEntry; 25] = [
        ini!("tz", Z),
        ini!("b", B),
        ini!("p", P),
        ini!("m", M),
        ini!("f", F),
        ini!("d", D),
        ini!("ts", C),
        ini!("t", T),
        ini!("n", N),
        ini!("l", L),
        ini!("g", G),
        ini!("k", K),
        ini!("hs", X),
        ini!("h", H),
        ini!("jh", ZH),
        ini!("j", J),
        ini!("q", Q),
        ini!("x", X),
        ini!("zh", ZH),
        ini!("ch", CH),
        ini!("sh", SH),
        ini!("r", R),
        ini!("z", Z),
        ini!("c", C),
        ini!("s", S),
    ];

    pub(super) const FINAL_MAPPING: [FinalMapEntry; 90] = [
        fin!("uang", Some(U), Some(ANG)),
        fin!("wang", Some(U), Some(ANG)),
        fin!("weng", Some(U), Some(ENG)),
        fin!("wong", Some(U), Some(ENG)),
        fin!("ying", Some(I), Some(ENG)),
        fin!("yung", Some(IU), Some(ENG)),
        fin!("yong", Some(IU), Some(ENG)),
        fin!("iung", Some(IU), Some(ENG)),
        fin!("iong", Some(IU), Some(ENG)),
        fin!("iang", Some(I), Some(ANG)),
        fin!("yang", Some(I), Some(ANG)),
        fin!("yuan", Some(IU), Some(AN)),
        fin!("iuan", Some(IU), Some(AN)),
        fin!("ing", Some(I), Some(ENG)),
        fin!("iao", Some(I), Some(AU)),
        fin!("iau", Some(I), Some(AU)),
        fin!("yao", Some(I), Some(AU)),
        fin!("yau", Some(I), Some(AU)),
        fin!("yun", Some(IU), Some(EN)),
        fin!("iun", Some(IU), Some(EN)),
        fin!("vn", Some(IU), Some(EN)),
        fin!("iou", Some(I), Some(OU)),
        fin!("iu", Some(I), Some(OU)),
        fin!("you", Some(I), Some(OU)),
        fin!("io", Some(I), Some(O)),
        fin!("yo", Some(I), Some(O)),
        fin!("ian", Some(I), Some(AN)),
        fin!("ien", Some(I), Some(AN)),
        fin!("yan", Some(I), Some(AN)),
        fin!("yen", Some(I), Some(AN)),
        fin!("yin", Some(I), Some(EN)),
        fin!("ang", None, Some(ANG)),
        fin!("eng", None, Some(ENG)),
        fin!("uei", Some(U), Some(EI)),
        fin!("ui", Some(U), Some(EI)),
        fin!("wei", Some(U), Some(EI)),
        fin!("uen", Some(U), Some(EN)),
        fin!("yueh", Some(IU), Some(EH)),
        fin!("yue", Some(IU), Some(EH)),
        fin!("iue", Some(IU), Some(EH)),
        fin!("ueh", Some(IU), Some(EH)),
        fin!("ue", Some(IU), Some(EH)),
        fin!("ve", Some(IU), Some(EH)),
        fin!("uai", Some(U), Some(AI)),
        fin!("wai", Some(U), Some(AI)),
        fin!("uan", Some(U), Some(AN)),
        fin!("wan", Some(U), Some(AN)),
        fin!("un", Some(U), Some(EN)),
        fin!("wen", Some(U), Some(EN)),
        fin!("wun", Some(U), Some(EN)),
        fin!("ung", Some(U), Some(ENG)),
        fin!("ong", Some(U), Some(ENG)),
        fin!("van", Some(IU), Some(AN)),
        fin!("er", None, Some(ER)),
        fin!("ai", None, Some(AI)),
        fin!("ei", None, Some(EI)),
        fin!("ao", None, Some(AU)),
        fin!("au", None, Some(AU)),
        fin!("ou", None, Some(OU)),
        fin!("an", None, Some(AN)),
        fin!("en", None, Some(EN)),
        fin!("yi", None, Some(I)),
        fin!("ia", Some(I), Some(A)),
        fin!("ya", Some(I), Some(A)),
        fin!("ieh", Some(I), Some(EH)),
        fin!("ie", Some(I), Some(EH)),
        fin!("yeh", Some(I), Some(EH)),
        fin!("ye", Some(I), Some(EH)),
        fin!("in", Some(I), Some(EN)),
        fin!("wu", Some(U), None),
        fin!("ua", Some(U), Some(A)),
        fin!("wa", Some(U), Some(A)),
        fin!("uo", Some(U), Some(O)),
        fin!("wo", Some(U), Some(O)),
        fin!("yu", Some(IU), None),
        fin!("ve", Some(IU), Some(EH)),
        fin!("vn", Some(IU), Some(EN)),
        fin!("ih", None, None),
        fin!("a", None, Some(A)),
        fin!("o", None, Some(O)),
        fin!("eh", None, Some(EH)),
        fin!("e", None, Some(E)),
        fin!("v", Some(IU), None),
        fin!("i", Some(I), None),
        fin!("u", Some(U), None),
        fin!("E", None, Some(EH)),
        fin!("n", None, Some(EN)),
        fin!("ng", None, Some(ENG)),
        fin!("r", None, None),
        fin!("z", None, None),
    ];
}

#[cfg(test)]
mod tests {
    use crate::{
        editor::{
            keyboard::{AnyKeyboardLayout, KeyCode, KeyboardLayout},
            zhuyin_layout::SyllableEditor,
        },
        syl,
        zhuyin::Bopomofo,
    };

    use super::Pinyin;

    #[test]
    fn hanyu_empty_rime_zi() {
        let keyboard = AnyKeyboardLayout::qwerty();
        let mut hanyu = Pinyin::hanyu();

        hanyu.key_press(keyboard.map(KeyCode::Z));
        hanyu.key_press(keyboard.map(KeyCode::I));
        hanyu.key_press(keyboard.map(KeyCode::N1));

        assert_eq!(syl![Bopomofo::Z], hanyu.read());
    }

    #[test]
    fn hanyu_empty_rime_zhi() {
        let keyboard = AnyKeyboardLayout::qwerty();
        let mut hanyu = Pinyin::hanyu();

        hanyu.key_press(keyboard.map(KeyCode::Z));
        hanyu.key_press(keyboard.map(KeyCode::H));
        hanyu.key_press(keyboard.map(KeyCode::I));
        hanyu.key_press(keyboard.map(KeyCode::N1));

        assert_eq!(syl![Bopomofo::ZH], hanyu.read());
    }

    #[test]
    fn hanyu_uan_un_u() {
        let keyboard = AnyKeyboardLayout::qwerty();
        let mut hanyu = Pinyin::hanyu();

        hanyu.key_press(keyboard.map(KeyCode::J));
        hanyu.key_press(keyboard.map(KeyCode::U));
        hanyu.key_press(keyboard.map(KeyCode::A));
        hanyu.key_press(keyboard.map(KeyCode::N));
        hanyu.key_press(keyboard.map(KeyCode::N1));

        assert_eq!(syl![Bopomofo::J, Bopomofo::IU, Bopomofo::AN], hanyu.read());

        hanyu.clear();
        hanyu.key_press(keyboard.map(KeyCode::Q));
        hanyu.key_press(keyboard.map(KeyCode::U));
        hanyu.key_press(keyboard.map(KeyCode::N));
        hanyu.key_press(keyboard.map(KeyCode::N1));

        assert_eq!(syl![Bopomofo::Q, Bopomofo::IU, Bopomofo::EN], hanyu.read());

        hanyu.clear();
        hanyu.key_press(keyboard.map(KeyCode::X));
        hanyu.key_press(keyboard.map(KeyCode::U));
        hanyu.key_press(keyboard.map(KeyCode::N1));

        assert_eq!(syl![Bopomofo::X, Bopomofo::IU], hanyu.read());
    }
}
