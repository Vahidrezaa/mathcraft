//! Parser — پارسر Pratt با اولویت‌های دقیق متلب (سند ۰۳ بخش ۵).
//!
//! اولویت‌ها (بالا به پایین):
//! `() [] .  ` `+' `.'` ترانهاده | `^` و `.^` چپ‌گرا | یگانی `- + ~` |
//! `.* ./ \ /` | `+ -` | `:` (کولن) | مقایسه‌ها | `&&` | `||`
//!
//! نکات حیاتی:
//! - توان چپ‌گرا است: `2^3^2 = 64` (سند ۰۳ بخش ۵ — تفاوت با ریاضی متعارف).
//! - یگاری توان با ضرب: `-3^2 = -9` ولی `2*-3^2 = 2*(-(3^2))`... سمانتیک متلب:
//!   یگانی فقط بعد از عملگرها/شروع، پیش از پایه توان اعمال می‌شود.
//! - ابهام ماتریس: `[1 -2]` دو عنصر، `[1 - 2]` یک عنصر (space flags لکسر).
//! - `end` در اندیس = Expr::End.
//! - کولن زنجیره‌ای: `a:b:c:d` = (a:b:c):d (سمانتیک متلب).

use crate::ast::*;
use crate::lexer::{tokenize, Tok, Token};
use mc_core::{ids, McError};

/// کلیدواژه‌های رزرو — مثل متلب نمی‌توان نام متغیر باشند (سند ۰۳ بخش ۲).
const KEYWORDS: &[&str] = &[
    "if", "elseif", "else", "end", "for", "while", "break", "continue", "return",
];

pub fn parse(src: &str) -> Result<Program, McError> {
    let tokens = tokenize(src)?;
    let mut p = Parser {
        toks: tokens,
        pos: 0,
        mat_sign_ctx: false,
        fn_depth: 0,
    };
    p.parse_program()
}

struct Parser {
    toks: Vec<Token>,
    pos: usize,
    /// داخل ماتریس لفظی `[...]` هستیم؟ — قانون ابهام علامت متلب (سند ۰۳ §4):
    /// فقط داخل براکت فعال است؛ بیرون آن `1 -2` همیشه تفریق است.
    mat_sign_ctx: bool,
    /// عمق تعریف تابع فایلی — تعریف تو در تو ممنوع (سمانتیک متلب P1).
    fn_depth: usize,
}

impl Parser {
    // ---------- ابزارها ----------
    fn peek(&self) -> &Token {
        &self.toks[self.pos.min(self.toks.len() - 1)]
    }

    fn peek_at(&self, off: usize) -> &Token {
        &self.toks[(self.pos + off).min(self.toks.len() - 1)]
    }

    fn bump(&mut self) -> Token {
        let t = self.toks[self.pos.min(self.toks.len() - 1)].clone();
        if self.pos < self.toks.len() - 1 {
            self.pos += 1;
        }
        t
    }

    fn at(&self, tok: &Tok) -> bool {
        std::mem::discriminant(&self.peek().tok) == std::mem::discriminant(tok)
    }

    fn eat(&mut self, tok: &Tok) -> bool {
        if self.at(tok) {
            self.bump();
            true
        } else {
            false
        }
    }

    fn expect(&mut self, tok: &Tok, what_en: &str, what_fa: &str) -> Result<Token, McError> {
        if self.at(tok) {
            Ok(self.bump())
        } else {
            let t = self.peek().clone();
            Err(McError::new(
                ids::PARSE_ERROR,
                t.line,
                t.col,
                format!("Expected {what_en}, found {}.", t.describe()),
                format!("{what_fa}، ولی {} یافت شد.", t.describe()),
            ))
        }
    }

    fn err(&self, t: &Token, en: impl Into<String>, fa: impl Into<String>) -> McError {
        McError::new(ids::PARSE_ERROR, t.line, t.col, en, fa)
    }

    // ---------- برنامه ----------
    fn parse_program(&mut self) -> Result<Program, McError> {
        let mut stmts = Vec::new();
        loop {
            self.skip_separators();
            if self.at(&Tok::Eof) {
                break;
            }
            let s = self.parse_stmt()?;
            stmts.push(s);
        }
        Ok(Program { stmts })
    }

    /// حذف `;`/newline/`,` های اضافی — هر سه جداکننده گزاره در سطح برنامه (سند ۰۳ §2).
    fn skip_separators(&mut self) {
        while self.at(&Tok::Semicolon) || self.at(&Tok::Newline) || self.at(&Tok::Comma) {
            self.bump();
        }
    }

