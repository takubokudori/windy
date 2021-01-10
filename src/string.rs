// Copyright takubokudori.
// This source code is licensed under the MIT or Apache-2.0 license.
use crate::{
    __lib::{
        convert::{TryFrom, TryInto},
        fmt::Write,
        ops, slice,
    },
    *,
};

macro_rules! str_impl_debug {
    ($x:ident) => {
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
    };
}

/// Represents wide string (unicode string).
#[repr(C)]
#[derive(Clone, PartialOrd, PartialEq, Eq, Ord, Hash)]
pub struct WString {
    inner: Box<[wchar_t]>,
}

impl WString {
    #[inline]
    pub fn as_bytes_with_nul(&self) -> &[u16] { &self.inner }

    #[inline]
    pub fn as_bytes_with_nul_mut(&mut self) -> &mut [u16] { &mut self.inner }

    #[inline]
    pub fn as_bytes(&self) -> &[u16] {
        &self.as_bytes_with_nul()[..self.inner.len() - 1]
    }

    #[inline]
    pub fn as_u8_bytes_with_nul(&self) -> &[u8] {
        unsafe {
            slice::from_raw_parts(
                self.inner.as_ptr() as *const u8,
                self.inner.len() * 2,
            )
        }
    }

    #[inline]
    pub fn as_u8_bytes(&self) -> &[u8] {
        &self.as_u8_bytes_with_nul()[..self.inner.len() - 2]
    }

    #[inline]
    pub fn as_c_str(&self) -> &WStr { &*self }

    /// Returns &mut [`WStr`].
    #[inline]
    pub fn as_mut_c_str(&mut self) -> &mut WStr {
        unsafe {
            WStr::from_bytes_with_nul_unchecked_mut(
                self.as_bytes_with_nul_mut(),
            )
        }
    }

    #[inline]
    pub fn as_ptr(&self) -> *const u16 { self.inner.as_ptr() }

    /// Returns the length of bytes.
    #[inline]
    pub fn len(&self) -> usize { self.inner.len() }

    /// Returns `true` if the length of bytes is 0.
    #[inline]
    pub fn is_empty(&self) -> bool { self.len() == 0 }

    /// Creates [`WString`] from [`Vec`]<u16> without any encoding checks.
    ///
    /// # Safety
    /// `v` must be a correct unicode string.
    pub unsafe fn new_unchecked<T: Into<Vec<u16>>>(v: T) -> Self {
        Self::_new(v.into())
    }

    /// Creates [`WString`] from [`Vec`]<u8> without any encoding checks.
    ///
    /// # Safety
    /// `v` must be a correct unicode string.
    pub unsafe fn new_c_unchecked<T: Into<Vec<u8>>>(v: T) -> Self {
        Self::_new2(v.into())
    }

    #[inline]
    pub(crate) unsafe fn _new(mut v: Vec<u16>) -> Self {
        let len = wcsnlen(v.as_ptr(), v.len());
        if len == v.len() {
            // append NULL.
            v.reserve_exact(1);
            v.push(0);
        }
        v.set_len(len + 1);
        Self::_new_nul_unchecked(v)
    }

    #[inline]
    pub(crate) unsafe fn _new2(mut v: Vec<u8>) -> Self {
        if v.len() & 1 == 1 {
            v.push(0);
        } // Make the length even.
        let v = v.leak();
        let x = Vec::from_raw_parts(
            v.as_ptr() as *mut u16,
            v.len() / 2,
            v.len() / 2,
        );
        Self::_new(x)
    }

    /// Creates [`WString`] from `v` without a null-terminated check and any encoding checks.
    ///
    /// # Safety
    /// `v` must be a null-terminated unicode string.
    pub unsafe fn new_nul_unchecked<T: Into<Vec<u16>>>(v: T) -> Self {
        let v = v.into();
        Self::_new_nul_unchecked(v)
    }

    #[inline]
    unsafe fn _new_nul_unchecked(v: Vec<u16>) -> Self {
        Self {
            inner: v.into_boxed_slice(),
        }
    }

    /// Converts `&str` to [`WString`].
    ///
    /// # Example
    ///
    /// ```no_run
    /// use windy::WString;
    /// let s = WString::from_str("test🍣").unwrap();
    /// println!("{:?}", s);
    /// ```
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(x: &str) -> ConvertResult<Self> {
        let wb = multi_byte_to_wide_char_wrap(
            CP_UTF8,
            MB_ERR_INVALID_CHARS,
            x.as_bytes(),
        )
        .map_err(conv_err!(@unicode))?;
        // valid UTF-8 string
        unsafe { Ok(Self::_new(wb)) }
    }

