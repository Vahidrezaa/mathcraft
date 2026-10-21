//! خطای ساختاریافته دوزبانه — قرارداد سند ۰۲ بخش ۹ و سند ۰۳ بخش ۱۴.
//!
//! این نوع، «قرارداد خطا» بین موتور و UI است (doc 07 §3 بند ۳):
//! هر خطا حتماً یکی از (الف) راه اصلاح، (ب) لینک مستند، (ج) توضیح مفهومی دارد.

use std::fmt;

/// شناسه‌های خطا مطابق کاتالوگ سند ۰۳ بخش ۱۴ (`MC:E*`).
pub mod ids {
    pub const UNDEFINED_VARIABLE: &str = "MC:E1001_UNDEFINED_VARIABLE";
    pub const SIZE_MISMATCH: &str = "MC:E2003_SIZE_MISMATCH";
    // MC:E2004 — خانواده عددی (mc-num): پیوت دقیقاً صفر در LU/مثلثی/Thomas (سند ۰۴ §3).
    pub const SINGULAR_MATRIX: &str = "MC:E2004_SINGULAR_MATRIX";
    // MC:E2005 — شکست Cholesky: ماتریس متقارن مثبت معین نیست (سند ۰۴ §3.1).
    pub const NOT_POSITIVE_DEFINITE: &str = "MC:E2005_NOT_POSITIVE_DEFINITE";
    pub const PARSE_ERROR: &str = "MC:E3001_PARSE_ERROR";
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
