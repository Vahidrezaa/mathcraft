//! mc-parser — Lexer، پارسر Pratt و AST (سند ۰۳).
//!
//! تصمیم D-09: پارسر **دست‌نویس** (Recursive Descent + Pratt) تا پیام‌های خطای
//! دقیق فارسی/انگلیسی با موقعیت (خط، ستون) ممکن شود.
//!
//! محدوده P0 (سند ۰۷): اعداد، شناسه‌ها، `+ - * / ^`، پرانتز، انتساب،
//! جداکننده‌های `,` `;`، کامنت `%` و ادامه خط `...`. اولویت‌ها دقیقاً طبق
//! جدول سند ۰۳ بخش ۵ — توان **چپ‌گرا** (رفتار MATLAB): `2^3^2 = 64`.
//! هر چیزی که هنوز در محدوده نیست، خطای صریح `MC:E9xxx_UNSUPPORTED` می‌دهد —
//! نه رفتار غلط بی‌صدا (سند ۰۳ بخش ۱ بند ۲).

#![cfg_attr(
    not(test),
    warn(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::todo,
        clippy::unimplemented
    )
)]

pub mod ast;
pub mod lexer;
pub mod parser;

pub use ast::{AssignTarget, BinOp, CmpOp, Expr, LogicOp, OutTarget, Program, Stmt, UnOp};
pub use parser::parse;
