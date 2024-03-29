// Copyright takubokudori.
// This source code is licensed under the MIT or Apache-2.0 license.
//! # Windy
//!
//! [![crates.io](https://img.shields.io/crates/v/windy.svg)](https://crates.io/crates/windy)
//! [![docs.rs](https://docs.rs/windy/badge.svg)](https://docs.rs/windy)
//!
//! A Windows strings library that supports AString (ANSI string) and WString (Unicode string).
//!
//! # Features
//!
//! - ANSI string(AString)
//! - Wide string(WString)
//! - AnsiString(ANSI_STRING)
//! - UnicodeString(UNICODE_STRING)
//! - Interconversion between AString, WString and String.
//! - no_std support
//! - Macros support
//!
//! # Installation
//!
//! Add the following lines to your Cargo.toml:
//!
//! ```toml
//! [dependencies]
//! windy = "0.2.0"
//! ```
//!
//! # no_std support
//!
//! If you don't want to use std, use `--no-default-features`.
//!
//! AString and WString are not available when no_std.
//!
//! # Macros support
//!
//! [windy-macros](https://github.com/takubokudori/windy-macros) to convert a UTF-8 string to WString or AString at compile
//! time.
//!
//! ```toml
//! [dependencies]
//! windy = "0.2.0"
//! windy-macros = "0.1.1"
//! ```
//!
//! # License
//!
//! This software is released under the MIT or Apache-2.0 License, see LICENSE-MIT or LICENSE-APACHE.
#![cfg(windows)]
#![cfg_attr(not(feature = "std"), no_std)]

mod ntstring;
mod raw;
#[cfg(feature = "std")]
mod string;
#[cfg(feature = "std")]
mod traits;
mod windy_str;

pub use ntstring::*;
use raw::*;
#[cfg(feature = "std")]
pub use string::*;
#[cfg(feature = "macros")]
pub use windy_macros as macros;
pub use windy_str::*;

#[cfg(not(feature = "std"))]
#[allow(unused_imports)]
pub(crate) mod __lib {
    pub(crate) use core::{cmp, convert, fmt, ops, ptr, slice};
}

#[cfg(feature = "std")]
#[allow(unused_imports)]
pub(crate) mod __lib {
    pub(crate) use std::{cmp, convert, fmt, ops, ptr, slice};
}

#[allow(unused)]
pub(crate) const CP_ACP: UINT = 0;
#[allow(unused)]
pub(crate) const CP_UTF8: UINT = 65001;
#[allow(unused)]
pub(crate) const MB_ERR_INVALID_CHARS: DWORD = 0x8;
#[allow(unused)]
pub(crate) const WC_ERR_INVALID_CHARS: DWORD = 0x80;
#[allow(unused)]
pub(crate) const WC_NO_BEST_FIT_CHARS: DWORD = 0x400;
#[allow(unused)]
pub(crate) const ERROR_INSUFFICIENT_BUFFER: DWORD = 0x7a;
#[allow(unused)]
pub(crate) const ERROR_NO_UNICODE_TRANSLATION: DWORD = 0x459;

use __lib::{
    fmt,
    ptr::{null, null_mut},
};

/// Represents a conversion error.
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum ConvertError {
    /// Failed to convert to UTF-8 string.
    ConvertToUtf8Error(u32),
    /// Failed to convert to ANSI string.
    ConvertToAnsiError(u32),
    /// Failed to convert to Unicode string.
    ConvertToUnicodeError(u32),
}

impl ConvertError {
    /// Returns a os error code.
    #[inline]
    pub fn as_error_code(&self) -> u32 {
        match self {
            Self::ConvertToUtf8Error(x) => *x,
            Self::ConvertToAnsiError(x) => *x,
            Self::ConvertToUnicodeError(x) => *x,
        }
    }
}

impl fmt::Debug for ConvertError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let st = match self {
            Self::ConvertToUtf8Error(_) => "ConvertToUtf8Error",
            Self::ConvertToAnsiError(_) => "ConvertToAnsiError",
            Self::ConvertToUnicodeError(_) => "ConvertToUnicodeError",
        };
        #[cfg(feature = "std")]
        {
            let e =
                std::io::Error::from_raw_os_error(self.as_error_code() as i32);
            f.debug_struct(st).field("", &e).finish()
        }
        #[cfg(not(feature = "std"))]
        {
            f.debug_struct(st).field("", &self.as_error_code()).finish()
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ConvertError {}

impl fmt::Display for ConvertError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        __lib::fmt::Debug::fmt(self, f)
    }
}

