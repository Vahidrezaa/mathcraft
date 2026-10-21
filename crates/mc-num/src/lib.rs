//! mc-num — موتور عددی MathCraft (سند ۰۴).
//!
//! P1: هسته جبر خطی (`linalg`) — facade روی faer + درخت dispatch بک‌اسلش (سند ۰۴ §3.2).
//! باقی ماژول‌ها (solvers/interp/poly/stats/rng) در P2 پر می‌شوند. نقشه ماژول‌ها طبق سند ۰۴ بخش ۱:
//!
//! ```text
//! linalg/    facade روی faer — LU/Cholesky/QR/SVD/Eigen + درخت dispatch بک‌اسلش (۷ شاخه، سند ۰۴ §3.2)
//! solvers/   fzero (Brent)، roots، integral (Gauss–Kronrod تطبیقی)، ode45/ode23 (Dormand–Prince)  ← P2
//! interp/    linear، pchip (Fritsch–Carlson)، cubic spline                                        ← P2
//! poly/      polyfit (QR — هیچ‌وقت normal equations)، roots (ماتریس همبند)                         ← P2
//! stats/     توصیفی NaN-aware، توزیع‌ها، corrcoef/cov، histogram binning                           ← P2
//! rng/       PCG64 + Ziggurat — تکرارپذیری با seed (سند ۰۴ بخش ۷)                                 ← P2
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

pub mod interp;
pub mod linalg;
pub mod poly;
pub mod rng;
pub mod solvers;
pub mod stats;

/// تشخیص تقارن دقیق — عمومی برای لایه‌های بالا (dispatch متقارن سند ۰۴ §3.2).
pub fn is_exactly_symmetric(a: &mc_core::matrix::Matrix) -> bool {
    linalg::convert::is_exactly_symmetric(a)
}

pub use linalg::{
    chol, cond, det, eig, eig_complex, eig_sym, inv, is_permuted_triangular, is_triangular,
    is_tridiagonal, lu, mldivide, mrdivide, norm, qr, qr_pivoted, rank, rref, svd, thomas_solve,
    trisolve_lower, trisolve_upper, DispatchStep,
};
