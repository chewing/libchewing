#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

mod ffi;
mod io;
mod public;

#[doc(hidden)]
pub mod internal;

/// Initializes chewing context and environment settings
///
/// Most of the Chewing IM APIs require a [ChewingContext]. To create a
/// ChewingContext you must use the [chewing_new] function.
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

    /// Sets the selectAreaLen, maxChiSymbolLen and selKey parameter from pcd
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

pub use io::*;
pub use public::*;
