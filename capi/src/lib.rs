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
///     /* do something */
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
    /// This function exists only for backword compatibility.
    ///
    /// The `chewing_Init` function is no-op now. The return value is always 0.
    pub use super::io::chewing_Init;

    /// This function exists only for backword compatibility.
    pub use super::io::chewing_Terminate;

    /// Creates a new instance of the Chewing IM.
    ///
    /// The return value is a pointer to the new Chewing IM instance.
    ///
    /// See also the [chewing_new2], and [chewing_delete] functions.
    pub use super::io::chewing_new;

    /// Creates a new instance of the Chewing IM.
    ///
    /// The `syspath` is the directory path to system dictionary. The `userpath`
    /// is file path to user dictionary. User shall have enough permission to
    /// update this file. The logger and loggerdata is logger function and its
    /// data.
    ///
    /// All parameters will be default if set to NULL.
    ///
    /// The return value is a pointer to the new Chewing IM instance. See also
    /// the [chewing_new], [chewing_delete] function.
    pub use super::io::chewing_new2;

    /// Releases the resources used by the given Chewing IM instance.
    pub use super::io::chewing_delete;

    /// Sets the selectAreaLen, maxChiSymbolLen and selKey parameter from pcd.
    ///
    /// The pcd argument is a pointer to a Chewing configuration data structure.
    /// See also the ChewingConfigData data type.
    ///
    /// The return value is 0 on success and -1 on failure.
    ///
    /// **Deprecated**, use the chewing_set_* function series to set parameters
    /// instead.
    pub use super::io::chewing_Configure;

    /// Resets all settings in the given Chewing IM instance.
    ///
    /// The return value is 0 on success and -1 on failure.
    pub use super::io::chewing_Reset;

    /// Releases the memory allocated by the Chewing IM and returned to the
    /// caller.
    ///
    /// There are functions returning pointers of strings or other data
    /// structures that are allocated on the heap. These memory must be freed to
    /// avoid memory leak. To avoid memory allocator mismatch between the
    /// Chewing IM and the caller, use this function to free the resource.
    ///
    /// Do nothing if ptr is NULL.
    pub use super::io::chewing_free;

    /// Sets the logger function logger.
    ///
    /// The logger function is used to provide log inside Chewing IM for
    /// debugging. The data in chewing_set_logger is passed directly to data in
    /// logger when logging.
    ///
    /// # Examples
    ///
    /// The following example shows how to use data:
    ///
    /// ```c
    /// void logger( void *data, int level, const char *fmt, ... )
    /// {
    ///     FILE *fd = (FILE *) data;
    ///     ...
    /// }
    ///
    /// int main()
    /// {
    ///     ChewingContext *ctx;
    ///     FILE *fd;
    ///     ...
    ///     chewing_set_logger(ctx, logger, fd);
    ///     ...
    /// }
    /// ```
    ///
    /// The level is log level.
    pub use super::io::chewing_set_logger;

    pub use super::public::ChewingContext;

    pub use super::public::CHEWING_LOG_VERBOSE;

    pub use super::public::CHEWING_LOG_DEBUG;

    pub use super::public::CHEWING_LOG_INFO;

    pub use super::public::CHEWING_LOG_WARN;

    pub use super::public::CHEWING_LOG_ERROR;
}

/// Keyboard input handling.
///
/// Functions to handle key strokes. The return value of these functions is 0 on
/// success and -1 on failure.
pub mod input {
    /// Handles all keys that do not have dedicated methods.
    ///
    /// The value of of key can be any printable ASCII characters.
    pub use super::io::chewing_handle_Default;

    /// Handles the Backspace key.
    pub use super::io::chewing_handle_Backspace;

    /// Handles the Capslock key.
    pub use super::io::chewing_handle_Capslock;

    /// Handles any number key with the Ctrl modifier.
    ///
    /// The value of key should be in the range between ASCII character code
    /// from 0 to 9.
    pub use super::io::chewing_handle_CtrlNum;

    /// Handles the Delete key.
    pub use super::io::chewing_handle_Del;

