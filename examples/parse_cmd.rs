// Copyright takubokudori.
// This source code is licensed under the MIT or Apache-2.0 license.
#[cfg(feature = "std")]
use std::process::Command;
#[cfg(feature = "std")]
use windy::DAString;

#[cfg(feature = "std")]
unsafe extern "system" {
    fn GetConsoleOutputCP() -> u32;
}

#[cfg(feature = "std")]
fn main() {
    let cp = unsafe { GetConsoleOutputCP() };
    let o = Command::new("cmd")
        .args(["/c", "ThisCommandDoesNotExist"])
        .output()
        .unwrap();
    let (stdout, stderr) = (
        DAString::from_vec_until_nul(cp, o.stdout),
        DAString::from_vec_until_nul(cp, o.stderr),
    );
    println!("stdout: {:?}", stdout);
    println!("stderr: {:?}", stderr);
}

#[cfg(not(feature = "std"))]
fn main() {
    panic!("Use std feature!");
}
