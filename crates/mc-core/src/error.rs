//! خطای ساختاریافته دوزبانه — قرارداد سند ۰۲ بخش ۹ و سند ۰۳ بخش ۱۴.
//!
//! این نوع، «قرارداد خطا» بین موتور و UI است (doc 07 §3 بند ۳):
//! هر خطا حتماً یکی از (الف) راه اصلاح، (ب) لینک مستند، (ج) توضیح مفهومی دارد.

use std::fmt;

/// شناسه‌های خطا مطابق کاتالوگ سند ۰۳ بخش ۱۴ (`MC:E*`).
///
/// تاکسونومی ROADMAP 2.11 (بند ۱۴ ممیزی، سطح‌بندی سند ۰۲ §9): E3001 فقط
/// برای خطای **سینتکسی** (lexer/parser)؛ خطای معنایی ارزیابی خانوادهٔ خودش را
/// می‌گیرد — نام‌ها (E1xxx)، دامنهٔ اندیس/ابعاد/نوع (E2xxx)، عددی (E2xxx)،
/// منابع (E4xxx)، مرزهای صادقانه (E9xxx).
pub mod ids {
    pub const UNDEFINED_VARIABLE: &str = "MC:E1001_UNDEFINED_VARIABLE";
    // MC:E1002 — نام شناخته‌نشده در موقعیت فراخوانی (کاتالوگ سند ۰۳ §۱۴؛
    // ROADMAP 2.11): `lenght(x)` → did-you-mean روی توابع/builtinها.
    pub const UNDEFINED_FUNCTION: &str = "MC:E1002_UNDEFINED_FUNCTION";
    // MC:E2001 — اندیس خارج از بازه (کاتالوگ؛ ROADMAP 2.11): همهٔ مسیرهای
    // خواندن/نوشتنِ linear و دو بعدی.
    pub const INDEX_OUT_OF_BOUNDS: &str = "MC:E2001_INDEX_OUT_OF_BOUNDS";
    // MC:E2002 — خطای نوع/کاربرد در سطح اجرا (سطح ۲ «اجرایی» سند ۰۲ §9):
    // کلاس نادرست، تعداد آرگومان، نام فیلد نامعتبر، هدف اندیس‌گذاری ناپذیر و…
    // (خانهٔ خالی خانوادهٔ E2xxx؛ ROADMAP 2.11).
    pub const TYPE_ERROR: &str = "MC:E2002_TYPE_ERROR";
    pub const SIZE_MISMATCH: &str = "MC:E2003_SIZE_MISMATCH";
    // MC:E2004 — خانواده عددی (mc-num): پیوت دقیقاً صفر در LU/مثلثی/Thomas (سند ۰۴ §3).
    pub const SINGULAR_MATRIX: &str = "MC:E2004_SINGULAR_MATRIX";
    // MC:E2005 — شکست Cholesky: ماتریس متقارن مثبت معین نیست (سند ۰۴ §3.1).
    pub const NOT_POSITIVE_DEFINITE: &str = "MC:E2005_NOT_POSITIVE_DEFINITE";
    pub const PARSE_ERROR: &str = "MC:E3001_PARSE_ERROR";
    // MC:E4001 — شکست تلورانس حلگر ODE (سند ۰۴ §4). MC:E4002 — لغو همکاری‌گر:
    // سطح ۴ «منابع» سند ۰۲ §9 (timeout/cancel)؛ نتیجهٔ جزئی دور ریخته می‌شود
    // و FFI آن را به JobStatus::Cancelled نگاشت می‌کند (ROADMAP 2.3).
    pub const CANCELLED: &str = "MC:E4002_CANCELLED";

    /// خطای فایل/دادهٔ I/O — گام 3.7 (گم‌بودن فایل، قالب خراب، نامعتبر).
    pub const FILE_ERROR: &str = "MC:E5001_FILE_ERROR";
    // MC:E4003 — سقف تخصیص حافظه: چک «قبل از تخصیص» در mc-core::budget
    // (ROADMAP 2.5 — بستن بند ۲ ممیزی؛ abort تخصیص با catch_unwind مهار نمی‌شود).
    pub const OUT_OF_MEMORY: &str = "MC:E4003_OUT_OF_MEMORY";
    pub const UNSUPPORTED: &str = "MC:E9xxx_UNSUPPORTED";
}

/// خطای ساختاریافته: کد، موقعیت، پیام FA/EN و پیشنهاد (did-you-mean).
///
/// پیام‌های `message_*` «جمله اول» خطا هستند؛ بخش «منظورتان X بود؟» از
/// طریق `suggestion` جداگانه منتقل می‌شود تا UI آن را با l10n خودش بچیند.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct McError {
    /// مثل `MC:E3001_PARSE_ERROR` (سند ۰۲ بخش ۳: `id` مثل MC:E1001_UNDEFINED).
    pub id: String,
    pub line: u32,
    pub col: u32,
    pub message_en: String,
    pub message_fa: String,
    /// did-you-mean: فاصله Levenshtein ≤ ۲ روی workspace + builtins (سند ۰۲ بخش ۹).
    pub suggestion: Option<String>,
}

impl McError {
    pub fn new(
        id: &str,
        line: u32,
        col: u32,
        message_en: impl Into<String>,
        message_fa: impl Into<String>,
    ) -> Self {
        Self {
            id: id.to_owned(),
            line,
            col,
            message_en: message_en.into(),
            message_fa: message_fa.into(),
            suggestion: None,
        }
    }

    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }
}

impl fmt::Display for McError {
    /// نمایش میزکاری (EN) — برای لاگ و تست؛ UI هرگز این را مستقیم نشان نمی‌دهد.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} at {}:{}: {}",
            self.id, self.line, self.col, self.message_en
        )?;
        if let Some(s) = &self.suggestion {
            write!(f, " (did you mean '{s}'?)")?;
        }
        Ok(())
    }
}

impl std::error::Error for McError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_contains_id_position_and_message() {
        let e = McError::new(
            ids::PARSE_ERROR,
            1,
            5,
            "Unexpected ')': expected expression.",
            "«)» غیرمنتظره: عبارت انتظار می‌رفت.",
        )
        .with_suggestion("length");
        let s = e.to_string();
        assert!(s.starts_with("MC:E3001_PARSE_ERROR at 1:5:"));
        assert!(s.contains("did you mean 'length'?"));
    }
}