    /// Handles the Enter or Return key.
    pub use super::io::chewing_handle_Enter;

    /// Handles the Esc key.
    pub use super::io::chewing_handle_Esc;

    /// Handles the Space key.
    pub use super::io::chewing_handle_Space;

    /// Handles the Tab key.
    pub use super::io::chewing_handle_Tab;

    /// Handles the Home key.
    pub use super::io::chewing_handle_Home;

    /// Handles the End key.
    pub use super::io::chewing_handle_End;

    /// Handles the Left key.
    pub use super::io::chewing_handle_Left;

    /// Handles the Right key.
    pub use super::io::chewing_handle_Right;

    /// Handles the Up key.
    ///
    /// See also [chewing_cand_close][super::candidates::chewing_cand_close] keyboardless API to close candidate
    /// window.
    pub use super::io::chewing_handle_Up;

    /// Handles the Down key.
    ///
    /// See also [super::io::chewing_cand_open] keyboardless API to open candidate window.
    pub use super::io::chewing_handle_Down;

    /// Handles the Left key with the Shift modifier.
    pub use super::io::chewing_handle_ShiftLeft;

    /// Handles the Right key with the Shift modifier.
    pub use super::io::chewing_handle_ShiftRight;

    /// Handles the Space key with the Shift modifier.
    pub use super::io::chewing_handle_ShiftSpace;

    /// Handles the PageUp key.
    pub use super::io::chewing_handle_PageUp;

    /// Handles the PageDown key.
    pub use super::io::chewing_handle_PageDown;

    /// Handles tapping the Tab key twice quickly.
    pub use super::io::chewing_handle_DblTab;

    /// Handles any numeric key from the keypad.
    ///
    /// The value of key should be in the range between ASCII character code
    /// from 0 to 9.
    pub use super::io::chewing_handle_Numlock;

    pub use super::public::KEYSTROKE_IGNORE;

    pub use super::public::KEYSTROKE_COMMIT;

    pub use super::public::KEYSTROKE_BELL;

    pub use super::public::KEYSTROKE_ABSORB;
}

/// Keyboard layout and variants setting.
///
/// The Chewing IM supports many different keyboard layout and variants. Use
/// functions in this module to set the current keyboard layout for the context.
pub mod layout {
    /// Sets the current keyboard layout for ctx.
    ///
    /// The kbtype argument must be a value defined in [KB].
    ///
    /// The return value is 0 on success and -1 on failure. The keyboard type
    /// will set to KB_DEFAULT if return value is -1.
    pub use super::io::chewing_set_KBType;

    /// Returns the current keyboard layout index for ctx.
    ///
    /// The return value is the layout index defined in [KB].
    pub use super::io::chewing_get_KBType;

    /// Returns the the current layout name string of ctx.
    ///
    /// The return value is the name of the current layout, see also function
    /// [chewing_KBStr2Num].
    ///
    /// The returned pointer must be freed by
    /// [chewing_free][super::setup::chewing_free].
    ///
    /// # Failures
    ///
    /// This function returns NULL when memory allocation fails.
    pub use super::io::chewing_get_KBString;

    /// Converts the keyboard layout name from string to corresponding layout
    /// index.
    ///
    /// If the string does not match any layout, this function returns
    /// KB_DEFAULT.
    ///
    /// The string str might be one of the following layouts:
    /// * KB_DEFAULT
    /// * KB_HSU
    /// * KB_IBM
    /// * KB_GIN_YIEH
    /// * KB_ET
    /// * KB_ET26
    /// * KB_DVORAK
    /// * KB_DVORAK_HSU
    /// * KB_DVORAK_CP26
    /// * KB_HANYU_PINYIN
    /// * KB_THL_PINYIN
    /// * KB_MPS2_PINYIN
    /// * KB_CARPALX
    /// * KB_COLEMAK
    /// * KB_COLEMAK_DH_ANSI
    /// * KB_COLEMAK_DH_ORTH
    /// * KB_WORKMAN
    ///
    /// See also [chewing_kbtype_Enumerate] for getting the list of supported
    /// layouts programmatically.
    pub use super::io::chewing_KBStr2Num;

