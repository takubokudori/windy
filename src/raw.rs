// Copyright takubokudori.
// This source code is licensed under the MIT or Apache-2 license.
pub(crate) use std::os::raw::{c_int, c_uint, c_ulong, c_char};

#[allow(non_camel_case_types)]
pub(crate) type wchar_t = u16;

pub(crate) type UINT = c_uint;
pub(crate) type DWORD = c_ulong;
pub(crate) type LPBOOL = *mut c_int;
pub(crate) type LPSTR = *mut c_char;
pub(crate) type LPCSTR = *const c_char;
pub(crate) type LPWSTR = *mut wchar_t;
pub(crate) type LPCWSTR = *const wchar_t;

#[allow(non_snake_case)]
extern "C" {
    pub(crate) fn wcslen(s: *const wchar_t) -> usize;

    pub(crate) fn strlen(s: *const u8) -> usize;

    pub(crate) fn wcsnlen(s: *const wchar_t, len: usize) -> usize;

    pub(crate) fn strnlen(s: *const u8, len: usize) -> usize;
}

#[allow(non_snake_case)]
extern "system" {
    pub(crate) fn MultiByteToWideChar(
        CodePage: UINT,
        dwFlags: DWORD,
        lpMultiByteStr: LPCSTR,
        cbMultiByte: c_int,
        lpWideCharStr: LPWSTR,
        cchWideChar: c_int,
    ) -> c_int;

    pub(crate) fn WideCharToMultiByte(
        CodePage: UINT,
        dwFlags: DWORD,
        lpWideCharStr: LPCWSTR,
        cchWideChar: c_int,
        lpMultiByteStr: LPSTR,
        cbMultiByte: c_int,
        lpDefaultChar: LPCSTR,
        lpUsedDefaultChar: LPBOOL,
    ) -> c_int;

    pub(crate) fn GetLastError() -> DWORD;
}
