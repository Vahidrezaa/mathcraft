//! گام 4.1 ROADMAP — CAS-0 پایه‌ها (سند ۰۵ §۲–۳) — بندهای DoD گام.
//!
//! پوشش: گزارهٔ `syms`، پل‌های `sym`/`double`، جای‌گذاری `subs`، رفت‌وبرگشتِ
//! دقیق DoD (`syms x; y = x^2 + 3*x; double(subs(y, x, 2.5))` = 13.75)،
//! حساب دقیق گویاها (D-12: 1/3+1/6=1/2)، صداقتِ Real (sym(0.1) تقریبی
//! می‌ماند)، کانونی‌سازی سطح-۰ در خروجی مفسر، و مسیرهای خطای صادقانه
//! (عمل ناموجود = خطای روشن، هرگز جواب غلطِ عددی نه — سند ۰۰ اصل ۵).

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

fn value(env: &Environment, name: &str) -> Value {
    env.get(name)
        .unwrap_or_else(|| panic!("متغیر {name} تعریف نشده"))
        .clone()
}

fn sym_str(env: &Environment, name: &str) -> String {
    match value(env, name) {
        Value::Sym(s) => mc_cas::sym_display(&s),
        other => panic!("{name} سمبولیک نیست: {other:?}"),
    }
}

fn scalar(env: &Environment, name: &str) -> f64 {
    match value(env, name) {
        Value::Double(m) if m.is_scalar() => m.get(0, 0),
        other => panic!("{name} اسکالر عددی نیست: {other:?}"),
    }
}

#[test]
fn syms_declares_symbols() {
    let env = run("syms x y");
    assert_eq!(sym_str(&env, "x"), "x");
    assert_eq!(sym_str(&env, "y"), "y");
    // class متلب‌گونه
    let env2 = run("syms x; c = class(x);");
    match value(&env2, "c") {
        Value::Char(s) => assert_eq!(s, "sym"),
        other => panic!("class(x) char نبود: {other:?}"),
    }
}

#[test]
fn syms_multi_and_override() {
    let env = run("syms a b c");
    for n in ["a", "b", "c"] {
        assert!(matches!(value(&env, n), Value::Sym(_)));
    }
    // متلب: syms x همیشه نماد می‌سازد، حتی با مقدار قبلی (اعلان دوباره)
    let env2 = run("syms x; x = 5; syms x;");
    assert_eq!(sym_str(&env2, "x"), "x");
}

#[test]
fn symbolic_arithmetic_builds_tree() {
    let env = run("syms x; y = x^2 + 3*x;");
    assert_eq!(sym_str(&env, "y"), "x^2 + 3*x");
    // کانونی‌سازی سطح-۰: x + x = 2*x؛ 2x + 3x = 5x
    let env2 = run("syms x; a = x + x; b = 2*x + 3*x;");
    assert_eq!(sym_str(&env2, "a"), "2*x");
    assert_eq!(sym_str(&env2, "b"), "5*x");
    // x^2 * x^3 = x^5
    let env3 = run("syms x; p = x^2 * x^3;");
    assert_eq!(sym_str(&env3, "p"), "x^5");
    // (x + 1)*s با ترتیب کانونی
    let env4 = run("syms x s; q = (x + 1)*s;");
    assert_eq!(sym_str(&env4, "q"), "s*(x + 1)");
}

#[test]
fn dod_roundtrip_exact() {
    // DoD گام 4.1 — رفت‌وبرگشت دقیق: 55/4 → 13.75
    let env = run("syms x; y = x^2 + 3*x; d = double(subs(y, x, 2.5));");
    assert_eq!(scalar(&env, "d"), 13.75);
    // و خودِ نتیجهٔ subs دقیق است: 55/4
    let env2 = run("syms x; y = x^2 + 3*x; s = subs(y, x, 2.5);");
    assert_eq!(sym_str(&env2, "s"), "55/4");
}

#[test]
fn exact_rational_arithmetic_d12() {
    // D-12 (سند ۰۵): 1/3 + 1/6 = 1/2 — دقیق، هرگز 0.49999…
    let env = run("a = sym(1)/3 + sym(1)/6;");
    assert_eq!(sym_str(&env, "a"), "1/2");
    // (1/3)*3 = 1 دقیق
    let env2 = run("b = sym(1)/3 * 3;");
    assert_eq!(sym_str(&env2, "b"), "1");
    // تقسیم دقیق: 1/7
    let env3 = run("c = sym(1)/7;");
    assert_eq!(sym_str(&env3, "c"), "1/7");
}

#[test]
fn sym_bridges_honest() {
    // دودویی مرتب → دقیق
    let env = run("a = sym(0.25); b = sym(3);");
    assert_eq!(sym_str(&env, "a"), "1/4");
    assert_eq!(sym_str(&env, "b"), "3");
    // غیردودویی → Real تقریبی — هرگز 1/10 جا نمی‌زند (اصل صداقت)
    let env2 = run("c = sym(0.1);");
    assert_eq!(sym_str(&env2, "c"), "0.1");
    // sym('t') نماد می‌سازد (نام دینامیک)
    let env3 = run("d = sym('t');");
    assert_eq!(sym_str(&env3, "d"), "t");
}

