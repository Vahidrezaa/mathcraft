//! mc-cas — موتور سمبلیک MathCraft (سند ۰۵).
//!
//! گام 4.1 / CAS-0 (ROADMAP §Step 4.1): پایه‌ها — درخت عبارت، حساب دقیق،
//! کانونی‌سازی سطح-۰ (ساده‌سازی خودکار هنگام ساخت، سند ۰۵ §۳)، `subs` و
//! تبدیل به double.
//!
//! گام 4.2 / CAS-1 (ROADMAP §Step 4.2): `diff` کامل + خودآزمایی عددی
//! (`verify`)، `expand/collect/simplify` روی فرم نرمال چندجمله‌ای/گویا
//! (`poly`/`simplify`)، رندر LaTeX (`latex`)، سری‌سازی S-اجر (`sexpr`)،
//! توابع روی آرگومان سمبولیک (`fntable` + `Sym::fn_of`) و موتور e-graph
//! پشت پرچم `egraph` (سند ۰۵ §۳ سطح ۱ — ساده‌سازی قاعده‌مند).
//!
//! **تصمیم پیاده‌سازی (CAS-0؛ بازبینی بنیادین ۲۰۲۶-۱۱-۲۱):** حساب دقیق
//! روی `num-bigint`/`num-rational` خالصِ Rust — نه rug/GMP — و مبنای
//! تصمیم کیفیت است، نه سادگی بیلد (حکم کاربر: همهٔ بیلدهای رسمی در
//! GitHub Actions اجرا می‌شوند، سادگیِ Termux هرگز معیار تصمیم نیست و
//! pure Rust به شرط «دقیق و حرفه‌ای» پذیرفته است). شواهد: num-rational
//! ساختاراً کوپریم است و تبدیل مقیاس‌بزرگش درست است (تست پین‌شده در
//! sym.rs)؛ برای بارهای کلاس‌درس میلی‌ثانیه‌ای کار می‌کند (اندازه‌ها در
//! سند ۰۵ §۲)؛ ممیزی ۲۰۲۶-۱۱-۲۱ لایهٔ صداقتِ خود ما را هم سخت‌گیرانه
//! آزمود و نقص‌هایش را گرفت. GMP برای این پروژهٔ GPL-3.0 مانع مجوزی
//! ندارد (تصحیح روایت پیشین) — فقط هزینهٔ کراس‌کامپایل C برای ۴ ABI در
//! CI می‌افزاید؛ انواع درخت opaque اند و تعویضِ پسینی لایهٔ عددی ممکن است.
//!
//! قاعدهٔ طلایی این کرِیت: **صداقت در شکست** — هر عملی که نمادین کامل نمی‌شود
//! خطای روشن می‌دهد، هرگز جواب غلطِ مطمئن‌نما نه (سند ۰۰ اصل ۵).

#![cfg_attr(
    not(test),
    warn(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::todo,
        clippy::unimplemented
    )
)]

pub mod diff;
#[cfg(feature = "egraph")]
pub mod egraph;
pub mod fntable;
pub mod int;
pub mod latex;
pub mod poly;
pub mod sexpr;
pub mod simplify;
pub mod solve;
pub mod sym;
#[cfg(test)]
pub(crate) mod testkit;
pub mod verify;

pub use fntable::{eval_f64, is_symbolic_fn, SYMBOLIC_FNS};
pub use sexpr::{from_sexp, to_sexp};
pub(crate) use sym::as_small_int as sym_as_small_int;
pub use sym::{sym_display, CasError, Sym};
