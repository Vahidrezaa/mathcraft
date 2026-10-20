//! تبدیل `Matrix` (mc-core) ↔ `faer::Mat` (سند ۰۴ §2).
//!
//! هر دو ستونی‌اند (سند ۰۴ §2: «چیدمان داده data[c*rows + r]»)؛ ولی layout داخلی
//! `faer::Mat` ممکن است بین ستون‌ها padding داشته باشد (سند faer: col stride نامشخص)،
//! پس تبدیل فقط از طریق API عمومی faer (ستون‌به‌ستون / عنصربه‌عنصر) انجام می‌شود —
//! هیچ فرضی بر هم‌خوانی بایت‌به‌بایت حافظه نداریم.

use faer::Mat;
use mc_core::matrix::Matrix;

/// ساخت `faer::Mat` از ماتریس MathCraft (کپی ستونی، O(mn)).
pub(crate) fn to_faer(a: &Matrix) -> Mat<f64> {
    let (rows, cols) = (a.rows(), a.cols());
    Mat::from_fn(rows, cols, |r, c| a.get(r, c))
}

/// ساخت ماتریس MathCraft از `faer::Mat` (کپی ستونی، O(mn)).
pub(crate) fn from_faer(m: &Mat<f64>) -> Matrix {
    let (rows, cols) = (m.nrows(), m.ncols());
    let mut data = Vec::with_capacity(rows * cols);
    for c in 0..cols {
        data.extend(m.col(c).iter().copied());
    }
    Matrix::from_columns(rows, cols, data)
}

/// ترانهاده ماتریس MathCraft — `(Aᵀ)ᵀ = A` (سمانتیک متلب `'`).
pub(crate) fn transpose(a: &Matrix) -> Matrix {
    let mut out = Matrix::full(a.cols(), a.rows(), 0.0);
    for c in 0..a.cols() {
        for r in 0..a.rows() {
            out.set(c, r, a.get(r, c));
        }
    }
    out
}

/// ماتریس همانی n×n.
pub(crate) fn identity(n: usize) -> Matrix {
    let mut out = Matrix::full(n, n, 0.0);
    for i in 0..n {
        out.set(i, i, 1.0);
    }
    out
}

/// درج ردیف `src_row` ماتریس `a` در ردیف `dst_row` خروجی (برای پرموتاسیون ردیفی dispatch).
pub(crate) fn permute_rows(a: &Matrix, perm: &[usize]) -> Matrix {
    let mut out = Matrix::full(a.rows(), a.cols(), 0.0);
    for (dst, src) in perm.iter().enumerate() {
        for c in 0..a.cols() {
            out.set(dst, c, a.get(*src, c));
        }
    }
    out
}

/// چک مربع‌بودن؛ در غیر این صورت خطای `SIZE_MISMATCH` (سند ۰۲ §9 — خطای اجرایی دوزبانه).
pub(crate) fn require_square(
    a: &Matrix,
    op_en: &str,
    op_fa: &str,
) -> Result<(), mc_core::error::McError> {
    if a.rows() == a.cols() {
        Ok(())
    } else {
        Err(crate::linalg::num_error(
            mc_core::error::ids::SIZE_MISMATCH,
            format!(
                "{op_en}: input must be square, got {r}\u{00d7}{c}.",
                r = a.rows(),
                c = a.cols()
            ),
            format!(
                "{op_fa}: ورودی باید مربع باشد، ولی {r}\u{00d7}{c} دریافت شد.",
                r = a.rows(),
                c = a.cols()
            ),
        ))
    }
}

/// تقارن دقیق (برابری `A == Aᵀ` عنصربه‌عنصر؛ ماتریس‌ها حقیقی‌اند — سند ۰۴ §3.2 شاخه Hermitian).
/// مثل متلب: NaN با هیچ‌چیز حتی خودش برابر نیست، پس NaN تقارن را نقض می‌کند.
pub(crate) fn is_exactly_symmetric(a: &Matrix) -> bool {
    a.rows() == a.cols()
        && (0..a.rows()).all(|i| (i + 1..a.cols()).all(|j| a.get(i, j) == a.get(j, i)))
}

/// جمع عنصری قدرمطلق‌ها برای نرمال‌سازی تست‌ها (فقط تست).
#[cfg(test)]
pub(crate) fn max_abs_diff(a: &Matrix, b: &Matrix) -> f64 {
    debug_assert_eq!((a.rows(), a.cols()), (b.rows(), b.cols()));
    let mut m = 0.0f64;
    for idx in 0..a.numel() {
        m = m.max((a.get_linear(idx) - b.get_linear(idx)).abs());
    }
    m
}
