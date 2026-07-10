// Copyright takubokudori.
// This source code is licensed under the MIT or Apache-2.0 license.
use crate::{
    __lib,
    __lib::{
        mem::MaybeUninit,
        ptr::{null, null_mut},
    },
    ConvertError, ConvertResult, ERROR_INSUFFICIENT_BUFFER,
    ERROR_INVALID_PARAMETER, ERROR_NO_UNICODE_TRANSLATION,
    MB_ERR_INVALID_CHARS,
    raw::*,
};

#[inline]
fn is_special_code_page(code_page: u32) -> bool {
    matches!(
        code_page,
        42 | 50220 | 50221 | 50222 | 50225 | 50227 | 50229 | 52936 | 57002
            ..=57011 | 65000
    )
}

#[cfg(feature = "std")]
/// Converts a multibyte string in `code_page` to a wide string with strict validation.
pub(crate) fn mb_to_wide(code_page: u32, x: &[u8]) -> ConvertResult<Vec<u16>> {
    // 特定のコードページはフラグを 0 にしなければならない
    let mb_flags = if is_special_code_page(code_page) {
        0
    } else {
        MB_ERR_INVALID_CHARS
    };
    multi_byte_to_wide_char_wrap(code_page, mb_flags, x)
}

#[cfg(feature = "std")]
/// Converts a multibyte string in `code_page` to a wide string using API-defined replacement.
pub(crate) fn mb_to_wide_lossy(
    code_page: u32,
    x: &[u8],
) -> ConvertResult<Vec<u16>> {
    multi_byte_to_wide_char_wrap(code_page, 0, x)
}

#[cfg(feature = "std")]
/// Converts a wide string to a multibyte string in `code_page` with best-fit prevention.
pub(crate) fn wide_to_mb(code_page: u32, x: &[u16]) -> ConvertResult<Vec<u8>> {
    if code_page == crate::CP_UTF8 {
        String::from_utf16(x)
            .map(|x| x.into_bytes())
            .map_err(|_| ConvertError::ConvertToUnicodeError)
    } else {
        let (wc_flags, query_used_default_char) =
            if is_special_code_page(code_page) || code_page == 54936 {
                (0, false)
            } else {
                (crate::WC_NO_BEST_FIT_CHARS, true)
            };
        let (v, used_default_char) = wide_char_to_multi_byte_wrap(
            code_page,
            wc_flags,
            x,
            query_used_default_char,
        )?;
        if used_default_char {
            return Err(ConvertError::ConvertToAnsiError);
        }
        Ok(v)
    }
}

#[cfg(feature = "std")]
/// Converts a wide string to a multibyte string in `code_page` using API-defined replacement.
pub(crate) fn wide_to_mb_lossy(
    code_page: u32,
    x: &[u16],
) -> ConvertResult<Vec<u8>> {
    if code_page == crate::CP_UTF8 {
        String::from_utf16(x)
            .map(|x| x.into_bytes())
            .map_err(|_| ConvertError::ConvertToUnicodeError)
    } else {
        let (wc_flags, query_used_default_char) =
            if is_special_code_page(code_page) {
                (0, false)
            } else if code_page == 54936 {
                (crate::WC_ERR_INVALID_CHARS, false)
            } else {
                (crate::WC_NO_BEST_FIT_CHARS, true)
            };
        Ok(wide_char_to_multi_byte_wrap(
            code_page,
            wc_flags,
            x,
            query_used_default_char,
        )?
        .0)
    }
}

#[cfg(feature = "std")]
pub(crate) fn mb_to_mb(
    old_code_page: u32,
    new_code_page: u32,
    x: &[u8],
) -> ConvertResult<Vec<u8>> {
    if old_code_page == new_code_page {
        return Ok(x.to_vec());
    }
    let wide = mb_to_wide(old_code_page, x)?;

    wide_to_mb(new_code_page, &wide)
}

#[cfg(feature = "std")]
pub(crate) fn mb_to_mb_lossy(
    old_code_page: u32,
    new_code_page: u32,
    x: &[u8],
) -> ConvertResult<Vec<u8>> {
    if old_code_page == new_code_page {
        return Ok(x.to_vec());
    }
    let wide = mb_to_wide_lossy(old_code_page, x)?;

    wide_to_mb_lossy(new_code_page, &wide)
}

/// Validates that `x` is decodable in `code_page` and returns the required wide-character count.
pub(crate) fn validate_mb(code_page: u32, x: &[u8]) -> ConvertResult<usize> {
    let x = if x.is_empty() { &[0] } else { x };
    let mb_flags = if is_special_code_page(code_page) {
        0
    } else {
        MB_ERR_INVALID_CHARS
    };
    match multi_byte_to_wide_char(code_page, mb_flags, x, &mut [], true) {
        Ok(x) => Ok(x),
        Err(ConvertError::ConvertToAnsiError)
        | Err(ConvertError::ConvertToUnicodeError) => {
            Err(ConvertError::InvalidAnsiString)
        }
        Err(e) => Err(e),
    }
}

pub(crate) fn validate_wide(x: &[u16]) -> ConvertResult<()> {
    let x = if x.is_empty() { &[0] } else { x };
    if __lib::char::decode_utf16(x.iter().copied()).all(|r| r.is_ok()) {
        return Ok(());
    }

    Err(ConvertError::InvalidUnicodeString)
}

