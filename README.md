# Windy

[![crates.io](https://img.shields.io/crates/v/windy.svg)](https://crates.io/crates/windy)
[![docs.rs](https://docs.rs/windy/badge.svg)](https://docs.rs/windy)

Windows string library for ANSI strings and wide strings.

# Features

- Owned ANSI strings: `AString`.
- Owned wide strings: `WString`.
- Borrowed ANSI strings: `AStr`.
- Borrowed wide strings: `WStr`.
- Dynamically-code-paged ANSI strings: `DAString` and `DAStr`.
- Windows `ANSI_STRING` and `UNICODE_STRING` wrappers.
- Conversions between ANSI strings, wide strings, and UTF-8 `String` values.
- `no_std` support for borrowed string types.

# Installation

Add this crate to `Cargo.toml`:

```toml
[dependencies]
windy = "0.4.0"
```

# no_std support

Disable default features to build without `std`:

```text
--no-default-features
```

Owned string types are available only with the `std` feature.

# Macros

[`windy-macros`](https://github.com/takubokudori/windy-macros) crate provides compile-time conversion from
UTF-8 string literals to ANSI strings or wide strings.

```toml
[dependencies]
windy = "0.4.0"
windy-macros = "0.3.0"
```

## Example

```rust
use std::ffi::c_void;
use windy::WStr;
use windy_macros::wstr;

#[allow(non_snake_case)]
#[link(name = "user32")]
unsafe extern "system" {
    pub fn MessageBoxW(
        hWnd: *mut c_void,
        lpText: *const u16,
        lpCaption: *const u16,
        uType: u32,
    ) -> i32;
}

const MB_OK: u32 = 0;

fn main() {
    let text: &WStr = wstr!("World");
    let caption: &WStr = wstr!("CaptionW");
    unsafe {
        MessageBoxW(std::ptr::null_mut(), text.as_ptr(), caption.as_ptr(), MB_OK);
    }
}
```

# License

This software is released under the MIT or Apache-2.0 License, see LICENSE-MIT or LICENSE-APACHE.
