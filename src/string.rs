// Copyright takubokudori.
// This source code is licensed under the MIT or Apache-2.0 license.
use crate::{
    __lib::{cmp::Ordering, convert::TryFrom, fmt, hash, ops, slice, str},
    convert::{validate_mb, validate_wide},
    traits::{ToAString, ToDAString, ToWString},
    *,
};
use std::{
    ffi::{OsStr, OsString},
    os::windows::ffi::{OsStrExt, OsStringExt},
};

/// Concatenates two slices into a newly allocated vector.
fn concat_slice<T: Clone>(x: &[T], y: &[T]) -> Vec<T> {
    let mut inner = Vec::with_capacity(x.len() + y.len());
    inner.extend_from_slice(x);
    inner.extend_from_slice(y);
    inner
}

/// An owned NUL-terminated wide string.
///
/// A valid `WString` contains valid UTF-16 code units followed by exactly one
/// terminating NUL in the retained string. Safe constructors reject interior
/// NULs and invalid UTF-16. Unsafe constructors rely on the caller to preserve
/// those invariants.
#[derive(Clone, PartialOrd, PartialEq, Eq, Ord)]
pub struct WString {
    inner: Vec<u16>,
}

impl WString {
    /// Returns the `u16` elements of the wide string, excluding the terminating NUL.
    #[inline]
    pub fn as_bytes(&self) -> &[u16] {
        &self.as_bytes_with_nul()[..self.inner.len() - 1]
    }

    /// Returns a mutable slice of the `u16` elements, excluding the terminating NUL.
    ///
    /// # Safety
    ///
    /// The caller must preserve a valid wide string without interior NULs.
    pub unsafe fn as_bytes_mut(&mut self) -> &mut [u16] {
        let bytes = unsafe { self.as_bytes_with_nul_mut() };
        let len = bytes.len();
        &mut bytes[..len - 1]
    }

    /// Returns the `u16` elements of the wide string, including the terminating NUL.
    #[inline]
    pub fn as_bytes_with_nul(&self) -> &[u16] { &self.inner }

    /// Returns a mutable slice of the `u16` elements, including the terminating NUL.
    ///
    /// # Safety
    ///
    /// The caller must preserve a valid, NUL-terminated wide string without interior NULs.
    #[inline]
    pub unsafe fn as_bytes_with_nul_mut(&mut self) -> &mut [u16] {
        &mut self.inner
    }

    /// Returns a byte view of the wide string, excluding the terminating NUL.
    ///
    /// The returned bytes are the in-memory representation of the stored `u16`
    /// code units. On Windows this corresponds to UTF-16LE bytes.
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
    /// The returned bytes are the in-memory representation of the stored `u16` code units,
    /// including the terminating NUL code unit.
    #[inline]
    pub fn as_u8_bytes_with_nul(&self) -> &[u8] {
        unsafe {
            slice::from_raw_parts(
                self.inner.as_ptr() as *const u8,
                self.inner.len() * size_of::<u16>(),
            )
        }
    }

    /// Returns the length of bytes excluding the terminating NUL.
    #[inline]
    pub fn len(&self) -> usize { self.as_bytes().len() }

    /// Returns the length of bytes including the terminating NUL.
    #[inline]
    pub fn len_with_nul(&self) -> usize { self.as_bytes_with_nul().len() }

    /// Returns the string as a borrowed [`WStr`].
    #[inline]
    pub fn as_wstr(&self) -> &WStr { self }

    /// Returns the string as a mutable borrowed [`WStr`].
    #[inline]
    pub fn as_mut_wstr(&mut self) -> &mut WStr {
        unsafe {
            WStr::from_bytes_with_nul_unchecked_mut(
                self.as_bytes_with_nul_mut(),
            )
        }
    }

    /// Returns `true` if the string is empty.
    #[inline]
    pub fn is_empty(&self) -> bool { self.as_bytes().is_empty() }

    /// Constructs a [`WString`] from wide-character data.
    ///
    /// The input must be a valid, wide string without interior NULs.
    /// If the input is not NUL-terminated, a terminating NUL is appended.
    pub fn new<T: Into<Vec<u16>>>(v: T) -> ConvertResult<Self> {
        let v = v.into();
        check_interior_nul_u16(&v)?;
        validate_wide(&v)?;

        unsafe { Ok(Self::from_vec_unchecked(v)) }
    }

    /// Constructs a [`WString`] from NUL-terminated wide string.
    ///
    /// The input must end with a NUL wide character, must not contain interior NULs, and must be valid UTF-16.
    pub fn from_vec_with_nul<T: Into<Vec<u16>>>(v: T) -> ConvertResult<Self> {
        let v = v.into();
        check_interior_nul_u16(&v)?;
        check_nul_terminated_u16(&v)?;
        validate_wide(&v)?;

        unsafe { Ok(Self::from_vec_with_nul_unchecked(v)) }
    }

    /// Constructs a [`WString`] from wide-character data, truncated at the first NUL.
    ///
    /// If the input contains no NUL, a terminating NUL is appended. The retained data must be valid UTF-16.
    pub fn from_vec_until_nul<T: Into<Vec<u16>>>(v: T) -> ConvertResult<Self> {
        let v = v.into();
        match find_nul_u16(&v) {
            None => {
                validate_wide(&v)?;

                let len = v.len();
                unsafe { Ok(Self::_new2(v, len)) }
            }
            Some(pos) => {
                if pos == v.len() - 1 {
                    validate_wide(&v)?;

                    unsafe { Ok(Self::_new2(v, pos)) }
                } else {
                    let v = &v[..pos + 1];

                    validate_wide(v)?;

                    unsafe { Ok(Self::from_vec_unchecked(v)) }
                }
            }
        }
    }

