// Copyright takubokudori.
// This source code is licensed under the MIT or Apache-2.0 license.
#![cfg(windows)]
#[cfg(test)]
#[cfg(feature = "std")]
pub mod tests {
    use std::{
        convert::TryFrom, ffi::OsString, os::windows::ffi::OsStringExt,
        str::FromStr,
    };
    use windy::{
        traits::{ToAString, ToDAString, ToWString},
        *,
    };

    const CP_LATIN1: u32 = 37;
    const CP_SJIS: u32 = 932;
    const CP_EUCJP: u32 = 20932;
    type SJISString = AString<CP_SJIS>;
    type EUCJPString = AString<CP_EUCJP>;
    type Utf8String = AString<CP_UTF8>;

    #[test]
    fn test_wstring() {
        let s_test = &[0x74, 0x65, 0x73, 0x74]; // "test"
        let sn_test = &[0x74, 0x65, 0x73, 0x74, 0x00]; // "test\0"
        let mut x = WString::new(s_test).unwrap(); // test
        assert_eq!(sn_test, x.as_bytes_with_nul());
        assert_eq!(sn_test, unsafe { x.as_bytes_with_nul_mut() });
        assert_eq!(s_test, x.as_bytes());
        assert_eq!(s_test, unsafe { x.as_bytes_mut() });
        assert!(!x.is_empty());
        let mut x = WString::new(sn_test).unwrap();
        assert_eq!(sn_test, x.as_bytes_with_nul());
        assert_eq!(sn_test, unsafe { x.as_bytes_with_nul_mut() });
        assert_eq!(s_test, x.as_bytes());
        assert_eq!(s_test, unsafe { x.as_bytes_mut() });
        let s = WString::new([]).unwrap();
        assert_eq!(s.as_bytes_with_nul(), &[0]);
        assert!(s.is_empty());
        let s = WString::new([0]).unwrap();
        assert_eq!(s.as_bytes_with_nul(), &[0]);
        let s = WString::from_vec_until_nul([]).unwrap();
        assert_eq!(s.as_bytes_with_nul(), &[0]);
        let s = WString::from_vec_until_nul([0]).unwrap();
        assert_eq!(s.as_bytes_with_nul(), &[0]);
        let s = WString::from_vec_until_nul(s_test).unwrap();
        assert_eq!(s.as_bytes_with_nul(), sn_test);
        let s = WString::from_vec_until_nul(sn_test).unwrap();
        assert_eq!(s.as_bytes_with_nul(), sn_test);
        let s = WString::from_vec_until_nul(vec![0x74, 0x65, 0x00, 0x73, 0x74])
            .unwrap();
        assert_eq!(s.as_bytes_with_nul(), &[0x74, 0x65, 0x00]);

        let x = unsafe {
            // "te\0st"
            WString::from_vec_unchecked(vec![0x74, 0x65, 0x00, 0x73, 0x74])
        };
        assert_eq!(&[0x74, 0x65, 0x00], x.as_bytes_with_nul());
        assert_eq!(x, WString::try_from("te").unwrap());
        assert_ne!(Ok(x), WString::try_from("Te"));
        let x = &mut [0x0074, 0x0065, 0x0073, 0x0074, 0x0000];
        unsafe {
            let s = WString::clone_from_raw(x.as_mut_ptr());
            assert_eq!("test", s.to_string());
            assert_eq!(sn_test, s.as_bytes_with_nul());
            let s = WString::clone_from_raw_s(x.as_mut_ptr(), 3);
            assert_eq!("tes", s.to_string());
            assert_eq!(&[0x74, 0x65, 0x73, 0x00], s.as_bytes_with_nul());
            let s = WString::clone_from_raw_s(x.as_mut_ptr(), 4);
            assert_eq!("test", s.to_string());
            assert_eq!(sn_test, s.as_bytes_with_nul());
        }

        let s = WString::from_utf8_lossy("テストtest");
        assert_eq!(&format!("{s}"), "テストtest");
        assert_eq!(&format!("{s:?}"), "\"テストtest\"");
        let s = s.as_wstr();
        assert_eq!(&format!("{s}"), "テストtest");
        assert_eq!(&format!("{s:?}"), "\"テストtest\"");
        let s = WString::from_utf8_lossy("te\0st");
        assert_eq!(s.as_bytes(), &[0x0074, 0x0065, 0xfffd, 0x0073, 0x0074]);
    }

