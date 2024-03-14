use std::{
    ffi::CString,
    io::{stdin, Read},
    ptr::null_mut,
};

use chewing::capi::{
    input::*,
    setup::{chewing_delete, chewing_new2},
};

enum ChewingHandle {
    Default,
    Backspace,
    Capslock,
    CtrlNum,
    Del,
    Enter,
    Esc,
    Space,
    Tab,
    Home,
    End,
    Left,
    Right,
    Up,
    Down,
    ShiftLeft,
    ShiftRight,
    ShiftSpace,
    PageUp,
    PageDown,
    DblTab,
    Numlock,
    Quit,
    Skip,
}

impl From<u8> for ChewingHandle {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Default,
            1 => Self::Backspace,
            2 => Self::Capslock,
            3 => Self::CtrlNum,
            4 => Self::Del,
            5 => Self::Enter,
            6 => Self::Esc,
            7 => Self::Space,
            8 => Self::Tab,
            9 => Self::Home,
            10 => Self::End,
            11 => Self::Left,
            12 => Self::Right,
            13 => Self::Up,
            14 => Self::Down,
            15 => Self::ShiftLeft,
            16 => Self::ShiftRight,
            17 => Self::ShiftSpace,
            18 => Self::PageUp,
            19 => Self::PageDown,
            20 => Self::DblTab,
            21 => Self::Numlock,
            22 => Self::Quit,
            _ => Self::Skip,
        }
    }
}

pub fn main() {
    env_logger::init();

    let flags = xflags::parse_or_exit! {
        /// system library path
        required syspath: String
    };
    let syspath = CString::new(flags.syspath).unwrap();

    let ctx = chewing_new2(
        syspath.as_ptr(),
        b":memory:\0".as_ptr().cast(),
        None,
        null_mut(),
    );

    let mut ops = stdin().bytes();
    while let Some(Ok(op)) = ops.next() {
        use ChewingHandle::*;

        match ChewingHandle::from(op) {
            Default => {
                if let Some(Ok(key)) = ops.next() {
                    if key.is_ascii() && !key.is_ascii_control() {
                        chewing_handle_Default(ctx, key as i32);
                    }
                }
            }
            Backspace => {
                chewing_handle_Backspace(ctx);
            }
            Capslock => {
                chewing_handle_Capslock(ctx);
            }
            CtrlNum => {
                if let Some(Ok(key)) = ops.next() {
                    if key.is_ascii_digit() {
                        chewing_handle_CtrlNum(ctx, key as i32);
                    }
                }
            }
            Del => {
                chewing_handle_Del(ctx);
            }
            Enter => {
                chewing_handle_Enter(ctx);
            }
            Esc => {
                chewing_handle_Esc(ctx);
            }
            Space => {
                chewing_handle_Space(ctx);
            }
            Tab => {
                chewing_handle_Tab(ctx);
            }
            Home => {
                chewing_handle_Home(ctx);
            }
            End => {
                chewing_handle_End(ctx);
            }
            Left => {
                chewing_handle_Left(ctx);
            }
            Right => {
                chewing_handle_Right(ctx);
            }
            Up => {
                chewing_handle_Up(ctx);
            }
            Down => {
                chewing_handle_Down(ctx);
            }
            ShiftLeft => {
                chewing_handle_ShiftLeft(ctx);
            }
            ShiftRight => {
                chewing_handle_ShiftRight(ctx);
            }
            ShiftSpace => {
                chewing_handle_ShiftSpace(ctx);
            }
            PageUp => {
                chewing_handle_PageUp(ctx);
            }
            PageDown => {
                chewing_handle_PageDown(ctx);
            }
            DblTab => {
                chewing_handle_DblTab(ctx);
            }
            Numlock => {
                if let Some(Ok(key)) = ops.next() {
                    if key.is_ascii_digit() {
                        chewing_handle_Numlock(ctx, key as i32);
                    }
                }
            }
            Quit => {
                chewing_delete(ctx);
                break;
            }
            Skip => (),
        }
    }
}
