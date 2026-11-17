//! گام 2.10 ROADMAP — اندیس‌گذاری زنجیره‌ای (بند ۱۳ ممیزی خارجی) — بندهای DoD.
//!
//! انتظارات از اوراکل Octave 10.3 (scripts/probe_chain210.m سمت دستیار) و
//! مشخصات گام: `[1 2 3](2)`، `ones(3,3)(2,2)`، `f(x)(1)`، `s.f(2)`؛ اصطلاح
//! سلولی `x(1)(2)` = خطای صادقانهٔ E4 (متلب ()() را نمی‌پذیرد؛ اکتاو زنجیرهٔ
//! آزاد می‌دهد = دلتای اوراکل مستند در COMPATIBILITY).
//! «تست قبل از کد» — این سوئیت قبل از پیاده‌سازی بر اساس اوراکل نوشته شد.

use mc_core::Value;
use mc_interp::{eval_program, Environment};
use mc_parser::parse;

fn run(src: &str) -> Environment {
    let prog = parse(src).expect("parse");
    let mut env = Environment::new();
    let out = eval_program(&prog, &mut env);
    assert!(out.error.is_none(), "engine error: {:?}", out.error);
    env
}

fn run_err(src: &str) -> mc_core::McError {
    let prog = match parse(src) {
        Ok(p) => p,
        Err(e) => return e,
    };
    let mut env = Environment::new();
    let out = eval_program(&prog, &mut env);
    out.error.expect("expected an error")
}

fn scalar(env: &Environment, name: &str) -> f64 {
    match env.get(name) {
        Some(Value::Double(m)) if m.is_scalar() => m.get(0, 0),
        other => panic!("'{name}' is not a scalar: {other:?}"),
    }
}

fn vec_of(env: &Environment, name: &str) -> Vec<f64> {
    match env.get(name) {
        Some(Value::Double(m)) => (0..m.numel()).map(|k| m.get_linear(k)).collect(),
        other => panic!("'{name}' is not a matrix: {other:?}"),
    }
}

fn char_of(env: &Environment, name: &str) -> String {
    match env.get(name) {
        Some(Value::Char(s)) => s.clone(),
        other => panic!("'{name}' is not a char: {other:?}"),
    }
}

// ---------- اشکال هدف گام (اوراکل Octave 10.3) ----------

#[test]
fn literal_indexing_with_end_and_ranges() {
    let env = run("a = [1 2 3](2); b = [1 2 3](end); c = [1 2 3](2:3); \
         d = [10;20](2); e = (1:4)(3); f = [1 2 3](1:2:end);");
    assert_eq!(scalar(&env, "a"), 2.0);
    assert_eq!(scalar(&env, "b"), 3.0);
    assert_eq!(vec_of(&env, "c"), vec![2.0, 3.0]);
    assert_eq!(scalar(&env, "d"), 20.0);
    assert_eq!(scalar(&env, "e"), 3.0);
    assert_eq!(vec_of(&env, "f"), vec![1.0, 3.0]);
}

#[test]
fn call_result_indexing_builtin_and_anon() {
    let env = run("f = @(x) [x x x]; \
         a = ones(3,3)(2,2); b = eye(2)(2,2); \
         c = f(7)(2); d = f(7)([1 2]); \
         g = @(x) [x; 2*x]; e = g(3)(2); h = g(3)(end); \
         i = f(7)(end);");
    assert_eq!(scalar(&env, "a"), 1.0);
    assert_eq!(scalar(&env, "b"), 1.0);
    assert_eq!(scalar(&env, "c"), 7.0);
    assert_eq!(vec_of(&env, "d"), vec![7.0, 7.0]);
    assert_eq!(scalar(&env, "e"), 6.0);
    assert_eq!(scalar(&env, "h"), 6.0);
    assert_eq!(scalar(&env, "i"), 7.0);
}

#[test]
fn deeper_chain_until_scalar_bounds() {
    // زنجیرهٔ عمیق تا برخورد طبیعی به مرز: f(1)(2)=1 اسکالر، (3) → out of bounds
    // (اوراکل: "out of bound 1 (dimensions are 1x1)")
    let e = run_err("f = @(x) [x x x]; y = f(1)(2)(3);");
    assert!(e.message_en.contains("out of bounds"), "{}", e.message_en);
    // خودِ زنجیرهٔ معتبر کار می‌کند
    let env = run("f = @(x) [x x x]; y = f(1)(2);");
    assert_eq!(scalar(&env, "y"), 1.0);
}

