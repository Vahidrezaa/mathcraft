//! mc-plot — مدل داده رسم (فقط داده؛ ترسیم ۱۰۰٪ در Dart است — تصمیم D-10).
//!
//! اسکلت P0: طبق وابستگی حیاتی سند ۰۷ بخش ۳ بند ۲، «PlotSpec قرارداد است نه
//! فیچر» — اما قطعی‌شدن نهایی قرارداد قبل از P3-a انجام می‌شود. در P0،
//! **نسخه پل (DTO) قرارداد رسم در `mc-ffi::api::PlotSpec`** تعریف و نسخه‌دار
//! شده است (`PLOT_CONTRACT_VERSION`) تا UI از همین حالا روی آن بسازد.
//! در P3-a این کرِیت مدل کامل موتور (سری‌های خام ۱۰۰k نقطه، meshgrid،
//! مثلث‌بازی 3D، نرمال‌ها، tickهای 1-2-5) و LTTB decimation را می‌گیرد و
//! خروجی را به DTO سند ۰۴ بخش ۸ تبدیل می‌کند.
//!
//! مرزها (سند ۰۲ بخش ۷):
//! - Rust: داده خام → decimation LTTB تا ≤ 4000 نقطه/سری → هندسه محورها → PlotSpec
//! - Dart: فقط ترسیم CustomPainter + تعامل (زوم/پن بدون FFI در حلقه فریم)
//! - داده کامل هرگز به UI منتقل نمی‌شود.

#![cfg_attr(
    not(test),
    warn(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::todo,
        clippy::unimplemented
    )
)]

/// نسخه قرارداد رسم — با هر تغییر شکانده (breaking) باید بالا برود.
pub const PLOT_CONTRACT_VERSION: u32 = 1;
