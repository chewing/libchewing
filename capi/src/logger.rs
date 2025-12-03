use std::{
    ffi::{CString, c_char, c_int, c_void},
    io::Write,
    mem,
};

use tracing::Level;
use tracing::dispatcher::DefaultGuard;
use tracing_subscriber::{fmt::MakeWriter, util::SubscriberInitExt};

use crate::setup::{
    CHEWING_LOG_DEBUG, CHEWING_LOG_ERROR, CHEWING_LOG_INFO, CHEWING_LOG_VERBOSE, CHEWING_LOG_WARN,
};

pub(crate) type ExternLoggerFn =
    unsafe extern "C" fn(data: *mut c_void, level: c_int, fmt: *const c_char, arg: ...);

#[derive(Clone)]
pub(crate) struct ChewingLogger {
    level: c_int,
    buffer: Vec<u8>,
    logger_fn: ExternLoggerFn,
    logger_data: *mut c_void,
}

impl ChewingLogger {
    pub(crate) fn new(logger_fn: ExternLoggerFn, logger_data: *mut c_void) -> ChewingLogger {
        ChewingLogger {
            level: 0,
            buffer: vec![],
            logger_fn,
            logger_data,
        }
    }
}

unsafe impl Send for ChewingLogger {}
unsafe impl Sync for ChewingLogger {}

impl<'a> MakeWriter<'a> for ChewingLogger {
    type Writer = ChewingLogger;
    fn make_writer(&'a self) -> Self::Writer {
        self.clone()
    }
    fn make_writer_for(&'a self, meta: &tracing::Metadata<'_>) -> Self::Writer {
        let mut writer = self.make_writer();
        writer.level = as_chewing_level(meta.level());
        writer
    }
}

impl Write for ChewingLogger {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.buffer.write(buf)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl Drop for ChewingLogger {
    fn drop(&mut self) {
        if !self.buffer.is_empty() {
            let buffer = mem::take(&mut self.buffer);
            let fmt_cstring = CString::new(buffer).unwrap();
            unsafe {
                (self.logger_fn)(
                    self.logger_data,
                    self.level,
                    c"%s".as_ptr().cast(),
                    fmt_cstring.as_ptr(),
                )
            }
            return;
        }
    }
}

fn as_chewing_level(level: &Level) -> c_int {
    (match *level {
        Level::ERROR => CHEWING_LOG_ERROR,
        Level::WARN => CHEWING_LOG_WARN,
        Level::INFO => CHEWING_LOG_INFO,
        Level::DEBUG => CHEWING_LOG_DEBUG,
        Level::TRACE => CHEWING_LOG_VERBOSE,
    }) as c_int
}

pub(crate) fn init_scoped_logging_subscriber(
    logger_fn: Option<ExternLoggerFn>,
    logger_data: *mut c_void,
) -> Option<DefaultGuard> {
    logger_fn.map(|logger_fn| {
        tracing_subscriber::fmt()
            .with_writer(ChewingLogger::new(logger_fn, logger_data))
            .without_time()
            .with_ansi(false)
            .finish()
            .set_default()
    })
}
