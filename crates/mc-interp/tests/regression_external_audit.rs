//! رگرسیون‌های گزارش ممیزی خارجی روی P1-v0.6 (بندهای ۱، ۳، ۴، ۷، ۱۵ + اکوی
//! compound/return-داخل-حلقه از batch 1.5) —
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

// ---------- بند ۳ (batch 1.5): اکوی متغیر برهنه با «نام =» + ست‌شدن ans ----------

#[test]
fn bare_variable_echo_uses_var_name_and_sets_ans() {
    let (env, texts, err) = run("x = 5; x");
    assert!(err.is_none(), "engine error: {err:?}");
    assert_eq!(texts, vec!["x =\n     5\n".to_string()]);
    // متلب: ارجاع برهنه هم ans را ست می‌کند
    let (_, _, d) = mat(&env, "ans");
    assert_eq!(d, vec![5.0]);
}

#[test]
fn bare_variable_suppressed_does_not_set_ans() {
    let (_, texts, err) = run("x = 5; x; ans");
    // متلب: «x;» سرکوب‌شده ans را ست نمی‌کند ← ans تعریف‌نشده می‌ماند
    assert!(err.is_some(), "ans must stay undefined after 'x;'");
    assert!(texts.is_empty());
}

// ---------- batch 1.5: اکوی گزاره‌های داخل بلاک‌های compound ----------

#[test]
fn if_branch_assignment_echoes_like_matlab() {
    let (env, texts, err) = run("if [], a = 1, else, a = 0, end");
    assert!(err.is_none(), "engine error: {err:?}");
    assert_eq!(texts, vec!["a =\n     0\n".to_string()]);
    let (_, _, d) = mat(&env, "a");
    assert_eq!(d, vec![0.0]);
}

#[test]
fn for_body_echoes_each_iteration() {
    let (_, texts, err) = run("for k = 1:3, k, end");
    assert!(err.is_none(), "engine error: {err:?}");
    assert_eq!(texts.len(), 3, "each iteration echoes: {texts:?}");
    assert!(texts[0].starts_with("k ="));
    assert!(texts[2].contains('3'));
}

#[test]
fn while_body_echoes_each_iteration() {
    let (_, texts, err) = run("n = 0; while n < 2, n = n + 1, end");
    assert!(err.is_none(), "engine error: {err:?}");
    assert_eq!(texts.len(), 2, "each iteration echoes: {texts:?}");
    assert!(texts[1].contains('2'));
}

