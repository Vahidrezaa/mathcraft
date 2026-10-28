//! تست‌های لغو همکاری‌گر + پیشرفت (ROADMAP 2.3 — سند ۰۲ §3–4).
//!
//! سناریوهای قفل‌شده:
//! - `while(1) end` و `for i=1:1e9; end` قابل لغو هستند (چک هر تکرار)
//! - کد خطی بلند از چک هر N گزاره لغو می‌شود (N = CANCEL_CHECK_EVERY)
//! - پرچم ode45 از کانتکست job plumb می‌شود (bi_ode45)
//! - progress حلگر در کانتکست منتشر می‌شود (stage = "ode45")
//! - workspace پس از لغو وسط حلقه consistent است (مثل stop متلب)
//!
//! **قید سریالی‌بودن:** کانتکست job سراسری است (invariant اجرای سریالی
//! سند ۰۲ §4) — همهٔ سناریوها با قفل [`SERIAL`] محافظت می‌شوند؛ نصب کانتکست
//! نخ‌آگاه است (mc-core::job) پس آزمون‌های بدون نصب هیچ تداخلی نمی‌بینند.

use std::sync::{Arc, Mutex, OnceLock};

use mc_core::job::{self, JobSignals};
use mc_core::Value;
use mc_interp::{eval_program, Environment};
use mc_parser::parse;

/// قفل سریال‌سازی آزمون‌های نصب‌کنندهٔ کانتکست (ماژول‌های تست cargo موازی‌اند).
static SERIAL: OnceLock<Mutex<()>> = OnceLock::new();

fn serial_guard() -> std::sync::MutexGuard<'static, ()> {
    SERIAL
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

struct CtxGuard(Arc<JobSignals>);
impl CtxGuard {
    fn install() -> Self {
        let ctx = Arc::new(JobSignals::new(1));
        job::install(ctx.clone());
        Self(ctx)
    }
}
impl Drop for CtxGuard {
    fn drop(&mut self) {
        job::clear();
    }
}

fn run(env: &mut Environment, src: &str) -> mc_interp::EvalOutcome {
    match parse(src) {
        Ok(p) => eval_program(&p, env),
        Err(e) => mc_interp::EvalOutcome {
            outputs: Vec::new(),
            error: Some(e),
        },
    }
}

#[test]
fn while_one_cancelled_from_other_thread() {
    let _guard = serial_guard();
    let _ctx = CtxGuard::install();

    // لغو ۱۰۰ms پس از شروع ارزیابی — حلقهٔ بی‌نهایت باید همان‌جا تمام شود
    let canceler_ctx = Arc::clone(&_ctx.0);
    let canceller = std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(100));
        canceler_ctx
            .cancel
            .store(true, std::sync::atomic::Ordering::Relaxed);
    });

    let t0 = std::time::Instant::now();
    let mut env = Environment::new();
    let out = run(&mut env, "while 1, x = 1; end");
    canceller.join().expect("canceller thread");

    let err = out.error.expect("while(1) must end with cancel error");
    assert_eq!(err.id, "MC:E4002_CANCELLED");
    assert!(t0.elapsed() < std::time::Duration::from_secs(10));

    // workspace consistent (مثل stop متلب): x با آخرین مقدار کامل باقی است
    match env.get("x") {
        Some(Value::Double(m)) if m.is_scalar() => assert_eq!(m.get(0, 0), 1.0),
        other => panic!("x must remain a consistent scalar, got {other:?}"),
    }
    // هیچ خروجی جزئی نمایشی تولید نشده (بدنه انتساب ساکت با ; است)
    assert!(out.outputs.is_empty());
}

