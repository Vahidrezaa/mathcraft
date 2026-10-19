//! mc-ffi — قرارداد ارتباطی Flutter ↔ Rust (سند ۰۲ بخش ۳، تصمیم D-02).
//!
//! این تنها کرِیتی است که با flutter_rust_bridge v2 صحبت می‌کند. قواعد:
//! 1. پیام‌های تایپ‌دار، نه رشته خام — JSON روی مسیر داغ ممنوع.
//! 2. خطا یک مقدار است — `McError` ساختاریافته دوزبانه (سند ۰۲ بخش ۹).
//! 3. سمانتیک اجرا سریالی: یک workspace جهانی با Mutex (thread pool کامل با
//!    `runScript`/cancel در P2 می‌آید — سند ۰۲ بخش ۴).
//! 4. `api_version()` در startup چک می‌شود؛ ناسازگاری = پیام واضح، نه کرش.

#![cfg_attr(
    not(test),
    warn(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::todo,
        clippy::unimplemented
    )
)]

pub mod api;

// این خط در انتهای فایل است، نه ابتدای آن: تزریق پیش‌فرض FRB بالای کامنت‌های
// داخلی می‌رود و Rust با E0753 شکست می‌خورد (درس P0.2.4). FRB با دیدن همین
// خط، دوباره تزریق نمی‌کند.
// کد تولیدی FRB ماشین‌ساخت است — lint های سخت‌گیرانه crate (unwrap/unimplemented و…)
// با allow سطح ماژول exempt می‌شود؛ این allow در lib.rs می‌ماند و با regenerate نمی‌پرد.
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::todo,
    clippy::unimplemented
)]
mod frb_generated; /* AUTO INJECTED BY flutter_rust_bridge. This line may not be accurate, and you can change it according to your needs. */
