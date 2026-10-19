//! mc-interp — ارزیاب AST با سمانتیک MATLAB (سند ۰۳ بخش‌های ۵ و ۱۱).
//!
//! محدوده P0: محاسبات اسکالری `+ - * / ^`، بروز‌کست اسکالر↔ماتریس برای
//! `+ -`، جمع/تفریق هم‌شکل، `ans`، انتساب، نمایش `format short`.
//! اپراتورهای ماتریسی کامل (`*` ماتریسی، `\`، `.^`…) با dispatch درست در P1
//! به mc-num می‌روند (سند ۰۴ بخش ۳) — اینجا خطای صریح «بعدی» می‌گیرند.

#![cfg_attr(
    not(test),
    warn(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::todo,
        clippy::unimplemented
    )
)]

pub mod env;
pub mod eval;
pub mod format;

pub use env::Environment;
pub use eval::{eval_program, EvalOutcome, StmtOutput};
pub use format::{format_short, format_value};