#[test]
fn for_loop_cancelled_at_first_iteration() {
    let _guard = serial_guard();
    let _ctx = CtxGuard::install();
    _ctx.0
        .cancel
        .store(true, std::sync::atomic::Ordering::Relaxed);

    // چک هر تکرار حلقه باید «پیش از اولین env.set» بگیرد.
    // نکتهٔ ثبت‌شده (STATUS §3.4): بازهٔ کولن در for امروز «خوددار» ساخته
    // می‌شود (`1:1e9` = ماتریس کامل؛ سقف حافظه = محدودهٔ گام 2.5) — این تست
    // بازهٔ کوچک با لغو پیش‌ست‌شده است.
    let mut env = Environment::new();
    let out = run(&mut env, "for i = 1:1000, end");
    let err = out.error.expect("pre-cancelled for must error");
    assert_eq!(err.id, "MC:E4002_CANCELLED");
    // i هرگز ست نشده (اولین چک پیش از env.set است)
    assert!(env.get("i").is_none());
}

#[test]
fn straight_line_code_cancelled_at_statement_budget() {
    let _guard = serial_guard();
    let _ctx = CtxGuard::install();
    _ctx.0
        .cancel
        .store(true, std::sync::atomic::Ordering::Relaxed);

    // ۲۰۰ گزارهٔ ساکت — چک گزاره‌ای (هر ۱۲۸) باید وسط دنباله بگیرد
    let mut src = String::new();
    for k in 0..200 {
        src.push_str(&format!("x{k} = {k};\n"));
    }
    let mut env = Environment::new();
    let out = run(&mut env, &src);
    let err = out.error.expect("pre-cancelled long script must error");
    assert_eq!(err.id, "MC:E4002_CANCELLED");
    // پس از لغو وسط دنباله، هم گزاره‌های قبل اجرا شده‌اند و هم بعدی‌ها نه —
    // هر دو سمت باید موجود باشند (لغو «وسط» است، نه ابتدای برنامه)
    assert!(env.get("x0").is_some(), "statements before checkpoint ran");
    assert!(
        env.get("x199").is_none(),
        "statements after checkpoint skipped"
    );
}

#[test]
fn ode45_cancel_flag_plumbed_from_job_ctx() {
    let _guard = serial_guard();
    let _ctx = CtxGuard::install();
    _ctx.0
        .cancel
        .store(true, std::sync::atomic::Ordering::Relaxed);

    // چک اولیهٔ solve_adaptive با پرچم از پیش ست‌شده فوراً خطا می‌دهد
    let mut env = Environment::new();
    let out = run(&mut env, "[T, Y] = ode45(@(x, y) -y, [0 1e6], 1);");
    let err = out.error.expect("pre-cancelled ode45 must error");
    assert_eq!(err.id, "MC:E4002_CANCELLED");
    assert!(env.get("T").is_none() && env.get("Y").is_none());
}

#[test]
fn ode45_publishes_progress_into_job_ctx() {
    let _guard = serial_guard();
    let _ctx = CtxGuard::install();

    // اجرای کامل با گام‌های زیاد (MaxStep کوچک → >۵۰ گام پذیرفته‌شده قطعی)
    // → حداقل یک گزارش هر-۵۰-گام منتشر شده
    let mut env = Environment::new();
    let out = run(
        &mut env,
        "[T, Y] = ode45(@(x, y) -y, [0 100], 1, 'MaxStep', 0.01);",
    );
    assert!(out.error.is_none(), "ode45 must succeed: {:?}", out.error);
    let snap = _ctx.0.snapshot();
    assert_eq!(snap.stage, "ode45");
    let frac = snap.fraction().expect("progress fraction must be set");
    // اجرای کامل → آخرین گزارش نزدیک ۱ است (چک هر ۵۰ گام؛ آخرین گزارش ممکن است
    // کمی قبل از پایان باشد)
    assert!(
        (0.0..=1.0).contains(&frac) && frac > 0.0,
        "unexpected fraction {frac}"
    );
    // بدون کانتکست دوم، هیچ چیز باقی نمانده باشد
}

#[test]
fn no_ctx_all_hooks_are_noop() {
    // بدون نصب کانتکست: رفتار کاملاً مثل قبل (تست‌های بدون لغو مصون‌اند)
    job::clear();
    let mut env = Environment::new();
    let out = run(&mut env, "for i = 1:300, s = i; end\ns");
    assert!(out.error.is_none(), "no-ctx loops must run freely");
    assert_eq!(out.outputs.len(), 1);
}
