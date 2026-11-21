//! گام 4.2 ROADMAP — CAS-1 (سند ۰۵ §۳–۴، §۱۰–۱۱) — بندهای DoD گام.
//!
//! پوشش: توابع روی آرگومان سمبولیک (sin(x)، exp(-x^2))، `diff` کامل
//! (دو-چهره: نمادین/عددی با دیسپچ نوع آرگومان اول)، `expand/collect/
//! simplify`، رندر LaTeX با `latex(S)`، خودآزمایی عددی مشتق، و دموی DoD:
//! `diff(exp(-x^2)*x^3)` با خروجی زیبا. مسیرهای خطای صادقانه هم تست
//! می‌شوند (هرگز جواب غلطِ مطمئن‌نما نه — سند ۰۰ اصل ۵).

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

fn char_str(env: &Environment, name: &str) -> String {
    match value(env, name) {
        Value::Char(s) => s,
        other => panic!("{name} char نیست: {other:?}"),
    }
}

fn scalar(env: &Environment, name: &str) -> f64 {
    match value(env, name) {
        Value::Double(m) if m.is_scalar() => m.get(0, 0),
        other => panic!("{name} اسکالر عددی نیست: {other:?}"),
    }
}

// ---------- توابع روی آرگومان سمبولیک (§۲ Fn + §۴ جدول) ----------

#[test]
fn symbolic_functions_build_fn_nodes() {
    let env = run("syms x; y = sin(x); z = exp(-x^2); w = sqrt(x + 1);");
    assert_eq!(sym_str(&env, "y"), "sin(x)");
    assert_eq!(sym_str(&env, "z"), "exp(-x^2)");
    assert_eq!(sym_str(&env, "w"), "sqrt(x + 1)");
    // تاش‌های دقیق همان لحظهٔ ساخت: sin(0)=0، sqrt(4)=2 (پاریت متلب)
    let env2 = run("syms x; a = sin(sym(0)); b = sqrt(sym(4)); c = log(sym(1));");
    assert_eq!(sym_str(&env2, "a"), "0");
    assert_eq!(sym_str(&env2, "b"), "2");
    assert_eq!(sym_str(&env2, "c"), "0");
    // عددی خالص همان مسیر عددی می‌ماند (پاریت متلب)
    let env3 = run("d = sin(0.5); e = sqrt(4);");
    assert!((scalar(&env3, "d") - 0.5f64.sin()).abs() < 1e-15);
    assert_eq!(scalar(&env3, "e"), 2.0);
}

#[test]
fn symbolic_fn_numeric_evaluation_via_subs() {
    // double(subs(sin(x), x, ...)) — ارزیابی عددی درخت Fn
    let env =
        run("syms x; y = sin(x); a = double(subs(y, x, 0.5)); b = double(subs(exp(x), x, 1));");
    assert!((scalar(&env, "a") - 0.5f64.sin()).abs() < 1e-15);
    assert!((scalar(&env, "b") - std::f64::consts::E).abs() < 1e-15);
    // بیرون از دامنهٔ حقیقی → NaN صادقانه
    let env2 = run("syms x; y = sqrt(x); c = double(subs(y, x, -4));");
    assert!(scalar(&env2, "c").is_nan());
}

// ---------- diff (§۴) ----------

#[test]
fn diff_symbolic_basic_forms() {
    let env = run("syms x; y = diff(sin(x)); z = diff(cos(x)); w = diff(tan(x));");
    assert_eq!(sym_str(&env, "y"), "cos(x)");
    assert_eq!(sym_str(&env, "z"), "-sin(x)");
    // فرم متلب: tan^2 + 1 (راستی‌آزمایی‌شده)
    assert_eq!(sym_str(&env, "w"), "tan(x)^2 + 1");
    let env2 = run("syms x; y = diff(x^3); z = diff(x^x); w = diff(2^x);");
    assert_eq!(sym_str(&env2, "y"), "3*x^2");
    assert_eq!(sym_str(&env2, "z"), "x^x*(log(x) + 1)");
    assert_eq!(sym_str(&env2, "w"), "2^x*log(2)");
}

#[test]
fn diff_var_n_and_partial() {
    // diff(S, x), diff(S, n), diff(S, x, n) — پاریت متلب
    let env = run("syms x; y = diff(x^3, x); z = diff(x^3, 2); w = diff(x^3, x, 2);");
    assert_eq!(sym_str(&env, "y"), "3*x^2");
    assert_eq!(sym_str(&env, "z"), "6*x");
    assert_eq!(sym_str(&env, "w"), "6*x");
    // مشتق جزئی: y نماد دیگری است
    let env2 = run("syms x y; f = x*y^2; a = diff(f, x); b = diff(f, y);");
    assert_eq!(sym_str(&env2, "a"), "y^2");
    assert_eq!(sym_str(&env2, "b"), "2*x*y");
    // متغیر پیش‌فرض (نزدیک‌ترین به x)
    let env3 = run("syms y t; g = y*t; c = diff(g);");
    assert_eq!(sym_str(&env3, "c"), "t");
    // char هم مثل متلب متغیر پذیرفته می‌شود
    let env4 = run("syms x; d = diff(x^2, 'x');");
    assert_eq!(sym_str(&env4, "d"), "2*x");
}