    /// Constructs a [`WString`] from wide-character data without validation.
    ///
    /// # Safety
    ///
    /// `v` must contain a valid wide string up to the first NUL,
    /// without interior NULs in the retained string.
    pub unsafe fn from_vec_unchecked<T: Into<Vec<u16>>>(v: T) -> Self {
        unsafe { Self::_new(v.into()) }
    }

    /// Constructs a [`WString`] from NUL-terminated wide-character data without validation.
    ///
    /// This function does not append a terminator. The supplied bytes must already contain the final NUL element.
    ///
    /// # Safety
    ///
    /// `v` must be a valid, NUL-terminated wide string without interior NULs.
    #[inline]
    pub unsafe fn from_vec_with_nul_unchecked<T: Into<Vec<u16>>>(v: T) -> Self {
        Self { inner: v.into() }
    }

    /// Normalizes `v` into the internal NUL-terminated representation without validating its encoding.
    ///
    /// # Safety
    ///
    /// `v` must contain a valid wide string up to the first NUL,
    /// without interior NULs in the retained string.
    #[inline]
    pub(crate) unsafe fn _new(v: Vec<u16>) -> Self {
        unsafe {
            let len = wcsnlen(v.as_ptr(), v.len());
            Self::_new2(v, len)
        }
    }

    #[inline]
    unsafe fn _new2(mut v: Vec<u16>, before_nul_pos: usize) -> Self {
        unsafe {
            if before_nul_pos == v.len() {
                // append NULL.
                v.push(0);
            }
            v.set_len(before_nul_pos.checked_add(1).expect("len + 1 overflow"));
            Self::from_vec_with_nul_unchecked(v)
        }
    }

    /// Converts UTF-8 text to a [`WString`].
    ///
    /// Returns an error if `s` contains an embedded NUL.
    pub fn from_utf8(s: impl AsRef<str>) -> ConvertResult<Self> {
        let s = s.as_ref();
        check_nul_in_str(s)?;
        let v: Vec<u16> = s.encode_utf16().chain(std::iter::once(0)).collect();
        unsafe { Ok(Self::from_vec_with_nul_unchecked(v)) }
    }

    /// Converts UTF-8 text to a [`WString`], replacing embedded NUL characters with `U+FFFD`.
    pub fn from_utf8_lossy(s: impl AsRef<str>) -> Self {
        let s = s.as_ref();

        let v: Vec<u16> = s
            .encode_utf16()
            .map(|c| if c == 0 { 0xfffd } else { c })
            .chain(std::iter::once(0))
            .collect();

        unsafe { Self::from_vec_with_nul_unchecked(v) }
    }

    /// Copies a [`WString`] from a raw NUL-terminated pointer.
    ///
    /// # Safety
    ///
    /// `ptr` must be non-null, properly aligned for `u16`, and readable through the first NUL.
    /// The scanned range must be contained in a single allocated object and must not exceed `isize::MAX` bytes.
    /// The pointed-to data before the first NUL must be valid UTF-16 and must not contain interior NULs.
    pub unsafe fn clone_from_raw(ptr: *const u16) -> Self {
        unsafe { Self::clone_from_raw_s(ptr, wcslen(ptr)) }
    }

    /// Copies a [`WString`] from a raw pointer and explicit copied length without scanning for a terminator.
    ///
    /// `len` is the number of `u16` elements copied from `ptr`.
    /// This function does not read `ptr.add(len)`.
    /// If the copied range does not already end in NUL, a terminating NUL is appended to the new `WString`.
    ///
    /// # Safety
    ///
    /// `ptr` must be non-null, properly aligned for `u16`, and readable for `len` elements.
    /// The copied range must be contained in a single allocated object and must not exceed `isize::MAX` bytes.
    /// The copied data must be valid UTF-16 and must not contain NULs except possibly as the final copied element.
    #[inline]
    pub unsafe fn clone_from_raw_s(ptr: *const u16, len: usize) -> Self {
        unsafe {
            let mut inner = slice::from_raw_parts(ptr, len).to_vec();
            if Some(&0) != inner.last() {
                inner.push(0);
            }
            Self { inner }
        }
    }

    /// Consumes the string and returns the internal NUL-terminated buffer.
    pub fn into_bytes_with_nul(self) -> Vec<u16> { self.inner }

    /// Constructs a [`WString`] from a Windows [`OsStr`].
    ///
    /// The conversion is lossless when the input contains valid UTF-16 and no
    /// interior NUL code units.
    ///
    /// # Errors
    ///
    /// Returns [`ConvertError::InvalidUnicodeString`] if `value` contains an
    /// unpaired UTF-16 surrogate.
    ///
    /// Returns [`ConvertError::InvalidStringFormat`] if `value` contains NUL.
    pub fn from_os_str(value: impl AsRef<OsStr>) -> ConvertResult<Self> {
        let units: Vec<u16> = value.as_ref().encode_wide().collect();
        Self::new(units)
    }

    /// Constructs a [`WString`] from a Windows [`OsStr`] using replacement.
    ///
    /// Unpaired surrogates and interior NUL characters are replaced with
    /// `U+FFFD`.
    pub fn from_os_str_lossy(value: impl AsRef<OsStr>) -> Self {
        let value = value.as_ref().to_string_lossy();
        Self::from_utf8_lossy(value.as_ref())
    }

    /// Consumes this string and converts it into an [`OsString`].
    ///
    /// The terminating NUL is not included in the returned value.
    pub fn into_os_string(self) -> OsString {
        OsString::from_wide(self.as_bytes())
    }
}

impl ops::Deref for WString {
    type Target = WStr;

