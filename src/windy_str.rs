use crate::*;
use crate::__lib::cmp::Ordering;
use crate::__lib::fmt::Write;
use crate::__lib::slice;

macro_rules! str_impl_debug {
    ($x:ident) => (
        impl fmt::Debug for $x {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_char('"')?;
                #[cfg(feature = "no_std")]
                {
                    fmt::Debug::fmt(&self.to_bytes_with_nul(), f)?;
                }
                #[cfg(not(feature = "no_std"))]
                {
                    fmt::Display::fmt(&self.to_string_lossy(), f)?;
                }
                f.write_char('"')
            }
        }
    )
}

#[repr(C)]
pub struct WStr {
    inner: [wchar_t],
}

impl WStr {
    #[inline]
    pub unsafe fn from_bytes_with_nul_unchecked(bytes: &[u16]) -> &Self {
        &*(bytes as *const [u16] as *const Self)
    }

    #[inline]
    pub unsafe fn from_bytes_with_nul_unchecked_mut(bytes: &mut [u16]) -> &mut Self {
        &mut *(bytes as *const [u16] as *mut Self)
    }

    #[inline]
    pub fn as_ptr(&self) -> *const wchar_t { self.inner.as_ptr() }

    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut wchar_t { self.inner.as_mut_ptr() }

    /// Returns the length of bytes.
    #[inline]
    pub fn len(&self) -> usize { self.inner.len() }

    /// Returns `true` if the length of bytes is 0.
    #[inline]
    pub fn is_empty(&self) -> bool { self.len() == 0 }

    #[inline]
    pub fn to_bytes_with_nul(&self) -> &[u16] { &self.inner }

    pub fn to_bytes(&self) -> &[u16] {
        let bytes = self.to_bytes_with_nul();
        &bytes[..bytes.len() - 1]
    }

    #[inline]
    pub fn to_u8_bytes_with_nul(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.inner.as_ptr() as *const u8, self.inner.len() * 2) }
    }

    pub fn to_u8_bytes(&self) -> &[u8] {
        let bytes = self.to_u8_bytes_with_nul();
        &bytes[..bytes.len() - 2]
    }

    #[cfg(not(feature = "no_std"))]
    /// Converts to UTF-8 string.
    ///
    /// If an input has an invalid character, this function returns [`ConvertError::ConvertToUtf8Error`].
    pub fn to_string(&self) -> ConvertResult<String> {
        unsafe {
            let mut mb = wide_char_to_multi_byte_wrap(
                CP_UTF8,
                WC_NO_BEST_FIT_CHARS | WC_ERR_INVALID_CHARS,
                self.to_bytes_with_nul(),
                false,
            ).map_err(conv_err!(@utf8))?;
            mb.set_len(mb.len() - 1); // remove NULL
            // valid UTF-8 string
            Ok(String::from_utf8_unchecked(mb))
        }
    }

    #[cfg(not(feature = "no_std"))]
    /// Converts to UTF-8 string.
    ///
    /// The function replaces Illegal sequences with with `\u{FFFD}`.
    pub fn to_string_lossy(&self) -> String {
        unsafe {
            let mut mb = wide_char_to_multi_byte_wrap(
                CP_UTF8,
                WC_NO_BEST_FIT_CHARS,
                self.to_bytes_with_nul(),
                false,
            ).map_err(conv_err!(@utf8)).unwrap();
            mb.set_len(mb.len() - 1); // remove NULL
            // valid UTF-8 string
            String::from_utf8_unchecked(mb)
        }
    }

    #[cfg(not(feature = "no_std"))]
    /// Converts [`WStr`] to [`AString`].
    ///
    /// # Example
    ///
    /// ```no_run
    /// use windy::{AString, WString};
    /// let s = AString::from_str("test").unwrap();
    /// let s2 = WString::from_str("test").unwrap().to_astring().unwrap();
    /// assert_eq!(s, s2);
    /// ```
    pub fn to_astring(&self) -> ConvertResult<AString> {
        let mb = wide_char_to_multi_byte_wrap(
            CP_ACP,
            WC_NO_BEST_FIT_CHARS,
            self.to_bytes_with_nul(),
            true,
        ).map_err(conv_err!(@ansi))?;
        // valid ANSI string
        unsafe { Ok(AString::new_unchecked(mb)) }
    }

    #[cfg(not(feature = "no_std"))]
    /// Converts [`WStr`] to [`AString`].
    ///
    /// # Example
    ///
    /// ```no_run
    /// use windy::{AString, WString};
    /// let s = AString::from_str("test").unwrap();
    /// let s2 = WString::from_str("test").unwrap().to_astring_lossy();
    /// assert_eq!(s, s2);
    /// ```
    pub fn to_astring_lossy(&self) -> AString {
        let mb = wide_char_to_multi_byte_wrap(
            CP_ACP,
            WC_NO_BEST_FIT_CHARS,
            self.to_bytes_with_nul(),
            false,
        ).unwrap();
        // valid ANSI string
        unsafe { AString::new_unchecked(mb) }
    }
}