// ---------- تعامل‌های پسوندی (ترانهاده/پرانتز) — سوئیت ضدی ----------

#[test]
fn transpose_then_index_and_index_then_transpose() {
    let env = run("A = [1 2; 3 4]; \
         a = [1 2 3]'(1); \
         b = A(1,:)'(2); \
         c = [1 2 3](2)'; \
         d = A(2,:)'(end); \
         e = ([1 2] + [3 4])(1);");
    // [1 2 3]' = [1;2;3] → (1) = 1
    assert_eq!(scalar(&env, "a"), 1.0);
    // A(1,:) = [1 2] → ' = [1;2] → (2) = 2
    assert_eq!(scalar(&env, "b"), 2.0);
    // [1 2 3](2) = 2 اسکالر → ' = خودش
    assert_eq!(scalar(&env, "c"), 2.0);
    // A(2,:) = [3 4] → ' = [3;4] → (end) = 4
    assert_eq!(scalar(&env, "d"), 4.0);
    assert_eq!(scalar(&env, "e"), 4.0);
}

#[test]
fn dot_transpose_after_call_result() {
    let env = run("f = @(x) [x x]; a = f(3).' (2); b = f(3)'(1);");
    assert_eq!(scalar(&env, "a"), 3.0);
    assert_eq!(scalar(&env, "b"), 3.0);
}

#[test]
fn index_inside_matrix_literal() {
    // عنصر ماتریس لفظی خودش اندیس‌دار: [[1 2](1) 5]
    let env = run("v = [[1 2](1), 5]; w = [ones(2,2)(2,1), 7];");
    assert_eq!(vec_of(&env, "v"), vec![1.0, 5.0]);
    assert_eq!(vec_of(&env, "w"), vec![1.0, 7.0]);
}

#[test]
fn char_literal_indexing() {
    let env =
        run("a = 'abc'(2); b = 'hello'(end); c = 'ab'([2 1]); d = ['xy'(2), 'z']; e = double(d);");
    assert_eq!(char_of(&env, "a"), "b");
    assert_eq!(char_of(&env, "b"), "o");
    assert_eq!(char_of(&env, "c"), "ba");
    // تلفیق char+char در ماتریس لفظی = char (ارتقای 2.8)
    assert_eq!(char_of(&env, "d"), "yz");
    assert_eq!(vec_of(&env, "e"), vec![121.0, 122.0]);
}

// ---------- فیلد + اندیس (وعدهٔ پیام خطای گام 2.9) ----------

#[test]
fn field_then_index_read() {
    let env = run("s = struct('a', [1 2 3], 't', 'word'); \
         a = s.a(2); b = s.a([1 3]); c = s.a(end); \
         nm = 'a'; d = s.(nm)(2); e = s.t(3);");
    assert_eq!(scalar(&env, "a"), 2.0);
    assert_eq!(vec_of(&env, "b"), vec![1.0, 3.0]);
    assert_eq!(scalar(&env, "c"), 3.0);
    assert_eq!(scalar(&env, "d"), 2.0);
    assert_eq!(char_of(&env, "e"), "r");
}

#[test]
fn field_then_index_write_and_auto_viv() {
    let env = run("s = struct('a', [1 2 3]); s.a(2) = 9; a = s.a; \
         t = struct(); t.b(3) = 5; b = t.b; \
         w = struct(); w.x.y = [1 2 3]; w.x.y(2) = 8; c = w.x.y; \
         q = struct('t', 'abc'); q.t(2) = 'z'; d = q.t;");
    assert_eq!(vec_of(&env, "a"), vec![1.0, 9.0, 3.0]);
    // auto-viv از خالی — اوراکل Octave 10.3: [0 0 5]
    assert_eq!(vec_of(&env, "b"), vec![0.0, 0.0, 5.0]);
    assert_eq!(vec_of(&env, "c"), vec![1.0, 8.0, 3.0]);
    // 'abc' با جایگذاری جای ۲ → 'azc'
    assert_eq!(char_of(&env, "d"), "azc");
}