impl From<ConvertError> for u32 {
    fn from(x: ConvertError) -> Self { x.as_error_code() }
}

impl From<ConvertError> for i32 {
    fn from(x: ConvertError) -> Self { x.as_error_code() as i32 }
}

#[macro_export]
macro_rules! conv_err {
    (@utf8 $e:expr) => {
        $crate::ConvertError::ConvertToUtf8Error($e)
    };
    (@ansi $e:expr) => {
        $crate::ConvertError::ConvertToAnsiError($e)
    };
    (@unicode $e:expr) => {
        $crate::ConvertError::ConvertToUnicodeError($e)
    };
    (@utf8) => {
        $crate::ConvertError::ConvertToUtf8Error
    };
    (@ansi) => {
        $crate::ConvertError::ConvertToAnsiError
    };
    (@unicode) => {
        $crate::ConvertError::ConvertToUnicodeError
    };
}

pub type ConvertResult<T> = Result<T, ConvertError>;

#[allow(unused)]
pub(crate) type OsResult<T> = Result<T, u32>;

#[inline(always)]
#[allow(non_snake_case)]
#[allow(clippy::too_many_arguments)]
#[allow(unused)]
pub(crate) unsafe fn MultiByteToWideChar(
    CodePage: UINT,
    dwFlags: DWORD,
    lpMultiByteStr: LPCSTR,
    cbMultiByte: c_int,
    lpWideCharStr: LPWSTR,
    cchWideChar: c_int,
) -> OsResult<c_int> {
    match crate::raw::MultiByteToWideChar(
        CodePage,
        dwFlags,
        lpMultiByteStr,
        cbMultiByte,
        lpWideCharStr,
        cchWideChar,
    ) {
        0 => Err(GetLastError()),
        x => Ok(x),
    }
}

#[inline(always)]
#[allow(non_snake_case)]
#[allow(clippy::too_many_arguments)]
#[allow(unused)]
pub(crate) unsafe fn WideCharToMultiByte(
    CodePage: UINT,
    dwFlags: DWORD,
    lpWideCharStr: LPCWSTR,
    cchWideChar: c_int,
    lpMultiByteStr: LPSTR,
    cbMultiByte: c_int,
    lpDefaultChar: LPCSTR,
    lpUsedDefaultChar: LPBOOL,
) -> OsResult<c_int> {
    match crate::raw::WideCharToMultiByte(
        CodePage,
        dwFlags,
        lpWideCharStr,
        cchWideChar,
        lpMultiByteStr,
        cbMultiByte,
        lpDefaultChar,
        lpUsedDefaultChar,
    ) {
        0 => Err(GetLastError()),
        x => Ok(x),
    }
}

/// Safe wrapper function of MultiByteToWideChar.
#[inline(always)]
#[allow(unused)]
pub(crate) fn multi_byte_to_wide_char(
    code_page: UINT,
    mb_flags: DWORD,
    mb_bytes: &[u8],
    wc_bytes: &mut [u16],
) -> OsResult<usize> {
    unsafe {
        MultiByteToWideChar(
            code_page,
            mb_flags,
            mb_bytes.as_ptr() as *const i8,
            mb_bytes.len() as i32,
            wc_bytes.as_mut_ptr(),
            wc_bytes.len() as i32,
        )
        .map(|x| x as usize)
    }
}

/// Safe wrapper function of WideCharToMultiByte.
#[inline(always)]
#[allow(unused)]
pub(crate) fn wide_char_to_multi_byte<'a, DC, UDC>(
    code_page: UINT,
    wc_flags: DWORD,
    wc_bytes: &[u16],
    mb_bytes: &mut [u8],
    default_char: DC,
    used_default_char: UDC,
) -> OsResult<usize>
where
    DC: Into<Option<char>>,
    UDC: Into<Option<&'a mut bool>>,
{
    let dc = default_char.into().map_or(null(), |x| &x);
    unsafe {
        WideCharToMultiByte(
            code_page,
            wc_flags,
            wc_bytes.as_ptr(),
            wc_bytes.len() as i32,
            mb_bytes.as_mut_ptr() as *mut i8,
            mb_bytes.len() as i32,
            dc as *const i8,
            used_default_char
                .into()
                .map_or(null_mut(), |x| x as *mut _ as *mut _),
        )
        .map(|x| x as usize)
    }
}

