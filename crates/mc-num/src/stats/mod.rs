//! آمار — median/std/var/cov/corrcoef (سند ۰۴ بخش ۷).
//!
//! قراردادهای متلب:
//! - `std/var`: نرمال‌سازی n−1 پیش‌فرض؛ پرچم w=1 → n.
//! - `mean/sum` و این‌ها NaN را منتشر می‌کنند (رفتار پیش‌فرض متلب 'includenan')؛
//!   پرچم 'omitnan' (سند ۰۴ §7) پشتیبانی می‌شود.
//! - `cov`: ماتریس m×n → n×n (مشاهدات = ردیف‌ها)؛ دو بردار → 2×2.
//! - `corrcoef`: ماتریس همبستگی پیرسون؛ دو بردار → 2×2.

use mc_core::{Matrix, McError};

use crate::linalg::num_error;

/// اعتبارسنجی بردار بودن + استخراج تخت.
fn flat_vector(name: &str, m: &Matrix) -> Result<Vec<f64>, McError> {
    if m.rows() != 1 && m.cols() != 1 {
        return Err(num_error(
            mc_core::error::ids::SIZE_MISMATCH,
            format!("{name}: input must be a vector."),
            format!("{name}: ورودی باید بردار باشد."),
        ));
    }
    Ok((0..m.numel()).map(|i| m.get_linear(i)).collect())
}

/// میانگین با پارامتر omit — NaN پیش‌فرض منتشر می‌شود.
pub fn mean_flagged(vals: &[f64], omit_nan: bool) -> f64 {
    if omit_nan {
        let mut s = 0.0;
        let mut n = 0.0;
        for &v in vals {
            if !v.is_nan() {
                s += v;
                n += 1.0;
            }
        }
        if n == 0.0 {
            return f64::NAN;
        }
        s / n
    } else {
        vals.iter().sum::<f64>() / vals.len() as f64
    }
}

/// `median(v, omit)` — میانه؛ با NaN (پیش‌فرض) → NaN.
pub fn median_flagged(vals: &[f64], omit_nan: bool) -> f64 {
    if vals.iter().any(|v| v.is_nan()) && !omit_nan {
        return f64::NAN;
    }
    let mut s: Vec<f64> = vals.iter().copied().filter(|v| !v.is_nan()).collect();
    if s.is_empty() {
        return f64::NAN;
    }
    s.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let n = s.len();
    if n % 2 == 1 {
        s[n / 2]
    } else {
        0.5 * (s[n / 2 - 1] + s[n / 2])
    }
}

/// واریانس — w=0 (پیش‌فرض) → n−1؛ w=1 → n.
pub fn var_flagged(vals: &[f64], w_norm: bool, omit_nan: bool) -> f64 {
    let mu = mean_flagged(vals, omit_nan);
    if mu.is_nan() {
        return f64::NAN;
    }
    let data: Vec<f64> = if omit_nan {
        vals.iter().copied().filter(|v| !v.is_nan()).collect()
    } else {
        vals.to_vec()
    };
    let n = data.len();
    if (w_norm && n == 0) || (!w_norm && n < 2) {
        return f64::NAN;
    }
    let acc: f64 = data.iter().map(|v| (v - mu).powi(2)).sum();
    let denom = if w_norm { n } else { n - 1 };
    acc / denom as f64
}

/// انحراف معیار = sqrt(var).
pub fn std_flagged(vals: &[f64], w_norm: bool, omit_nan: bool) -> f64 {
    var_flagged(vals, w_norm, omit_nan).sqrt()
}

/// ماتریس کوواریانس — ستون‌ها متغیر، ردیف‌ها مشاهدات (n−1 نرمال).
pub fn cov(a: &Matrix) -> Result<Matrix, McError> {
    let (m, n) = (a.rows(), a.cols());
    if n == 0 || m == 0 {
        return Ok(Matrix::full(n, n, f64::NAN));
    }
    // میانگین هر ستون
    let mut mu = vec![0.0f64; n];
    for c in 0..n {
        let mut s = 0.0;
        for r in 0..m {
            s += a.get(r, c);
        }
        mu[c] = s / m as f64;
    }
    // ماتریس متمرکز X = A − μ  (m×n)
    let mut xc = Matrix::full(m, n, 0.0);
    for c in 0..n {
        for r in 0..m {
            xc.set(r, c, a.get(r, c) - mu[c]);
        }
    }
    // C = Xᵀ·X / (m−1) — ضرب دستی کوچک (n×m ضرب m×n)
    let denom = (m as f64 - 1.0).max(1.0);
    let mut out = Matrix::full(n, n, 0.0);
    for i in 0..n {
        for j in i..n {
            let mut acc = 0.0;
            for r in 0..m {
                acc += xc.get(r, i) * xc.get(r, j);
            }
            let v = acc / denom;
            out.set(i, j, v);
            out.set(j, i, v);
        }
    }
    Ok(out)
}

/// cov دو بردار — 2×2 متلب.
pub fn cov2(x: &Matrix, y: &Matrix) -> Result<Matrix, McError> {
    let xs = flat_vector("cov", x)?;
    let ys = flat_vector("cov", y)?;
    if xs.len() != ys.len() {
        return Err(num_error(
            mc_core::error::ids::SIZE_MISMATCH,
            format!("cov: vectors must have the same length ({} vs {}).", xs.len(), ys.len()),
            format!("cov: بردارها باید هم‌طول باشند ({} در برابر {}).", xs.len(), ys.len()),
        ));
    }
    let m = Matrix::from_columns(xs.len(), 2, {
        let mut d = Vec::with_capacity(xs.len() * 2);
        d.extend_from_slice(&xs);
        d.extend_from_slice(&ys);
        d
    });
    cov(&m)
}