    // ---------- گزاره‌ها ----------
    fn parse_stmt(&mut self) -> Result<Stmt, McError> {
        let t = self.peek().clone();
        match &t.tok {
            Tok::Ident(name) if name == "if" => self.parse_if(),
            Tok::Ident(name) if name == "for" => self.parse_for(),
            Tok::Ident(name) if name == "while" => self.parse_while(),
            Tok::Ident(name) if name == "break" => {
                self.bump();
                Ok(Stmt::Break {
                    line: t.line,
                    col: t.col,
                })
            }
            Tok::Ident(name) if name == "continue" => {
                self.bump();
                Ok(Stmt::Continue {
                    line: t.line,
                    col: t.col,
                })
            }
            Tok::Ident(name) if name == "return" => {
                self.bump();
                Ok(Stmt::Return {
                    line: t.line,
                    col: t.col,
                })
            }
            Tok::Ident(name) if name == "function" => {
                // تابع تو در تو ممنوع (سند ۰۳ §9 — تابع‌های محلی فقط سطح فایل)
                if self.fn_depth > 0 {
                    return Err(self.err(
                        &t,
                        "Nested function definitions are not supported. Define local functions at file level.",
                        "تعریف تابع تو در تو پشتیبانی نمی‌شود؛ توابع محلی را در سطح فایل تعریف کنید.",
                    ));
                }
                self.parse_fn_def()
            }
            Tok::Ident(name) if name == "format" => {
                // فرمان متلب `format short` / `format short e` (ROADMAP 2.4 —
                // آیتم ۱۱): فقط وقتی بعدش شناسهٔ غیرکلیدواژه بیاید (فرمان)؛
                // `format = 5` و `format(2)` و متغیر هم‌نام مسیر عادی می‌روند.
                if matches!(&self.peek_at(1).tok, Tok::Ident(n2) if !KEYWORDS.contains(&n2.as_str()))
                {
                    self.parse_format_command()
                } else {
                    self.parse_expr_stmt()
                }
            }
            _ => self.parse_expr_stmt(),
        }
    }

    /// فرمان `format <mode...>` — کلمات حالت تا پایان گزاره آرگومان رشته‌ای
    /// می‌شوند (`format short e` → format("short","e"))؛ معنا را ارزیاب می‌دهد
    /// (eval.rs::parse_format_words). سینتکس فرمانی متلب — ROADMAP 2.4.
    fn parse_format_command(&mut self) -> Result<Stmt, McError> {
        let t = self.bump(); // format
        let mut args = Vec::new();
        while let Tok::Ident(n) = &self.peek().tok {
            if KEYWORDS.contains(&n.as_str()) {
                break;
            }
            let n = n.clone();
            self.bump();
            args.push(Expr::Str(n));
        }
        let suppress = self.eat(&Tok::Semicolon);
        if args.is_empty() {
            return Err(self.err(
                &t,
                "format: missing mode (usage: format short|long|short e|long e|short g|long g|compact|loose).",
                "format: حالت داده نشده (استفاده: format short|long|short e|long e|short g|long g|compact|loose).",
            ));
        }
        Ok(Stmt::Expr {
            expr: Expr::Call {
                name: "format".into(),
                args,
                nargout: 1,
                line: t.line,
                col: t.col,
            },
            suppress,
        })
    }

    /// تعریف تابع فایلی — `function [out1,out2] = name(in1,in2) body end`.
    /// سمانتیک متلب §9: نام تابع پس از `=` می‌آید؛ خروجی‌ها قبل از آن؛
    /// هر دو اختیاری‌اند؛ بدنه تا `end` (P1: پایان‌دادن با `end` الزامی است).
    fn parse_fn_def(&mut self) -> Result<Stmt, McError> {
        let t = self.bump(); // function
        let mut outputs = Vec::new();
        if self.at(&Tok::LBracket) {
            // `[a,b] = name(...)` — چند خروجی
            self.bump();
            loop {
                if self.at(&Tok::RBracket) {
                    self.bump();
                    break;
                }
                let ot = self.expect(&Tok::Ident("".into()), "output name", "نام خروجی")?;
                if let Tok::Ident(n) = ot.tok.clone() {
                    if KEYWORDS.contains(&n.as_str()) {
                        return Err(self.err(
                            &ot,
                            format!("'{n}' is reserved and cannot be an output name."),
                            format!("«{n}» رزرو است و نمی‌تواند نام خروجی باشد."),
                        ));
                    }
                    outputs.push(n);
                }
                if !self.eat(&Tok::Comma) {
                    self.expect(&Tok::RBracket, "']'", "«]»")?;
                    break;
                }
            }
        } else if let Tok::Ident(n) = self.peek().tok.clone() {
            // تشخیص `function y = name` از `function name(...)`: نگاه به توکن بعد
            if self.peek_at(1).tok == Tok::Assign {
                self.bump();
                outputs.push(n);
            }
        }
        if !outputs.is_empty() {
            self.expect(
                &Tok::Assign,
                "'=' between outputs and function name",
                "«=» بین خروجی‌ها و نام تابع",
            )?;
        }
        let name_tok = self.expect(&Tok::Ident("".into()), "function name", "نام تابع")?;
        let name = match name_tok.tok.clone() {
            Tok::Ident(n) if !KEYWORDS.contains(&n.as_str()) => n,
            Tok::Ident(n) => {
                return Err(self.err(
                    &name_tok,
                    format!("'{n}' is a reserved keyword and cannot be a function name."),
                    format!("«{n}» کلیدواژه رزرو است و نمی‌تواند نام تابع باشد."),
                ))
            }
            _ => unreachable!(),
        };
        let mut params = Vec::new();
        if self.eat(&Tok::LParen) {
            if !self.at(&Tok::RParen) {
                loop {
                    let pt =
                        self.expect(&Tok::Ident("".into()), "parameter name", "نام پارامتر")?;
                    if let Tok::Ident(n) = pt.tok.clone() {
                        if KEYWORDS.contains(&n.as_str()) {
                            return Err(self.err(
                                &pt,
                                format!("'{n}' is reserved and cannot be a parameter."),
                                format!("«{n}» رزرو است و نمی‌تواند پارامتر باشد."),
                            ));
                        }
                        params.push(n);
                    }
                    if !self.eat(&Tok::Comma) {
                        break;
                    }
                }
            }
            self.expect(&Tok::RParen, "')' after parameters", "«)» بعد از پارامترها")?;
        }
        self.fn_depth += 1;
        let body = self.parse_block();
        self.fn_depth -= 1;
        let body = body?;
        self.expect_end("function")?;
        Ok(Stmt::FnDef {
            name,
            outputs,
            params,
            body,
            line: t.line,
            col: t.col,
        })
    }

