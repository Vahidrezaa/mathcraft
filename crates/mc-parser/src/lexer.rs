//! Lexer — ساختار واژگانی سند ۰۳ بخش ۲ (نسخه P1).
//!
//! موقعیت‌ها ۱-پایه‌اند (خط، ستون) و ستون به‌ازای هر codepoint جلو می‌رود
//! (سازگار با قاعده یونیکد سند ۰۳ بخش ۱۰).
//!
//! P1 اضافه می‌کند (سند ۰۳ بخش‌های ۴-۹): ماتریس لفظی `[]`، کولن، مقایسه/منطق،
//! عملگرهای عنصری `.* ./ .^`، بک‌اسلش `\`، ترانهاده `'`/`.'`، رشته‌ها `'..'`/`".."`،
//! توابع بی‌نام `@`، آکولاد `{}` برای case-list.
//!
//! دو پرچم حیاتی برای ابهام‌زدایی متلب:
//! - `space_before`: فاصلهٔ واقعی قبل از توکن — قانون سند ۰۳ بخش ۴:
//!   `[1 -2]` = دو عنصر (منوس یگانی: فاصله قبل، بدون فاصله بعد) ولی
//!   `[1 - 2]` و `[1-2]` = یک عنصر (دویی). پارسر ترکیب
//!   «space_before علامت» و «space_before توکن بعدی» را می‌سنجد.
//! - `last_was_operand_end`: توکن قبلی پایان‌یافتهٔ عملوند است — تمایز
//!   ترانهاده `'` از شروع رشته (سند ۰۳ بخش ۵: `a'` ترانهاده، `f('x')` رشته).

use mc_core::{ids, McError};

#[derive(Debug, Clone, PartialEq)]
pub enum Tok {
    Num(f64),
    /// عدد موهومی `4i`/`2.5j` — اندیس فقط اندازه؛ علامت با Unary مینوس می‌آید
    Imag(f64),
    Ident(String),
    Str(String),
    Plus,
    Minus,
    Star,
    Slash,
    Caret,
    Backslash,
    DotStar,
    DotSlash,
    DotCaret,
    /// ترانهاده مزدوج `'` (فقط بعد از پایان‌یافتهٔ عملوند)
    Transpose,
    /// ترانهاده بدون مزدوج `.'`
    DotTranspose,
    Colon,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    AndAnd,
    OrOr,
    Not,
    At,
    LParen,
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,
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
    /// فاصلهٔ واقعی (space/tab/خط جدید) قبل از این توکن — ابهام‌زدایی ماتریس (سند ۰۳ بخش ۴).
    pub space_before: bool,
}

