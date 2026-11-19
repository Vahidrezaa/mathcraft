//! mc-io — ورودی/خروجی فایل و دادهٔ گام 3.7 (ROADMAP §3.7، سند ۱۴ §۵.۱).
//!
//! چهار ماژول، همه روی انواع فعلی زبان (بدون نوع زبانی جدید — پیمان ضد-انحراف):
//!
//! ```text
//! mcws.rs   قالب MathCraft Workspace v1 — رفت‌وبرگشت بیت‌به‌بیت
//!           Double/Complex/Char/Struct (f64::to_bits؛ NaN payload و -0 حفظ می‌شود)
//! matv5.rs  خوانندهٔ زیرمجموعهٔ .mat Level 5 (عددی/char/struct) —
//!           اوراکل CI = فایل‌های تولیدی Octave؛ نوشتن .mat نمی‌کنیم (5.5)
//! delim.rs  متن محدودشده — readmatrix (کشف خودکار جداساز/سرصفحه) + writematrix
//! vfs.rs    فایل‌سازمان مجازی نشست — موتور روی اندروید مسیر نمی‌بیند؛
//!           میزبان (Dart) بایت‌ها را می‌نویسد/می‌خواند (الگوی register_m_source)
//! ```
//!
//! مرزها: این کرِیت فقط mc-core + flate2 (بک‌اند rust) می‌بیند؛ نه مفسر، نه FFI.
//! خطاها با [`IoError`] بازمی‌گردند و لایهٔ builtin آن‌ها را به `McError`
//! (کدهای mc-core) نگاشت می‌کند — اینجا کار خط‌/ستون نیست.

use mc_core::value::Value;

pub mod delim;
pub mod matv5;
pub mod mcws;
pub mod vfs;

/// خطای I/O — پیام دوزبانه مثل بقیهٔ پروژه؛ نگاشت به `McError` در لایهٔ بالا.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IoError {
    pub message_en: String,
    pub message_fa: String,
}

impl IoError {
    pub fn new(en: impl Into<String>, fa: impl Into<String>) -> Self {
        IoError {
            message_en: en.into(),
            message_fa: fa.into(),
        }
    }
}

impl std::fmt::Display for IoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message_en)
    }
}

/// نتیجهٔ خواندن یک متغیر از فایل — نام + مقدار.
pub type NamedValue = (String, Value);
