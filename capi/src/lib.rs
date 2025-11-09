#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
#![allow(deprecated)]

//! C compatible APIs for libchewing.
//!
//! All items are available via the C header file `<chewing.h>`.
//!
//! Function symbols are exposed from the libchewing shared library.
//!
//! Functions are organized into several modules according to the services
//! provided by them.
//!
//! # Overview
//!
//! As far as we expect, input method (in fact, the way for text input and
//! output, especially in multi-lingual environments) implementations are
//! becoming more and more complex in system integrations.  The emerging
//! standard for practical and flexible development cycles is required, so that
//! we are facing the impacts from various Chinese input method implementations
//! and the integration into existing framework or system designs.  At the
//! result, Chewing Input Method (abbreviated as "Chewing") attempts to be one
//! of the approaches to solve and ease the problems with the inclusion of base
//! classes, on the basis of cross-platform, and cross-operating-environment
//! derivative classes, which are built as the abstract backbone of intelligent
//! Chinese phonetic input method implementations.  From the perspectives,
//! Chewing defines the abstract behavior how an intelligent phonetic IM should
//! works via the common interface, and Chewing permits the extra parameters and
//! properties for the input method implementations to extend their specific
//! functionality.

mod io;
mod logger;
mod public;

pub mod version;

/// Initializes chewing context and environment settings.
///
/// Most of the Chewing IM APIs require a
/// [ChewingContext][setup::ChewingContext]. To create a ChewingContext you must
/// use the [chewing_new][setup::chewing_new] function.
///
/// # Examples
///
/// Create a chewing context and deletes it after use.
///
/// ```c
/// #include <chewing.h>
/// int main(int argc, char *argv[])
/// {
///     ChewingContext *ctx = chewing_new();
///
///     // do something
///
///     chewing_delete( ctx );
///
///     return 0;
/// }
/// ```
///
/// # Environment variables
///
/// * `CHEWING_PATH`
///     * The CHEWING_PATH environment variable is used to set the search path
///       of static data used by the Chewing IM. The format of CHEWING_PATH is
///       the same as PATH, which is multiple paths separated by ‘:’ on POSIX
///       and Unix-like platforms, or separated by ‘;’ on Windows platform. The
///       directories in CHEWING_PATH could be read-only.
/// * `CHEWING_USER_PATH`
///     * The CHEWING_USER_PATH environment variable is used to specifies the
///       path where user-defined hash data stores. This path should be writable
///       by the user, or the Chewing IM will lose the ability to remember the
///       learned phrases.
pub mod setup {
    pub use super::io::chewing_Configure;
    pub use super::io::chewing_Init;
    pub use super::io::chewing_Reset;
    pub use super::io::chewing_Terminate;
    pub use super::io::chewing_delete;
    pub use super::io::chewing_free;
    pub use super::io::chewing_get_defaultDictionaryNames;
    pub use super::io::chewing_new;
    pub use super::io::chewing_new2;
    pub use super::io::chewing_new3;
    pub use super::io::chewing_set_logger;
    pub use super::public::CHEWING_LOG_DEBUG;
    pub use super::public::CHEWING_LOG_ERROR;
    pub use super::public::CHEWING_LOG_INFO;
    pub use super::public::CHEWING_LOG_VERBOSE;
    pub use super::public::CHEWING_LOG_WARN;
    pub use super::public::ChewingContext;
}

