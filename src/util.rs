use anyhow::{Context, Result};
use std::io::Write;
use std::{env, ffi::CStr, mem, path::Path, ptr};
use tempfile::NamedTempFile;

use tracing::instrument;

#[instrument(skip_all, err, fields(path = %path.display()))]
pub fn atomic_write(path: &Path, contents: &str) -> Result<()> {
    let parent = path.parent().context("failed to get parent directory")?;
    let mut tmp = NamedTempFile::new_in(parent)
        .with_context(|| format!("failed to create temp file in {:?}", parent))?;

    tmp.write_all(contents.as_bytes())
        .with_context(|| format!("failed to write to temp file for {:?}", path))?;

    tmp.as_file()
        .sync_all()
        .with_context(|| format!("failed to sync temp file for {:?}", path))?;

    tmp.persist(path)
        .with_context(|| format!("failed to persist temp file to {:?}", path))?;

    Ok(())
}

#[instrument(ret)]
pub fn get_shell() -> Option<String> {
    let cfg_shell = crate::config::get().shell.clone();
    cfg_shell.or_else(|| {
        if env::var("IN_NIX_SHELL").ok().is_none() {
            env::var("SHELL").ok()
        } else {
            get_shell_by_pwd()
        }
    })
}

fn get_shell_by_pwd() -> Option<String> {
    let mut passwd = unsafe { mem::zeroed::<libc::passwd>() };
    let mut buf = vec![0; 2048];
    let mut result = ptr::null_mut::<libc::passwd>();

    let uid = unsafe { libc::getuid() };

    loop {
        let r =
            unsafe { libc::getpwuid_r(uid, &mut passwd, buf.as_mut_ptr(), buf.len(), &mut result) };

        if r != libc::ERANGE {
            break;
        }

        let newsize = buf.len().checked_mul(2)?;
        buf.resize(newsize, 0);
    }

    if result.is_null() || result != &mut passwd {
        return None;
    }

    let shell_ptr = passwd.pw_shell;
    if shell_ptr.is_null() {
        return None;
    }

    unsafe { CStr::from_ptr(shell_ptr).to_str().ok().map(String::from) }
}
