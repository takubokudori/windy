// Copyright takubokudori.
// This source code is licensed under the MIT or Apache-2.0 license.
use crate::{
    AStr, ConvertError, ConvertResult, WStr,
    raw::{PSTR, PWSTR, USHORT},
};
use core::ops;

/// [UNICODE_STRING](https://learn.microsoft.com/en-us/windows/win32/api/ntdef/ns-ntdef-_unicode_string)
#[allow(non_snake_case, non_camel_case_types)]
#[repr(C)]
#[derive(Debug, Clone)]
pub struct UNICODE_STRING {
    pub Length: USHORT,
    pub MaximumLength: USHORT,
    pub Buffer: PWSTR,
}

/// [ANSI_STRING](https://learn.microsoft.com/en-us/windows/win32/api/ntdef/ns-ntdef-string)
#[allow(non_snake_case, non_camel_case_types)]
#[repr(C)]
#[derive(Debug, Clone)]
pub struct ANSI_STRING {
    pub Length: USHORT,
    pub MaximumLength: USHORT,
    pub Buffer: PSTR,
}

/// A borrowed wrapper for a Windows [`UNICODE_STRING`].
#[derive(Debug, Clone)]
pub struct UnicodeString<'a> {
    us: UNICODE_STRING,
    s: &'a WStr,
}

impl<'a> UnicodeString<'a> {
    /// Creates a borrowed [`UNICODE_STRING`] wrapper for `s`.
    ///
    /// Returns [`ConvertError::ConvertToUnicodeError`](crate::ConvertError::ConvertToUnicodeError)
    /// if the input is too long for a `UNICODE_STRING` length field.
    pub fn new(s: &'a WStr) -> ConvertResult<Self> {
        if s.as_u8_bytes_with_nul().len() > u16::MAX as usize {
            return Err(ConvertError::TryFromIntError);
        }

        let us = UNICODE_STRING {
            Length: s.as_u8_bytes().len() as u16,
            MaximumLength: s.as_u8_bytes_with_nul().len() as u16,
            Buffer: s.as_ptr() as *mut u16,
        };

        Ok(Self { us, s })
    }

    /// Returns a shared reference to the raw [`UNICODE_STRING`] representation.
    pub fn as_raw(&self) -> &UNICODE_STRING { &self.us }

    /// Returns a mutable reference to the raw [`UNICODE_STRING`] representation.
    ///
    /// # Safety
    ///
    /// The returned value must not be modified in a way that makes it outlive
    /// or stop describing the borrowed [`WStr`] stored in this wrapper.
    pub unsafe fn as_raw_mut(&mut self) -> &mut UNICODE_STRING { &mut self.us }

    /// Returns a raw pointer to the raw [`UNICODE_STRING`] representation.
    pub fn as_ptr(&self) -> *const UNICODE_STRING { &self.us }

    /// Returns the borrowed [`WStr`] associated with this.
    pub fn as_wstr(&self) -> &'a WStr { self.s }
}

impl<'a> ops::Deref for UnicodeString<'a> {
    type Target = WStr;

    fn deref(&self) -> &Self::Target { self.s }
}

impl PartialEq for UnicodeString<'_> {
    fn eq(&self, other: &Self) -> bool { self.s.eq(other.s) }
}

impl Eq for UnicodeString<'_> {}

/// Alias for [`AnsiString`] using the process ANSI code page.
pub type ACPAnsiString<'a> = AnsiString<'a, 0>;

/// A borrowed wrapper for a Windows [`ANSI_STRING`].
#[derive(Debug, Clone)]
pub struct AnsiString<'a, const CP: u32> {
    us: ANSI_STRING,
    s: &'a AStr<CP>,
}

impl<'a, const CP: u32> AnsiString<'a, CP> {
    /// Creates a borrowed [`ANSI_STRING`] wrapper for `s`.
    ///
    /// Returns [`ConvertError::ConvertToAnsiError`](crate::ConvertError::ConvertToAnsiError)
    /// if the input is too long for an `ANSI_STRING` length field.
    pub fn new(s: &'a AStr<CP>) -> ConvertResult<Self> {
        if s.as_bytes_with_nul().len() > u16::MAX as usize {
            return Err(ConvertError::TryFromIntError);
        }
        let us = ANSI_STRING {
            Length: s.as_bytes().len() as u16,
            MaximumLength: s.as_bytes_with_nul().len() as u16,
            Buffer: s.as_ptr() as *mut i8,
        };
        Ok(Self { us, s })
    }

    /// Returns a shared reference to the raw [`ANSI_STRING`] representation.
    pub fn as_raw(&self) -> &ANSI_STRING { &self.us }

    /// Returns a mutable reference to the raw [`ANSI_STRING`] representation.
    ///
    /// # Safety
    ///
    /// The returned value must not be modified in a way that makes it outlive
    /// or stop describing the borrowed [`AStr`] stored in this wrapper.
    pub unsafe fn as_raw_mut(&mut self) -> &mut ANSI_STRING { &mut self.us }

    /// Returns a raw pointer to the raw [`ANSI_STRING`] representation.
    pub fn as_ptr(&self) -> *const ANSI_STRING { &self.us as _ }

    /// Returns the borrowed [`AStr`] associated with this.
    pub fn as_astr(&self) -> &'a AStr<CP> { self.s }
}

impl<'a, const CP: u32> ops::Deref for AnsiString<'a, CP> {
    type Target = AStr<CP>;

    fn deref(&self) -> &Self::Target { self.s }
}

impl<'a, const CP: u32> PartialEq for AnsiString<'a, CP> {
    fn eq(&self, other: &Self) -> bool { self.s.eq(other.s) }
}

impl<'a, const CP: u32> Eq for AnsiString<'a, CP> {}
