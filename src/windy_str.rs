// Copyright takubokudori.
// This source code is licensed under the MIT or Apache-2.0 license.
use crate::{
    __lib::{cmp::Ordering, fmt, hash, slice},
    *,
};

use crate::convert::*;
#[cfg(feature = "std")]
use crate::traits::*;
#[cfg(feature = "std")]
use std::{ffi::OsString, os::windows::ffi::OsStringExt};

/// A borrowed NUL-terminated wide string.
///
/// A valid `WStr` points to valid UTF-16 code units followed by a terminating NUL.
/// Safe constructors validate this.
/// Unsafe constructors and mutable byte accessors rely on the caller to preserve the invariant for the entire borrowed lifetime.
#[repr(transparent)]
pub struct WStr {
    inner: [u16],
}

impl WStr {
    /// Returns a raw pointer to the first wide character.
    #[inline]
    pub fn as_ptr(&self) -> *const u16 { self.inner.as_ptr() }

    /// Returns a mutable raw pointer to the first wide character.
    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut u16 { self.inner.as_mut_ptr() }

    /// Returns `true` if the string is empty.
    #[inline]
    pub fn is_empty(&self) -> bool { self.as_bytes().is_empty() }

    /// Returns the `u16` elements of the wide string, excluding the terminating NUL.
    #[inline]
    pub fn as_bytes(&self) -> &[u16] {
        let bytes = self.as_bytes_with_nul();
        &bytes[..bytes.len() - 1]
    }

    /// Returns the `u16` elements of the wide string, including the terminating NUL.
    #[inline]
    pub fn as_bytes_with_nul(&self) -> &[u16] { &self.inner }

    /// Returns a mutable slice of the `u16` elements, excluding the terminating NUL.
    ///
    /// # Safety
    ///
    /// The caller must preserve a valid wide string without interior NULs.
    #[inline]
    pub unsafe fn as_bytes_mut(&mut self) -> &mut [u16] {
        let bytes = unsafe { self.as_bytes_with_nul_mut() };
        let len = bytes.len();
        &mut bytes[..len - 1]
    }

    /// Returns a mutable slice of the `u16` elements, including the terminating NUL.
    ///
    /// # Safety
    ///
    /// The caller must preserve a valid, NUL-terminated wide string without
    /// interior NULs.
    #[inline]
    pub unsafe fn as_bytes_with_nul_mut(&mut self) -> &mut [u16] {
        &mut self.inner
    }

    /// Returns a byte view of the wide string, excluding the terminating NUL.
    ///
    /// The returned bytes are the in-memory representation of the stored `u16` code units.
    /// On Windows this corresponds to UTF-16LE bytes.
    #[inline]
    pub fn as_u8_bytes(&self) -> &[u8] {
        unsafe {
            slice::from_raw_parts(
                self.inner.as_ptr() as *const u8,
                (self.inner.len() - 1) * size_of::<u16>(),
            )
        }
    }

    /// Returns a byte view of the wide string, including the terminating NUL.
    ///
    /// The returned bytes are the in-memory representation of the stored `u16` code units, including the terminating NUL code unit.
    #[inline]
    pub fn as_u8_bytes_with_nul(&self) -> &[u8] {
        unsafe {
            slice::from_raw_parts(
                self.inner.as_ptr() as *const u8,
                size_of_val(&self.inner),
            )
        }
    }

    /// Returns a mutable byte slice over this wide string, excluding the terminating NUL.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the modified bytes remain a valid wide string
    /// and do not introduce interior NULs.
    #[inline]
    pub unsafe fn as_u8_bytes_mut(&mut self) -> &mut [u8] {
        unsafe {
            slice::from_raw_parts_mut(
                self.inner.as_mut_ptr() as *mut u8,
                (self.inner.len() - 1) * size_of::<u16>(),
            )
        }
    }

    /// Returns a mutable byte view of the wide string, including the terminating NUL.
    ///
    /// # Safety
    ///
    /// The caller must preserve a valid, NUL-terminated wide string without interior NULs.
    #[inline]
    pub unsafe fn as_u8_bytes_with_nul_mut(&mut self) -> &mut [u8] {
        unsafe {
            slice::from_raw_parts_mut(
                self.inner.as_mut_ptr() as *mut u8,
                size_of_val(&self.inner),
            )
        }
    }

    /// Returns the length of bytes excluding the terminating NUL.
    #[inline]
    pub fn len(&self) -> usize { self.as_bytes().len() }

    /// Returns the length of bytes including the terminating NUL.
    #[inline]
    pub fn len_with_nul(&self) -> usize { self.as_bytes_with_nul().len() }

    /// Validates that `bytes` is a valid, NUL-terminated wide string without interior NULs.
    #[inline]
    pub(crate) fn check_nul_encoding(bytes: &[u16]) -> ConvertResult<()> {
        check_interior_nul_u16(bytes)?;
        check_nul_terminated_u16(bytes)?;
        validate_wide(bytes)?;

        Ok(())
    }

    /// Constructs a borrowed [`WStr`] from wide-character data.
    ///
    /// The input must be a valid, NUL-terminated wide string without interior NULs.
    pub fn from_bytes_with_nul(bytes: &[u16]) -> ConvertResult<&Self> {
        Self::check_nul_encoding(bytes)?;

        unsafe { Ok(Self::from_bytes_with_nul_unchecked(bytes)) }
    }

