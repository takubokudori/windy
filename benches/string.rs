// Copyright takubokudori.
// This source code is licensed under the MIT or Apache-2.0 license.
use criterion::{criterion_group, criterion_main, Criterion};
use std::convert::TryFrom;
use windy::*;

fn bench_utf8_to_unicode(c: &mut Criterion) {
    c.bench_function("UTF-8 to Unicode", |b| b.iter(|| {
        WString::try_from("Hello World!🍣食べたい").unwrap();
    }));
}

fn bench_unicode_to_utf8(c: &mut Criterion) {
    let s = WString::try_from("Hello World!🍣食べたい").unwrap();
    c.bench_function("Unicode to UTF-8", |b| b.iter(|| {
        s.to_string().unwrap();
    }));
}

fn bench_unicode_to_ansi(c: &mut Criterion) {
    let s = WString::try_from("Hello World").unwrap();
    c.bench_function("Unicode to ANSI", |b| b.iter(|| {
        s.to_astring().unwrap();
    }));
}

fn bench_ansi_to_unicode(c: &mut Criterion) {
    let s = AString::try_from("Hello World").unwrap();
    c.bench_function("ANSI to Unicode", |b| b.iter(|| {
        s.to_wstring().unwrap();
    }));
}

fn bench_utf8_to_ansi(c: &mut Criterion) {
    c.bench_function("UTF-8 to ANSI", |b| b.iter(|| {
        AString::try_from("Hello World").unwrap();
    }));
}

fn bench_ansi_to_utf8(c: &mut Criterion) {
    let s = AString::try_from("Hello World").unwrap();
    c.bench_function("ANSI to UTF-8", |b| b.iter(|| {
        s.to_string().unwrap();
    }));
}

fn bench_utf8_to_unicode_lossy(c: &mut Criterion) {
    c.bench_function("UTF-8 to Unicode lossy", |b| b.iter(|| {
        let _ = WString::from_str_lossy("Hello World!🍣食べたい");
    }));
}

fn bench_unicode_to_utf8_lossy(c: &mut Criterion) {
    let s = WString::try_from("Hello World!🍣食べたい").unwrap();
    c.bench_function("Unicode to UTF-8 lossy", |b| b.iter(|| {
        let _ = s.to_string_lossy();
    }));
}

fn bench_unicode_to_ansi_lossy(c: &mut Criterion) {
    let s = WString::try_from("Hello World").unwrap();
    c.bench_function("Unicode to ANSI lossy", |b| b.iter(|| {
        let _ = s.to_astring_lossy();
    }));
}

fn bench_ansi_to_unicode_lossy(c: &mut Criterion) {
    let s = AString::try_from("Hello World").unwrap();
    c.bench_function("ANSI to Unicode lossy", |b| b.iter(|| {
        let _ = s.to_wstring_lossy();
    }));
}

fn bench_utf8_to_ansi_lossy(c: &mut Criterion) {
    c.bench_function("UTF-8 to ANSI lossy", |b| b.iter(|| {
        let _ = AString::from_str_lossy("Hello World!🍣食べたい");
    }));
}

fn bench_ansi_to_utf8_lossy(c: &mut Criterion) {
    let s = AString::try_from("Hello World").unwrap();
    c.bench_function("ANSI to UTF-8 lossy", |b| b.iter(|| {
        let _ = s.to_string_lossy();
    }));
}

criterion_group!(conversion_benches,
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