    #[test]
    fn test_astring() {
        let s_test = &[0x74, 0x65, 0x73, 0x74]; // "test"
        let sn_test = &[0x74, 0x65, 0x73, 0x74, 0x00]; // "test\0"
        let mut x = ACPString::new(s_test).unwrap(); // test
        assert_eq!(sn_test, x.as_bytes_with_nul());
        assert_eq!(sn_test, unsafe { x.as_bytes_with_nul_mut() });
        assert_eq!(s_test, x.as_bytes());
        assert_eq!(s_test, unsafe { x.as_bytes_mut() });
        assert!(!x.is_empty());
        let mut x = ACPString::new(sn_test).unwrap();
        assert_eq!(sn_test, x.as_bytes_with_nul());
        assert_eq!(sn_test, unsafe { x.as_bytes_with_nul_mut() });
        assert_eq!(s_test, x.as_bytes());
        assert_eq!(s_test, unsafe { x.as_bytes_mut() });
        let s = ACPString::new([]).unwrap();
        assert_eq!(s.as_bytes_with_nul(), &[0]);
        assert!(s.is_empty());
        let s = ACPString::new([0]).unwrap();
        assert_eq!(s.as_bytes_with_nul(), &[0]);
        let s = ACPString::from_vec_until_nul([]).unwrap();
        assert_eq!(s.as_bytes_with_nul(), &[0]);
        let s = ACPString::from_vec_until_nul([0]).unwrap();
        assert_eq!(s.as_bytes_with_nul(), &[0]);
        let s = ACPString::from_vec_until_nul(s_test).unwrap();
        assert_eq!(s.as_bytes_with_nul(), sn_test);
        let s = ACPString::from_vec_until_nul(sn_test).unwrap();
        assert_eq!(s.as_bytes_with_nul(), sn_test);
        let s =
            ACPString::from_vec_until_nul(vec![0x74, 0x65, 0x00, 0x73, 0x74])
                .unwrap();
        assert_eq!(s.as_bytes_with_nul(), &[0x74, 0x65, 0x00]);

        let sn_test = &mut [0x74, 0x65, 0x73, 0x74, 0x00]; // "test\0"
        unsafe {
            let s = ACPString::clone_from_raw(sn_test.as_mut_ptr());
            assert_eq!("test", s.to_string());
            assert_eq!(sn_test, s.as_bytes_with_nul());
            let s = ACPString::clone_from_raw_s(sn_test.as_mut_ptr(), 3);
            assert_eq!("tes", s.to_string());
            assert_eq!(&[0x74, 0x65, 0x73, 0x00], s.as_bytes_with_nul());
            let s = ACPString::clone_from_raw_s(sn_test.as_mut_ptr(), 4);
            assert_eq!("test", s.to_string());
            assert_eq!(sn_test, s.as_bytes_with_nul());
        }
    }

    #[test]
    fn test_dastring() {
        let s_test = &[0x74, 0x65, 0x73, 0x74]; // "test"
        let sn_test = &[0x74, 0x65, 0x73, 0x74, 0x00]; // "test\0"
        let mut x = DAString::new(CP_ACP, s_test).unwrap(); // test
        assert_eq!(sn_test, x.as_bytes_with_nul());
        assert_eq!(sn_test, unsafe { x.as_bytes_with_nul_mut() });
        assert_eq!(s_test, x.as_bytes());
        assert_eq!(s_test, unsafe { x.as_bytes_mut() });
        assert!(!x.is_empty());
        let mut x = DAString::new(CP_ACP, sn_test).unwrap();
        assert_eq!(sn_test, x.as_bytes_with_nul());
        assert_eq!(sn_test, unsafe { x.as_bytes_with_nul_mut() });
        assert_eq!(s_test, x.as_bytes());
        assert_eq!(s_test, unsafe { x.as_bytes_mut() });
        let s = DAString::new(CP_ACP, []).unwrap();
        assert_eq!(s.as_bytes_with_nul(), &[0]);
        assert!(s.is_empty());
        let s = DAString::new(CP_ACP, [0]).unwrap();
        assert_eq!(s.as_bytes_with_nul(), &[0]);
        let s = DAString::from_vec_until_nul(CP_ACP, []).unwrap();
        assert_eq!(s.as_bytes_with_nul(), &[0]);
        let s = DAString::from_vec_until_nul(CP_ACP, [0]).unwrap();
        assert_eq!(s.as_bytes_with_nul(), &[0]);
        let s = DAString::from_vec_until_nul(CP_ACP, s_test).unwrap();
        assert_eq!(s.as_bytes_with_nul(), sn_test);
        let s = DAString::from_vec_until_nul(CP_ACP, sn_test).unwrap();
        assert_eq!(s.as_bytes_with_nul(), sn_test);
        let s = DAString::from_vec_until_nul(
            CP_ACP,
            vec![0x74, 0x65, 0x00, 0x73, 0x74],
        )
        .unwrap();
        assert_eq!(s.as_bytes_with_nul(), &[0x74, 0x65, 0x00]);

        let sn_test = &mut [0x74, 0x65, 0x73, 0x74, 0x00]; // "test\0"
        unsafe {
            let s = DAString::clone_from_raw(CP_ACP, sn_test.as_mut_ptr());
            assert_eq!("test", s.to_string());
            assert_eq!(sn_test, s.as_bytes_with_nul());
            let s = DAString::clone_from_raw_s(CP_ACP, sn_test.as_mut_ptr(), 3);
            assert_eq!("tes", s.to_string());
            assert_eq!(&[0x74, 0x65, 0x73, 0x00], s.as_bytes_with_nul());
            let s = DAString::clone_from_raw_s(CP_ACP, sn_test.as_mut_ptr(), 4);
            assert_eq!("test", s.to_string());
            assert_eq!(sn_test, s.as_bytes_with_nul());
        }
    }

    #[test]
    fn test_string_concatenation() {
        let x = ACPString::from_str("hello ").unwrap();
        let y = ACPString::from_str("world").unwrap();

        let xy = x + &y;
        assert_eq!("hello world", xy.try_to_string().unwrap());
        assert_eq!("hello world", xy.to_string());
        let x = WString::from_str("ハロー🍣").unwrap();
        let y = WString::from_str("world!☆").unwrap();
        let xy = x + &y;
        assert_eq!("ハロー🍣world!☆", xy.to_string());
        assert_eq!("ハロー🍣world!☆", xy.to_string());
    }

