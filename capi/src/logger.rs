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
    env_logger: Mutex<Option<env_logger::Logger>>,
    logger: Mutex<Option<(ExternLoggerFn, AtomicPtr<c_void>)>>,
}

impl ChewingLogger {
    pub(crate) const fn new() -> ChewingLogger {
        ChewingLogger {
            env_logger: Mutex::new(None),
            logger: Mutex::new(None),
        }
    }
    pub(crate) fn init(&self) {
        if let Ok(mut prev) = self.env_logger.lock() {
            *prev = Some(env_logger::Logger::from_default_env());
        }
    }
    pub(crate) fn set(&self, logger: Option<(ExternLoggerFn, *mut c_void)>) {
        if let Ok(mut prev) = self.logger.lock() {
            *prev = logger.map(|(l, d)| (l, d.into()));
        }
    }
}

impl Log for ChewingLogger {
    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        if let Ok(logger) = self.logger.lock() {
            if logger.is_some() && metadata.level() <= Level::Debug {
                return true;
            }
        }
        if let Ok(logger) = self.env_logger.lock() {
            if let Some(el) = logger.as_ref() {
                return el.enabled(metadata);
            }
        }
        false
    }

    fn log(&self, record: &Record<'_>) {
        if let Ok(logger) = self.logger.lock() {
            if let Some((logger, logger_data)) = logger.as_ref() {
                let fmt = format!(
                    "[{}:{} {}] {}",
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
                        c"%s\n".as_ptr().cast(),
                        fmt_cstring.as_ptr(),
                    )
                }
                return;
            }
        }
        if let Ok(logger) = self.env_logger.lock() {
            if let Some(el) = logger.as_ref() {
                el.log(record);
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