    fn deref(&self) -> &Self::Target {
        unsafe { WStr::from_bytes_with_nul_unchecked(self.as_bytes_with_nul()) }
    }
}

impl ops::Index<ops::RangeFull> for WString {
    type Output = WStr;

    #[inline]
    fn index(&self, _: ops::RangeFull) -> &Self::Output { self }
}

impl From<WString> for String {
    fn from(value: WString) -> Self { value.to_string() }
}

impl str::FromStr for WString {
    type Err = ConvertError;

    /// Converts `&str` into a [`WString`].
    fn from_str(s: &str) -> Result<Self, Self::Err> { Self::from_utf8(s) }
}

impl TryFrom<&str> for WString {
    type Error = ConvertError;

    #[inline]
    fn try_from(x: &str) -> Result<Self, Self::Error> { Self::from_utf8(x) }
}

impl TryFrom<String> for WString {
    type Error = ConvertError;

    #[inline]
    fn try_from(x: String) -> Result<Self, Self::Error> {
        Self::try_from(x.as_str())
    }
}

impl TryFrom<&String> for WString {
    type Error = ConvertError;

    #[inline]
    fn try_from(x: &String) -> Result<Self, Self::Error> {
        Self::try_from(x.as_str())
    }
}

impl AsRef<WStr> for WString {
    #[inline]
    fn as_ref(&self) -> &WStr { self }
}

impl From<&WStr> for WString {
    fn from(x: &WStr) -> Self {
        unsafe {
            Self::from_vec_with_nul_unchecked(x.as_bytes_with_nul().to_vec())
        }
    }
}

impl PartialEq<WStr> for WString {
    fn eq(&self, other: &WStr) -> bool { self.as_bytes() == other.as_bytes() }
}

impl PartialEq<&WStr> for WString {
    fn eq(&self, other: &&WStr) -> bool { self.as_bytes() == other.as_bytes() }
}

impl hash::Hash for WString {
    #[inline]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.as_bytes().hash(state);
    }
}

impl<const CP: u32> TryFrom<&AStr<CP>> for WString {
    type Error = ConvertError;

    #[inline]
    fn try_from(x: &AStr<CP>) -> Result<Self, Self::Error> {
        x.try_to_wstring()
    }
}

impl<const CP: u32> TryFrom<AString<CP>> for WString {
    type Error = ConvertError;

    #[inline]
    fn try_from(x: AString<CP>) -> Result<Self, Self::Error> {
        Self::try_from(x.as_astr())
    }
}

impl<const CP: u32> TryFrom<&AString<CP>> for WString {
    type Error = ConvertError;

    #[inline]
    fn try_from(x: &AString<CP>) -> Result<Self, Self::Error> {
        Self::try_from(x.as_astr())
    }
}
impl TryFrom<&OsStr> for WString {
    type Error = ConvertError;

    #[inline]
    fn try_from(value: &OsStr) -> Result<Self, Self::Error> {
        Self::from_os_str(value)
    }
}

impl TryFrom<OsString> for WString {
    type Error = ConvertError;

    #[inline]
    fn try_from(value: OsString) -> Result<Self, Self::Error> {
        Self::from_os_str(value.as_os_str())
    }
}

impl TryFrom<&OsString> for WString {
    type Error = ConvertError;

    #[inline]
    fn try_from(value: &OsString) -> Result<Self, Self::Error> {
        Self::from_os_str(value.as_os_str())
    }
}

impl From<WString> for OsString {
    #[inline]
    fn from(value: WString) -> Self { value.into_os_string() }
}

impl From<&WString> for OsString {
    #[inline]
    fn from(value: &WString) -> Self { value.to_os_string() }
}

impl ops::Add<&WStr> for WString {
    type Output = Self;

    fn add(self, rhs: &WStr) -> Self::Output {
        let inner = concat_slice(self.as_bytes(), rhs.as_bytes_with_nul());
        unsafe { Self::from_vec_with_nul_unchecked(inner) }
    }
}

impl fmt::Debug for WString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        #[cfg(not(feature = "std"))]
        {
            fmt::Debug::fmt(&self.as_bytes_with_nul(), f)
        }
        #[cfg(feature = "std")]
        {
            fmt::Debug::fmt(&self.as_wstr(), f)
        }
    }
}

impl fmt::Display for WString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        #[cfg(not(feature = "std"))]
        {
            fmt::Debug::fmt(&self.as_bytes_with_nul(), f)
        }
        #[cfg(feature = "std")]
        {
            fmt::Display::fmt(&self.as_wstr(), f)
        }
    }
}

/// Owned ANSI string using `CP_ACP`.
pub type ACPString = AString<CP_ACP>;

/// An owned NUL-terminated ANSI string for code page `CP`.
///
/// A valid `AString<CP>` contains bytes accepted by Windows for code page `CP`,
/// followed by exactly one terminating NUL in the retained string.
/// Safe constructors reject interior NULs and invalid byte sequences.
/// Unsafe constructors rely on the caller to preserve those invariants.
#[derive(Clone)]
pub struct AString<const CP: u32> {
    inner: Vec<u8>,
}

impl<const CP: u32> AString<CP> {
    /// Gets the code page.
    #[inline]
    pub fn code_page(&self) -> u32 { CP }