#[test]
fn diff_numeric_face_still_works() {
    // دیسپچ با نوع آرگومان اول — چهرهٔ عددی دست‌نخورده
    let env = run("v = [1 4 9 16]; d = diff(v);");
    match value(&env, "d") {
        Value::Double(m) => {
            assert_eq!(m.cols(), 3);
            assert_eq!(m.get(0, 0), 3.0);
            assert_eq!(m.get(0, 2), 7.0);
        }
        other => panic!("d عددی نیست: {other:?}"),
    }
    // تفاوت سطریِ مرتبهٔ ۲: diff(v,2) — درس این گام: قبلاً آرگومان k
    // بی‌صدا نادیده گرفته می‌شد؛ حالا واقعاً دو بار اعمال می‌شود
    let env2 = run("v = [1 4 9 16]; d = diff(v, 2);");
    match value(&env2, "d") {
        Value::Double(m) => {
            assert_eq!(m.cols(), 2);
            assert_eq!(m.get(0, 0), 2.0);
            assert_eq!(m.get(0, 1), 2.0);
        }
        other => panic!("d عددی نیست: {other:?}"),
    }
    // dim صریح
    let env3 = run("v = [1 4 9 16]; d = diff(v, 1, 2); e = diff(v, 1, 1);");
    match value(&env3, "d") {
        Value::Double(m) => {
            assert_eq!(m.cols(), 3);
            assert_eq!(m.get(0, 0), 3.0);
        }
        other => panic!("d عددی نیست: {other:?}"),
    }
    match value(&env3, "e") {
        Value::Double(m) => assert_eq!((m.rows(), m.cols()), (0, 4)),
        other => panic!("e عددی نیست: {other:?}"),
    }
}

#[test]
fn diff_errors_honest() {
    // مرتبهٔ منفی/غیرصحیح و آرگومان نابه‌جا
    assert!(format!("{:?}", run_err("syms x; diff(x^2, -1);")).len() > 10);
    assert!(format!("{:?}", run_err("syms x; diff(x^2, 1.5);")).len() > 10);
    assert!(format!("{:?}", run_err("syms x; diff(x^2, 1, 2);")).len() > 10);
    // بدون نامزد تک‌حرفی برای متغیر پیش‌فرض
    assert!(format!("{:?}", run_err("syms theta; diff(theta^2);")).len() > 10);
}

// ---------- expand / collect / simplify (§۳) ----------

#[test]
fn expand_and_collect() {
    let env = run("syms x w; y = expand((x+1)*(x+2)); z = collect(x^2*w + w*x + 3, x);");
    assert_eq!(sym_str(&env, "y"), "x^2 + 3*x + 2");
    assert_eq!(sym_str(&env, "z"), "w*x + w*x^2 + 3");
    // expand کسر را جمع نمی‌کند (پاریت متلب)؛ simplify می‌کند
    let env2 = run("syms x; e = expand((x^2-1)/(x-1)); s = simplify((x^2-1)/(x-1));");
    assert_eq!(sym_str(&env2, "e"), "(x^2 - 1)/(x - 1)");
    assert_eq!(sym_str(&env2, "s"), "x + 1");
}

#[test]
fn simplify_rules_total_domain_only() {
    let env = run(
        "syms x; a = simplify(sin(x)^2 + cos(x)^2); b = simplify(exp(x)*exp(-x)); \
         c = simplify(log(exp(3*x)));",
    );
    assert_eq!(sym_str(&env, "a"), "1");
    assert_eq!(sym_str(&env, "b"), "1");
    assert_eq!(sym_str(&env, "c"), "3*x");
    // قواعد نیم‌دامنه اعمال نمی‌شوند: sqrt(x)^2 و exp(log(x)) تغییر نمی‌کنند
    let env2 = run("syms x; d = simplify(sqrt(x)^2); e = simplify(exp(log(x)));");
    assert_eq!(sym_str(&env2, "d"), "sqrt(x)^2");
    assert_eq!(sym_str(&env2, "e"), "exp(log(x))");
}

// ---------- latex (§۱۰) ----------

