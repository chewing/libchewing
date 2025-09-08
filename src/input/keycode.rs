//! Layout independent keycode based on xkbcommon

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct Keycode(pub u8);

impl Keycode {
    pub const VOID: Keycode = Keycode(0);
}