#[inline(always)]
fn multi_byte_to_wide_char(
    code_page: UINT,
    mb_flags: DWORD,
    mb_bytes: &[u8],
    wc_bytes: &mut [MaybeUninit<u16>],
    query_required_len: bool,
) -> ConvertResult<usize> {
    unsafe {
        match MultiByteToWideChar(
            code_page,
            mb_flags,
            mb_bytes.as_ptr() as *const i8,
            mb_bytes
                .len()
                .try_into()
                .map_err(|_| ConvertError::TryFromIntError)?,
            wc_bytes.as_mut_ptr() as *mut u16,
            if query_required_len {
                0
            } else {
                wc_bytes
                    .len()
                    .try_into()
                    .map_err(|_| ConvertError::TryFromIntError)?
            },
        ) {
            0 => match GetLastError() {
                ERROR_INVALID_PARAMETER => {
                    Err(ConvertError::InvalidCodePage(code_page))
                }
                ERROR_INSUFFICIENT_BUFFER => {
                    Err(ConvertError::InsufficientBuffer)
                }
                ERROR_NO_UNICODE_TRANSLATION => {
                    Err(ConvertError::ConvertToUnicodeError)
                }
                e => Err(ConvertError::OsError(e)),
            },
            x => Ok(x as usize),
        }
    }
}

#[cfg(feature = "std")]
fn multi_byte_to_wide_char_wrap(
    code_page: UINT,
    mb_flags: DWORD,
    x: &[u8],
) -> ConvertResult<Vec<u16>> {
    let mut ret: Vec<u16> = Vec::with_capacity(x.len());
    let r = ret.spare_capacity_mut();

    match multi_byte_to_wide_char(code_page, mb_flags, x, r, false) {
        Ok(x) => {
            unsafe {
                ret.set_len(x);
            }
            Ok(ret)
        }
        Err(ConvertError::InsufficientBuffer) => {
            let len = multi_byte_to_wide_char(code_page, mb_flags, x, r, true)?;
            ret.reserve_exact(len - ret.len());
            let r = ret.spare_capacity_mut();
            let len2 =
                multi_byte_to_wide_char(code_page, mb_flags, x, r, false)?;
            assert_eq!(len, len2);
            unsafe {
                ret.set_len(len);
            }
            Ok(ret)
        }
        Err(e) => Err(e),
    }
}

#[inline(always)]
#[allow(unused)]
fn wide_char_to_multi_byte(
    code_page: UINT,
    wc_flags: DWORD,
    wc_bytes: &[u16],
    mb_bytes: &mut [MaybeUninit<u8>],
    query_required_len: bool,
    query_used_default_char: bool,
) -> ConvertResult<(usize, bool)> {
    let mut used_default_char = 0;
    let ptr_udc = if query_used_default_char {
        &mut used_default_char
    } else {
        null_mut()
    };
    unsafe {
        match WideCharToMultiByte(
            code_page,
            wc_flags,
            wc_bytes.as_ptr(),
            wc_bytes
                .len()
                .try_into()
                .map_err(|_| ConvertError::TryFromIntError)?,
            mb_bytes.as_mut_ptr() as *mut i8,
            if query_required_len {
                0
            } else {
                mb_bytes
                    .len()
                    .try_into()
                    .map_err(|_| ConvertError::TryFromIntError)?
            },
            null(),
            ptr_udc,
        ) {
            0 => match GetLastError() {
                ERROR_INVALID_PARAMETER => {
                    Err(ConvertError::InvalidCodePage(code_page))
                }
                ERROR_INSUFFICIENT_BUFFER => {
                    Err(ConvertError::InsufficientBuffer)
                }
                ERROR_NO_UNICODE_TRANSLATION => {
                    Err(ConvertError::ConvertToAnsiError)
                }
                e => Err(ConvertError::OsError(e)),
            },
            x => Ok((x as usize, used_default_char != 0)),
        }
    }
}

#[cfg(feature = "std")]
pub(crate) fn wide_char_to_multi_byte_wrap(
    code_page: UINT,
    wc_flags: DWORD,
    x: &[u16],
    query_used_default_char: bool,
) -> ConvertResult<(Vec<u8>, bool)> {
    let x = if x.is_empty() { &[0] } else { x };

    let l = x.len().checked_mul(2).unwrap_or(x.len());
    let mut ret: Vec<u8> = Vec::with_capacity(l);
    let r = ret.spare_capacity_mut();

    match wide_char_to_multi_byte(
        code_page,
        wc_flags,
        x,
        r,
        false,
        query_used_default_char,
    ) {
        Ok((l2, used_default_char)) => {
            unsafe {
                ret.set_len(l2);
            }
            Ok((ret, used_default_char))
        }
        Err(ConvertError::InsufficientBuffer) => {
            let (len, _) = wide_char_to_multi_byte(
                code_page,
                wc_flags,
                x,
                r,
                true,
                query_used_default_char,
            )?;
            ret.reserve_exact(len - ret.len());
            let r = ret.spare_capacity_mut();
            let (len2, used_default_char) = wide_char_to_multi_byte(
                code_page,
                wc_flags,
                x,
                r,
                false,
                query_used_default_char,
            )?;
            assert_eq!(len, len2);
            unsafe {
                ret.set_len(len2);
            }
            Ok((ret, used_default_char))
        }
        Err(e) => Err(e),
    }
}