    /// Constructs a mutable borrowed [`WStr`] from wide-character data.
    ///
    /// The input must be a valid, NUL-terminated wide string without interior NULs.
    pub fn from_bytes_with_nul_mut(
        bytes: &mut [u16],
    ) -> ConvertResult<&mut Self> {
        Self::check_nul_encoding(bytes)?;

        unsafe { Ok(Self::from_bytes_with_nul_unchecked_mut(bytes)) }
    }

    /// Constructs a borrowed [`WStr`] from wide-character data, truncated at the first NUL.
    ///
    /// Returns an error if no NUL is present or if the truncated data is not a valid wide string.
    pub fn from_bytes_until_nul(bytes: &[u16]) -> ConvertResult<&Self> {
        let Some(pos) = find_nul_u16(bytes) else {
            return Err(StringFormatError::NotNulTerminated.into());
        };
        let bytes = &bytes[..pos + 1];
        validate_wide(bytes)?;

        unsafe { Ok(Self::from_bytes_with_nul_unchecked(bytes)) }
    }

    /// Constructs a mutable borrowed [`WStr`] from wide-character data, truncated at the first NUL.
    ///
    /// Returns an error if no NUL is present or if the truncated data is not a valid wide string.
    pub fn from_bytes_until_nul_mut(
        bytes: &mut [u16],
    ) -> ConvertResult<&mut Self> {
        let Some(pos) = find_nul_u16(bytes) else {
            return Err(StringFormatError::NotNulTerminated.into());
        };
        let bytes = &mut bytes[..pos + 1];
        validate_wide(bytes)?;

        unsafe { Ok(Self::from_bytes_with_nul_unchecked_mut(bytes)) }
    }

    /// Reinterprets `bytes` as a [`WStr`] without validation.
    ///
    /// # Safety
    ///
    /// `bytes` must be a valid, NUL-terminated wide string without interior
    /// NULs.
    #[inline]
    pub unsafe fn from_bytes_with_nul_unchecked(bytes: &[u16]) -> &Self {
        unsafe { &*(bytes as *const [u16] as *const Self) }
    }

    /// Reinterprets `bytes` as a mutable [`WStr`] without validation.
    ///
    /// # Safety
    ///
    /// `bytes` must be a valid, NUL-terminated wide string without interior NULs.
    #[inline]
    pub unsafe fn from_bytes_with_nul_unchecked_mut(
        bytes: &mut [u16],
    ) -> &mut Self {
        unsafe { &mut *(bytes as *mut [u16] as *mut Self) }
    }

    /// Constructs a borrowed [`WStr`] from a raw NUL-terminated pointer.
    ///
    /// # Safety
    ///
    /// `ptr` must be non-null, properly aligned for `u16`, and readable through the first NUL.
    /// The scanned range must be contained in a single allocated object and must not exceed `isize::MAX` bytes.
    /// The pointed-to data before the first NUL must contain valid UTF-16 and must not contain interior NULs.
    /// The returned reference must not outlive the pointed-to allocation.
    ///
    /// # Panics
    ///
    /// Panics if the scanned length plus the terminator overflows `usize`.
    pub unsafe fn from_raw<'a>(ptr: *const u16) -> &'a Self {
        unsafe {
            Self::from_raw_with_nul(
                ptr,
                wcslen(ptr)
                    .checked_add(1)
                    .expect("length of string overflow"),
            )
        }
    }

    /// Constructs a mutable borrowed [`WStr`] from a raw NUL-terminated pointer.
    ///
    /// This function scans for the first NUL with `wcslen`.
    ///
    /// # Safety
    ///
    /// `ptr` must be non-null, properly aligned for `u16`, readable and writable through the first NUL,
    /// and contained in a single allocated object not exceeding `isize::MAX` bytes.
    /// The pointed-to data must be valid UTF-16 without interior NULs.
    /// The returned mutable reference must be exclusive for `'a`,
    /// and the allocation must remain valid for that lifetime.
    ///
    /// # Panics
    ///
    /// Panics if the scanned length plus the terminator overflows `usize`.
    pub unsafe fn from_raw_mut<'a>(ptr: *mut u16) -> &'a mut Self {
        unsafe {
            Self::from_raw_with_nul_mut(
                ptr,
                wcslen(ptr)
                    .checked_add(1)
                    .expect("length of string overflow"),
            )
        }
    }

