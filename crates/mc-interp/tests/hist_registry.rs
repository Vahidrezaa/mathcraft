//! histcounts / histogram — تاریخ‌بندی متلب (ROADMAP 2.6، تسک ۳).
//!
//! پوشش لایهٔ رجیستری: فرم‌های histcounts (خودکار / nbins / مرزهای صریح) و
//! گزینه‌های histogram (BinMethod/BinWidth/BinLimits)، خروجی‌های چندگانه
//! [N,edges,bin]، مرز بستهٔ راست، نادیده‌گرفتن NaN/±Inf، خطاهای صادقانه.
//! مرجع «auto» = قاعدهٔ مستند خود موتور (اسکات×2^(1/3) + گرد خوش‌فرم) —
//! پین با MATLAB واقعی کاربر طبق COMPATIBILITY §4.5.

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

fn row_of(env: &Environment, name: &str) -> Vec<f64> {
    let m = mat(env, name);
    assert_eq!(m.rows(), 1, "{name} must be a row");
    (0..m.cols()).map(|c| m.get(0, c)).collect()
}

fn expect_error(src: &str, what: &str) {
    let prog = parse(src).expect(what);
    let mut env = Environment::new();
    let out = eval_program(&prog, &mut env);
    assert!(out.error.is_some(), "expected honest error for {what}");
}

#[test]
fn histcounts_explicit_edges_matlab_semantics() {
    let env = eval_script(
        "x = [0.5 0.8 1.2 1.9 2.3 3.0];\n\
         [N, edges] = histcounts(x, [0 1 2 3]);\n",
    );
    assert_eq!(
        row_of(&env, "N"),
        vec![2.0, 2.0, 2.0],
        "last bin right-closed"
    );
    assert_eq!(row_of(&env, "edges"), vec![0.0, 1.0, 2.0, 3.0]);
}

#[test]
fn histcounts_interior_boundary_counts_right() {
    let env = eval_script("x = [0 1 2];\nN = histcounts(x, [0 1 2]);\n");
    assert_eq!(row_of(&env, "N"), vec![1.0, 2.0], "x == edges(k) → bin k");
}

#[test]
fn histcounts_nbins_form_uniform_edges() {
    let env = eval_script("x = [1 2 3 4];\n[N, edges] = histcounts(x, 3);\n");
    assert_eq!(row_of(&env, "N"), vec![1.0, 1.0, 2.0]);
    assert_eq!(row_of(&env, "edges"), vec![1.0, 2.0, 3.0, 4.0]);
}

#[test]
fn histcounts_three_outputs_bin_indices() {
    let env = eval_script(
        "x = [-1 0.5 0.7 2.0];\n\
         [N, edges, bin] = histcounts(x, [0 1 2]);\n",
    );
    assert_eq!(row_of(&env, "N"), vec![2.0, 1.0]);
    // bin هم‌شکل X: بیرون = 0، داخل = اندیس بین (۱-پایه)
    assert_eq!(row_of(&env, "bin"), vec![0.0, 1.0, 1.0, 2.0]);
}

#[test]
fn histcounts_nan_inf_ignored() {
    let env = eval_script(
        "x = [NaN Inf 0.5 -Inf 0.2];\n\
         N = histcounts(x, [0 1]);\n",
    );
    assert_eq!(row_of(&env, "N"), vec![2.0], "only finite in-range values");
}

#[test]
fn histcounts_auto_row_outputs() {
    let env = eval_script(
        "x = [1 2 3 4 5];\n\
         [N, edges] = histcounts(x);\n",
    );
    // قاعدهٔ auto مستند موتور (مثال پین‌شده در تست‌های mc-num): مرز [0 5]
    assert_eq!(row_of(&env, "edges"), vec![0.0, 5.0]);
    assert_eq!(row_of(&env, "N"), vec![5.0]);
}

#[test]
fn histogram_binwidth_option() {
    let env = eval_script(
        "x = [0.2 0.7 1.1];\n\
         [N, edges] = histogram(x, 'BinWidth', 0.5);\n",
    );
    assert_eq!(row_of(&env, "edges"), vec![0.0, 0.5, 1.0, 1.5]);
    assert_eq!(row_of(&env, "N"), vec![1.0, 1.0, 1.0]);
}

#[test]
fn histogram_binlimits_option() {
    let env = eval_script(
        "x = [-5 -0.5 0.2 0.9 5];\n\
         [N, edges] = histogram(x, 'BinLimits', [-1 1], 'BinWidth', 0.5);\n",
    );
    // مرزها با محدوده: [-1 -0.5 0 0.5 1]؛ دادهٔ بیرون محدوده شمرده نمی‌شود؛
    // -0.5 دقیقاً روی مرز میانی → بین سمت راست (قرارداد متلب edges(k) ≤ x)
    assert_eq!(row_of(&env, "edges"), vec![-1.0, -0.5, 0.0, 0.5, 1.0]);
    assert_eq!(row_of(&env, "N"), vec![0.0, 1.0, 1.0, 1.0]);
}

#[test]
fn histogram_binmethod_option() {
    let env = eval_script(
        "x = 1:32;\n\
         [N, edges] = histogram(x, 'BinMethod', 'sturges');\n",
    );
    assert_eq!(row_of(&env, "edges").len(), 7, "sturges: 6 bins for n=32");
    assert_eq!(row_of(&env, "N").iter().sum::<f64>(), 32.0);
}

#[test]
fn histogram_single_output_capture() {
    // خروجی تنها = counts (رفتار ناحیهٔ عددی؛ اکوی برهنه `ans` در لایهٔ
    // session/evaluate_line انجام می‌شود — تأیید mc-cli در DoD گام)
    let env = eval_script("x = [0.5 1.5 1.7];\nz = histogram(x, [0 1 2]);\n");
    assert_eq!(row_of(&env, "z"), vec![1.0, 2.0]);
}

#[test]
fn hist_family_honest_errors() {
    expect_error("N = histcounts([1 2], 0);", "nbins < 1");
    expect_error("N = histcounts([1 2], [2 1]);", "descending edges");
    expect_error("N = histcounts([1 2], 'auto');", "char second arg");
    expect_error("[a,b,c,d] = histcounts([1 2 3]);", "more than 3 outputs");
    expect_error(
        "h = histogram([1 2 3], 'NumBins', 4);",
        "unsupported option NumBins",
    );
    expect_error(
        "h = histogram([1 2 3], 'BinMethod', 'auto', 'scott');",
        "dangling option value",
    );
}
