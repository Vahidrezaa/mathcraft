//! mc-num — موتور عددی MathCraft (سند ۰۴).
//!
//! اسکلت P0: این کرِیت در مایل‌استون P2 (با زیرمجموعه جبر خطی در P1) پر می‌شود.
//! نقشه ماژول‌ها طبق سند ۰۴ بخش ۱:
//!
//! ```text
//! linalg/    facade روی faer — LU/Cholesky/QR/SVD/Eigen + درخت dispatch بک‌اسلش (۷ شاخه، سند ۰۴ §3.2)
//! solvers/   fzero (Brent)، roots، integral (Gauss–Kronrod تطبیقی)، ode45/ode23 (Dormand–Prince)
//! interp/    linear، pchip (Fritsch–Carlson)، cubic spline
//! poly/      polyfit (QR — هیچ‌وقت normal equations)، roots (همراه یونیک، ماتریس همبند)
//! stats/     توصیفی NaN-aware، توزیع‌ها، corrcoef/cov، histogram binning
//! rng/       PCG64 + Ziggurat — تکرارپذیری با seed (سند ۰۴ بخش ۷)
//! ```
//!
//! قواعد غیرقابل‌مذاکره (سند ۰۴ بخش ۹):
//! 1. toleranceهای پیش‌فرض = متلب (RelTol=1e-3 برای ode، 1e-10 نسبی برای integral).
//! 2. هر الگوریتم قبل از بهینه‌شدن، مقابل مرجع تحلیلی/Octave تست می‌شود.
//! 3. هیچ panic در مسیر محصول؛ شکست = `McError` یا هشدار ساختاریافته.

#![cfg_attr(
    not(test),
    warn(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::todo,
        clippy::unimplemented
    )
)]
