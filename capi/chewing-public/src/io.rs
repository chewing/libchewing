use std::ffi::c_int;

use crate::capi::internal::{binding, types::ChewingContext};

#[no_mangle]
pub extern "C" fn chewing_new() -> *mut ChewingContext {
    unsafe { binding::chewing_new_c().cast() }
}

#[no_mangle]
pub extern "C" fn chewing_set_maxChiSymbolLen(ctx: *mut ChewingContext, n: c_int) {
    unsafe { binding::chewing_set_maxChiSymbolLen_c(ctx.cast(), n) }
}

#[no_mangle]
pub extern "C" fn chewing_delete(ctx: *mut ChewingContext) {
    unsafe { binding::chewing_delete_c(ctx.cast()) }
}