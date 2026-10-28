//! Job signals — progress + cancel for long-running jobs (ROADMAP 2.3, سند ۰۲ §3–4).
//!
//! **Invariant اجرای سریالی (سند ۰۲ §4):** در هر لحظه دقیقاً «یک» job در حال
//! ارزیابی است — قفل موتور در mc-ffi (و حلقهٔ REPL تک‌نخ≐ی mc-cli) این را
//! تضمین می‌کند؛ پس یک اسلات سراسری کافی است و مسیر داغ، lock-free می‌ماند.
//! صاحبِ اسلات mc-ffi است: قبل از ارزیابی [`install`]، بعد از آن [`clear`]
//! (از طریق گارد Drop). حلگرهای mc-num و حلقهٔ ارزیابی mc-interp فقط
//! [`cancel_requested`] و [`report`] را می‌خوانند.
//!
//! **تست‌ها:** آزمون‌هایی که کانتکست نصب می‌کنند باید خودشان سریالی باشند
//! (cargo test پیش‌فرض موازی است) — در mc-ffi همهٔ سناریوها پشت یک #[test]
//! اجرا می‌شوند؛ آزمون‌هایی که نصب نمی‌کنند به hooks بی‌اثر می‌خورند (no-op).

use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::thread::ThreadId;

/// درصد در «بیس‌پوینت» (0..=10_000) — عدد صحیح، بدون رقابت در خواندن اتمیک.
/// نشانگر «کسر بی‌معنا» (ارزیابی خطی/مجموع نامعلوم) — UI نوار نامعین می‌کشد.
pub const INDETERMINATE_BP: u32 = u32::MAX;

/// سیگنال‌های job در حال اجرا — مالک: mc-ffi؛ خواننده: mc-interp/mc-num.
pub struct JobSignals {
    /// شناسهٔ job (ترتیب نشست — همان job_id قرارداد EvalResult).
    pub job_id: AtomicU64,
    /// true = درخواست لغو صادر شده (CancellationToken سند ۰۲ §4).
    pub cancel: AtomicBool,
    /// برچسب مرحلهٔ فعلی — مثل `eval` / `ode45` / `ode23` / `integral`.
    pub stage: Mutex<String>,
    /// درصد × ۱۰۰ (0..=10_000) یا [`INDETERMINATE_BP`].
    pub percent_bp: AtomicU32,
    /// نخِ ارزیابِ این job — در install ثبت می‌شود؛ چک لغو/گزارش فقط از
    /// همین نخ معنادار است (هم ایزوله‌سازی تست‌های موازی cargo، هم
    /// جلوگیری از آلودگی cross-thread در زمان اجرا).
    eval_thread: ThreadId,
}

impl JobSignals {
    /// کانتکست تازه: مرحلهٔ `eval`، درصد نامعین، بدون لغو؛ نخِ فراخوان = نخ ارزیاب.
    pub fn new(job_id: u64) -> Self {
        Self {
            job_id: AtomicU64::new(job_id),
            cancel: AtomicBool::new(false),
            stage: Mutex::new("eval".to_owned()),
            percent_bp: AtomicU32::new(INDETERMINATE_BP),
            eval_thread: std::thread::current().id(),
        }
    }

    /// اسنپ‌شات رشته‌ای برای رویداد Progress و تست‌ها.
    pub fn snapshot(&self) -> JobSnapshot {
        JobSnapshot {
            job_id: self.job_id.load(Ordering::Relaxed),
            cancelled: self.cancel.load(Ordering::Relaxed),
            stage: self
                .stage
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .clone(),
            percent_bp: self.percent_bp.load(Ordering::Relaxed),
        }
    }
}

/// اسنپ‌شات ثابت از وضعیت یک job (برای رویداد `Progress{job_id, stage, percent}`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JobSnapshot {
    pub job_id: u64,
    pub cancelled: bool,
    pub stage: String,
    pub percent_bp: u32,
}

impl JobSnapshot {
    /// درصد 0.0..=1.0 — None یعنی نامعین (نوار پیشرفت نامعین UI).
    pub fn fraction(&self) -> Option<f64> {
        match self.percent_bp {
            INDETERMINATE_BP => None,
            bp => Some((bp.min(10_000) as f64) / 10_000.0),
        }
    }
}

static CURRENT: RwLock<Option<Arc<JobSignals>>> = RwLock::new(None);

/// نصب سیگنال‌های job پیش از ارزیابی — فقط صاحب قفل موتور صدا می‌زند.
/// اگر جایی نصبِ هم‌زمان دوم رخ دهد یعنی invariant سریالی شکسته شده؛ برای
/// جلوگیری از panic در مسیر محصول، «آخرین نویسنده برنده» است (مستند در ماژول).
pub fn install(signals: Arc<JobSignals>) -> Option<Arc<JobSignals>> {
    CURRENT
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .replace(signals)
}

/// برداشتن کانتکست پایان job — گارد Drop در mc-ffi صدا می‌زند.
pub fn clear() -> Option<Arc<JobSignals>> {
    CURRENT
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .take()
}

/// کانتکست فعلی (برای cancel_job و تست‌ها) — Arc ارزان.
pub fn current() -> Option<Arc<JobSignals>> {
    CURRENT
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone()
}

/// چک همکاری‌گر لغو — یک load اتمیک؛ حلقه‌ها هر تکرار، مفسر هر
/// `CANCEL_CHECK_EVERY` گزاره (mc-interp) صدا می‌زنند. فقط نخِ ارزیابِ job
/// جواب مثبت می‌گیرد (تست‌های موازی و نخ‌های دیگر مصون‌اند).
pub fn cancel_requested() -> bool {
    let Some(ctx) = current() else {
        return false;
    };
    if ctx.eval_thread != std::thread::current().id() {
        return false;
    }
    ctx.cancel.load(Ordering::Relaxed)
}

/// گزارش پیشرفت حلگر — بدون کانتکست یا از نخِ غیرارزیاب no-op
/// (تست‌های mc-num، اجرای دسته‌ای mc-cli).
/// `frac ∈ [0,1]` کلمپ می‌شود؛ None یعنی این مرحله کسر معنادار ندارد.
pub fn report(stage: &str, frac: Option<f64>) {
    let Some(ctx) = current() else {
        return;
    };
    if ctx.eval_thread != std::thread::current().id() {
        return;
    }
    if let Ok(mut slot) = ctx.stage.lock() {
        *slot = stage.to_owned();
    }
    let bp = match frac {
        None => INDETERMINATE_BP,
        Some(f) => {
            if !f.is_finite() {
                INDETERMINATE_BP
            } else {
                ((f.clamp(0.0, 1.0) * 10_000.0) as u32).min(10_000)
            }
        }
    };
    ctx.percent_bp.store(bp, Ordering::Relaxed);
}