    /// عبارت یا انتساب (ساده، اندیس‌دار، یا چندخروجی `[a,b] = f(...)`).
    fn parse_expr_stmt(&mut self) -> Result<Stmt, McError> {
        let start = self.peek().clone();
        let expr = self.parse_expr()?;

        // انتساب؟ `x = e` یا `A(i,j) = e` یا `[a,b] = f(...)`
        if self.at(&Tok::Assign) {
            self.bump();
            let rhs = self.parse_expr()?;
            let suppress = self.at(&Tok::Semicolon);
            // انتساب چندخروجی: ماتریس لفظی که همه سلول‌هایش Var باشند
            if let Expr::Matrix { rows, .. } = &expr {
                if let Some(targets) = out_targets(rows) {
                    return Ok(Stmt::MultiAssign {
                        targets,
                        expr: rhs,
                        suppress,
                    });
                }
            }
            let target = match expr {
                Expr::Var { name, line, col } => AssignTarget::Var { name, line, col },
                Expr::Call {
                    name,
                    args,
                    line,
                    col,
                    ..
                } if !args.is_empty() => AssignTarget::Indexed {
                    name,
                    subs: args,
                    line,
                    col,
                },
                _ => {
                    return Err(self.err(
                        &start,
                        "Invalid assignment target (left of '=').",
                        "سمت چپ «=» هدف انتساب معتبری نیست.",
                    ));
                }
            };
            return Ok(Stmt::Assign {
                target,
                expr: rhs,
                suppress,
            });
        }

        let suppress = self.at(&Tok::Semicolon);
        let _ = start;
        Ok(Stmt::Expr { expr, suppress })
    }

    fn parse_if(&mut self) -> Result<Stmt, McError> {
        let t = self.bump(); // if
        let mut branches = Vec::new();
        let cond = self.parse_expr()?;
        let body = self.parse_block()?;
        branches.push((cond, body));
        let mut else_body = None;
        loop {
            // elseif / else / end
            if matches!(self.peek().tok, Tok::Ident(ref n) if n == "elseif") {
                self.bump();
                let c = self.parse_expr()?;
                let b = self.parse_block()?;
                branches.push((c, b));
            } else if matches!(self.peek().tok, Tok::Ident(ref n) if n == "else") {
                self.bump();
                else_body = Some(self.parse_block()?);
                self.expect_end("if")?;
                break;
            } else {
                self.expect_end("if")?;
                break;
            }
        }
        Ok(Stmt::If {
            branches,
            else_body,
            line: t.line,
            col: t.col,
        })
    }

