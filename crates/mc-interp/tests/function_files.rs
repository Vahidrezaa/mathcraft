//! تست‌های توابع فایلی و اسکریپت‌ها (سند ۰۳ §9 — آیتم‌های DoD P1: #2 و #9).
//!
//! سمانتیک متلب §9 که اینجا قفل می‌شود:
//! - ورک‌اسپیس جدا برای تابع (بدون دید متقابل متغیرها)
//! - پارامترهای موقعیتی، nargin/nargout، خروجی‌های چندگانه به ترتیب تعریف
//! - تقدم: متغیر > تابع فایلی > builtin (سایه‌کردن builtin مجاز)
//! - return از تابع/اسکریپت؛ بازگشتی؛ توابع محلی بعد از تابع اصلی
//! - هندل نام‌دار `@f` برای تابع کاربر و builtin

use mc_core::Value;
use mc_interp::{eval_program, Environment};
use mc_parser::parse;

fn run(env: &mut Environment, src: &str) -> mc_interp::EvalOutcome {
    match parse(src) {
        Ok(p) => eval_program(&p, env),
        Err(e) => mc_interp::EvalOutcome {
            outputs: Vec::new(),
            error: Some(e),
        },
    }
}

fn run_ok(env: &mut Environment, src: &str) -> mc_interp::EvalOutcome {
    let out = run(env, src);
    assert!(
        out.error.is_none(),
        "expected success for `{src}`, got {:?}",
        out.error.unwrap()
    );
    out
}

fn scalar_of(env: &Environment, name: &str) -> f64 {
    match env.get(name) {
        Some(Value::Double(m)) if m.is_scalar() => m.get(0, 0),
        other => panic!("expected scalar {name}, got {other:?}"),
    }
}

fn expect_error(env: &mut Environment, src: &str) -> mc_core::McError {
    let out = run(env, src);
    out.error
        .unwrap_or_else(|| panic!("expected error for `{src}`"))
}

/// اجرا روی پشته بزرگ — فریم‌های eval در بیلد debug بزرگ‌اند و عمق مجاز
/// بازگشتی (۱۲۸) به پشتهٔ پیش‌فرض تست (۲MB) جا نمی‌شود؛ mc-ffi در موبایل
/// روی job-thread با پشتهٔ تضمینی اجرا می‌کند (سند ۰۲ بخش ۴).
fn run_deep<F, T>(f: F) -> T
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    std::thread::Builder::new()
        .stack_size(32 * 1024 * 1024)
        .spawn(f)
        .expect("spawn deep stack thread")
        .join()
        .expect("deep thread panicked")
}

// ---------- تعریف و فراخوانی ----------

#[test]
fn define_and_call_single_output() {
    let mut env = Environment::new();
    run_ok(&mut env, "function y = sq(x)\ny = x^2\nend\nsq(5)");
    assert_eq!(scalar_of(&env, "ans"), 25.0);
    // متغیرهای تابع نشت نکرده‌اند
    assert!(!env.contains("y"));
    assert!(!env.contains("x"));
}

#[test]
fn function_file_produces_no_console_output() {
    let mut env = Environment::new();
    let out = run_ok(&mut env, "function y = sq(x)\ny = x^2\nend");
    assert!(out.outputs.is_empty(), "تعریف تابع خروجی کنسول ندارد");
}

#[test]
fn one_liner_with_separators() {
    let mut env = Environment::new();
    run_ok(&mut env, "function y = add(a,b); y = a + b; end\nadd(2,3)");
    assert_eq!(scalar_of(&env, "ans"), 5.0);
}

#[test]
fn multi_output_call_in_definition_order() {
    let mut env = Environment::new();
    run_ok(
        &mut env,
        "function [s,p] = stats(v)\ns = sum(v)\np = prod(v)\nend",
    );
    run_ok(&mut env, "[a,b] = stats([1 2 4])");
    assert_eq!(scalar_of(&env, "a"), 7.0);
    assert_eq!(scalar_of(&env, "b"), 8.0);
    // نمایش‌دار: هر خروجی یک سطر
    run_ok(&mut env, "[s2,p2] = stats([1 2 4])");
    assert!(env.contains("s2") && env.contains("p2"));
}

#[test]
fn single_output_from_builtin_multiassign() {
    let mut env = Environment::new();
    run_ok(&mut env, "[a] = sin(0)");
    assert_eq!(scalar_of(&env, "a"), 0.0);
}

#[test]
fn multi_output_from_builtin_is_rejected() {
    let mut env = Environment::new();
    let e = expect_error(&mut env, "[a,b] = sin(0)");
    assert_eq!(e.id, "MC:E3001_PARSE_ERROR");
}