    /// Reinterprets `ptr..ptr.add(len)` as a [`WStr`].
    ///
    /// `len` is the number of `u16` elements including the final NUL.
    ///
    /// # Safety
    ///
    /// `ptr` must be non-null, properly aligned for `u16`, and readable for `len` elements.
    /// `len` must be greater than zero, the range must be contained in a single allocated object,
    /// and the range must not exceed `isize::MAX` bytes.
    /// The final element must be NUL, the preceding data must be valid UTF-16 without interior NULs,
    /// and the returned reference must not outlive that range.
    #[inline]
    pub unsafe fn from_raw_with_nul<'a>(
        ptr: *const u16,
        len: usize,
    ) -> &'a Self {
        unsafe {
            let slice = slice::from_raw_parts(ptr, len);
            debug_assert_eq!(slice.last(), Some(&0));
            Self::from_bytes_with_nul_unchecked(slice)
        }
    }

    /// Reinterprets `ptr..ptr.add(len)` as a mutable [`WStr`].
    ///
    /// `len` is the number of `u16` elements including the final NUL.
    ///
    /// # Safety
    ///
    /// `ptr` must be non-null, properly aligned for `u16`, readable, and writable for `len` elements.
    /// `len` must be greater than zero, the range must be contained in a single allocated object,
    /// and the range must not exceed `isize::MAX` bytes.
    /// The final element must be NUL, the preceding data must be valid UTF-16 without interior NULs,
    /// and the returned reference must be exclusive for `'a`.
    #[inline]
    pub unsafe fn from_raw_with_nul_mut<'a>(
        ptr: *mut u16,
        len: usize,
    ) -> &'a mut Self {
        unsafe {
            let slice = slice::from_raw_parts_mut(ptr, len);
            debug_assert_eq!(slice.last(), Some(&0));
            Self::from_bytes_with_nul_unchecked_mut(slice)
        }
    }

    /// Converts into an [`OsString`].
    ///
    /// The terminating NUL is not included in the returned value.
    #[cfg(feature = "std")]
    pub fn to_os_string(&self) -> OsString {
        OsString::from_wide(self.as_bytes())
    }
}

#[cfg(feature = "std")]
impl ToWString for WStr {
    fn try_to_wstring(&self) -> ConvertResult<WString> {
        unsafe { Ok(WString::from_vec_with_nul_unchecked(&self.inner)) }
    }
}

#[cfg(feature = "std")]
impl<const CP: u32> ToAString<CP> for WStr {
    fn try_to_astring(&self) -> ConvertResult<AString<CP>> {
        let mb = wide_to_mb(CP, self.as_bytes_with_nul())?;
        // valid ANSI string
        unsafe { Ok(AString::from_vec_with_nul_unchecked(mb)) }
    }

    fn try_to_astring_lossy(&self) -> ConvertResult<AString<CP>> {
        let mb = wide_to_mb_lossy(CP, self.as_bytes_with_nul())?;
        unsafe { Ok(AString::from_vec_with_nul_unchecked(mb)) }
    }
}

#[cfg(feature = "std")]
impl ToDAString for WStr {
    fn try_to_dastring(&self, code_page: u32) -> ConvertResult<DAString> {
        let mb = wide_to_mb(code_page, self.as_bytes_with_nul())?;
        unsafe { Ok(DAString::from_vec_with_nul_unchecked(code_page, mb)) }
    }

    fn try_to_dastring_lossy(&self, code_page: u32) -> ConvertResult<DAString> {
        let mb = wide_to_mb_lossy(code_page, self.as_bytes_with_nul())?;
        unsafe { Ok(DAString::from_vec_with_nul_unchecked(code_page, mb)) }
    }
}

#[cfg(feature = "std")]
impl From<&WStr> for OsString {
    #[inline]
    fn from(value: &WStr) -> Self { value.to_os_string() }
}

impl PartialEq for WStr {
    fn eq(&self, other: &Self) -> bool { self.as_bytes() == other.as_bytes() }
}

impl Eq for WStr {}

#[cfg(feature = "std")]
impl PartialEq<WString> for WStr {
    fn eq(&self, other: &WString) -> bool {
        self.as_bytes() == other.as_bytes()
    }
}

impl PartialOrd for WStr {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for WStr {
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_bytes().cmp(other.as_bytes())
    }
}

impl AsRef<WStr> for WStr {
    #[inline]
    fn as_ref(&self) -> &WStr { self }
}

#[cfg(feature = "std")]
impl ToOwned for WStr {
    type Owned = WString;

    fn to_owned(&self) -> Self::Owned { self.to_wstring() }
}

#[cfg(feature = "std")]
impl __lib::borrow::Borrow<WStr> for WString {
    fn borrow(&self) -> &WStr { self }
}

impl hash::Hash for WStr {
    #[inline]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.as_bytes().hash(state);
    }
}

impl fmt::Debug for WStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        #[cfg(not(feature = "std"))]
        {
            fmt::Debug::fmt(&self.as_bytes_with_nul(), f)
        }
        #[cfg(feature = "std")]
        {
            fmt::Debug::fmt(&self.to_string(), f)
        }
    }
}

impl fmt::Display for WStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        #[cfg(not(feature = "std"))]
        {
            fmt::Debug::fmt(&self.as_bytes_with_nul(), f)
        }
        #[cfg(feature = "std")]
        {
            fmt::Display::fmt(
                &String::from_utf16(self.as_bytes())
                    .expect("Failed to convert to String"),
                f,
            )
        }
    }
}

/// Borrowed ANSI string using `CP_ACP`.
pub type ACPStr = AStr<CP_ACP>;

/// A borrowed NUL-terminated ANSI string for code page `CP`.
///
/// A valid `AStr<CP>` points to bytes accepted by Windows for code page `CP`,
/// followed by a terminating NUL byte. Safe constructors validate this.
/// Unsafe constructors and mutable byte accessors rely on the caller to preserve the invariant for the entire borrowed lifetime.
#[repr(transparent)]
pub struct AStr<const CP: u32> {
    inner: [u8],
}

