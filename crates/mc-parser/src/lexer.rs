//! Lexer — ساختار واژگانی سند ۰۳ بخش ۲.
//!
//! موقعیت‌ها ۱-پایه‌اند (خط، ستون) و ستون به‌ازای هر codepoint جلو می‌رود
//! (سازگار با قاعده یونیکد سند ۰۳ بخش ۱۰).

use mc_core::{ids, McError};

#[derive(Debug, Clone, PartialEq)]
pub enum Tok {
    Num(f64),
    Ident(String),
    Plus,
    Minus,
    Star,
    Slash,
    Caret,
    LParen,
    RParen,
    Comma,
    Semicolon,
    Assign,
    Newline,
    Eof,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub tok: Tok,
    pub line: u32,
    pub col: u32,
}

impl Token {
    /// توصیف کوتاه برای پیام‌های خطا (سند ۰۳ بخش ۱۴: «'x' غیرمنتظره»).
    pub fn describe(&self) -> String {
        match &self.tok {
            Tok::Num(_) => "number".to_owned(),
            Tok::Ident(n) => format!("'{n}'"),
            Tok::Plus => "'+'".to_owned(),
            Tok::Minus => "'-'".to_owned(),
            Tok::Star => "'*'".to_owned(),
            Tok::Slash => "'/'".to_owned(),
            Tok::Caret => "'^'".to_owned(),
            Tok::LParen => "'('".to_owned(),
            Tok::RParen => "')'".to_owned(),
            Tok::Comma => "','".to_owned(),
            Tok::Semicolon => "';'".to_owned(),
            Tok::Assign => "'='".to_owned(),
            Tok::Newline => "end of line".to_owned(),
            Tok::Eof => "end of input".to_owned(),
        }
    }
}

struct Lexer {
    chars: Vec<char>,
    pos: usize,
    line: u32,
    col: u32,
}

/// Maximum significant identifier length, like MATLAB `namelengthmax` (doc 03 §2).
const MAX_NAME_LEN: usize = 63;

pub fn tokenize(src: &str) -> Result<Vec<Token>, McError> {
    let mut lx = Lexer {
        chars: src.chars().collect(),
        pos: 0,
        line: 1,
        col: 1,
    };
    let mut out = Vec::new();
    while let Some(c) = lx.peek() {
        match c {
            ' ' | '\t' | '\r' => {
                lx.advance();
            }
            '\n' => {
                out.push(lx.make(Tok::Newline));
                lx.advance();
                lx.line += 1;
                lx.col = 1;
            }
            '%' => {
                // کامنت تا انتهای خط (سند ۰۳ بخش ۲) — `%%` هم همین‌جا پوشش داده می‌شود.
                while let Some(ch) = lx.peek() {
                    if ch == '\n' {
                        break;
                    }
                    lx.advance();
                }
            }
            '.' if lx.starts_with("...") => {
                // ادامه خط: تا انتهای خط (شامل خود newline) بدون تولید توکن Newline.
                while let Some(ch) = lx.peek() {
                    if ch == '\n' {
                        break;
                    }
                    lx.advance();
                }
                if lx.peek().is_some() {
                    lx.advance(); // خود newline را مصرف کن
                    lx.line += 1;
                    lx.col = 1;
                }
            }
            '0'..='9' => out.push(lx.lex_number()?),
            '.' if lx.peek_at(1).is_some_and(|n| n.is_ascii_digit()) => out.push(lx.lex_number()?),
            'A'..='Z' | 'a'..='z' | '_' => out.push(lx.lex_ident()?),
            '+' => {
                out.push(lx.make(Tok::Plus));
                lx.advance();
            }
            '-' => {
                out.push(lx.make(Tok::Minus));
                lx.advance();
            }
            '*' => {
                out.push(lx.make(Tok::Star));
                lx.advance();
            }
            '/' => {
                out.push(lx.make(Tok::Slash));
                lx.advance();
            }
            '^' => {
                out.push(lx.make(Tok::Caret));
                lx.advance();
            }
            '(' => {
                out.push(lx.make(Tok::LParen));
                lx.advance();
            }
            ')' => {
                out.push(lx.make(Tok::RParen));
                lx.advance();
            }
            ',' => {
                out.push(lx.make(Tok::Comma));
                lx.advance();
            }
            ';' => {
                out.push(lx.make(Tok::Semicolon));
                lx.advance();
            }
            '=' => {
                if lx.peek_at(1) == Some('=') {
                    return Err(lx.err_here(
                        ids::UNSUPPORTED,
                        "Comparison operators ('==') arrive in a later milestone.",
                        "اپراتورهای مقایسه ('==') در مایل‌استون‌های بعدی می‌آیند.",
                    ));
                }
                out.push(lx.make(Tok::Assign));
                lx.advance();
            }
            other => {
                // هیچ کاراکتری «بی‌صدا» نادیده گرفته نمی‌شود (سند ۰۳ بخش ۱ بند ۲).
                return Err(lx.err_here(
                    ids::UNSUPPORTED,
                    format!("Character '{other}' is not supported in this milestone."),
                    format!("کاراکتر «{other}» در این مایل‌استون پشتیبانی نمی‌شود."),
                ));
            }
        }
    }
    out.push(Token {
        tok: Tok::Eof,
        line: lx.line,
        col: lx.col,
    });
    Ok(out)
}

