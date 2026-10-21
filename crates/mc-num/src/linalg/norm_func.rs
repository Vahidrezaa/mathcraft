//! نُرم‌ها، شرط‌پذیری، رتبه و RREF (سند ۰۴ §3.1 جدول کاربردها).
//!
//! سمانتیک متلب:
//! - `norm(v)` روی بردار = نُرم اقلیدسی ۲؛ `norm(A)` روی ماتریس = بزرگ‌ترین مقدار تکین.
//! - `norm(x,p)` با `p ∈ {1, 2, inf, fro}` (پذیرش `"i"`/`"inf"`/`"Inf"`).
//! - `cond(A)` = σmax/σmin (نُرم ۲) — متلب پیش‌فرض ۲ است.
//! - `rank(A)` با آستانه روی σها (faer SVD) یا R قطری‌شده QR پیموت‌دار.
//! - `rref` با Gauss–Jordan و آستانه نسبی (رفتار `rref` متلب: tol پیش‌فرض
//!   `max(size(A))*eps*norm(A,inf)`).

use crate::linalg::convert;
use crate::linalg::num_error;
use mc_core::error::ids;
use mc_core::matrix::Matrix;

/// نُرم متلب `norm(x, p)` — `p=None` یعنی نُرم ۲ (بردار: اقلیدسی، ماتریس: σmax).
pub fn norm(a: &Matrix, p: Option<&str>) -> Result<f64, mc_core::error::McError> {
    let (m, n) = (a.rows(), a.cols());
    let is_vector = m == 1 || n == 1;

    // کلید نُرم — None و "2" یکسان‌اند.
    let key: &str = match p {
        None => "2",
        Some(s) => s,
    };

    match key {
        "1" => {
            if is_vector {
                // جمع قدرمطلق عناصر (بردار ستونی/سطری).
                Ok((0..a.numel()).map(|i| a.get_linear(i).abs()).sum())
            } else {
                // نُرم ماتریسی ۱ = بیشینهٔ جمع ستونی.
                let mut best = 0.0f64;
                for c in 0..n {
                    let s: f64 = (0..m).map(|r| a.get(r, c).abs()).sum();
                    best = best.max(s);
                }
                Ok(best)
            }
        }
        "2" => {
            if is_vector {
                // اقلیدسی — با sqrt جمع مربع‌ها (بردارهای بلند: overflow-ایمن با scale).
                let mut scale = 0.0f64;
                for i in 0..a.numel() {
                    scale = scale.max(a.get_linear(i).abs());
                }
                if scale == 0.0 || !scale.is_finite() {
                    return Ok(scale); // صفر یا Inf/NaN (رفتار IEEE، سند ۰۳ §11)
                }
                let sumsq: f64 = (0..a.numel())
                    .map(|i| {
                        let t = a.get_linear(i) / scale;
                        t * t
                    })
                    .sum();
                Ok(scale * sumsq.sqrt())
            } else {
                // نُرم ۲ ماتریس = بزرگ‌ترین مقدار تکین.
                let (_, s, _) = crate::linalg::svd(a)?;
                Ok(s.first().copied().unwrap_or(0.0))
            }
        }
        "inf" | "i" | "Inf" | "INF" => {
            if is_vector {
                // نُرم بی‌نهایت بردار = بیشینهٔ قدرمطلق.
                Ok((0..a.numel())
                    .map(|i| a.get_linear(i).abs())
                    .fold(0.0f64, f64::max))
            } else {
                // نُرم بی‌نهایت ماتریس = بیشینهٔ جمع ردیفی.
                let mut best = 0.0f64;
                for r in 0..m {
                    let s: f64 = (0..n).map(|c| a.get(r, c).abs()).sum();
                    best = best.max(s);
                }
                Ok(best)
            }
        }
        "fro" => {
            // نُرم فروبنیوس = اقلیدسیِ همه عناصر (overflow-ایمن با scale).
            let mut scale = 0.0f64;
            for idx in 0..a.numel() {
                scale = scale.max(a.get_linear(idx).abs());
            }
            if scale == 0.0 || !scale.is_finite() {
                return Ok(scale);
            }
            let sumsq: f64 = (0..a.numel())
                .map(|idx| {
                    let t = a.get_linear(idx) / scale;
                    t * t
                })
                .sum();
            Ok(scale * sumsq.sqrt())
        }
        other => Err(num_error(
            ids::UNSUPPORTED,
            format!("norm: unsupported norm type '{other}'. Use 1, 2, inf or fro."),
            format!("norm: نوع نُرم «{other}» پشتیبانی نمی‌شود. از 1، 2، inf یا fro استفاده کنید."),
        )),
    }
}