    #[test]
    fn test_astring_sjis() {
        let x = SJISString::from_str("テスト").unwrap();
        let xb = &[0x83, 0x65, 0x83, 0x58, 0x83, 0x67, 0];
        assert_eq!(x.as_bytes_with_nul(), xb);

        let k = SJISString::new(xb).unwrap();
        assert_eq!(k, x);

        SJISString::new([0x83, 0x65]).unwrap();
        SJISString::new([0x83, 0x65, 0]).unwrap();
        SJISString::new([0x82, 0x80, 0]).unwrap_err();

        let y = SJISString::from_str("test").unwrap();
        assert_eq!(y.as_bytes_with_nul(), &[0x74, 0x65, 0x73, 0x74, 0]);

        let xy = x + &y;
        assert_eq!(
            xy.as_bytes_with_nul(),
            &[
                0x83, 0x65, 0x83, 0x58, 0x83, 0x67, 0x74, 0x65, 0x73, 0x74, 0
            ]
        );
        assert_eq!(&xy.to_string(), &"テストtest");
        assert_eq!(&xy.try_to_string().unwrap(), &"テストtest");
        assert_eq!(&format!("{xy}"), "テストtest");
        assert_eq!(&format!("{xy:?}"), "\"テストtest\"");
        let xy2 = xy.as_astr();
        assert_eq!(&format!("{xy2}"), "テストtest");
        assert_eq!(&format!("{xy2:?}"), "\"テストtest\"");
        let xy3: AString<CP_SJIS> = xy2.to_astring();
        assert_eq!(xy.as_bytes_with_nul(), xy3.as_bytes_with_nul());

        assert_eq!(
            SJISString::from_str("€").unwrap_err(),
            ConvertError::ConvertToAnsiError
        );

        assert_eq!(
            SJISString::from_utf8_lossy("€").as_bytes_with_nul(),
            &[0x3f, 0]
        );

        assert_eq!(
            SJISString::from_utf8_lossy("te\0st").as_bytes_with_nul(),
            b"te?st\0"
        );
    }

    #[test]
    fn test_astring_eucjp() {
        let x = EUCJPString::from_str("テスト").unwrap();
        assert_eq!(
            x.as_bytes_with_nul(),
            &[0xa5, 0xc6, 0xa5, 0xb9, 0xa5, 0xc8, 0]
        );

        let y = EUCJPString::from_str("test").unwrap();
        assert_eq!(y.as_bytes_with_nul(), &[0x74, 0x65, 0x73, 0x74, 0]);

        let xy = x + &y;
        assert_eq!(
            xy.as_bytes_with_nul(),
            &[
                0xa5, 0xc6, 0xa5, 0xb9, 0xa5, 0xc8, 0x74, 0x65, 0x73, 0x74, 0
            ]
        );
        assert_eq!(&xy.to_string(), &"テストtest");
        assert_eq!(&xy.try_to_string().unwrap(), &"テストtest");

        let sjis_xy: SJISString = xy.to_astring();
        assert_eq!(
            sjis_xy.as_bytes_with_nul(),
            &[
                0x83, 0x65, 0x83, 0x58, 0x83, 0x67, 0x74, 0x65, 0x73, 0x74, 0
            ]
        );

        assert_eq!(
            EUCJPString::from_str("€").unwrap_err(),
            ConvertError::ConvertToAnsiError
        );

        assert_eq!(
            EUCJPString::from_utf8_lossy("€").as_bytes_with_nul(),
            &[0x3f, 0]
        );
        let s: ConvertResult<AString<CP_LATIN1>> = xy.try_to_astring();
        assert_eq!(s.unwrap_err(), ConvertError::ConvertToAnsiError)
    }

    #[test]
    fn test_dastring_sjis() {
        let x = DAString::from_utf8(CP_SJIS, "テスト").unwrap();
        let xb = &[0x83, 0x65, 0x83, 0x58, 0x83, 0x67, 0];
        assert_eq!(x.as_bytes_with_nul(), xb);

        let k = DAString::new(CP_SJIS, xb).unwrap();
        assert_eq!(k, x);

        SJISString::new([0x83, 0x65]).unwrap();
        SJISString::new([0x83, 0x65, 0]).unwrap();
        SJISString::new([0x82, 0x80, 0]).unwrap_err();

        let y = DAString::from_utf8(CP_SJIS, "test").unwrap();
        assert_eq!(y.as_bytes_with_nul(), &[0x74, 0x65, 0x73, 0x74, 0]);

        let xy = x.concat_dastr(y.as_dastr()).unwrap();
        assert_eq!(
            xy.as_bytes_with_nul(),
            &[
                0x83, 0x65, 0x83, 0x58, 0x83, 0x67, 0x74, 0x65, 0x73, 0x74, 0
            ]
        );
        assert_eq!(&xy.to_string(), &"テストtest");
        assert_eq!(&xy.try_to_string().unwrap(), &"テストtest");
        assert_eq!(&format!("{xy}"), "テストtest");
        assert_eq!(&format!("{xy:?}"), "\"テストtest\"");
        let xy2 = xy.as_dastr();
        assert_eq!(&format!("{xy2}"), "テストtest");
        assert_eq!(&format!("{xy2:?}"), "\"テストtest\"");
        let xy3 = xy2.to_dastring(xy2.code_page());
        assert_eq!(xy.as_bytes_with_nul(), xy3.as_bytes_with_nul());

        assert_eq!(
            DAString::from_utf8(CP_SJIS, "€").unwrap_err(),
            ConvertError::ConvertToAnsiError
        );

        assert_eq!(
            DAString::from_utf8_lossy(CP_SJIS, "€").as_bytes_with_nul(),
            &[0x3f, 0]
        );

        assert_eq!(
            DAString::from_utf8_lossy(CP_SJIS, "te\0st").as_bytes_with_nul(),
            b"te?st\0"
        );

        let a = DAString::from_utf8_lossy(CP_SJIS, "test");
        let b = DAString::from_utf8_lossy(CP_UTF8, "test");
        assert_ne!(a, b);
        a.concat_dastr(b.as_dastr()).unwrap_err();
        let a = DAString::from_utf8_lossy(CP_SJIS, "テスト");
        let b = DAString::from_utf8_lossy(CP_UTF8, "テスト");
        assert_ne!(a, b);
        let c = a.as_dastr().to_dastring(CP_UTF8);
        assert_eq!(b, c);
    }