    /// Returns the number of keyboard layouts supported by the Chewing IM.
    pub use super::io::chewing_kbtype_Total;

    /// Starts the enumeration of the keyboard layouts.
    ///
    /// This function stores an iterator in the context. The iterator is only
    /// destroyed after enumerate all keyboard layouts using
    /// [chewing_kbtype_hasNext].
    pub use super::io::chewing_kbtype_Enumerate;

    /// Checks whether there are more keyboard layouts to enumerate.
    ///
    /// Returns 1 when there are more and 0 when it's the end of the iterator.
    pub use super::io::chewing_kbtype_hasNext;

    /// Returns the current enumerated keyboard layout name.
    ///
    /// The returned string is emtpy string when enumeration is over.
    ///
    /// The returned value is a pointer to a character string. The memory must
    /// be freed by the caller using function
    /// [chewing_free][super::setup::chewing_free].
    ///
    /// # Failures
    ///
    /// This function returns NULL when memory allocation fails.
    pub use super::io::chewing_kbtype_String;

    /// Returns the current enumerated keyboard layout name.
    ///
    /// The returned string is emtpy string when enumeration is over.
    ///
    /// The return value is a const pointer to a character string. The pointer
    /// is only valid immediately after checking the [chewing_kbtype_hasNext]
    /// condition.
    pub use super::io::chewing_kbtype_String_static;

    pub use super::public::KB;
}

/// Input mode settings.
///
/// The Chewing IM can switch between Chinese input mode or English mode. The
/// English mode supports input English characters directly. These functions set
/// the current input mode.
pub mod modes {
    /// Sets the input mode to Chinese or English.
    ///
    /// The *mode* argument is one of the [CHINESE_MODE] and [SYMBOL_MODE]
    /// constants.
    pub use super::io::chewing_set_ChiEngMode;

    /// Returns the current Chinese/English mode setting.
    pub use super::io::chewing_get_ChiEngMode;

    /// Sets the current punctuation input mode.
    ///
    /// The *mode* argument is one of the [FULLSHAPE_MODE] and [HALFSHAPE_MODE]
    /// constants.
    pub use super::io::chewing_set_ShapeMode;

    /// Returns the current punctuation mode.
    pub use super::io::chewing_get_ShapeMode;

    pub use super::public::CHINESE_MODE;
    pub use super::public::SYMBOL_MODE;

    pub use super::public::FULLSHAPE_MODE;
    pub use super::public::HALFSHAPE_MODE;
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
    /// Returns the number of pages of the candidates.
    ///
    /// If the return value is greater than zero, then the IM interface should
    /// display a selection window of the candidates for the user to choose a
    /// candidate. Otherwise hide the selection window.
    pub use super::io::chewing_cand_TotalPage;

    /// Returns the current candidate page number.
    ///
    /// # Examples
    ///
    /// The candidates pagination could be displayed as:
    ///
    /// ```c
    /// sprintf(buf, "[%d / %d]",
    ///     chewing_cand_CurrentPage(ctx),
    ///     chewing_cand_TotalPage(ctx));
    /// ```
    pub use super::io::chewing_cand_CurrentPage;

    /// Returns the number of the coices per page.
    ///
    /// See also the [chewing_set_candPerPage] function.
    pub use super::io::chewing_cand_ChoicePerPage;

    /// Returns the total number of the available choices.
    pub use super::io::chewing_cand_TotalChoice;

    /// Starts the enumeration of the candidates starting from the first one in
    /// the current page.
    ///
    /// This function stores an iterator in the context. The iterator is only
    /// destroyed after enumerate candidates using
    /// [chewing_cand_hasNext].
    pub use super::io::chewing_cand_Enumerate;

    /// Checks if there are more candidates to enumerate.
    ///
    /// <p style="background:rgba(255,181,77,0.16);padding:0.75em;">
    /// <strong>⚠ Warning:</strong> This function checks the end of total choices
    /// instead of the end of current page.
    /// </p>
    pub use super::io::chewing_cand_hasNext;

