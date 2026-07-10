// Copyright takubokudori.
// This source code is licensed under the MIT or Apache-2.0 license.
#![allow(dead_code)]
#[cfg(feature = "std")]
use windy::*;

const ERROR_SUCCESS: u32 = 0x00;
const ERROR_ENVVAR_NOT_FOUND: u32 = 0xcb;
const ERROR_INSUFFICIENT_BUFFER: u32 = 0x7a;

#[allow(non_snake_case)]
#[cfg(feature = "std")]
unsafe extern "system" {
    fn GetEnvironmentVariableA(
        lpName: *const i8,
        lpBuffer: *mut u8,
        nSize: u32,
    ) -> u32;

    fn GetEnvironmentVariableW(
        lpName: *const u16,
        lpBuffer: *mut u16,
        nSize: u32,
    ) -> u32;

    fn GetLastError() -> u32;

    fn SetLastError(dwErrCode: u32);
}

#[cfg(feature = "std")]
fn get_environment_variable_a(
    name: impl AsRef<ACPStr>,
) -> Result<Option<ACPString>, u32> {
    let name = name.as_ref();
    unsafe {
        SetLastError(ERROR_SUCCESS);
        let l = GetEnvironmentVariableA(name.as_ptr(), std::ptr::null_mut(), 0);
        if l == 0 {
            return match GetLastError() {
                ERROR_SUCCESS => Ok(Some(ACPString::from_utf8_lossy(""))),
                ERROR_ENVVAR_NOT_FOUND => Ok(None),
                e => Err(e),
            };
        }
        SetLastError(ERROR_SUCCESS);
        let mut buf = Vec::with_capacity(l as usize);
        let l2 = GetEnvironmentVariableA(name.as_ptr(), buf.as_mut_ptr(), l);
        if l2 == 0 {
            return match GetLastError() {
                ERROR_SUCCESS => Ok(Some(ACPString::from_utf8_lossy(""))),
                ERROR_ENVVAR_NOT_FOUND => Ok(None),
                e => Err(e),
            };
        }
        if l2 >= l {
            return Err(ERROR_INSUFFICIENT_BUFFER);
        }
        // SAFETY: GetEnvironmentVariableA writes a null-terminated ANSI string to buf.
        buf.set_len(l2 as usize + 1);
        Ok(Some(ACPString::from_vec_with_nul(buf)?))
    }
}

#[cfg(feature = "std")]
fn get_environment_variable_w(
    name: impl AsRef<WStr>,
) -> Result<Option<WString>, u32> {
    let name = name.as_ref();
    unsafe {
        SetLastError(ERROR_SUCCESS);
        let l = GetEnvironmentVariableW(name.as_ptr(), std::ptr::null_mut(), 0);
        if l == 0 {
            return match GetLastError() {
                ERROR_SUCCESS => Ok(Some(WString::from_utf8("")?)),
                ERROR_ENVVAR_NOT_FOUND => Ok(None),
                e => Err(e),
            };
        }
        SetLastError(ERROR_SUCCESS);
        let mut buf = Vec::with_capacity(l as usize);
        let l2 = GetEnvironmentVariableW(name.as_ptr(), buf.as_mut_ptr(), l);
        if l2 == 0 {
            return match GetLastError() {
                ERROR_SUCCESS => Ok(Some(WString::from_utf8("")?)),
                ERROR_ENVVAR_NOT_FOUND => Ok(None),
                e => Err(e),
            };
        }
        if l2 >= l {
            return Err(ERROR_INSUFFICIENT_BUFFER);
        }
        buf.set_len(l2 as usize + 1);
        // SAFETY: GetEnvironmentVariableW writes a null-terminated Unicode string to buf.
        Ok(Some(WString::from_vec_with_nul(buf)?))
    }
}

#[cfg(feature = "std")]
fn main() {
    println!("*****get_environment_variable_a*****");
    match get_environment_variable_a(ACPString::from_utf8_lossy("PATH")) {
        Ok(None) => {
            println!("PATH not found");
        }
        Ok(Some(s)) => {
            println!("PATH = {s}");
        }
        Err(e) => {
            println!("get_environment_variable_a failed: {e}");
        }
    }

    println!("*****get_environment_variable_w*****");
    match get_environment_variable_w(WString::from_utf8("PATH").unwrap()) {
        Ok(None) => {
            println!("PATH not found");
        }
        Ok(Some(s)) => {
            println!("PATH = {s}");
        }
        Err(e) => {
            println!("get_environment_variable_w failed: {e}");
        }
    }
}

#[cfg(not(feature = "std"))]
fn main() {
    panic!("Use std feature!");
}