    #[test]
    fn test_utf8() {
        let s = Utf8String::from_utf8("テスト🍣").unwrap();
        assert_eq!(s.as_bytes_with_nul(), "テスト🍣\0".as_bytes());
        assert_eq!(s.try_to_string().unwrap(), "テスト🍣");
        let s = Utf8String::from_utf8_lossy("テスト🍣");
        assert_eq!(s.as_bytes_with_nul(), "テスト🍣\0".as_bytes());
        assert_eq!(s.try_to_string().unwrap(), "テスト🍣");
    }

    #[test]
    fn test_os_string() {
        let os1 = OsString::from("テスト");
        let s = WString::try_from(os1.as_os_str()).unwrap();
        let os2 = s.to_os_string();
        assert_eq!(os1, os2);

        let b = OsString::from_wide(&[0x41, 0x00, 0x42]);
        assert_eq!(
            WString::try_from(b).unwrap_err(),
            StringFormatError::InteriorNul(1).into()
        );

        let b = OsString::from_wide(&[0xd800, 0, 0x41]);

        assert_eq!(
            WString::from_os_str_lossy(b).as_bytes(),
            &[0xfffd, 0xfffd, 0x41]
        );
    }

    #[test]
    fn test_empty_string() {
        // assert_eq!("", &WString::try_from("").unwrap().to_string());
        // assert_eq!("", &WString::try_from("").unwrap().to_string());
        // assert_eq!("", &WString::from_utf8("").unwrap().to_string());
        // assert_eq!("", &WString::from_utf8("").unwrap().to_string());
        assert_eq!(
            "",
            &ACPString::try_from("").unwrap().try_to_string().unwrap()
        );

        assert_eq!("", &ACPString::try_from("").unwrap().to_string());
        assert_eq!(
            "",
            &ACPString::from_utf8_lossy("").try_to_string().unwrap()
        );
        assert_eq!("", &ACPString::from_utf8_lossy("").to_string());
        let s: AString<CP_ACP> = WString::from_utf8("").unwrap().to_astring();

        assert_eq!("", s.to_string());

        let s: AString<CP_ACP> =
            WString::from_utf8("").unwrap().to_astring_lossy();
        assert_eq!("", s.to_string());
        assert_eq!(
            "",
            &ACPString::from_utf8_lossy("").to_wstring().to_string()
        );
        assert_eq!(
            "",
            &ACPString::from_utf8_lossy("").to_wstring().to_string()
        );
    }

    #[test]
    fn test_invalid_string() {
        ACPString::from_str("te\0st").unwrap_err();
        ACPString::from_str("\0st").unwrap_err();
        ACPString::from_str("st\0").unwrap_err();
        ACPString::new(b"te\0st").unwrap_err();
        ACPString::new(b"\0st").unwrap_err();
        WString::new([0x0074, 0x0065, 0x0000, 0x0073, 0x0074, 0x0000])
            .unwrap_err();
        WString::new([0x0000, 0x0065, 0x0073, 0x0074, 0x0000]).unwrap_err();
        WString::from_str("te\0st").unwrap_err();
        WString::from_str("\0st").unwrap_err();
        WString::from_str("st\0").unwrap_err();

        assert_eq!(
            ACPString::new(b"te\0st"),
            Err(StringFormatError::InteriorNul(2).into())
        );
        assert_eq!(
            ACPString::new(b"\0st"),
            Err(StringFormatError::InteriorNul(0).into()),
        );
        assert_eq!(
            SJISString::from_vec_until_nul([0x81, 0x7f, 0x00]),
            Err(ConvertError::InvalidAnsiString),
        );
        assert_eq!(
            SJISString::from_vec_until_nul([0x81, 0x7f, 0x00]),
            Err(ConvertError::InvalidAnsiString),
        );
        assert_eq!(
            WString::new([0x0074, 0x0065, 0x0000, 0x0073, 0x0074]),
            Err(StringFormatError::InteriorNul(2).into())
        );
        assert_eq!(
            WString::new([0x0000, 0x0073, 0x0074]),
            Err(StringFormatError::InteriorNul(0).into()),
        );
        assert_eq!(
            WString::new([0xd800, 0x0041, 0x0000]),
            Err(ConvertError::InvalidUnicodeString),
        );
        assert_eq!(
            WString::from_vec_until_nul([0xd800, 0x0041, 0x0000]),
            Err(ConvertError::InvalidUnicodeString),
        );
        assert_eq!(
            AString::<99999>::from_vec_until_nul([0x00]),
            Err(ConvertError::InvalidCodePage(99999)),
        );
        assert_eq!(
            DAString::from_vec_until_nul(99999, [0x00]),
            Err(ConvertError::InvalidCodePage(99999)),
        );
        assert_eq!(
            AString::<99999>::new([]),
            Err(ConvertError::InvalidCodePage(99999)),
        );
        assert_eq!(
            DAString::new(99999, []),
            Err(ConvertError::InvalidCodePage(99999)),
        );
    }