impl<const CP: u32> AStr<CP> {
    /// Gets the code page.
    #[inline]
    pub fn code_page(&self) -> u32 { CP }

    /// Returns the length of bytes excluding the terminating NUL.
    #[inline]
    pub fn len(&self) -> usize { self.as_bytes().len() }

    /// Returns the length of bytes including the terminating NUL.
    #[inline]
    pub fn len_with_nul(&self) -> usize { self.as_bytes_with_nul().len() }

    /// Returns a raw `i8` pointer to the first byte.
    #[inline]
    pub fn as_ptr(&self) -> *const i8 { self.inner.as_ptr() as *const i8 }

    /// Returns a mutable raw `i8` pointer to the first byte.
    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut i8 {
        self.inner.as_mut_ptr() as *mut i8
    }

    /// Returns a raw `u8` pointer to the first byte.
    #[inline]
    pub fn as_u8_ptr(&self) -> *const u8 { self.inner.as_ptr() }

    /// Returns a mutable raw `u8` pointer to the first byte.
    #[inline]
    pub fn as_mut_u8_ptr(&mut self) -> *mut u8 { self.inner.as_mut_ptr() }

    /// Returns `true` if the string is empty.
    #[inline]
    pub fn is_empty(&self) -> bool { self.as_bytes().is_empty() }