#[test]
fn return_inside_for_loop_exits_early() {
    // سمانتیک متلب: return داخل حلقه فوراً از تابع/اسکریپت خارج می‌شود
    let (env, _, err) = run("for k = 1:3, if k == 2, return, end, end");
    assert!(err.is_none(), "engine error: {err:?}");
    let (_, _, d) = mat(&env, "k");
    assert_eq!(d, vec![2.0], "loop must stop at k == 2, not run to 3");
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

// ---------- بند ۵ (ROADMAP 2.4): جهت‌گیری نتیجهٔ اندیس‌گذاری ----------
// متلب: نتیجهٔ اندیس‌گذاری خطی از «جهتِ خود اندیس» پیروی می‌کند.

#[test]
fn row_vector_reversed_by_row_index_stays_row() {
    // a(end:-1:1) روی بردار سطری سطری می‌ماند (خود موردِ ممیزی)
    let (env, _, err) = run("a = 1:5; b = a(end:-1:1);");
    assert!(err.is_none(), "engine error: {err:?}");
    let (r, c, d) = mat(&env, "b");
    assert_eq!((r, c), (1, 5));
    assert_eq!(d, vec![5.0, 4.0, 3.0, 2.0, 1.0]);
}

#[test]
fn row_vector_indexed_by_column_index_becomes_column() {
    let (env, _, err) = run("a = 1:5; b = a([1;3]);");
    assert!(err.is_none(), "engine error: {err:?}");
    let (r, c, d) = mat(&env, "b");
    assert_eq!((r, c), (2, 1));
    assert_eq!(d, vec![1.0, 3.0]);
}

#[test]
fn column_vector_indexed_by_column_index_stays_column() {
    let (env, _, err) = run("a = [1;2;3;4;5]; b = a([2;4]);");
    assert!(err.is_none(), "engine error: {err:?}");
    let (r, c, d) = mat(&env, "b");
    assert_eq!((r, c), (2, 1));
    assert_eq!(d, vec![2.0, 4.0]);
}

#[test]
fn column_vector_indexed_by_row_index_becomes_row() {
    let (env, _, err) = run("a = [1;2;3;4;5]; b = a([2 4]);");
    assert!(err.is_none(), "engine error: {err:?}");
    let (r, c, d) = mat(&env, "b");
    assert_eq!((r, c), (1, 2));
    assert_eq!(d, vec![2.0, 4.0]);
}

#[test]
fn matrix_row_subscript_with_scalar_col_follows_index_orientation() {
    let (env, _, err) = run("A = [1 2; 3 4; 5 6]; r = A([1 2], 1); c = A([1;2], 1);");
    assert!(err.is_none(), "engine error: {err:?}");
    let (r, c, d) = mat(&env, "r");
    assert_eq!((r, c), (1, 2), "A([1 2],1) is 1x2 row (MATLAB)");
    assert_eq!(d, vec![1.0, 3.0]);
    let (r, c, d) = mat(&env, "c");
    assert_eq!((r, c), (2, 1), "A([1;2],1) is 2x1 column (MATLAB)");
    assert_eq!(d, vec![1.0, 3.0]);
}

#[test]
fn matrix_col_subscript_with_scalar_row_follows_index_orientation() {
    let (env, _, err) = run("A = [1 2; 3 4; 5 6]; r = A(1, [1 2]); c = A(1, [1;2]);");
    assert!(err.is_none(), "engine error: {err:?}");
    let (r, c, d) = mat(&env, "r");
    assert_eq!((r, c), (1, 2), "A(1,[1 2]) is 1x2 row (MATLAB)");
    assert_eq!(d, vec![1.0, 2.0]);
    let (r, c, d) = mat(&env, "c");
    assert_eq!((r, c), (2, 1), "A(1,[1;2]) is 2x1 column (MATLAB)");
    assert_eq!(d, vec![1.0, 2.0]);
}

#[test]
fn matrix_two_vector_subscripts_keep_lengths_shape() {
    let (env, _, err) = run("A = [1 2; 3 4; 5 6]; B = A([3 1], [2 1]);");
    assert!(err.is_none(), "engine error: {err:?}");
    let (r, c, d) = mat(&env, "B");
    assert_eq!((r, c), (2, 2));
    // B = [6 5; 2 1] — ستونی: [6, 2, 5, 1]
    assert_eq!(d, vec![6.0, 2.0, 5.0, 1.0]);
}

#[test]
fn all_colon_shapes_unchanged_by_orientation_rule() {
    let (env, _, err) = run("A = [1 2; 3 4; 5 6]; r = A([1 3], :); c = A(:, [2]);");
    assert!(err.is_none(), "engine error: {err:?}");
    let (r, c, d) = mat(&env, "r");
    assert_eq!((r, c), (2, 2));
    assert_eq!(d, vec![1.0, 5.0, 2.0, 6.0]);
    let (r, c, d) = mat(&env, "c");
    assert_eq!((r, c), (3, 1));
    assert_eq!(d, vec![2.0, 4.0, 6.0]);
}

#[test]
fn scalar_linear_read_is_1x1_and_index_matrix_gives_index_shape() {
    let (env, _, err) = run("a = 1:5; s = a(3); B = a([1 2; 3 4]);");
    assert!(err.is_none(), "engine error: {err:?}");
    let (r, c, d) = mat(&env, "s");
    assert_eq!((r, c), (1, 1));
    assert_eq!(d, vec![3.0]);
    // اندیس ماتریسی 2×2 → نتیجه 2×2؛ چون a(i)=i، نتیجه خودِ ماتریس اندیس است
    // ([1 2;3 4]) — ستونی: [1, 3, 2, 4]
    let (r, c, d) = mat(&env, "B");
    assert_eq!((r, c), (2, 2));
    assert_eq!(d, vec![1.0, 3.0, 2.0, 4.0]);
}

#[test]
fn empty_index_shape_flows_to_result() {
    // a(1:0) → 1×0 خالی سطری (بند ۱۰ هم همین جهت را نمایش می‌دهد)
    let (env, _, err) = run("a = 1:5; b = a(1:0);");
    assert!(err.is_none(), "engine error: {err:?}");
    let (r, c, _) = mat(&env, "b");
    assert_eq!((r, c), (1, 0));
}
