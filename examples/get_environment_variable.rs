// Copyright takubokudori.
// This source code is licensed under the MIT or Apache-2.0 license.
#[cfg(feature = "std")]
use windy::*;

#[allow(non_snake_case)]
#[cfg(feature = "std")]
extern "system" {
    fn GetEnvironmentVariableA(
        lpName: *const i8,
        lpBuffer: *mut u8,
        nSize: u32,
    ) -> u32;
    fn GetEnvironmentVariableW(
        lpName: *const u16,
        lpBuffer: *mut u16,
        nSize: u32,
    ) -> u32;
}

#[cfg(feature = "std")]
fn get_environment_variable_a() {
    let name = AString::from_str("PATH").unwrap();
    let mut buf = Vec::with_capacity(0x1000);
    unsafe {
        let l =
            GetEnvironmentVariableA(name.as_ptr(), buf.as_mut_ptr(), 0x1000);
        if l == 0 {
            println!("GetEnvironmentVariableA failed");
            return;
        }
        buf.set_len(l as usize);
        let value = AString::new_unchecked(buf);
        println!("value: {}", value.to_string_lossy());
    }
}

#[cfg(feature = "std")]
fn get_environment_variable_w() {
    let name = WString::from_str("PATH").unwrap();
    let mut buf = Vec::with_capacity(0x1000);
    unsafe {
        let l =
            GetEnvironmentVariableW(name.as_ptr(), buf.as_mut_ptr(), 0x1000);
        if l == 0 {
            println!("GetEnvironmentVariableW failed");
            return;
        }
        buf.set_len(l as usize);
        let value = WString::new_unchecked(buf);
        println!("value: {}", value.to_string_lossy());
    }
}

#[cfg(feature = "std")]
fn main() {
    println!("*****get_environment_variable_a*****");
    get_environment_variable_a();
    println!("*****get_environment_variable_w*****");
    get_environment_variable_w();
}

#[cfg(not(feature = "std"))]
fn main() {
    panic!("Use std feature!");
}