impl Lexer {
    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn peek_at(&self, offset: usize) -> Option<char> {
        self.chars.get(self.pos + offset).copied()
    }

    fn starts_with(&self, s: &str) -> bool {
        s.chars()
            .enumerate()
            .all(|(i, sc)| self.peek_at(i) == Some(sc))
    }

    fn advance(&mut self) {
        self.pos += 1;
        self.col += 1;
    }

    fn make(&self, tok: Tok) -> Token {
        Token {
            tok,
            line: self.line,
            col: self.col,
        }
    }

    fn err_here(&self, id: &str, en: impl Into<String>, fa: impl Into<String>) -> McError {
        McError::new(id, self.line, self.col, en, fa)
    }

    /// عدد اعشاری با نماد علمی (سند ۰۳ بخش ۲: `3`، `2.5`، `1e-6`؛ پسوند `_` مجاز نیست).
    fn lex_number(&mut self) -> Result<Token, McError> {
        let start_line = self.line;
        let start_col = self.col;
        let mut text = String::new();

        while self.peek().is_some_and(|c| c.is_ascii_digit()) {
            text.push(self.peek().unwrap_or('0'));
            self.advance();
        }
        if self.peek() == Some('.') {
            text.push('.');
            self.advance();
            while self.peek().is_some_and(|c| c.is_ascii_digit()) {
                text.push(self.peek().unwrap_or('0'));
                self.advance();
            }
        }
        if matches!(self.peek(), Some('e') | Some('E')) {
            let mut exp = String::new();
            exp.push(self.peek().unwrap_or('e'));
            let mut lookahead = 1;
            if matches!(self.peek_at(1), Some('+') | Some('-')) {
                exp.push(self.peek_at(1).unwrap_or('+'));
                lookahead = 2;
            }
            if self.peek_at(lookahead).is_some_and(|c| c.is_ascii_digit()) {
                text.push_str(&exp);
                for _ in 0..lookahead {
                    self.advance();
                }
                while self.peek().is_some_and(|c| c.is_ascii_digit()) {
                    text.push(self.peek().unwrap_or('0'));
                    self.advance();
                }
            } else {
                return Err(McError::new(
                    ids::PARSE_ERROR,
                    start_line,
                    start_col,
                    "Malformed number: exponent needs digits (like 1e-6).",
                    "عدد بدشکل: نماد علمی باید رقم داشته باشد (مثل 1e-6).",
                ));
            }
        }

        // پسوند موهومی i/j — کامل در نسخه ۱ (سند ۰۳ بخش ۳) اما نه در P0: خطای صریح.
        if matches!(self.peek(), Some('i') | Some('j')) {
            return Err(McError::new(
                ids::UNSUPPORTED,
                start_line,
                start_col,
                "Complex literals (like 3i) arrive in a later milestone.",
                "اعداد مختلط (مثل 3i) در مایل‌استون‌های بعدی می‌آیند.",
            ));
        }

        let value: f64 = text.parse().unwrap_or(f64::INFINITY); // سرریز مثل 1e999 → Inf (رفتار IEEE، سند ۰۳ بخش ۱۱)
        Ok(Token {
            tok: Tok::Num(value),
            line: start_line,
            col: start_col,
        })
    }

