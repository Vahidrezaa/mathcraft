//! AST — درخت مشترک (سند ۰۷ بخش ۳ بند ۴: CAS و عددی هر دو از همین AST
//! می‌خوانند؛ هیچ پارسر دومی ساخته نمی‌شود).
//!
//! P1 (سند ۰۳ بخش‌های ۴-۹): ماتریس لفظی، کولن، فراخوانی/اندیس‌گذاری،
//! مقایسه/منطق، ترانهاده، عملگرهای عنصری، توابع بی‌نام، کنترل جریان.

/// اپراتورهای یگانی.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnOp {
    Neg,
    Pos,
    /// منطقی NOT با سمانتیک متلب: نتیجه 0/1 و عملوند باید اسکالر باشد (سند ۰۳ §8).
    Not,
}

/// اپراتورهای دویی.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    /// `\` — mldivide (سند ۰۴ §3.2 درخت dispatch)
    Backslash,
    Pow,
    /// عنصری‌ها (سند ۰۳ §5)
    ElemMul,
    ElemDiv,
    ElemPow,
}

/// مقایسه — نتیجه همیشه 0/1 اسکالر (سند ۰۳ §11 بند 6: `==` بی‌رحم و آموزشی).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CmpOp {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

/// منطق دوگانه short-circuit — فقط اسکالر (سند ۰۳ §8).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogicOp {
    And,
    Or,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Num(f64),
    /// عدد موهومی `4i` — یعنی 0+4i (سند ۰۳ بخش ۳ — P2)
    Imag(f64),
    Str(String),
    Var {
        name: String,
        line: u32,
        col: u32,
    },
    /// `end` داخل اندیس — به آخرین عنصر آن بعد ترجمه می‌شود (سند ۰۳ §6).
    End {
        line: u32,
        col: u32,
    },
    Unary {
        op: UnOp,
        operand: Box<Expr>,
        /// موقعیت عملگر — برای خطاهای زمان اجرا (ROADMAP 2.11 — span سراسر AST).
        line: u32,
        col: u32,
    },
    Binary {
        op: BinOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
        /// موقعیت اپراتور — برای خطاهای زمان اجرا (سند ۰۲ بخش ۹).
        line: u32,
        col: u32,
    },
    Compare {
        op: CmpOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
        line: u32,
        col: u32,
    },
    Logic {
        op: LogicOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
        /// موقعیت عملگر && / || (ROADMAP 2.11).
        line: u32,
        col: u32,
    },
    /// ترانهاده: `conj=true` برای `'` و `false` برای `.'`
    Transpose {
        operand: Box<Expr>,
        conj: bool,
        line: u32,
        col: u32,
    },
    /// ماتریس لفظی `[...]` — ردیف‌ها با `;`/newline، سلول‌ها با `,`/فاصله (سند ۰۳ §4).
    Matrix {
        rows: Vec<Vec<Expr>>,
        line: u32,
        col: u32,
    },
    /// کولن `a:b` / `a:s:b` — سمانتیک دقیق سند ۰۳ §7.
    Range {
        start: Box<Expr>,
        step: Option<Box<Expr>>,
        end: Box<Expr>,
        /// موقعیت نخستین «:» (ROADMAP 2.11).
        line: u32,
        col: u32,
    },
    /// فراخوانی/اندیس — در زمان اجرا تصمیم: متغیر؟ builtin؟ تابع بی‌نام؟ (سمانتیک متلب §9)
    Call {
        name: String,
        args: Vec<Expr>,
        /// `[a,b] = f(...)` چند-خروجی — تعداد خروجی خواسته‌شده (nargout).
        nargout: usize,
        line: u32,
        col: u32,
    },
    /// تابع بی‌نام `@(x,y) body` — capture-by-value در لحظه ساخت (سند ۰۳ §9).
    AnonFn {
        params: Vec<String>,
        body: Box<Expr>,
        line: u32,
        col: u32,
    },
    /// هندل تابع نام‌دار `@f` — resolve در لحظهٔ ساخت؛ فراخوانی با هر تعداد
    /// آرگومان به تابع مقصد ارجاع می‌شود (سمانتیک متلب §9).
    FnHandle {
        name: String,
        line: u32,
        col: u32,
    },
    /// دسترسی به فیلد ساختار `s.f` (نامی) یا `s.(expr)` (پویا) — ROADMAP 2.9
    /// تسک ۱. اندیس‌گذاری پس از فیلد (`s.f(2)`) = `Expr::Index` روی پایهٔ فیلد
    /// (ROADMAP 2.10)؛ انتساب اندیس‌دار داخل فیلد = `AssignTarget::IndexedField`.
    Field {
        base: Box<Expr>,
        field: FieldName,
        line: u32,
        col: u32,
    },
    /// اندیس‌گذاری پسوندی روی نتیجهٔ هر عبارت (ROADMAP 2.10 — بند ۱۳ ممیزی):
    /// `[1 2 3](2)`، `ones(3,3)(2,2)`، `f(x)(1)`، `s.f(2)`. زنجیرهٔ اندیس روی
    /// «اندیسِ متغیر» (`x(1)(2)` — اصطلاح سلول) در ارزیاب خطای صادقانهٔ E4
    /// می‌گیرد (سلول = انبساط E4؛ متلب هم ()() را نمی‌پذیرد).
    Index {
        base: Box<Expr>,
        subs: Vec<Expr>,
        line: u32,
        col: u32,
    },
}

