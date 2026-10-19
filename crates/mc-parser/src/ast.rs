//! AST — درخت عبارت مشترک (سند ۰۷ بخش ۳ بند ۴: CAS و عددی هر دو از همین AST
//! می‌خوانند؛ هیچ پارسر دومی ساخته نمی‌شود).

/// اپراتورهای یگانی P0 (سند ۰۳ بخش ۵ سطح ۲: unary `+ -`؛ `~` در P1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnOp {
    Neg,
    Pos,
}

/// اپراتورهای دویی P0 (سند ۰۳ بخش ۵ سطوح ۱ تا ۴).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Num(f64),
    Var {
        name: String,
        line: u32,
        col: u32,
    },
    Unary {
        op: UnOp,
        operand: Box<Expr>,
    },
    Binary {
        op: BinOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
        /// موقعیت اپراتور — برای خطاهای زمان اجرا (سند ۰۲ بخش ۹).
        line: u32,
        col: u32,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    /// عبارت تنها: `1+1` — اگر `suppress` نباشد، در `ans` ذخیره و نمایش داده می‌شود.
    Expr { expr: Expr, suppress: bool },
    /// انتساب: `x = expr` — `=` راست‌گرا و بدون مقدار بازگشتی (سند ۰۳ بخش ۵ سطح ۱۰).
    Assign {
        name: String,
        expr: Expr,
        suppress: bool,
        line: u32,
        col: u32,
    },
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Program {
    pub stmts: Vec<Stmt>,
}
