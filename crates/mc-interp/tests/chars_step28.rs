//! گام 2.8 ROADMAP — char type & strings (بندهای ۶ و ۱۲ ممیزی خارجی).
//!
//! رفتارهای کلیدی با اوراکل Octave 10.3 (scripts/probe_chars.m) و سند رسمی
//! متلب char تطبیق داده شده‌اند؛ موارد انحراف‌دار مستند COMPATIBILITY.md اند.
//! دامنهٔ v1: ردیف char با ابعاد 1×N؛ آرایهٔ char دوبعدی = انبساط E4 با
//! خطای صادقانه.

use mc_core::Value;
use mc_interp::{eval_program, Environment};
use mc_parser::parse;

/// اجرای موفق — بدون خطای موتور.
fn eval_ok(src: &str) -> Environment {
    let prog = parse(src).expect("parse");
    let mut env = Environment::new();
    let out = eval_program(&prog, &mut env);
    assert!(out.error.is_none(), "engine error: {:?}", out.error);
    env
}

/// اجرای خطادار — متن انگلیسی خطا برای ادعای صادقانه بودن.
fn eval_err(src: &str) -> String {
    let prog = parse(src).expect("parse");
    let mut env = Environment::new();
    let out = eval_program(&prog, &mut env);
    match out.error {
        Some(e) => e.message_en,
        None => panic!("expected an engine error, got none"),
    }
}

/// مقدار متغیر char به‌صورت رشته.
fn char_var(env: &Environment, name: &str) -> String {
    match env.get(name) {
        Some(Value::Char(s)) => s.clone(),
        other => panic!("'{name}' is not char: {other:?}"),
    }
}

/// ردیف عددی 1×N.
fn row_var(env: &Environment, name: &str) -> Vec<f64> {
    match env.get(name) {
        Some(Value::Double(m)) if m.rows() == 1 => (0..m.cols()).map(|c| m.get(0, c)).collect(),
        other => panic!("'{name}' is not a double row: {other:?}"),
    }
}

/// اسکالر عددی.
fn scalar_var(env: &Environment, name: &str) -> f64 {
    match env.get(name) {
        Some(Value::Double(m)) if m.is_scalar() => m.get(0, 0),
        other => panic!("'{name}' is not a scalar: {other:?}"),
    }
}

// ---------- تلفیق مخلوط: عددی → char (بند ۶ ممیزی) ----------

#[test]
fn mixed_concat_promotes_to_char() {
    // رپرو بند ۶/۱۲ ممیزی: [65 'a'] → 'Aa' (کلاس char)
    let env = eval_ok("m = [65 'a'];");
    assert_eq!(char_var(&env, "m"), "Aa");
}

#[test]
fn mixed_concat_char_plus_numeric_row() {
    // اوراکل Octave: ['ab' [32 33]] → 'ab !'
    let env = eval_ok("m = ['ab' [32 33]];");
    assert_eq!(char_var(&env, "m"), "ab !");
}

#[test]
fn all_char_concat() {
    let env = eval_ok("m = ['abc' 'de'];");
    assert_eq!(char_var(&env, "m"), "abcde");
}

#[test]
fn mixed_concat_with_range_result() {
    // ['abc' 1:2] → کدهای کنترلی 1 و 2 (اوراکل Octave: [abc])
    let env = eval_ok("m = ['abc' 1:2];");
    assert_eq!(char_var(&env, "m"), "abc\u{1}\u{2}");
}

#[test]
fn concat_complex_char_is_honest_error() {
    // اوراکل Octave 10.3: wrong type argument 'complex scalar'
    let msg = eval_err("m = [1i 'a'];");
    assert!(msg.contains("Complex values cannot"), "got: {msg}");
}

#[test]
fn concat_2d_char_is_e4_error() {
    let msg = eval_err("m = ['ab';'cd'];");
    assert!(msg.contains("expansion E4"), "got: {msg}");
}

#[test]
fn concat_mixed_2d_char_is_e4_error() {
    let msg = eval_err("m = [65 'a'; 66 'b'];");
    assert!(msg.contains("expansion E4"), "got: {msg}");
}

#[test]
fn concat_numeric_only_stays_double() {
    // بدون char در اجزا نتیجه double می‌ماند (گارد رگرسیون)
    let env = eval_ok("m = [65 97];");
    assert_eq!(row_var(&env, "m"), vec![65.0, 97.0]);
}

// ---------- ارتقای عددی: char در زمینهٔ عددی (بند ۱۲ ممیزی) ----------

#[test]
fn arith_char_plus_one() {
    // رپرو بند ۱۲: 'abc'+1 → [98 99 100]
    let env = eval_ok("y = 'abc' + 1;");
    assert_eq!(row_var(&env, "y"), vec![98.0, 99.0, 100.0]);
}

