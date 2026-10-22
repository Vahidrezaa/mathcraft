//! رگرسیون‌های گزارش ممیزی خارجی روی P1-v0.6 (بندهای ۱، ۴، ۷، ۱۵) —
//! رفتار مرجع: متلب R2026b. همهٔ موارد قبل از فیکس، بازتولیدِ خرابی بودند.

use mc_core::Value;
use mc_interp::{eval_program, Environment};
use mc_parser::parse;

/// اجرا و بازگرداندن (env، متن خروجی‌ها، خطا).
fn run(src: &str) -> (Environment, Vec<String>, Option<mc_core::McError>) {
    let prog = parse(src).expect("parse");
    let mut env = Environment::new();
    let out = eval_program(&prog, &mut env);
    let texts: Vec<String> = out.outputs.iter().map(|o| o.text.clone()).collect();
    (env, texts, out.error)
}

/// ماتریس متغیر به‌صورت (rows, cols, دادهٔ ستونی).
fn mat(env: &Environment, name: &str) -> (usize, usize, Vec<f64>) {
    match env.get(name) {
        Some(Value::Double(m)) => {
            let data = (0..m.numel()).map(|k| m.get_linear(k)).collect();
            (m.rows(), m.cols(), data)
        }
        other => panic!("'{name}' is not a Double matrix: {other:?}"),
    }
}

// ---------- بند ۱: حذف با انتساب خالی (قبلاً panic در matrix.rs:74) ----------

#[test]
fn delete_row_vector_element_keeps_orientation() {
    let (env, _, err) = run("x=1:5; x(2)=[];");
    assert!(err.is_none(), "engine error: {err:?}");
    let (r, c, d) = mat(&env, "x");
    assert_eq!((r, c), (1, 4));
    assert_eq!(d, vec![1.0, 3.0, 4.0, 5.0]);
}

#[test]
fn delete_col_vector_element_keeps_orientation() {
    let (env, _, err) = run("x=[1;2;3;4;5]; x(2)=[];");
    assert!(err.is_none(), "engine error: {err:?}");
    let (r, c, d) = mat(&env, "x");
    assert_eq!((r, c), (4, 1));
    assert_eq!(d, vec![1.0, 3.0, 4.0, 5.0]);
}

#[test]
fn delete_multiple_elements() {
    let (env, _, err) = run("x=1:5; x([2 4])=[];");
    assert!(err.is_none(), "engine error: {err:?}");
    let (r, c, d) = mat(&env, "x");
    assert_eq!((r, c), (1, 3));
    assert_eq!(d, vec![1.0, 3.0, 5.0]);
}

#[test]
fn delete_matrix_row_via_linear_index() {
    let (env, _, err) = run("A=[1 2;3 4;5 6]; A(2)=[];");
    assert!(err.is_none(), "engine error: {err:?}");
    let (r, c, d) = mat(&env, "A");
    assert_eq!((r, c), (2, 2));
    assert_eq!(d, vec![1.0, 5.0, 2.0, 6.0]); // ستونی: [1 5; 2 6]
}

#[test]
fn delete_column_with_colon() {
    let (env, _, err) = run("A=[1 2 3;4 5 6]; A(:,2)=[];");
    assert!(err.is_none(), "engine error: {err:?}");
    let (r, c, d) = mat(&env, "A");
    assert_eq!((r, c), (2, 2));
    assert_eq!(d, vec![1.0, 4.0, 3.0, 6.0]);
}

#[test]
fn delete_row_with_colon() {
    let (env, _, err) = run("A=[1 2 3;4 5 6]; A(2,:)=[];");
    assert!(err.is_none(), "engine error: {err:?}");
    let (r, c, d) = mat(&env, "A");
    assert_eq!((r, c), (1, 3));
    assert_eq!(d, vec![1.0, 2.0, 3.0]);
}

