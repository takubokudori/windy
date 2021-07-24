// Copyright takubokudori.
// This source code is licensed under the MIT or Apache-2.0 license.
use core::ffi::c_void;
#[cfg(feature = "std")]
use windy::*;

#[allow(non_snake_case)]
#[cfg(feature = "std")]
#[link(name = "user32")]
extern "system" {
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
    let text = AString::from_str("Hello").unwrap();
    let caption = AString::from_str("CaptionA").unwrap();
    unsafe {
        MessageBoxA(0 as _, text.as_ptr(), caption.as_ptr(), 0);
    }
}

#[cfg(feature = "std")]
fn message_box_w() {
    let text = WString::from_str("World").unwrap();
    let caption = WString::from_str("CaptionW").unwrap();
    unsafe {
        MessageBoxW(0 as _, text.as_ptr(), caption.as_ptr(), 0);
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