    /// بدنه تا `end`/`elseif`/`else` — جداکننده‌ها را می‌خورد.
    fn parse_block(&mut self) -> Result<Vec<Stmt>, McError> {
        self.skip_separators();
        let mut stmts = Vec::new();
        loop {
            if self.at(&Tok::Eof) {
                let t = self.peek().clone();
                return Err(self.err(
                    &t,
                    "Missing 'end' to close the block.",
                    "«end» برای بستن بلوک جا افتاده است.",
                ));
            }
            if self.stmt_terminator_ahead() {
                break;
            }
            stmts.push(self.parse_stmt()?);
            // جداکننده‌های بین گزاره‌ها
            let mut had_sep = false;
            while self.at(&Tok::Semicolon) || self.at(&Tok::Newline) || self.at(&Tok::Comma) {
                self.bump();
                had_sep = true;
            }
            let _ = had_sep;
            if self.stmt_terminator_ahead() {
                break;
            }
            if self.at(&Tok::Eof) {
                let t = self.peek().clone();
                return Err(self.err(
                    &t,
                    "Missing 'end' to close the block.",
                    "«end» برای بستن بلوک جا افتاده است.",
                ));
            }
        }
        Ok(stmts)
    }

    /// آیا توکن فعلی پایان‌دهنده بلوک است (end/elseif/else)؟
    fn stmt_terminator_ahead(&self) -> bool {
        if let Tok::Ident(n) = &self.peek().tok {
            matches!(n.as_str(), "end" | "elseif" | "else")
        } else {
            false
        }
    }

    fn expect_end(&mut self, what: &str) -> Result<(), McError> {
        let t = self.peek().clone();
        match &t.tok {
            Tok::Ident(n) if n == "end" => {
                self.bump();
                Ok(())
            }
            _ => Err(self.err(
                &t,
                format!("Missing 'end' to close '{what}'."),
                format!("«end» برای بستن «{what}» جا افتاده است."),
            )),
        }
    }

    fn parse_for(&mut self) -> Result<Stmt, McError> {
        let t = self.bump(); // for
        let var_tok = self.expect(&Tok::Ident("".into()), "loop variable", "متغیر حلقه")?;
        let var = match var_tok.tok.clone() {
            Tok::Ident(n) if !KEYWORDS.contains(&n.as_str()) => n,
            Tok::Ident(n) => {
                return Err(self.err(
                    &var_tok,
                    format!("'{n}' is a reserved keyword and cannot be a loop variable."),
                    format!("«{n}» کلیدواژه رزرو است و نمی‌تواند متغیر حلقه باشد."),
                ))
            }
            _ => unreachable!(),
        };
        self.expect(
            &Tok::Assign,
            "'=' after loop variable",
            "«=» بعد از متغیر حلقه",
        )?;
        let iter_expr = self.parse_expr()?;
        let body = self.parse_block()?;
        self.expect_end("for")?;
        Ok(Stmt::For {
            var,
            iter_expr,
            body,
            line: t.line,
            col: t.col,
        })
    }

    fn parse_while(&mut self) -> Result<Stmt, McError> {
        let t = self.bump(); // while
        let cond = self.parse_expr()?;
        let body = self.parse_block()?;
        self.expect_end("while")?;
        Ok(Stmt::While {
            cond,
            body,
            line: t.line,
            col: t.col,
        })
    }

    // ---------- عبارت‌ها (Pratt) ----------
    pub fn parse_expr(&mut self) -> Result<Expr, McError> {
        self.parse_or()
    }

    fn parse_or(&mut self) -> Result<Expr, McError> {
        let mut lhs = self.parse_and()?;
        while self.at(&Tok::OrOr) {
            let t = self.bump();
            let rhs = self.parse_and()?;
            lhs = Expr::Logic {
                op: LogicOp::Or,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            };
            let _ = t;
        }
        Ok(lhs)
    }

    fn parse_and(&mut self) -> Result<Expr, McError> {
        let mut lhs = self.parse_compare()?;
        while self.at(&Tok::AndAnd) {
            self.bump();
            let rhs = self.parse_compare()?;
            lhs = Expr::Logic {
                op: LogicOp::And,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            };
        }
        Ok(lhs)
    }

    fn parse_compare(&mut self) -> Result<Expr, McError> {
        let mut lhs = self.parse_colon()?;
        loop {
            let op = match self.peek().tok {
                Tok::Eq => CmpOp::Eq,
                Tok::Ne => CmpOp::Ne,
                Tok::Lt => CmpOp::Lt,
                Tok::Le => CmpOp::Le,
                Tok::Gt => CmpOp::Gt,
                Tok::Ge => CmpOp::Ge,
                _ => break,
            };
            let t = self.bump();
            let rhs = self.parse_colon()?;
            lhs = Expr::Compare {
                op,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
                line: t.line,
                col: t.col,
            };
        }
        Ok(lhs)
    }

