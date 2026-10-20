//! حلگرهای دست‌نویس درخت dispatch (سند ۰۴ §3.2):
//! - مثلثی بالا/پایین: بازجایگذاری O(n²) با چند بردار سمت راست
//! - سه‌قطری: الگوریتم Thomas بدون pivoting، O(n)
//! - LDLᵀ: تجزیه متقارن نامعین با پیوت‌های ۱×۱ (شاخه Hermitian بدون chol)
//!
//! هیچ‌کدام panic نمی‌کنند؛ پیوت دقیقاً صفر → `SINGULAR_MATRIX` (سند ۰۲ §9: خطا = مقدار).

use crate::linalg::convert;
use crate::linalg::num_error;
use crate::linalg::structure::is_tridiagonal;
use mc_core::error::ids;
use mc_core::matrix::Matrix;

/// خطای داخلی LDLᵀ: پیوت صفر (dispatch به LU برمی‌گردد).
pub(crate) enum LdltFail {
    ZeroPivot { index: usize },
}

/// حل `A X = B` برای A بالا-مثلثی مربع — بازجایگذاری پس‌رو، O(n²·k).
pub fn trisolve_upper(a: &Matrix, b: &Matrix) -> Result<Matrix, mc_core::error::McError> {
    convert::require_square(a, "trisolve_upper", "trisolve_upper")?;
    check_rhs(a, b, "trisolve_upper")?;
    let n = a.rows();
    let mut x = b.clone();
    for c in 0..x.cols() {
        for i in (0..n).rev() {
            let mut acc = x.get(i, c);
            for j in (i + 1)..n {
                acc -= a.get(i, j) * x.get(j, c);
            }
            let piv = a.get(i, i);
            if piv == 0.0 {
                return Err(singular(
                    "zero pivot in upper-triangular solve",
                    "پیوت صفر در حل مثلثی بالا",
                ));
            }
            x.set(i, c, acc / piv);
        }
    }
    Ok(x)
}

/// حل `A X = B` برای A پایین-مثلثی مربع — بازجایگذاری پیش‌رو، O(n²·k).
pub fn trisolve_lower(a: &Matrix, b: &Matrix) -> Result<Matrix, mc_core::error::McError> {
    convert::require_square(a, "trisolve_lower", "trisolve_lower")?;
    check_rhs(a, b, "trisolve_lower")?;
    let n = a.rows();
    let mut x = b.clone();
    for c in 0..x.cols() {
        for i in 0..n {
            let mut acc = x.get(i, c);
            for j in 0..i {
                acc -= a.get(i, j) * x.get(j, c);
            }
            let piv = a.get(i, i);
            if piv == 0.0 {
                return Err(singular(
                    "zero pivot in lower-triangular solve",
                    "پیوت صفر در حل مثلثی پایین",
                ));
            }
            x.set(i, c, acc / piv);
        }
    }
    Ok(x)
}

/// حل `A X = B` برای A سه‌قطری مربع با الگوریتم Thomas — O(n·k) بدون pivoting (سند ۰۴ §3.2).
///
/// بدون pivoting است؛ اگر در حذف پیش‌رو پیوت دقیقاً صفر شود خطای `SINGULAR_MATRIX`
/// می‌دهد (dispatch این را می‌گیرد و به LU برمی‌گردد — ثبت صادقانه در `DispatchStep`).
pub fn thomas_solve(a: &Matrix, b: &Matrix) -> Result<Matrix, mc_core::error::McError> {
    convert::require_square(a, "thomas_solve", "thomas_solve")?;
    check_rhs(a, b, "thomas_solve")?;
    if !is_tridiagonal(a) {
        return Err(num_error(
            ids::UNSUPPORTED,
            "thomas_solve: input must be square tridiagonal (only three diagonals).",
            "thomas_solve: ورودی باید مربعِ سه‌قطری باشد (فقط سه قطر اصلی).",
        ));
    }
    let n = a.rows();
    if n == 0 {
        // 0×0: چیزی برای حل نیست — خروجی همان B تهی (سمانتیک متلب روی ابعاد صفر).
        return Ok(b.clone());
    }
    let singular_thomas = || singular("zero pivot in Thomas elimination", "پیوت صفر در حذف Thomas");

    // ضریب‌های حذف پیش‌رو مستقل از ستون‌های B هستند؛ یک‌بار محاسبه می‌شوند:
    // m_i = b_i - a_i * c_{i-1} / m_{i-1}   (b: قطر، a: زیرقطر، c: بالاقطر — از خود A)
    let mut m = vec![0.0f64; n];
    for i in 0..n {
        let sub = if i > 0 { a.get(i, i - 1) } else { 0.0 };
        let c_prev = if i > 0 { a.get(i - 1, i) } else { 0.0 };
        let mi = a.get(i, i) - if i > 0 { sub * c_prev / m[i - 1] } else { 0.0 };
        if mi == 0.0 {
            return Err(singular_thomas());
        }
        m[i] = mi;
    }

    let mut x = b.clone();
    for c in 0..x.cols() {
        // پاس پیش‌رو: X(i) ← r'_i = X(i) - (a_i/m_{i-1}) * r'_{i-1}
        for i in 1..n {
            let factor = a.get(i, i - 1) / m[i - 1];
            let v = x.get(i, c) - factor * x.get(i - 1, c);
            x.set(i, c, v);
        }
        // بازجایگذاری پس‌رو روی دو-قطریِ بالا: x_{n-1} = r'_{n-1}/m_{n-1} ؛ x_i = (r'_i - c_i x_{i+1})/m_i
        let last = n - 1;
        x.set(last, c, x.get(last, c) / m[last]);
        for i in (0..last).rev() {
            let v = (x.get(i, c) - a.get(i, i + 1) * x.get(i + 1, c)) / m[i];
            x.set(i, c, v);
        }
    }
    Ok(x)
}

