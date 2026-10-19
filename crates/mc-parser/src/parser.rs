//! پارسر Pratt — اولویت‌ها دقیقاً طبق جدول سند ۰۳ بخش ۵.
//!
//! نکته‌های سمانتیکی که اینجا قفل می‌شوند:
//! - توان `^` **چپ‌گرا** است (رفتار MATLAB): `2^3^2 = (2^3)^2 = 64`.
//! - یگاری زیر توان است: `-2^2 = -(2^2) = -4`؛ اما در توان `2^-2` مجاز است.
//! - `*` `/` و `+` `-` چپ‌گرا: `6/2*3 = 9`، `1-2-3 = -4`.
//! - `=` فقط در سطح دستور (statement) دیده می‌شود و مقدار برنمی‌گرداند.

use mc_core::{ids, McError};

use crate::ast::{BinOp, Expr, Program, Stmt, UnOp};
use crate::lexer::{self, Tok, Token};

/// حداقل قدرت اتصال (binding power) برای عملوند یگاری — بین توان (۹۰) و ضرب (۸۰).
const UNARY_RBP: u8 = 85;

const POW_BP: u8 = 90;
const MUL_BP: u8 = 80;
const ADD_BP: u8 = 70;

pub fn parse(src: &str) -> Result<Program, McError> {
    let tokens = lexer::tokenize(src)?;
    Parser { tokens, pos: 0 }.program()
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

/// پیام خطای سینتکسی — الگوی مرجع سند ۰۳ بخش ۱۴ (MC:E3001).
fn parse_err(line: u32, col: u32, unexpected: &str, expected: &str) -> McError {
    McError::new(
        ids::PARSE_ERROR,
        line,
        col,
        format!("Unexpected {unexpected}: expected {expected}."),
        format!("«{unexpected}» غیرمنتظره: {expected} انتظار می‌رفت."),
    )
}

impl Parser {
    fn peek(&self) -> &Token {
        // Eof همیشه آخرین توکن است؛ پس pos بیرون از بازه یعنی روی Eof بمانیم.
        let i = self.pos.min(self.tokens.len() - 1);
        &self.tokens[i]
    }

    fn peek_ahead(&self, offset: usize) -> Option<&Token> {
        self.tokens.get(self.pos + offset)
    }

    fn advance(&mut self) -> Token {
        let t = self.peek().clone();
        if !matches!(t.tok, Tok::Eof) {
            self.pos += 1;
        }
        t
    }

    fn program(&mut self) -> Result<Program, McError> {
        let mut stmts = Vec::new();
        loop {
            // خطوط خالی متوالی مجازند (رفتار متلب با `1;;2` و خطوط خالی).
            match self.peek().tok {
                Tok::Semicolon | Tok::Comma => {
                    self.advance();
                    continue;
                }
                Tok::Newline => {
                    self.advance();
                    continue;
                }
                Tok::Eof => break,
                _ => {}
            }
            let stmt = self.statement()?;
            stmts.push(stmt);
            match self.peek().tok {
                Tok::Semicolon => {
                    self.mark_last_suppressed(&mut stmts);
                    self.advance();
                }
                Tok::Comma => {
                    self.advance();
                }
                Tok::Newline | Tok::Eof => {}
                _ => {
                    let t = self.peek();
                    return Err(parse_err(
                        t.line,
                        t.col,
                        &t.describe(),
                        "',', ';' or end of line",
                    ));
                }
            }
        }
        Ok(Program { stmts })
    }

    /// `;` به دستور «قبلی» چسبیده است (`x = 1;`) — پرچم suppress را عقب‌گرد اعمال می‌کنیم.
    fn mark_last_suppressed(&mut self, stmts: &mut [Stmt]) {
        if let Some(last) = stmts.last_mut() {
            match last {
                Stmt::Expr { suppress, .. } | Stmt::Assign { suppress, .. } => {
                    *suppress = true;
                }
            }
        }
    }

    fn statement(&mut self) -> Result<Stmt, McError> {
        // انتساب: Ident '=' (سند ۰۳ بخش ۵ سطح ۱۰)
        if let Tok::Ident(_) = self.peek().tok {
            if matches!(self.peek_ahead(1).map(|t| &t.tok), Some(Tok::Assign)) {
                let name_tok = self.advance();
                let Tok::Ident(name) = name_tok.tok else {
                    return Err(parse_err(
                        name_tok.line,
                        name_tok.col,
                        "identifier",
                        "identifier",
                    ));
                };
                // نام‌های رزرو هرگز به‌عنوان متغیر قابل ساخت نیستند (سند ۰۳ بخش ۲).
                check_keyword(&name, name_tok.line, name_tok.col)?;
                self.advance(); // '='
                let expr = self.expr_bp(0)?;
                return Ok(Stmt::Assign {
                    name,
                    expr,
                    suppress: false,
                    line: name_tok.line,
                    col: name_tok.col,
                });
            }
        }
        let expr = self.expr_bp(0)?;
        Ok(Stmt::Expr {
            expr,
            suppress: false,
        })
    }

    fn expr_bp(&mut self, min_bp: u8) -> Result<Expr, McError> {
        let mut lhs = self.unary()?;
        while let Some((l_bp, r_bp, op)) = self.infix_op() {
            if l_bp < min_bp {
                break;
            }
            let op_tok = self.advance();
            let rhs = self.expr_bp(r_bp)?;
            lhs = Expr::Binary {
                op,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
                line: op_tok.line,
                col: op_tok.col,
            };
        }
        Ok(lhs)
    }

    /// (قدرت چپ، قدرت راست، اپراتور) — همه اپراتورهای P0 **چپ‌گرا**اند (رفتار متلب
    /// و سند ۰۳ بخش ۵): برای چپ‌گرایی، عملوند راست با `l_bp + 1` پارس می‌شود تا
    /// اپراتور هم‌سطح بعدی به lhs بچسبد نه به rhs (`6/2*3 = (6/2)*3 = 9`).
    fn infix_op(&self) -> Option<(u8, u8, BinOp)> {
        let bp = match self.peek().tok {
            Tok::Plus => (ADD_BP, ADD_BP + 1, BinOp::Add),
            Tok::Minus => (ADD_BP, ADD_BP + 1, BinOp::Sub),
            Tok::Star => (MUL_BP, MUL_BP + 1, BinOp::Mul),
            Tok::Slash => (MUL_BP, MUL_BP + 1, BinOp::Div),
            Tok::Caret => (POW_BP, POW_BP + 1, BinOp::Pow),
            _ => return None,
        };
        Some(bp)
    }

    fn unary(&mut self) -> Result<Expr, McError> {
        match self.peek().tok {
            Tok::Minus => {
                self.advance();
                let operand = self.expr_bp(UNARY_RBP)?;
                Ok(Expr::Unary {
                    op: UnOp::Neg,
                    operand: Box::new(operand),
                })
            }
            Tok::Plus => {
                self.advance();
                let operand = self.expr_bp(UNARY_RBP)?;
                Ok(Expr::Unary {
                    op: UnOp::Pos,
                    operand: Box::new(operand),
                })
            }
            Tok::Num(_) | Tok::Ident(_) | Tok::LParen => {
                // اول advance تا بorrow تمام شود، بعد روی توکن مالک‌شده کار کن.
                let tok = self.advance();
                match tok.tok {
                    Tok::Num(v) => Ok(Expr::Num(v)),
                    Tok::Ident(name) => {
                        check_keyword(&name, tok.line, tok.col)?;
                        Ok(Expr::Var {
                            name,
                            line: tok.line,
                            col: tok.col,
                        })
                    }
                    Tok::LParen => {
                        let inner = self.expr_bp(0)?;
                        if !matches!(self.peek().tok, Tok::RParen) {
                            let t = self.peek();
                            return Err(parse_err(t.line, t.col, &t.describe(), "')'"));
                        }
                        self.advance();
                        Ok(inner)
                    }
                    _ => Err(parse_err(
                        tok.line,
                        tok.col,
                        &tok.describe(),
                        "an expression",
                    )),
                }
            }
            _ => {
                let t = self.peek();
                Err(parse_err(t.line, t.col, &t.describe(), "an expression"))
            }
        }
    }
}

/// کلمات کلیدی رزرو (سند ۰۳ بخش ۲) — هرگز به‌عنوان شناسه پذیرفته نمی‌شوند.
fn check_keyword(name: &str, line: u32, col: u32) -> Result<(), McError> {
    const PERMANENT: &[(&str, &str, &str)] = &[
        (
            "eval",
            "eval does not exist in MathCraft; use functions and variables instead.",
            "eval در MathCraft وجود ندارد؛ از توابع و متغیرها استفاده کنید.",
        ),
        (
            "evalin",
            "evalin does not exist in MathCraft; use functions and variables instead.",
            "evalin در MathCraft وجود ندارد؛ از توابع و متغیرها استفاده کنید.",
        ),
        (
            "parfor",
            "parfor is intentionally not supported in version 1.",
            "parfor در نسخه ۱ عمداً پشتیبانی نمی‌شود.",
        ),
        (
            "classdef",
            "classdef (OOP) is intentionally not supported in version 1.",
            "classdef (برنامه‌نویسی شیءگرا) در نسخه ۱ عمداً پشتیبانی نمی‌شود.",
        ),
    ];
    for (kw, en, fa) in PERMANENT {
        if *kw == name {
            return Err(McError::new(ids::UNSUPPORTED, line, col, *en, *fa));
        }
    }
    const CONTROL: &[&str] = &[
        "if",
        "elseif",
        "else",
        "end",
        "for",
        "while",
        "switch",
        "case",
        "otherwise",
        "try",
        "catch",
        "break",
        "continue",
        "return",
        "function",
        "global",
        "persistent",
    ];
    if CONTROL.contains(&name) {
        return Err(McError::new(
            ids::UNSUPPORTED,
            line,
            col,
            format!("'{name}' is reserved; it arrives with the language milestone."),
            format!("«{name}» رزرو شده است؛ همراه مایل‌استون زبان می‌آید."),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_ok(src: &str) -> Program {
        parse(src).unwrap_or_else(|e| panic!("parse failed for {src:?}: {e}"))
    }

    #[test]
    fn one_plus_one_is_one_statement() {
        let p = parse_ok("1+1");
        assert_eq!(p.stmts.len(), 1);
        match &p.stmts[0] {
            Stmt::Expr { expr, suppress } => {
                assert!(!suppress);
                assert_eq!(
                    expr,
                    &Expr::Binary {
                        op: BinOp::Add,
                        lhs: Box::new(Expr::Num(1.0)),
                        rhs: Box::new(Expr::Num(1.0)),
                        line: 1,
                        col: 2
                    }
                );
            }
            other => panic!("expected expr stmt, got {other:?}"),
        }
    }

    #[test]
    fn power_is_left_associative_like_matlab() {
        // سند ۰۳ بخش ۵: 2^3^2 = (2^3)^2 = 64
        let p = parse_ok("2^3^2");
        let Stmt::Expr { expr, .. } = &p.stmts[0] else {
            panic!("expr expected");
        };
        let Expr::Binary { op, lhs, .. } = expr else {
            panic!("binary expected");
        };
        assert_eq!(*op, BinOp::Pow);
        let Expr::Binary { op: inner_op, .. } = lhs.as_ref() else {
            panic!("nested pow expected");
        };
        assert_eq!(*inner_op, BinOp::Pow);
    }

    #[test]
    fn unary_minus_binds_looser_than_power() {
        // -2^2 باید در ارزیابی -4 بدهد؛ ساختار: Neg(2^2)
        let p = parse_ok("-2^2");
        let Stmt::Expr { expr, .. } = &p.stmts[0] else {
            panic!("expr expected");
        };
        let Expr::Unary { operand, .. } = expr else {
            panic!("unary expected");
        };
        assert!(matches!(
            operand.as_ref(),
            Expr::Binary { op: BinOp::Pow, .. }
        ));
    }

    #[test]
    fn negative_exponent_allowed() {
        assert!(parse("2^-2").is_ok());
        assert!(parse("2 ^ -2 ^ 2").is_ok());
    }

    #[test]
    fn left_associativity_of_arithmetic() {
        // 6/2*3: ساختار (6/2)*3
        let p = parse_ok("6/2*3");
        let Stmt::Expr { expr, .. } = &p.stmts[0] else {
            panic!("expr expected");
        };
        let Expr::Binary { op, lhs, .. } = expr else {
            panic!("binary expected");
        };
        assert_eq!(*op, BinOp::Mul);
        assert!(matches!(lhs.as_ref(), Expr::Binary { op: BinOp::Div, .. }));

        // 1-2-3: ساختار (1-2)-3
        let p = parse_ok("1-2-3");
        let Stmt::Expr { expr, .. } = &p.stmts[0] else {
            panic!("expr expected");
        };
        let Expr::Binary { op, lhs, .. } = expr else {
            panic!("binary expected");
        };
        assert_eq!(*op, BinOp::Sub);
        assert!(matches!(lhs.as_ref(), Expr::Binary { op: BinOp::Sub, .. }));
    }

    #[test]
    fn assignment_and_suppression() {
        let p = parse_ok("x = 1+2; y = 3, 4");
        assert_eq!(p.stmts.len(), 3);
        assert!(matches!(&p.stmts[0], Stmt::Assign { suppress: true, .. }));
        assert!(matches!(
            &p.stmts[1],
            Stmt::Assign {
                suppress: false,
                ..
            }
        ));
        assert!(matches!(
            &p.stmts[2],
            Stmt::Expr {
                suppress: false,
                ..
            }
        ));
    }

    #[test]
    fn error_positions_are_exact() {
        let err = parse("1 + (2 * 3").unwrap_err();
        assert_eq!(err.id, ids::PARSE_ERROR);
        assert_eq!(err.line, 1);
        assert_eq!(err.col, 11); // انتهای ورودی (Eof)
    }

    #[test]
    fn trailing_token_is_syntax_error() {
        let err = parse("2 3").unwrap_err();
        assert_eq!(err.id, ids::PARSE_ERROR);
    }

    #[test]
    fn keywords_are_rejected_with_honest_messages() {
        let e1 = parse("if x > 0").unwrap_err();
        assert_eq!(e1.id, ids::UNSUPPORTED);
        // eval ممنوع دائمی (سند ۰۳ بخش ۱۵) — در موقعیت عبارتی
        let e2 = parse("eval + 1").unwrap_err();
        assert!(e2.message_fa.contains("وجود ندارد"));
        // کلمات رزرو حتی به‌عنوان هدف انتساب مجاز نیستند
        let e3 = parse("if = 5").unwrap_err();
        assert_eq!(e3.id, ids::UNSUPPORTED);
    }

    #[test]
    fn empty_and_comment_only_input() {
        assert!(parse("").unwrap().stmts.is_empty());
        assert!(parse("% فقط کامنت").unwrap().stmts.is_empty());
    }
}
