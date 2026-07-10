// Copyright takubokudori.
// This source code is licensed under the MIT or Apache-2.0 license.
use criterion::{Criterion, criterion_group, criterion_main};
use std::convert::TryFrom;
use windy::{
    traits::{ToAString, ToWString},
    *,
};

fn bench_utf8_to_unicode(c: &mut Criterion) {
    let s = "😀".repeat(30);
    c.bench_function("UTF-8 to Unicode", |b| {
        b.iter(|| WString::try_from(&s).unwrap())
    });
}

fn bench_unicode_to_utf8(c: &mut Criterion) {
    let s = WString::try_from("😀".repeat(30)).unwrap();
    c.bench_function("Unicode to UTF-8", |b| b.iter(|| s.to_string()));
}

fn bench_unicode_to_ansi(c: &mut Criterion) {
    let s = WString::try_from("Hello World").unwrap();
    c.bench_function("Unicode to ANSI", |b| {
        b.iter(|| <WStr as ToAString<CP_ACP>>::to_astring(&s))
    });
}

fn bench_ansi_to_unicode(c: &mut Criterion) {
    let s = ACPString::try_from("Hello World").unwrap();
    c.bench_function("ANSI to Unicode", |b| b.iter(|| s.to_wstring()));
}

fn bench_utf8_to_ansi(c: &mut Criterion) {
    c.bench_function("UTF-8 to ANSI", |b| {
        b.iter(|| ACPString::try_from("Hello World").unwrap())
    });
}

fn bench_ansi_to_utf8(c: &mut Criterion) {
    let s = ACPString::try_from("Hello World").unwrap();
    c.bench_function("ANSI to UTF-8", |b| {
        b.iter(|| s.try_to_string().unwrap())
    });
}

fn bench_utf8_to_unicode_lossy(c: &mut Criterion) {
    let s = "😀".repeat(30);
    c.bench_function("UTF-8 to Unicode lossy", |b| {
        b.iter(|| WString::from_utf8_lossy(&s))
    });
}

fn bench_unicode_to_utf8_lossy(c: &mut Criterion) {
    let s = WString::try_from("😀".repeat(30)).unwrap();
    c.bench_function("Unicode to UTF-8 lossy", |b| b.iter(|| s.to_string()));
}

fn bench_unicode_to_ansi_lossy(c: &mut Criterion) {
    let s = WString::try_from("Hello World").unwrap();
    c.bench_function("Unicode to ANSI lossy", |b| {
        b.iter(|| <windy::WStr as ToAString<CP_ACP>>::to_astring_lossy(&s))
    });
}

fn bench_ansi_to_unicode_lossy(c: &mut Criterion) {
    let s = ACPString::try_from("Hello World").unwrap();
    c.bench_function("ANSI to Unicode lossy", |b| b.iter(|| s.to_wstring()));
}

fn bench_utf8_to_ansi_lossy(c: &mut Criterion) {
    let s = "😀".repeat(30);
    c.bench_function("UTF-8 to ANSI lossy", |b| {
        b.iter(|| ACPString::from_utf8_lossy(&s))
    });
}

fn bench_ansi_to_utf8_lossy(c: &mut Criterion) {
    let s = ACPString::try_from("Hello World").unwrap();
    c.bench_function("ANSI to UTF-8 lossy", |b| b.iter(|| s.to_string()));
}

criterion_group!(
    conversion_benches,
    bench_utf8_to_unicode,
    bench_unicode_to_utf8,
    bench_unicode_to_ansi,
    bench_ansi_to_unicode,
    bench_utf8_to_ansi,
    bench_ansi_to_utf8,
    bench_utf8_to_unicode_lossy,
    bench_unicode_to_utf8_lossy,
    bench_unicode_to_ansi_lossy,
    bench_ansi_to_unicode_lossy,
    bench_utf8_to_ansi_lossy,
    bench_ansi_to_utf8_lossy,
);

criterion_main!(conversion_benches);
