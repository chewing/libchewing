use std::env;
use std::error::Error;
use std::ffi::CStr;
use std::ffi::CString;
use std::ffi::c_int;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::ptr::null_mut;
use std::sync::Mutex;

use chewing_capi::input::chewing_handle_Default;
use chewing_capi::output::chewing_buffer_String;
use chewing_capi::setup::ChewingContext;
use chewing_capi::setup::chewing_delete;
use chewing_capi::setup::chewing_new2;
use tempfile::TempDir;
use tempfile::tempdir;

static ENV_LOCK: Mutex<()> = Mutex::new(());

fn golden_data_path(filename: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../")
        .join("tests/data")
        .join(filename)
}

fn syspath() -> Result<CString, Box<dyn Error>> {
    Ok(CString::new(format!(
        "{}/../tests/data",
        env!("CARGO_MANIFEST_DIR")
    ))?)
}

fn tempdir_and_file(filename: &str) -> Result<(TempDir, CString), Box<dyn Error>> {
    let dir = tempdir()?;
    let path = CString::new(dir.path().join(filename).display().to_string())?;
    Ok((dir, path))
}

unsafe fn assert_phrase_only_in_user_dictionary(
    ctx: *mut ChewingContext,
) -> Result<(), Box<dyn Error>> {
    assert!(!ctx.is_null());

    chewing_handle_Default(ctx, b'h' as c_int);
    chewing_handle_Default(ctx, b'k' as c_int);
    chewing_handle_Default(ctx, b'4' as c_int);
    chewing_handle_Default(ctx, b'g' as c_int);
    chewing_handle_Default(ctx, b'4' as c_int);

    let preedit = chewing_buffer_String(ctx);
    let preedit = unsafe { CStr::from_ptr(preedit) };
    assert_eq!(preedit, CString::new("策試")?.as_c_str());

    Ok(())
}

#[test]
fn explicit_load_chewing_dat() -> Result<(), Box<dyn Error>> {
    let syspath = syspath()?;
    let (tmpdir, userpath) = tempdir_and_file("chewing.dat")?;
    let chewing_golden = golden_data_path("chewing.dat");
    fs::copy(chewing_golden, tmpdir.path().join("chewing.dat"))?;

    unsafe {
        let ctx = chewing_new2(syspath.as_ptr(), userpath.as_ptr(), None, null_mut());
        assert_phrase_only_in_user_dictionary(ctx)?;
        chewing_delete(ctx);
    }
    Ok(())
}

#[cfg(feature = "sqlite")]
#[test]
fn explicit_load_chewing_sqlite3() -> Result<(), Box<dyn Error>> {
    let syspath = syspath()?;
    let (tmpdir, userpath) = tempdir_and_file("chewing.sqlite3")?;
    let chewing_golden = golden_data_path("golden-chewing.sqlite3");
    fs::copy(chewing_golden, tmpdir.path().join("chewing.sqlite3"))?;

    unsafe {
        let ctx = chewing_new2(syspath.as_ptr(), userpath.as_ptr(), None, null_mut());
        assert_phrase_only_in_user_dictionary(ctx)?;
        chewing_delete(ctx);
    }
    Ok(())
}

#[cfg(feature = "sqlite")]
#[test]
fn env_load_chewing_sqlite3() -> Result<(), Box<dyn Error>> {
    use std::ptr::null;

    let syspath = syspath()?;
    let (tmpdir, _userpath) = tempdir_and_file("chewing.sqlite3")?;
    let chewing_golden = golden_data_path("golden-chewing.sqlite3");
    fs::copy(chewing_golden, tmpdir.path().join("chewing.sqlite3"))?;

    unsafe {
        let ctx = {
            let _lock = ENV_LOCK.lock()?;
            env::set_var("CHEWING_PATH", syspath.to_str()?);
            env::set_var("CHEWING_USER_PATH", tmpdir.path().display().to_string());
            chewing_new2(null(), null(), None, null_mut())
        };
        assert_phrase_only_in_user_dictionary(ctx)?;
        chewing_delete(ctx);
    }
    Ok(())
}