    #[test]
    fn test_code_pages() {
        assert!(!is_valid_code_page(CP_ACP));
        assert!(is_acceptable_code_page(CP_ACP));
        assert!(is_valid_code_page(CP_UTF8));
        assert!(is_acceptable_code_page(CP_UTF8));

        let sn_test: &[u8] = &[0x74, 0x65, 0x73, 0x74, 0x00]; // ASCII "test\0"
        let sn_test2: &[u16] = &[0x74, 0x65, 0x73, 0x74, 0x00]; // UTF-16LE "test\0"
        let ebcdic_test: &[u8] = &[0xA3, 0x85, 0xA2, 0xA3, 0x00]; // EBCDIC "test\0"
        macro_rules! assert_s {
            ($cp:expr) => {
                let s = AStr::<$cp>::from_bytes_with_nul(sn_test).unwrap();
                assert_eq!(
                    s.as_bytes_with_nul(),
                    sn_test,
                    "AStr (code page {}) test failed",
                    $cp
                );
            };
        }

        macro_rules! assert_s2 {
            ($cp:expr) => {
                let astr = AStr::<$cp>::from_bytes_with_nul(sn_test).unwrap();
                assert_eq!(
                    astr.as_bytes_with_nul(),
                    sn_test,
                    "AStr (code page {}) test failed",
                    $cp
                );
                let wstr = astr.to_wstring();
                assert_eq!(
                    wstr.as_bytes_with_nul(),
                    sn_test2,
                    "to_wstring (code page {}) test failed",
                    $cp
                );
                let astr2: AString<$cp> = wstr.to_astring();
                assert_eq!(
                    &astr2, astr,
                    "to_astring (code page {}) test failed",
                    $cp
                );
            };
        }

        macro_rules! assert_ebcdic {
            ($cp:expr) => {
                let astr =
                    AStr::<$cp>::from_bytes_with_nul(ebcdic_test).unwrap();
                assert_eq!(
                    astr.as_bytes_with_nul(),
                    ebcdic_test,
                    "AStr (code page {}) test failed",
                    $cp
                );
                let wstr = astr.to_wstring();
                assert_eq!(
                    wstr.as_bytes_with_nul(),
                    sn_test2,
                    "to_wstring (code page {}) test failed",
                    $cp
                );
                let astr2: AString<$cp> = wstr.to_astring();
                assert_eq!(
                    &astr2, astr,
                    "to_astring (code page {}) test failed",
                    $cp
                );
            };
        }

        // https://learn.microsoft.com/en-us/windows/win32/intl/code-page-identifiers

        assert_s2!(0);
        assert_s2!(1);
        assert_s2!(2);
        assert_s2!(3);
        assert_ebcdic!(37);
        assert_s!(42); // CP_SYMBOL
        assert_s2!(437);
        assert_ebcdic!(500);
        assert_s2!(708);
        // assert_s!(709); // InvalidCodePage
        // assert_s!(710); // InvalidCodePage
        assert_s2!(720);
        assert_s2!(737);
        assert_s2!(775);
        assert_s2!(850);
        assert_s2!(852);
        assert_s2!(855);
        assert_s2!(857);
        assert_s2!(858);
        assert_s2!(860);
        assert_s2!(861);
        assert_s2!(862);
        assert_s2!(863);
        assert_s2!(864);
        assert_s2!(865);
        assert_s2!(866);
        assert_s2!(869);
        assert_ebcdic!(870);
        assert_s2!(874);
        assert_ebcdic!(875);
        assert_s2!(932);
        assert_s2!(936);
        assert_s2!(949);
        assert_s2!(950);
        assert_ebcdic!(1026);
        assert_ebcdic!(1047);
        assert_ebcdic!(1140);
        assert_ebcdic!(1141);
        assert_ebcdic!(1142);
        assert_ebcdic!(1143);
        assert_ebcdic!(1144);
        assert_ebcdic!(1145);
        assert_ebcdic!(1146);
        assert_ebcdic!(1147);
        assert_ebcdic!(1148);
        assert_ebcdic!(1149);
        // assert_s!(1200); // InvalidCodePage
        // assert_s!(1201); // InvalidCodePage
        assert_s2!(1250);
        assert_s2!(1251);
        assert_s2!(1252);
        assert_s2!(1253);
        assert_s2!(1254);
        assert_s2!(1255);
        assert_s2!(1256);
        assert_s2!(1257);
        assert_s2!(1258);
        assert_s2!(1361);
        assert_s2!(10000);
        assert_s2!(10001);
        assert_s2!(10002);
        assert_s2!(10003);
        assert_s2!(10004);
        assert_s2!(10005);
        assert_s2!(10006);
        assert_s2!(10007);
        assert_s2!(10008);
        assert_s2!(10010);
        assert_s2!(10017);
        assert_s2!(10021);
        assert_s2!(10029);
        assert_s2!(10079);
        assert_s2!(10081);
        assert_s2!(10082);
        // assert_s!(12000); // InvalidCodePage
        // assert_s!(12001); // InvalidCodePage
        assert_s2!(20000);
        assert_s2!(20001);
        assert_s2!(20002);
        assert_s2!(20003);
        assert_s2!(20004);
        assert_s2!(20005);
        assert_s2!(20105);
        assert_s2!(20106);
        assert_s2!(20107);
        assert_s2!(20108);
        assert_s2!(20127);
        assert_s2!(20261);
        assert_s2!(20269);
        assert_ebcdic!(20273);
        assert_ebcdic!(20277);
        assert_ebcdic!(20278);
        assert_ebcdic!(20280);
        assert_ebcdic!(20284);
        assert_ebcdic!(20285);
        // assert_ebcdic!(20290);
        assert_ebcdic!(20297);
        assert_ebcdic!(20420);
        assert_ebcdic!(20423);
        assert_ebcdic!(20424);
        assert_ebcdic!(20833);
        assert_ebcdic!(20838);
        assert_s2!(20866);
        assert_ebcdic!(20871);
        assert_ebcdic!(20880);
        assert_ebcdic!(20905);
        assert_ebcdic!(20924);
        assert_s2!(20932);
        assert_s2!(20936);
        assert_s2!(20949);
        assert_ebcdic!(21025);
        // assert_ebcdic!(21027); // deprecated
        assert_s2!(21866);
        assert_s2!(28591);
        assert_s2!(28592);
        assert_s2!(28593);
        assert_s2!(28594);
        assert_s2!(28595);
        assert_s2!(28596);
        assert_s2!(28597);
        assert_s2!(28598);
        assert_s2!(28599);
        assert_s2!(28603);
        assert_s2!(28605);
        // assert_s!(29001); // InvalidCodePage
        assert_s2!(38598);
        assert_s2!(50220);
        assert_s2!(50221);
        assert_s2!(50222);
        assert_s2!(50225);
        assert_s2!(50227);
        assert_s2!(50229);
        // assert_s!(50930); // InvalidCodePage
        // assert_s!(50931); // InvalidCodePage
        // assert_s!(50933); // InvalidCodePage
        // assert_s!(50935); // InvalidCodePage
        // assert_s!(50936); // InvalidCodePage
        // assert_s!(50937); // InvalidCodePage
        // assert_s!(50939); // InvalidCodePage
        // assert_s!(51932); // InvalidCodePage
        // assert_s!(51936); // InvalidCodePage
        // assert_s!(51949); // InvalidCodePage
        // assert_s!(51950); // InvalidCodePage
        assert_s2!(52936);
        assert_s2!(54936);
        assert_s2!(57002);
        assert_s2!(57003);
        assert_s2!(57004);
        assert_s2!(57005);
        assert_s2!(57006);
        assert_s2!(57007);
        assert_s2!(57008);
        assert_s2!(57009);
        assert_s2!(57010);
        assert_s2!(57011);
        assert_s2!(65000);
        assert_s2!(65001);
    }
}