#[test]
fn delete_all_with_colon_becomes_empty() {
    let (env, _, err) = run("x=1:3; x(:)=[];");
    assert!(err.is_none(), "engine error: {err:?}");
    let (r, c, _) = mat(&env, "x");
    assert_eq!((r, c), (0, 0)); // متلب/اکتاو: 0×0 خالی
}

#[test]
fn delete_with_empty_index_is_noop() {
    let (env, _, err) = run("x=1:3; x([])=[];");
    assert!(err.is_none(), "engine error: {err:?}");
    let (r, c, d) = mat(&env, "x");
    assert_eq!((r, c), (1, 3));
    assert_eq!(d, vec![1.0, 2.0, 3.0]);
}

#[test]
fn delete_2d_without_colon_is_honest_error() {
    let (_, _, err) = run("A=[1 2;3 4]; A(1,2)=[];");
    assert!(err.is_some(), "must error instead of partial semantics");
}

#[test]
fn delete_out_of_range_is_error_not_panic() {
    let (_, _, err) = run("x=1:3; x(9)=[];");
    assert!(err.is_some());
}

#[test]
fn growth_assignment_still_works_after_deletion_path() {
    let (env, _, err) = run("x=10; x(3)=5;");
    assert!(err.is_none(), "engine error: {err:?}");
    let (r, c, d) = mat(&env, "x");
    assert_eq!((r, c), (1, 3));
    assert_eq!(d, vec![10.0, 0.0, 5.0]);
}

// ---------- بند ۴: نمایش انتساب اندیس‌دار بدون «;» ----------

#[test]
fn indexed_assignment_without_semicolon_displays() {
    let (_, texts, err) = run("a=[1 2 3]; a(2)=5");
    assert!(err.is_none(), "engine error: {err:?}");
    assert!(
        texts.iter().any(|t| t.contains("a =") && t.contains('5')),
        "indexed assignment must display 'a =': {texts:?}"
    );
}

#[test]
fn deletion_without_semicolon_displays_too() {
    let (_, texts, err) = run("x=1:5; x(2)=[]");
    assert!(err.is_none(), "engine error: {err:?}");
    assert!(
        texts.iter().any(|t| t.contains("x =")),
        "deletion must display 'x =': {texts:?}"
    );
}

// ---------- بند ۷: شرط خالی/آرایه‌ای if-while (سمانتیک متلب) ----------

#[test]
fn if_empty_condition_takes_else() {
    let (env, _, err) = run("if [], y=1; else, y=0; end");
    assert!(err.is_none(), "engine error: {err:?}");
    let (_, _, d) = mat(&env, "y");
    assert_eq!(d, vec![0.0]);
}

#[test]
fn if_matrix_all_nonzero_is_true() {
    let (env, _, err) = run("if [2 3], y=1; end");
    assert!(err.is_none(), "engine error: {err:?}");
    let (_, _, d) = mat(&env, "y");
    assert_eq!(d, vec![1.0]);
}

#[test]
fn if_matrix_with_zero_is_false() {
    let (env, _, err) = run("if [2 0], y=1; else, y=0; end");
    assert!(err.is_none(), "engine error: {err:?}");
    let (_, _, d) = mat(&env, "y");
    assert_eq!(d, vec![0.0]);
}

#[test]
fn if_nan_is_nonzero_true() {
    let (env, _, err) = run("if NaN, y=1; end");
    assert!(err.is_none(), "engine error: {err:?}");
    let (_, _, d) = mat(&env, "y");
    assert_eq!(d, vec![1.0]);
}

#[test]
fn while_empty_never_runs() {
    let (env, _, err) = run("n=0; while [], n=n+1; end");
    assert!(err.is_none(), "engine error: {err:?}");
    let (_, _, d) = mat(&env, "n");
    assert_eq!(d, vec![0.0]);
}

#[test]
fn logical_and_still_requires_scalar() {
    let (_, _, err) = run("if [1 1] && [0 0], y=1; end");
    assert!(err.is_some(), "&&/|| must stay scalar-only (MATLAB)");
}