    /// Returns the bytes excluding the terminating NUL.
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        &self.as_bytes_with_nul()[..self.inner.len() - 1]
    }

    /// Returns the mutable bytes excluding the terminating NUL.
    ///
    /// # Safety
    ///
    /// The caller must preserve a string valid for code page `CP` without
    /// interior NULs.
    pub unsafe fn as_bytes_mut(&mut self) -> &mut [u8] {
        let bytes = unsafe { self.as_bytes_with_nul_mut() };
        let len = bytes.len();
        &mut bytes[..len - 1]
    }

    /// Returns the bytes including the terminating NUL.
    #[inline]
    pub fn as_bytes_with_nul(&self) -> &[u8] { &self.inner }

    /// Returns the mutable bytes including the terminating NUL.
    ///
    /// # Safety
    ///
    /// The caller must preserve a valid, NUL-terminated string for code page
    /// `CP` without interior NULs.
    #[inline]
    pub unsafe fn as_bytes_with_nul_mut(&mut self) -> &mut [u8] {
        &mut self.inner
    }

    /// Returns the length of bytes excluding the terminating NUL.
    #[inline]
    pub fn len(&self) -> usize { self.as_bytes().len() }

    /// Returns the length of bytes including the terminating NUL.
    #[inline]
    pub fn len_with_nul(&self) -> usize { self.as_bytes_with_nul().len() }

    /// Returns the string as a borrowed [`AStr`].
    #[inline]
    pub fn as_astr(&self) -> &AStr<CP> { self }

    /// Returns the string as a mutable borrowed [`AStr`].
    #[inline]
    pub fn as_mut_astr(&mut self) -> &mut AStr<CP> {
        unsafe {
            AStr::from_bytes_with_nul_unchecked_mut(
                self.as_bytes_with_nul_mut(),
            )
        }
    }

    /// Returns `true` if the string is empty.
    #[inline]
    pub fn is_empty(&self) -> bool { self.as_bytes().is_empty() }

    /// Constructs an [`AString`] from bytes in code page `CP`.
    ///
    /// The input must be valid for code page `CP` and must not contain
    /// interior NULs. If it is not NUL-terminated, a terminating NUL is appended.
    pub fn new<T: Into<Vec<u8>>>(v: T) -> ConvertResult<Self> {
        let v = v.into();
        check_interior_nul_u8(&v)?;
        validate_mb(CP, &v)?;

        unsafe { Ok(Self::from_vec_unchecked(v)) }
    }

    /// Constructs an [`AString`] from NUL-terminated bytes in code page `CP`.
    ///
    /// The input must be valid for code page `CP`, must end with a NUL byte,
    /// and must not contain interior NULs.
    pub fn from_vec_with_nul<T: Into<Vec<u8>>>(v: T) -> ConvertResult<Self> {
        let v = v.into();
        check_interior_nul_u8(&v)?;
        check_nul_terminated_u8(&v)?;
        validate_mb(CP, &v)?;

        unsafe { Ok(Self::from_vec_with_nul_unchecked(v)) }
    }

    /// Constructs an [`AString`] from bytes in code page `CP`.
    ///
    /// If the input contains a NUL, the string is truncated at the first NUL.
    /// If the input contains no NUL, a terminating NUL is appended.
    pub fn from_vec_until_nul<T: Into<Vec<u8>>>(v: T) -> ConvertResult<Self> {
        let v = v.into();
        match find_nul_u8(&v) {
            None => {
                validate_mb(CP, &v)?;
                let len = v.len();

                unsafe { Ok(Self::_new(v, len)) }
            }
            Some(pos) => {
                if pos == v.len() - 1 {
                    validate_mb(CP, &v)?;

                    unsafe { Ok(Self::_new(v, pos)) }
                } else {
                    let v = &v[..pos + 1];

                    validate_mb(CP, v)?;

                    unsafe { Ok(Self::from_vec_unchecked(v)) }
                }
            }
        }
    }

    /// Constructs an [`AString`] from bytes without validating the code page encoding.
    ///
    /// # Safety
    ///
    /// `v` must contain data valid for code page `CP` up to the first NUL,
    /// without interior NULs in the retained string.
    pub unsafe fn from_vec_unchecked<T: Into<Vec<u8>>>(v: T) -> Self {
        unsafe {
            let mut v = v.into();
            let len = strnlen(v.as_ptr(), v.len());
            if len == v.len() {
                v.push(0);
            }
            v.set_len(len.checked_add(1).expect("len + 1 overflow"));
            Self::from_vec_with_nul_unchecked(v)
        }
    }

    /// Constructs an [`AString`] from NUL-terminated bytes without validation.
    ///
    /// This function does not append a terminator.
    /// The supplied bytes must already contain the final NUL byte.
    ///
    /// # Safety
    ///
    /// `v` must be valid for code page `CP`, NUL-terminated, and without interior NULs.
    #[inline]
    pub unsafe fn from_vec_with_nul_unchecked<T: Into<Vec<u8>>>(v: T) -> Self {
        Self { inner: v.into() }
    }

    #[inline]
    unsafe fn _new(mut v: Vec<u8>, len: usize) -> Self {
        unsafe {
            if len == v.len() {
                // append NULL.
                v.push(0);
            }
            v.set_len(len.checked_add(1).expect("len + 1 overflow"));
            Self::from_vec_with_nul_unchecked(v)
        }
    }

    /// Converts UTF-8 text to an [`AString`] using code page `CP`.
    #[allow(clippy::should_implement_trait)]
    pub fn from_utf8(s: impl AsRef<str>) -> ConvertResult<Self> {
        // UTF-8 -> Wide -> ANSI
        let s = s.as_ref();
        check_nul_in_str(s)?;
        WString::try_from(s)?.try_to_astring()
    }

    /// Attempts to convert UTF-8 text to an [`AString`] using replacement when required by code page `CP`.
    ///
    /// Embedded NUL characters are replaced before conversion. This can still
    /// fail if Windows rejects `CP` or the conversion parameters.
    pub fn try_from_utf8_lossy(s: impl AsRef<str>) -> ConvertResult<Self> {
        // UTF-8 -> Wide -> ANSI
        WString::from_utf8_lossy(s.as_ref()).try_to_astring_lossy()
    }

    /// Converts UTF-8 text to an [`AString`] using replacement when required by code page `CP`.
    ///
    /// # Panics
    ///
    /// Panics if Windows rejects `CP` or if the underlying lossy conversion fails.
    /// Use [`try_from_utf8_lossy`](Self::try_from_utf8_lossy) to handle that error explicitly.
    pub fn from_utf8_lossy(s: impl AsRef<str>) -> Self {
        // UTF-8 -> Wide -> ANSI
        Self::try_from_utf8_lossy(s).expect("Failed to convert from String")
    }

    /// Copies an [`AString`] from a raw NUL-terminated pointer.
    ///
    /// This function scans for the first NUL with `strlen`,
    /// then delegates to [`clone_from_raw_s`](Self::clone_from_raw_s).
    ///
    /// # Safety
    ///
    /// `ptr` must be non-null and readable through the first NUL.
    /// The scanned range must be contained in a single allocated object and must not exceed
    /// `isize::MAX` bytes. The pointed-to data before the first NUL must be
    /// valid for code page `CP` and must not contain interior NULs.
    pub unsafe fn clone_from_raw(ptr: *const u8) -> Self {
        unsafe { Self::clone_from_raw_s(ptr, strlen(ptr)) }
    }

    /// Copies an [`AString`] from a raw pointer and explicit copied byte length without scanning for a terminator.
    ///
    /// `len` is the number of bytes copied from `ptr`.
    /// This function does not read `ptr.add(len)`.
    /// If the copied range does not already end in NUL, a terminating NUL is appended to the new `AString`.
    ///
    /// # Safety
    ///
    /// `ptr` must be non-null and readable for `len` bytes.
    /// The copied range must be contained in a single allocated object and must not exceed `isize::MAX` bytes.
    /// The copied data must be valid for code page `CP` and must not contain NULs except possibly as the final copied byte.
    #[inline]
    pub unsafe fn clone_from_raw_s(ptr: *const u8, len: usize) -> Self {
        unsafe {
            let mut inner = slice::from_raw_parts(ptr, len).to_vec();
            if Some(&0) != inner.last() {
                inner.push(0);
            }
            Self { inner }
        }
    }

    /// Consumes the string and returns the internal NUL-terminated buffer.
    pub fn into_bytes_with_nul(self) -> Vec<u8> { self.inner }
}

