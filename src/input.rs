//! Input handling modules

use crate::input::{keycode::Keycode, keysym::Keysym};

pub mod keycode;
pub mod keymap;
pub mod keysym;

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct KeyboardEvent {
    /// Code that identifies a physical key on a keyboard.
    ///
    /// Keycodes are the result of the low-level processing of the data that
    /// keyboards send to a computer. For instance 36 may represent the return
    /// key.
    ///
    /// Symbolic names are assigned to raw keycodes in order to facilitate
    /// their mapping to symbols. By convention keycode names are based on US
    /// QWERTY layout. For example the keycode for the return key is
    /// Keycode::RETURN.
    ///
    /// Chewing keycodes have same numeric encoding as X11 or xkbcommon
    /// keycodes.
    pub code: Keycode,

    /// The symbol on the cap of a key.
    ///
    /// Keysyms (short for "key symbol") are translated from keycodes via a
    /// keymap. On different layout (qwerty, dvorak, etc.) all keyboards emit
    /// the same keycodes but produce different keysyms after translation.
    pub key: Keysym,

    /// The key press / release state and state of modifier keys.
    ///
    /// Use the state mask to read whether a modifier key is active and
    /// whether the key is pressed.
    pub state: u32,
}

impl KeyboardEvent {
    /// Shift is activated.
    pub const SHIFT_MASK: u32 = 1 << 0;
    /// Caps Lock is activated.
    pub const CAPSLOCK_MASK: u32 = 1 << 1;
    /// Control is activated.
    pub const CONTROL_MASK: u32 = 1 << 2;
    /// Alt or Meta is activated.
    pub const ALT_MASK: u32 = 1 << 3;
    /// Num Lock is activated.
    pub const NUMLOCK_MASK: u32 = 1 << 4;
    /// Super is activated.
    pub const SUPER_MASK: u32 = 1 << 6;
    /// Key is on the keypad block.
    pub const KEYPAD_MASK: u32 = 1 << 29;
    /// Key is released.
    pub const RELEASE_MASK: u32 = 1 << 30;

    /// Create a builder to construct the KeyboardEvent.
    ///
    /// # Example
    ///
    /// ```
    /// use chewing::input::KeyboardEvent;
    ///
    /// let shift_pressed = true;
    /// let evt = KeyboardEvent::builder()
    ///     .shift_if(shift_pressed)
    ///     .build();
    /// ```
    pub fn builder() -> KeyboardEventBuilder {
        KeyboardEventBuilder {
            evt: KeyboardEvent::default(),
        }
    }
    pub fn is_invalid(&self) -> bool {
        self.code.0 == 0 && self.key == Keysym::NoSymbol && self.state == 0
    }
    pub fn is_keypad(&self) -> bool {
        self.is_flag_on(Self::KEYPAD_MASK)
    }

    fn is_flag_on(&self, mask: u32) -> bool {
        self.state & mask == mask
    }
}

#[derive(Clone, Copy, Debug)]
pub struct KeyboardEventBuilder {
    evt: KeyboardEvent,
}

impl KeyboardEventBuilder {
    pub fn code(&mut self, code: Keycode) -> &mut KeyboardEventBuilder {
        self.evt.code = code;
        self
    }
    pub fn key(&mut self, key: Keysym) -> &mut KeyboardEventBuilder {
        self.evt.key = key;
        self
    }
    pub fn shift_if(&mut self, shift: bool) -> &mut KeyboardEventBuilder {
        if shift {
            self.evt.state |= KeyboardEvent::SHIFT_MASK;
        }
        self
    }
    pub fn caps_lock_if(&mut self, caps_lock: bool) -> &mut KeyboardEventBuilder {
        if caps_lock {
            self.evt.state |= KeyboardEvent::CAPSLOCK_MASK;
        }
        self
    }
    pub fn control_if(&mut self, control: bool) -> &mut KeyboardEventBuilder {
        if control {
            self.evt.state |= KeyboardEvent::CONTROL_MASK;
        }
        self
    }
    pub fn alt_if(&mut self, alt: bool) -> &mut KeyboardEventBuilder {
        if alt {
            self.evt.state |= KeyboardEvent::ALT_MASK;
        }
        self
    }
    pub fn num_lock_if(&mut self, num_lock: bool) -> &mut KeyboardEventBuilder {
        if num_lock {
            self.evt.state |= KeyboardEvent::NUMLOCK_MASK;
        }
        self
    }
    pub fn super_if(&mut self, supa: bool) -> &mut KeyboardEventBuilder {
        if supa {
            self.evt.state |= KeyboardEvent::SUPER_MASK;
        }
        self
    }
    pub fn keypad_if(&mut self, keypad: bool) -> &mut KeyboardEventBuilder {
        if keypad {
            self.evt.state |= KeyboardEvent::KEYPAD_MASK;
        }
        self
    }
    pub fn release_if(&mut self, release: bool) -> &mut KeyboardEventBuilder {
        if release {
            self.evt.state |= KeyboardEvent::RELEASE_MASK;
        }
        self
    }
    pub fn release(&mut self) -> &mut KeyboardEventBuilder {
        self.evt.state |= KeyboardEvent::RELEASE_MASK;
        self
    }
    pub fn build(&mut self) -> KeyboardEvent {
        self.evt
    }
}