    /// Returns the current enumerated candidate string.
    ///
    /// The returned value is a pointer to a character string. The memory must
    /// be freed by the caller using function
    /// [chewing_free][super::setup::chewing_free].
    ///
    /// # Failures
    ///
    /// This function returns NULL when memory allocation fails.
    pub use super::io::chewing_cand_String;

    /// Returns the current enumerated candidate string.
    ///
    /// The returned string is emtpy string when enumeration is over.
    ///
    /// The return value is a const pointer to a character string. The pointer
    /// is only valid immediately after checking the [chewing_cand_hasNext]
    /// condition.
    pub use super::io::chewing_cand_String_static;

    /// Checks if the candidates selection has finished.
    ///
    /// <p style="background:rgba(255,181,77,0.16);padding:0.75em;">
    /// <strong>⚠ Warning:</strong> Not implemented.
    /// </p>
    pub use super::io::chewing_cand_CheckDone;

    /// Sets the number of candidates returned per page.
    ///
    /// The setting is ignored if *n* is not between [MIN_SELKEY] and
    /// [MAX_SELKEY] inclusive.
    ///
    /// The default value is MAX_SELKEY.
    pub use super::io::chewing_set_candPerPage;

    /// Gets the number of candidates returned per page.
    ///
    /// The default value is MAX_SELKEY.
    pub use super::io::chewing_get_candPerPage;

    /// Sets the key codes for candidate selection.
    ///
    /// *selkeys* is an ASCII code integer array of length [MAX_SELKEY]. The
    /// second argument is unused.
    ///
    /// The default selection key is `1234567890`.
    pub use super::io::chewing_set_selKey;

    /// Returns the current selection key setting.
    ///
    /// The returned value is a pointer to an integer array. The memory must
    /// be freed by the caller using function
    /// [chewing_free][super::setup::chewing_free].
    pub use super::io::chewing_get_selKey;

    /// This function is no-op now. Use [chewing_set_selKey] instead.
    pub use super::io::chewing_set_hsuSelKeyType;

    /// This function is no-op now. Use [chewing_get_selKey] instead.
    pub use super::io::chewing_get_hsuSelKeyType;

    /// Opens the candidate selection window.
    ///
    /// This operation is only allowed when the IM editor is in entering state.
    ///
    /// Returns 0 when success, -1 otherwise.
    pub use super::io::chewing_cand_open;

    /// Closes the candidate selection window.
    ///
    /// Returns 0 when success, -1 otherwise.
    pub use super::io::chewing_cand_close;

    /// Returns the candidate string by its index.
    ///
    /// The *index* must be between 0 and [chewing_cand_TotalChoice] inclusive.
    ///
    /// The returned value is a pointer to a character string. The memory must
    /// be freed by the caller using function
    /// [chewing_free][super::setup::chewing_free].
    ///
    /// # Failures
    ///
    /// This function returns NULL when memory allocation fails.
    pub use super::io::chewing_cand_string_by_index;

    /// Returns the candidate string by its index.
    ///
    /// The *index* must be between 0 and [chewing_cand_TotalChoice] inclusive.
    ///
    /// The return value is a const pointer to a character string. The pointer
    /// is only valid immediately after calling this function.
    pub use super::io::chewing_cand_string_by_index_static;

    /// Selects the candidate by its index.
    ///
    /// The *index* must be between 0 and [chewing_cand_TotalChoice] inclusive.
    ///
    /// Returns 0 when success, -1 otherwise.
    ///
    /// # Errors
    ///
    /// This function fails if the *index* is out of range or the candidate
    /// selection window is not currently open.
    pub use super::io::chewing_cand_choose_by_index;

    /// Sets the candidate list to the first (longest) candidate list.
    ///
    /// Returns 0 when success, -1 otherwise.
    ///
    /// # Errors
    ///
    /// This function fails if the candidate selection window is not currently
    /// open.
    pub use super::io::chewing_cand_list_first;

