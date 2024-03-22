use std::{
    ffi::{c_char, c_int, c_void, CString},
    sync::{
        atomic::{AtomicPtr, Ordering::Relaxed},
        Mutex,
    },
};

use log::{Level, Log, Metadata, Record};

use super::setup::{
    CHEWING_LOG_DEBUG, CHEWING_LOG_ERROR, CHEWING_LOG_INFO, CHEWING_LOG_VERBOSE, CHEWING_LOG_WARN,
};

type ExternLoggerFn =
    unsafe extern "C" fn(data: *mut c_void, level: c_int, fmt: *const c_char, arg: ...);

pub(crate) struct ChewingLogger {
    logger: Mutex<Option<(ExternLoggerFn, AtomicPtr<c_void>)>>,
}

impl ChewingLogger {
    pub(crate) const fn new() -> ChewingLogger {
        ChewingLogger {
            logger: Mutex::new(None),
        }
    }
    pub(crate) fn init(&self) {}
    pub(crate) fn set(&self, logger: Option<(ExternLoggerFn, *mut c_void)>) {
        if let Ok(mut prev) = self.logger.lock() {
            *prev = logger.map(|(l, d)| (l, d.into()));
        }
    }
}

impl Log for ChewingLogger {
    fn enabled(&self, _metadata: &Metadata<'_>) -> bool {
        true
    }

    fn log(&self, record: &Record<'_>) {
        if let Ok(logger) = self.logger.lock() {
            if let Some((logger, logger_data)) = logger.as_ref() {
                let fmt = format!(
                    "[{}:{} {}] {}\n",
                    record.file().unwrap_or("unknown"),
                    record.line().unwrap_or_default(),
                    record.module_path().unwrap_or("unknown"),
                    record.args()
                );
                let fmt_cstring = CString::new(fmt).unwrap();
                unsafe {
                    logger(
                        logger_data.load(Relaxed),
                        as_chewing_level(record.level()),
                        fmt_cstring.as_ptr(),
                    )
                }
                return;
            }
        }
    }

    fn flush(&self) {}
}

fn as_chewing_level(level: Level) -> c_int {
    (match level {
        Level::Error => CHEWING_LOG_ERROR,
        Level::Warn => CHEWING_LOG_WARN,
        Level::Info => CHEWING_LOG_INFO,
        Level::Debug => CHEWING_LOG_DEBUG,
        Level::Trace => CHEWING_LOG_VERBOSE,
    }) as c_int
}