#[test]
fn multiassign_from_non_call_is_rejected() {
    let mut env = Environment::new();
    let e = expect_error(&mut env, "[a,b] = [1 2]");
    assert!(e.message_en.contains("Only function calls"));
}

#[test]
fn function_calling_registered_function() {
    let mut env = Environment::new();
    run_ok(
        &mut env,
        "function y = sq(x); y = x.^2; end\nfunction y = sqsum(v); y = sum(sq(v)); end",
    );
    run_ok(&mut env, "r = sqsum([1 2 3])");
    assert_eq!(scalar_of(&env, "r"), 14.0);
}

#[test]
fn recursion_factorial() {
    let mut env = Environment::new();
    run_ok(
        &mut env,
        "function y = fact(n)\nif n <= 1\ny = 1\nelse\ny = n * fact(n-1)\nend\nend",
    );
    run_ok(&mut env, "f6 = fact(6)");
    assert_eq!(scalar_of(&env, "f6"), 720.0);
}

#[test]
fn infinite_recursion_is_bounded() {
    run_deep(|| {
        let mut env = Environment::new();
        run_ok(&mut env, "function y = loop(n); y = loop(n+1); end");
        let e = expect_error(&mut env, "loop(1)");
        assert!(e.message_en.contains("recursion"), "{e:?}");
    });
}

// ---------- nargin / nargout / پارامتر فراهم‌نشده ----------

#[test]
fn nargin_and_nargout_inside_body() {
    let mut env = Environment::new();
    run_ok(
        &mut env,
        "function [a,b] = probe(x,z)\na = nargin\nb = nargout\nend",
    );
    run_ok(&mut env, "[na,nb] = probe(1,2)");
    assert_eq!(scalar_of(&env, "na"), 2.0);
    assert_eq!(scalar_of(&env, "nb"), 2.0);
    // nargout یک‌خروجی‌ای
    run_ok(&mut env, "na2 = probe(1,2)");
    assert_eq!(scalar_of(&env, "na2"), 2.0);
}

#[test]
fn missing_parameter_allows_nargin_pattern() {
    let mut env = Environment::new();
    run_ok(
        &mut env,
        "function y = power(base,p)\nif nargin < 2\np = 2\nend\ny = base^p\nend",
    );
    run_ok(&mut env, "a = power(5)"); // p ندهید — پیش‌فرض ۲
    assert_eq!(scalar_of(&env, "a"), 25.0);
    run_ok(&mut env, "b = power(5,3)");
    assert_eq!(scalar_of(&env, "b"), 125.0);
}

#[test]
fn too_many_inputs_is_rejected() {
    let mut env = Environment::new();
    run_ok(&mut env, "function y = f(x); y = x; end");
    let e = expect_error(&mut env, "f(1,2)");
    assert!(e.message_en.contains("Too many input"), "{e:?}");
}

#[test]
fn too_many_outputs_is_rejected() {
    let mut env = Environment::new();
    run_ok(&mut env, "function y = f(x); y = x; end");
    let e = expect_error(&mut env, "[a,b] = f(1)");
    assert!(e.message_en.contains("Too many output"), "{e:?}");
}

#[test]
fn zero_output_function_as_statement() {
    let mut env = Environment::new();
    run_ok(&mut env, "function show(x)\nt = x + 1\nend");
    // فراخوانی گزاره‌ای بدون خروجی — متلب مجاز می‌داند
    run_ok(&mut env, "show(5)");
    // در عبارت = خطا
    let e = expect_error(&mut env, "z = show(5)");
    assert!(e.message_en.contains("Too many output"), "{e:?}");
}

#[test]
fn unassigned_output_is_an_error() {
    let mut env = Environment::new();
    run_ok(&mut env, "function y = f(x); t = x + 1; end");
    let e = expect_error(&mut env, "f(1)");
    assert!(e.message_en.contains("Output argument 'y'"), "{e:?}");
}

// ---------- سمانتیک ورک‌اسپیس ----------

#[test]
fn workspace_isolation_both_ways() {
    let mut env = Environment::new();
    run_ok(
        &mut env,
        "secret = 42\nfunction y = peek()\ny = secret\nend",
    );
    // تابع متغیر فراخوان را نمی‌بیند
    let e = expect_error(&mut env, "peek()");
    assert_eq!(e.id, "MC:E1001_UNDEFINED_VARIABLE");
    // تعریف دوباره با استفاده از پارامتر
    run_ok(&mut env, "function y = peek2(secret); y = secret; end");
    run_ok(&mut env, "out = peek2(7)");
    assert_eq!(scalar_of(&env, "out"), 7.0);
    assert!(!env.contains("y"));
}