/// Tests std.
#[cfg(test)]
pub mod tests_no_std {
    use windy::*;

    const CP_SJIS: u32 = 932;
    type SJISStr = AStr<CP_SJIS>;

    #[test]
    fn test_wstr() {
        let s_test = &mut [0x0074, 0x0065, 0x0073, 0x0074];
        let sn_test = &mut [0x0074, 0x0065, 0x0073, 0x0074, 0x0000];

        unsafe {
            let s = WStr::from_raw_mut(sn_test.as_mut_ptr());
            assert_eq!(sn_test, s.as_bytes_with_nul());
            assert_eq!(&sn_test[..4], s.as_bytes());
            assert_eq!(sn_test, s.as_bytes_with_nul());
            assert_eq!(sn_test, s.as_bytes_with_nul_mut());
            assert_eq!(s_test, s.as_bytes());
            assert_eq!(s_test, s.as_bytes_mut());
            assert!(!s.is_empty());

            let s = WStr::from_raw_with_nul(sn_test.as_ptr(), 5);
            assert_eq!(sn_test, s.as_bytes_with_nul());
        }

        let s = WStr::from_bytes_with_nul(&[0]).unwrap();
        const ARR_EMPTY: &[u16] = &[];
        assert_eq!(&[0], s.as_bytes_with_nul());
        assert_eq!(ARR_EMPTY, s.as_bytes());
        assert!(s.is_empty());
        let s = WStr::from_bytes_with_nul(sn_test).unwrap();
        assert_eq!(sn_test, s.as_bytes_with_nul());
        assert_eq!(&sn_test[..4], s.as_bytes());
        let s2 = WStr::from_bytes_with_nul(sn_test).unwrap();
        assert_eq!(sn_test, s2.as_bytes_with_nul());
        assert_eq!(&sn_test[..4], s2.as_bytes());
        assert_eq!(s, s2);

        let s3 = WStr::from_bytes_with_nul(&[0x41, 0x41, 0]).unwrap();
        assert_ne!(s, s3);

        // truncated
        let s = WStr::from_bytes_until_nul(sn_test).unwrap();
        assert_eq!(sn_test, s.as_bytes_with_nul());
        let x2 = &mut [0x0074, 0x0065, 0x0000, 0x0073, 0x0074];
        let s = WStr::from_bytes_until_nul(x2).unwrap();
        assert_eq!(&[0x74, 0x65, 0x00], s.as_bytes_with_nul());
        let x2 = &mut [0x00, 0x0074, 0x0065, 0x0000, 0x0073, 0x0074, 0x00];
        let s = WStr::from_bytes_until_nul(x2).unwrap();
        assert_eq!(&[0x00], s.as_bytes_with_nul());

        // truncated_mut
        let s = WStr::from_bytes_until_nul_mut(sn_test).unwrap();
        assert_eq!(&[0x74, 0x65, 0x73, 0x74, 0x00], s.as_bytes_with_nul());
        let x2 = &mut [0x0074, 0x0065, 0x0000, 0x0073, 0x0074];
        let s = WStr::from_bytes_until_nul_mut(x2).unwrap();
        assert_eq!(&[0x74, 0x65, 0x00], s.as_bytes_with_nul());
        let x2 = &mut [0x00, 0x0074, 0x0065, 0x0000, 0x0073, 0x0074, 0x00];
        let s = WStr::from_bytes_until_nul_mut(x2).unwrap();
        assert_eq!(&[0x00], s.as_bytes_with_nul());
    }

