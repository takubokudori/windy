// Copyright takubokudori.
// This source code is licensed under the MIT or Apache-2.0 license.
//! # Windy
//!
//! [![crates.io](https://img.shields.io/crates/v/windy.svg)](https://crates.io/crates/windy)
//! [![docs.rs](https://docs.rs/windy/badge.svg)](https://docs.rs/windy)
//!
//! Windows string library for ANSI strings and wide strings.
//!
//! # Features
//!
//! - Owned ANSI strings: [`AString`].
//! - Owned wide strings: [`WString`].
//! - Borrowed ANSI strings: [`AStr`].
//! - Borrowed wide strings: [`WStr`].
//! - Dynamically-code-paged ANSI strings: [`DAString`] and [`DAStr`].
//! - Windows `ANSI_STRING` and `UNICODE_STRING` wrappers.
//! - Conversions between ANSI strings, wide strings, and UTF-8 [`String`] values.
//! - `no_std` support for borrowed string types.
//!
//! # Installation
//!
//! Add this crate to `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! windy = "0.4.0"
//! ```
//!
//! # no_std support
//!
//! Disable default features to build without `std`:
//!
//! ```text
//! --no-default-features
//! ```
//!
//! Owned string types are available only with the `std` feature.
//!
//! # Macros
//!
//! The companion `windy-macros` crate provides compile-time conversion from
//! UTF-8 string literals to [`WString`] or [`AString`].
//!
//! ```toml
//! [dependencies]
//! windy = "0.4.0"
//! windy-macros = "0.3.0"
//! ```
//!
//! # License
//!
//! This software is released under the MIT or Apache-2.0 License, see LICENSE-MIT or LICENSE-APACHE.
#![cfg(windows)]
#![cfg_attr(not(feature = "std"), no_std)]

mod convert;
mod ntstring;
mod raw;
#[cfg(feature = "std")]
mod string;
#[cfg(feature = "std")]
pub mod traits;
mod windy_str;

pub use ntstring::*;
use raw::*;
#[cfg(feature = "std")]
pub use string::*;
pub use windy_str::*;

#[cfg(not(feature = "std"))]
#[allow(unused_imports)]
pub(crate) mod __lib {
    pub(crate) use core::{
        borrow, char, cmp, convert, error, fmt, hash, mem, ops, ptr, slice, str,
    };
}

#[cfg(feature = "std")]
#[allow(unused_imports)]
pub(crate) mod __lib {
    pub(crate) use std::{
        borrow, char, cmp, convert, error, fmt, hash, mem, ops, ptr, slice, str,
    };
}

use crate::convert::validate_mb;
use __lib::fmt;

/// An invalid NUL-terminated string format.
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum StringFormatError {
    /// The input contains a NUL before the final terminator.
    ///
    /// For ANSI strings, `position` is a byte index.
    /// For wide strings, `position` is a `u16` element index.
    InteriorNul(usize),
    /// The input does not contain a terminating NUL.
    NotNulTerminated,
}

impl fmt::Display for StringFormatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InteriorNul(position) => {
                write!(f, "interior NUL found at index {position}")
            }
            Self::NotNulTerminated => {
                f.write_str("the string is not NUL-terminated")
            }
        }
    }
}

impl __lib::error::Error for StringFormatError {}

/// An error returned by string validation or conversion.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ConvertError {
    /// ANSI validation is failed.
    InvalidAnsiString,
    /// Unicode validation is failed.
    InvalidUnicodeString,
    /// Failed to convert to ANSI string.
    ConvertToAnsiError,
    /// Failed to convert to wide string.
    ConvertToUnicodeError,
    /// The code page is invalid.
    InvalidCodePage(u32),
    /// The two code pages do not match.
    CodePageMismatch { left: u32, right: u32 },
    /// The input is not a valid string.
    InvalidStringFormat(StringFormatError),
    /// A raw Windows error code returned by an underlying API.
    OsError(u32),
    /// The caller-provided output buffer was too small.
    InsufficientBuffer,
    /// An integer conversion failed while preparing arguments for a Windows API.
    TryFromIntError,
}

impl ConvertError {
    /// Returns the Windows error code for this error.
    ///
    /// Format errors that are detected before calling a Windows API are mapped
    /// to [`ERROR_INVALID_PARAMETER`].
    #[inline]
    pub fn as_error_code(&self) -> u32 {
        match self {
            Self::InvalidAnsiString => ERROR_NO_UNICODE_TRANSLATION,
            Self::InvalidUnicodeString => ERROR_NO_UNICODE_TRANSLATION,
            Self::ConvertToAnsiError => ERROR_NO_UNICODE_TRANSLATION,
            Self::ConvertToUnicodeError => ERROR_NO_UNICODE_TRANSLATION,
            Self::InvalidStringFormat(_) => ERROR_INVALID_PARAMETER,
            ConvertError::InvalidCodePage(_) => ERROR_INVALID_PARAMETER,
            ConvertError::CodePageMismatch { left: _, right: _ } => {
                ERROR_INVALID_PARAMETER
            }
            ConvertError::TryFromIntError => ERROR_INVALID_PARAMETER,
            ConvertError::OsError(e) => *e,
            ConvertError::InsufficientBuffer => ERROR_INSUFFICIENT_BUFFER,
        }
    }
}

impl __lib::error::Error for ConvertError {}