    /// Sets the candidate list to the last (shortest) candidate list.
    ///
    /// Returns 0 when success, -1 otherwise.
    ///
    /// # Errors
    ///
    /// This function fails if the candidate selection window is not currently
    /// open.
    pub use super::io::chewing_cand_list_last;

    /// Checks whether there is a next (shorter) candidate list.
    ///
    /// Returns 1 (true) when there is a next candidate list, 0 otherwise.
    pub use super::io::chewing_cand_list_has_next;

    /// Checks whether there is a previous (longer) candidate list.
    ///
    /// Returns 1 (true) when there is a previous candidate list, 0 otherwise.
    pub use super::io::chewing_cand_list_has_prev;

    /// Changes current candidate list to next candidate list.
    ///
    /// Returns 0 when success, -1 otherwise.
    ///
    /// # Errors
    ///
    /// This function fails if the candidate selection window is not currently
    /// open.
    pub use super::io::chewing_cand_list_next;

    /// Changes current candidate list to previous candidate list.
    ///
    /// Returns 0 when success, -1 otherwise.
    ///
    /// # Errors
    ///
    /// This function fails if the candidate selection window is not currently
    /// open.
    pub use super::io::chewing_cand_list_prev;

    pub use super::public::MAX_SELKEY;
    pub use super::public::MIN_SELKEY;

    pub use super::public::HSU_SELKEY_TYPE1;
    pub use super::public::HSU_SELKEY_TYPE2;
}

/// Output handling.
pub mod output {
    /// Checks whether the commit buffer has something to read.
    ///
    /// Returns 1 when true, 0 when false.
    pub use super::io::chewing_commit_Check;

    /// Returns the string in the commit buffer.
    ///
    /// The returned value is a pointer to a character string. The memory must
    /// be freed by the caller using function
    /// [chewing_free][super::setup::chewing_free].
    ///
    /// # Failures
    ///
    /// This function returns NULL when memory allocation fails.
    pub use super::io::chewing_commit_String;

    /// Returns the string in the commit buffer.
    ///
    /// The return value is a const pointer to a character string. The pointer
    /// is only valid immediately after checking the [chewing_commit_Check]
    /// condition.
    pub use super::io::chewing_commit_String_static;

    /// Checks whether the previous keystroke is ignored or not.
    ///
    /// Returns 1 when true, 0 when false.
    pub use super::io::chewing_keystroke_CheckIgnore;

    /// Checks whether the previous keystroke is absorbed or not.
    ///
    /// Returns 1 when true, 0 when false.
    ///
    /// Absorbed key means the Chewing IM state machine has accepted the key and
    /// changed its state accordingly. Caller should check various output
    /// buffers to see if they need to update the display.
    pub use super::io::chewing_keystroke_CheckAbsorb;

    /// Checks whether there is output in the pre-edit buffer.
    ///
    /// Returns 1 when true, 0 when false.
    pub use super::io::chewing_buffer_Check;

    /// Returns the length of the string in current pre-edit buffer.
    ///
    /// <p style="background:rgba(255,181,77,0.16);padding:0.75em;">
    /// <strong>⚠ Warning:</strong> The length is calculated in terms of
    /// unicode characters. One character might occupy multiple bytes.
    /// </p>
    pub use super::io::chewing_buffer_Len;

    /// Returns the current output in the pre-edit buffer.
    ///
    /// The returned value is a pointer to a character string. The memory must
    /// be freed by the caller using function
    /// [chewing_free][super::setup::chewing_free].
    ///
    /// # Failures
    ///
    /// This function returns NULL when memory allocation fails.
    pub use super::io::chewing_buffer_String;

    /// Returns the current output in the pre-edit buffer.
    ///
    /// The return value is a const pointer to a character string. The pointer
    /// is only valid immediately after checking the [chewing_buffer_Check]
    /// condition.
    pub use super::io::chewing_buffer_String_static;

    /// Returns whether there are phonetic pre-edit string in the buffer.
    ///
    /// Returns 1 when true, 0 when false.
    pub use super::io::chewing_bopomofo_Check;