    fn lex_ident(&mut self) -> Result<Token, McError> {
        let start_line = self.line;
        let start_col = self.col;
        let mut name = String::new();
        while self
            .peek()
            .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_')
        {
            name.push(self.peek().unwrap_or('_'));
            self.advance();
            if name.len() == MAX_NAME_LEN + 1 {
                return Err(McError::new(
                    ids::UNSUPPORTED,
                    start_line,
                    start_col,
                    format!("Identifier exceeds the maximum length of {MAX_NAME_LEN} characters."),
                    format!("نام شناسه از حد مجاز {MAX_NAME_LEN} کاراکتر بیشتر است."),
                ));
            }
        }
        Ok(Token {
            tok: Tok::Ident(name),
            line: start_line,
            col: start_col,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn toks(src: &str) -> Vec<Tok> {
        tokenize(src).unwrap().into_iter().map(|t| t.tok).collect()
    }

    #[test]
    fn basic_arithmetic_stream() {
        assert_eq!(
            toks("1+2.5*x"),
            vec![
                Tok::Num(1.0),
                Tok::Plus,
                Tok::Num(2.5),
                Tok::Star,
                Tok::Ident("x".into()),
                Tok::Eof
            ]
        );
    }

    #[test]
    fn comments_and_semicolons() {
        assert_eq!(
            toks("% hello\n1; % bye\n2"),
            vec![
                Tok::Newline,
                Tok::Num(1.0),
                Tok::Semicolon,
                Tok::Newline,
                Tok::Num(2.0),
                Tok::Eof
            ]
        );
    }

    #[test]
    fn line_continuation_glues_lines() {
        assert_eq!(
            toks("1 + ...\n 2"),
            vec![Tok::Num(1.0), Tok::Plus, Tok::Num(2.0), Tok::Eof]
        );
    }

    #[test]
    fn scientific_notation() {
        assert_eq!(toks("1e-6"), vec![Tok::Num(1e-6), Tok::Eof]);
        assert_eq!(toks(".5E+3"), vec![Tok::Num(500.0), Tok::Eof]);
        assert_eq!(toks("1."), vec![Tok::Num(1.0), Tok::Eof]);
    }

    #[test]
    fn malformed_exponent_is_parse_error() {
        let err = tokenize("1e").unwrap_err();
        assert_eq!(err.id, ids::PARSE_ERROR);
        assert_eq!(err.line, 1);
        assert_eq!(err.col, 1);
    }

    #[test]
    fn complex_literal_is_explicitly_unsupported() {
        let err = tokenize("3i").unwrap_err();
        assert_eq!(err.id, ids::UNSUPPORTED);
    }

    #[test]
    fn unknown_char_is_explicitly_unsupported() {
        let err = tokenize("1 & 2").unwrap_err();
        assert_eq!(err.id, ids::UNSUPPORTED);
        assert!(err.message_en.contains('&'));
    }

    #[test]
    fn name_length_limit_like_matlab() {
        let long = "a".repeat(64);
        let err = tokenize(&long).unwrap_err();
        assert_eq!(err.id, ids::UNSUPPORTED);
        let ok = "a".repeat(63);
        assert!(tokenize(&ok).is_ok());
    }

    #[test]
    fn unicode_positions_are_codepoint_based() {
        // کامنت فارسی قبل از خط دوم؛ موقعیت توکن بعدی باید درست باشد.
        let err = tokenize("% سلام\n1 & 2").unwrap_err();
        assert_eq!(err.line, 2);
    }
}
