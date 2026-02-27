// Copyright takubokudori.
// This source code is licensed under the MIT or Apache-2.0 license.
use crate::{
    __lib::ptr::null_mut,
    AStr, WStr,
    raw::{
        ANSI_STRING, RtlInitAnsiString, RtlInitUnicodeString, UNICODE_STRING,
    },
};
use core::ops;

/// Represents [UNICODE_STRING](https://docs.microsoft.com/en-us/windows/win32/api/ntdef/ns-ntdef-_unicode_string).
#[derive(Debug, Clone)]
pub struct UnicodeString<'a> {
    us: UNICODE_STRING,
    s: &'a WStr,
}

impl<'a> UnicodeString<'a> {
    /// Creates UnicodeString.
    pub fn new(s: &'a WStr) -> Self {
        let mut us = UNICODE_STRING {
            Length: 0,
            MaximumLength: 0,
            Buffer: null_mut(),
        };
        unsafe {
            RtlInitUnicodeString(&mut us, s.as_ptr());
        }
        Self { us, s }
    }

    /// Returns &[`UNICODE_STRING`].
    pub fn as_raw(&self) -> &UNICODE_STRING { &self.us }

    /// Returns *const [`UNICODE_STRING`].
    pub fn as_ptr(&self) -> *const UNICODE_STRING { &self.us as _ }

    /// Returns *mut [`UNICODE_STRING`].
    pub fn as_mut_ptr(&mut self) -> *mut UNICODE_STRING { &mut self.us as _ }
}

impl<'a> ops::Deref for UnicodeString<'a> {
    type Target = WStr;

    fn deref(&self) -> &Self::Target { self.s }
}

impl PartialEq for UnicodeString<'_> {
    fn eq(&self, other: &Self) -> bool { self.s.eq(other.s) }
}

impl Eq for UnicodeString<'_> {}

/// Represents [ANSI_STRING](https://docs.microsoft.com/en-us/windows/win32/api/ntdef/ns-ntdef-string).
#[derive(Debug, Clone)]
pub struct AnsiString<'a> {
    us: ANSI_STRING,
    s: &'a AStr,
}

impl<'a> AnsiString<'a> {
    /// Creates AnsiString.
    pub fn new(s: &'a AStr) -> Self {
        let mut us = ANSI_STRING {
            Length: 0,
            MaximumLength: 0,
            Buffer: null_mut(),
        };
        unsafe {
            RtlInitAnsiString(&mut us, s.as_ptr());
        }
        Self { us, s }
    }

    /// Returns &[`ANSI_STRING`].
    pub fn as_raw(&self) -> &ANSI_STRING { &self.us }

    /// Returns *const [`ANSI_STRING`].
    pub fn as_ptr(&self) -> *const ANSI_STRING { &self.us as _ }

    /// Returns *mut [`ANSI_STRING`].
    pub fn as_mut_ptr(&mut self) -> *mut ANSI_STRING { &mut self.us as _ }
}

impl<'a> ops::Deref for AnsiString<'a> {
    type Target = AStr;

    fn deref(&self) -> &Self::Target { self.s }
}

impl PartialEq for AnsiString<'_> {
    fn eq(&self, other: &Self) -> bool { self.s.eq(other.s) }
}

impl Eq for AnsiString<'_> {}