#[test]
fn arith_char_plus_char() {
    let env = eval_ok("y = 'ab' + 'cd';");
    assert_eq!(row_var(&env, "y"), vec![196.0, 198.0]);
}

#[test]
fn arith_char_scalar_mult() {
    // اوراکل Octave: 'ab' * 2 → [194 196] (double)
    let env = eval_ok("y = 'ab' * 2;");
    assert_eq!(row_var(&env, "y"), vec![194.0, 196.0]);
}

#[test]
fn arith_char_elementwise_div() {
    let env = eval_ok("y = 'ab' ./ 2;");
    assert_eq!(row_var(&env, "y"), vec![48.5, 49.0]);
}

#[test]
fn arith_char_plus_complex() {
    // 'ab' + 1i → [97+1i 98+1i]
    let env = eval_ok("y = 'ab' + 1i;");
    match env.get("y") {
        Some(Value::Complex(z)) => {
            assert_eq!(z.rows(), 1);
            assert_eq!(z.cols(), 2);
            assert_eq!(z.get(0, 0), (97.0, 1.0));
            assert_eq!(z.get(0, 1), (98.0, 1.0));
        }
        other => panic!("'y' is not complex: {other:?}"),
    }
}

#[test]
fn compare_char_elementwise() {
    let env = eval_ok("q = 'abc' == 'abd';");
    assert_eq!(row_var(&env, "q"), vec![1.0, 1.0, 0.0]);
}

#[test]
fn compare_char_against_code() {
    let env = eval_ok("q = 'a' == 97;");
    assert_eq!(scalar_var(&env, "q"), 1.0);
}

#[test]
fn compare_char_less_than() {
    let env = eval_ok("q = 'a' < 'b';");
    assert_eq!(scalar_var(&env, "q"), 1.0);
}

// ---------- ترانهاده: 1×1 مجاز؛ 1×N → E4 ----------

#[test]
fn transpose_char_1x1_is_identity() {
    let env = eval_ok("y = ('a')';");
    assert_eq!(char_var(&env, "y"), "a");
}

#[test]
fn transpose_char_row_is_e4_error() {
    // ('abc')' در متلب 3×1 است = char دوبعدی → E4 صادقانه (نه معکوس خام!)
    let msg = eval_err("y = ('abc')';");
    assert!(msg.contains("expansion E4"), "got: {msg}");
}

// ---------- اندیس‌گذاری خواندنی روی char ----------

#[test]
fn index_char_scalar() {
    let env = eval_ok("s = 'hello'; y = s(2);");
    assert_eq!(char_var(&env, "y"), "e");
}

#[test]
fn index_char_vector() {
    let env = eval_ok("s = 'hello'; y = s([1 3]);");
    assert_eq!(char_var(&env, "y"), "hl");
}

#[test]
fn index_char_reverse_with_end() {
    let env = eval_ok("s = 'hello'; y = s(end:-1:1);");
    assert_eq!(char_var(&env, "y"), "olleh");
}

#[test]
fn index_char_two_subscripts() {
    let env = eval_ok("s = 'hello'; y = s(1,2);");
    assert_eq!(char_var(&env, "y"), "e");
}

#[test]
fn index_char_column_shape_is_e4_error() {
    // s([1;3]) در متلب 2×1 است = char دوبعدی → E4
    let msg = eval_err("s = 'hello'; y = s([1;3]);");
    assert!(msg.contains("expansion E4"), "got: {msg}");
}

// ---------- انتساب اندیس‌دار روی char (کلاس هدف برنده است) ----------

#[test]
fn indexed_assign_char_target() {
    let env = eval_ok("s = 'hello'; s(2) = 'x';");
    assert_eq!(char_var(&env, "s"), "hxllo");
}

#[test]
fn indexed_assign_char_target_numeric_value() {
    // 99 → 'c': کلاس هدف (char) برنده است — متلب
    let env = eval_ok("s = 'ab'; s(1) = 99;");
    assert_eq!(char_var(&env, "s"), "cb");
}

#[test]
fn indexed_assign_char_growth_zero_fill() {
    // رشد مثل متلب با char(0) پر می‌شود
    let env = eval_ok("s = 'abc'; s(5) = 'x';");
    assert_eq!(char_var(&env, "s"), "abc\u{0}x");
}

#[test]
fn indexed_assign_double_target_char_value() {
    // x(1)='a' → 97 (کلاس هدف double برنده است — متلب)
    let env = eval_ok("x = [1 2]; x(1) = 'a';");
    assert_eq!(row_var(&env, "x"), vec![97.0, 2.0]);
}

#[test]
fn indexed_assign_char_delete_element() {
    let env = eval_ok("s = 'hello'; s(2) = [];");
    assert_eq!(char_var(&env, "s"), "hllo");
}
