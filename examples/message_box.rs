// Copyright takubokudori.
// This source code is licensed under the MIT or Apache-2.0 license.
#![allow(dead_code)]
#[cfg(feature = "std")]
use core::ffi::c_void;
#[cfg(feature = "std")]
use windy::*;

const MB_OK: u32 = 0;

#[allow(non_snake_case)]
#[cfg(feature = "std")]
#[link(name = "user32")]
unsafe extern "system" {
    pub fn MessageBoxA(
        hWnd: *mut c_void,
        lpText: *const i8,
        lpCaption: *const i8,
        uType: u32,
    ) -> i32;

    pub fn MessageBoxW(
        hWnd: *mut c_void,
        lpText: *const u16,
        lpCaption: *const u16,
        uType: u32,
    ) -> i32;
}

#[cfg(feature = "std")]
fn message_box_a() {
    let text = ACPString::from_utf8("Hello").unwrap();
    let caption = ACPString::from_utf8("CaptionA").unwrap();
    unsafe {
        MessageBoxA(
            std::ptr::null_mut(),
            text.as_ptr(),
            caption.as_ptr(),
            MB_OK,
        );
    }
}

#[cfg(feature = "std")]
fn message_box_w() {
    let text = WString::from_utf8("World").unwrap();
    let caption = WString::from_utf8("CaptionW").unwrap();
    unsafe {
        MessageBoxW(
            std::ptr::null_mut(),
            text.as_ptr(),
            caption.as_ptr(),
            MB_OK,
        );
    }
}

#[cfg(feature = "std")]
fn main() {
    println!("*****message_box_a*****");
    message_box_a();
    println!("*****message_box_w*****");
    message_box_w();
}

#[cfg(not(feature = "std"))]
fn main() {
    panic!("Use std feature!");
}