/// ماتریس همبستگی پیرسون — corr(X) n×n (ستون‌ها = متغیر).
pub fn corrcoef(a: &Matrix) -> Result<Matrix, McError> {
    let c = cov(a)?;
    let n = c.rows();
    let mut r = Matrix::full(n, n, f64::NAN);
    for i in 0..n {
        for j in i..n {
            let denom = (c.get(i, i) * c.get(j, j)).sqrt();
            let v = if denom == 0.0 {
                if i == j { 1.0 } else { f64::NAN }
            } else {
                c.get(i, j) / denom
            };
            r.set(i, j, v);
            r.set(j, i, v);
        }
    }
    Ok(r)
}

/// corrcoef دو بردار — 2×2.
pub fn corrcoef2(x: &Matrix, y: &Matrix) -> Result<Matrix, McError> {
    let xs = flat_vector("corrcoef", x)?;
    let ys = flat_vector("corrcoef", y)?;
    if xs.len() != ys.len() {
        return Err(num_error(
            mc_core::error::ids::SIZE_MISMATCH,
            format!(
                "corrcoef: vectors must have the same length ({} vs {}).",
                xs.len(),
                ys.len()
            ),
            format!(
                "corrcoef: بردارها باید هم‌طول باشند ({} در برابر {}).",
                xs.len(),
                ys.len()
            ),
        ));
    }
    let m = Matrix::from_columns(xs.len(), 2, {
        let mut d = Vec::with_capacity(xs.len() * 2);
        d.extend_from_slice(&xs);
        d.extend_from_slice(&ys);
        d
    });
    corrcoef(&m)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn col(v: &[f64]) -> Matrix {
        Matrix::from_columns(v.len(), 1, v.to_vec())
    }

    fn row(v: &[f64]) -> Matrix {
        Matrix::from_columns(1, v.len(), v.to_vec())
    }

    #[test]
    fn median_odd_even_and_nan() {
        assert_eq!(median_flagged(&[3.0, 1.0, 2.0], false), 2.0);
        assert_eq!(median_flagged(&[4.0, 1.0, 3.0, 2.0], false), 2.5);
        assert!(median_flagged(&[1.0, f64::NAN, 2.0], false).is_nan());
        assert_eq!(median_flagged(&[1.0, f64::NAN, 2.0], true), 1.5);
    }

    #[test]
    fn var_std_n_minus_one_vs_n() {
        let v = [1.0, 2.0, 3.0, 4.0];
        // var نمونه (n−1): 5/3
        assert!((var_flagged(&v, false, false) - 5.0 / 3.0).abs() < 1e-15);
        // var جامعه (n): 1.25
        assert!((var_flagged(&v, true, false) - 1.25).abs() < 1e-15);
        assert!((std_flagged(&v, false, false) - (5.0f64 / 3.0).sqrt()).abs() < 1e-15);
        // NaN منتشر می‌شود
        assert!(var_flagged(&[1.0, f64::NAN, 3.0], false, false).is_nan());
        // omitnan
        assert!((var_flagged(&[1.0, f64::NAN, 3.0], false, true) - 2.0).abs() < 1e-15);
    }

    #[test]
    fn cov_matrix_two_vars() {
        // X = [[1, 2], [2, 1], [3, 4]] — کوواریانس مرجع دست‌محاسبه
        let x = Matrix::from_columns(3, 2, vec![1.0, 2.0, 3.0, 2.0, 1.0, 4.0]);
        let c = cov(&x).unwrap_or_else(|e| panic!("{e:?}"));
        // var(x₁) = 1 → C(0,0) = 1
        assert!((c.get(0, 0) - 1.0).abs() < 1e-14);
        // var(x₂): (2,1,4) → mean=7/3؛ انحراف‌ها −1/3،−4/3،5/3 → (1+16+25)/9/2 = 7/3
        assert!((c.get(1, 1) - 7.0 / 3.0).abs() < 1e-14);
        // cov(x₁,x₂) = ((−1)(−1/3)+(0)(−4/3)+(1)(5/3))/2 = (1/3 + 5/3)/2 = 1
        assert!((c.get(0, 1) - 1.0).abs() < 1e-14);
        assert!(c.get(1, 0) == c.get(0, 1));
    }

    #[test]
    fn corrcoef_perfect_and_uncorrelated() {
        let x = row(&[1.0, 2.0, 3.0, 4.0]);
        let y = row(&[2.0, 4.0, 6.0, 8.0]); // y = 2x → r = 1
        let r = corrcoef2(&x, &y).unwrap_or_else(|e| panic!("{e:?}"));
        assert!((r.get(0, 1) - 1.0).abs() < 1e-12);
        assert!((r.get(0, 0) - 1.0).abs() < 1e-12);
        // متعامد دقیق (کوواریانس صفر) → r = 0
        let u = row(&[-3.0, -1.0, 1.0, 3.0]);
        let v = row(&[1.0, -1.0, -1.0, 1.0]);
        let r2 = corrcoef2(&u, &v).unwrap();
        assert!(r2.get(0, 1).abs() < 1e-12);
        // بردار واریانس-صفر → قطر 1، خارج قطر NaN (رفتار متلب)
        let c0 = row(&[2.0, 2.0, 2.0, 2.0]);
        let r3 = corrcoef2(&c0, &v).unwrap();
        assert!((r3.get(0, 0) - 1.0).abs() < 1e-12);
        assert!(r3.get(0, 1).is_nan());
    }
}
