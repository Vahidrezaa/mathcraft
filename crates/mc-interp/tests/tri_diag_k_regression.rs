//! رگرسیون سازگاری متلب — قطر k-اُم در tril/triu (کشف هنگام ROADMAP 2.6).
//!
//! متلب (R2026b doc): قطر k-اُم، k>0 بالای قطر اصلی و k<0 پایینِ آن:
//! - `tril(A,k)` = عناصر روی و زیرِ قطر k-اُم (i−j ≥ −k)
//! - `triu(A,k)` = عناصر روی و بالای قطر k-اُم (i−j ≤ −k)
//!
//! باگ قبلی: علامت k برعکس بود — `tril(A,1)` خروجی `tril(A,−1)` متلب را می‌داد.
//! قاعدهٔ R2 سند ۰۷: هر باگ سازگاری یافته‌شده = تست رگرسیون دائمی.

use mc_core::{Matrix, Value};
use mc_interp::{eval_program, Environment};
use mc_parser::parse;

fn eval_matrix(src: &str, var: &str) -> Matrix {
    let prog = parse(src).expect("parse");
    let mut env = Environment::new();
    let out = eval_program(&prog, &mut env);
    assert!(out.error.is_none(), "engine error: {:?}", out.error);
    match env.get(var) {
        Some(Value::Double(m)) => m.clone(),
        other => panic!("'{var}' is not a double matrix: {other:?}"),
    }
}

fn assert_eq_matrix(got: &Matrix, want: &[&[f64]], what: &str) {
    let (r, c) = (got.rows(), got.cols());
    assert_eq!((r, c), (want.len(), want[0].len()), "{what} shape");
    for (i, row) in want.iter().enumerate() {
        for (j, w) in row.iter().enumerate() {
            assert_eq!(got.get(i, j), *w, "{what} at ({i},{j})");
        }
    }
}

#[test]
fn tril_k_positive_is_above_main_diagonal_like_matlab() {
    let m = eval_matrix("A = [1 2 3; 4 5 6; 7 8 9];\nz = tril(A, 1);\n", "z");
    assert_eq_matrix(
        &m,
        &[&[1.0, 2.0, 0.0], &[4.0, 5.0, 6.0], &[7.0, 8.0, 9.0]],
        "tril(A,1)",
    );
}

#[test]
fn tril_k_negative_is_strictly_below_like_matlab() {
    let m = eval_matrix("A = [1 2 3; 4 5 6; 7 8 9];\nz = tril(A, -1);\n", "z");
    assert_eq_matrix(
        &m,
        &[&[0.0, 0.0, 0.0], &[4.0, 0.0, 0.0], &[7.0, 8.0, 0.0]],
        "tril(A,-1)",
    );
}

#[test]
fn triu_k_positive_is_strictly_above_like_matlab() {
    let m = eval_matrix("A = [1 2 3; 4 5 6; 7 8 9];\nz = triu(A, 1);\n", "z");
    assert_eq_matrix(
        &m,
        &[&[0.0, 2.0, 3.0], &[0.0, 0.0, 6.0], &[0.0, 0.0, 0.0]],
        "triu(A,1)",
    );
}

#[test]
fn triu_k_negative_is_below_main_diagonal_like_matlab() {
    let m = eval_matrix("A = [1 2 3; 4 5 6; 7 8 9];\nz = triu(A, -1);\n", "z");
    assert_eq_matrix(
        &m,
        &[&[1.0, 2.0, 3.0], &[4.0, 5.0, 6.0], &[0.0, 8.0, 9.0]],
        "triu(A,-1)",
    );
}

#[test]
fn tri_default_k_zero_unchanged() {
    // گارد رگرسیون: شکل بدون k (k=0) نباید تغییری کند
    let m = eval_matrix("A = [1 2; 3 4];\nz = tril(A);\n", "z");
    assert_eq_matrix(&m, &[&[1.0, 0.0], &[3.0, 4.0]], "tril(A)");
    let m = eval_matrix("A = [1 2; 3 4];\nz = triu(A);\n", "z");
    assert_eq_matrix(&m, &[&[1.0, 2.0], &[0.0, 4.0]], "triu(A)");
}
