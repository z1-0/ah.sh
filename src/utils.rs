use std::{env, ffi::CStr, mem, ptr};

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

pub fn get_shell_by_pwd() -> Option<String> {
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

    if result.is_null() {
        // There is no such user, or an error has occurred.
        // errno gets set if there’s an error.
        return None;
    }

    if result != &mut passwd {
        // The result of getpwuid_r should be its input passwd.
        return None;
    }

    let shell_ptr = passwd.pw_shell;
    if shell_ptr.is_null() {
        return None;
    }

    unsafe { CStr::from_ptr(shell_ptr).to_str().ok().map(String::from) }
}
