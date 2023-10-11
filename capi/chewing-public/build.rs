use std::{
    env,
    fs::File,
    io::{Result, Write},
    path::PathBuf,
};

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    cbindgen::generate(crate_dir)
        .expect("Unable to generate C headers for Rust code")
        .write_to_file("include/chewing_rs.h");

    println!("cargo:rerun-if-env-changed=CMAKE_BINARY_DIR");
    if let Ok(cmake_build_dir) = env::var("CMAKE_BINARY_DIR") {
        let mut path = PathBuf::new();
        path.push(cmake_build_dir);
        path.push("symbols.map");
        let mut symbols_file = File::create(path).expect("open file");
        #[cfg(target_os = "linux")]
        write_version_script(&mut symbols_file).expect("writing to file");
        #[cfg(target_os = "macos")]
        write_exported_symbols(&mut symbols_file).expect("writing to file");
        #[cfg(target_os = "windows")]
        write_def(&mut symbols_file).expect("writing to file");
    }
}

#[cfg(target_os = "linux")]
fn write_version_script(f: &mut impl Write) -> Result<()> {
    for (version, symbol_list) in SYMBOLS {
        writeln!(f, "{version} {{")?;
        writeln!(f, "    global:")?;
        for sym in symbol_list.iter() {
            writeln!(f, "        {sym};")?;
        }
        writeln!(f, "}};")?;
    }
    writeln!(f, "LOCAL {{ local: *; }};")?;
    Ok(())
}

#[cfg(target_os = "windows")]
fn write_def(f: &mut impl Write) -> Result<()> {
    writeln!(f, "EXPORTS")?;
    for (version, symbol_list) in SYMBOLS {
        for sym in symbol_list.iter() {
            writeln!(f, "    {sym};")?;
        }
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn write_exported_symbols(f: &mut impl Write) -> Result<()> {
    for (version, symbol_list) in SYMBOLS {
        for sym in symbol_list.iter() {
            writeln!(f, "_{sym}")?;
        }
    }
    Ok(())
}

const SYMBOLS: &[(&str, &[&str])] = &[
    (
        "CHEWING_0.5",
        &[
            "chewing_aux_Check",
            "chewing_aux_Length",
            "chewing_aux_String",
            "chewing_aux_String_static",
            "chewing_bopomofo_Check",
            "chewing_bopomofo_String_static",
            "chewing_buffer_Check",
            "chewing_buffer_Len",
            "chewing_buffer_String",
            "chewing_buffer_String_static",
            "chewing_cand_CheckDone",
            "chewing_cand_ChoicePerPage",
            "chewing_cand_choose_by_index",
            "chewing_cand_close",
            "chewing_cand_CurrentPage",
            "chewing_cand_Enumerate",
            "chewing_cand_hasNext",
            "chewing_cand_list_first",
            "chewing_cand_list_has_next",
            "chewing_cand_list_has_prev",
            "chewing_cand_list_last",
            "chewing_cand_list_next",
            "chewing_cand_list_prev",
            "chewing_cand_open",
            "chewing_cand_String",
            "chewing_cand_string_by_index_static",
            "chewing_cand_String_static",
            "chewing_cand_TotalChoice",
            "chewing_cand_TotalPage",
            "chewing_clean_bopomofo_buf",
            "chewing_clean_preedit_buf",
            "chewing_commit_Check",
            "chewing_commit_preedit_buf",
            "chewing_commit_String",
            "chewing_commit_String_static",
            "chewing_Configure",
            "chewing_cursor_Current",
            "chewing_delete",
            "chewing_free",
            "chewing_get_addPhraseDirection",
            "chewing_get_autoLearn",
            "chewing_get_autoShiftCur",
            "chewing_get_candPerPage",
            "chewing_get_ChiEngMode",
            "chewing_get_easySymbolInput",
            "chewing_get_escCleanAllBuf",
            "chewing_get_hsuSelKeyType",
            "chewing_get_KBString",
            "chewing_get_KBType",
            "chewing_get_maxChiSymbolLen",
            "chewing_get_phoneSeq",
            "chewing_get_phoneSeqLen",
            "chewing_get_phraseChoiceRearward",
            "chewing_get_selKey",
            "chewing_get_ShapeMode",
            "chewing_get_spaceAsSelection",
            "chewing_handle_Backspace",
            "chewing_handle_Capslock",
            "chewing_handle_CtrlNum",
            "chewing_handle_DblTab",
            "chewing_handle_Default",
            "chewing_handle_Del",
            "chewing_handle_Down",
            "chewing_handle_End",
            "chewing_handle_Enter",
            "chewing_handle_Esc",
            "chewing_handle_Home",
            "chewing_handle_Left",
            "chewing_handle_Numlock",
            "chewing_handle_PageDown",
            "chewing_handle_PageUp",
            "chewing_handle_Right",
            "chewing_handle_ShiftLeft",
            "chewing_handle_ShiftRight",
            "chewing_handle_ShiftSpace",
            "chewing_handle_Space",
            "chewing_handle_Tab",
            "chewing_handle_Up",
            "chewing_Init",
            "chewing_interval_Enumerate",
            "chewing_interval_Get",
            "chewing_interval_hasNext",
            "chewing_KBStr2Num",
            "chewing_kbtype_Enumerate",
            "chewing_kbtype_hasNext",
            "chewing_kbtype_String",
            "chewing_kbtype_String_static",
            "chewing_kbtype_Total",
            "chewing_keystroke_CheckAbsorb",
            "chewing_keystroke_CheckIgnore",
            "chewing_new",
            "chewing_new2",
            "chewing_phone_to_bopomofo",
            "chewing_Reset",
            "chewing_set_addPhraseDirection",
            "chewing_set_autoLearn",
            "chewing_set_autoShiftCur",
            "chewing_set_candPerPage",
            "chewing_set_ChiEngMode",
            "chewing_set_easySymbolInput",
            "chewing_set_escCleanAllBuf",
            "chewing_set_hsuSelKeyType",
            "chewing_set_KBType",
            "chewing_set_logger",
            "chewing_set_maxChiSymbolLen",
            "chewing_set_phraseChoiceRearward",
            "chewing_set_selKey",
            "chewing_set_ShapeMode",
            "chewing_set_spaceAsSelection",
            "chewing_Terminate",
            "chewing_userphrase_add",
            "chewing_userphrase_enumerate",
            "chewing_userphrase_get",
            "chewing_userphrase_has_next",
            "chewing_userphrase_lookup",
            "chewing_userphrase_remove",
            "chewing_zuin_Check",
            "chewing_zuin_String",
        ],
    ),
    (
        "CHEWING_INTERNAL",
        &[
            "find_path_by_files",
            "get_search_path",
            "ueBytesFromChar",
            "ueConstStrSeek",
            "ueStrLen",
            "ueStrNBytes",
            "ueStrNCpy",
            "ueStrSeek",
            "ueStrStr",
            "UintFromPhone",
        ],
    ),
];