impl<const CP: u32> ops::Deref for AString<CP> {
    type Target = AStr<CP>;

    fn deref(&self) -> &Self::Target {
        unsafe { AStr::from_bytes_with_nul_unchecked(self.as_bytes_with_nul()) }
    }
}

impl<const CP: u32> ops::Index<ops::RangeFull> for AString<CP> {
    type Output = AStr<CP>;

    #[inline]
    fn index(&self, _: ops::RangeFull) -> &Self::Output { self }
}

impl<const CP: u32> From<&AStr<CP>> for AString<CP> {
    fn from(x: &AStr<CP>) -> Self {
        unsafe {
            Self::from_vec_with_nul_unchecked(x.as_bytes_with_nul().to_vec())
        }
    }
}

impl<const CP: u32> TryFrom<AString<CP>> for String {
    type Error = ConvertError;

    #[inline]
    fn try_from(value: AString<CP>) -> Result<Self, Self::Error> {
        value.try_to_string()
    }
}

impl<const CP: u32> str::FromStr for AString<CP> {
    type Err = ConvertError;

    fn from_str(s: &str) -> Result<Self, Self::Err> { Self::from_utf8(s) }
}

impl<const CP: u32> TryFrom<&str> for AString<CP> {
    type Error = ConvertError;

    #[inline]
    fn try_from(x: &str) -> Result<Self, Self::Error> { Self::from_utf8(x) }
}

impl<const CP: u32> AsRef<AStr<CP>> for AString<CP> {
    #[inline]
    fn as_ref(&self) -> &AStr<CP> { self }
}

impl<const CP: u32> TryFrom<String> for AString<CP> {
    type Error = ConvertError;

    #[inline]
    fn try_from(x: String) -> Result<Self, Self::Error> {
        Self::try_from(x.as_str())
    }
}

impl<const CP: u32> TryFrom<&String> for AString<CP> {
    type Error = ConvertError;

    #[inline]
    fn try_from(x: &String) -> Result<Self, Self::Error> {
        Self::try_from(x.as_str())
    }
}

impl<const CP: u32> TryFrom<&WStr> for AString<CP> {
    type Error = ConvertError;

    #[inline]
    fn try_from(x: &WStr) -> Result<Self, Self::Error> { x.try_to_astring() }
}

impl<const CP: u32> TryFrom<WString> for AString<CP> {
    type Error = ConvertError;

    #[inline]
    fn try_from(x: WString) -> Result<Self, Self::Error> {
        Self::try_from(x.as_wstr())
    }
}

impl<const CP: u32> TryFrom<&WString> for AString<CP> {
    type Error = ConvertError;

    #[inline]
    fn try_from(x: &WString) -> Result<Self, Self::Error> {
        Self::try_from(x.as_wstr())
    }
}

impl<const CP: u32> PartialEq for AString<CP> {
    #[inline]
    fn eq(&self, other: &Self) -> bool { self.as_bytes() == other.as_bytes() }
}

impl<const CP: u32> Eq for AString<CP> {}

impl<const CP: u32> PartialOrd for AString<CP> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<const CP: u32> Ord for AString<CP> {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_bytes().cmp(other.as_bytes())
    }
}

