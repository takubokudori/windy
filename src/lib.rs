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
//! windy = "0.3.1"
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
//! windy = "0.3.1"
//! windy-macros = "0.2.0"
//! ```
//!
//! # License
//!
//! This software is released under the MIT or Apache-2.0 License, see LICENSE-MIT or LICENSE-APACHE.
#![cfg(windows)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
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
    pub(crate) use core::{cmp, convert, fmt, ops, ptr, slice};
}

#[cfg(feature = "std")]
#[allow(unused_imports)]
pub(crate) mod __lib {
    pub(crate) use std::{cmp, convert, fmt, ops, ptr, slice};
}

use __lib::fmt;

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
