use std::{
    env,
    ffi::CString,
    io::{stdin, Read, Result},
    ptr::null_mut,
};

use chewing_capi::{
    candidates::{
        chewing_cand_Enumerate, chewing_cand_String, chewing_cand_String_static,
        chewing_cand_TotalPage, chewing_cand_hasNext, chewing_set_candPerPage,
    },
    globals::{
        chewing_set_addPhraseDirection, chewing_set_autoLearn, chewing_set_autoShiftCur,
        chewing_set_easySymbolInput, chewing_set_escCleanAllBuf, chewing_set_maxChiSymbolLen,
        chewing_set_phraseChoiceRearward, chewing_set_spaceAsSelection,
    },
    input::*,
    layout::chewing_set_KBType,
    modes::{
        chewing_get_ChiEngMode, chewing_get_ShapeMode, chewing_set_ChiEngMode,
        chewing_set_ShapeMode,
    },
    output::{
        chewing_aux_Check, chewing_aux_Length, chewing_aux_String, chewing_aux_String_static,
        chewing_bopomofo_Check, chewing_bopomofo_String_static, chewing_buffer_Check,
        chewing_buffer_Len, chewing_buffer_String, chewing_buffer_String_static,
        chewing_commit_Check, chewing_commit_String, chewing_commit_String_static,
        chewing_cursor_Current, chewing_get_phoneSeq, chewing_get_phoneSeqLen,
        chewing_interval_Enumerate, chewing_interval_Get, chewing_interval_hasNext,
        chewing_keystroke_CheckAbsorb, chewing_keystroke_CheckIgnore, IntervalType,
    },
    setup::{chewing_Reset, chewing_delete, chewing_free, chewing_new2},
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
    Reset,
    ChiEngMode,
    ShapeMode,
}

impl From<u8> for ChewingHandle {
    fn from(value: u8) -> Self {
        match value % 25 {
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
            22 => Self::Reset,
            23 => Self::ChiEngMode,
            24 => Self::ShapeMode,
            _ => unreachable!(),
        }
    }
}

pub fn main() -> Result<()> {
    env_logger::init();

    let syspath = env::args()
        .nth(1)
        .expect("The required argument system dictionary path <PATH> is not provided.");
    let syspath = CString::new(syspath).unwrap();

    unsafe {
        let ctx = chewing_new2(
            syspath.as_ptr(),
            b":memory:\0".as_ptr().cast(),
            None,
            null_mut(),
        );

        let mut ops = stdin().bytes();

        // Take first few bytes as mode settings
        chewing_set_KBType(ctx, ops.next().transpose()?.unwrap_or_default().into());
        chewing_set_candPerPage(
            ctx,
            ops.next()
                .transpose()?
                .unwrap_or_default()
                .clamp(5, 10)
                .into(),
        );
        chewing_set_maxChiSymbolLen(
            ctx,
            ops.next()
                .transpose()?
                .unwrap_or_default()
                .clamp(0, 39)
                .into(),
        );
        chewing_set_addPhraseDirection(
            ctx,
            ops.next()
                .transpose()?
                .unwrap_or_default()
                .clamp(0, 1)
                .into(),
        );
        chewing_set_spaceAsSelection(
            ctx,
            ops.next()
                .transpose()?
                .unwrap_or_default()
                .clamp(0, 1)
                .into(),
        );
        chewing_set_escCleanAllBuf(
            ctx,
            ops.next()
                .transpose()?
                .unwrap_or_default()
                .clamp(0, 1)
                .into(),
        );
        chewing_set_autoShiftCur(
            ctx,
            ops.next()
                .transpose()?
                .unwrap_or_default()
                .clamp(0, 1)
                .into(),
        );
        chewing_set_easySymbolInput(
            ctx,
            ops.next()
                .transpose()?
                .unwrap_or_default()
                .clamp(0, 1)
                .into(),
        );
        chewing_set_phraseChoiceRearward(
            ctx,
            ops.next()
                .transpose()?
                .unwrap_or_default()
                .clamp(0, 1)
                .into(),
        );
        chewing_set_autoLearn(
            ctx,
            ops.next()
                .transpose()?
                .unwrap_or_default()
                .clamp(0, 1)
                .into(),
        );

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
                Reset => {
                    chewing_Reset(ctx);
                }
                ChiEngMode => {
                    if let Some(Ok(key)) = ops.next() {
                        chewing_set_ChiEngMode(ctx, (key % 2) as i32);
                    }
                }
                ShapeMode => {
                    if let Some(Ok(key)) = ops.next() {
                        chewing_set_ShapeMode(ctx, (key % 2) as i32);
                    }
                }
            }
            chewing_get_ChiEngMode(ctx);
            chewing_get_ShapeMode(ctx);
            if chewing_aux_Check(ctx) == 1 {
                chewing_aux_Length(ctx);
                chewing_aux_String_static(ctx);
                chewing_free(chewing_aux_String(ctx).cast());
            }
            if chewing_bopomofo_Check(ctx) == 1 {
                chewing_bopomofo_String_static(ctx);
            }
            if chewing_buffer_Check(ctx) == 1 {
                chewing_buffer_Len(ctx);
                chewing_buffer_String_static(ctx);
                chewing_free(chewing_buffer_String(ctx).cast());
                let mut it = IntervalType { from: 0, to: 0 };
                chewing_interval_Enumerate(ctx);
                while chewing_interval_hasNext(ctx) == 1 {
                    chewing_interval_Get(ctx, std::ptr::addr_of_mut!(it));
                }
            }
            if chewing_cand_TotalPage(ctx) != 0 {
                chewing_cand_Enumerate(ctx);
                while chewing_cand_hasNext(ctx) == 1 {
                    chewing_cand_String_static(ctx);
                    chewing_free(chewing_cand_String(ctx).cast());
                }
            }
            if chewing_commit_Check(ctx) == 1 {
                chewing_commit_String_static(ctx);
                chewing_free(chewing_commit_String(ctx).cast());
            }
            chewing_cursor_Current(ctx);
            chewing_get_phoneSeqLen(ctx);
            chewing_free(chewing_get_phoneSeq(ctx).cast());
            chewing_keystroke_CheckAbsorb(ctx);
            chewing_keystroke_CheckIgnore(ctx);
        }

        chewing_delete(ctx);
    }

    Ok(())
}