impl fmt::Display for ConvertError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidAnsiString => {
                f.write_str("the input is not a valid ANSI string")
            }
            Self::InvalidUnicodeString => {
                f.write_str("the input is not a valid UTF-16 string")
            }
            Self::ConvertToAnsiError => f.write_str(
                "failed to encode the string using the requested code page",
            ),
            Self::ConvertToUnicodeError => f.write_str(
                "failed to decode the string using the requested code page",
            ),
            Self::InvalidCodePage(code_page) => {
                write!(f, "invalid Windows code page: {code_page}")
            }
            Self::CodePageMismatch { left, right } => {
                write!(
                    f,
                    "code page mismatch: left is {left}, but right is {right}",
                )
            }
            Self::InvalidStringFormat(error) => {
                write!(f, "invalid string format: {error}")
            }
            Self::OsError(error_code) => {
                write!(
                    f,
                    "Windows API failed with error code {error_code} \
                     (0x{error_code:08X})",
                )
            }
            Self::InsufficientBuffer => {
                f.write_str("the output buffer is too small")
            }
            Self::TryFromIntError => f.write_str(
                "integer conversion failed while preparing a Windows API \
                 argument",
            ),
        }
    }
}

impl From<ConvertError> for u32 {
    fn from(x: ConvertError) -> Self { x.as_error_code() }
}

impl From<ConvertError> for i32 {
    fn from(x: ConvertError) -> Self { x.as_error_code() as i32 }
}

impl From<StringFormatError> for ConvertError {
    fn from(x: StringFormatError) -> Self { Self::InvalidStringFormat(x) }
}

/// Result type for fallible string operations.
pub type ConvertResult<T> = Result<T, ConvertError>;

pub const CP_ACP: u32 = 0;
pub const CP_UTF8: u32 = 65001;
pub const MB_ERR_INVALID_CHARS: DWORD = 0x8;
pub const WC_ERR_INVALID_CHARS: DWORD = 0x80;
pub const WC_NO_BEST_FIT_CHARS: DWORD = 0x400;
pub const ERROR_INVALID_PARAMETER: DWORD = 0x57;
pub const ERROR_INSUFFICIENT_BUFFER: DWORD = 0x7a;
pub const ERROR_NO_UNICODE_TRANSLATION: DWORD = 0x459;

/// Verifies that any NUL in `x` is the final byte.
#[inline]
pub(crate) fn check_interior_nul_u8(x: &[u8]) -> ConvertResult<()> {
    let Some((_last, body)) = x.split_last() else {
        return Ok(());
    };

    match body.iter().position(|&b| b == 0) {
        None => Ok(()),
        Some(pos) => Err(StringFormatError::InteriorNul(pos).into()),
    }
}

/// Verifies that a UTF-8 string contains no NUL.
#[inline]
#[allow(unused)]
pub(crate) fn check_nul_in_str(x: &str) -> ConvertResult<()> {
    match x.as_bytes().iter().position(|&b| b == 0) {
        None => Ok(()),
        Some(pos) => Err(StringFormatError::InteriorNul(pos).into()),
    }
}

/// Verifies that any NUL in `x` is the final element.
#[inline]
pub(crate) fn check_interior_nul_u16(x: &[u16]) -> ConvertResult<()> {
    let Some((_last, body)) = x.split_last() else {
        return Ok(());
    };

    match body.iter().position(|&b| b == 0) {
        None => Ok(()),
        Some(pos) => Err(StringFormatError::InteriorNul(pos).into()),
    }
}

/// Returns the index of the first NUL in `x`.
#[inline]
pub(crate) fn find_nul_u8(x: &[u8]) -> Option<usize> {
    x.iter().position(|&b| b == 0)
}

/// Returns the index of the first NUL in `x`.
#[inline]
pub(crate) fn find_nul_u16(x: &[u16]) -> Option<usize> {
    x.iter().position(|&b| b == 0)
}

/// Verifies that `x` ends with a NUL.
#[inline]
pub(crate) fn check_nul_terminated_u8(x: &[u8]) -> ConvertResult<()> {
    if x.last() == Some(&0) {
        Ok(())
    } else {
        Err(StringFormatError::NotNulTerminated.into())
    }
}

/// Verifies that `x` ends with a NUL.
#[inline]
pub(crate) fn check_nul_terminated_u16(x: &[u16]) -> ConvertResult<()> {
    if x.last() == Some(&0) {
        Ok(())
    } else {
        Err(StringFormatError::NotNulTerminated.into())
    }
}

/// Returns whether `code_page` identifies a code page installed on the
/// current operating system, as determined by
/// [`IsValidCodePage`](https://learn.microsoft.com/en-us/windows/win32/api/winnls/nf-winnls-isvalidcodepage).
///
/// Context-dependent selector values such as `CP_ACP` are not considered
/// installed code-page identifiers by this function. Therefore, this
/// function accepts a narrower set of values than conversion APIs and types
/// such as [`AStr`].
///
/// Use [`is_acceptable_code_page`] to determine whether a value can be used
/// as a code-page argument by this crate.
pub fn is_valid_code_page(code_page: u32) -> bool {
    unsafe { IsValidCodePage(code_page) != 0 }
}

/// Returns `true` if the specified code page identifier can be used with types such as [`AStr`].
pub fn is_acceptable_code_page(code_page: u32) -> bool {
    match validate_mb(code_page, &[0]) {
        Ok(_) => true,
        Err(ConvertError::InvalidCodePage(_)) => false,
        Err(e) => {
            unreachable!(
                "internal error while checking code page {code_page}: {e}"
            )
        }
    }
}