    /// کولن زنجیره‌ای: `a:b`، `a:s:b`، و زنجیره‌های بلندتر با تاشدن چپ‌گرا
    /// (سند ۰۳ §7 — سمانتیک دقیق در eval).
    fn parse_colon(&mut self) -> Result<Expr, McError> {
        let first = self.parse_addsub()?;
        if !self.at(&Tok::Colon) {
            return Ok(first);
        }
        let mut parts = vec![first];
        while self.at(&Tok::Colon) {
            self.bump();
            parts.push(self.parse_addsub()?);
        }
        // تاشدن چپ‌گرا: [a,b,c,d] → Range(Range(a,b,c), 1, d)
        let mut it = parts.into_iter();
        let mut acc = it.next().unwrap_or(Expr::Num(0.0));
        let mut rest: Vec<Expr> = it.collect();
        while rest.len() >= 2 {
            let b = rest.remove(0);
            let c = rest.remove(0);
            acc = Expr::Range {
                start: Box::new(acc),
                step: Some(Box::new(b)),
                end: Box::new(c),
            };
        }
        if let Some(last) = rest.pop() {
            acc = Expr::Range {
                start: Box::new(acc),
                step: None,
                end: Box::new(last),
            };
        }
        Ok(acc)
    }

    fn parse_addsub(&mut self) -> Result<Expr, McError> {
        let mut lhs = self.parse_muldiv()?;
        loop {
            let op = match self.peek().tok {
                Tok::Plus => BinOp::Add,
                Tok::Minus => BinOp::Sub,
                _ => break,
            };
            // قانون واژگانی متلب (سند ۰۳ §4): داخل براکت، علامتِ «فاصله‌قبل +
            // بدون‌فاصله‌بعد» یگانی است و عنصر جدید می‌سازد — اینجا دودی مصرفش نکن:
            // [1 -2] = [1,-2] ولی [1 - 2] و [1-2] = تفریق.
            if self.mat_sign_ctx && self.peek().space_before && !self.peek_at(1).space_before {
                break;
            }
            let t = self.bump();
            let rhs = self.parse_muldiv()?;
            lhs = Expr::Binary {
                op,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
                line: t.line,
                col: t.col,
            };
        }
        Ok(lhs)
    }

    /// `* / \ .* ./` — ضرب/تقسیم ماتریسی و عنصری و بک‌اسلش در یک سطح (سند ۰۳ §5).
    fn parse_muldiv(&mut self) -> Result<Expr, McError> {
        let mut lhs = self.parse_unary()?;
        loop {
            let op = match self.peek().tok {
                Tok::Star => BinOp::Mul,
                Tok::Slash => BinOp::Div,
                Tok::Backslash => BinOp::Backslash,
                Tok::DotStar => BinOp::ElemMul,
                Tok::DotSlash => BinOp::ElemDiv,
                _ => break,
            };
            let t = self.bump();
            let rhs = self.parse_unary()?;
            lhs = Expr::Binary {
                op,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
                line: t.line,
                col: t.col,
            };
        }
        Ok(lhs)
    }

    /// یگانی: `- + ~` — سمانتیک متلب: یگانی پایهٔ توان را می‌گیرد ولی خودش
    /// زیر توان می‌رود: `-3^2 = -(3^2)` (سند ۰۳ §5).
    fn parse_unary(&mut self) -> Result<Expr, McError> {
        let t = self.peek().clone();
        match t.tok {
            Tok::Minus => {
                self.bump();
                let operand = self.parse_unary()?;
                Ok(Expr::Unary {
                    op: UnOp::Neg,
                    operand: Box::new(operand),
                })
            }
            Tok::Plus => {
                self.bump();
                let operand = self.parse_unary()?;
                Ok(Expr::Unary {
                    op: UnOp::Pos,
                    operand: Box::new(operand),
                })
            }
            Tok::Not => {
                self.bump();
                let operand = self.parse_unary()?;
                Ok(Expr::Unary {
                    op: UnOp::Not,
                    operand: Box::new(operand),
                })
            }
            _ => self.parse_pow(),
        }
    }

    /// توان چپ‌گرا (سند ۰۳ §5 سطح 90/91): `2^3^2 = 64`.
    /// پایهٔ توان می‌تواند خودش یگانی‌دار باشد (بالا هندل شد) —
    /// توان روی «توان-پایه» تکرار می‌شود: `a^b^c` = (a^b)^c.
    fn parse_pow(&mut self) -> Result<Expr, McError> {
        let mut base = self.parse_postfix()?;
        loop {
            let op = match self.peek().tok {
                Tok::Caret => BinOp::Pow,
                Tok::DotCaret => BinOp::ElemPow,
                _ => break,
            };
            let t = self.bump();
            // توان راستش می‌تواند یگانی‌دار باشد: 2^-3
            let rhs = self.parse_pow_operand()?;
            base = Expr::Binary {
                op,
                lhs: Box::new(base),
                rhs: Box::new(rhs),
                line: t.line,
                col: t.col,
            };
        }
        Ok(base)
    }

    fn parse_pow_operand(&mut self) -> Result<Expr, McError> {
        if matches!(self.peek().tok, Tok::Minus | Tok::Plus) {
            return self.parse_unary();
        }
        self.parse_postfix()
    }