/// شرط‌پذیری نُرم-۲: σmax/σmin (سند ۰۴ §3.1: SVD کاربرد `cond`).
///
/// ماتریس صفر → Inf؛ اگر σmin دقیقاً صفر باشد → Inf (ماتریس تکین).
pub fn cond(a: &Matrix) -> Result<f64, mc_core::error::McError> {
    if a.is_empty() {
        return Ok(0.0); // متلب: cond([]) = 0
    }
    let (_, s, _) = crate::linalg::svd(a)?;
    let smax = s.first().copied().unwrap_or(0.0);
    let smin = s.last().copied().unwrap_or(0.0);
    if smin == 0.0 {
        Ok(f64::INFINITY)
    } else {
        Ok(smax / smin)
    }
}

/// آستانه پیش‌فرض رتبه متلب: `max(m,n) * eps * σmax`.
fn default_rank_tol(smax: f64, m: usize, n: usize) -> f64 {
    (m.max(n) as f64) * f64::EPSILON * smax
}

/// رتبه ماتریس با شمارش σهای بزرگ‌تر از آستانه (سند ۰۴ §3.1: SVD کاربرد `rank`).
///
/// `tol=None` → آستانه متلب: `max(m,n)*eps*σmax`.
pub fn rank(a: &Matrix, tol: Option<f64>) -> usize {
    if a.is_empty() {
        return 0;
    }
    let (_, s, _) = match crate::linalg::svd(a) {
        Ok(v) => v,
        Err(_) => return 0, // مسیر قابل‌رسد نیست (SVD هر ماتریسی را تجزیه می‌کند)
    };
    let smax = s.first().copied().unwrap_or(0.0);
    let t = tol.unwrap_or_else(|| default_rank_tol(smax, a.rows(), a.cols()));
    s.iter().filter(|&&v| v > t).count()
}

/// RREF با Gauss–Jordan (رفتار متلب `rref`؛ سند ۰۴ §3.1/جدول P1 سند ۰۷).
///
/// - آستانه پیش‌فرض: `max(m,n)*eps*norm(A,inf)` — همان رفرنس متلب.
/// - خروجی: ماتریس RREF + اندیس ستون‌های پیموت.
/// - هیچ panic مسیر محصول؛ برای ورودی تهی، خروجی تهی.
pub fn rref(a: &Matrix) -> (Matrix, Vec<usize>) {
    let (m, n) = (a.rows(), a.cols());
    let mut work = a.clone();
    let mut pivots: Vec<usize> = Vec::new();
    if m == 0 || n == 0 {
        return (work, pivots);
    }

    // آستانه نسبی متلب.
    let mut inf_norm = 0.0f64;
    for r in 0..m {
        let s: f64 = (0..n).map(|c| a.get(r, c).abs()).sum();
        inf_norm = inf_norm.max(s);
    }
    let tol = (m.max(n) as f64) * f64::EPSILON * inf_norm;

    let mut row = 0usize;
    for col in 0..n {
        if row >= m {
            break;
        }
        // یافتن بزرگ‌ترین قدرمطلق در ستون `col` از ردیف `row` به پایین (پیموت جزئی).
        let mut best_r = row;
        let mut best_v = work.get(row, col).abs();
        for r in (row + 1)..m {
            let v = work.get(r, col).abs();
            if v > best_v {
                best_v = v;
                best_r = r;
            }
        }
        if best_v <= tol {
            continue; // ستون مستقل نیست — پیموت نداریم
        }
        if best_r != row {
            // جابه‌جایی کامل دو ردیف (رفتار متلب — ردیف‌های بالای `row` هم جابه‌جا می‌شوند).
            for c in 0..n {
                let t = work.get(row, c);
                work.set(row, c, work.get(best_r, c));
                work.set(best_r, c, t);
            }
        }
        // نرمال‌سازی ردیف پیموت.
        let pv = work.get(row, col);
        for c in 0..n {
            work.set(row, c, work.get(row, c) / pv);
        }
        // حذف ستون از همه ردیف‌های دیگر (Gauss–Jordan — نه فقط پایین).
        for r in 0..m {
            if r == row {
                continue;
            }
            let factor = work.get(r, col);
            if factor == 0.0 {
                continue;
            }
            for c in 0..n {
                work.set(r, c, work.get(r, c) - factor * work.get(row, c));
            }
        }
        pivots.push(col);
        row += 1;
    }

    (work, pivots)
}

