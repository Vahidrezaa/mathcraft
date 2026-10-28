//! بودجهٔ تخصیص حافظه — ROADMAP گام 2.5 (بستن بندهای ۲ و ۱۵ ممیزی خارجی).
//!
//! **چرا این ماژول وجود دارد:** تخصیص ناموفق در Rust «abort» می‌شود
//! (SIGABRT، کد خروج ۱۳۴) و `catch_unwind` آن را مهار نمی‌کند — یعنی یک
//! `zeros(100000,100000)` کل پروسهٔ اپ را روی دستگاه می‌کشد (بند ۲ دفتر
//! COMPATIBILITY §4.8). راه‌حل: چکِ «قبل از تخصیص» در یک نقطهٔ واحد.
//!
//! **یک جای مستند (مطابق ROADMAP 2.5 وظیفهٔ ۱):**
//! - سقف پیش‌فرض: [`DEFAULT_MAX_ELEMENTS`] عنصر `f64` — ۱۰۰ میلیون عنصر
//!   = ۸۰۰ مگابایت؛ مناسب دستگاه‌های میان‌ردهٔ اندروید (۴ تا ۸ گیگ رم، با
//!   جای کافی برای خود اپ و چند ماتریس هم‌زمان).
//! - بازنویسی runtime: متغیر محیطی `MATHCRAFT_MAX_ELEMENTS` (عدد صحیح > ۰؛
//!   برای دسکتاپ/تست؛ روی اندروید ست نمی‌شود و پیش‌فرض برقرار است).
//!   مقدار نامعتبر یا ≤ ۰ نادیده گرفته می‌شود (بدون panic — سیاست گام 2.5).
//! - خطا: `MC:E4003_OUT_OF_MEMORY` (سند ۰۳ §14) — خانوادهٔ «منابع»
//!   (سطح ۴ سند ۰۲ §9) در کنار MC:E4002_CANCELLED؛ پیام در روح متلب
//!   («Out of memory») با راه اصلاح.

use std::sync::OnceLock;

use crate::error::{ids, McError};

/// سقف پیش‌فرض عناصر ماتریس (۱۰⁸ عنصر = ۸۰۰MB برای `f64`).
pub const DEFAULT_MAX_ELEMENTS: usize = 100_000_000;

/// متغیر محیطی برای بازنویسی سقف (دسکتاپ/تست — «یک جای مستند» همین‌جاست).
pub const ENV_OVERRIDE: &str = "MATHCRAFT_MAX_ELEMENTS";

fn max_elements_cell() -> &'static OnceLock<usize> {
    static CELL: OnceLock<usize> = OnceLock::new();
    &CELL
}

/// سقف فعلی عناصر مجاز برای هر تخصیص ماتریس (عناصر، نه بایت).
pub fn max_elements() -> usize {
    *max_elements_cell().get_or_init(|| {
        std::env::var(ENV_OVERRIDE)
            .ok()
            .and_then(|v| v.trim().parse::<usize>().ok())
            .filter(|&n| n > 0)
            .unwrap_or(DEFAULT_MAX_ELEMENTS)
    })
}

/// خطای OOM استاندارد — متن دوزبانه در روح متلب + راه اصلاح (سند ۰۳ §14).
pub fn out_of_memory_error(rows: usize, cols: usize) -> McError {
    // اگر خود ضرب هم سرریز کند، بایت‌ها قابل محاسبه نیستند — بدون MB چاپ می‌کنیم.
    // واحد: مگابایت اعشاری (÷۱۰⁶) — هماهنگ با «۸۰۰MB» همین سند و ROADMAP 2.5.
    let size_note = match rows.checked_mul(cols) {
        Some(n) => match n.checked_mul(std::mem::size_of::<f64>()) {
            Some(bytes) => format!(" ({:.1} MB)", bytes as f64 / 1_000_000.0),
            None => String::new(),
        },
        None => String::new(),
    };
    let limit_mb = max_elements() as f64 * 8.0 / 1_000_000.0;
    McError::new(
        ids::OUT_OF_MEMORY,
        0,
        0,
        format!(
            "Out of memory: requested {rows}x{cols} array{size_note} exceeds the \
             allocation budget ({limit_mb:.0} MB). Reduce the problem size."
        ),
        format!(
            "خارج از حافظه: آرایهٔ {rows}×{cols}{size_note_fa} از بودجهٔ تخصیص \
             ({limit_mb:.0} مگابایت) فراتر می‌رود. اندازهٔ مسئله را کاهش دهید.",
            size_note_fa = size_note.replace(" MB", " مگابایت")
        ),
    )
}

/// چک بودجه **قبل از تخصیص**: `rows × cols` (با ضرب امن) باید ≤ سقف باشد.
///
/// خروجی `Ok(n)` = تعداد عناصر مجاز (برای استفادهٔ مستقیم در تخصیص).
pub fn check_elements(rows: usize, cols: usize) -> Result<usize, McError> {
    check_elements_with(rows, cols, max_elements())
}

/// نسخهٔ تزریق‌پذیر سقف — برای تست مرز بودجه بدون دست‌کاری متغیر محیطی.
pub fn check_elements_with(rows: usize, cols: usize, limit: usize) -> Result<usize, McError> {
    match rows.checked_mul(cols) {
        Some(n) if n <= limit => Ok(n),
        _ => Err(out_of_memory_error(rows, cols)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boundary_exact_limit_passes_one_over_fails() {
        // مرز دقیق: خودِ سقف مجاز است، سقف+۱ خطا.
        assert_eq!(check_elements_with(10, 10, 100).unwrap(), 100);
        assert!(check_elements_with(10, 11, 100).is_err());
        // سقف بسیار بزرگ = عملاً بدون محدودیت.
        assert!(check_elements_with(1000, 1000, usize::MAX).is_ok());
    }

    #[test]
    fn zero_dimensions_always_pass() {
        // ماتریس خالی (1×0، 0×0) همیشه مجاز است.
        assert_eq!(check_elements_with(0, 0, 0).unwrap(), 0);
        assert_eq!(check_elements_with(1, 0, 0).unwrap(), 0);
        assert_eq!(check_elements_with(0, 5, 0).unwrap(), 0);
    }

    #[test]
    fn overflow_is_oom_not_panic() {
        // ابعاد سرریزکنندهٔ usize — باید خطای OOM بدهد، نه panic/abort.
        let err = check_elements_with(usize::MAX, usize::MAX, 100).unwrap_err();
        assert_eq!(err.id, ids::OUT_OF_MEMORY);
        assert!(err.message_en.contains("Out of memory"));
        // یک بعد بزرگ + یک بعد کوچک: بدون سرریز ولی بالای سقف.
        assert!(check_elements_with(usize::MAX / 4, 8, 100).is_err());
    }

    #[test]
    fn error_message_is_matlab_spirit_bilingual_with_remedy() {
        let e = check_elements_with(100_000, 100_000, DEFAULT_MAX_ELEMENTS).unwrap_err();
        assert_eq!(e.id, ids::OUT_OF_MEMORY);
        // روح متلب: «Out of memory» + ابعاد درخواستی + بودجه + راه اصلاح.
        assert!(e.message_en.contains("Out of memory"));
        assert!(e.message_en.contains("100000x100000"));
        // 1e10 عنصر × ۸ بایت = ۸×۱۰¹⁰ بایت = 80000.0 MB اعشاری.
        assert!(e.message_en.contains("80000.0 MB"));
        assert!(e.message_en.contains("Reduce the problem size"));
        assert!(e.message_fa.contains("خارج از حافظه"));
        assert!(e.message_fa.contains("کاهش دهید"));
    }
}