    /// Returns the bytes excluding the terminating NUL.
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        let bytes = self.as_bytes_with_nul();
        &bytes[..bytes.len() - 1]
    }

    /// Returns the bytes including the terminating NUL.
    #[inline]
    pub fn as_bytes_with_nul(&self) -> &[u8] { &self.inner }

    /// Returns the mutable bytes excluding the terminating NUL.
    ///
    /// # Safety
    ///
    /// After mutation, the retained bytes must still be valid for code page `CP` and must not contain interior NULs.
    /// The final terminator is not included in the returned slice and remains present.
    #[inline]
    pub unsafe fn as_bytes_mut(&mut self) -> &mut [u8] {
        let bytes = unsafe { self.as_bytes_with_nul_mut() };
        let len = bytes.len();
        &mut bytes[..len - 1]
    }

    /// Returns the mutable bytes including the terminating NUL.
    ///
    /// # Safety
    ///
    /// After mutation, the buffer must be non-empty, must end with NUL,
    /// must be valid for code page `CP`, and must not contain interior NULs.
    #[inline]
    pub unsafe fn as_bytes_with_nul_mut(&mut self) -> &mut [u8] {
        &mut self.inner
    }

    /// Converts this borrowed ANSI string to a UTF-8 [`String`].
    ///
    /// This method returns a [`ConvertError`] if Windows rejects the code page or byte sequence.
    /// Values produced by unsafe constructors are not revalidated before conversion.
    #[cfg(feature = "std")]
    pub fn try_to_string(&self) -> ConvertResult<String> {
        // ANSI -> Wide -> UTF-8
        Ok(self.try_to_wstring()?.to_string())
    }

    /// Validates that `bytes` is valid for code page `CP`, NUL-terminated, and without interior NULs.
    #[inline]
    pub(crate) fn check_nul_encoding(bytes: &[u8]) -> ConvertResult<()> {
        check_interior_nul_u8(bytes)?;
        check_nul_terminated_u8(bytes)?;
        validate_mb(CP, bytes)?;

        Ok(())
    }

    /// Constructs a borrowed [`AStr`] from bytes in code page `CP`.
    ///
    /// The input must be valid for code page `CP`, NUL-terminated, and without interior NULs.
    pub fn from_bytes_with_nul(bytes: &[u8]) -> ConvertResult<&Self> {
        Self::check_nul_encoding(bytes)?;

        unsafe { Ok(Self::from_bytes_with_nul_unchecked(bytes)) }
    }

    /// Constructs a mutable borrowed [`AStr`] from bytes in code page `CP`.
    ///
    /// The input must be valid for code page `CP`, NUL-terminated, and without interior NULs.
    pub fn from_bytes_with_nul_mut(
        bytes: &mut [u8],
    ) -> ConvertResult<&mut Self> {
        Self::check_nul_encoding(bytes)?;

        unsafe { Ok(Self::from_bytes_with_nul_unchecked_mut(bytes)) }
    }

    /// Constructs a borrowed [`AStr`] from bytes in code page `CP`, truncated at the first NUL.
    ///
    /// Returns an error if no NUL is present
    /// or if the truncated data is not valid for code page `CP`.
    pub fn from_bytes_until_nul(bytes: &[u8]) -> ConvertResult<&Self> {
        let Some(pos) = find_nul_u8(bytes) else {
            return Err(StringFormatError::NotNulTerminated.into());
        };
        let bytes = &bytes[..pos + 1];
        validate_mb(CP, bytes)?;

        unsafe { Ok(Self::from_bytes_with_nul_unchecked(bytes)) }
    }

    /// Constructs a mutable borrowed [`AStr`] from bytes in code page `CP`, truncated at the first NUL.
    ///
    /// Returns an error if no NUL is present or if the truncated data is not valid for code page `CP`.
    pub fn from_bytes_until_nul_mut(
        bytes: &mut [u8],
    ) -> ConvertResult<&mut Self> {
        let Some(pos) = find_nul_u8(bytes) else {
            return Err(StringFormatError::NotNulTerminated.into());
        };
        let bytes = &mut bytes[..pos + 1];
        validate_mb(CP, bytes)?;

        unsafe { Ok(Self::from_bytes_with_nul_unchecked_mut(bytes)) }
    }

    /// Reinterprets `bytes` as an [`AStr`] without validation.
    ///
    /// # Safety
    ///
    /// `bytes` must be non-empty, must end with NUL, must be valid for code page
    /// `CP`, and must not contain interior NULs. The returned reference inherits
    /// the lifetime and aliasing constraints of `bytes`.
    #[inline]
    pub unsafe fn from_bytes_with_nul_unchecked(bytes: &[u8]) -> &Self {
        unsafe { &*(bytes as *const [u8] as *const Self) }
    }

    /// Reinterprets `bytes` as a mutable [`AStr`] without validation.
    ///
    /// # Safety
    ///
    /// `bytes` must be non-empty, must end with NUL, must be valid for code page
    /// `CP`, and must not contain interior NULs. The returned mutable reference
    /// must be the only active reference to that range for its lifetime.
    #[inline]
    pub unsafe fn from_bytes_with_nul_unchecked_mut(
        bytes: &mut [u8],
    ) -> &mut Self {
        unsafe { &mut *(bytes as *mut [u8] as *mut Self) }
    }

    /// Constructs a borrowed [`AStr`] from a raw NUL-terminated pointer.
    ///
    /// # Safety
    ///
    /// `ptr` must be non-null and readable through the first NUL.
    /// The scanned range must be contained in a single allocated object and must not exceed `isize::MAX` bytes.
    /// The pointed-to data must be valid for code page `CP` without interior NULs.
    /// The returned reference must not outlive the
    /// pointed-to allocation.
    ///
    /// # Panics
    ///
    /// Panics if the scanned length plus the terminator overflows `usize`.
    pub unsafe fn from_raw<'a>(ptr: *const u8) -> &'a Self {
        unsafe {
            Self::from_raw_with_nul(
                ptr,
                strlen(ptr)
                    .checked_add(1)
                    .expect("length of string overflow"),
            )
        }
    }

    /// Constructs a mutable borrowed [`AStr`] from a raw NUL-terminated pointer.
    ///
    /// # Safety
    ///
    /// `ptr` must be non-null, readable, and writable through the first NUL. The
    /// scanned range must be contained in a single allocated object and must not
    /// exceed `isize::MAX` bytes. The pointed-to data must be valid for code page
    /// `CP` without interior NULs. The returned reference must be exclusive for
    /// `'a`, and the allocation must remain valid for that lifetime.
    ///
    /// # Panics
    ///
    /// Panics if the scanned length plus the terminator overflows `usize`.
    pub unsafe fn from_raw_mut<'a>(ptr: *mut u8) -> &'a mut Self {
        unsafe {
            Self::from_raw_with_nul_mut(
                ptr,
                strlen(ptr)
                    .checked_add(1)
                    .expect("length of string overflow"),
            )
        }
    }

    /// Reinterprets `ptr..ptr.add(len)` as [`AStr`].
    ///
    /// `len` is the number of bytes including the final NUL.
    ///
    /// # Safety
    ///
    /// `ptr` must be non-null and readable for `len` bytes. `len` must be
    /// greater than zero, the range must be contained in a single allocated
    /// object, and the range must not exceed `isize::MAX` bytes. The final byte
    /// must be NUL, the preceding bytes must be valid for code page `CP` without
    /// interior NULs, and the returned reference must not outlive that range.
    #[inline]
    pub unsafe fn from_raw_with_nul<'a>(
        ptr: *const u8,
        len: usize,
    ) -> &'a Self {
        unsafe {
            let slice = slice::from_raw_parts(ptr, len);
            Self::from_bytes_with_nul_unchecked(slice)
        }
    }

    /// Reinterprets `ptr..ptr.add(len)` as a mutable [`AStr`].
    ///
    /// `len` is the number of bytes including the final NUL.
    ///
    /// # Safety
    ///
    /// `ptr` must be non-null, readable, and writable for `len` bytes.
    /// `len` must be greater than zero, the range must be contained in a single allocated object,
    /// and the range must not exceed `isize::MAX` bytes.
    /// The final byte must be NUL,
    /// the preceding bytes must be valid for code page `CP` without interior NULs, and the returned reference must be exclusive for `'a`.
    #[inline]
    pub unsafe fn from_raw_with_nul_mut<'a>(
        ptr: *mut u8,
        len: usize,
    ) -> &'a mut Self {
        unsafe {
            let slice = slice::from_raw_parts_mut(ptr, len);
            Self::from_bytes_with_nul_unchecked_mut(slice)
        }
    }
}

#[cfg(feature = "std")]
impl<const CP: u32> ToWString for AStr<CP> {
    fn try_to_wstring(&self) -> ConvertResult<WString> {
        let wc = mb_to_wide(CP, self.as_bytes_with_nul())?;
        unsafe { Ok(WString::_new(wc)) }
    }
}