#[test]
fn field_index_write_boundaries() {
    // حذف با []: s.a(2) = [] → [1 3]
    let env = run("s = struct('a', [1 2 3]); s.a(2) = []; a = s.a;");
    assert_eq!(vec_of(&env, "a"), vec![1.0, 3.0]);
    // نوع غیرعددی فیلد در انتساب اندیس‌دار
    let e = run_err("s = struct('a', struct('x', 1)); s.a(2) = 5;");
    assert!(e.message_en.contains("numeric array"), "{}", e.message_en);
    // فیلد غایب روی مسیر تودرتو با ساختار میانی غایب → auto-viv کل زنجیره
    let env2 = run("w = struct(); w.a.b(2) = 7; v = w.a.b;");
    assert_eq!(vec_of(&env2, "v"), vec![0.0, 7.0]);
}

// ---------- اصطلاح سلولی x(1)(2) — خطای صادقانهٔ E4 ----------

#[test]
fn cell_idiom_is_an_honest_e4_error() {
    let e1 = run_err("x = [1 2]; y = x(1)(2);");
    assert!(
        e1.message_en.contains("cell") && e1.message_en.contains("E4"),
        "{}",
        e1.message_en
    );
    let e2 = run_err("A = [1 2; 3 4]; y = A(1,2)(1);");
    assert!(e2.message_en.contains("cell"), "{}", e2.message_en);
    // فرم انتساب هم همان خطا را می‌گیرد (پارسر — متلب هم lvalue نمی‌داند)
    let e3 = run_err("x = [1 2]; x(1)(2) = 3;");
    assert!(e3.message_en.contains("cell"), "{}", e3.message_en);
    // اکتاو زنجیرهٔ آزاد می‌دهد (A(1,2)(1) = 4) — دلتای اوراکل مستند؛ ما = متلب
}

#[test]
fn anon_handle_result_chains_are_not_the_cell_idiom() {
    // f تابع بی‌نام است نه متغیر ماتریسی → f(3)(1) فراخوانی + اندیس نتیجه
    let env = run("f = @(x) [x x x]; y = f(3)(1);");
    assert_eq!(scalar(&env, "y"), 3.0);
}

// ---------- خطاها و مرزها ----------

#[test]
fn bounds_and_arity_errors() {
    let e1 = run_err("y = [1 2 3](5);");
    assert!(e1.message_en.contains("out of bounds"), "{}", e1.message_en);
    let e2 = run_err("y = zeros(2,2)(1,1,1);");
    assert!(
        e2.message_en.contains("subscript") || e2.message_en.contains("unsupported"),
        "{}",
        e2.message_en
    );
    // اندیس خالی expr()
    let e3 = run_err("y = [1 2 3]();");
    assert!(e3.message_en.contains("subscript"), "{}", e3.message_en);
    // اندیس (1) روی اسکالرِ نتیجهٔ فراخوانی مجاز است (خودِ اسکالر)
    let env = run("f = @(x) x; y = f(1)(1);");
    assert_eq!(scalar(&env, "y"), 1.0);
    // اندیس پرانتزی روی ساختار = آرایهٔ ساختاری = E4
    let e5 = run_err("s = struct('a', 1); y = s(1);");
    assert!(e5.message_en.contains("indexable"), "{}", e5.message_en);
}

#[test]
fn regression_name_based_indexing_untouched() {
    // مسیر قدیمی نام‌محور — رگرسیون (رفتار 2.8/2.4 حفظ شده)
    let env = run("A = [1 2; 3 4]; a = A(2,1); b = A(3); c = A(1, :); \
         x = []; x(3) = 5; \
         st = 'hello'; e = st(2); \
         s = struct('a', 1); s.a = 9; f = s.a;");
    assert_eq!(scalar(&env, "a"), 3.0);
    // ستون‌محوری: A(3) = A(1,2) = 2
    assert_eq!(scalar(&env, "b"), 2.0);
    assert_eq!(vec_of(&env, "c"), vec![1.0, 2.0]);
    assert_eq!(vec_of(&env, "x"), vec![0.0, 0.0, 5.0]);
    assert_eq!(char_of(&env, "e"), "e");
    assert_eq!(scalar(&env, "f"), 9.0);
}
