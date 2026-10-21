//! لایه facade جبر خطی روی faer (سند ۰۴ بخش ۳).
//!
//! ساختار ماژول‌ها:
//! - `convert`: تبدیل `Matrix` (mc-core، ستونی) ↔ `faer::Mat` (ستونی با پدینگ احتمالی).
//! - `structure`: تشخیص ساختاری (مثلثی/سه‌قطری/مثلثیِ پرموته) — فقط با آزمون دقیق صفر.
//! - `solve`: حلگرهای دست‌نویس — مثلثی O(n²)، Thomas O(n)، LDLᵀ.
//! - `decomp`: تجزیه‌ها روی faer — lu/det/inv/qr/svd/eig/chol.
//! - `norm_func`: نُرم‌ها، cond، rank، rref.
//! - `dispatch`: درخت dispatch `\` و `/` (سند ۰۴ §3.2) با ثبت `DispatchStep` —
//!   «یک درخت، دو مصرف»: اجرا + افزونه آموزشی `steps(A\b)` (سند ۰۴ §3.2، سند ۰۳ §13).

pub mod convert;
pub mod decomp;
pub mod dispatch;
pub mod norm_func;
pub mod solve;
pub mod structure;

use mc_core::error::McError;

/// یک گام از درخت dispatch بک‌اسلش — برای اجرا و افزونه آموزشی `steps(A\b)` (سند ۰۴ §3.2).
///
/// - `branch`: نام کوتاه شاخه مثل `"LU"` یا `"TRIANGULAR"` (ثابت 'static).
/// - `detail`: توضیح دوزبانهٔ کوتاه با قالب «فارسی — English».
#[derive(Debug, Clone, PartialEq)]
pub struct DispatchStep {
    pub branch: &'static str,
    pub detail: String,
}

impl DispatchStep {
    /// ساخت گام dispatch با پیام دوزبانهٔ کوتاه.
    pub fn new(branch: &'static str, detail_fa: &str, detail_en: &str) -> Self {
        Self {
            branch,
            detail: format!("{detail_fa} — {detail_en}"),
        }
    }
}

/// خطای استاندارد موتور عددی: موقعیت `(0,0)` — لایه interp موقعیت واقعی AST را اضافه می‌کند
/// (قرارداد سند ۰۲ بخش ۹: خطا یک مقدار است، نه استثنای مهلک).
pub(crate) fn num_error(
    id: &str,
    message_en: impl Into<String>,
    message_fa: impl Into<String>,
) -> McError {
    McError::new(id, 0, 0, message_en, message_fa)
}

pub use decomp::{chol, det, eig, eig_complex, eig_sym, inv, lu, qr, qr_pivoted, svd};
pub use dispatch::{mldivide, mrdivide};
pub use norm_func::{cond, norm, rank, rref};
pub use solve::{thomas_solve, trisolve_lower, trisolve_upper};
pub use structure::{is_permuted_triangular, is_triangular, is_tridiagonal};

/// ضرب ماتریسی داخلی crate (dispatch و تست‌ها) — عمومی در سطح crate.
pub(crate) use norm_func::matmul;