#[test]
fn double_bridge_roundtrip() {
    let env = run("syms x; y = x^2; d = double(subs(y, x, 0.5));");
    assert_eq!(scalar(&env, "d"), 0.25);
    // double روی عدد هم‌خانوادهٔ قبلی خودش است
    let env2 = run("d = double(sym(7)/2);");
    assert_eq!(scalar(&env2, "d"), 3.5);
}

#[test]
fn subs_symbolic_value() {
    // subs با مقدار سمبولیک: y با x=t → t^2 + 3*t
    let env = run("syms x t; y = x^2 + 3*x; z = subs(y, x, t);");
    assert_eq!(sym_str(&env, "z"), "t^2 + 3*t");
    // زنجیره: سپس t=1 → 4
    let env2 = run("syms x t; y = x^2 + 3*x; z = subs(y, x, t); w = double(subs(z, t, 1));");
    assert_eq!(scalar(&env2, "w"), 4.0);
    // subs(y, 'x', 2) با نام به‌صورت char هم می‌شود
    let env3 = run("syms x; y = x^2 + 3*x; w = double(subs(y, 'x', 2));");
    assert_eq!(scalar(&env3, "w"), 10.0);
}

#[test]
fn negative_and_division_forms() {
    let env = run("syms x; n = -x; m = -3*x; d = x/3; e = 1/x;");
    assert_eq!(sym_str(&env, "n"), "-x");
    assert_eq!(sym_str(&env, "m"), "-3*x");
    assert_eq!(sym_str(&env, "d"), "1/3*x");
    assert_eq!(sym_str(&env, "e"), "1/x");
    // y/x متلب‌گونه
    let env2 = run("syms x y; q = y/x;");
    assert_eq!(sym_str(&env2, "q"), "y/x");
}

#[test]
fn guard_zero_times_undefined() {
    // 0*x = 0؛ ولی 0*(1/x) جمع نمی‌شود — گارد سند ۰۵ §۲ (x=0 تعریف‌نشده).
    // CAS-1: نمایش شکل تقسیم صریح 0/x — دامنه را پیش‌چشم نگه می‌دارد
    let env = run("syms x; a = 0*x; b = 0*(1/x);");
    assert_eq!(sym_str(&env, "a"), "0");
    assert_eq!(sym_str(&env, "b"), "0/x");
}

#[test]
fn error_double_with_symbol_left() {
    // نماد مانده → خطای صادقانه با نام نماد (نه حدس عددی)
    let e = run_err("syms x; y = x^2; d = double(y);");
    assert!(
        format!("{e:?}").contains("x"),
        "پیام باید نام نماد را بگوید: {e:?}"
    );
}

#[test]
fn error_division_by_zero_exact() {
    let e = run_err("syms x; a = x/0;");
    let msg = format!("{e:?}");
    assert!(msg.contains("صفر") || msg.contains("zero"), "پیام: {msg}");
    let e2 = run_err("a = sym(0)^-1;");
    assert!(!format!("{e2:?}").is_empty());
}

#[test]
fn error_unsupported_ops_honest() {
    // sin روی سمبولیک از CAS-1 (گام 4.2) درخت Fn می‌سازد — دیگر خطا نیست؛
    // بقیهٔ مسیرهای ناموجود همچنان صادقانه خطا می‌دهند
    let env = run("syms x; y = sin(x);");
    assert_eq!(sym_str(&env, "y"), "sin(x)");
    // ماتریسِ سمبولیک → خطا
    let e2 = run_err("syms x; m = [x 1];");
    assert!(format!("{e2:?}").len() > 10);
    // اندیس‌گذاری سمبولیک → خطا
    let e3 = run_err("syms x; y = x(1);");
    assert!(format!("{e3:?}").len() > 10);
    // تکرار روی سمبولیک → خطا
    let e4 = run_err("syms x; for v = x; end");
    assert!(format!("{e4:?}").len() > 10);
    // ترانهاده سمبولیک → خطا (CAS-1)
    let e5 = run_err("syms x; y = x';");
    assert!(format!("{e5:?}").len() > 10);
    // حل با \ سمبولیک → خطا (CAS-2)
    let e6 = run_err("syms x; y = x \\ 1;");
    assert!(format!("{e6:?}").len() > 10);
}

#[test]
fn error_comparison_symbolic() {
    // مقایسهٔ سمبولیک در CAS-0 معنا ندارد — خطای صادقانه (فرض دامنه نداریم)
    let e = run_err("syms x; if x > 1; a = 1; end");
    assert!(format!("{e:?}").len() > 10);
}

