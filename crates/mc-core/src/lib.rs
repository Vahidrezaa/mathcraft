//! mc-core — پایه‌ترین کرِیت MathCraft.
//!
//! قواعد این کرِیت (سند ۰۲ بخش ۵، تصمیم D-11):
//!
//! 1. **Column-major، بدون استثنا** — سازگاری سمانتیک با MATLAB
//!    (`reshape`، `A(:)`، `for k = A`، اندیس خطی همه بر این چیدمان تعریف شده‌اند؛ سند ۰۳ بخش ۱۲).
//! 2. **Copy-on-Write** — داده ماتریس در `Arc<ThinVec<f64>>` نگه داشته می‌شود؛
//!    پاس‌دادن به توابع O(1) است و فقط هنگام نوشتن clone می‌شود
//!    (رفتار value-semantics متلب با هزینه reference).
//! 3. **بدون وابستگی platform و بدون وابستگی به کرِیت‌های دیگر MathCraft** —
//!    این کرِیت باید روی هر هدفی (اندروید/دسکتاپ/WASM) بدون تغییر کامپایل شود (سند ۰۲ بخش ۱۱).
//! 4. **خطا یک مقدار است** — `McError` ساختاریافته دوزبانه (سند ۰۲ بخش ۹)؛
//!    در مسیر محصول panic وجود ندارد.

#![cfg_attr(
    not(test),
    warn(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::todo,
        clippy::unimplemented
    )
)]

pub mod error;
pub mod matrix;
pub mod value;

pub use error::{ids, McError};
pub use matrix::Matrix;
pub use value::Value;