#[cfg(feature = "std")]
impl<const CP: u32, const CP2: u32> ToAString<CP2> for AStr<CP> {
    fn try_to_astring(&self) -> ConvertResult<AString<CP2>> {
        let v = mb_to_mb(self.code_page(), CP2, self.as_bytes_with_nul())?;
        unsafe { Ok(AString::from_vec_with_nul_unchecked(v)) }
    }

    fn try_to_astring_lossy(&self) -> ConvertResult<AString<CP2>> {
        let v =
            mb_to_mb_lossy(self.code_page(), CP2, self.as_bytes_with_nul())?;
        unsafe { Ok(AString::from_vec_with_nul_unchecked(v)) }
    }
}

#[cfg(feature = "std")]
impl<const CP: u32> ToDAString for AStr<CP> {
    fn try_to_dastring(&self, code_page: u32) -> ConvertResult<DAString> {
        let v =
            mb_to_mb(self.code_page(), code_page, self.as_bytes_with_nul())?;
        unsafe { Ok(DAString::from_vec_with_nul_unchecked(code_page, v)) }
    }

    fn try_to_dastring_lossy(&self, code_page: u32) -> ConvertResult<DAString> {
        let v = mb_to_mb_lossy(
            self.code_page(),
            code_page,
            self.as_bytes_with_nul(),
        )?;
        unsafe { Ok(DAString::from_vec_with_nul_unchecked(code_page, v)) }
    }
}

impl<const CP: u32> PartialEq for AStr<CP> {
    fn eq(&self, other: &Self) -> bool { self.as_bytes() == other.as_bytes() }
}

impl<const CP: u32> Eq for AStr<CP> {}

#[cfg(feature = "std")]
impl<const CP: u32> PartialEq<AString<CP>> for AStr<CP> {
    #[inline]
    fn eq(&self, other: &AString<CP>) -> bool {
        self.as_bytes() == other.as_bytes()
    }
}

#[cfg(feature = "std")]
impl<const CP: u32> PartialEq<&AString<CP>> for AStr<CP> {
    #[inline]
    fn eq(&self, other: &&AString<CP>) -> bool {
        self.as_bytes() == other.as_bytes()
    }
}

impl<const CP: u32> PartialOrd for AStr<CP> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<const CP: u32> Ord for AStr<CP> {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_bytes().cmp(other.as_bytes())
    }
}

#[cfg(feature = "std")]
impl<const CP: u32> PartialOrd<AString<CP>> for AStr<CP> {
    #[inline]
    fn partial_cmp(&self, other: &AString<CP>) -> Option<Ordering> {
        Some(self.as_bytes().cmp(other.as_bytes()))
    }
}

#[cfg(feature = "std")]
impl<const CP: u32> PartialOrd<&AString<CP>> for AStr<CP> {
    #[inline]
    fn partial_cmp(&self, other: &&AString<CP>) -> Option<Ordering> {
        self.partial_cmp(*other)
    }
}

impl<const CP: u32> AsRef<AStr<CP>> for AStr<CP> {
    #[inline]
    fn as_ref(&self) -> &AStr<CP> { self }
}

#[cfg(feature = "std")]
impl<const CP: u32> ToOwned for AStr<CP> {
    type Owned = AString<CP>;

    fn to_owned(&self) -> Self::Owned { self.to_astring() }
}

#[cfg(feature = "std")]
impl<const CP: u32> __lib::borrow::Borrow<AStr<CP>> for AString<CP> {
    fn borrow(&self) -> &AStr<CP> { self }
}

impl<const CP: u32> hash::Hash for AStr<CP> {
    #[inline]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        CP.hash(state);
        self.as_bytes().hash(state);
    }
}

impl<const CP: u32> fmt::Debug for AStr<CP> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        #[cfg(not(feature = "std"))]
        {
            fmt::Debug::fmt(&self.as_bytes_with_nul(), f)
        }
        #[cfg(feature = "std")]
        {
            match self.try_to_string() {
                Ok(s) => fmt::Debug::fmt(&s, f),
                Err(_) => Err(fmt::Error),
            }
        }
    }
}

impl<const CP: u32> fmt::Display for AStr<CP> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        #[cfg(not(feature = "std"))]
        {
            fmt::Debug::fmt(&self.as_bytes_with_nul(), f)
        }
        #[cfg(feature = "std")]
        {
            match self.try_to_string() {
                Ok(s) => fmt::Display::fmt(&s, f),
                Err(_) => Err(fmt::Error),
            }
        }
    }
}

/// A borrowed NUL-terminated ANSI string with a runtime-selected code page.
///
/// A valid `DAStr` points to bytes accepted by Windows for its stored code page, followed by a terminating NUL byte.
/// Safe constructors validate this.
/// Unsafe constructors rely on the caller to preserve the invariant for the entire borrowed lifetime.
#[derive(Clone, Copy)]
pub struct DAStr<'a> {
    inner: &'a [u8],
    code_page: u32,
}

impl<'a> DAStr<'a> {
    /// Gets the code page.
    #[inline]
    pub fn code_page(&self) -> u32 { self.code_page }

