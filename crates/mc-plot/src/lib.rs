//! mc-plot — مدل داده رسم (فقط داده؛ ترسیم ۱۰۰٪ در Dart است — تصمیم D-10).
//!
//! معماری (سند ۰۲ §7 + سند ۰۴ §8، فریز نسخهٔ v2 در P3-a):
//!
//! ```text
//! مفسر (mc-interp)                این کرِیت                     UI (Dart)
//! ───────────────────             ───────────────────────       ─────────────
//! توابع رسم گزاره‌ای        →     PlotOp (گرام عملیات)
//! (خالص؛ بدون حالت)                FigureState = fold مرتب   →   snapshot (قرینهٔ
//!                                  + LTTB ≤ 4000 نقطه/سری        DTO انتقالی)
//!                                  + تیک‌های 1-2-5 و %.4g     ←  get_plots (pull)
//! ```
//!
//! مرزها (سند ۰۲ بخش ۷):
//! - Rust: داده خام → decimation LTTB → هندسه محور/تیک → snapshot
//! - Dart: فقط ترسیم CustomPainter + تعامل (زوم/پن 2D بدون FFI در حلقهٔ فریم؛
//!   تبدیل affine روی مختصات کشیده‌شده)
//! - دادهٔ کامل هرگز به UI منتقل نمی‌شود؛ هر re-decimation از دادهٔ خام.
//!
//! گرام داخلی ([`spec::PlotOp`]) نسخه‌بندی نمی‌شود (داخلی است)؛ نسخهٔ
//! قرارداد فقط روی DTO انتقالی `mc_ffi::api` است (`PLOT_CONTRACT_VERSION`).

#![cfg_attr(
    not(test),
    warn(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::todo,
        clippy::unimplemented
    )
)]

pub mod figure;
pub mod histogram;
pub mod linespec;
pub mod lttb;
pub mod palette;
pub mod session;
pub mod spec;
pub mod ticks;

/// نسخهٔ قرارداد رسم (روی DTO در `mc_ffi::api` تعریف می‌شود؛ اینجا فقط
/// مرجع مفهومی) — با هر تغییر شکاندهٔ DTO باید بالا برود. v1 = پل P0؛
/// v2 = فریز P3-a (رنگ معنایی/دقیق، تیک‌ها، Area/Fill/ErrorBar، …).
pub const PLOT_CONTRACT_VERSION: u32 = 2;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contract_version_is_two_after_p3a_freeze() {
        assert_eq!(PLOT_CONTRACT_VERSION, 2);
    }
}