    /// Returns whether there are phonetic pre-edit string in the buffer. Here
    /// “zuin” means bopomofo, a phonetic system for transcribing Chinese,
    /// especially Mandarin.
    ///
    /// Returns **0** when true, **1** when false.
    ///
    /// <p style="background:rgba(255,181,77,0.16);padding:0.75em;">
    /// <strong>⚠ Warning:</strong> The return value of this function is
    /// different from other newer functions that returns boolean value.
    /// </p>
    pub use super::io::chewing_zuin_Check;

    /// Returns the phonetic characters in the pre-edit buffer.
    ///
    /// The returned value is a pointer to a character string. The memory must
    /// be freed by the caller using function
    /// [chewing_free][super::setup::chewing_free].
    ///
    /// # Failures
    ///
    /// This function returns NULL when memory allocation fails.
    pub use super::io::chewing_bopomofo_String;

    /// Returns the phonetic characters in the pre-edit buffer.
    ///
    /// The return value is a const pointer to a character string. The pointer
    /// is only valid immediately after checking the [chewing_bopomofo_Check]
    /// condition.
    pub use super::io::chewing_bopomofo_String_static;

    /// Returns the phonetic characters in the pre-edit buffer.
    ///
    /// The bopomofo_count argument is a output argument. It will contain the
    /// number of phonetic characters in the returned string.
    ///
    /// The returned value is a pointer to a character string. The memory must
    /// be freed by the caller using function
    /// [chewing_free][super::setup::chewing_free].
    ///
    /// # Failures
    ///
    /// This function returns NULL when memory allocation fails.
    pub use super::io::chewing_zuin_String;

    /// Returns the current cursor position in the pre-edit buffer.
    pub use super::io::chewing_cursor_Current;

    /// Starts the enumeration of intervals of recognized phrases.
    ///
    /// This function stores an iterator in the context. The iterator is only
    /// destroyed after enumerate all intervals using
    /// [chewing_interval_hasNext].
    pub use super::io::chewing_interval_Enumerate;

    /// Checks whether there are more intervals or not.
    ///
    /// Returns 1 when true, 0 when false.
    pub use super::io::chewing_interval_hasNext;

    /// Returns the current enumerated interval.
    ///
    /// The *it* argument is an output argument.
    pub use super::io::chewing_interval_Get;

    /// Returns whether there is auxiliary string in the auxiliary buffer.
    ///
    /// Returns 1 when true, 0 when false.
    pub use super::io::chewing_aux_Check;

    /// Returns the length of the auxiliary string in the auxiliary buffer.
    ///
    /// <p style="background:rgba(255,181,77,0.16);padding:0.75em;">
    /// <strong>⚠ Warning:</strong> The length is calculated in terms of
    /// unicode characters. One character might occupy multiple bytes.
    /// </p>
    pub use super::io::chewing_aux_Length;

    /// Returns the current auxiliary string.
    ///
    /// The returned value is a pointer to a character string. The memory must
    /// be freed by the caller using function
    /// [chewing_free][super::setup::chewing_free].
    ///
    /// # Failures
    ///
    /// This function returns NULL when memory allocation fails.
    pub use super::io::chewing_aux_String;

    /// Returns the current auxiliary string.
    ///
    /// The return value is a const pointer to a character string. The pointer
    /// is only valid immediately after checking the [chewing_aux_Check]
    /// condition.
    pub use super::io::chewing_aux_String_static;

    /// Returns the phonetic sequence in the Chewing IM internal state machine.
    ///
    /// The return value is a pointer to a unsigned short array. The values in
    /// the array is encoded Bopomofo phone. The memory must be freed by the
    /// caller using function [chewing_free][super::setup::chewing_free].
    pub use super::io::chewing_get_phoneSeq;

    /// Returns the length of the phonetic sequence in the Chewing IM internal
    /// state machine.
    pub use super::io::chewing_get_phoneSeqLen;