#[cfg(feature = "sqlite")]
#[test]
fn env_load_and_migrate_chewing_sqlite3_v1() -> Result<(), Box<dyn Error>> {
    use std::ptr::null;

    let syspath = syspath()?;
    let (tmpdir, _userpath) = tempdir_and_file("chewing.sqlite3")?;
    let chewing_golden = golden_data_path("golden-chewing-v1.sqlite3");
    fs::copy(chewing_golden, tmpdir.path().join("chewing.sqlite3"))?;

    unsafe {
        let ctx = {
            let _lock = ENV_LOCK.lock()?;
            env::set_var("CHEWING_PATH", syspath.to_str()?);
            env::set_var("CHEWING_USER_PATH", tmpdir.path().display().to_string());
            chewing_new2(null(), null(), None, null_mut())
        };
        assert_phrase_only_in_user_dictionary(ctx)?;
        chewing_delete(ctx);
    }
    Ok(())
}

#[cfg(not(feature = "sqlite"))]
#[test]
fn explicit_load_chewing_sqlite3_should_fail() -> Result<(), Box<dyn Error>> {
    let syspath = syspath()?;
    let (tmpdir, userpath) = tempdir_and_file("chewing.sqlite3")?;
    let chewing_golden = golden_data_path("golden-chewing.sqlite3");
    fs::copy(chewing_golden, tmpdir.path().join("chewing.sqlite3"))?;

    unsafe {
        let ctx = chewing_new2(syspath.as_ptr(), userpath.as_ptr(), None, null_mut());
        chewing_delete(ctx);
    }
    Ok(())
}

#[cfg(not(feature = "sqlite"))]
#[test]
fn env_load_chewing_trie() -> Result<(), Box<dyn Error>> {
    use std::ptr::null;

    let syspath = syspath()?;
    let (tmpdir, _userpath) = tempdir_and_file("chewing.dat")?;
    let chewing_golden = golden_data_path("chewing.dat");
    fs::copy(chewing_golden, tmpdir.path().join("chewing.dat"))?;

    unsafe {
        let ctx = {
            let _lock = ENV_LOCK.lock()?;
            env::set_var("CHEWING_PATH", syspath.to_str()?);
            env::set_var("CHEWING_USER_PATH", tmpdir.path().display().to_string());
            chewing_new2(null(), null(), None, null_mut())
        };
        assert_phrase_only_in_user_dictionary(ctx)?;
        chewing_delete(ctx);
    }
    Ok(())
}

#[cfg(all(target_endian = "little", target_pointer_width = "64"))]
#[test]
fn env_load_and_migrate_uhash_le_64_to_trie() -> Result<(), Box<dyn Error>> {
    use std::ptr::null;

    let syspath = syspath()?;
    let (tmpdir, _userpath) = tempdir_and_file("chewing.dat")?;
    let chewing_golden = golden_data_path("golden-uhash-le-64.dat");
    fs::copy(chewing_golden, tmpdir.path().join("uhash.dat"))?;

    unsafe {
        let ctx = {
            let _lock = ENV_LOCK.lock()?;
            env::set_var("CHEWING_PATH", syspath.to_str()?);
            env::set_var("CHEWING_USER_PATH", tmpdir.path().display().to_string());
            chewing_new2(null(), null(), None, null_mut())
        };
        assert_phrase_only_in_user_dictionary(ctx)?;
        chewing_delete(ctx);
    }
    Ok(())
}

#[test]
fn env_load_and_migrate_uhash_text_to_trie() -> Result<(), Box<dyn Error>> {
    use std::ptr::null;

    let syspath = syspath()?;
    let (tmpdir, _userpath) = tempdir_and_file("chewing.dat")?;
    let chewing_golden = golden_data_path("golden-uhash-text.dat");
    fs::copy(chewing_golden, tmpdir.path().join("uhash.dat"))?;

    unsafe {
        let ctx = {
            let _lock = ENV_LOCK.lock()?;
            env::set_var("CHEWING_PATH", syspath.to_str()?);
            env::set_var("CHEWING_USER_PATH", tmpdir.path().display().to_string());
            chewing_new2(null(), null(), None, null_mut())
        };
        assert_phrase_only_in_user_dictionary(ctx)?;
        chewing_delete(ctx);
    }
    Ok(())
}

#[test]
fn env_load_and_create_user_path() -> Result<(), Box<dyn Error>> {
    use std::ptr::null;

    let syspath = syspath()?;
    let tmpdir = tempdir()?;
    let user_path = tmpdir.path().join("chewing");

    unsafe {
        let ctx = {
            let _lock = ENV_LOCK.lock()?;
            env::set_var("CHEWING_PATH", syspath.to_str()?);
            env::set_var("CHEWING_USER_PATH", user_path.display().to_string());
            chewing_new2(null(), null(), None, null_mut())
        };
        assert!(!ctx.is_null());
        chewing_delete(ctx);
    }
    Ok(())
}