/// Keyboard input handling.
///
/// Functions to handle key strokes. The return value of these functions is 0 on
/// success and -1 on failure.
pub mod input {
    pub use super::io::chewing_handle_Backspace;
    pub use super::io::chewing_handle_Capslock;
    pub use super::io::chewing_handle_CtrlNum;
    pub use super::io::chewing_handle_DblTab;
    pub use super::io::chewing_handle_Default;
    pub use super::io::chewing_handle_Del;
    pub use super::io::chewing_handle_Down;
    pub use super::io::chewing_handle_End;
    pub use super::io::chewing_handle_Enter;
    pub use super::io::chewing_handle_Esc;
    pub use super::io::chewing_handle_Home;
    pub use super::io::chewing_handle_KeyboardEvent;
    pub use super::io::chewing_handle_Left;
    pub use super::io::chewing_handle_Numlock;
    pub use super::io::chewing_handle_PageDown;
    pub use super::io::chewing_handle_PageUp;
    pub use super::io::chewing_handle_Right;
    pub use super::io::chewing_handle_ShiftLeft;
    pub use super::io::chewing_handle_ShiftRight;
    pub use super::io::chewing_handle_ShiftSpace;
    pub use super::io::chewing_handle_Space;
    pub use super::io::chewing_handle_Tab;
    pub use super::io::chewing_handle_Up;
    pub use super::public::KEYSTROKE_ABSORB;
    pub use super::public::KEYSTROKE_BELL;
    pub use super::public::KEYSTROKE_COMMIT;
    pub use super::public::KEYSTROKE_IGNORE;
}

/// Keyboard layout and variants setting.
///
/// The Chewing IM supports many different keyboard layout and variants. Use
/// functions in this module to set the current keyboard layout for the context.
pub mod layout {
    pub use super::io::chewing_KBStr2Num;
    pub use super::io::chewing_get_KBString;
    pub use super::io::chewing_get_KBType;
    pub use super::io::chewing_kbtype_Enumerate;
    pub use super::io::chewing_kbtype_String;
    pub use super::io::chewing_kbtype_String_static;
    pub use super::io::chewing_kbtype_Total;
    pub use super::io::chewing_kbtype_hasNext;
    pub use super::io::chewing_set_KBType;
    pub use super::public::KB;
}

/// Input mode settings.
///
/// The Chewing IM can switch between Chinese input mode or English mode. The
/// English mode supports input English characters directly. These functions set
/// the current input mode.
pub mod modes {
    pub use super::io::chewing_get_ChiEngMode;
    pub use super::io::chewing_get_ShapeMode;
    pub use super::io::chewing_set_ChiEngMode;
    pub use super::io::chewing_set_ShapeMode;
    pub use super::public::CHINESE_MODE;
    pub use super::public::FULLSHAPE_MODE;
    pub use super::public::HALFSHAPE_MODE;
    pub use super::public::SYMBOL_MODE;
}

/// Candidate selection related functions.
///
/// These functions can be used to transit the Chewing IM into candidate
/// selection state and enumerate candidates.
///
/// # Keyboardless APIs
///
/// The traditional chewing APIs are coupled to keyboards. They cause some
/// problems if the program like to design its own keyboard scheme, or if a
/// platform does not have certain keyboard keys (ex: mobile device). To
/// overcome these problems, the new keyboardless APIs are provided. With these
/// APIs, program can have better control over libchewing, instead of hacking
/// libchewing via fake keyboard event.
pub mod candidates {
    pub use super::io::chewing_cand_CheckDone;
    pub use super::io::chewing_cand_ChoicePerPage;
    pub use super::io::chewing_cand_CurrentPage;
    pub use super::io::chewing_cand_Enumerate;
    pub use super::io::chewing_cand_String;
    pub use super::io::chewing_cand_String_static;
    pub use super::io::chewing_cand_TotalChoice;
    pub use super::io::chewing_cand_TotalPage;
    pub use super::io::chewing_cand_choose_by_index;
    pub use super::io::chewing_cand_close;
    pub use super::io::chewing_cand_hasNext;
    pub use super::io::chewing_cand_list_first;
    pub use super::io::chewing_cand_list_has_next;
    pub use super::io::chewing_cand_list_has_prev;
    pub use super::io::chewing_cand_list_last;
    pub use super::io::chewing_cand_list_next;
    pub use super::io::chewing_cand_list_prev;
    pub use super::io::chewing_cand_open;
    pub use super::io::chewing_cand_string_by_index;
    pub use super::io::chewing_cand_string_by_index_static;
    pub use super::io::chewing_get_candPerPage;
    pub use super::io::chewing_get_hsuSelKeyType;
    pub use super::io::chewing_get_selKey;
    pub use super::io::chewing_set_candPerPage;
    pub use super::io::chewing_set_hsuSelKeyType;
    pub use super::io::chewing_set_selKey;
    pub use super::public::HSU_SELKEY_TYPE1;
    pub use super::public::HSU_SELKEY_TYPE2;
    pub use super::public::MAX_SELKEY;
    pub use super::public::MIN_SELKEY;
}

