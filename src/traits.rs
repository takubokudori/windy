// Copyright takubokudori.
// This source code is licensed under the MIT or Apache-2.0 license.
use crate::*;

pub trait ToWString {
    fn to_wstring(&self) -> ConvertResult<WString>;
    fn to_wstring_lossy(&self) -> WString;
}

pub trait ToAString {
    fn to_astring(&self) -> ConvertResult<AString>;
    fn to_astring_lossy(&self) -> AString;
}

macro_rules! impl_to_wstring {
    ($x:ident) => {
        impl ToWString for $x {
            fn to_wstring(&self) -> ConvertResult<WString> {
                WString::from_str(self)
            }

            fn to_wstring_lossy(&self) -> WString {
                WString::from_str_lossy(self)
            }
        }
    };
}

macro_rules! impl_to_astring {
    ($x:ident) => {
        impl ToAString for $x {
            fn to_astring(&self) -> ConvertResult<AString> {
                AString::from_str(self)
            }

            fn to_astring_lossy(&self) -> AString {
                AString::from_str_lossy(self)
            }
        }
    };
}

impl_to_astring!(String);
impl_to_wstring!(String);
impl_to_astring!(str);
impl_to_wstring!(str);