#[test]
fn variable_shadows_function_and_script() {
    let mut env = Environment::new();
    run_ok(&mut env, "function y = f(x); y = x + 100; end");
    run_ok(&mut env, "f = 9");
    // متغیر بر تابع مقدم است — فراخوانی با پرانتز = اندیس‌گذاری
    run_ok(&mut env, "v = f(1)");
    assert_eq!(scalar_of(&env, "v"), 9.0);
}

// ---------- اسکریپت‌ها ----------

#[test]
fn script_runs_in_caller_workspace() {
    let mut env = Environment::new();
    run_ok(&mut env, "base = 10\nfunction y = f(x); y = x * 2; end");
    run_ok(&mut env, "r = f(base)");
    assert_eq!(scalar_of(&env, "r"), 20.0);
}

#[test]
fn script_shares_caller_vars_and_sets_variables() {
    let mut env = Environment::new();
    // ثبت اسکریپت: گزاره‌های معمولی + تعریف‌ها — اسکریپت = فایل بدون function
    env.register_script("setup", parse("a = 5\nb = a * 2").expect("script parse"));
    run_ok(&mut env, "seed = 3");
    run_ok(&mut env, "setup");
    // اسکریپت در همان فضای کاری اجرا شد
    assert_eq!(scalar_of(&env, "a"), 5.0);
    assert_eq!(scalar_of(&env, "b"), 10.0);
    assert_eq!(scalar_of(&env, "seed"), 3.0);
}

#[test]
fn script_return_stops_script_but_not_caller() {
    let mut env = Environment::new();
    env.register_script(
        "early",
        parse("x = 1\nreturn\nx = 999").expect("script parse"),
    );
    run_ok(&mut env, "early\ny = 7");
    assert_eq!(scalar_of(&env, "x"), 1.0);
    assert_eq!(scalar_of(&env, "y"), 7.0);
}

#[test]
fn script_cannot_take_arguments_or_return_value() {
    let mut env = Environment::new();
    env.register_script("s", parse("x = 1").expect("script parse"));
    let e = expect_error(&mut env, "s(2)");
    assert!(e.message_en.contains("Script 's'"), "{e:?}");
    let e = expect_error(&mut env, "v = s");
    assert!(e.message_en.contains("Script 's'"), "{e:?}");
}

#[test]
fn script_with_local_functions() {
    let mut env = Environment::new();
    // R2016b+: اسکریپت با توابع محلی در انتها
    run_ok(
        &mut env,
        "function y = helper(n); y = n; end\na = 1\nlocal2 = helper(4)\nfunction y = helper2(x); y = x * 10; end",
    );
    assert_eq!(scalar_of(&env, "local2"), 4.0);
    run_ok(&mut env, "z = helper2(3)");
    assert_eq!(scalar_of(&env, "z"), 30.0);
}

#[test]
fn bare_script_name_runs_it() {
    let mut env = Environment::new();
    env.register_script("setup", parse("k = 11\nm = k + 1").expect("script parse"));
    run_ok(&mut env, "setup");
    assert_eq!(scalar_of(&env, "k"), 11.0);
    assert_eq!(scalar_of(&env, "m"), 12.0);
}

#[test]
fn bare_zero_param_function_call() {
    let mut env = Environment::new();
    run_ok(&mut env, "function y = hello(); y = 99; end");
    run_ok(&mut env, "hello");
    assert_eq!(scalar_of(&env, "ans"), 99.0);
    // در عبارت هم قابل استفاده است
    run_ok(&mut env, "r = hello + 1");
    assert_eq!(scalar_of(&env, "r"), 100.0);
}

#[test]
fn bare_parameterized_function_gives_matlab_error() {
    let mut env = Environment::new();
    run_ok(&mut env, "function y = sq(x); y = x^2; end");
    let e = expect_error(&mut env, "sq");
    assert!(e.message_en.contains("Not enough input"), "{e:?}");
}

// ---------- سایه‌کردن builtin و did-you-mean ----------

#[test]
fn user_function_shadows_builtin() {
    let mut env = Environment::new();
    // mean اختصاصی: میانگین ضربدر ۱۰ (عمداً متفاوت با builtin)
    run_ok(
        &mut env,
        "function m = mean(v); m = sum(v)/numel(v) * 10; end",
    );
    run_ok(&mut env, "r = mean([1 2 3])");
    assert_eq!(scalar_of(&env, "r"), 20.0);
}

#[test]
fn redefinition_last_wins() {
    let mut env = Environment::new();
    run_ok(&mut env, "function y = f(x); y = x + 1; end");
    run_ok(&mut env, "function y = f(x); y = x + 2; end");
    run_ok(&mut env, "r = f(1)");
    assert_eq!(scalar_of(&env, "r"), 3.0);
}