/// Output handling.
pub mod output {
    pub use super::io::chewing_ack;
    pub use super::io::chewing_aux_Check;
    pub use super::io::chewing_aux_Length;
    pub use super::io::chewing_aux_String;
    pub use super::io::chewing_aux_String_static;
    pub use super::io::chewing_bopomofo_Check;
    pub use super::io::chewing_bopomofo_String;
    pub use super::io::chewing_bopomofo_String_static;
    pub use super::io::chewing_buffer_Check;
    pub use super::io::chewing_buffer_Len;
    pub use super::io::chewing_buffer_String;
    pub use super::io::chewing_buffer_String_static;
    pub use super::io::chewing_clean_bopomofo_buf;
    pub use super::io::chewing_clean_preedit_buf;
    pub use super::io::chewing_commit_Check;
    pub use super::io::chewing_commit_String;
    pub use super::io::chewing_commit_String_static;
    pub use super::io::chewing_commit_preedit_buf;
    pub use super::io::chewing_cursor_Current;
    pub use super::io::chewing_get_phoneSeq;
    pub use super::io::chewing_get_phoneSeqLen;
    pub use super::io::chewing_interval_Enumerate;
    pub use super::io::chewing_interval_Get;
    pub use super::io::chewing_interval_hasNext;
    pub use super::io::chewing_keystroke_CheckAbsorb;
    pub use super::io::chewing_keystroke_CheckIgnore;
    pub use super::io::chewing_phone_to_bopomofo;
    pub use super::io::chewing_zuin_Check;
    pub use super::io::chewing_zuin_String;
    pub use super::public::IntervalType;
}

/// Userphrase handling.
pub mod userphrase {
    pub use super::io::chewing_userphrase_add;
    pub use super::io::chewing_userphrase_enumerate;
    pub use super::io::chewing_userphrase_get;
    pub use super::io::chewing_userphrase_has_next;
    pub use super::io::chewing_userphrase_lookup;
    pub use super::io::chewing_userphrase_remove;
}

/// Global settings.
///
/// The Chewing IM could be customized in some small details. These functions
/// provide the configuration interfaces to the front-end.
pub mod globals {
    pub use super::io::chewing_config_get_int;
    pub use super::io::chewing_config_get_str;
    pub use super::io::chewing_config_has_option;
    pub use super::io::chewing_config_set_int;
    pub use super::io::chewing_config_set_str;
    pub use super::io::chewing_get_addPhraseDirection;
    pub use super::io::chewing_get_autoLearn;
    pub use super::io::chewing_get_autoShiftCur;
    pub use super::io::chewing_get_easySymbolInput;
    pub use super::io::chewing_get_escCleanAllBuf;
    pub use super::io::chewing_get_maxChiSymbolLen;
    pub use super::io::chewing_get_phraseChoiceRearward;
    pub use super::io::chewing_get_spaceAsSelection;
    pub use super::io::chewing_set_addPhraseDirection;
    pub use super::io::chewing_set_autoLearn;
    pub use super::io::chewing_set_autoShiftCur;
    pub use super::io::chewing_set_easySymbolInput;
    pub use super::io::chewing_set_escCleanAllBuf;
    pub use super::io::chewing_set_maxChiSymbolLen;
    pub use super::io::chewing_set_phraseChoiceRearward;
    pub use super::io::chewing_set_spaceAsSelection;
    pub use super::public::AUTOLEARN_DISABLED;
    pub use super::public::AUTOLEARN_ENABLED;
    pub use super::public::MAX_CHI_SYMBOL_LEN;
    pub use super::public::MAX_PHONE_SEQ_LEN;
    pub use super::public::MAX_PHRASE_LEN;
    pub use super::public::MIN_CHI_SYMBOL_LEN;
}
