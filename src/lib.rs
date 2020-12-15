// Copyright takubokudori.
// This source code is licensed under the MIT or Apache-2.0 license.
//! # Windy
//!
//! Windows strings library
//!
//! This crate supports AString (ANSI string) and WString (Unicode string).
//!
//! # Features
//!
//! - ANSI string(AString)
//! - Unicode string(WString)
//! - Interconversion between AString, WString and String.
//!
//! # Example
//!
//! An example of parsing the outputs of cmd.exe.
//!
//! ```rust
//! use windy::AString;
//! use std::process::Command;
//!
//! fn main() {
//!     let o = Command::new("cmd")
//!         .args(&["/c", "ThisCommandDoesNotExist"])
//!         .output().unwrap();
//!     let (stdout, stderr) = unsafe {
//!         (
//!             AString::new_unchecked(o.stdout),
//!             AString::new_unchecked(o.stderr)
//!         )
//!     };
//!     println!("stdout: {:?}", stdout);
//!     println!("stderr: {:?}", stderr);
//! }
//! ```
//! # License
//!
//! This software is released under the MIT or Apache-2.0 License, see LICENSE-MIT or LICENSE-APACHE.
#![cfg(windows)]

mod raw;
mod string;

pub use string::*;
use raw::*;
use std::fmt;
use std::ptr::{null, null_mut};

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
    pub fn to_error_code(&self) -> u32 {
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
        let e = std::io::Error::from_raw_os_error(self.to_error_code() as i32);
        f.debug_struct(st)
            .field("", &e)
            .finish()
    }
}

impl std::error::Error for ConvertError {}

impl fmt::Display for ConvertError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        format!("{:?}", self).fmt(f)
    }
}

impl Into<u32> for ConvertError {
    fn into(self) -> u32 {
        self.to_error_code()
    }
}

impl Into<i32> for ConvertError {
    fn into(self) -> i32 {
        self.to_error_code() as i32
    }
}

#[macro_export]
macro_rules! conv_err {
    (@utf8 $e:expr) => (ConvertError::ConvertToUtf8Error($e));
    (@ansi $e:expr) => (ConvertError::ConvertToAnsiError($e));
    (@unicode $e:expr) => (ConvertError::ConvertToUnicodeError($e));
}

pub type ConvertResult<T> = Result<T, ConvertError>;

pub(crate) type OsResult<T> = Result<T, u32>;

#[inline(always)]
#[allow(non_snake_case)]
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
        ).and_then(|x| Ok(x as usize))
    }
}

/// Safe wrapper function of WideCharToMultiByte.
#[inline(always)]
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
        UDC: Into<Option<&'a mut bool>>
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
            used_default_char.into().map_or(null_mut(), |x| x as *mut _ as *mut _),
        ).and_then(|x| Ok(x as usize))
    }
}
