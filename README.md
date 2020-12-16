# Windy

A Windows strings library supports AString (ANSI string) and WString (Unicode string).

# Features

- ANSI string(AString)
- Unicode string(WString)
- Interconversion between AString, WString and String.

# Installation

Add the following lines to your Cargo.toml:

```
[dependencies]
windy = "0.1.1"
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

# License

This software is released under the MIT or Apache-2.0 License, see LICENSE-MIT or LICENSE-APACHE.