impl<const CP: u32> PartialEq<AStr<CP>> for AString<CP> {
    #[inline]
    fn eq(&self, other: &AStr<CP>) -> bool {
        self.as_bytes() == other.as_bytes()
    }
}

impl<const CP: u32> PartialEq<&AStr<CP>> for AString<CP> {
    #[inline]
    fn eq(&self, other: &&AStr<CP>) -> bool {
        self.as_bytes() == other.as_bytes()
    }
}

impl<const CP: u32> PartialOrd<AStr<CP>> for AString<CP> {
    #[inline]
    fn partial_cmp(&self, other: &AStr<CP>) -> Option<Ordering> {
        Some(self.as_bytes().cmp(other.as_bytes()))
    }
}

impl<const CP: u32> PartialOrd<&AStr<CP>> for AString<CP> {
    #[inline]
    fn partial_cmp(&self, other: &&AStr<CP>) -> Option<Ordering> {
        self.partial_cmp(*other)
    }
}

impl<'a, const CP: u32> PartialEq<DAStr<'a>> for AString<CP> {
    fn eq(&self, other: &DAStr<'a>) -> bool {
        self.code_page() == other.code_page()
            && self.as_bytes() == other.as_bytes()
    }
}

impl<const CP: u32> PartialEq<DAString> for AString<CP> {
    fn eq(&self, other: &DAString) -> bool {
        self.code_page() == other.code_page()
            && self.as_bytes() == other.as_bytes()
    }
}

impl<const CP: u32> hash::Hash for AString<CP> {
    #[inline]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        CP.hash(state);
        self.as_bytes().hash(state);
    }
}

impl<const CP: u32> ops::Add<&AStr<CP>> for AString<CP> {
    type Output = Self;

    fn add(self, rhs: &AStr<CP>) -> Self::Output {
        let inner = concat_slice(self.as_bytes(), rhs.as_bytes_with_nul());
        unsafe { Self::from_vec_with_nul_unchecked(inner) }
    }
}

impl<const CP: u32> fmt::Debug for AString<CP> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        #[cfg(not(feature = "std"))]
        {
            fmt::Debug::fmt(&self.as_bytes_with_nul(), f)
        }
        #[cfg(feature = "std")]
        {
            fmt::Debug::fmt(&self.as_astr(), f)
        }
    }
}

impl<const CP: u32> fmt::Display for AString<CP> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        #[cfg(not(feature = "std"))]
        {
            fmt::Debug::fmt(&self.as_bytes_with_nul(), f)
        }
        #[cfg(feature = "std")]
        {
            fmt::Display::fmt(&self.as_astr(), f)
        }
    }
}

/// An owned NUL-terminated ANSI string with a runtime-selected code page.
///
/// A valid `DAString` contains bytes accepted by Windows for its stored code page,
/// followed by exactly one terminating NUL in the retained string.
/// Safe constructors reject interior NULs and invalid byte sequences.
/// Unsafe constructors rely on the caller to preserve those invariants.
#[derive(Clone, PartialEq, Eq)]
pub struct DAString {
    inner: Vec<u8>,
    code_page: u32,
}

impl DAString {
    /// Gets the code page.
    #[inline]
    pub fn code_page(&self) -> u32 { self.code_page }

    /// Returns a raw `i8` pointer to the first byte.
    #[inline]
    pub fn as_ptr(&self) -> *const i8 { self.as_dastr().as_ptr() }

    /// Returns a raw `u8` pointer to the first byte.
    #[inline]
    pub fn as_u8_ptr(&self) -> *const u8 { self.as_dastr().as_u8_ptr() }

    /// Returns `true` if the string is empty.
    #[inline]
    pub fn is_empty(&self) -> bool { self.as_bytes().is_empty() }

