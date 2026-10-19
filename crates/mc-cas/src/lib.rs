//! mc-cas — موتور سمبلیک MathCraft (سند ۰۵).
//!
//! اسکلت P0: پرشدن این کرِیت در سه زیرمایل‌استون CAS-1/2/3 است (سند ۰۵ بخش ۱۱،
//! تصمیم D-05). معماری هدف:
//!
//! ```text
//! SymTree/     درخت عبارت با حساب دقیق rug/GMP (تصمیم D-12: 1/3+1/6 دقیقاً 1/2)
//! simplify/    سه سطح: خودکار + egg e-graphs (timeout 100ms) + normal forms
//! diff/        مشتق بازگشتی + خودآزمایی diff-verify اجباری هر نتیجه
//! int/         پایپ‌لاین ۸ مرحله‌ای Risch-lite (سند ۰۵ بخش ۵)
//! limit/ taylor/ solve/ dsolve/   زیرمجموعه دقیق ترم ۱-۲
//! latex/       رندر LaTeX برای flutter_math_fork (سند ۰۵ بخش ۱۰)
//! ```
//!
//! قاعده طلایی این کرِیت: **صداقت در شکست** — اگر CAS نتوانست، صادقانه اعلام و
//! fallback عددی پیشنهاد می‌شود؛ هرگز نتیجه اشتباهِ مطمئن‌نما (سند ۰۰ اصل ۵).

#![cfg_attr(
    not(test),
    warn(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::todo,
        clippy::unimplemented
    )
)]
