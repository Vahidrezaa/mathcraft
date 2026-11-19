//! پاریتی CI با اوراکل **Octave** — گام 3.7 DoD («.mat subset parity vs
//! Octave-written fixtures»).
//!
//! اجرای کامل فقط وقتی `MC_OCTAVE_FIXTURES` به دایرکتوری فیکسچرهای تولیدشده
//! با `tests/gen_octave_fixtures.m` اشاره کند (جاب CI mat-parity). در حالت
//! محلی بدون Octave، تست با پیام skip می‌گذرد — فیکسچرهای کامیت‌شدهٔ scipy
//! پوشش محلی را می‌دهند (پروونانس: `tests/fixtures/README.md`).
//!
//! برابری **ارزشی** است نه بایتی (هدر متنی نویسنده‌ها فرق دارد؛ اسپک نه).

use std::env;
use std::fs;
use std::path::PathBuf;

use mc_core::value::Value;
use mc_io::matv5::read_mat;

fn dir() -> Option<PathBuf> {
    env::var_os("MC_OCTAVE_FIXTURES").map(PathBuf::from)
}

fn load(name: &str) -> Vec<(String, Value)> {
    let d = dir().expect("caller checks");
    let bytes = fs::read(d.join(name)).unwrap_or_else(|e| panic!("read {name}: {e}"));
    read_mat(&bytes).unwrap_or_else(|e| panic!("parse {name}: {}", e.message_en))
}

fn dbl(v: &Value) -> Vec<f64> {
    let Value::Double(m) = v else {
        panic!("expected double, got {v:?}")
    };
    m.as_column_slice().to_vec()
}

#[test]
fn octave_written_files_parse_identically() {
    let Some(_) = dir() else {
        eprintln!("skipped: MC_OCTAVE_FIXTURES not set (CI-only oracle)");
        return;
    };

    // v6 بدون فشرده‌سازی
    let vars = load("dbl_2x3.mat");
    assert_eq!(vars[0].0, "M");
    assert_eq!(dbl(&vars[0].1), vec![1.0, 4.0, 2.0, 5.0, 3.0, 6.0]);

    // v7 فشرده (مسیر flate2) — مختلط
    let vars = load("cplx_2x2.mat");
    let Value::Complex(c) = &vars[0].1 else {
        panic!("expected complex")
    };
    assert_eq!(c.re_column_slice(), &[1.0, 3.5, -0.0, 0.0]);
    assert_eq!(c.im_column_slice(), &[2.0, -4.0, -1.0, 0.5]);

    // char
    let vars = load("char_row.mat");
    assert_eq!(vars[0].1, Value::Char("hello".to_owned()));

    // struct تودرتو
    let vars = load("struct_1x1.mat");
    let Value::Struct(st) = &vars[0].1 else {
        panic!("expected struct")
    };
    assert_eq!(dbl(st.get("a").expect("a")), vec![1.25]);
    assert_eq!(st.get("b"), Some(&Value::Char("txt".to_owned())));

    // الگوهای ویژه — بیت‌ها زنده
    let vars = load("nan_inf.mat");
    let m = dbl(&vars[0].1);
    assert!(m[0].is_nan());
    assert_eq!(m[1], f64::INFINITY);
    assert_eq!(m[2], f64::NEG_INFINITY);
    assert_eq!(m[3].to_bits(), (-0.0f64).to_bits());

    // کلاس‌های عددی → cast به double؛ logical → دلتای مستند
    let vars = load("int32_col.mat");
    assert_eq!(dbl(&vars[0].1), vec![7.0, -8.0, 9.0]);
    let vars = load("single_2x2.mat");
    assert_eq!(dbl(&vars[0].1), vec![0.5, -2.5, 1.5, 3.5]);
    let vars = load("logical_1x3.mat");
    assert_eq!(dbl(&vars[0].1), vec![1.0, 0.0, 1.0]);

    // خالی
    let vars = load("empty_0x0.mat");
    assert_eq!(dbl(&vars[0].1).len(), 0);
}