    #[test]
    fn test_astr() {
        let s_test = &mut [0x74, 0x65, 0x73, 0x74];
        let sn_test = &mut [0x74, 0x65, 0x73, 0x74, 0x00];
        unsafe {
            let s = ACPStr::from_raw_mut(sn_test.as_mut_ptr());
            assert_eq!(sn_test, s.as_bytes_with_nul());
            assert_eq!(&sn_test[..4], s.as_bytes());
            assert_eq!(sn_test, s.as_bytes_with_nul());
            assert_eq!(sn_test, s.as_bytes_with_nul_mut());
            assert_eq!(s_test, s.as_bytes());
            assert_eq!(s_test, s.as_bytes_mut());
            assert!(!s.is_empty());
            let s = ACPStr::from_raw_with_nul(sn_test.as_mut_ptr(), 5);
            assert_eq!(sn_test, s.as_bytes_with_nul());
        }

        let s = ACPStr::from_bytes_with_nul(&[0]).unwrap();
        const ARR_EMPTY: &[u8] = &[];
        assert_eq!(&[0], s.as_bytes_with_nul());
        assert_eq!(ARR_EMPTY, s.as_bytes());
        assert!(s.is_empty());
        let s = ACPStr::from_bytes_with_nul(sn_test).unwrap();
        assert_eq!(sn_test, s.as_bytes_with_nul());
        assert_eq!(&sn_test[..4], s.as_bytes());
        let s2 = ACPStr::from_bytes_with_nul(sn_test).unwrap();
        assert_eq!(sn_test, s2.as_bytes_with_nul());
        assert_eq!(&sn_test[..4], s2.as_bytes());

        assert_eq!(s, s2);

        let s_aa = &mut [0x41, 0x41, 0];
        let s3 = ACPStr::from_bytes_with_nul_mut(s_aa).unwrap();
        assert_ne!(s, s3);

        // truncated
        let s = ACPStr::from_bytes_until_nul(sn_test).unwrap();
        assert_eq!(sn_test, s.as_bytes_with_nul());
        let x2 = &mut [0x74, 0x65, 0x00, 0x73, 0x74];
        let s = ACPStr::from_bytes_until_nul(x2).unwrap();
        assert_eq!(&[0x74, 0x65, 0x00], s.as_bytes_with_nul());
        let x2 = &mut [0x00, 0x74, 0x65, 0x00, 0x73, 0x74, 0x00];
        let s = ACPStr::from_bytes_until_nul(x2).unwrap();
        assert_eq!(&[0x00], s.as_bytes_with_nul());

        // truncated_mut
        let s = ACPStr::from_bytes_until_nul_mut(sn_test).unwrap();
        assert_eq!(&[0x74, 0x65, 0x73, 0x74, 0x00], s.as_bytes_with_nul());
        let x2 = &mut [0x74, 0x65, 0x00, 0x73, 0x74];
        let s = ACPStr::from_bytes_until_nul_mut(x2).unwrap();
        assert_eq!(&[0x74, 0x65, 0x00], s.as_bytes_with_nul());
        let x2 = &mut [0x00, 0x74, 0x65, 0x00, 0x73, 0x74, 0x00];
        let s = ACPStr::from_bytes_until_nul_mut(x2).unwrap();
        assert_eq!(&[0x00], s.as_bytes_with_nul());
    }

    #[test]
    fn test_dastr() {
        let s_test = &mut [0x74, 0x65, 0x73, 0x74];
        let sn_test = &mut [0x74, 0x65, 0x73, 0x74, 0x00];
        unsafe {
            let s = DAStr::from_raw(CP_ACP, sn_test.as_mut_ptr());
            assert_eq!(sn_test, s.as_bytes_with_nul());
            assert_eq!(&sn_test[..4], s.as_bytes());
            assert_eq!(sn_test, s.as_bytes_with_nul());
            assert_eq!(s_test, s.as_bytes());
            assert!(!s.is_empty());
            let s = DAStr::from_raw_with_nul(CP_ACP, sn_test.as_mut_ptr(), 5);
            assert_eq!(sn_test, s.as_bytes_with_nul());
        }

        let s = DAStr::from_bytes_with_nul(CP_ACP, &[0]).unwrap();
        const ARR_EMPTY: &[u8] = &[];
        assert_eq!(&[0], s.as_bytes_with_nul());
        assert_eq!(ARR_EMPTY, s.as_bytes());
        assert!(s.is_empty());
        let s = DAStr::from_bytes_with_nul(CP_ACP, sn_test).unwrap();
        assert_eq!(sn_test, s.as_bytes_with_nul());
        assert_eq!(&sn_test[..4], s.as_bytes());
        let s2 = DAStr::from_bytes_with_nul(CP_ACP, sn_test).unwrap();
        assert_eq!(sn_test, s2.as_bytes_with_nul());
        assert_eq!(&sn_test[..4], s2.as_bytes());

        assert_eq!(s, s2);

        let s_aa = &mut [0x41, 0x41, 0];
        let s3 = DAStr::from_bytes_with_nul(CP_ACP, s_aa).unwrap();
        assert_ne!(s, s3);

        // truncated
        let s = DAStr::from_bytes_until_nul(CP_ACP, sn_test).unwrap();
        assert_eq!(sn_test, s.as_bytes_with_nul());
        let x2 = &mut [0x74, 0x65, 0x00, 0x73, 0x74];
        let s = DAStr::from_bytes_until_nul(CP_ACP, x2).unwrap();
        assert_eq!(&[0x74, 0x65, 0x00], s.as_bytes_with_nul());
        let x2 = &mut [0x00, 0x74, 0x65, 0x00, 0x73, 0x74, 0x00];
        let s = DAStr::from_bytes_until_nul(CP_ACP, x2).unwrap();
        assert_eq!(&[0x00], s.as_bytes_with_nul());
    }

