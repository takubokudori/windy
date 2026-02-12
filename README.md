# Windy

[![crates.io](https://img.shields.io/crates/v/windy.svg)](https://crates.io/crates/windy)
[![docs.rs](https://docs.rs/windy/badge.svg)](https://docs.rs/windy)

A Windows strings library that supports AString (ANSI string) and WString (Unicode string).

# Features

- ANSI string(AString)
- Wide string(WString)
- AnsiString(ANSI_STRING)
- UnicodeString(UNICODE_STRING)
- Interconversion between AString, WString and String.
- no_std support
- Macros support

# Installation

Add the following lines to your Cargo.toml:

```toml
[dependencies]
windy = "0.2.0"
```

# Example

An example of parsing the outputs of cmd.exe.

```rust
use windy::AString;
use std::process::Command;

fn main() {
    let o = Command::new("cmd")
        .args(&["/c", "ThisCommandDoesNotExist"])
        .output().unwrap();
    let (stdout, stderr) = unsafe {
        (
            AString::new_unchecked(o.stdout),
            AString::new_unchecked(o.stderr)
        )
    };
    println!("stdout: {:?}", stdout);
    println!("stderr: {:?}", stderr);
}
```

# no_std support

If you don't want to use std, use `--no-default-features`.

AString and WString are not available when no_std.

# Macros support

[windy-macros](https://github.com/takubokudori/windy-macros) to convert a UTF-8 string to WString or AString at compile
time.

```toml
[dependencies]
windy = "0.3.0"
windy-macros = "0.1.1"
```

## Example

```rust
use std::ffi::c_void;
use windy::WStr;
use windy_macros::wstr;

#[allow(non_snake_case)]
#[link(name = "user32")]
extern "system" {
    pub fn MessageBoxW(
        hWnd: *mut c_void,
        lpText: *const u16,
        lpCaption: *const u16,
        uType: u32,
    ) -> i32;
}

fn main() {
    let text: &WStr = wstr!("World");
    let caption: &WStr = wstr!("CaptionW");
    unsafe {
        MessageBoxW(0 as _, text.as_ptr(), caption.as_ptr(), 0);
    }
}
```

# License

This software is released under the MIT or Apache-2.0 License, see LICENSE-MIT or LICENSE-APACHE.
