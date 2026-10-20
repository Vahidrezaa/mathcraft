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

// Lint waivers for the GENERATED glue live here (module-level), NOT inside the
// generated file itself — so `flutter_rust_bridge_codegen generate` output stays
// byte-stable and the CI "glue is up to date" gate can verify it with git diff.
// (Supersedes the old "re-apply the manual header after regeneration" ritual.)
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::unimplemented)]
mod frb_generated; /* AUTO INJECTED BY flutter_rust_bridge. This line may not be accurate, and you can change it according to your needs. */

pub mod api;
