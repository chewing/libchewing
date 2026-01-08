use std::{
    cell::RefCell,
    ffi::{CString, c_char, c_int, c_void},
};

use env_logger::Logger as EnvLogger;
use log::{Level, LevelFilter, Log, Metadata, Record, SetLoggerError};

use crate::setup::{
    CHEWING_LOG_DEBUG, CHEWING_LOG_ERROR, CHEWING_LOG_INFO, CHEWING_LOG_VERBOSE, CHEWING_LOG_WARN,
};

thread_local! {
    static CTX_LOGGER: RefCell<Vec<(ExternLoggerFn, *mut c_void)>> = const { RefCell::new(vec![]) };
}

pub(crate) type ExternLoggerFn =
    unsafe extern "C" fn(data: *mut c_void, level: c_int, fmt: *const c_char, arg: ...);

pub(crate) struct ChewingLogger {
    env_logger: EnvLogger,
}

pub(crate) struct ChewingLoggerGuard;

pub(crate) fn init() -> Result<(), SetLoggerError> {
    log::set_boxed_logger(Box::new(ChewingLogger::new()))
        .map(|()| log::set_max_level(LevelFilter::Trace))
}

pub(crate) fn init_scoped_logging(
    logger_fn: Option<ExternLoggerFn>,
    data: *mut c_void,
) -> ChewingLoggerGuard {
    if let Some(logger_fn) = logger_fn {
        CTX_LOGGER.with_borrow_mut(|ctx_logger| ctx_logger.push((logger_fn, data)));
    }
    ChewingLoggerGuard
}

impl ChewingLogger {
    pub(crate) fn new() -> ChewingLogger {
        ChewingLogger {
            env_logger: EnvLogger::from_default_env(),
        }
    }
}

impl Drop for ChewingLoggerGuard {
    fn drop(&mut self) {
        CTX_LOGGER.with_borrow_mut(|ctx_logger| ctx_logger.pop());
    }
}

impl Log for ChewingLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        self.env_logger.enabled(metadata)
    }
    fn log(&self, record: &Record) {
        self.env_logger.log(record);
        CTX_LOGGER.with_borrow(|ctx_logger| {
            if let Some((logger_fn, data)) = ctx_logger.last() {
                let fmt = format!(
                    "[{}:{} {}] {}",
                    record.file().unwrap_or("unknown"),
                    record.line().unwrap_or_default(),
                    record.module_path().unwrap_or("unknown"),
                    record.args()
                );
                let fmt_cstring = CString::new(fmt).unwrap();
                unsafe {
                    logger_fn(
                        *data,
                        as_chewing_level(record.level()),
                        c"%s\n".as_ptr().cast(),
                        fmt_cstring.as_ptr(),
                    )
                }
            }
        })
    }
    fn flush(&self) {
        self.env_logger.flush();
    }
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
