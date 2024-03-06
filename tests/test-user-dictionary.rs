use std::env;
use std::error::Error;
use std::ffi::c_int;
use std::ffi::CStr;
use std::ffi::CString;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::ptr::null_mut;

use chewing::capi::input::chewing_handle_Default;
use chewing::capi::output::chewing_buffer_String;
use chewing::capi::setup::chewing_delete;
use chewing::capi::setup::chewing_new2;
use tempfile::tempdir;
use tempfile::TempDir;

fn golden_data_path(filename: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/data")
        .join(filename)
}

fn syspath() -> Result<CString, Box<dyn Error>> {
    Ok(CString::new(format!(
        "{}/tests/data",
        env!("CARGO_MANIFEST_DIR")
    ))?)
}

fn tempdir_and_file(filename: &str) -> Result<(TempDir, CString), Box<dyn Error>> {
    let dir = tempdir()?;
    let path = CString::new(dir.path().join(filename).display().to_string())?;
    Ok((dir, path))
}

#[test]
fn explicit_load_chewing_cdb() -> Result<(), Box<dyn Error>> {
    let syspath = syspath()?;
    let (tmpdir, userpath) = tempdir_and_file("chewing.cdb")?;
    let chewing_cdb = golden_data_path("chewing.cdb");
    fs::copy(chewing_cdb, tmpdir.path().join("chewing.cdb"))?;

    let ctx = chewing_new2(syspath.as_ptr(), userpath.as_ptr(), None, null_mut());
    assert!(!ctx.is_null());

    chewing_handle_Default(ctx, b'h' as c_int);
    chewing_handle_Default(ctx, b'k' as c_int);
    chewing_handle_Default(ctx, b'4' as c_int);
    chewing_handle_Default(ctx, b'g' as c_int);
    chewing_handle_Default(ctx, b'4' as c_int);

    let preedit = chewing_buffer_String(ctx);
    let preedit = unsafe { CStr::from_ptr(preedit) };
    assert_eq!(preedit, CString::new("策試")?.as_c_str());

    chewing_delete(ctx);
    Ok(())
}

#[cfg(feature = "sqlite")]
#[test]
fn explicit_load_chewing_sqlite3() -> Result<(), Box<dyn Error>> {
    let syspath = syspath()?;
    let (tmpdir, userpath) = tempdir_and_file("chewing.sqlite3")?;
    let chewing_cdb = golden_data_path("golden-chewing.sqlite3");
    fs::copy(chewing_cdb, tmpdir.path().join("chewing.sqlite3"))?;

    let ctx = chewing_new2(syspath.as_ptr(), userpath.as_ptr(), None, null_mut());
    assert!(!ctx.is_null());

    chewing_handle_Default(ctx, b'h' as c_int);
    chewing_handle_Default(ctx, b'k' as c_int);
    chewing_handle_Default(ctx, b'4' as c_int);
    chewing_handle_Default(ctx, b'g' as c_int);
    chewing_handle_Default(ctx, b'4' as c_int);

    let preedit = chewing_buffer_String(ctx);
    let preedit = unsafe { CStr::from_ptr(preedit) };
    assert_eq!(preedit, CString::new("策試")?.as_c_str());

    chewing_delete(ctx);
    Ok(())
}