impl PartialEq for WStr {
    fn eq(&self, other: &Self) -> bool { self.to_bytes().eq(other.to_bytes()) }
}

impl Eq for WStr {}

impl PartialOrd for WStr {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.to_bytes().partial_cmp(&other.to_bytes())
    }
}

impl Ord for WStr {
    fn cmp(&self, other: &Self) -> Ordering {
        self.to_bytes().cmp(&other.to_bytes())
    }
}

#[repr(C)]
pub struct AStr {
    inner: [u8],
}

impl AStr {
    #[inline]
    pub unsafe fn from_bytes_with_nul_unchecked(bytes: &[u8]) -> &Self {
        &*(bytes as *const [u8] as *const Self)
    }

    #[inline]
    pub unsafe fn from_bytes_with_nul_unchecked_mut(bytes: &mut [u8]) -> &mut Self {
        &mut *(bytes as *const [u8] as *mut Self)
    }

    #[inline]
    pub fn as_ptr(&self) -> *const i8 { self.inner.as_ptr() as *const i8 }

    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut i8 { self.inner.as_mut_ptr() as *mut i8 }

    #[inline]
    pub fn as_u8_ptr(&self) -> *const u8 { self.inner.as_ptr() }

    #[inline]
    pub fn as_mut_u8_ptr(&mut self) -> *mut u8 { self.inner.as_mut_ptr() }

    /// Returns the length of bytes.
    #[inline]
    pub fn len(&self) -> usize { self.inner.len() }

    /// Returns `true` if the length of bytes is 0.
    #[inline]
    pub fn is_empty(&self) -> bool { self.len() == 0 }

    #[inline]
    pub fn to_bytes_with_nul(&self) -> &[u8] { &self.inner }

    #[inline]
    pub fn to_bytes(&self) -> &[u8] {
        let bytes = self.to_bytes_with_nul();
        &bytes[..bytes.len() - 1]
    }

    #[cfg(not(feature = "no_std"))]
    pub fn to_string(&self) -> ConvertResult<String> {
        // ANSI -> Unicode -> UTF-8
        self.to_wstring()?.to_string()
    }

    #[cfg(not(feature = "no_std"))]
    pub fn to_string_lossy(&self) -> String {
        // ANSI -> Unicode -> UTF-8
        self.to_wstring_lossy().to_string_lossy()
    }

    /// Converts [`AStr`] to [`WString`].
    ///
    /// Returns [`ConvertError::ConvertToUnicodeError`] if an input cannot be converted to a wide char.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use windy::{AString, WString};
    /// let s = WString::from_str("test").unwrap();
    /// let s2 = AString::from_str("test").unwrap().to_wstring().unwrap();
    /// assert_eq!(s, s2);
    /// ```
    #[cfg(not(feature = "no_std"))]
    pub fn to_wstring(&self) -> ConvertResult<WString> {
        let wc = multi_byte_to_wide_char_wrap(
            CP_ACP,
            MB_ERR_INVALID_CHARS,
            self.to_bytes(),
        ).map_err(conv_err!(@unicode))?;
        // valid unicode string
        unsafe { Ok(WString::_new(wc)) }
    }

    /// Converts [`AStr`] to [`WString`].
    ///
    /// # Example
    ///
    /// ```no_run
    /// use windy::{AString, WString};
    /// let s = WString::from_str("test").unwrap();
    /// let s2 = AString::from_str("test").unwrap().to_wstring_lossy();
    /// assert_eq!(s, s2);
    /// ```
    #[cfg(not(feature = "no_std"))]
    pub fn to_wstring_lossy(&self) -> WString {
        let wc = multi_byte_to_wide_char_wrap(
            CP_ACP,
            0,
            self.to_bytes(),
        ).map_err(conv_err!(@unicode)).unwrap();
        // valid unicode string
        unsafe { WString::_new(wc) }
    }
}

impl PartialEq for AStr {
    fn eq(&self, other: &Self) -> bool {
        self.to_bytes().eq(other.to_bytes())
    }
}

impl Eq for AStr {}

impl PartialOrd for AStr {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.to_bytes().partial_cmp(other.to_bytes())
    }
}

impl Ord for AStr {
    fn cmp(&self, other: &Self) -> Ordering {
        self.to_bytes().cmp(&other.to_bytes())
    }
}

str_impl_debug!(WStr);
str_impl_debug!(AStr);