#[test]
fn num2str_and_whos_class() {
    let env = run("syms x; y = x^2 + 3*x; s = num2str(y);");
    match value(&env, "s") {
        Value::Char(t) => assert_eq!(t, "x^2 + 3*x"),
        other => panic!("num2str(y) char نبود: {other:?}"),
    }
}

#[test]
fn sym_save_load_roundtrip_mcws() {
    // CAS-1 (گام 4.2، سند ۰۵ §۱۲): KIND_SYM اضافه شد — در CAS-0 این مسیر
    // صادقانه رد می‌شد؛ اکنون ذخیره/بازیابی بی‌اتلاف است. میزبان صف خروجی
    // را برداشته و دوباره ثبت می‌کند (همان الگوی تست واحد io_builtins)
    let env = run("syms x; y = x^2 + 3*x; save('ws42-cas1.mcws', 'x', 'y');");
    drop(env);
    let outs = mc_io::vfs::take_outputs();
    assert_eq!(outs.len(), 1, "save باید یک خروجی می‌زد");
    mc_io::vfs::register("ws42-cas1.mcws", outs[0].1.clone());
    let mut fresh = Environment::new();
    let prog = parse("load('ws42-cas1.mcws');").expect("parse");
    let out = eval_program(&prog, &mut fresh);
    assert!(out.error.is_none(), "load خطا داد: {:?}", out.error);
    assert_eq!(sym_str(&fresh, "y"), "x^2 + 3*x");
    mc_io::vfs::clear_all();
    // syms دوباره لازم نیست — کلاس پس از بارگذاری همان sym است
    match value(&fresh, "x") {
        Value::Sym(_) => {}
        other => panic!("x پس از load سمبولیک نیست: {other:?}"),
    }
}

#[test]
fn user_override_of_syms_name() {
    // تابع کاربر با نام syms → تقدم کاربر (الگوی rng/hold)
    let env = run("function r = syms(a); r = a + 1; end; z = syms(41);");
    assert_eq!(scalar(&env, "z"), 42.0);
}

#[test]
fn symbolic_scalar_with_numeric_matrix_errors() {
    // سمبولیک × ماتریس → CAS-1 (پیام روشن)
    let e = run_err("syms x; m = [1 2; 3 4]; y = m * x;");
    assert!(format!("{e:?}").len() > 10);
}

#[test]
fn power_identities_in_interp() {
    let env = run("syms x; a = x^0; b = x^1; c = (x^2)^3;");
    assert_eq!(sym_str(&env, "a"), "1");
    assert_eq!(sym_str(&env, "b"), "x");
    assert_eq!(sym_str(&env, "c"), "x^6");
    // توان کسری عددی → Real تقریبی صادقانه: sym(4)^(1/2) → 2
    let env2 = run("syms x; d = double(sym(4)^(1/2));");
    assert_eq!(scalar(&env2, "d"), 2.0);
}

// ---------- ممیزی کیفیت ۲۰۲۶-۱۱-۲۱ (تسک ۴۹) ----------

#[test]
fn approx_terms_mixed_arithmetic_terminates() {
    // قبلاً سرریز پشته می‌داد: جمعِ ترم تقریبی (حاوی Real) با ترم نمادین
    // در Sym::plus دچار بازگشت بی‌پایان می‌شد — مسیر واقعی کاربر
    let env = run("syms x; y = x + 0.1*x; z = x + 0.1;");
    assert_eq!(sym_str(&env, "y"), "x + 0.1*x");
    assert_eq!(sym_str(&env, "z"), "x + 0.1");
    // ارزیابی صادقانه: x=10 → 10 + 0.1*10 = 11
    let env2 = run("syms x; y = x + 0.1*x; d = double(subs(y, x, 10));");
    assert_eq!(scalar(&env2, "d"), 11.0);
    // دو ترم تقریبی مجزا ادغام نمی‌شوند (جمع f64 ضرایب ضریب‌پذیر نیست)
    let env3 = run("syms x; w = 0.1*x + 0.2*x;");
    assert_eq!(sym_str(&env3, "w"), "0.1*x + 0.2*x");
}

#[test]
fn zero_times_infinite_honest_matlab() {
    // متلب: 0*Inf = NaN — فروختن 0*(inf*x) به 0 غلط بود؛ درخت می‌ماند و
    // ارزیابی صادقانه NaN می‌دهد
    let env = run("syms x; y = 0*inf*x; d = double(subs(y, x, 1));");
    assert!(scalar(&env, "d").is_nan());
}

#[test]
fn negative_terms_display_matlab_style() {
    // قبلاً «x + -3*y» — اکنون متلب‌گونه (ممیزی ۲۰۲۶-۱۱-۲۱)
    let env = run("syms x y; a = x - 3*y; b = x - 0.1*y;");
    assert_eq!(sym_str(&env, "a"), "x - 3*y");
    assert_eq!(sym_str(&env, "b"), "x - 0.1*y");
}