#[cfg(feature = "std")]
pub(crate) fn wide_char_to_multi_byte_wrap(
    code_page: UINT,
    wc_flags: DWORD,
    x: &[u16],
    used_default_char: bool,
) -> OsResult<Vec<u8>> {
    let x = if x.is_empty() { &[0x00] } else { x };
    let l = x.len() * 4;
    let mut ret: Vec<u8> = Vec::with_capacity(l);
    unsafe {
        ret.set_len(l);
    }
    let mut udc_flag = false;
    let udc = if used_default_char {
        Some(&mut udc_flag)
    } else {
        None
    };

    match wide_char_to_multi_byte(
        code_page,
        wc_flags,
        x,
        ret.as_mut_slice(),
        None,
        udc,
    ) {
        Ok(l2) => {
            if udc_flag {
                return Err(ERROR_NO_UNICODE_TRANSLATION);
            }
            unsafe {
                ret.set_len(l2);
            }
            Ok(ret)
        }
        Err(ERROR_INSUFFICIENT_BUFFER) => {
            #[cfg(feature = "debug_insufficient_buffer")]
            {
                println!("WCTMB: ERROR_INSUFFICIENT_BUFFER returned"); // for debug
            }
            wide_char_to_multi_byte2(code_page, wc_flags, x, used_default_char)
        }
        Err(x) => Err(x),
    }
}

/// Gets the required buffer size and gets a multi-byte string.
#[inline]
#[cfg(feature = "std")]
pub(crate) fn wide_char_to_multi_byte2(
    code_page: UINT,
    wc_flags: DWORD,
    x: &[u16],
    used_default_char: bool,
) -> OsResult<Vec<u8>> {
    // get the required buffer size.
    let l =
        wide_char_to_multi_byte(code_page, wc_flags, x, &mut [], None, None)?;
    let mut ret: Vec<u8> = Vec::with_capacity(l);
    unsafe {
        ret.set_len(l);
    }
    let mut udc_flag = false;
    let udc = if used_default_char {
        Some(&mut udc_flag)
    } else {
        None
    };

    let l2 = wide_char_to_multi_byte(
        code_page,
        wc_flags,
        x,
        ret.as_mut_slice(),
        None,
        udc,
    )?;
    if udc_flag {
        return Err(ERROR_NO_UNICODE_TRANSLATION);
    }
    assert_eq!(l, l2);
    Ok(ret)
}

#[cfg(feature = "std")]
pub(crate) fn multi_byte_to_wide_char_wrap(
    code_page: UINT,
    mb_flags: DWORD,
    x: &[u8],
) -> OsResult<Vec<u16>> {
    let x = if x.is_empty() { &[0x00] } else { x };
    let l = x.len();
    let mut ret: Vec<u16> = Vec::with_capacity(l);
    unsafe {
        ret.set_len(l);
    }

    match multi_byte_to_wide_char(code_page, mb_flags, x, ret.as_mut_slice()) {
        Ok(l2) => {
            unsafe {
                ret.set_len(l2);
            }
            Ok(ret)
        }
        Err(ERROR_INSUFFICIENT_BUFFER) => {
            #[cfg(feature = "debug_insufficient_buffer")]
            {
                println!("MBTWC: ERROR_INSUFFICIENT_BUFFER returned"); // for debug
            }
            multi_byte_to_wide_char2(code_page, mb_flags, x)
        }
        Err(x) => Err(x),
    }
}

/// Gets the required buffer size and gets a wide string.
#[inline]
#[cfg(feature = "std")]
pub(crate) fn multi_byte_to_wide_char2(
    code_page: UINT,
    mb_flags: DWORD,
    x: &[u8],
) -> OsResult<Vec<u16>> {
    // get the required buffer size.
    let l = multi_byte_to_wide_char(code_page, mb_flags, x, &mut [])?;
    let mut ret: Vec<u16> = Vec::with_capacity(l);
    unsafe {
        ret.set_len(l);
    }

    let l2 =
        multi_byte_to_wide_char(code_page, mb_flags, x, ret.as_mut_slice())?;
    assert_eq!(l, l2);
    Ok(ret)
}