/// نام فیلد در دسترسی/انتساب فیلد — نامی `s.f` یا پویا `s.(expr)` (ROADMAP 2.9).
#[derive(Debug, Clone, PartialEq)]
pub enum FieldName {
    Name(String),
    /// عبارتی که به نام فیلد (char) ارزیابی می‌شود.
    Dynamic(Box<Expr>),
}

/// هدف انتساب: `x = ...` یا `A(i,j) = ...` (سند ۰۳ §6 اندیس‌گذاری) یا
/// `s.f = ...` / `s.(f) = ...` (ROADMAP 2.9 — پایه باید متغیر یا زنجیرهٔ فیلد باشد).
#[derive(Debug, Clone, PartialEq)]
pub enum AssignTarget {
    Var {
        name: String,
        line: u32,
        col: u32,
    },
    Indexed {
        name: String,
        subs: Vec<Expr>,
        line: u32,
        col: u32,
    },
    Field {
        base: Box<Expr>,
        field: FieldName,
        line: u32,
        col: u32,
    },
    /// `s.f(i) = v` / `s.(f)(i) = v` — اندیس‌گذاری داخل فیلد (ROADMAP 2.10؛
    /// تکمیل وعدهٔ پیام خطای گام 2.9). فیلد غایب با ماتریس خالی auto-viv
    /// می‌شود (رفتار متلب: `s.a(3) = 5` → `s.a = [0 0 5]`).
    IndexedField {
        base: Box<Expr>,
        field: FieldName,
        subs: Vec<Expr>,
        line: u32,
        col: u32,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    /// عبارت تنها — اگر `suppress` نباشد، در `ans` ذخیره و نمایش داده می‌شود.
    Expr {
        expr: Expr,
        suppress: bool,
    },
    /// انتساب ساده/اندیس‌دار.
    Assign {
        target: AssignTarget,
        expr: Expr,
        suppress: bool,
    },
    /// `if/elseif/else` — شرط‌ها باید اسکالر باشند (سند ۰۳ §8).
    If {
        branches: Vec<(Expr, Vec<Stmt>)>,
        else_body: Option<Vec<Stmt>>,
        line: u32,
        col: u32,
    },
    /// `for k = expr ... end` — پیمایش روی ستون‌های ماتریس (سند ۰۳ §8).
    For {
        var: String,
        iter_expr: Expr,
        body: Vec<Stmt>,
        line: u32,
        col: u32,
    },
    /// `while cond ... end`
    While {
        cond: Expr,
        body: Vec<Stmt>,
        line: u32,
        col: u32,
    },
    Break {
        line: u32,
        col: u32,
    },
    Continue {
        line: u32,
        col: u32,
    },
    /// `return` — خروج از تابع جاری یا توقف اسکریپت (سمانتیک متلب).
    Return {
        line: u32,
        col: u32,
    },
    /// تعریف تابع فایلی — گزارهٔ سطح برنامه (فایل‌های .m):
    /// `function y = sq(x) ... end` / `function [a,b] = f(x) ... end` /
    /// `function show(x) ... end` (بدون خروجی). توابع محلی بعد از تابع اصلی
    /// نیز با همین گزاره می‌آیند.
    FnDef {
        name: String,
        /// خروجی‌ها به ترتیب — `function y` → ["y"]؛ بدون خروجی → خالی.
        outputs: Vec<String>,
        params: Vec<String>,
        body: Vec<Stmt>,
        line: u32,
        col: u32,
    },
    /// انتساب چندخروجی `[a,b] = f(...)` — سمانتیک متلب §9: هر هدف، یکی از
    /// خروجی‌های تابع را می‌گیرد (ترتیب خروجی‌های تعریف).
    MultiAssign {
        targets: Vec<OutTarget>,
        expr: Expr,
        suppress: bool,
    },
}

/// هدف یک خروجی در انتساب چندتایی `[a,b] = f(...)` — نام با موقعیت برای خطاها.
#[derive(Debug, Clone, PartialEq)]
pub struct OutTarget {
    pub name: String,
    pub line: u32,
    pub col: u32,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Program {
    pub stmts: Vec<Stmt>,
}
