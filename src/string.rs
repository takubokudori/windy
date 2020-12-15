// Copyright takubokudori.
// This source code is licensed under the MIT or Apache-2.0 license.
use crate::*;
use std::cmp::Ordering;
use std::convert::TryFrom;
use std::fmt::Write;
use std::ops;
use std::mem::ManuallyDrop;

const CP_ACP: UINT = 0;
const CP_UTF8: UINT = 65001;
const MB_ERR_INVALID_CHARS: DWORD = 0x8;
const WC_ERR_INVALID_CHARS: DWORD = 0x80;
const WC_NO_BEST_FIT_CHARS: DWORD = 0x400;
const ERROR_INSUFFICIENT_BUFFER: DWORD = 0x7a;
const ERROR_NO_UNICODE_TRANSLATION: DWORD = 0x459;

macro_rules! str_impl_debug {
    ($x:ident) => (
        impl fmt::Debug for $x {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_char('"')?;
                fmt::Display::fmt(&self.to_string_lossy(), f)?;
                f.write_char('"')
            }
        }
    )
}

fn wide_char_to_multi_byte_wrap(
    code_page: UINT,
    wc_flags: DWORD,
    x: &[u16],
    used_default_char: bool,
) -> OsResult<Vec<u8>>
{
    let x = if x.len() == 0 { &[0x00] } else { x };
    let l = x.len() * 4;
    let mut ret: Vec<u8> = Vec::with_capacity(l);
    unsafe { ret.set_len(l); }
    let mut udc_flag = false;
    let udc = if used_default_char { Some(&mut udc_flag) } else { None };

    match wide_char_to_multi_byte(
        code_page,
        wc_flags,
        x,
        ret.as_mut_slice(),
        None,
        udc,
    ) {
        Ok(l2) => {
            if udc_flag {
                return Err(ERROR_NO_UNICODE_TRANSLATION);
            }
            unsafe { ret.set_len(l2); }
            Ok(ret)
        }
        Err(ERROR_INSUFFICIENT_BUFFER) => {
            #[cfg(feature = "debug_insufficient_buffer")]
                {
                    println!("WCTMB: ERROR_INSUFFICIENT_BUFFER returned"); // for debug
                }
            wide_char_to_multi_byte2(code_page, wc_flags, x, used_default_char)
        }
        Err(x) => Err(x),
    }
}

/// Gets the required buffer size and gets a multi-byte string.
#[inline]
fn wide_char_to_multi_byte2(
    code_page: UINT,
    wc_flags: DWORD,
    x: &[u16],
    used_default_char: bool,
) -> OsResult<Vec<u8>>
{
    // get the required buffer size.
    let l = wide_char_to_multi_byte(
        code_page,
        wc_flags,
        x,
        &mut [],
        None,
        None,
    )?;
    let mut ret: Vec<u8> = Vec::with_capacity(l);
    unsafe { ret.set_len(l); }
    let mut udc_flag = false;
    let udc = if used_default_char { Some(&mut udc_flag) } else { None };

    let l2 = wide_char_to_multi_byte(
        code_page,
        wc_flags,
        x,
        ret.as_mut_slice(),
        None,
        udc,
    )?;
    if udc_flag {
        return Err(ERROR_NO_UNICODE_TRANSLATION);
    }
    assert_eq!(l, l2);
    Ok(ret)
}

fn multi_byte_to_wide_char_wrap(
    code_page: UINT,
    mb_flags: DWORD,
    x: &[u8],
) -> OsResult<Vec<u16>> {
    let x = if x.len() == 0 { &[0x00] } else { x };
    let l = x.len();
    let mut ret: Vec<u16> = Vec::with_capacity(l);
    unsafe { ret.set_len(l); }

    match multi_byte_to_wide_char(
        code_page,
        mb_flags,
        x,
        ret.as_mut_slice(),
    ) {
        Ok(l2) => {
            unsafe { ret.set_len(l2); }
            Ok(ret)
        }
        Err(ERROR_INSUFFICIENT_BUFFER) => {
            #[cfg(feature = "debug_insufficient_buffer")]
                {
                    println!("MBTWC: ERROR_INSUFFICIENT_BUFFER returned"); // for debug
                }
            multi_byte_to_wide_char2(code_page, mb_flags, x)
        }
        Err(x) => Err(x),
    }
}