    /// postfix: ترانهاده‌ها و فراخوانی/اندیس — `A(2,3)'(1)` مثل متلب زنجیره می‌شود.
    fn parse_postfix(&mut self) -> Result<Expr, McError> {
        let mut expr = self.parse_primary()?;
        loop {
            match self.peek().tok {
                Tok::Transpose => {
                    let t = self.bump();
                    expr = Expr::Transpose {
                        operand: Box::new(expr),
                        conj: true,
                        line: t.line,
                        col: t.col,
                    };
                }
                Tok::DotTranspose => {
                    let t = self.bump();
                    expr = Expr::Transpose {
                        operand: Box::new(expr),
                        conj: false,
                        line: t.line,
                        col: t.col,
                    };
                }
                Tok::LParen => {
                    // فراخوانی/اندیس — نام پایه باید Var باشد (سمانتیک متلب: نام‌محور)
                    let name = match &expr {
                        Expr::Var { name, .. } => name.clone(),
                        _ => {
                            let t = self.peek().clone();
                            return Err(self.err(
                                &t,
                                "Only named calls like f(x) or A(i,j) are supported here.",
                                "در اینجا فقط فراخوانی نام‌دار مثل f(x) یا A(i,j) پشتیبانی می‌شود.",
                            ));
                        }
                    };
                    let (line, col) = match &expr {
                        Expr::Var { line, col, .. } => (*line, *col),
                        _ => (0, 0),
                    };
                    self.bump(); // (
                    let mut args = Vec::new();
                    let mut nargout = 0usize;
                    // آرگومان‌ها تا ) — جداکننده , یا فاصله؛ end مجاز
                    if !self.at(&Tok::RParen) {
                        loop {
                            let arg = self.parse_expr()?;
                            args.push(arg);
                            if self.at(&Tok::Comma) {
                                self.bump();
                                // `[a,b] = f(...)` — [ بعد از , یعنی ادامه خروجی‌ها؛
                                // اینجا پارسر ساده: چندخروجی فقط در انتساب [a,b]=f() هندل می‌شود
                                continue;
                            }
                            break;
                        }
                    }
                    self.expect(&Tok::RParen, "')'", "«)»")?;
                    let _ = nargout;
                    nargout = 1;
                    expr = Expr::Call {
                        name,
                        args,
                        nargout,
                        line,
                        col,
                    };
                }
                _ => break,
            }
        }
        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expr, McError> {
        let t = self.peek().clone();
        match t.tok {
            Tok::Colon => {
                // کولن تنها = زیرنویس «همه» (: در اندیس/آرگومان) — A(:)، A(:,2)
                self.bump();
                Ok(Expr::Var {
                    name: ":".to_owned(),
                    line: t.line,
                    col: t.col,
                })
            }
            Tok::Num(v) => {
                self.bump();
                Ok(Expr::Num(v))
            }
            Tok::Imag(v) => {
                self.bump();
                Ok(Expr::Imag(v))
            }
            Tok::Str(s) => {
                self.bump();
                Ok(Expr::Str(s))
            }
            Tok::LBracket => self.parse_matrix(),
            Tok::At => {
                self.bump();
                // هندل نام‌دار `@f` — توکن بعد Ident است نه پرانتز (سمانتیک متلب §9)
                if let Tok::Ident(n) = &self.peek().tok {
                    let name = n.clone();
                    let t2 = self.peek().clone();
                    self.bump();
                    if KEYWORDS.contains(&name.as_str()) {
                        return Err(self.err(
                            &t2,
                            format!("'{name}' is reserved and cannot be a function handle target."),
                            format!("«{name}» رزرو است و نمی‌تواند هدف function handle باشد."),
                        ));
                    }
                    return Ok(Expr::FnHandle {
                        name,
                        line: t2.line,
                        col: t2.col,
                    });
                }
                // @(x,y) body — capture-by-value در eval (سند ۰۳ §9)
                self.expect(&Tok::LParen, "'(' after @", "«(» بعد از @")?;
                let mut params = Vec::new();
                if !self.at(&Tok::RParen) {
                    loop {
                        let pt =
                            self.expect(&Tok::Ident("".into()), "parameter name", "نام پارامتر")?;
                        if let Tok::Ident(n) = pt.tok.clone() {
                            if KEYWORDS.contains(&n.as_str()) {
                                return Err(self.err(
                                    &pt,
                                    format!("'{n}' is reserved and cannot be a parameter."),
                                    format!("«{n}» رزرو است و نمی‌تواند پارامتر باشد."),
                                ));
                            }
                            params.push(n);
                        }
                        if !self.eat(&Tok::Comma) {
                            break;
                        }
                    }
                }
                self.expect(&Tok::RParen, "')' after @ params", "«)» بعد از پارامترها")?;
                let body = self.parse_expr()?;
                Ok(Expr::AnonFn {
                    params,
                    body: Box::new(body),
                    line: t.line,
                    col: t.col,
                })
            }
            Tok::LParen => {
                self.bump();
                let e = self.parse_expr()?;
                self.expect(&Tok::RParen, "')'", "«)»")?;
                Ok(e)
            }
            Tok::Ident(ref name) => {
                let name = name.clone();
                self.bump();
                if KEYWORDS.contains(&name.as_str()) {
                    if name == "end" {
                        // `end` فقط در اندیس معنادار است — اینجا به‌عنوان نود End
                        // (eval فقط در زمینهٔ اندیس آن را می‌پذیرد)
                        return Ok(Expr::End {
                            line: t.line,
                            col: t.col,
                        });
                    }
                    return Err(self.err(
                        &t,
                        format!("'{name}' is a reserved keyword here."),
                        format!("«{name}» اینجا کلیدواژهٔ رزرو است."),
                    ));
                }
                Ok(Expr::Var {
                    name,
                    line: t.line,
                    col: t.col,
                })
            }
            _ => Err(self.err(
                &t,
                format!("Unexpected {}.", t.describe()),
                format!("{} غیرمنتظره بود.", t.describe()),
            )),
        }
    }

    /// ماتریس لفظی `[...]` (سند ۰۳ §4) — قلب ابهام متلب.
    ///
    /// جداکننده ستون: `,` یا فاصله (با قانون یگانی/دویی +/- از space flags)
    /// جداکننده ردیف: `;` یا newline.
    fn parse_matrix(&mut self) -> Result<Expr, McError> {
        // فعال‌سازی قانون ابهام علامت فقط در گسترهٔ این براکت (خروجی‌ها بازگردانی)
        let saved = self.mat_sign_ctx;
        self.mat_sign_ctx = true;
        let r = self.parse_matrix_inner();
        self.mat_sign_ctx = saved;
        r
    }

    fn parse_matrix_inner(&mut self) -> Result<Expr, McError> {
        let t = self.bump(); // [
        let mut rows: Vec<Vec<Expr>> = vec![Vec::new()];
        loop {
            if self.at(&Tok::Eof) {
                return Err(self.err(
                    self.peek(),
                    "Missing ']' to close the matrix.",
                    "«]» برای بستن ماتریس جا افتاده است.",
                ));
            }
            if self.at(&Tok::RBracket) {
                self.bump();
                break;
            }
            if self.at(&Tok::Semicolon) || self.at(&Tok::Newline) {
                self.bump();
                // ردیف خنجین تهیِ آخر را نساز
                if !self.at(&Tok::RBracket) {
                    rows.push(Vec::new());
                }
                continue;
            }
            if self.at(&Tok::Comma) {
                self.bump();
                continue;
            }
            let e = self.parse_matrix_element(&rows)?;
            rows.last_mut().unwrap_or(&mut Vec::new()).push(e);
        }
        // ردیف‌های تهی انتهایی را حذف نکن — `[]` = ماتریس 0×0 (سند ۰۳ §4)
        if rows.len() == 1 && rows[0].is_empty() {
            return Ok(Expr::Matrix {
                rows: Vec::new(),
                line: t.line,
                col: t.col,
            });
        }
        Ok(Expr::Matrix {
            rows,
            line: t.line,
            col: t.col,
        })
    }

    /// یک سلول ماتریس — با قانون ابهام علامت:
    /// داخل ماتریس بعد از یک سلول کامل، `+/-` با فاصله‌قبل و بدون‌فاصله‌بعد = یگانی.
    fn parse_matrix_element(&mut self, rows: &[Vec<Expr>]) -> Result<Expr, McError> {
        // قانون متلب: بعد از سلولِ کامل، علامتِ «فاصله‌قبل + بدون‌فاصله‌بعد» یگانی است.
        if (self.at(&Tok::Plus) || self.at(&Tok::Minus))
            && self.peek().space_before
            && !self.peek_at(1).space_before
            && !rows.last().is_some_and(|r| r.is_empty())
        {
            let op_tok = self.bump();
            let operand = self.parse_matrix_element_signless()?;
            let expr = match op_tok.tok {
                Tok::Minus => Expr::Unary {
                    op: UnOp::Neg,
                    operand: Box::new(operand),
                },
                _ => Expr::Unary {
                    op: UnOp::Pos,
                    operand: Box::new(operand),
                },
            };
            // ادامهٔ سطح-بالای عبارت را با این یگانی به‌عنوان شروع ادامه بده
            return self.finish_matrix_element(expr);
        }
        let e = self.parse_expr()?;
        self.finish_matrix_element(e)
    }

    /// بعد از یگانیِ ماتریسی، ادامهٔ عبارت (مثلاً `[1 -2 +3]`).
    fn parse_matrix_element_signless(&mut self) -> Result<Expr, McError> {
        self.parse_postfix_pow_base()
    }

    fn parse_postfix_pow_base(&mut self) -> Result<Expr, McError> {
        let e = self.parse_postfix()?;
        // توان چسبیده: [1 -2^2] → -2^2 = -(2^2)
        let op = match self.peek().tok {
            Tok::Caret => BinOp::Pow,
            Tok::DotCaret => BinOp::ElemPow,
            _ => return Ok(e),
        };
        let t = self.bump();
        let rhs = self.parse_pow_operand()?;
        Ok(Expr::Binary {
            op,
            lhs: Box::new(e),
            rhs: Box::new(rhs),
            line: t.line,
            col: t.col,
        })
    }

    /// ادامهٔ یک سلول شروع‌شده — عملگرهای دویی سطح بالا (بدون کولن/مقایسه داخل ماتریسِ چندهنری).
    /// علامتِ ابهام‌دار (فاصله‌قبل + بدون‌فاصله‌بعد) اینجا هم دودی مصرف نمی‌شود —
    /// عنصر جدید است: [1 -2 +3] = سه عنصر، نه 1 و (-2+3).
    fn finish_matrix_element(&mut self, first: Expr) -> Result<Expr, McError> {
        let mut lhs = first;
        loop {
            if matches!(self.peek().tok, Tok::Plus | Tok::Minus)
                && self.peek().space_before
                && !self.peek_at(1).space_before
            {
                break;
            }
            let op = match self.peek().tok {
                Tok::Plus => BinOp::Add,
                Tok::Minus => BinOp::Sub,
                Tok::Star => BinOp::Mul,
                Tok::Slash => BinOp::Div,
                Tok::Backslash => BinOp::Backslash,
                Tok::DotStar => BinOp::ElemMul,
                Tok::DotSlash => BinOp::ElemDiv,
                Tok::Caret => BinOp::Pow,
                Tok::DotCaret => BinOp::ElemPow,
                Tok::Transpose | Tok::DotTranspose | Tok::LParen => {
                    // postfix بعد از عنصر — مثل a(1)' — با parse_postfix ادامه بده
                    lhs = self.parse_postfix_from(lhs)?;
                    continue;
                }
                _ => break,
            };
            let t = self.bump();
            let rhs = self.parse_matrix_element_rhs()?;
            lhs = Expr::Binary {
                op,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
                line: t.line,
                col: t.col,
            };
        }
        Ok(lhs)
    }

    fn parse_postfix_from(&mut self, expr: Expr) -> Result<Expr, McError> {
        // بازاستفاده از منطق postfix — ترانهاده/اندیس رویexpr موجود
        let mut lhs = expr;
        loop {
            match self.peek().tok {
                Tok::Transpose => {
                    let t = self.bump();
                    lhs = Expr::Transpose {
                        operand: Box::new(lhs),
                        conj: true,
                        line: t.line,
                        col: t.col,
                    };
                }
                Tok::DotTranspose => {
                    let t = self.bump();
                    lhs = Expr::Transpose {
                        operand: Box::new(lhs),
                        conj: false,
                        line: t.line,
                        col: t.col,
                    };
                }
                _ => break,
            }
        }
        Ok(lhs)
    }

    fn parse_matrix_element_rhs(&mut self) -> Result<Expr, McError> {
        // سمت راست عملگر داخل ماتریس — یگانی‌ها و postfix مجاز
        let t = self.peek().clone();
        match t.tok {
            Tok::Minus => {
                self.bump();
                let operand = self.parse_matrix_element_rhs()?;
                Ok(Expr::Unary {
                    op: UnOp::Neg,
                    operand: Box::new(operand),
                })
            }
            Tok::Plus => {
                self.bump();
                let operand = self.parse_matrix_element_rhs()?;
                Ok(Expr::Unary {
                    op: UnOp::Pos,
                    operand: Box::new(operand),
                })
            }
            _ => {
                let e = self.parse_postfix_pow_base()?;
                // توان/ترانهاده زنجیره‌ای
                self.finish_matrix_element(e)
            }
        }
    }
}

/// ماتریسِ سمت چپ انتساب، چندخروجی است؟ — فقط اگر همه سلول‌ها Var معتبر باشند
/// (`[a,b]`، `[a b]`، حتی `[a;b]`)؛ در غیر این صورت None (ماتریس معمولی).
fn out_targets(rows: &[Vec<Expr>]) -> Option<Vec<OutTarget>> {
    let mut targets = Vec::new();
    for row in rows {
        for e in row {
            match e {
                Expr::Var { name, line, col }
                    if name != ":" && !KEYWORDS.contains(&name.as_str()) =>
                {
                    targets.push(OutTarget {
                        name: name.clone(),
                        line: *line,
                        col: *col,
                    });
                }
                _ => return None,
            }
        }
    }
    if targets.is_empty() {
        None
    } else {
        Some(targets)
    }
}