    /// Returns the bytes excluding the terminating NUL.
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        &self.as_bytes_with_nul()[..self.inner.len() - 1]
    }

    /// Returns the mutable bytes excluding the terminating NUL.
    ///
    /// # Safety
    ///
    /// The caller must preserve a string valid for this value's code page without interior NULs.
    pub unsafe fn as_bytes_mut(&mut self) -> &mut [u8] {
        let bytes = unsafe { self.as_bytes_with_nul_mut() };
        let len = bytes.len();
        &mut bytes[..len - 1]
    }

    /// Returns the bytes including the terminating NUL.
    #[inline]
    pub fn as_bytes_with_nul(&self) -> &[u8] { &self.inner }

    /// Returns the mutable bytes including the terminating NUL.
    ///
    /// # Safety
    ///
    /// The caller must preserve a valid, NUL-terminated string for this value's code page without interior NULs.
    #[inline]
    pub unsafe fn as_bytes_with_nul_mut(&mut self) -> &mut [u8] {
        &mut self.inner
    }

    /// Returns the length of bytes excluding the terminating NUL.
    #[inline]
    pub fn len(&self) -> usize { self.as_bytes().len() }

    /// Returns the length of bytes including the terminating NUL.
    #[inline]
    pub fn len_with_nul(&self) -> usize { self.as_bytes_with_nul().len() }

    /// Returns the string as a borrowed [`DAStr`].
    #[inline]
    pub fn as_dastr(&self) -> DAStr<'_> {
        unsafe {
            DAStr::from_bytes_with_nul_unchecked(
                self.code_page,
                self.as_bytes_with_nul(),
            )
        }
    }

    /// Converts this string to a UTF-8 [`String`].
    pub fn try_to_string(&self) -> ConvertResult<String> {
        self.as_dastr().try_to_string()
    }

    /// Constructs a [`DAString`] from bytes in `code_page`.
    ///
    /// The input must be valid for `code_page` and must not contain interior NULs.
    /// If it is not NUL-terminated, a terminating NUL is appended.
    pub fn new<T: Into<Vec<u8>>>(code_page: u32, v: T) -> ConvertResult<Self> {
        let v = v.into();
        check_interior_nul_u8(&v)?;
        validate_mb(code_page, &v)?;

        unsafe { Ok(Self::from_vec_unchecked(code_page, v)) }
    }

    /// Constructs a [`DAString`] from NUL-terminated bytes in `code_page`.
    ///
    /// The input must be valid for `code_page`,
    /// must end with a NUL byte, and must not contain interior NULs.
    pub fn from_vec_with_nul<T: Into<Vec<u8>>>(
        code_page: u32,
        v: T,
    ) -> ConvertResult<Self> {
        let v = v.into();
        check_interior_nul_u8(&v)?;
        check_nul_terminated_u8(&v)?;
        validate_mb(code_page, &v)?;

        unsafe { Ok(Self::from_vec_with_nul_unchecked(code_page, v)) }
    }

    /// Constructs a [`DAString`] from bytes in `code_page`.
    ///
    /// If the input contains a NUL, the string is truncated at the first NUL.
    /// If the input contains no NUL, a terminating NUL is appended.
    pub fn from_vec_until_nul<T: Into<Vec<u8>>>(
        code_page: u32,
        v: T,
    ) -> ConvertResult<Self> {
        let v = v.into();
        match find_nul_u8(&v) {
            None => {
                validate_mb(code_page, &v)?;
                let len = v.len();

                unsafe { Ok(Self::_new(code_page, v, len)) }
            }
            Some(pos) => {
                if pos == v.len() - 1 {
                    validate_mb(code_page, &v)?;

                    unsafe { Ok(Self::_new(code_page, v, pos)) }
                } else {
                    let v = &v[..pos + 1];

                    validate_mb(code_page, v)?;

                    unsafe { Ok(Self::from_vec_unchecked(code_page, v)) }
                }
            }
        }
    }

    /// Constructs a [`DAString`] from bytes without validating the code page encoding.
    ///
    /// If `v` contains a NUL, only data through the first NUL is retained.
    /// If it contains no NUL, this function appends a final terminator.
    ///
    /// # Safety
    ///
    /// The retained data before the final terminator must be valid for `code_page` and
    /// must not contain interior NULs.
    pub unsafe fn from_vec_unchecked<T: Into<Vec<u8>>>(
        code_page: u32,
        v: T,
    ) -> Self {
        unsafe {
            let mut v = v.into();
            let len = strnlen(v.as_ptr(), v.len());
            if len == v.len() {
                v.push(0);
            }
            v.set_len(len.checked_add(1).expect("len + 1 overflow"));
            Self::from_vec_with_nul_unchecked(code_page, v)
        }
    }

    /// Constructs a [`DAString`] from NUL-terminated bytes without validation.
    ///
    /// This function does not append a terminator. The supplied bytes must
    /// already contain the final NUL byte.
    ///
    /// # Safety
    ///
    /// `v` must be valid for `code_page`, NUL-terminated, and without interior NULs.
    #[inline]
    pub unsafe fn from_vec_with_nul_unchecked<T: Into<Vec<u8>>>(
        code_page: u32,
        v: T,
    ) -> Self {
        Self {
            inner: v.into(),
            code_page,
        }
    }

    #[inline]
    unsafe fn _new(code_page: u32, mut v: Vec<u8>, len: usize) -> Self {
        unsafe {
            if len == v.len() {
                // append NULL.
                v.push(0);
            }
            v.set_len(len.checked_add(1).expect("len + 1 overflow"));
            Self::from_vec_with_nul_unchecked(code_page, v)
        }
    }

    /// Converts UTF-8 text to a [`DAString`] using `code_page`.
    ///
    /// Returns an error if `s` contains an embedded NUL or cannot be represented
    /// in `code_page` without loss.
    #[allow(clippy::should_implement_trait)]
    pub fn from_utf8(
        code_page: u32,
        s: impl AsRef<str>,
    ) -> ConvertResult<Self> {
        // UTF-8 -> Wide -> ANSI
        let s = s.as_ref();
        check_nul_in_str(s)?;
        WString::try_from(s)?.try_to_dastring(code_page)
    }

    /// Attempts to convert UTF-8 text to a [`DAString`] using replacement when required by `code_page`.
    ///
    /// This can still fail if Windows rejects `code_page`.
    pub fn try_from_utf8_lossy(
        code_page: u32,
        s: impl AsRef<str>,
    ) -> ConvertResult<Self> {
        // UTF-8 -> Wide -> ANSI
        WString::from_utf8_lossy(s.as_ref()).try_to_dastring_lossy(code_page)
    }

    /// Converts UTF-8 text to a [`DAString`] using replacement when required by `code_page`.
    ///
    /// # Panics
    ///
    /// Panics if Windows rejects `code_page` or if the underlying lossy conversion fails.
    /// Use [`try_from_utf8_lossy`](Self::try_from_utf8_lossy) to handle that error explicitly.
    pub fn from_utf8_lossy(code_page: u32, s: impl AsRef<str>) -> Self {
        // UTF-8 -> Wide -> ANSI
        Self::try_from_utf8_lossy(code_page, s)
            .expect("Failed to convert from String")
    }

    /// Copies a [`DAString`] from a raw NUL-terminated pointer.
    ///
    /// This function scans for the first NUL with `strlen`, then delegates to
    /// [`clone_from_raw_s`](Self::clone_from_raw_s).
    ///
    /// # Safety
    ///
    /// `ptr` must be non-null and readable through the first NUL.
    /// The scanned range must be contained in a single allocated object and must not exceed `isize::MAX` bytes.
    /// The pointed-to data before the first NUL must be valid for `code_page` and must not contain interior NULs.
    pub unsafe fn clone_from_raw(code_page: u32, ptr: *const u8) -> Self {
        unsafe { Self::clone_from_raw_s(code_page, ptr, strlen(ptr)) }
    }

    /// Copies a [`DAString`] from a raw pointer and explicit copied byte length without scanning for a terminator.
    ///
    /// `len` is the number of bytes copied from `ptr`.
    /// This function does not read `ptr.add(len)`.
    /// If the copied range does not already end in NUL, a terminating NUL is appended to the new `DAString`.
    ///
    /// # Safety
    ///
    /// `ptr` must be non-null and readable for `len` bytes.
    /// The copied range must be contained in a single allocated object and must not exceed `isize::MAX` bytes.
    /// The copied data must be valid for `code_page` and must not contain NULs except possibly as the final copied byte.
    #[inline]
    pub unsafe fn clone_from_raw_s(
        code_page: u32,
        ptr: *const u8,
        len: usize,
    ) -> Self {
        unsafe {
            let mut inner = slice::from_raw_parts(ptr, len).to_vec();
            if Some(&0) != inner.last() {
                inner.push(0);
            }
            Self { inner, code_page }
        }
    }

    /// Returns a new [`DAString`] containing `self` followed by `rhs`.
    ///
    /// Returns [`ConvertError::CodePageMismatch`] if the two strings use different code pages.
    pub fn concat_dastr(&self, rhs: DAStr<'_>) -> ConvertResult<Self> {
        if self.code_page() != rhs.code_page() {
            return Err(ConvertError::CodePageMismatch {
                left: self.code_page(),
                right: rhs.code_page(),
            });
        }

        let inner = concat_slice(self.as_bytes(), rhs.as_bytes_with_nul());
        unsafe {
            Ok(Self::from_vec_with_nul_unchecked(self.code_page(), inner))
        }
    }

    /// Consumes the string and returns the internal NUL-terminated buffer.
    pub fn into_bytes_with_nul(self) -> Vec<u8> { self.inner }

    /// Consumes the string and returns the code page and the internal NUL-terminated buffer.
    pub fn into_parts_with_nul(self) -> (u32, Vec<u8>) {
        (self.code_page, self.inner)
    }
}

