//! تست‌های پاریتی .mat v5 روی فیکسچرهای کامیت‌شده (ROADMAP 3.7 DoD).
//!
//! فیکسچرها با scipy.io.savemat 1.14 (اسپک Level 5) تولید شده‌اند —
//! `scripts/gen_mat_fixtures.py` (پروونانس). اوراکل CI = بازتولید همان محتوا
//! با Octave در جاب mat-parity و مقایسهٔ ارزشی از راه همان این تست‌ها.

use mc_core::matrix::Matrix;
use mc_core::value::Value;
use mc_io::matv5::read_mat;

macro_rules! load {
    ($f:expr) => {{
        let bytes: &[u8] = include_bytes!(concat!("fixtures/", $f));
        read_mat(bytes)
    }};
}

fn as_double(v: &Value) -> &Matrix {
    let Value::Double(m) = v else {
        panic!("expected double, got {v:?}")
    };
    m
}

#[test]
fn double_2x3_uncompressed() {
    let vars = load!("dbl_2x3.mat").expect("read");
    assert_eq!(vars.len(), 1);
    assert_eq!(vars[0].0, "M");
    let m = as_double(&vars[0].1);
    assert_eq!((m.rows(), m.cols()), (2, 3));
    // ستونی: [1 4 | 2 5 | 3 6]
    assert_eq!(m.as_column_slice(), &[1.0, 4.0, 2.0, 5.0, 3.0, 6.0]);
}

#[test]
fn double_2x2_compressed() {
    let vars = load!("dbl_2x2_comp.mat").expect("read");
    let m = as_double(&vars[0].1);
    assert_eq!(m.as_column_slice(), &[1.0, 3.0, 2.0, 4.0]);
}

#[test]
fn complex_2x2() {
    let vars = load!("cplx_2x2.mat").expect("read");
    assert_eq!(vars[0].0, "C");
    let Value::Complex(c) = &vars[0].1 else {
        panic!("expected complex")
    };
    assert_eq!((c.rows(), c.cols()), (2, 2));
    assert_eq!(c.re_column_slice(), &[1.0, 3.5, -0.0, 0.0]);
    assert_eq!(c.im_column_slice(), &[2.0, -4.0, -1.0, 0.5]);
}

#[test]
fn char_row() {
    let vars = load!("char_row.mat").expect("read");
    assert_eq!(vars[0].0, "s");
    assert_eq!(vars[0].1, Value::Char("hello".to_owned()));
}

#[test]
fn struct_1x1_nested() {
    let vars = load!("struct_1x1.mat").expect("read");
    let Value::Struct(st) = &vars[0].1 else {
        panic!("expected struct")
    };
    let a = st.get("a").expect("field a");
    assert_eq!(*as_double(a), Matrix::scalar(1.25));
    assert_eq!(st.get("b"), Some(&Value::Char("txt".to_owned())));
    let Value::Struct(inner) = st.get("inner").expect("field inner") else {
        panic!("inner struct")
    };
    let x = as_double(inner.get("x").expect("field x"));
    assert_eq!((x.rows(), x.cols()), (2, 1));
    assert_eq!(x.as_column_slice(), &[1.0, 2.0]);
}

#[test]
fn logical_becomes_double_zero_one() {
    // دلتای مستند: logical → double 0/1 (نوع logical زبان نیست)
    let vars = load!("logical_1x3.mat").expect("read");
    let m = as_double(&vars[0].1);
    assert_eq!(m.as_column_slice(), &[1.0, 0.0, 1.0]);
}

#[test]
fn int32_and_single_cast_to_double() {
    let vars = load!("int32_col.mat").expect("read");
    assert_eq!(as_double(&vars[0].1).as_column_slice(), &[7.0, -8.0, 9.0]);
    let vars = load!("single_2x2.mat").expect("read");
    assert_eq!(
        as_double(&vars[0].1).as_column_slice(),
        &[0.5, -2.5, 1.5, 3.5]
    );
}

#[test]
fn empty_and_special_bit_patterns() {
    let vars = load!("empty_0x0.mat").expect("read");
    assert_eq!(as_double(&vars[0].1).numel(), 0);
    let vars = load!("nan_inf.mat").expect("read");
    let m = as_double(&vars[0].1);
    assert!(m.get(0, 0).is_nan());
    assert_eq!(m.get(0, 1), f64::INFINITY);
    assert_eq!(m.get(1, 0), f64::NEG_INFINITY);
    assert!(m.get(1, 1) == 0.0 && m.get(1, 1).is_sign_negative());
}

#[test]
fn v73_hdf5_rejected_with_guidance() {
    let err = load!("fake_v73.mat").unwrap_err();
    assert!(err.message_en.contains("v7.3"), "{}", err.message_en);
    assert!(err.message_fa.contains("v7.3"));
    assert!(err.message_en.contains("save -v7"));
}

#[test]
fn big_endian_rejected() {
    assert!(load!("big_endian.mat").is_err());
}

#[test]
fn sparse_rejected_honestly() {
    let err = load!("sparse_3x3.mat").unwrap_err();
    assert!(err.message_en.contains("sparse"), "{}", err.message_en);
}

#[test]
fn cell_rejected_honestly() {
    let err = load!("cell_1x2.mat").unwrap_err();
    assert!(err.message_en.contains("cell"), "{}", err.message_en);
}

#[test]
fn garbage_rejected() {
    // کوتاه‌تر از هدر / هدر بی‌معنا / بیت‌های تصادفی — همه خرابی صادقانه
    assert!(read_mat(b"short").is_err());
    assert!(read_mat(&[0u8; 128]).is_err());
    let mut noise = vec![0xFFu8; 256];
    noise[124..126].copy_from_slice(&[0x00, 0x01]); // version معتبر، بقیه هرزه
    noise[126..128].copy_from_slice(b"IM");
    assert!(read_mat(&noise).is_err());
}
