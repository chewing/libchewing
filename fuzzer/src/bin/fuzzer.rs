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
        chewing_config_set_int, chewing_set_addPhraseDirection, chewing_set_autoLearn,
        chewing_set_autoShiftCur, chewing_set_easySymbolInput, chewing_set_escCleanAllBuf,
        chewing_set_maxChiSymbolLen, chewing_set_phraseChoiceRearward,
        chewing_set_spaceAsSelection,
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

    let gen = env::var("GEN").is_ok();

    unsafe {
        let ctx = chewing_new2(
            syspath.as_ptr(),
            c":memory:".as_ptr().cast(),
            None,
            null_mut(),
        );

        let mut ops = stdin().bytes();

        // Take first few bytes as mode settings
        let kb_type = ops.next().transpose()?.unwrap_or_default().into();
        let cand_per_page = ops
            .next()
            .transpose()?
            .unwrap_or_default()
            .clamp(5, 10)
            .into();
        let max_chi_symbol_len = ops
            .next()
            .transpose()?
            .unwrap_or_default()
            .clamp(0, 39)
            .into();
        let add_phrase_direction = ops
            .next()
            .transpose()?
            .unwrap_or_default()
            .clamp(0, 1)
            .into();
        let space_as_selection = ops
            .next()
            .transpose()?
            .unwrap_or_default()
            .clamp(0, 1)
            .into();
        let esc_clean_all = ops
            .next()
            .transpose()?
            .unwrap_or_default()
            .clamp(0, 1)
            .into();
        let auto_shift_cur = ops
            .next()
            .transpose()?
            .unwrap_or_default()
            .clamp(0, 1)
            .into();
        let easy_symbol_input = ops
            .next()
            .transpose()?
            .unwrap_or_default()
            .clamp(0, 1)
            .into();
        let phrase_choice_rearward = ops
            .next()
            .transpose()?
            .unwrap_or_default()
            .clamp(0, 1)
            .into();
        let auto_learn = ops
            .next()
            .transpose()?
            .unwrap_or_default()
            .clamp(0, 1)
            .into();
        let fullwidth_toggle = ops
            .next()
            .transpose()?
            .unwrap_or_default()
            .clamp(0, 1)
            .into();
        let conversion_engine = ops
            .next()
            .transpose()?
            .unwrap_or_default()
            .clamp(0, 2)
            .into();
        chewing_set_KBType(ctx, kb_type);
        chewing_set_candPerPage(ctx, cand_per_page);
        chewing_set_maxChiSymbolLen(ctx, max_chi_symbol_len);
        chewing_set_addPhraseDirection(ctx, add_phrase_direction);
        chewing_set_spaceAsSelection(ctx, space_as_selection);
        chewing_set_escCleanAllBuf(ctx, esc_clean_all);
        chewing_set_autoShiftCur(ctx, auto_shift_cur);
        chewing_set_easySymbolInput(ctx, easy_symbol_input);
        chewing_set_phraseChoiceRearward(ctx, phrase_choice_rearward);
        chewing_set_autoLearn(ctx, auto_learn);
        chewing_config_set_int(
            ctx,
            c"chewing.enable_fullwidth_toggle_key".as_ptr(),
            fullwidth_toggle,
        );
        chewing_config_set_int(
            ctx,
            c"chewing.conversion_engine".as_ptr(),
            conversion_engine,
        );
        if gen {
            println!("# chewing_set_KBType(ctx, {});", kb_type);
            println!("# chewing_set_candPerPage(ctx, {});", cand_per_page);
            println!(
                "# chewing_set_maxChiSymbolLen(ctx, {});",
                max_chi_symbol_len
            );
            println!(
                "# chewing_set_addPhraseDirection(ctx, {});",
                add_phrase_direction
            );
            println!(
                "# chewing_set_spaceAsSelection(ctx, {});",
                space_as_selection
            );
            println!("# chewing_set_escCleanAllBuf(ctx, {});", esc_clean_all);
            println!("# chewing_set_autoShiftCur(ctx, {});", auto_shift_cur);
            println!("# chewing_set_easySymbolInput(ctx, {});", easy_symbol_input);
            println!(
                "# chewing_set_phraseChoiceRearward(ctx, {});",
                phrase_choice_rearward
            );
            println!("# chewing_set_autoLearn(ctx, {});", auto_learn);
            println!(
                "# chewing.enable_fullwidth_toggle_key = {}",
                fullwidth_toggle
            );
            println!("# chewing.conversion_engine = {}", conversion_engine);
            println!()
        }

        while let Some(Ok(op)) = ops.next() {
            use ChewingHandle::*;

            match ChewingHandle::from(op) {
                Default => {
                    if let Some(Ok(key)) = ops.next() {
                        if key.is_ascii() && !key.is_ascii_control() {
                            if gen {
                                print!("{}", char::from(key));
                            }
                            chewing_handle_Default(ctx, key as i32);
                        }
                    }
                }
                Backspace => {
                    if gen {
                        print!("<B>");
                    }
                    chewing_handle_Backspace(ctx);
                }
                Capslock => {
                    if gen {
                        print!("<CB>");
                    }
                    chewing_handle_Capslock(ctx);
                }
                CtrlNum => {
                    if let Some(Ok(key)) = ops.next() {
                        if key.is_ascii_digit() {
                            if gen {
                                print!("<C{}>", key);
                            }
                            chewing_handle_CtrlNum(ctx, key as i32);
                        }
                    }
                }
                Del => {
                    if gen {
                        print!("<DC>");
                    }
                    chewing_handle_Del(ctx);
                }
                Enter => {
                    if gen {
                        print!("<E>");
                    }
                    chewing_handle_Enter(ctx);
                }
                Esc => {
                    if gen {
                        print!("<EE>");
                    }
                    chewing_handle_Esc(ctx);
                }
                Space => {
                    if gen {
                        print!(" ");
                    }
                    chewing_handle_Space(ctx);
                }
                Tab => {
                    if gen {
                        print!("<T>");
                    }
                    chewing_handle_Tab(ctx);
                }
                Home => {
                    if gen {
                        print!("<H>");
                    }
                    chewing_handle_Home(ctx);
                }
                End => {
                    if gen {
                        print!("<EN>");
                    }
                    chewing_handle_End(ctx);
                }
                Left => {
                    if gen {
                        print!("<L>");
                    }
                    chewing_handle_Left(ctx);
                }
                Right => {
                    if gen {
                        print!("<R>");
                    }
                    chewing_handle_Right(ctx);
                }
                Up => {
                    if gen {
                        print!("<U>");
                    }
                    chewing_handle_Up(ctx);
                }
                Down => {
                    if gen {
                        print!("<D>");
                    }
                    chewing_handle_Down(ctx);
                }
                ShiftLeft => {
                    if gen {
                        print!("<SL>");
                    }
                    chewing_handle_ShiftLeft(ctx);
                }
                ShiftRight => {
                    if gen {
                        print!("<SR>");
                    }
                    chewing_handle_ShiftRight(ctx);
                }
                ShiftSpace => {
                    if gen {
                        print!("<SS>");
                    }
                    chewing_handle_ShiftSpace(ctx);
                }
                PageUp => {
                    if gen {
                        print!("<PU>");
                    }
                    chewing_handle_PageUp(ctx);
                }
                PageDown => {
                    if gen {
                        print!("<PD>");
                    }
                    chewing_handle_PageDown(ctx);
                }
                DblTab => {
                    if gen {
                        print!("<TT>");
                    }
                    chewing_handle_DblTab(ctx);
                }
                Numlock => {
                    if let Some(Ok(key)) = ops.next() {
                        if key.is_ascii_digit() {
                            if gen {
                                print!("<N{}>", key);
                            }
                            chewing_handle_Numlock(ctx, key as i32);
                        }
                    }
                }
                Reset => {
                    if gen {
                        print!("\n# chewing_Reset(ctx);\n");
                    }
                    chewing_Reset(ctx);
                }
                ChiEngMode => {
                    if let Some(Ok(key)) = ops.next() {
                        if gen {
                            print!("\n# chewing_set_ChiEngMode(ctx, {});\n", key % 2);
                        }
                        chewing_set_ChiEngMode(ctx, (key % 2) as i32);
                    }
                }
                ShapeMode => {
                    if let Some(Ok(key)) = ops.next() {
                        if gen {
                            print!("\n# chewing_set_ShapeMode(ctx, {});\n", key % 2);
                        }
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
        if gen {
            println!();
        }
    }

    Ok(())
}