/// ضرب ماتریسی عمومی `A*B` — فقط مصرف داخلی mc-num و تست‌ها.
///
/// (ضرب موتور اصلی در mc-interpDispatchِ سند ۰۲ §5 انجام می‌شود؛ این نسخه ساده برای
/// لایه عددی کافی است — dispatch داخلی خودش از حلگرها استفاده می‌کند.)
pub(crate) fn matmul(a: &Matrix, b: &Matrix) -> Result<Matrix, mc_core::error::McError> {
    if a.cols() != b.rows() {
        return Err(num_error(
            ids::SIZE_MISMATCH,
            format!(
                "matrix multiply: inner dimensions must agree, got {}×{} and {}×{}.",
                a.rows(),
                a.cols(),
                b.rows(),
                b.cols()
            ),
            format!(
                "ضرب ماتریسی: ابعاد داخلی باید هم‌خوان باشند؛ دریافت شد {}×{} و {}×{}.",
                a.rows(),
                a.cols(),
                b.rows(),
                b.cols()
            ),
        ));
    }
    let af = convert::to_faer(a);
    let bf = convert::to_faer(b);
    Ok(convert::from_faer(&(af * bf)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mat(rows: usize, cols: usize, data: &[f64]) -> Matrix {
        Matrix::from_columns(rows, cols, data.to_vec())
    }

    #[test]
    fn norm_vector_2_is_euclidean() {
        let v = mat(3, 1, &[3.0, 4.0, 0.0]);
        assert!((norm(&v, None).unwrap() - 5.0).abs() < 1e-12);
    }

    #[test]
    fn norm_vector_1_and_inf() {
        let v = mat(3, 1, &[3.0, -4.0, 1.0]);
        assert!((norm(&v, Some("1")).unwrap() - 8.0).abs() < 1e-12);
        assert!((norm(&v, Some("inf")).unwrap() - 4.0).abs() < 1e-12);
    }

    #[test]
    fn norm_matrix_1_inf_fro() {
        // A = [1 -2; 3 4]
        let a = mat(2, 2, &[1.0, 3.0, -2.0, 4.0]);
        assert!((norm(&a, Some("1")).unwrap() - 6.0).abs() < 1e-12); // max col-sum: |3|+|4|=7? → [1,3]=4, [-2,4]=6 → 6
        assert!((norm(&a, Some("inf")).unwrap() - 7.0).abs() < 1e-12); // max row-sum: 1+2=3? rows: |1|+|-2|=3, |3|+|4|=7 → 7
        let fro = (1.0f64 + 4.0 + 9.0 + 16.0).sqrt();
        assert!((norm(&a, Some("fro")).unwrap() - fro).abs() < 1e-12);
    }

    #[test]
    fn norm_matrix_2_is_largest_singular_value() {
        // A = diag(3, 4) → norm2 = 4
        let a = mat(2, 2, &[3.0, 0.0, 0.0, 4.0]);
        assert!((norm(&a, None).unwrap() - 4.0).abs() < 1e-10);
    }

    #[test]
    fn cond_of_orthogonal_is_one_and_singular_is_inf() {
        let i = convert::identity(3);
        assert!((cond(&i).unwrap() - 1.0).abs() < 1e-10);
        let s = mat(2, 2, &[1.0, 2.0, 2.0, 4.0]); // رتبه ۱
        assert_eq!(cond(&s).unwrap(), f64::INFINITY);
    }

    #[test]
    fn rank_counts_singular_values_above_tol() {
        let s = mat(2, 2, &[1.0, 2.0, 2.0, 4.0]); // رتبه ۱
        assert_eq!(rank(&s, None), 1);
        assert_eq!(rank(&s, Some(0.0)), 1); // σmin دقیقاً 0
        assert_eq!(rank(&convert::identity(4), None), 4);
    }

    #[test]
    fn rref_reduces_invertible_to_identity() {
        let a = mat(2, 2, &[1.0, 3.0, 2.0, 4.0]);
        let (r, piv) = rref(&a);
        let i = convert::identity(2);
        assert!(convert::max_abs_diff(&r, &i) < 1e-12);
        assert_eq!(piv, vec![0, 1]);
    }

    #[test]
    fn rref_free_variable_like_matlab() {
        // A = [1 2 3; 2 4 6] → rref = [1 2 3; 0 0 0]، پیموت [0]
        // (داده column-major: [1,2, 2,4, 3,6])
        let a = mat(2, 3, &[1.0, 2.0, 2.0, 4.0, 3.0, 6.0]);
        let (r, piv) = rref(&a);
        assert_eq!(piv, vec![0]);
        assert!(r.get(0, 0) == 1.0 && r.get(0, 1) == 2.0 && r.get(0, 2) == 3.0);
        assert!(r.get(1, 0).abs() < 1e-12 && r.get(1, 1).abs() < 1e-12);
    }

    #[test]
    fn rref_zero_matrix_stays_zero() {
        let z = Matrix::full(2, 2, 0.0);
        let (r, piv) = rref(&z);
        assert_eq!(piv, Vec::<usize>::new());
        assert!(convert::max_abs_diff(&r, &z) == 0.0);
    }

    #[test]
    fn norm_unsupported_type_is_error() {
        let v = mat(2, 1, &[1.0, 2.0]);
        assert_eq!(norm(&v, Some("9")).unwrap_err().id, ids::UNSUPPORTED);
    }

    #[test]
    fn matmul_inner_dim_mismatch_is_size_error() {
        let a = mat(2, 3, &[1.0; 6]);
        let b = mat(2, 2, &[1.0; 4]);
        assert_eq!(matmul(&a, &b).unwrap_err().id, ids::SIZE_MISMATCH);
    }
}
