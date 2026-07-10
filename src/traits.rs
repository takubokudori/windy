// Copyright takubokudori.
// This source code is licensed under the MIT or Apache-2.0 license.
use crate::*;

/// Converts a value into a [`WString`].
pub trait ToWString {
    /// Converts this value into a [`WString`].
    ///
    /// # Panics
    ///
    /// Panics if the input contains an embedded NUL,
    /// if an unchecked source value violates this crate's string invariants,
    /// or if the underlying conversion fails.
    /// Use [`try_to_wstring`](Self::try_to_wstring) when invalid input should be reported as a [`ConvertError`] instead of panicking.
    fn to_wstring(&self) -> WString {
        self.try_to_wstring().expect("Failed to convert to WString")
    }

    /// Attempts to convert this value into a [`WString`].
    fn try_to_wstring(&self) -> ConvertResult<WString>;
}

/// Converts a value into an [`AString`] using the compile-time code page `CP`.
pub trait ToAString<const CP: u32> {
    /// Converts this value into an [`AString`].
    ///
    /// # Panics
    ///
    /// Panics if the input contains an embedded NUL,
    /// if the target code page cannot represent the input without loss, if Windows rejects the code page,
    /// or if an unchecked source value violates this crate's string invariants.
    /// Use [`try_to_astring`](Self::try_to_astring) when invalid input should be reported as a [`ConvertError`] instead of panicking.
    fn to_astring(&self) -> AString<CP> {
        self.try_to_astring().expect("Failed to convert to AString")
    }

    /// Attempts to convert this value into an [`AString`] using code page `CP`.
    fn try_to_astring(&self) -> ConvertResult<AString<CP>>;

    /// Converts this value into an [`AString`] using replacement when required by code page `CP`.
    ///
    /// # Panics
    ///
    /// Panics if Windows rejects `CP`, if the underlying Windows conversion fails even in lossy mode,
    /// or if an unchecked source value violates this crate's string invariants.
    /// Use [`try_to_astring_lossy`](Self::try_to_astring_lossy) to handle that error explicitly.
    fn to_astring_lossy(&self) -> AString<CP> {
        self.try_to_astring_lossy()
            .expect("Failed to convert to AString")
    }

    /// Attempts to convert this value into an [`AString`] using replacement when required by code page `CP`.
    fn try_to_astring_lossy(&self) -> ConvertResult<AString<CP>>;
}

/// Converts a value into a [`DAString`] using a runtime-selected code page.
pub trait ToDAString {
    /// Converts this value into a [`DAString`].
    ///
    /// `code_page` is passed to the Windows conversion APIs.
    ///
    /// # Panics
    ///
    /// Panics if the input contains an embedded NUL, if `code_page` cannot represent the
    /// input without loss, if Windows rejects `code_page`, or if an unchecked source value violates this crate's string invariants.
    /// Use [`try_to_dastring`](Self::try_to_dastring) when invalid input
    /// or an invalid code page should be reported as a [`ConvertError`] instead of panicking.
    fn to_dastring(&self, code_page: u32) -> DAString {
        self.try_to_dastring(code_page)
            .expect("Failed to convert to DAString")
    }

    /// Attempts to convert this value into a [`DAString`] using `code_page`.
    fn try_to_dastring(&self, code_page: u32) -> ConvertResult<DAString>;

    /// Converts this value into a [`DAString`] using replacement when required by `code_page`.
    ///
    /// # Panics
    ///
    /// Panics if Windows rejects `code_page`,
    /// if the underlying Windows conversion fails even in lossy mode,
    /// or if an unchecked source value violates this crate's string invariants.
    /// Use [`try_to_dastring_lossy`](Self::try_to_dastring_lossy) to handle that error explicitly.
    fn to_dastring_lossy(&self, code_page: u32) -> DAString {
        self.try_to_dastring_lossy(code_page)
            .expect("Failed to convert to DAString")
    }

    /// Attempts to convert this value into a [`DAString`] using replacement when required by `code_page`.
    fn try_to_dastring_lossy(&self, code_page: u32) -> ConvertResult<DAString>;
}

macro_rules! impl_to_wstring {
    ($x:ident) => {
        impl $crate::traits::ToWString for $x {
            /// Converts this UTF-8 string into a [`WString`].
            ///
            /// # Panics
            ///
            /// Panics if it contains an embedded NUL character.
            fn to_wstring(&self) -> WString {
                self.try_to_wstring().expect("Failed to convert to WString")
            }

            /// Attempts to convert this UTF-8 string into a [`WString`].
            fn try_to_wstring(&self) -> ConvertResult<WString> {
                $crate::WString::from_utf8(self)
            }
        }
    };
}

macro_rules! impl_to_astring {
    ($x:ident) => {
        impl<const CP: u32> ToAString<CP> for $x {
            /// Attempts to convert this UTF-8 string into an [`AString`] using code page `CP`.
            fn try_to_astring(&self) -> ConvertResult<AString<CP>> {
                AString::from_utf8(self)
            }

            /// Attempts to convert this UTF-8 string into an [`AString`]
            /// with lossy replacement where required by the target code page.
            fn try_to_astring_lossy(&self) -> ConvertResult<AString<CP>> {
                AString::try_from_utf8_lossy(self)
            }
        }
    };
}

macro_rules! impl_to_dastring {
    ($x:ident) => {
        impl ToDAString for $x {
            /// Attempts to convert this UTF-8 string into a [`DAString`] using `code_page`.
            fn try_to_dastring(
                &self,
                code_page: u32,
            ) -> ConvertResult<DAString> {
                DAString::from_utf8(code_page, self)
            }

            /// Attempts to convert this UTF-8 string into a [`DAString`]
            /// with lossy replacement where required by `code_page`.
            fn try_to_dastring_lossy(
                &self,
                code_page: u32,
            ) -> ConvertResult<DAString> {
                DAString::try_from_utf8_lossy(code_page, self)
            }
        }
    };
}

impl_to_wstring!(String);
impl_to_astring!(String);
impl_to_dastring!(String);
impl_to_wstring!(str);
impl_to_astring!(str);
impl_to_dastring!(str);