/// Gets the required buffer size and gets a wide string.
#[inline]
fn multi_byte_to_wide_char2(
    code_page: UINT,
    mb_flags: DWORD,
    x: &[u8],
) -> OsResult<Vec<u16>> {
    // get the required buffer size.
    let l = multi_byte_to_wide_char(
        code_page,
        mb_flags,
        x,
        &mut [],
    )?;
    let mut ret: Vec<u16> = Vec::with_capacity(l);
    unsafe { ret.set_len(l); }

    let l2 = multi_byte_to_wide_char(
        code_page,
        mb_flags,
        x,
        ret.as_mut_slice(),
    )?;
    assert_eq!(l, l2);
    Ok(ret)
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
    pub fn as_bytes(&self) -> &[u16] { &self.as_bytes_with_nul()[..self.inner.len() - 1] }

    #[inline]
    pub fn as_u8_bytes_with_nul(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.inner.as_ptr() as *const u8, self.inner.len() * 2) }
    }

    #[inline]
    pub fn as_u8_bytes(&self) -> &[u8] { &self.as_u8_bytes_with_nul()[..self.inner.len() - 2] }

    #[inline]
    pub fn as_c_str(&self) -> &WStr { &*self }

    /// Returns &mut [`WStr`].
    #[inline]
    pub unsafe fn as_mut_c_str(&mut self) -> &mut WStr {
        WStr::from_bytes_with_nul_unchecked_mut(self.as_bytes_with_nul())
    }

    #[inline]
    pub fn as_ptr(&self) -> *const u16 { self.inner.as_ptr() }

    #[inline]
    pub fn len(&self) -> usize { self.inner.len() }

    /// Creates [`WString`] from [`Vec`]<u16> without any encoding checks.
    pub unsafe fn new_unchecked<T: Into<Vec<u16>>>(v: T) -> Self { Self::_new(v.into()) }

    /// Creates [`WString`] from [`Vec`]<u8> without any encoding checks.
    pub unsafe fn new_c_unchecked<T: Into<Vec<u8>>>(v: T) -> Self { Self::_new2(v.into()) }

    #[inline]
    unsafe fn _new(mut v: Vec<u16>) -> Self {
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
    unsafe fn _new2(mut v: Vec<u8>) -> Self {
        if v.len() & 1 == 1 { v.push(0); } // Make the length even.
        let v = v.leak();
        let x = Vec::from_raw_parts(v.as_ptr() as *mut u16, v.len() / 2, v.len() / 2);
        Self::_new(x)
    }

    /// # Safety
    /// `v` must be a null-terminated unicode string.
    pub unsafe fn new_nul_unchecked<T: Into<Vec<u16>>>(v: T) -> Self {
        let v = v.into();
        Self::_new_nul_unchecked(v)
    }

    unsafe fn _new_nul_unchecked(v: Vec<u16>) -> Self {
        Self { inner: v.into_boxed_slice() }
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
    pub fn from_str(x: &str) -> ConvertResult<Self> {
        let wb = multi_byte_to_wide_char_wrap(
            CP_UTF8,
            MB_ERR_INVALID_CHARS,
            x.as_bytes(),
        ).map_err(|x| conv_err!(@unicode x))?;
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
        let wb = multi_byte_to_wide_char_wrap(
            CP_UTF8,
            0,
            x.as_bytes(),
        ).unwrap();
        // valid UTF-8 string
        unsafe { Self::_new(wb) }
    }

    /// Converts `ptr` string to [`WString`].
    ///
    /// # Safety
    /// `ptr` must be a null-terminated unicode string.
    pub unsafe fn from_raw(ptr: *mut wchar_t) -> ManuallyDrop<Self> {
        let len = wcslen(ptr);
        let slice = std::slice::from_raw_parts_mut(ptr, len as usize + 1);
        ManuallyDrop::new(
            Self { inner: Box::from_raw(slice) }
        )
    }

    pub unsafe fn from_raw_s(ptr: *mut wchar_t, len: usize) -> ManuallyDrop<Self> {
        let v = Vec::from_raw_parts(ptr, len, len);
        ManuallyDrop::new(
            Self::_new(v)
        )
    }

    /// Converts `ptr` string to [`WString`].
    ///
    /// # Safety
    /// `ptr` must be a null-terminated unicode string.
    pub unsafe fn clone_from_raw(ptr: *mut wchar_t) -> Self {
        let len = wcslen(ptr);
        let slice = std::slice::from_raw_parts_mut(ptr, len as usize + 1);

        Self { inner: slice.to_vec().into_boxed_slice() }
    }

    pub unsafe fn clone_from_raw_s(ptr: *mut wchar_t, len: usize) -> Self {
        let v = Vec::from_raw_parts(ptr, len, len);
        Self::_new(v.clone())
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
    fn index(&self, _: ops::RangeFull) -> &Self::Output {
        self
    }
}

impl Drop for WString {
    fn drop(&mut self) {
        unsafe {
            *self.inner.as_mut_ptr() = 0;
            std::mem::forget(self)
        }
    }
}

impl TryFrom<&str> for WString {
    type Error = ConvertError;

    #[inline]
    fn try_from(x: &str) -> Result<Self, Self::Error> {
        Self::from_str(x)
    }
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

impl TryFrom<&AStr> for WString {
    type Error = ConvertError;

    #[inline]
    fn try_from(x: &AStr) -> Result<Self, Self::Error> {
        x.to_wstring()
    }
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
    pub unsafe fn from_bytes_with_nul_unchecked_mut(bytes: &[u16]) -> &mut Self {
        &mut *(bytes as *const [u16] as *mut Self)
    }

    #[inline]
    pub fn as_ptr(&self) -> *const wchar_t { self.inner.as_ptr() }

    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut wchar_t { self.inner.as_mut_ptr() }

    #[inline]
    pub fn len(&self) -> usize { self.inner.len() }

    #[inline]
    pub fn to_bytes_with_nul(&self) -> &[u16] { &self.inner }

    pub fn to_bytes(&self) -> &[u16] {
        let bytes = self.to_bytes_with_nul();
        &bytes[..bytes.len() - 1]
    }

    #[inline]
    pub fn to_u8_bytes_with_nul(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.inner.as_ptr() as *const u8, self.inner.len() * 2) }
    }

    pub fn to_u8_bytes(&self) -> &[u8] {
        let bytes = self.to_u8_bytes_with_nul();
        &bytes[..bytes.len() - 2]
    }

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
            ).map_err(|x| conv_err!(@utf8 x))?;
            mb.set_len(mb.len() - 1); // remove NULL
            // valid UTF-8 string
            Ok(String::from_utf8_unchecked(mb))
        }
    }

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
            ).map_err(|x| conv_err!(@utf8 x)).unwrap();
            mb.set_len(mb.len() - 1); // remove NULL
            // valid UTF-8 string
            String::from_utf8_unchecked(mb)
        }
    }

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
        ).map_err(|x| conv_err!(@ansi x))?;
        // valid ANSI string
        unsafe { Ok(AString::new_unchecked(mb)) }
    }

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
    pub fn as_bytes(&self) -> &[u8] { &self.as_bytes_with_nul()[..self.inner.len() - 1] }

    #[inline]
    pub fn as_c_str(&self) -> &AStr { &*self }

    /// Returns &mut [`AStr`].
    #[inline]
    pub unsafe fn as_mut_c_str(&mut self) -> &mut AStr {
        AStr::from_bytes_with_nul_unchecked_mut(self.as_bytes_with_nul())
    }

    #[inline]
    pub fn as_ptr(&self) -> *const u8 { self.inner.as_ptr() }

    #[inline]
    pub fn len(&self) -> usize { self.inner.len() }

    /// Creates [`AString`] from `v` without any encoding checks.
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

    /// Creates [`AString`] from `v` without a null-terminated check.
    ///
    /// # Safety
    /// `v` must be a null-terminated ANSI string.
    #[inline]
    pub unsafe fn new_nul_unchecked<T: Into<Vec<u8>>>(v: T) -> Self {
        let v = v.into();
        Self { inner: v.into_boxed_slice() }
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

    /// Converts `ptr` string to [`AString`].
    ///
    /// # Safety
    /// `ptr` must be a null-terminated ANSI string.
    pub unsafe fn from_raw(ptr: *mut u8) -> ManuallyDrop<Self> {
        let len = strlen(ptr);
        let slice = std::slice::from_raw_parts_mut(ptr, len as usize + 1);
        ManuallyDrop::new(
            Self { inner: Box::from_raw(slice) }
        )
    }

    pub unsafe fn from_raw_s(ptr: *mut u8, len: usize) -> ManuallyDrop<Self> {
        let v = Vec::from_raw_parts(ptr, len, len);
        ManuallyDrop::new(
            Self::new_unchecked(v)
        )
    }

    /// Converts `ptr` string to [`AString`].
    ///
    /// # Safety
    /// `ptr` must be a null-terminated ANSI string.
    pub unsafe fn clone_from_raw(ptr: *mut u8) -> Self {
        let len = strlen(ptr);
        let slice = std::slice::from_raw_parts_mut(ptr, len as usize + 1);
        Self { inner: slice.to_vec().into_boxed_slice() }
    }

    pub unsafe fn clone_from_raw_s(ptr: *mut u8, len: usize) -> Self {
        let v = Vec::from_raw_parts(ptr, len, len);
        Self::new_unchecked(v.clone())
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
    fn index(&self, _: ops::RangeFull) -> &Self::Output {
        self
    }
}

impl Drop for AString {
    fn drop(&mut self) {
        unsafe {
            *self.inner.as_mut_ptr() = 0;
            std::mem::forget(self)
        }
    }
}

impl TryFrom<&str> for AString {
    type Error = ConvertError;

    #[inline]
    fn try_from(x: &str) -> Result<Self, Self::Error> {
        Self::from_str(x)
    }
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
    fn try_from(x: &WStr) -> Result<Self, Self::Error> {
        x.to_astring()
    }
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
    pub unsafe fn from_bytes_with_nul_unchecked_mut(bytes: &[u8]) -> &mut Self {
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

    #[inline]
    pub fn len(&self) -> usize { self.inner.len() }

    #[inline]
    pub fn to_bytes_with_nul(&self) -> &[u8] { &self.inner }

    #[inline]
    pub fn to_bytes(&self) -> &[u8] {
        let bytes = self.to_bytes_with_nul();
        &bytes[..bytes.len() - 1]
    }

    pub fn to_string(&self) -> ConvertResult<String> {
        // ANSI -> Unicode -> UTF-8
        self.to_wstring()?.to_string()
    }

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
    pub fn to_wstring(&self) -> ConvertResult<WString> {
        let wc = multi_byte_to_wide_char_wrap(
            CP_ACP,
            MB_ERR_INVALID_CHARS,
            self.to_bytes_with_nul(),
        ).map_err(|x| conv_err!(@unicode x))?;
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
    pub fn to_wstring_lossy(&self) -> WString {
        let wc = multi_byte_to_wide_char_wrap(
            CP_ACP,
            0,
            self.to_bytes_with_nul(),
        ).map_err(|x| conv_err!(@unicode x)).unwrap();
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
        self.to_bytes().partial_cmp(&other.to_bytes())
    }
}

impl Ord for AStr {
    fn cmp(&self, other: &Self) -> Ordering {
        self.to_bytes().cmp(&other.to_bytes())
    }
}

str_impl_debug!(WString);
str_impl_debug!(AString);
str_impl_debug!(WStr);
str_impl_debug!(AStr);
