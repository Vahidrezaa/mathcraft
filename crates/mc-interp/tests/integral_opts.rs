//! integral با گزینه‌های نامی — RelTol/AbsTol/Waypoints (ROADMAP 2.6، تسک ۴).
//!
//! مرجع‌ها تحلیلی‌اند (∫x² = 1/3، انتگرال پله‌ای = 1.5)؛ سطر «integral
//! گزینه‌ها» در COMPATIBILITY §4.5 با این کامیت بسته می‌شود. انحراف مستند:
//! Waypoints با مرز بی‌نهایت پشتیبانی نمی‌شود (خطای صادقانه).

use mc_interp::{eval_program, Environment};
use mc_parser::parse;

fn eval_scalar(src: &str, var: &str) -> f64 {
    let prog = parse(src).expect("parse");
    let mut env = Environment::new();
    let out = eval_program(&prog, &mut env);
    assert!(out.error.is_none(), "engine error: {:?}", out.error);
    match env.get(var) {
        Some(mc_core::Value::Double(m)) if m.is_scalar() => m.get(0, 0),
        other => panic!("'{var}' is not a scalar: {other:?}"),
    }
}

fn expect_error(src: &str, what: &str) {
    let prog = parse(src).expect(what);
    let mut env = Environment::new();
    let out = eval_program(&prog, &mut env);
    assert!(
        out.error.is_some(),
        "expected honest error for {what}: {out:?}"
    );
}

#[test]
fn integral_reltol_abstol_accepted() {
    let q = eval_scalar(
        "q = integral(@(x) x.^2, 0, 1, 'RelTol', 1e-12, 'AbsTol', 1e-12);\n",
        "q",
    );
    assert!((q - 1.0 / 3.0).abs() < 1e-12, "got {q}");
}

#[test]
fn integral_default_path_unchanged() {
    // گارد رگرسیون: بدون گزینه مثل قبل (p2_04/p2_05 کورپوس)
    let q = eval_scalar("q = integral(@(x) x.^2, 0, 1);\n", "q");
    assert!((q - 1.0 / 3.0).abs() < 1e-10, "got {q}");
}

#[test]
fn integral_waypoints_piecewise_exact() {
    // پله: 1 روی [0,0.5) و 2 روی [0.5,1] → 1.5؛ مرجع تحلیلی
    let q = eval_scalar(
        "q = integral(@(x) 1 + (x >= 0.5), 0, 1, 'Waypoints', [0.5]);\n",
        "q",
    );
    assert!((q - 1.5).abs() < 1e-10, "got {q}");
}

#[test]
fn integral_waypoints_abs_kink() {
    let q = eval_scalar("q = integral(@(x) abs(x), -1, 1, 'Waypoints', 0);\n", "q");
    assert!((q - 1.0).abs() < 1e-10, "got {q}");
}

#[test]
fn integral_waypoints_unsorted_vector() {
    // ترتیب نقاط بی‌اهمیت است (doc متلب) — نقاط تکراری/مرزی هم بی‌اثر
    let q = eval_scalar(
        "q = integral(@(x) x.^2, 0, 1, 'Waypoints', [1 0.5 0 0.5]);\n",
        "q",
    );
    assert!((q - 1.0 / 3.0).abs() < 1e-10, "got {q}");
}

#[test]
fn integral_waypoints_outside_honest_error() {
    expect_error(
        "q = integral(@(x) x.^2, 0, 1, 'Waypoints', [0.5 2.0]);\n",
        "waypoint outside interval",
    );
}

#[test]
fn integral_waypoints_infinite_bounds_honest_error() {
    expect_error(
        "q = integral(@(x) exp(-x), 0, Inf, 'Waypoints', [1]);\n",
        "waypoints + infinite bounds",
    );
}

#[test]
fn integral_unknown_option_honest_error() {
    expect_error(
        "q = integral(@(x) x.^2, 0, 1, 'MaxIntervalCount', 50);\n",
        "unknown option",
    );
    expect_error(
        "q = integral(@(x) x.^2, 0, 1, 'RelTol');\n",
        "dangling option",
    );
    expect_error(
        "q = integral(@(x) x.^2, 0, 1, 'RelTol', 0);\n",
        "non-positive RelTol",
    );
}