    /// Returns the length of bytes excluding the terminating NUL.
    #[inline]
    pub fn len(&self) -> usize { self.as_bytes().len() }

    /// Returns the length of bytes including the terminating NUL.
    #[inline]
    pub fn len_with_nul(&self) -> usize { self.as_bytes_with_nul().len() }

    /// Returns a raw `i8` pointer to the first byte.
    #[inline]
    pub fn as_ptr(&self) -> *const i8 { self.inner.as_ptr() as *const i8 }

    /// Returns a raw `u8` pointer to the first byte.
    #[inline]
    pub fn as_u8_ptr(&self) -> *const u8 { self.inner.as_ptr() }

    /// Returns `true` if the string is empty.
    #[inline]
    pub fn is_empty(&self) -> bool { self.as_bytes().is_empty() }

    /// Returns the bytes excluding the terminating NUL.
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        let bytes = self.as_bytes_with_nul();
        &bytes[..bytes.len() - 1]
    }

    /// Returns the bytes including the terminating NUL.
    #[inline]
    pub fn as_bytes_with_nul(&self) -> &[u8] { self.inner }

    /// Converts this borrowed ANSI string to a UTF-8 [`String`].
    ///
    /// This method returns a [`ConvertError`] if Windows rejects the code page or byte sequence.
    /// Values produced by unsafe constructors are not revalidated before conversion.
    #[cfg(feature = "std")]
    pub fn try_to_string(&self) -> ConvertResult<String> {
        // ANSI -> Wide -> UTF-8
        Ok(self.try_to_wstring()?.to_string())
    }

    /// Validates that `bytes` is valid for `code_page`, NUL-terminated, and without interior NULs.
    #[inline]
    pub(crate) fn check_nul_encoding(
        code_page: u32,
        bytes: &[u8],
    ) -> ConvertResult<()> {
        check_interior_nul_u8(bytes)?;
        check_nul_terminated_u8(bytes)?;
        validate_mb(code_page, bytes)?;

        Ok(())
    }

    /// Constructs a borrowed [`DAStr`] from bytes in `code_page`.
    ///
    /// The input must be valid for `code_page`, NUL-terminated, and without interior NULs.
    pub fn from_bytes_with_nul(
        code_page: u32,
        bytes: &'a [u8],
    ) -> ConvertResult<Self> {
        Self::check_nul_encoding(code_page, bytes)?;

        unsafe { Ok(Self::from_bytes_with_nul_unchecked(code_page, bytes)) }
    }

    /// Constructs a borrowed [`DAStr`] from bytes in `code_page`, truncated at the first NUL.
    ///
    /// Returns an error if no NUL is present
    /// or if the truncated data is not valid for `code_page`.
    pub fn from_bytes_until_nul(
        code_page: u32,
        bytes: &'a [u8],
    ) -> ConvertResult<Self> {
        let Some(pos) = find_nul_u8(bytes) else {
            return Err(StringFormatError::NotNulTerminated.into());
        };
        let bytes = &bytes[..pos + 1];
        validate_mb(code_page, bytes)?;

        unsafe { Ok(Self::from_bytes_with_nul_unchecked(code_page, bytes)) }
    }

    /// Reinterprets `bytes` as a [`DAStr`] without validation.
    ///
    /// # Safety
    ///
    /// `bytes` must be non-empty, must end with NUL, must be valid for `code_page`, and must not contain interior NULs.
    /// The returned value must not outlive `bytes`.
    #[inline]
    pub unsafe fn from_bytes_with_nul_unchecked(
        code_page: u32,
        bytes: &'a [u8],
    ) -> Self {
        Self {
            inner: bytes,
            code_page,
        }
    }

    /// Constructs a borrowed [`DAStr`] from a raw NUL-terminated pointer.
    ///
    /// This function scans for the first NUL with `strlen`. It can panic if the
    /// scanned length plus the terminator overflows `usize`.
    ///
    /// # Safety
    ///
    /// `ptr` must be non-null and readable through the first NUL.
    /// The scanned range must be contained in a single allocated object and must not exceed `isize::MAX` bytes.
    /// The pointed-to data must be valid for `code_page` without interior NULs.
    /// The returned value must not outlive the pointed-to allocation.
    ///
    /// # Panics
    ///
    /// Panics if the scanned length plus the terminator overflows `usize`.
    pub unsafe fn from_raw(code_page: u32, ptr: *const u8) -> Self {
        unsafe {
            Self::from_raw_with_nul(
                code_page,
                ptr,
                strlen(ptr)
                    .checked_add(1)
                    .expect("length of string overflow"),
            )
        }
    }

    /// Reinterprets `ptr..ptr.add(len)` as a [`DAStr`].
    ///
    /// `len` is the number of bytes including the final NUL.
    ///
    /// # Safety
    ///
    /// `ptr` must be non-null and readable for `len` bytes. `len` must be greater than zero,
    /// the range must be contained in a single allocated object,
    /// and the range must not exceed `isize::MAX` bytes.
    /// The final byte must be NUL, the preceding bytes must be valid for `code_page` without interior NULs,
    /// and the returned value must not outlive that range.
    #[inline]
    pub unsafe fn from_raw_with_nul(
        code_page: u32,
        ptr: *const u8,
        len: usize,
    ) -> Self {
        unsafe {
            let slice = slice::from_raw_parts(ptr, len);
            Self::from_bytes_with_nul_unchecked(code_page, slice)
        }
    }
}