impl<'a> From<&DAStr<'a>> for DAString {
    fn from(x: &DAStr<'a>) -> Self {
        unsafe {
            Self::from_vec_with_nul_unchecked(
                x.code_page(),
                x.as_bytes_with_nul().to_vec(),
            )
        }
    }
}

impl TryFrom<DAString> for String {
    type Error = ConvertError;

    #[inline]
    fn try_from(value: DAString) -> Result<Self, Self::Error> {
        value.try_to_string()
    }
}

impl<const CP: u32> From<&AStr<CP>> for DAString {
    fn from(value: &AStr<CP>) -> Self {
        unsafe {
            DAString::from_vec_with_nul_unchecked(CP, value.as_bytes_with_nul())
        }
    }
}

impl<const CP: u32> From<AString<CP>> for DAString {
    fn from(value: AString<CP>) -> Self {
        unsafe {
            DAString::from_vec_with_nul_unchecked(
                CP,
                value.into_bytes_with_nul(),
            )
        }
    }
}

impl<const CP: u32> From<&AString<CP>> for DAString {
    fn from(value: &AString<CP>) -> Self {
        unsafe {
            DAString::from_vec_with_nul_unchecked(CP, value.as_bytes_with_nul())
        }
    }
}

impl<const CP: u32> PartialEq<AString<CP>> for DAString {
    fn eq(&self, other: &AString<CP>) -> bool {
        self.code_page() == other.code_page()
            && self.as_bytes() == other.as_bytes()
    }
}

impl<'a> PartialEq<DAStr<'a>> for DAString {
    fn eq(&self, other: &DAStr<'a>) -> bool {
        self.code_page() == other.code_page()
            && self.as_bytes() == other.as_bytes()
    }
}

impl<'a> PartialEq<&DAStr<'a>> for DAString {
    fn eq(&self, other: &&DAStr<'a>) -> bool {
        self.code_page() == other.code_page()
            && self.as_bytes() == other.as_bytes()
    }
}

impl PartialOrd for DAString {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for DAString {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.code_page()
            .cmp(&other.code_page())
            .then_with(|| self.as_bytes().cmp(other.as_bytes()))
    }
}

impl<'a> PartialOrd<DAStr<'a>> for DAString {
    #[inline]
    fn partial_cmp(&self, other: &DAStr<'a>) -> Option<Ordering> {
        Some(
            self.code_page()
                .cmp(&other.code_page())
                .then_with(|| self.as_bytes().cmp(other.as_bytes())),
        )
    }
}

impl<'a> PartialOrd<&DAStr<'a>> for DAString {
    #[inline]
    fn partial_cmp(&self, other: &&DAStr<'a>) -> Option<Ordering> {
        self.partial_cmp(*other)
    }
}

impl hash::Hash for DAString {
    #[inline]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.code_page().hash(state);
        self.as_bytes().hash(state);
    }
}

impl fmt::Debug for DAString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        #[cfg(not(feature = "std"))]
        {
            fmt::Debug::fmt(&self.as_bytes_with_nul(), f)
        }
        #[cfg(feature = "std")]
        {
            fmt::Debug::fmt(&self.as_dastr(), f)
        }
    }
}

impl fmt::Display for DAString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        #[cfg(not(feature = "std"))]
        {
            fmt::Debug::fmt(&self.as_bytes_with_nul(), f)
        }
        #[cfg(feature = "std")]
        {
            fmt::Display::fmt(&self.as_dastr(), f)
        }
    }
}
