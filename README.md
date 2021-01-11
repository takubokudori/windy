# Windy

[![crates.io](https://img.shields.io/crates/v/windy.svg)](https://crates.io/crates/windy)
[![docs.rs](https://docs.rs/windy/badge.svg)](https://docs.rs/windy)

A Windows strings library that supports AString (ANSI string) and WString (Unicode string).

# Features

- ANSI string(AString)
- Unicode string(WString)
- Interconversion between AString, WString and String.
- no_std support
- Macros support

# Installation

Add the following lines to your Cargo.toml:

```
[dependencies]
windy = "0.1.3"
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

If you want to use no_std, turn on the `no_std` feature.

AString and WString are not available when `no_std` feature is on.

# Macros support

[windy-macros](https://github.com/takubokudori/windy-macros) to convert a UTF-8 string to WString or AString at compile
time.

If you want to use macros, turn on the `macros` feature.

## Example

```rust
use windy::WStr;
use windy::macros::wstr;

fn main() {
    let s: &WStr = wstr!("test");
}
```

# License

This software is released under the MIT or Apache-2.0 License, see LICENSE-MIT or LICENSE-APACHE.
