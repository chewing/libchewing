//! Input handling modules

use keycode::Keycode;
use keysym::Keysym;

use crate::input::keysym::SYM_NONE;

pub mod keycode;
pub mod keymap;
pub mod keysym;

/// Keyboard layout independent KeyboardEvent
///
/// Use `code` to identify a physical key on a keyboard, use `ksym` to
/// represent the output after keymap translation.
///
/// With this abstraction, chewing can handle any keyboard layout supported
/// by the operating system.
///
/// # Examples
///
/// Use the [`KeyboardEventBuilder`] to construct a KeyboardEvent:
///
/// ```rust
/// use chewing::input::KeyboardEvent;
/// use chewing::input::keycode;
/// use chewing::input::keysym;
///
/// let control_down = true;
/// let caps_lock = false;
/// let evt = KeyboardEvent::builder()
///     .code(keycode::KEY_A)
///     .ksym(keysym::SYM_LOWER_A)
///     .shift()
///     .control_if(control_down)
///     .caps_lock_if(caps_lock)
///     .build();
/// # assert_eq!(keycode::KEY_A, evt.code);
/// # assert_eq!(keysym::SYM_LOWER_A, evt.ksym);
/// # assert!(evt.is_flag_on(KeyboardEvent::SHIFT_MASK));
/// # assert!(evt.is_flag_on(KeyboardEvent::CONTROL_MASK));
/// # assert!(!evt.is_flag_on(KeyboardEvent::CAPSLOCK_MASK));
/// ```
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
    pub ksym: Keysym,

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
        self.code.0 == 0 && self.ksym == SYM_NONE && self.state == 0
    }
    pub fn is_flag_on(&self, mask: u32) -> bool {
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
    pub fn ksym(&mut self, key: Keysym) -> &mut KeyboardEventBuilder {
        self.evt.ksym = key;
        self
    }
    pub fn shift(&mut self) -> &mut KeyboardEventBuilder {
        self.shift_if(true)
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

#[cfg(test)]
mod tests {
    use super::KeyboardEvent;
    use super::keycode;
    use super::keysym;

    #[test]
    fn keyboard_event_builder() {
        let control_down = true;
        let caps_lock = false;
        let evt = KeyboardEvent::builder()
            .code(keycode::KEY_A)
            .ksym(keysym::SYM_LOWER_A)
            .shift()
            .control_if(control_down)
            .caps_lock_if(caps_lock)
            .num_lock_if(false)
            .alt_if(false)
            .super_if(false)
            .release_if(false)
            .build();
        assert_eq!(keycode::KEY_A, evt.code);
        assert_eq!(keysym::SYM_LOWER_A, evt.ksym);
        assert!(evt.is_flag_on(KeyboardEvent::SHIFT_MASK));
        assert!(evt.is_flag_on(KeyboardEvent::CONTROL_MASK));
        assert!(!evt.is_flag_on(KeyboardEvent::CAPSLOCK_MASK));
    }
}
