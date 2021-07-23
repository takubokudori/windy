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

```
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

```
[dependencies]
windy = "0.2.0"
windy-macros = "0.1.1"
```

## Example

```rust
use windy::WStr;
use windy_macros::wstr;

fn main() {
    let s: &WStr = wstr!("test");
}
```

# License

This software is released under the MIT or Apache-2.0 License, see LICENSE-MIT or LICENSE-APACHE.