    #[test]
    fn test_invalid_str() {
        assert_eq!(
            ACPStr::from_bytes_with_nul(&[]),
            Err(StringFormatError::NotNulTerminated.into())
        );
        assert_eq!(
            ACPStr::from_bytes_with_nul(b"test"),
            Err(StringFormatError::NotNulTerminated.into())
        );
        assert_eq!(
            ACPStr::from_bytes_with_nul(b"te\0st"),
            Err(StringFormatError::InteriorNul(2).into())
        );
        assert_eq!(
            ACPStr::from_bytes_with_nul(b"\0st"),
            Err(StringFormatError::InteriorNul(0).into()),
        );
        assert_eq!(
            ACPStr::from_bytes_until_nul(b"test"),
            Err(StringFormatError::NotNulTerminated.into())
        );
        assert_eq!(
            SJISStr::from_bytes_with_nul(&[0x81, 0x7f, 0x00]),
            Err(ConvertError::InvalidAnsiString),
        );
        assert_eq!(
            SJISStr::from_bytes_until_nul(&[0x81, 0x7f, 0x00]),
            Err(ConvertError::InvalidAnsiString),
        );

        assert_eq!(
            DAStr::from_bytes_with_nul(CP_ACP, &[]),
            Err(StringFormatError::NotNulTerminated.into())
        );
        assert_eq!(
            DAStr::from_bytes_with_nul(CP_ACP, b"test"),
            Err(StringFormatError::NotNulTerminated.into())
        );
        assert_eq!(
            DAStr::from_bytes_with_nul(CP_ACP, b"te\0st"),
            Err(StringFormatError::InteriorNul(2).into())
        );
        assert_eq!(
            DAStr::from_bytes_with_nul(CP_ACP, b"\0st"),
            Err(StringFormatError::InteriorNul(0).into()),
        );
        assert_eq!(
            DAStr::from_bytes_until_nul(CP_ACP, b"test"),
            Err(StringFormatError::NotNulTerminated.into())
        );
        assert_eq!(
            DAStr::from_bytes_with_nul(CP_SJIS, &[0x81, 0x7f, 0x00]),
            Err(ConvertError::InvalidAnsiString),
        );
        assert_eq!(
            DAStr::from_bytes_until_nul(CP_SJIS, &[0x81, 0x7f, 0x00]),
            Err(ConvertError::InvalidAnsiString),
        );

        assert_eq!(
            WStr::from_bytes_with_nul(&[0x0074, 0x0065, 0x0073, 0x0074]),
            Err(StringFormatError::NotNulTerminated.into())
        );
        assert_eq!(
            WStr::from_bytes_with_nul(&[
                0x0074, 0x0065, 0x0000, 0x0073, 0x0074
            ]),
            Err(StringFormatError::InteriorNul(2).into())
        );
        assert_eq!(
            WStr::from_bytes_with_nul(&[0x0000, 0x0073, 0x0074]),
            Err(StringFormatError::InteriorNul(0).into()),
        );
        assert_eq!(
            WStr::from_bytes_until_nul(&[0x0074, 0x0065, 0x0073, 0x0074]),
            Err(StringFormatError::NotNulTerminated.into()),
        );
        assert_eq!(
            WStr::from_bytes_with_nul(&[0xd800, 0x0041, 0x0000]),
            Err(ConvertError::InvalidUnicodeString),
        );
        assert_eq!(
            WStr::from_bytes_until_nul(&[0xd800, 0x0041, 0x0000]),
            Err(ConvertError::InvalidUnicodeString),
        );
    }

    #[test]
    fn test_ansi_string() {
        let x = &mut [0x74, 0x65, 0x73, 0x74, 0x00];
        unsafe {
            let s = ACPStr::from_raw(x.as_mut_ptr());
            assert_eq!(x, s.as_bytes_with_nul());
            let ans = AnsiString::new(s).unwrap();
            assert_eq!(x, ans.as_bytes_with_nul());
            assert_eq!(4, ans.as_raw().Length);
            assert_eq!(5, ans.as_raw().MaximumLength);
            assert_eq!(s.as_ptr(), ans.as_raw().Buffer);
        }
    }

    #[test]
    fn test_unicode_string() {
        let x = &mut [0x0074, 0x0065, 0x0073, 0x0074, 0x0000];
        let s = WStr::from_bytes_with_nul(x).unwrap();
        assert_eq!(x, s.as_bytes_with_nul());
        let us = UnicodeString::new(s).unwrap();
        assert_eq!(x, us.as_bytes_with_nul());
        assert_eq!(8, us.as_raw().Length);
        assert_eq!(10, us.as_raw().MaximumLength);
        assert_eq!(s.as_ptr(), us.as_raw().Buffer);
    }
}