    /// Converts `&str` to [`WString`].
    ///
    /// # Example
    ///
    /// ```no_run
    /// use windy::WString;
    /// let s = WString::from_str_lossy("test🍣");
    /// println!("{:?}", s);
    /// ```
    pub fn from_str_lossy(x: &str) -> Self {
        let wb =
            multi_byte_to_wide_char_wrap(CP_UTF8, 0, x.as_bytes()).unwrap();
        // valid UTF-8 string
        unsafe { Self::_new(wb) }
    }

    /// Creates [`WString`] from `ptr`.
    ///
    /// # Safety
    /// `ptr` must be a null-terminated unicode string.
    pub unsafe fn clone_from_raw(ptr: *mut wchar_t) -> Self {
        Self::clone_from_raw_s_unchecked(ptr, wcslen(ptr))
    }

    /// Creates [`WString`] from `ptr` and `len`.
    ///
    /// # Safety
    /// `ptr` must be a null-terminated unicode string.
    pub unsafe fn clone_from_raw_s(ptr: *mut wchar_t, mut len: usize) -> Self {
        let len2 = wcsnlen(ptr, len);
        if len2 < len {
            len = len2;
        }
        Self::clone_from_raw_s_unchecked(ptr, len)
    }

    /// Creates [`WString`] from `ptr` and `len` without length check.
    ///
    /// # Safety
    /// `ptr` must be a null-terminated unicode string.
    #[inline]
    pub unsafe fn clone_from_raw_s_unchecked(
        ptr: *mut wchar_t,
        len: usize,
    ) -> Self {
        let slice = slice::from_raw_parts_mut(ptr, len as usize + 1);
        Self {
            inner: slice.to_vec().into_boxed_slice(),
        }
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

impl Drop for WString {
    fn drop(&mut self) {
        unsafe {
            *self.inner.as_mut_ptr() = 0;
        }
    }
}

impl TryInto<String> for WString {
    type Error = ConvertError;

    #[inline]
    fn try_into(self) -> Result<String, Self::Error> { self.to_string() }
}

impl TryFrom<&str> for WString {
    type Error = ConvertError;

    #[inline]
    fn try_from(x: &str) -> Result<Self, Self::Error> { Self::from_str(x) }
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

impl From<&WStr> for WString {
    fn from(x: &WStr) -> Self {
        unsafe { Self::new_nul_unchecked(x.to_bytes_with_nul().to_vec()) }
    }
}

impl TryFrom<&AStr> for WString {
    type Error = ConvertError;

    #[inline]
    fn try_from(x: &AStr) -> Result<Self, Self::Error> { x.to_wstring() }
}

impl TryFrom<AString> for WString {
    type Error = ConvertError;

    #[inline]
    fn try_from(x: AString) -> Result<Self, Self::Error> {
        Self::try_from(x.as_c_str())
    }
}

impl TryFrom<&AString> for WString {
    type Error = ConvertError;

    #[inline]
    fn try_from(x: &AString) -> Result<Self, Self::Error> {
        Self::try_from(x.as_c_str())
    }
}

/// Represents ANSI string.
#[repr(C)]
#[derive(Clone, PartialOrd, PartialEq, Eq, Ord, Hash)]
pub struct AString {
    inner: Box<[u8]>,
}

impl AString {
    #[inline]
    pub fn as_bytes_with_nul(&self) -> &[u8] { &self.inner }

    #[inline]
    unsafe fn as_bytes_with_nul_mut(&mut self) -> &mut [u8] { &mut self.inner }

    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        &self.as_bytes_with_nul()[..self.inner.len() - 1]
    }

    #[inline]
    pub fn as_c_str(&self) -> &AStr { &*self }

    /// Returns &mut [`AStr`].
    #[inline]
    pub fn as_mut_c_str(&mut self) -> &mut AStr {
        unsafe {
            AStr::from_bytes_with_nul_unchecked_mut(
                self.as_bytes_with_nul_mut(),
            )
        }
    }

    #[inline]
    pub fn as_ptr(&self) -> *const u8 { self.inner.as_ptr() }

    /// Returns the length of bytes.
    #[inline]
    pub fn len(&self) -> usize { self.inner.len() }

    /// Returns `true` if the length of bytes is 0.
    #[inline]
    pub fn is_empty(&self) -> bool { self.len() == 0 }

    /// Creates [`AString`] from `v` without any encoding checks.
    ///
    /// # Safety
    /// `v` must be a correct ANSI string.
    pub unsafe fn new_unchecked<T: Into<Vec<u8>>>(v: T) -> Self {
        let mut v = v.into();
        let len = strnlen(v.as_ptr(), v.len());
        if len == v.len() {
            v.reserve_exact(1);
            v.push(0);
        }
        v.set_len(len + 1);
        Self::new_nul_unchecked(v)
    }

    /// Creates [`AString`] from `v` without a null-terminated check and any encoding checks.
    ///
    /// # Safety
    /// `v` must be a null-terminated ANSI string.
    #[inline]
    pub unsafe fn new_nul_unchecked<T: Into<Vec<u8>>>(v: T) -> Self {
        let v = v.into();
        Self {
            inner: v.into_boxed_slice(),
        }
    }

    /// Converts `&str` to [`AString`].
    ///
    /// # Example
    ///
    /// ```no_run
    /// use windy::AString;
    /// let s = AString::from_str("test").unwrap();
    /// println!("{:?}", s);
    /// ```
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(x: &str) -> ConvertResult<Self> {
        // UTF-8 -> Unicode -> ANSI
        WString::try_from(x)?.to_astring()
    }

    /// Converts `&str` to [`AString`].
    ///
    /// # Example
    ///
    /// ```no_run
    /// use windy::AString;
    /// let s = AString::from_str_lossy("test🍣");
    /// println!("{:?}", s);
    /// ```
    pub fn from_str_lossy(x: &str) -> Self {
        // UTF-8 -> Unicode -> ANSI
        WString::from_str_lossy(x).to_astring_lossy()
    }

    /// Creates [`AString`] from `ptr`.
    ///
    /// # Safety
    /// `ptr` must be a null-terminated ANSI string.
    pub unsafe fn clone_from_raw(ptr: *mut u8) -> Self {
        Self::clone_from_raw_s_unchecked(ptr, strlen(ptr))
    }

    /// Creates [`AString`] from `ptr` and `len`.
    ///
    /// # Safety
    /// `ptr` must be a null-terminated ANSI string.
    pub unsafe fn clone_from_raw_s(ptr: *mut u8, mut len: usize) -> Self {
        let len2 = strnlen(ptr, len);
        if len2 < len {
            len = len2;
        }
        Self::clone_from_raw_s_unchecked(ptr, len)
    }

    /// Creates [`AString`] from `ptr` and `len` without length check.
    ///
    /// # Safety
    /// `ptr` must be a null-terminated ANSI string.
    #[inline]
    pub unsafe fn clone_from_raw_s_unchecked(ptr: *mut u8, len: usize) -> Self {
        let slice = slice::from_raw_parts_mut(ptr, len as usize + 1);
        Self {
            inner: slice.to_vec().into_boxed_slice(),
        }
    }
}

impl ops::Deref for AString {
    type Target = AStr;

    fn deref(&self) -> &Self::Target {
        unsafe { AStr::from_bytes_with_nul_unchecked(self.as_bytes_with_nul()) }
    }
}

impl ops::Index<ops::RangeFull> for AString {
    type Output = AStr;

    #[inline]
    fn index(&self, _: ops::RangeFull) -> &Self::Output { self }
}

impl Drop for AString {
    fn drop(&mut self) {
        unsafe {
            *self.inner.as_mut_ptr() = 0;
        }
    }
}

impl From<&AStr> for AString {
    fn from(x: &AStr) -> Self {
        unsafe { Self::new_nul_unchecked(x.to_bytes_with_nul().to_vec()) }
    }
}

impl TryInto<String> for AString {
    type Error = ConvertError;

    #[inline]
    fn try_into(self) -> Result<String, Self::Error> { self.to_string() }
}

impl TryFrom<&str> for AString {
    type Error = ConvertError;

    #[inline]
    fn try_from(x: &str) -> Result<Self, Self::Error> { Self::from_str(x) }
}

impl TryFrom<String> for AString {
    type Error = ConvertError;

    #[inline]
    fn try_from(x: String) -> Result<Self, Self::Error> {
        Self::try_from(x.as_str())
    }
}

impl TryFrom<&String> for AString {
    type Error = ConvertError;

    #[inline]
    fn try_from(x: &String) -> Result<Self, Self::Error> {
        Self::try_from(x.as_str())
    }
}

impl TryFrom<&WStr> for AString {
    type Error = ConvertError;

    #[inline]
    fn try_from(x: &WStr) -> Result<Self, Self::Error> { x.to_astring() }
}

impl TryFrom<WString> for AString {
    type Error = ConvertError;

    #[inline]
    fn try_from(x: WString) -> Result<Self, Self::Error> {
        Self::try_from(x.as_c_str())
    }
}

impl TryFrom<&WString> for AString {
    type Error = ConvertError;

    #[inline]
    fn try_from(x: &WString) -> Result<Self, Self::Error> {
        Self::try_from(x.as_c_str())
    }
}

str_impl_debug!(WString);
str_impl_debug!(AString);
