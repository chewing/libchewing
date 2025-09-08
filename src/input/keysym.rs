#![allow(non_upper_case_globals)]
//! Key symbols.
//!
//! Keysyms (short for "key symbol") are translated from keycodes via a
//! keymap. On different layout (qwerty, dvorak, etc.) all keyboards emit
//! the same keycodes but produce different keysyms after translation.
//!
//! Keysyms has numerical encoding compatible with X11 / xkbcommon.

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct Keysym(pub u32);

impl Keysym {
    const UNICODE_OFFSET: u32 = 0x01000000;
    pub const NoSymbol: Keysym = Keysym(0x000000);
    pub const VoidSymbol: Keysym = Keysym(0xffffff);

    pub const BackSpace: Keysym = Keysym(0xff08);
    pub const Tab: Keysym = Keysym(0xff09);
    pub const Linefeed: Keysym = Keysym(0xff0a);
    pub const Clear: Keysym = Keysym(0xff0b);
    pub const Return: Keysym = Keysym(0xff0d);
    pub const Pause: Keysym = Keysym(0xff13);
    pub const Scroll_Lock: Keysym = Keysym(0xff14);
    pub const Sys_Req: Keysym = Keysym(0xff15);
    pub const Escape: Keysym = Keysym(0xff1b);
    pub const Delete: Keysym = Keysym(0xffff);

    pub const Home: Keysym = Keysym(0xff50);
    pub const Left: Keysym = Keysym(0xff51);
    pub const Up: Keysym = Keysym(0xff52);
    pub const Right: Keysym = Keysym(0xff53);
    pub const Down: Keysym = Keysym(0xff54);
    pub const Page_Up: Keysym = Keysym(0xff55);
    pub const Page_Down: Keysym = Keysym(0xff56);
    pub const End: Keysym = Keysym(0xff57);
    pub const Begin: Keysym = Keysym(0xff58);

    pub const Num_Lock: Keysym = Keysym(0xff7f);

    pub const KP_Space: Keysym = Keysym(0xff80);
    pub const KP_Tab: Keysym = Keysym(0xff89);
    pub const KP_Enter: Keysym = Keysym(0xff8d);
    pub const KP_F1: Keysym = Keysym(0xff91);
    pub const KP_F2: Keysym = Keysym(0xff92);
    pub const KP_F3: Keysym = Keysym(0xff93);
    pub const KP_F4: Keysym = Keysym(0xff94);
    pub const KP_Home: Keysym = Keysym(0xff95);
    pub const KP_Left: Keysym = Keysym(0xff96);
    pub const KP_Up: Keysym = Keysym(0xff97);
    pub const KP_Right: Keysym = Keysym(0xff98);
    pub const KP_Down: Keysym = Keysym(0xff99);
    pub const KP_Page_Up: Keysym = Keysym(0xff9a);
    pub const KP_Page_Down: Keysym = Keysym(0xff9b);
    pub const KP_End: Keysym = Keysym(0xff9c);
    pub const KP_Begin: Keysym = Keysym(0xff9d);
    pub const KP_Insert: Keysym = Keysym(0xff9e);
    pub const KP_Delete: Keysym = Keysym(0xff9f);
    pub const KP_Equal: Keysym = Keysym(0xffbd);
    pub const KP_Multiply: Keysym = Keysym(0xffaa);
    pub const KP_Add: Keysym = Keysym(0xffab);
    pub const KP_Separator: Keysym = Keysym(0xffac);
    pub const KP_Subtract: Keysym = Keysym(0xffad);
    pub const KP_Decimal: Keysym = Keysym(0xffae);
    pub const KP_Divide: Keysym = Keysym(0xffaf);
    pub const KP_0: Keysym = Keysym(0xffb0);
    pub const KP_1: Keysym = Keysym(0xffb1);
    pub const KP_2: Keysym = Keysym(0xffb2);
    pub const KP_3: Keysym = Keysym(0xffb3);
    pub const KP_4: Keysym = Keysym(0xffb4);
    pub const KP_5: Keysym = Keysym(0xffb5);
    pub const KP_6: Keysym = Keysym(0xffb6);
    pub const KP_7: Keysym = Keysym(0xffb7);
    pub const KP_8: Keysym = Keysym(0xffb8);
    pub const KP_9: Keysym = Keysym(0xffb9);

    pub const F1: Keysym = Keysym(0xffbe);
    pub const F2: Keysym = Keysym(0xffbf);
    pub const F3: Keysym = Keysym(0xffc0);
    pub const F4: Keysym = Keysym(0xffc1);
    pub const F5: Keysym = Keysym(0xffc2);
    pub const F6: Keysym = Keysym(0xffc3);
    pub const F7: Keysym = Keysym(0xffc4);
    pub const F8: Keysym = Keysym(0xffc5);
    pub const F9: Keysym = Keysym(0xffc6);
    pub const F10: Keysym = Keysym(0xffc7);
    pub const F11: Keysym = Keysym(0xffc8);
    pub const F12: Keysym = Keysym(0xffc9);