#[test]
fn undefined_user_fn_gets_did_you_mean() {
    let mut env = Environment::new();
    run_ok(&mut env, "function y = sq(x); y = x^2; end");
    let e = expect_error(&mut env, "sq2(4)");
    assert_eq!(e.suggestion.as_deref(), Some("sq"));
}

// ---------- جریان کنترل ----------

#[test]
fn return_exits_function_early() {
    let mut env = Environment::new();
    run_ok(
        &mut env,
        "function y = firstneg(v)\ny = 0\nfor k = v\nif k < 0\ny = k\nreturn\nend\nend\nend",
    );
    run_ok(&mut env, "a = firstneg([3 1 -2 5])");
    assert_eq!(scalar_of(&env, "a"), -2.0);
}

#[test]
fn break_inside_function_loop_is_fine() {
    let mut env = Environment::new();
    run_ok(
        &mut env,
        "function y = upto(n)\ns = 0\nfor k = 1:n\nif k > 3\nbreak\nend\ns = s + k\nend\ny = s\nend",
    );
    run_ok(&mut env, "r = upto(10)");
    assert_eq!(scalar_of(&env, "r"), 6.0);
}

#[test]
fn break_outside_loop_inside_function_errors() {
    let mut env = Environment::new();
    run_ok(&mut env, "function y = f(x); y = 1; break; y = 2; end");
    let e = expect_error(&mut env, "f(1)");
    assert!(e.message_en.contains("break/continue"), "{e:?}");
}

#[test]
fn nested_function_definition_is_rejected() {
    let mut env = Environment::new();
    let e = expect_error(
        &mut env,
        "function y = outer(x)\nfunction z = inner()\nz = 1\nend\ny = x\nend",
    );
    assert!(e.message_en.contains("Nested"), "{e:?}");
}

// ---------- هندل نام‌دار `@f` ----------

#[test]
fn handle_to_builtin() {
    let mut env = Environment::new();
    run_ok(&mut env, "f = @sin\nr = f(pi/2)");
    assert!((scalar_of(&env, "r") - 1.0).abs() < 1e-12);
}

#[test]
fn handle_to_user_function() {
    let mut env = Environment::new();
    run_ok(
        &mut env,
        "function y = sq(x); y = x^2; end\nh = @sq\nr = h(9)",
    );
    assert_eq!(scalar_of(&env, "r"), 81.0);
}

#[test]
fn handle_passed_as_argument() {
    let mut env = Environment::new();
    run_ok(
        &mut env,
        "function y = apply(g,x); y = g(x); end\nfunction y = cube(x); y = x^3; end",
    );
    run_ok(&mut env, "r = apply(@cube, 3)");
    assert_eq!(scalar_of(&env, "r"), 27.0);
}

#[test]
fn anon_fn_can_call_user_function() {
    let mut env = Environment::new();
    run_ok(
        &mut env,
        "function y = sq(x); y = x^2; end\ng = @(t) sq(t) + 1",
    );
    run_ok(&mut env, "r = g(4)");
    assert_eq!(scalar_of(&env, "r"), 17.0);
}

#[test]
fn handle_to_undefined_function_errors() {
    let mut env = Environment::new();
    let e = expect_error(&mut env, "h = @nosuchfn");
    assert_eq!(e.id, "MC:E1001_UNDEFINED_VARIABLE");
}

// ---------- ماتریس‌ها و عناصر درونی ----------

#[test]
fn function_receives_and_returns_matrices() {
    let mut env = Environment::new();
    run_ok(&mut env, "function c = addcols(a,b)\nc = a + b\nend");
    run_ok(&mut env, "M = addcols([1 2; 3 4], [10 20; 30 40])");
    match env.get("M") {
        Some(Value::Double(m)) => {
            assert_eq!(m.rows(), 2);
            assert_eq!(m.cols(), 2);
            assert_eq!(m.get(1, 1), 44.0);
        }
        other => panic!("expected matrix, got {other:?}"),
    }
}

#[test]
fn fndef_among_statements_registers_without_interrupting() {
    let mut env = Environment::new();
    let out = run_ok(
        &mut env,
        "a = 1\nfunction y = f(x); y = x + 1; end\nb = f(a)",
    );
    // فقط دو سطر نمایش: a و b (تعریف نمایش ندارد)
    assert_eq!(out.outputs.len(), 2);
    assert_eq!(scalar_of(&env, "b"), 2.0);
}

#[test]
fn builtin_still_works_alongside_user_fns() {
    let mut env = Environment::new();
    run_ok(&mut env, "function y = sq(x); y = x^2; end");
    run_ok(&mut env, "r = sqrt(sq(4))");
    assert_eq!(scalar_of(&env, "r"), 4.0);
}