/// حل `A X = B` برای A متقارن با تجزیه A = L D Lᵀ (پیوت‌های ۱×۱، بدون pivoting) — سند ۰۴ §3.2.
///
/// شکست فقط با پیوت دقیقاً صفر رخ می‌دهد (`LdltFail::ZeroPivot`) تا dispatch
/// با برچسب صادقانه «LDLᵀ(fallback LU)» به مسیر LU برود.
pub(crate) fn ldlt_solve(a: &Matrix, b: &Matrix) -> Result<Matrix, LdltFail> {
    let n = a.rows();
    // L: مثلثی پایین با قطر واحد؛ d: بردار D
    let mut l = convert::identity(n);
    let mut d = vec![0.0f64; n];
    for j in 0..n {
        // d_j = a_jj - Σ_{k<j} l_jk² d_k
        let mut acc = a.get(j, j);
        for (k, dk) in d[..j].iter().enumerate() {
            acc -= l.get(j, k) * l.get(j, k) * dk;
        }
        if acc == 0.0 {
            return Err(LdltFail::ZeroPivot { index: j });
        }
        d[j] = acc;
        // l_ij = (a_ij - Σ_{k<j} l_ik d_k l_jk) / d_j
        for i in (j + 1)..n {
            let mut acc2 = a.get(i, j);
            for (k, dk) in d[..j].iter().enumerate() {
                acc2 -= l.get(i, k) * dk * l.get(j, k);
            }
            l.set(i, j, acc2 / d[j]);
        }
    }
    // حل: L Y = B (پیش‌رو)، D Z = Y، Lᵀ X = Z (پس‌رو)
    let mut y = b.clone();
    for c in 0..y.cols() {
        for i in 0..n {
            let mut acc = y.get(i, c);
            for k in 0..i {
                acc -= l.get(i, k) * y.get(k, c);
            }
            y.set(i, c, acc);
        }
    }
    for (i, di) in d.iter().enumerate() {
        for c in 0..y.cols() {
            y.set(i, c, y.get(i, c) / di);
        }
    }
    for c in 0..y.cols() {
        for i in (0..n).rev() {
            let mut acc = y.get(i, c);
            for k in (i + 1)..n {
                acc -= l.get(k, i) * y.get(k, c);
            }
            y.set(i, c, acc);
        }
    }
    Ok(y)
}

/// نسخه عمومی `ldlt_solve` برای شاخه LDLᵀ dispatch — شکست پیوت صفر را به
/// `McError` استاندارد نگاشت نمی‌کند؛ dispatch خودش سقوط به LU را مدیریت می‌کند.
pub(crate) fn ldlt_solve_pub(a: &Matrix, b: &Matrix) -> Result<Matrix, mc_core::error::McError> {
    ldlt_solve(a, b).map_err(|LdltFail::ZeroPivot { index }| {
        num_error(
            mc_core::error::ids::SINGULAR_MATRIX,
            format!("LDLᵀ factorization hit a zero pivot at index {index}."),
            format!("تجزیه LDLᵀ در اندیس {index} به پیوت صفر برخورد."),
        )
    })
}

/// چک هم‌خوانی ردیف‌های B با A (سند ۰۲ §9).
fn check_rhs(a: &Matrix, b: &Matrix, op: &str) -> Result<(), mc_core::error::McError> {
    if b.rows() == a.rows() {
        Ok(())
    } else {
        Err(num_error(
            ids::SIZE_MISMATCH,
            format!(
                "{op}: matrix dimensions must agree: A is {ar}\u{00d7}{ac}, B is {br}\u{00d7}{bc}.",
                ar = a.rows(),
                ac = a.cols(),
                br = b.rows(),
                bc = b.cols()
            ),
            format!(
                "{op}: ابعاد ماتریس‌ها نمی‌خواند: A برابر {ar}\u{00d7}{ac} است ولی B برابر {br}\u{00d7}{bc}.",
                ar = a.rows(),
                ac = a.cols(),
                br = b.rows(),
                bc = b.cols()
            ),
        ))
    }
}

/// خطای تکینگی با پیشنهاد آموزشی (سند ۰۴ §9 بند ۵: پیشنهاد rcond/cond).
fn singular(en: &str, fa: &str) -> mc_core::error::McError {
    num_error(ids::SINGULAR_MATRIX, format!("{en}."), format!("{fa}."))
        .with_suggestion("check rcond/cond")
}
