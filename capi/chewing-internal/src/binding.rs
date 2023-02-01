use libc::c_int;

use ffi_opaque::opaque;

opaque! {
    pub struct ChewingData;
    pub struct ChewingContext;
}

extern "C" {
    pub fn toPreeditBufIndex(pgdata: *mut ChewingData, pos: c_int) -> c_int;
    pub fn HaninSymbolInput(pgdata: *mut ChewingData) -> c_int;
}