impl Token {
    /// توصیف کوتاه برای پیام‌های خطا (سند ۰۳ بخش ۱۴: «'x' غیرمنتظره»).
    pub fn describe(&self) -> String {
        match &self.tok {
            Tok::Num(_) | Tok::Imag(_) => "number".to_owned(),
            Tok::Ident(n) => format!("'{n}'"),
            Tok::Str(_) => "string".to_owned(),
            Tok::Plus => "'+'".to_owned(),
            Tok::Minus => "'-'".to_owned(),
            Tok::Star => "'*'".to_owned(),
            Tok::Slash => "'/'".to_owned(),
            Tok::Caret => "'^'".to_owned(),
            Tok::Backslash => "'\\'".to_owned(),
            Tok::DotStar => "'.*'".to_owned(),
            Tok::DotSlash => "'./'".to_owned(),
            Tok::DotCaret => "'.^'".to_owned(),
            Tok::Transpose => "'''".to_owned(),
            Tok::DotTranspose => "\" .'\"".to_owned(),
            Tok::Colon => "':'".to_owned(),
            Tok::Eq => "'=='".to_owned(),
            Tok::Ne => "'~='".to_owned(),
            Tok::Lt => "'<'".to_owned(),
            Tok::Le => "'<='".to_owned(),
            Tok::Gt => "'>'".to_owned(),
            Tok::Ge => "'>='".to_owned(),
            Tok::AndAnd => "'&&'".to_owned(),
            Tok::OrOr => "'||'".to_owned(),
            Tok::Not => "'~'".to_owned(),
            Tok::At => "'@'".to_owned(),
            Tok::LParen => "'('".to_owned(),
            Tok::RParen => "')'".to_owned(),
            Tok::LBracket => "'['".to_owned(),
            Tok::RBracket => "']'".to_owned(),
            Tok::LBrace => "'{'".to_owned(),
            Tok::RBrace => "'}'".to_owned(),
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
    /// فاصلهٔ انباشته‌شده قبل از توکن بعدی.
    pending_space: bool,
    /// توکن قبلی پایان‌یافتهٔ عملوند؟ (تمایز `'` ترانهاده/رشته)
    last_was_operand_end: bool,
    /// آیا در این سطر (قبل از موقعیت فعلی) کد/توکنی آمده؟ — شرط کامنت
    /// بلوکی متلب: `%{` و `%}` باید تنها محتوای سطر خودشان باشند
    /// (ROADMAP 2.4 — بخشی از آیتم ۹ ممیزی).
    line_has_code: bool,
}

/// Maximum significant identifier length, like MATLAB `namelengthmax` (doc 03 §2).
const MAX_NAME_LEN: usize = 63;

pub fn tokenize(src: &str) -> Result<Vec<Token>, McError> {
    let mut lx = Lexer {
        chars: src.chars().collect(),
        pos: 0,
        line: 1,
        col: 1,
        pending_space: false,
        last_was_operand_end: false,
        line_has_code: false,
    };
    let mut out: Vec<Token> = Vec::new();
    while let Some(c) = lx.peek() {
        match c {
            ' ' | '\t' | '\r' => {
                lx.pending_space = true;
                lx.advance();
            }
            '\n' => {
                lx.pending_space = true;
                out.push(lx.make(Tok::Newline));
                lx.advance();
                lx.line += 1;
                lx.col = 1;
                lx.last_was_operand_end = false;
                lx.line_has_code = false;
            }
            '%' => {
                // کامنت بلوکی متلب `%{ ... %}` (ROADMAP 2.4 — بخشی از آیتم ۹):
                // فقط وقتی %{ در ابتدای سطر (فقط فاصله/تب قبلش) و تنها محتوای
                // سطر باشد. در غیر این صورت کامنت خطی تا انتهای سطر.
                if !lx.line_has_code && lx.starts_with("%{") && lx.rest_of_line_blank_from(2) {
                    lx.skip_block_comment()?;
                } else {
                    // کامنت تا انتهای خط (سند ۰۳ بخش ۲) — `%%` هم همین‌جا پوشش داده می‌شود.
                    while let Some(ch) = lx.peek() {
                        if ch == '\n' {
                            break;
                        }
                        lx.advance();
                    }
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
            '\'' if lx.last_was_operand_end => {
                // ترانهاده مزدوج — فقط بعد از پایان‌یافتهٔ عملوند (سند ۰۳ بخش ۵).
                out.push(lx.make(Tok::Transpose));
                lx.advance();
                lx.last_was_operand_end = true;
            }
            '\'' => out.push(lx.lex_char_string()?),
            '"' => out.push(lx.lex_double_string()?),
            '@' => {
                out.push(lx.make(Tok::At));
                lx.advance();
                lx.last_was_operand_end = false;
            }
            '+' => {
                out.push(lx.make(Tok::Plus));
                lx.advance();
                lx.last_was_operand_end = false;
            }
            '-' => {
                out.push(lx.make(Tok::Minus));
                lx.advance();
                lx.last_was_operand_end = false;
            }
            '*' => {
                out.push(lx.make(Tok::Star));
                lx.advance();
                lx.last_was_operand_end = false;
            }
            '/' => {
                out.push(lx.make(Tok::Slash));
                lx.advance();
                lx.last_was_operand_end = false;
            }
            '\\' => {
                out.push(lx.make(Tok::Backslash));
                lx.advance();
                lx.last_was_operand_end = false;
            }
            '^' => {
                out.push(lx.make(Tok::Caret));
                lx.advance();
                lx.last_was_operand_end = false;
            }
            '.' => {
                // عملگرهای عنصری .* ./ .^ .' (عدد/ادامه‌خط در بالا هندل شده‌اند)
                match lx.peek_at(1) {
                    Some('*') => {
                        out.push(lx.make(Tok::DotStar));
                        lx.advance();
                        lx.advance();
                    }
                    Some('/') => {
                        out.push(lx.make(Tok::DotSlash));
                        lx.advance();
                        lx.advance();
                    }
                    Some('^') => {
                        out.push(lx.make(Tok::DotCaret));
                        lx.advance();
                        lx.advance();
                    }
                    Some('\'') => {
                        out.push(lx.make(Tok::DotTranspose));
                        lx.advance();
                        lx.advance();
                        lx.last_was_operand_end = true;
                    }
                    _ => {
                        return Err(lx.err_here(
                            ids::UNSUPPORTED,
                            "Unsupported '.' usage (elementwise ops are .* ./ .^ .').",
                            "استفادهٔ ناشناخته از «.» (عملگرهای عنصری: .* ./ .^ .').",
                        ));
                    }
                }
            }
            ':' => {
                out.push(lx.make(Tok::Colon));
                lx.advance();
                lx.last_was_operand_end = false;
            }
            '=' => {
                if lx.peek_at(1) == Some('=') {
                    out.push(lx.make(Tok::Eq));
                    lx.advance();
                    lx.advance();
                } else {
                    out.push(lx.make(Tok::Assign));
                    lx.advance();
                }
                lx.last_was_operand_end = false;
            }
            '~' => {
                if lx.peek_at(1) == Some('=') {
                    out.push(lx.make(Tok::Ne));
                    lx.advance();
                    lx.advance();
                } else {
                    out.push(lx.make(Tok::Not));
                    lx.advance();
                }
                lx.last_was_operand_end = false;
            }
            '<' => {
                if lx.peek_at(1) == Some('=') {
                    out.push(lx.make(Tok::Le));
                    lx.advance();
                    lx.advance();
                } else {
                    out.push(lx.make(Tok::Lt));
                    lx.advance();
                }
                lx.last_was_operand_end = false;
            }
            '>' => {
                if lx.peek_at(1) == Some('=') {
                    out.push(lx.make(Tok::Ge));
                    lx.advance();
                    lx.advance();
                } else {
                    out.push(lx.make(Tok::Gt));
                    lx.advance();
                }
                lx.last_was_operand_end = false;
            }
            '&' => {
                if lx.peek_at(1) == Some('&') {
                    out.push(lx.make(Tok::AndAnd));
                    lx.advance();
                    lx.advance();
                } else {
                    // & تک‌نویسی (and برداری) — P2؛ فعلاً خطای صریح
                    return Err(lx.err_here(
                        ids::UNSUPPORTED,
                        "Single '&' (elementwise and) arrives in a later milestone; use '&&'.",
                        "«&» تکی (and عنصری) در مایل‌استون بعدی می‌آید؛ از '&&' استفاده کنید.",
                    ));
                }
                lx.last_was_operand_end = false;
            }
            '|' => {
                if lx.peek_at(1) == Some('|') {
                    out.push(lx.make(Tok::OrOr));
                    lx.advance();
                    lx.advance();
                } else {
                    return Err(lx.err_here(
                        ids::UNSUPPORTED,
                        "Single '|' (elementwise or) arrives in a later milestone; use '||'.",
                        "«|» تکی (or عنصری) در مایل‌استون بعدی می‌آید؛ از '||' استفاده کنید.",
                    ));
                }
                lx.last_was_operand_end = false;
            }
            '(' => {
                out.push(lx.make(Tok::LParen));
                lx.advance();
                lx.last_was_operand_end = false;
            }
            ')' => {
                out.push(lx.make(Tok::RParen));
                lx.advance();
                lx.last_was_operand_end = true;
            }
            '[' => {
                out.push(lx.make(Tok::LBracket));
                lx.advance();
                lx.last_was_operand_end = false;
            }
            ']' => {
                out.push(lx.make(Tok::RBracket));
                lx.advance();
                lx.last_was_operand_end = true;
            }
            '{' => {
                out.push(lx.make(Tok::LBrace));
                lx.advance();
                lx.last_was_operand_end = false;
            }
            '}' => {
                out.push(lx.make(Tok::RBrace));
                lx.advance();
                lx.last_was_operand_end = true;
            }
            ',' => {
                out.push(lx.make(Tok::Comma));
                lx.advance();
                lx.last_was_operand_end = false;
            }
            ';' => {
                out.push(lx.make(Tok::Semicolon));
                lx.advance();
                lx.last_was_operand_end = false;
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
        space_before: true,
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

    /// آیا از offset فعلی + skip تا انتهای سطر فقط فاصله/تب است؟
    /// (شرط «تنها محتوای سطر» برای %{ و %})
    fn rest_of_line_blank_from(&self, skip: usize) -> bool {
        let mut k = skip;
        loop {
            match self.peek_at(k) {
                Some(' ') | Some('\t') | Some('\r') => k += 1,
                Some('\n') | None => return true,
                Some(_) => return false,
            }
        }
    }

    /// پرش کامنت بلوکی — نشانگر روی `%` آغازین است. سطرهای بلوک هیچ توکنی
    /// نمی‌دهند؛ سطرِ `%}` مصرف می‌شود ولی newline آن به جریان عادی می‌ماند
    /// (مثل یک سطر خالی). `%}` باید تنها محتوای سطر خودش باشد؛
    /// EOF بدون بستن → خطای پارس (رفتار متلب). تو در تو پشتیبانی نمی‌شود
    /// (مثل متلب — %{ داخلی فقط متن کامنت است).
    fn skip_block_comment(&mut self) -> Result<(), McError> {
        // بقیهٔ سطر آغازین تا newline
        while let Some(ch) = self.peek() {
            if ch == '\n' {
                break;
            }
            self.advance();
        }
        if self.peek().is_none() {
            return Err(self.unterminated_block_comment());
        }
        self.advance(); // newline سطر آغازین
        self.line += 1;
        self.col = 1;
        self.pending_space = true;
        self.line_has_code = false;
        loop {
            // فاصله/تب ابتدای سطر
            while matches!(self.peek(), Some(' ') | Some('\t') | Some('\r')) {
                self.advance();
            }
            if self.starts_with("%}") && self.rest_of_line_blank_from(2) {
                // مصرف %} و بقیهٔ سطر؛ newline بعدش عادی پردازش می‌شود
                while let Some(ch) = self.peek() {
                    if ch == '\n' {
                        break;
                    }
                    self.advance();
                }
                return Ok(());
            }
            match self.peek() {
                Some('\n') => {
                    self.advance();
                    self.line += 1;
                    self.col = 1;
                }
                Some(_) => self.advance(),
                None => return Err(self.unterminated_block_comment()),
            }
        }
    }

    fn unterminated_block_comment(&self) -> McError {
        self.err_here(
            ids::PARSE_ERROR,
            "Unterminated block comment: '%{' without a closing '%}' on its own line.",
            "کامنت بلوکی بسته نشده: «%{» بدون «%}» تنها در یک سطر.",
        )
    }

    fn advance(&mut self) {
        self.pos += 1;
        self.col += 1;
    }

    /// ساخت توکن با مصرف `pending_space` (فقط یک‌بار مصرف می‌شود).
    fn make(&mut self, tok: Tok) -> Token {
        let space_before = self.pending_space;
        self.pending_space = false;
        self.line_has_code = true;
        Token {
            tok,
            line: self.line,
            col: self.col,
            space_before,
        }
    }

    fn err_here(&self, id: &str, en: impl Into<String>, fa: impl Into<String>) -> McError {
        McError::new(id, self.line, self.col, en, fa)
    }

    /// عدد اعشاری با نماد علمی (سند ۰۳ بخش ۲: `3`، `2.5`، `1e-6`؛ پسوند `_` مجاز نیست).
    fn lex_number(&mut self) -> Result<Token, McError> {
        let start_line = self.line;
        let start_col = self.col;
        let space_before = self.pending_space;
        self.pending_space = false;
        let mut text = String::new();

        while self.peek().is_some_and(|c| c.is_ascii_digit()) {
            text.push(self.peek().unwrap_or('0'));
            self.advance();
        }
        // نقطهٔ اعشار فقط اگر رقم بعدش بیاید یا پایان ورودی باشد — رفتار متلب:
        // `2.'` و `2.*3` نقطه را نمی‌بلعد (نقطه متعلق به .'/.* است) ولی `1.5` و `1.` اعشارند.
        if self.peek() == Some('.') && self.peek_at(1).is_none_or(|c| c.is_ascii_digit()) {
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

        // پسوند موهومی i/j — سند ۰۳ بخش ۳ (P2): `4i` توکن Imag(4) می‌دهد.
        // `i` برهنه همچنان Ident است (متغیر/ثابت) — این مسیر فقط پس از رقم می‌آید.
        let imag = matches!(self.peek(), Some('i') | Some('j'));
        if imag {
            self.advance();
        }

        let value: f64 = text.parse().unwrap_or(f64::INFINITY); // سرریز مثل 1e999 → Inf (رفتار IEEE، سند ۰۳ بخش ۱۱)
        self.last_was_operand_end = true;
        if imag {
            return Ok(Token {
                tok: Tok::Imag(value),
                line: start_line,
                col: start_col,
                space_before,
            });
        }
        Ok(Token {
            tok: Tok::Num(value),
            line: start_line,
            col: start_col,
            space_before,
        })
    }

    fn lex_ident(&mut self) -> Result<Token, McError> {
        let start_line = self.line;
        let start_col = self.col;
        let space_before = self.pending_space;
        self.pending_space = false;
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
        self.last_was_operand_end = true;
        Ok(Token {
            tok: Tok::Ident(name),
            line: start_line,
            col: start_col,
            space_before,
        })
    }

    /// رشته کاراکتری با `'` (سند ۰۳ بخش ۱۰) — `''` داخل رشته = خود `'`.
    fn lex_char_string(&mut self) -> Result<Token, McError> {
        let start_line = self.line;
        let start_col = self.col;
        let space_before = self.pending_space;
        self.pending_space = false;
        self.advance(); // کوتیشن باز
        let mut s = String::new();
        loop {
            match self.peek() {
                None | Some('\n') => {
                    return Err(McError::new(
                        ids::PARSE_ERROR,
                        start_line,
                        start_col,
                        "Unterminated character string (missing closing ').",
                        "رشته کاراکتری بسته‌نشده (کوتیشن پایانی ' جا افتاده).",
                    ));
                }
                Some('\'') => {
                    if self.peek_at(1) == Some('\'') {
                        s.push('\'');
                        self.advance();
                        self.advance();
                    } else {
                        self.advance(); // کوتیشن بسته
                        break;
                    }
                }
                Some(ch) => {
                    s.push(ch);
                    self.advance();
                }
            }
        }
        self.last_was_operand_end = true;
        Ok(Token {
            tok: Tok::Str(s),
            line: start_line,
            col: start_col,
            space_before,
        })
    }

    /// رشته scalar با `"` (سند ۰۳ بخش ۱۰) — `""` داخل رشته = خود `"`.
    fn lex_double_string(&mut self) -> Result<Token, McError> {
        let start_line = self.line;
        let start_col = self.col;
        let space_before = self.pending_space;
        self.pending_space = false;
        self.advance();
        let mut s = String::new();
        loop {
            match self.peek() {
                None | Some('\n') => {
                    return Err(McError::new(
                        ids::PARSE_ERROR,
                        start_line,
                        start_col,
                        "Unterminated string (missing closing \").",
                        "رشته بسته‌نشده (کوتیشن پایانی \" جا افتاده).",
                    ));
                }
                Some('"') => {
                    if self.peek_at(1) == Some('"') {
                        s.push('"');
                        self.advance();
                        self.advance();
                    } else {
                        self.advance();
                        break;
                    }
                }
                Some(ch) => {
                    s.push(ch);
                    self.advance();
                }
            }
        }
        self.last_was_operand_end = true;
        Ok(Token {
            tok: Tok::Str(s),
            line: start_line,
            col: start_col,
            space_before,
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
    fn block_comments_are_swallowed() {
        // ROADMAP 2.4 — بخشی از آیتم ۹: %{ ... %} (فقط سطر مستقل)
        assert_eq!(
            toks("a = 1\n%{\njunk ~%% $\n%}\nb = 2"),
            vec![
                Tok::Ident("a".into()),
                Tok::Assign,
                Tok::Num(1.0),
                Tok::Newline,
                Tok::Newline,
                Tok::Ident("b".into()),
                Tok::Assign,
                Tok::Num(2.0),
                Tok::Eof
            ]
        );
    }

    #[test]
    fn block_comment_at_start_with_indent() {
        // بلوک در ابتدای برنامه + تورفتگی سطر بستن
        assert_eq!(
            toks("%{\nhidden\n   %}\n1"),
            vec![Tok::Newline, Tok::Num(1.0), Tok::Eof]
        );
    }

    #[test]
    fn opener_with_trailing_content_is_line_comment() {
        // %{ با محتوای بعدش در همان سطر = کامنت خطی (نه بلوک)
        assert_eq!(toks("%{ x\n1"), vec![Tok::Newline, Tok::Num(1.0), Tok::Eof]);
        // %{ بعد از کد در همان سطر هم کامنت خطی است
        assert_eq!(
            toks("x = 1 %{ note\n2"),
            vec![
                Tok::Ident("x".into()),
                Tok::Assign,
                Tok::Num(1.0),
                Tok::Newline,
                Tok::Num(2.0),
                Tok::Eof
            ]
        );
    }

    #[test]
    fn closer_with_trailing_content_does_not_close() {
        // %} با محتوای بعدش سطر بستن نیست → بلوک ادامه می‌یابد → EOF خطا
        let err = tokenize("%{\nx\n%} x\n1").unwrap_err();
        assert_eq!(err.id, ids::PARSE_ERROR);
    }

    #[test]
    fn unterminated_block_comment_is_parse_error() {
        let err = tokenize("a = 1\n%{\nno close").unwrap_err();
        assert_eq!(err.id, ids::PARSE_ERROR);
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
    fn complex_literal_is_tokenized_as_imag() {
        // P2: `3i` توکن Imag(3) می‌دهد (سند ۰۳ بخش ۳)
        let toks = tokenize("3i").unwrap_or_else(|e| panic!("{e:?}"));
        assert!(matches!(toks[0].tok, Tok::Imag(v) if v == 3.0));
        // `2.5j` هم همین‌طور
        let toks = tokenize("2.5j").unwrap_or_else(|e| panic!("{e:?}"));
        assert!(matches!(toks[0].tok, Tok::Imag(v) if v == 2.5));
        // `i` برهنه هنوز Ident است (ثابت/متغیر)
        let toks = tokenize("i").unwrap_or_else(|e| panic!("{e:?}"));
        assert!(matches!(toks[0].tok, Tok::Ident(_)));
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

    // ---- تست‌های P1 ----

    #[test]
    fn transpose_vs_string_disambiguation() {
        // بعد از پایان‌یافته → ترانهاده؛ بعد از عملگر/شروع → رشته
        assert_eq!(
            toks("a'"),
            vec![Tok::Ident("a".into()), Tok::Transpose, Tok::Eof]
        );
        assert_eq!(
            toks("2.'"),
            vec![Tok::Num(2.0), Tok::DotTranspose, Tok::Eof]
        );
        assert_eq!(
            toks("f('x')"),
            vec![
                Tok::Ident("f".into()),
                Tok::LParen,
                Tok::Str("x".into()),
                Tok::RParen,
                Tok::Eof
            ]
        );
        // '' escape
        assert_eq!(toks("'it''s'"), vec![Tok::Str("it's".into()), Tok::Eof]);
        // رشته با "
        assert_eq!(toks("\"سلام\""), vec![Tok::Str("سلام".into()), Tok::Eof]);
    }

    #[test]
    fn comparison_and_logical_tokens() {
        assert_eq!(
            toks("a == b ~= c <= d >= e && f || ~g"),
            vec![
                Tok::Ident("a".into()),
                Tok::Eq,
                Tok::Ident("b".into()),
                Tok::Ne,
                Tok::Ident("c".into()),
                Tok::Le,
                Tok::Ident("d".into()),
                Tok::Ge,
                Tok::Ident("e".into()),
                Tok::AndAnd,
                Tok::Ident("f".into()),
                Tok::OrOr,
                Tok::Not,
                Tok::Ident("g".into()),
                Tok::Eof
            ]
        );
    }

    #[test]
    fn elementwise_and_backslash_tokens() {
        assert_eq!(
            toks("A.*2 ./b .^c \\d"),
            vec![
                Tok::Ident("A".into()),
                Tok::DotStar,
                Tok::Num(2.0),
                Tok::DotSlash,
                Tok::Ident("b".into()),
                Tok::DotCaret,
                Tok::Ident("c".into()),
                Tok::Backslash,
                Tok::Ident("d".into()),
                Tok::Eof
            ]
        );
    }

    #[test]
    fn space_flags_for_matrix_ambiguity() {
        // [1 -2] — منوس با فاصلهٔ قبلی → یگانی (پارسر با نگاه به بعدی تصمیم می‌گیرد)
        let ts = tokenize("[1 -2]").unwrap();
        assert!(ts[2].space_before, "spaced minus flagged");

        // [1 - 2] — منوس با فاصله ولی توکن بعدی هم فاصله دارد → دوویی
        let ts2 = tokenize("[1 - 2]").unwrap();
        assert!(ts2[2].space_before);
        // [1- 2] — منوس چسبیده به 1، فاصله بعد → بدون فاصله قبل
        assert!(!tokenize("[1- 2]").unwrap()[2].space_before);
        // [1-2] — منوس چسبیده → بدون فاصله
        assert!(!tokenize("[1-2]").unwrap()[2].space_before);
    }

    #[test]
    fn colon_and_brackets() {
        assert_eq!(
            toks("a(1:end)"),
            vec![
                Tok::Ident("a".into()),
                Tok::LParen,
                Tok::Num(1.0),
                Tok::Colon,
                Tok::Ident("end".into()),
                Tok::RParen,
                Tok::Eof
            ]
        );
        assert_eq!(
            toks("[1 2; 3 4]"),
            vec![
                Tok::LBracket,
                Tok::Num(1.0),
                Tok::Num(2.0),
                Tok::Semicolon,
                Tok::Num(3.0),
                Tok::Num(4.0),
                Tok::RBracket,
                Tok::Eof
            ]
        );
    }

    #[test]
    fn anonymous_fn_at_token() {
        assert_eq!(
            toks("f = @(x) x.^2"),
            vec![
                Tok::Ident("f".into()),
                Tok::Assign,
                Tok::At,
                Tok::LParen,
                Tok::Ident("x".into()),
                Tok::RParen,
                Tok::Ident("x".into()),
                Tok::DotCaret,
                Tok::Num(2.0),
                Tok::Eof
            ]
        );
    }
}
