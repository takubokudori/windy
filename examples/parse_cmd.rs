// Copyright takubokudori.
// This source code is licensed under the MIT or Apache-2.0 license.
//! An example of parsing the outputs of cmd.exe.
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