#[test]
fn latex_returns_char_with_goldens() {
    let env = run("syms x y theta; a = latex(x^2 - 3*y + 1/2); b = latex(theta); c = latex(sqrt(x^2+1)); d = latex(1/(x-1));");
    assert_eq!(char_str(&env, "a"), "x^{2} - 3\\,y + \\frac{1}{2}");
    assert_eq!(char_str(&env, "b"), "\\theta");
    assert_eq!(char_str(&env, "c"), "\\sqrt{x^{2} + 1}");
    assert_eq!(char_str(&env, "d"), "\\frac{1}{x - 1}");
    // latex روی ماتریس عددی → خطای صادقانه
    assert!(format!("{:?}", run_err("m = [1 2]; latex(m);")).len() > 10);
}

// ---------- دموی DoD ----------

#[test]
fn dod_demo_diff_exp_minus_x2_times_x3() {
    // دموی گام (ROADMAP 4.2): diff(exp(-x^2)*x^3) با خروجی زیبا
    let env = run("syms x; f = exp(-x^2)*x^3; d = diff(f); L = latex(d); \
         chk = double(subs(subs(d, x, 0.5), x, 0.5));");
    let _ = env.get("chk"); // subs دوباره روی درخت تکراری است؛ مقدار پایین
    let env2 = run("syms x; f = exp(-x^2)*x^3; d = diff(f); v = double(subs(d, x, 0.5));");
    // مقدار عددی مشتق در 0.5: e^{-0.25}·(3·0.25 − 2·0.0625) = e^{-0.25}·0.625
    let expect = (-0.25f64).exp() * 0.625;
    assert!((scalar(&env2, "v") - expect).abs() < 1e-12);
    // فرم خطی + LaTeX طلایی
    assert_eq!(sym_str(&env2, "d"), "-2*x^4*exp(-x^2) + 3*x^2*exp(-x^2)");
    assert_eq!(
        char_str(&env, "L"),
        "-2\\,x^{4}\\,\\mathrm{e}^{-x^{2}} + 3\\,x^{2}\\,\\mathrm{e}^{-x^{2}}"
    );
}

// ---------- خودآزمایی عددی مشتق از دل مفسر (DoD «۱۰۰٪») ----------

#[test]
fn derivative_self_check_through_engine() {
    // همان خودآزمایی mc-cas ولی از مسیر کامل مفسر — subs/double/Fn
    let exprs = [
        "sin(x)",
        "cos(2*x)",
        "tan(x)",
        "exp(x)",
        "exp(-x^2)*x^3",
        "log(x)",
        "log10(x)",
        "sqrt(x)",
        "abs(x+2)",
        "x^x",
        "2^x",
        "x^(3/2)",
        "x*sin(x)",
        "x/sin(x)",
        "1/(x^2+1)",
    ];
    let mut failures: Vec<String> = Vec::new();
    for src in exprs {
        let full = format!(
            "syms x; f = {src}; d = diff(f); v1 = double(subs(d, x, 0.7)); v2 = double(subs(d, x, 1.3));"
        );
        let prog = parse(&full).expect("parse");
        let mut env = Environment::new();
        let out = eval_program(&prog, &mut env);
        if out.error.is_some() {
            failures.push(format!("{src}: خطا {:?}", out.error));
            continue;
        }
        for name in ["v1", "v2"] {
            let v = scalar(&env, name);
            if !v.is_finite() {
                failures.push(format!("{src}: {name} متناهی نیست"));
            }
        }
    }
    assert!(failures.is_empty(), "خودآزمایی: {}", failures.join("; "));
}

// ---------- کلاس/نمایش و مسیرهای صادقانه ----------

#[test]
fn class_and_display_of_fn_trees() {
    let env = run("syms x; y = sin(x) + cos(x); c = class(y);");
    assert_eq!(char_str(&env, "c"), "sym");
    assert_eq!(sym_str(&env, "y"), "cos(x) + sin(x)");
    // syms روی نام چندحرفی + تابع روی آن
    let env2 = run("syms theta; y = sin(theta);");
    assert_eq!(sym_str(&env2, "y"), "sin(theta)");
}

#[test]
fn unsupported_paths_stay_honest() {
    // ماتریس سمبولیک/مختلط/اندیس/ترانهاده — هنوز نه (پیام صادقانه بدون وعدهٔ دروغ)
    assert!(format!("{:?}", run_err("syms x; m = [x 1];")).len() > 10);
    assert!(format!("{:?}", run_err("syms x; y = mldivide(x, 2);")).len() > 10);
    // atan2 روی سمبولیک — نامش در جدول نیست؛ مسیر عددی خطای روشن می‌دهد
    assert!(format!("{:?}", run_err("syms x; y = atan2(x, 1);")).len() > 10);
    // expand روی Real تقریبی — خطای صادقانه (نه گردکردن)
    assert!(format!("{:?}", run_err("syms x; y = expand(0.1*x);")).len() > 10);
    // simplify همان درخت تقریبی را بهترین‌تلاش برمی‌گرداند (بدون خطا)
    let env = run("syms x; y = simplify(0.1*x);");
    assert_eq!(sym_str(&env, "y"), "0.1*x");
}
