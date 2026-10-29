//! چندخروجی‌های builtin — [Q,R] = qr و [U,S,V] = svd (ROADMAP 2.6، تسک ۱).
//!
//! قرارداد متلب (R2026b مرجع COMPATIBILITY):
//! - `[Q,R] = qr(A)`: Q یکانی m×m، R مثلثی بالایی m×n؛ `qr(A)` تک‌خروجی = فقط R.
//! - `[U,S,V] = svd(A)`: U یکانی m×m، S قطری m×n، V یکانی n×n؛
//!   `svd(A)` تک‌خروجی = بردار ستونی σها به‌ترتیب نزولی.
//!
//! علامت ستون‌های Q/U/V پیاده‌سازی‌وابسته است (متلب/Octave از LAPACK، ما از
//! faer) — پس فقط ناورداها اورگاشته می‌شوند: A = Q·R، ارتونالیتی، مثلثی‌بودن
//! R، نزولی‌بودن σ ≥ 0. این همان استراتژی کورپوس p2_14/p2_15 است.

use mc_core::{Matrix, Value};
use mc_interp::{eval_program, Environment};
use mc_parser::parse;

fn eval_script(src: &str) -> Environment {
    let prog = parse(src).expect("parse");
    let mut env = Environment::new();
    let out = eval_program(&prog, &mut env);
    assert!(out.error.is_none(), "engine error: {:?}", out.error);
    env
}

fn mat(env: &Environment, name: &str) -> Matrix {
    match env.get(name) {
        Some(Value::Double(m)) => m.clone(),
        other => panic!("'{name}' is not a double matrix: {other:?}"),
    }
}

fn scalar(env: &Environment, name: &str) -> f64 {
    mat(env, name).get(0, 0)
}

fn dims(m: &Matrix) -> (usize, usize) {
    (m.rows(), m.cols())
}

fn assert_close(a: f64, b: f64, tol: f64, what: &str) {
    assert!(
        (a - b).abs() <= tol * b.abs().max(1.0),
        "{what}: {a} vs {b} (tol {tol})"
    );
}

#[test]
fn qr_multi_outputs_match_matlab_contract() {
    let env = eval_script(
        "A = [1 2; 3 4; 5 6];\n\
         [Q, R] = qr(A);\n\
         errA = max(max(abs(Q*R - A)));\n\
         errQ = max(max(abs(Q'*Q - eye(3))));\n\
         lower = max(max(abs(tril(R, -1))));\n\
         szQ = size(Q);\n\
         szR = size(R);\n",
    );
    let q = mat(&env, "Q");
    let r = mat(&env, "R");
    assert_eq!(dims(&q), (3, 3), "Q must be m×m");
    assert_eq!(dims(&r), (3, 2), "R must be m×n");
    assert_close(scalar(&env, "errA"), 0.0, 1e-12, "Q*R = A");
    assert_close(scalar(&env, "errQ"), 0.0, 1e-12, "Q'*Q = I");
    assert_close(scalar(&env, "lower"), 0.0, 1e-15, "R upper triangular");
    let szq = mat(&env, "szQ");
    let szr = mat(&env, "szR");
    assert_eq!((szq.get_linear(0), szq.get_linear(1)), (3.0, 3.0));
    assert_eq!((szr.get_linear(0), szr.get_linear(1)), (3.0, 2.0));
}

#[test]
fn qr_wide_multi_outputs_match_matlab_contract() {
    let env = eval_script(
        "B = [1 2 3; 4 5 6];\n\
         [Q, R] = qr(B);\n\
         errB = max(max(abs(Q*R - B)));\n\
         errQ = max(max(abs(Q'*Q - eye(2))));\n\
         szR = size(R);\n",
    );
    assert_eq!(dims(&mat(&env, "R")), (2, 3), "wide R must be m×n");
    assert_close(scalar(&env, "errB"), 0.0, 1e-12, "Q*R = B");
    assert_close(scalar(&env, "errQ"), 0.0, 1e-12, "Q'*Q = I");
    let szr = mat(&env, "szR");
    assert_eq!((szr.get_linear(0), szr.get_linear(1)), (2.0, 3.0));
}

