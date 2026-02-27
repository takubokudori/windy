// Copyright takubokudori.
// This source code is licensed under the MIT or Apache-2.0 license.
#![allow(
    unused,
    non_camel_case_types,
    clippy::upper_case_acronyms,
    non_snake_case
)]

pub(crate) type c_char = i8;
pub(crate) type c_ushort = u16;
pub(crate) type c_int = i32;
pub(crate) type c_uint = u32;
pub(crate) type c_ulong = u32;
pub(crate) type wchar_t = u16;

pub(crate) type USHORT = c_ushort;
pub(crate) type UINT = c_uint;
pub(crate) type DWORD = c_ulong;
pub(crate) type LPBOOL = *mut c_int;
pub(crate) type LPSTR = *mut c_char;
pub(crate) type LPCSTR = *const c_char;
pub(crate) type PSTR = LPSTR;
pub(crate) type PCSTR = LPCSTR;
pub(crate) type LPWSTR = *mut wchar_t;
pub(crate) type LPCWSTR = *const wchar_t;
pub(crate) type PWSTR = LPWSTR;
pub(crate) type PCWSTR = LPCWSTR;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct UNICODE_STRING {
    pub Length: USHORT,
    pub MaximumLength: USHORT,
    pub Buffer: PWSTR,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct ANSI_STRING {
    pub Length: USHORT,
    pub MaximumLength: USHORT,
    pub Buffer: PSTR,
}

unsafe extern "C" {
    pub(crate) fn wcslen(s: *const wchar_t) -> usize;

    pub(crate) fn strlen(s: *const u8) -> usize;

    pub(crate) fn wcsnlen(s: *const wchar_t, len: usize) -> usize;

    pub(crate) fn strnlen(s: *const u8, len: usize) -> usize;
}

unsafe extern "system" {
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

#[link(name = "ntdll")]
unsafe extern "system" {
    pub(crate) fn RtlInitUnicodeString(
        DestinationString: *mut UNICODE_STRING,
        SourceString: PCWSTR,
    );

    pub(crate) fn RtlInitAnsiString(
        DestinationString: *mut ANSI_STRING,
        SourceString: PCSTR,
    );
}