    /// Converts the u16 encoded syllables to a bopomofo string.
    ///
    /// If both of the buf and the len are 0, this function will return buf
    /// length for bopomofo including the null character so that caller can
    /// prepare enough buffer for it.
    ///
    /// Returns 0 on success, -1 on failure.
    pub use super::io::chewing_phone_to_bopomofo;

    /// Commits the current preedit buffer content to the commit buffer.
    ///
    /// Returns 0 when success, -1 otherwise.
    ///
    /// # Errors
    ///
    /// This function fails if the IM editor is not in entering state.
    pub use super::io::chewing_commit_preedit_buf;

    /// Clears the current preedit buffer content.
    ///
    /// Returns 0 when success, -1 otherwise.
    ///
    /// # Errors
    ///
    /// This function fails if the IM editor is not in entering state.
    pub use super::io::chewing_clean_preedit_buf;

    /// Clears the current bopomofo buffer content.
    ///
    /// Returns 0 when success, -1 otherwise.
    ///
    /// # Errors
    ///
    /// This function fails if the IM editor is not in entering state.
    pub use super::io::chewing_clean_bopomofo_buf;

    /// Acknowledge the commit buffer and aux output buffer.
    ///
    /// Chewing automatically acknowledges and clear the output buffers after
    /// processing new input events.
    ///
    /// After handling the ephemeral output buffer like the commit buffer and
    /// the aux output buffer, IM wrappers can proactively acknowledge and clear
    /// the buffers. This can be used so that IM wrappers don't have to remember
    /// whether an output has been handled or not.
    pub use super::io::chewing_ack;

    pub use super::public::IntervalType;
}

/// Userphrase handling.
pub mod userphrase {
    /// Starts a userphrase enumeration.
    ///
    /// Caller shall call this function prior [chewing_userphrase_has_next] and
    /// [chewing_userphrase_get] in order to enumerate userphrase correctly.
    ///
    /// This function stores an iterator in the context. The iterator is only
    /// destroyed after enumerate all userphrases using
    /// [chewing_userphrase_has_next].
    ///
    /// Returns 0 on success, -1 on failure.
    ///
    /// # Examples
    ///
    /// ```c
    /// chewing_userphrase_enumerate(ctx);
    /// while (chewing_userphrase_has_next(ctx, &phrase_len, &bopomofo_len)) {
    ///     phrase = malloc(phrase_len);
    ///     if (!phrase) goto error;
    ///     bopomofo = malloc(bopomofo_len);
    ///     if (!bopomofo) goto error;
    ///
    ///     chewing_userphrase_get(ctx, phrase, phrase_len, bopomofo, bopomofo_len);
    ///     /* do somthing */
    /// }
    /// ```
    pub use super::io::chewing_userphrase_enumerate;

    /// Checks if there is another userphrase in current enumeration.
    ///
    /// The *phrase_len* and *bopomofo_len* are output buffer length needed by the userphrase and its bopomofo string.
    ///
    /// Returns 1 when true, 0 when false.
    pub use super::io::chewing_userphrase_has_next;

    /// Gets the current enumerated userphrase.
    ///
    /// The *phrase_buf* and *bopomofo_buf* are userphrase and its bopomofo
    /// buffer provided by caller. The length of the buffers can be retrived
    /// from [chewing_userphrase_has_next].
    ///
    /// Returns 0 on success, -1 on failure.
    pub use super::io::chewing_userphrase_get;

    /// Adds new userphrase to the user dictionary.
    ///
    /// Returns how many phrases are added, -1 on failure.
    pub use super::io::chewing_userphrase_add;

    /// Removes a userphrase from the user dictionary.
    ///
    /// Returns how many phrases are removed, -1 on failure.
    pub use super::io::chewing_userphrase_remove;

    /// Searchs if a userphrase is in the user dictionary.
    ///
    /// Returns 1 when true, 0 when false.
    pub use super::io::chewing_userphrase_lookup;
}