    pub const Shift_L: Keysym = Keysym(0xffe1);
    pub const Shift_R: Keysym = Keysym(0xffe2);
    pub const Control_L: Keysym = Keysym(0xffe3);
    pub const Control_R: Keysym = Keysym(0xffe4);
    pub const Caps_Lock: Keysym = Keysym(0xffe5);
    pub const Shift_Lock: Keysym = Keysym(0xffe6);
    pub const Meta_L: Keysym = Keysym(0xffe7);
    pub const Meta_R: Keysym = Keysym(0xffe8);
    pub const Alt_L: Keysym = Keysym(0xffe9);
    pub const Alt_R: Keysym = Keysym(0xffea);
    pub const Super_L: Keysym = Keysym(0xffeb);
    pub const Super_R: Keysym = Keysym(0xffec);
    pub const Hyper_L: Keysym = Keysym(0xffed);
    pub const Hyper_R: Keysym = Keysym(0xffee);

    pub const Space: Keysym = Keysym(0x0020);
    pub const Grave: Keysym = Keysym(0x0060);
    pub const J: Keysym = Keysym(0x006a);
    pub const K: Keysym = Keysym(0x006b);
}

impl Keysym {
    pub const fn from_char(value: char) -> Keysym {
        if value.is_ascii_control() {
            // ASCII control characters are not valid Keysm
            return Keysym::NoSymbol;
        }
        if value.is_ascii() {
            return Keysym(value as u32);
        }
        Keysym(value as u32 + Self::UNICODE_OFFSET)
    }
    pub fn is_ascii(&self) -> bool {
        self.0 >= 0x20 && self.0 <= 0x7e
    }
    pub fn is_unicode(&self) -> bool {
        self.is_ascii() || (self.0 >= 0x01000100 && self.0 <= 0x0110ffff)
    }
    pub fn is_keypad(&self) -> bool {
        (self.0 >= Keysym::KP_0.0 && self.0 <= Keysym::KP_9.0)
            || [
                Keysym::KP_Add,
                Keysym::KP_Subtract,
                Keysym::KP_Multiply,
                Keysym::KP_Divide,
                Keysym::KP_Decimal,
                Keysym::KP_Equal,
            ]
            .contains(self)
    }
    pub fn to_unicode(&self) -> char {
        if self.is_keypad() {
            return match *self {
                Keysym::KP_0 => '0',
                Keysym::KP_1 => '1',
                Keysym::KP_2 => '2',
                Keysym::KP_3 => '3',
                Keysym::KP_4 => '4',
                Keysym::KP_5 => '5',
                Keysym::KP_6 => '6',
                Keysym::KP_7 => '7',
                Keysym::KP_8 => '8',
                Keysym::KP_9 => '9',
                Keysym::KP_Add => '+',
                Keysym::KP_Subtract => '-',
                Keysym::KP_Multiply => '*',
                Keysym::KP_Divide => '/',
                Keysym::KP_Decimal => '.',
                Keysym::KP_Equal => '=',
                _ => unreachable!(),
            };
        }
        if !self.is_unicode() {
            return char::REPLACEMENT_CHARACTER;
        }
        if self.0 > Self::UNICODE_OFFSET {
            return char::from_u32(self.0 - Self::UNICODE_OFFSET)
                .unwrap_or(char::REPLACEMENT_CHARACTER);
        }
        char::from_u32(self.0).unwrap_or(char::REPLACEMENT_CHARACTER)
    }
    pub fn is_digit(&self) -> bool {
        self.to_digit().is_some()
    }
    pub fn to_digit(&self) -> Option<u8> {
        if self.0 >= 0x0030 && self.0 <= 0x0039 {
            return Some((self.0 - 0x0030) as u8);
        }
        if self >= &Keysym::KP_0 && self <= &Keysym::KP_9 {
            return Some((self.0 - Keysym::KP_0.0) as u8);
        }
        None
    }
    pub fn is_atoz(&self) -> bool {
        (self.0 >= 'a' as u32 && self.0 <= 'z' as u32)
            || (self.0 >= 'A' as u32 && self.0 <= 'Z' as u32)
    }
}

impl From<char> for Keysym {
    fn from(value: char) -> Self {
        Self::from_char(value)
    }
}

#[cfg(test)]
mod tests {
    use super::Keysym;

    #[test]
    fn tty_function_keys() {
        assert_eq!(Keysym::BackSpace, Keysym(0xff08));
    }

    #[test]
    fn latin1_keys() {
        assert_eq!(Keysym::from('a'), Keysym(0x0061));
        assert_eq!(Keysym::from('1'), Keysym(0x0031));
    }

    #[test]
    fn unicode_keys() {
        assert_eq!(Keysym::from('\u{20A0}'), Keysym(0x10020a0));
    }

    #[test]
    fn digits() {
        let ksym = Keysym::from('5');
        assert!(ksym.is_digit());
        assert_eq!(Some(5), ksym.to_digit());
    }

    #[test]
    fn alphabet() {
        let mut ksyms = ('a'..'z').chain('A'..'Z').map(Keysym::from);
        assert!(ksyms.all(|ksym| ksym.is_atoz()))
    }
}