impl<'a> PartialEq for DAStr<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.code_page() == other.code_page()
            && self.as_bytes() == other.as_bytes()
    }
}

impl<'a> Eq for DAStr<'a> {}

impl<'a> PartialOrd for DAStr<'a> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> Ord for DAStr<'a> {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.code_page()
            .cmp(&other.code_page())
            .then_with(|| self.as_bytes().cmp(other.as_bytes()))
    }
}

#[cfg(feature = "std")]
impl<'a, const CP: u32> PartialEq<DAStr<'a>> for AStr<CP> {
    fn eq(&self, other: &DAStr) -> bool {
        self.code_page() == other.code_page()
            && self.as_bytes() == other.as_bytes()
    }
}

#[cfg(feature = "std")]
impl<'a, const CP: u32> PartialEq<AStr<CP>> for DAStr<'a> {
    fn eq(&self, other: &AStr<CP>) -> bool {
        self.code_page() == other.code_page()
            && self.as_bytes() == other.as_bytes()
    }
}

#[cfg(feature = "std")]
impl<const CP: u32> PartialEq<DAString> for AStr<CP> {
    fn eq(&self, other: &DAString) -> bool {
        self.code_page() == other.code_page()
            && self.as_bytes() == other.as_bytes()
    }
}

#[cfg(feature = "std")]
impl<'a, const CP: u32> PartialEq<AString<CP>> for DAStr<'a> {
    fn eq(&self, other: &AString<CP>) -> bool {
        self.code_page() == other.code_page()
            && self.as_bytes() == other.as_bytes()
    }
}

#[cfg(feature = "std")]
impl<'a> PartialEq<DAString> for DAStr<'a> {
    fn eq(&self, other: &DAString) -> bool {
        self.code_page() == other.code_page()
            && self.as_bytes() == other.as_bytes()
    }
}

#[cfg(feature = "std")]
impl<'a> PartialOrd<DAString> for DAStr<'a> {
    #[inline]
    fn partial_cmp(&self, other: &DAString) -> Option<Ordering> {
        Some(
            self.code_page()
                .cmp(&other.code_page())
                .then_with(|| self.as_bytes().cmp(other.as_bytes())),
        )
    }
}

impl<'a> hash::Hash for DAStr<'a> {
    #[inline]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.code_page().hash(state);
        self.as_bytes().hash(state);
    }
}

impl<'a> AsRef<DAStr<'a>> for DAStr<'a> {
    #[inline]
    fn as_ref(&self) -> &DAStr<'a> { self }
}

#[cfg(feature = "std")]
impl<'a> ToWString for DAStr<'a> {
    fn try_to_wstring(&self) -> ConvertResult<WString> {
        let wc = mb_to_wide(self.code_page, self.as_bytes_with_nul())?;
        unsafe { Ok(WString::_new(wc)) }
    }
}

#[cfg(feature = "std")]
impl<'a, const CP: u32> ToAString<CP> for DAStr<'a> {
    fn try_to_astring(&self) -> ConvertResult<AString<CP>> {
        let v = mb_to_mb(self.code_page(), CP, self.as_bytes_with_nul())?;
        unsafe { Ok(AString::from_vec_with_nul_unchecked(v)) }
    }

    fn try_to_astring_lossy(&self) -> ConvertResult<AString<CP>> {
        let v = mb_to_mb_lossy(self.code_page(), CP, self.as_bytes_with_nul())?;
        unsafe { Ok(AString::from_vec_with_nul_unchecked(v)) }
    }
}

#[cfg(feature = "std")]
impl<'a> ToDAString for DAStr<'a> {
    fn try_to_dastring(&self, code_page: u32) -> ConvertResult<DAString> {
        let v =
            mb_to_mb(self.code_page(), code_page, self.as_bytes_with_nul())?;
        unsafe { Ok(DAString::from_vec_with_nul_unchecked(code_page, v)) }
    }

    fn try_to_dastring_lossy(&self, code_page: u32) -> ConvertResult<DAString> {
        let v = mb_to_mb_lossy(
            self.code_page(),
            code_page,
            self.as_bytes_with_nul(),
        )?;
        unsafe { Ok(DAString::from_vec_with_nul_unchecked(code_page, v)) }
    }
}

impl<'a> fmt::Debug for DAStr<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        #[cfg(not(feature = "std"))]
        {
            fmt::Debug::fmt(&self.as_bytes_with_nul(), f)
        }
        #[cfg(feature = "std")]
        {
            match self.try_to_string() {
                Ok(s) => fmt::Debug::fmt(&s, f),
                Err(_) => Err(fmt::Error),
            }
        }
    }
}

impl<'a> fmt::Display for DAStr<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        #[cfg(not(feature = "std"))]
        {
            fmt::Debug::fmt(&self.as_bytes_with_nul(), f)
        }
        #[cfg(feature = "std")]
        {
            match self.try_to_string() {
                Ok(s) => fmt::Display::fmt(&s, f),
                Err(_) => Err(fmt::Error),
            }
        }
    }
}