/// Global settings.
///
/// The Chewing IM could be customized in some small details. These functions
/// provide the configuration interfaces to the front-end.
pub mod globals {
    /// Sets the maximum number of the Chinese characters allowed in the
    /// pre-edit buffer.
    ///
    /// If the pre-edit string is longer than this number then the leading part
    /// will be committed automatically. The range of n shall between
    /// [MIN_CHI_SYMBOL_LEN] and [MAX_CHI_SYMBOL_LEN].
    pub use super::io::chewing_set_maxChiSymbolLen;

    /// Returns the maximum number of the Chinese characters allowed in the
    /// pre-edit buffer.
    pub use super::io::chewing_get_maxChiSymbolLen;

    /// Sets the direction to add new phrases when using CtrlNum.
    ///
    /// The direction argument is 0 when the direction is backward and 1 when
    /// the direction is forward.
    pub use super::io::chewing_set_addPhraseDirection;

    /// Returns the direction to add new phrases when using CtrlNum.
    ///
    /// The direction argument is 0 when the direction is backward and 1 when
    /// the direction is forward.
    pub use super::io::chewing_get_addPhraseDirection;

    /// Sets whether the Space key is treated as a selection key.
    ///
    /// When the mode argument is 1, the Space key will initiate the candidates
    /// selection mode.
    pub use super::io::chewing_set_spaceAsSelection;

    /// Returns whether the Space key is treated as a selection key.
    ///
    /// Returns 1 when the Space key will initiate the candidates selection
    /// mode.
    pub use super::io::chewing_get_spaceAsSelection;

    /// Sets whether the Esc key will flush the current pre-edit buffer.
    ///
    /// When the mode argument is 1, the Esc key will flush the pre-edit buffer.
    pub use super::io::chewing_set_escCleanAllBuf;

    /// Returns whether the Esc key will flush the current pre-edit buffer.
    ///
    /// Returns 1 when the Esc key will flush the pre-edit buffer.
    pub use super::io::chewing_get_escCleanAllBuf;

    /// Sets whether the Chewing IM will automatically shift cursor after
    /// selection.
    pub use super::io::chewing_set_autoShiftCur;

    /// Returns whether the Chewing IM will automatically shift cursor after
    /// selection.
    pub use super::io::chewing_get_autoShiftCur;

    /// Sets the current normal/easy symbol mode.
    ///
    /// In easy symbol mode, the key be will changed to its related easy symbol
    /// in swkb.dat. The format of swkb.dat is key symbol pair per line. The
    /// valid value of key is [0-9A-Z]. The lower case character in key will be
    /// changed to upper case when loading swkb.dat. However, in easy symbol
    /// mode, only [0-9A-Z] are accepted.
    ///
    /// The mode argument is 0 for normal mode or other for easy symbol mode.
    pub use super::io::chewing_set_easySymbolInput;

    /// Gets the current normal/easy symbol mode.
    pub use super::io::chewing_get_easySymbolInput;

    /// Sets whether the phrase for candidates selection is before the cursor or
    /// after the cursor.
    pub use super::io::chewing_set_phraseChoiceRearward;

    /// Returns the phrase choice rearward setting.
    pub use super::io::chewing_get_phraseChoiceRearward;

    /// Sets enable or disable the automatic learning.
    ///
    /// The mode argument is be one of the [AUTOLEARN_ENABLED] and
    /// [AUTOLEARN_DISABLED] constants.
    pub use super::io::chewing_set_autoLearn;

    /// Returns whether the automatic learning is enabled or disabled.
    pub use super::io::chewing_get_autoLearn;

    pub use super::io::chewing_config_has_option;

    pub use super::io::chewing_config_get_int;
    pub use super::io::chewing_config_set_int;

    pub use super::io::chewing_config_get_str;
    pub use super::io::chewing_config_set_str;

    pub use super::public::MAX_CHI_SYMBOL_LEN;
    pub use super::public::MIN_CHI_SYMBOL_LEN;

    pub use super::public::MAX_PHONE_SEQ_LEN;
    pub use super::public::MAX_PHRASE_LEN;

    pub use super::public::AUTOLEARN_DISABLED;
    pub use super::public::AUTOLEARN_ENABLED;
}
