//! گام 2.8 ROADMAP — builtinهای char: char/double/ischar/blanks/str2double.
//!
//! جداشده از chars_step28.rs برای سری کامیت per-item (موتور جدا از رجیستری).
//! اوراکل: سند رسمی متلب char + Octave 10.3.

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

// ---------- builtinهای گام 2.8 ----------

#[test]
fn builtin_char_truncates_toward_zero() {
    // سند رسمی متلب char: «Rounds nonintegers toward zero»
    let env = eval_ok("y = char(65.9);");
    assert_eq!(char_var(&env, "y"), "A");
}

#[test]
fn builtin_char_from_code_row() {
    let env = eval_ok("y = char([72 105]);");
    assert_eq!(char_var(&env, "y"), "Hi");
}

#[test]
fn builtin_char_identity() {
    let env = eval_ok("y = char('abc');");
    assert_eq!(char_var(&env, "y"), "abc");
}

#[test]
fn builtin_char_out_of_range_is_honest_error() {
    // انحراف مستند: متلب به 0/65535 اشباع می‌کند؛ ما صادقانه خطا می‌دهیم
    let msg = eval_err("y = char(-1);");
    assert!(
        msg.contains("outside the valid Unicode range"),
        "got: {msg}"
    );
}

#[test]
fn builtin_double_from_char() {
    let env = eval_ok("y = double('abc');");
    assert_eq!(row_var(&env, "y"), vec![97.0, 98.0, 99.0]);
}

#[test]
fn builtin_double_numeric_identity() {
    let env = eval_ok("y = double(5);");
    assert_eq!(scalar_var(&env, "y"), 5.0);
}

#[test]
fn builtin_ischar() {
    let env = eval_ok("a = ischar('x'); b = ischar(5);");
    assert_eq!(scalar_var(&env, "a"), 1.0);
    assert_eq!(scalar_var(&env, "b"), 0.0);
}

#[test]
fn builtin_blanks() {
    let env = eval_ok("y = blanks(3); z = blanks(0);");
    assert_eq!(char_var(&env, "y"), "   ");
    assert_eq!(char_var(&env, "z"), "");
}

#[test]
fn builtin_str2double_ok() {
    let env = eval_ok("y = str2double('1.5'); z = str2double('1e3'); w = str2double(' 2.25 ');");
    assert_eq!(scalar_var(&env, "y"), 1.5);
    assert_eq!(scalar_var(&env, "z"), 1000.0);
    assert_eq!(scalar_var(&env, "w"), 2.25);
}

#[test]
fn builtin_str2double_failure_is_nan() {
    // رفتار متلب: تبدیل ناموفق → NaN
    let env = eval_ok("y = str2double('abc');");
    assert!(scalar_var(&env, "y").is_nan());
}

// ---------- چرخهٔ کامل char (concat + حلقه) ----------

#[test]
fn for_over_char_accumulates_via_concat() {
    let env = eval_ok("c = ''; for k = 'abc'; c = [c k]; end");
    assert_eq!(char_var(&env, "c"), "abc");
}

#[test]
fn class_and_num2str_interplay() {
    // «char/double/ischar/num2str interplay» از هدف گام 2.8
    let env = eval_ok("y = class('abc'); z = num2str('abc'); w = ischar(num2str(5));");
    assert_eq!(char_var(&env, "y"), "char");
    assert_eq!(char_var(&env, "z"), "abc");
    assert_eq!(scalar_var(&env, "w"), 1.0);
}
