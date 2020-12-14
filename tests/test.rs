// Copyright takubokudori.
// This source code is licensed under the MIT or Apache-2 license.
#[cfg(test)]
pub mod tests {
    use windy::*;
    use std::convert::TryFrom;

    const ERROR_NO_UNICODE_TRANSLATION: u32 = 1113;
    macro_rules! wn { ($x:expr)=> (unsafe { WString::new_unchecked($x) });}
    macro_rules! wnc { ($x:expr)=>(unsafe { WString::new_c_unchecked($x) });}
    macro_rules! an { ($x:expr)=> (unsafe { AString::new_unchecked($x) });}

    #[test]
    fn test_wstring() {
        let x = wn!(vec![]); // empty vec
        assert_eq!(&[0x00], x.as_bytes_with_nul());
        let x = wn!(vec![0x00]); // empty vec
        assert_eq!(&[0x00], x.as_bytes_with_nul());
        let x = wn!(vec![0x74, 0x65, 0x73, 0x74]); // test
        assert_eq!(&[0x74, 0x65, 0x73, 0x74, 0x00], x.as_bytes_with_nul());
        let x = wn!(vec![0x74, 0x65, 0x73, 0x74, 0x00]); // test\0
        assert_eq!(&[0x74, 0x65, 0x73, 0x74, 0x00], x.as_bytes_with_nul());
        let x = wn!(vec![0x74, 0x65, 0x00, 0x73, 0x74]); // te\0st
        assert_eq!(&[0x74, 0x65, 0x00], x.as_bytes_with_nul());
        let x = wnc!(vec![0x74, 0x00, 0x65, 0x00, 0x73, 0x00, 0x74, 0x00]); // test
        assert_eq!(&[0x74, 0x65, 0x73, 0x74, 0x00], x.as_bytes_with_nul());
        let x = wnc!(vec![0x74, 0x00, 0x65, 0x00, 0x73, 0x00, 0x74, 0x00, 0x00]); // test\0 (odd)
        assert_eq!(&[0x74, 0x65, 0x73, 0x74, 0x00], x.as_bytes_with_nul());
        let x = wnc!(vec![0x74, 0x00, 0x65, 0x00, 0x73, 0x00, 0x74, 0x00, 0x00, 0x00]); // test\0\0 (even)
        assert_eq!(&[0x74, 0x65, 0x73, 0x74, 0x00], x.as_bytes_with_nul());
        assert_eq!(x.as_bytes_with_nul(), x.as_c_str().to_bytes_with_nul());
        assert_eq!(x, WString::try_from("test").unwrap());
        assert_ne!(Ok(x), WString::try_from("Test"));
        let x = &mut [0x0074, 0x0065, 0x0073, 0x0074, 0x0000];
        unsafe {
            let x = WString::from_raw(x.as_mut_ptr());
            assert_eq!("test", x.to_string_lossy());
            std::mem::forget(x);
        }
    }

    #[test]
    fn test_astring() {
        let x = an!(vec![]); // empty vec
        assert_eq!(&[0x00], x.as_bytes_with_nul());
        let x = an!(vec![0x74, 0x65, 0x73, 0x74]); // test
        assert_eq!(&[0x74, 0x65, 0x73, 0x74, 0x00], x.as_bytes_with_nul());
        let x = an!(vec![0x74, 0x65, 0x73, 0x74, 0x00]); // test\0
        assert_eq!(&[0x74, 0x65, 0x73, 0x74, 0x00], x.as_bytes_with_nul());
        let x = an!(vec![0x74, 0x65, 0x00, 0x73, 0x74, 0x00]); // te\0st\0
        assert_eq!(&[0x74, 0x65, 0x00], x.as_bytes_with_nul());
        assert_eq!(x.as_bytes_with_nul(), x.as_c_str().to_bytes_with_nul());
        assert_eq!(x, AString::try_from("te").unwrap());
        assert_ne!(Ok(x), AString::try_from("Te"));
        let x = &mut [0x74, 0x65, 0x73, 0x74, 0x00];
        unsafe {
            let x = AString::from_raw(x.as_mut_ptr());
            assert_eq!("test", x.to_string_lossy());
            std::mem::forget(x);
        }
    }

    #[test]
    fn test_string_conversion() {
        // UTF-8 -> ANSI
        let sa = AString::try_from("test").unwrap();
        assert_eq!("test", sa.to_string().unwrap());  // ANSI -> UTF-8
        assert_eq!("test", sa.to_string_lossy()); // ANSI -> UTF-8 lossy
        // UTF-8 -> Unicode
        let sw = WString::try_from("test").unwrap();
        assert_eq!("test", sw.to_string().unwrap()); // Unicode -> UTF-8
        assert_eq!("test", sw.to_string_lossy()); // Unicode -> UTF-8 lossy
        // Unicode -> ANSI
        let sa = sw.to_astring().unwrap();
        assert_eq!("test", sa.to_string().unwrap());
        assert_eq!("test", sa.to_string_lossy());
        // ANSI -> Unicode
        let sw = sa.to_wstring().unwrap();
        assert_eq!("test", sw.to_string().unwrap()); // ANSI -> UTF-8
        assert_eq!("test", sw.to_string_lossy()); // ANSI -> UTF-8 lossy
        // ANSI -> Unicode lossy
        let sw = sa.to_wstring_lossy();
        assert_eq!("test", sw.to_string().unwrap()); // ANSI -> UTF-8
        assert_eq!("test", sw.to_string_lossy()); // ANSI -> UTF-8 lossy
    }

    #[test]
    fn test_string_conversion_invalid() {
        // UTF-8 -> ANSI (Invalid)
        assert_eq!(Err(conv_err!(@ansi ERROR_NO_UNICODE_TRANSLATION)), AString::try_from("testテスト🍣"));
        // UTF-8 -> Unicode
        let sw = WString::try_from("testテスト🍣").unwrap();
        assert_eq!("testテスト🍣", sw.to_string().unwrap()); // Unicode -> UTF-8
        assert_eq!("testテスト🍣", sw.to_string_lossy()); // Unicode -> UTF-8 lossy
        // Unicode -> ANSI (Invalid)
        assert_eq!(Err(conv_err!(@ansi ERROR_NO_UNICODE_TRANSLATION)), sw.to_astring());
        let _ = sw.to_astring_lossy().to_string_lossy();
    }
}