#[test]
fn qr_single_output_is_r_like_matlab() {
    let env = eval_script(
        "A = [1 2; 3 4; 5 6];\n\
         r1 = qr(A);\n\
         [Q2, R2] = qr(A);\n\
         d = max(max(abs(r1 - R2)));\n",
    );
    assert_close(scalar(&env, "d"), 0.0, 0.0, "single-output qr = R");
}

#[test]
fn svd_multi_outputs_match_matlab_contract() {
    // A = [3 0; 0 2; 0 0] — σها تحلیلی: [3; 2]
    let env = eval_script(
        "A = [3 0; 0 2; 0 0];\n\
         [U, S, V] = svd(A);\n\
         s1 = svd(A);\n\
         errA = max(max(abs(U*S*V' - A)));\n\
         errU = max(max(abs(U'*U - eye(3))));\n\
         errV = max(max(abs(V'*V - eye(2))));\n\
         szS = size(S);\n",
    );
    let u = mat(&env, "U");
    let s = mat(&env, "S");
    let v = mat(&env, "V");
    assert_eq!(dims(&u), (3, 3), "U must be m×m");
    assert_eq!(dims(&s), (3, 2), "S must be m×n diagonal");
    assert_eq!(dims(&v), (2, 2), "V must be n×n");
    assert_close(s.get(0, 0), 3.0, 1e-12, "sigma_1");
    assert_close(s.get(1, 1), 2.0, 1e-12, "sigma_2");
    assert_close(s.get(0, 1), 0.0, 1e-15, "S off-diagonal");
    assert_close(s.get(2, 0), 0.0, 1e-15, "S off-diagonal");
    assert_close(scalar(&env, "errA"), 0.0, 1e-12, "U*S*V' = A");
    assert_close(scalar(&env, "errU"), 0.0, 1e-12, "U'*U = I");
    assert_close(scalar(&env, "errV"), 0.0, 1e-12, "V'*V = I");
    // تک‌خروجی = بردار ستونی σها (نزولی)
    let svec = mat(&env, "s1");
    assert_eq!(dims(&svec), (2, 1), "svd(A) single output is a column");
    assert_close(svec.get(0, 0), 3.0, 1e-12, "s1 descending");
    assert_close(svec.get(1, 0), 2.0, 1e-12, "s1 descending");
    let szs = mat(&env, "szS");
    assert_eq!((szs.get_linear(0), szs.get_linear(1)), (3.0, 2.0));
}

#[test]
fn svd_multi_wide_and_descending_order() {
    // B = [1 4 7; 2 5 8] — σها نزولی؛ S دوزنقه‌ای 2×3
    let env = eval_script(
        "B = [1 4 7; 2 5 8];\n\
         [U, S, V] = svd(B);\n\
         s = svd(B);\n\
         errB = max(max(abs(U*S*V' - B)));\n\
         descending = s(1) >= s(2);\n\
         szS = size(S);\n",
    );
    assert_eq!(dims(&mat(&env, "S")), (2, 3), "wide S must be m×n");
    assert_close(scalar(&env, "errB"), 0.0, 1e-12, "U*S*V' = B");
    assert_close(scalar(&env, "descending"), 1.0, 0.0, "sigma descending");
    let szs = mat(&env, "szS");
    assert_eq!((szs.get_linear(0), szs.get_linear(1)), (2.0, 3.0));
}

#[test]
fn svd_single_output_is_column_vector_like_matlab() {
    let env = eval_script(
        "A = [3 0; 0 2; 0 0];\n\
         [U, S, V] = svd(A);\n\
         s1 = svd(A);\n\
         d = max(max(abs(s1 - [S(1,1); S(2,2)])));\n",
    );
    assert_close(scalar(&env, "d"), 0.0, 1e-12, "single svd = diag(S)");
}
