//! Layout independent keycode based on xkbcommon

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct Keycode(pub u8);

impl Keycode {
    pub const KEY_ESC: Keycode = Keycode(9);
    pub const KEY_1: Keycode = Keycode(10);
    pub const KEY_2: Keycode = Keycode(11);
    pub const KEY_3: Keycode = Keycode(12);
    pub const KEY_4: Keycode = Keycode(13);
    pub const KEY_5: Keycode = Keycode(14);
    pub const KEY_6: Keycode = Keycode(15);
    pub const KEY_7: Keycode = Keycode(16);
    pub const KEY_8: Keycode = Keycode(17);
    pub const KEY_9: Keycode = Keycode(18);
    pub const KEY_0: Keycode = Keycode(19);
    pub const KEY_MINUS: Keycode = Keycode(20);
    pub const KEY_EQUAL: Keycode = Keycode(21);
    pub const KEY_BACKSPACE: Keycode = Keycode(22);
    pub const KEY_TAB: Keycode = Keycode(23);
    pub const KEY_Q: Keycode = Keycode(24);
    pub const KEY_W: Keycode = Keycode(25);
    pub const KEY_E: Keycode = Keycode(26);
    pub const KEY_R: Keycode = Keycode(27);
    pub const KEY_T: Keycode = Keycode(28);
    pub const KEY_Y: Keycode = Keycode(29);
    pub const KEY_U: Keycode = Keycode(30);
    pub const KEY_I: Keycode = Keycode(31);
    pub const KEY_O: Keycode = Keycode(32);
    pub const KEY_P: Keycode = Keycode(33);
    pub const KEY_LEFTBRACE: Keycode = Keycode(34);
    pub const KEY_RIGHTBRACE: Keycode = Keycode(35);
    pub const KEY_ENTER: Keycode = Keycode(36);
    pub const KEY_LEFTCTRL: Keycode = Keycode(37);
    pub const KEY_A: Keycode = Keycode(38);
    pub const KEY_S: Keycode = Keycode(39);
    pub const KEY_D: Keycode = Keycode(40);
    pub const KEY_F: Keycode = Keycode(41);
    pub const KEY_G: Keycode = Keycode(42);
    pub const KEY_H: Keycode = Keycode(43);
    pub const KEY_J: Keycode = Keycode(44);
    pub const KEY_K: Keycode = Keycode(45);
    pub const KEY_L: Keycode = Keycode(46);
    pub const KEY_SEMICOLON: Keycode = Keycode(47);
    pub const KEY_APOSTROPHE: Keycode = Keycode(48);
    pub const KEY_GRAVE: Keycode = Keycode(49);
    pub const KEY_LEFTSHIFT: Keycode = Keycode(50);
    pub const KEY_BACKSLASH: Keycode = Keycode(51);
    pub const KEY_Z: Keycode = Keycode(52);
    pub const KEY_X: Keycode = Keycode(53);
    pub const KEY_C: Keycode = Keycode(54);
    pub const KEY_V: Keycode = Keycode(55);
    pub const KEY_B: Keycode = Keycode(56);
    pub const KEY_N: Keycode = Keycode(57);
    pub const KEY_M: Keycode = Keycode(58);
    pub const KEY_COMMA: Keycode = Keycode(59);
    pub const KEY_DOT: Keycode = Keycode(60);
    pub const KEY_SLASH: Keycode = Keycode(61);
    pub const KEY_RIGHTSHIFT: Keycode = Keycode(62);
    pub const KEY_KPASTERISK: Keycode = Keycode(63);
    pub const KEY_LEFTALT: Keycode = Keycode(64);
    pub const KEY_SPACE: Keycode = Keycode(65);
    pub const KEY_CAPSLOCK: Keycode = Keycode(66);
    pub const KEY_F1: Keycode = Keycode(67);
    pub const KEY_F2: Keycode = Keycode(68);
    pub const KEY_F3: Keycode = Keycode(69);
    pub const KEY_F4: Keycode = Keycode(70);
    pub const KEY_F5: Keycode = Keycode(71);
    pub const KEY_F6: Keycode = Keycode(72);
    pub const KEY_F7: Keycode = Keycode(73);
    pub const KEY_F8: Keycode = Keycode(74);
    pub const KEY_F9: Keycode = Keycode(75);
    pub const KEY_F10: Keycode = Keycode(76);
    pub const KEY_NUMLOCK: Keycode = Keycode(77);
    pub const KEY_SCROLLLOCK: Keycode = Keycode(78);
    pub const KEY_KP7: Keycode = Keycode(79);
    pub const KEY_KP8: Keycode = Keycode(80);
    pub const KEY_KP9: Keycode = Keycode(81);
    pub const KEY_KPMINUS: Keycode = Keycode(82);
    pub const KEY_KP4: Keycode = Keycode(83);
    pub const KEY_KP5: Keycode = Keycode(84);
    pub const KEY_KP6: Keycode = Keycode(85);
    pub const KEY_KPPLUS: Keycode = Keycode(86);
    pub const KEY_KP1: Keycode = Keycode(87);
    pub const KEY_KP2: Keycode = Keycode(88);
    pub const KEY_KP3: Keycode = Keycode(89);
    pub const KEY_KP0: Keycode = Keycode(90);
    pub const KEY_KPDOT: Keycode = Keycode(91);

    pub const KEY_F11: Keycode = Keycode(95);
    pub const KEY_F12: Keycode = Keycode(96);

    pub const KEY_KPENTER: Keycode = Keycode(104);
    pub const KEY_RIGHTCTRL: Keycode = Keycode(105);
    pub const KEY_KPSLASH: Keycode = Keycode(106);
    pub const KEY_SYSRQ: Keycode = Keycode(107);
    pub const KEY_RIGHTALT: Keycode = Keycode(108);
    pub const KEY_LINEFEED: Keycode = Keycode(109);
    pub const KEY_HOME: Keycode = Keycode(110);
    pub const KEY_UP: Keycode = Keycode(111);
    pub const KEY_PAGEUP: Keycode = Keycode(112);
    pub const KEY_LEFT: Keycode = Keycode(113);
    pub const KEY_RIGHT: Keycode = Keycode(114);
    pub const KEY_END: Keycode = Keycode(115);
    pub const KEY_DOWN: Keycode = Keycode(116);
    pub const KEY_PAGEDOWN: Keycode = Keycode(117);
    pub const KEY_INSERT: Keycode = Keycode(118);
    pub const KEY_DELETE: Keycode = Keycode(119);

    pub const KEY_KPEQUAL: Keycode = Keycode(125);
    pub const KEY_KPPLUSMINUS: Keycode = Keycode(126);

    pub const KEY_KPCOMMA: Keycode = Keycode(129);

    pub const KEY_LEFTMETA: Keycode = Keycode(133);
    pub const KEY_RIGHTMETA: Keycode = Keycode(134);

    pub const KEY_KPLEFTPAREN: Keycode = Keycode(187);
    pub const KEY_KPRIGHTPAREN: Keycode = Keycode(188);
}
