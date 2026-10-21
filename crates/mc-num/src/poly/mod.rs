//! چندجمله‌ای‌ها — polyfit/polyval/conv/deconv/roots (سند ۰۴ بخش ۶ + §5.2).
//!
//! قواعد سند ۰۴:
//! - `polyfit` با **Householder QR** (از طریق `linalg::qr`) — هرگز normal equations
//!   (AᵀA شرط‌بندی بد می‌سازد). رتبه‌ناقص → خطای صادقانه.
//! - `polyval` با Horner — پایدار.
//! - `roots` از طریق مقادیر ویژه ماتریس همبند (روش استاندارد متلب) — خروجی مختلط
//!   n×1 (ستون‌بردار، رفتار متلب).
//!
//! قرارداد ضرایب: **نزولی** مثل متلب — `p(1)` ضریب بالاترین توان.

use mc_core::{ComplexMatrix, Matrix, McError};

use crate::linalg::{eig_complex, qr, trisolve_upper};

use crate::linalg::num_error;

/// حذف صفرهای پیشرو (ضرایب نزولی) — `roots([0 0 1 2])` = `roots([1 2])`.
/// همه صفر → خروجی خالی (رفتار متلب: `roots(zeros(1,3))` = 0×1).
fn trim_leading_zeros(p: &[f64]) -> &[f64] {
    let first = p
        .iter()
        .position(|c| *c != 0.0 && !c.is_nan())
        .unwrap_or(p.len());
    &p[first..]
}

/// `roots(p)` — ریشه‌های چندجمله‌ای از eig ماتریس همبند (سند ۰۴ §5.2).
///
/// - ورودی بردار (هر جهتی) با ضرایب نزولی؛ صفرهای پیشرو حذف می‌شوند.
/// - خروجی ستون‌بردار n×1 مختلط (n = درجه) — مرتب‌سازی: صعودی (re، im)؛
///   متلب ترتیب تضمین‌شده‌ای نمی‌دهد و اوراکل کورپوس هر دو طرف مرتب می‌کند.
pub fn roots(p: &Matrix) -> Result<ComplexMatrix, McError> {
    if p.rows() != 1 && p.cols() != 1 {
        return Err(num_error(
            mc_core::error::ids::SIZE_MISMATCH,
            "roots: input must be a vector.",
            "roots: ورودی باید بردار باشد.",
        ));
    }
    let coeffs: Vec<f64> = (0..p.numel()).map(|i| p.get_linear(i)).collect();
    let p = trim_leading_zeros(&coeffs);
    let n = p.len().saturating_sub(1);
    if n == 0 {
        return Ok(ComplexMatrix::full(0, 1, 0.0, 0.0));
    }
    // ماتریس همبند: ردیف اول = -p(2:end)/p(1)؛ زیرقطر = 1 (قرارداد متلب compan)
    let p0 = p[0];
    let mut c = Matrix::full(n, n, 0.0);
    for j in 0..n {
        c.set(0, j, -p[j + 1] / p0);
    }
    for i in 1..n {
        c.set(i, i - 1, 1.0);
    }
    let (values, _) = eig_complex(&c)?;
    // ستون‌بردار n×1 مختلط — مرتب‌سازی صعودی (re، im) برای تکرارپذیری
    let mut sorted = values;
    sorted.sort_by(|a, b| {
        a.0.partial_cmp(&b.0)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
    });
    let mut re = Vec::with_capacity(n);
    let mut im = Vec::with_capacity(n);
    for (r, i) in sorted {
        re.push(r);
        im.push(i);
    }
    Ok(ComplexMatrix::from_columns(n, 1, re, im))
}

/// `polyfit(x, y, n)` — برازش چندجمله‌ای با QR (سند ۰۴ §6: هرگز AᵀA).
///
/// - ورودها بردار هم‌طول؛ درجه n با m = numel ≥ n+1.
/// - خروجی ضرایب نزولی (n+1)×1.
/// - رتبه‌ناقص (قطری R تقریباً صفر) → خطای صادقانه — مثل متلب که warning می‌دهد،
///   ولی MathCraft نتیجه مشکوک silent نمی‌دهد (سند ۰۴ §5.3 قاعده کیفیت).
pub fn polyfit(x: &Matrix, y: &Matrix, degree: usize) -> Result<Matrix, McError> {
    let (mx, nx) = (x.rows(), x.cols());
    let (my, ny) = (y.rows(), y.cols());
    if (mx != 1 && nx != 1) || (my != 1 && ny != 1) {
        return Err(num_error(
            mc_core::error::ids::SIZE_MISMATCH,
            "polyfit: X and Y must be vectors.",
            "polyfit: X و Y باید بردار باشند.",
        ));
    }
    let xs: Vec<f64> = (0..x.numel()).map(|i| x.get_linear(i)).collect();
    let ys: Vec<f64> = (0..y.numel()).map(|i| y.get_linear(i)).collect();
    if xs.len() != ys.len() {
        return Err(num_error(
            mc_core::error::ids::SIZE_MISMATCH,
            format!(
                "polyfit: X and Y must have the same length ({} vs {}).",
                xs.len(),
                ys.len()
            ),
            format!(
                "polyfit: X و Y باید هم‌طول باشند ({} در برابر {}).",
                xs.len(),
                ys.len()
            ),
        ));
    }
    let k = degree + 1; // تعداد ضریب‌ها
    if xs.len() < k {
        return Err(num_error(
            mc_core::error::ids::SIZE_MISMATCH,
            format!(
                "polyfit: degree {degree} needs at least {k} points, got {}.",
                xs.len()
            ),
            format!(
                "polyfit: درجه {degree} دست‌کم {k} نقطه می‌خواهد، {} داده شد.",
                xs.len()
            ),
        ));
    }
    // ماتریس واندرمونده m×k: ستون j = x.^ (k-1-j) — ضرایب نزولی
    let m = xs.len();
    let mut vander = Matrix::full(m, k, 0.0);
    for (r, &xv) in xs.iter().enumerate() {
        for c in 0..k {
            vander.set(r, c, xv.powi((k - 1 - c) as i32));
        }
    }
    let ymat = Matrix::from_columns(m, 1, ys);
    // QR خانه‌دار — حل least squares: R(1:k,1:k) \ (Qᵀ y)(1:k)
    let (q, r_full) = qr(&vander)?;
    // qy = Qᵀ·y — فقط k مؤلفه اول لازم است
    let mut qy = vec![0.0f64; k];
    for (c, qy_c) in qy.iter_mut().enumerate() {
        let mut acc = 0.0;
        for r in 0..m {
            acc += q.get(r, c) * ymat.get(r, 0);
        }
        *qy_c = acc;
    }
    // R مثلثی بالایی k×k (بخش بالای r_full)
    let mut rtri = Matrix::full(k, k, 0.0);
    for r in 0..k {
        for c in 0..k {
            rtri.set(r, c, r_full.get(r, c));
        }
    }
    // چک رتبه: قطری‌ها در برابر بزرگ‌ترین — آستانه نسبی
    let mut rmax = 0.0f64;
    for i in 0..k {
        rmax = rmax.max(rtri.get(i, i).abs());
    }
    if rmax == 0.0 {
        return Err(num_error(
            mc_core::error::ids::SINGULAR_MATRIX,
            "polyfit: Vandermonde system is identically zero (rank deficient).",
            "polyfit: دستگاه واندرمونده دقیقاً صفر است (رتبه ناقص).",
        ));
    }
    for i in 0..k {
        if rtri.get(i, i).abs() <= 1e-12 * rmax {
            return Err(num_error(
                mc_core::error::ids::SINGULAR_MATRIX,
                format!(
                    "polyfit: rank deficient (degree {degree} too high for this data); \
                     center/scale the points or lower the degree."
                ),
                format!(
                    "polyfit: رتبه ناقص است (درجه {degree} برای این داده زیاد است)؛ \
                     نقاط را مرکز/مقیاس کنید یا درجه را کم کنید."
                ),
            )
            .with_suggestion("use polyfit with lower degree or center the x data"));
        }
    }
    let b = trisolve_upper(&rtri, &Matrix::from_columns(k, 1, qy))?;
    Ok(b)
}

/// `polyval(p, x)` — ارزیابی چندجمله‌ای با Horner (سند ۰۴ §6) — عنصری روی ماتریس.
pub fn polyval(p: &Matrix, x: &Matrix) -> Result<Matrix, McError> {
    if p.rows() != 1 && p.cols() != 1 {
        return Err(num_error(
            mc_core::error::ids::SIZE_MISMATCH,
            "polyval: P must be a vector of coefficients.",
            "polyval: P باید بردار ضرایب باشد.",
        ));
    }
    let coeffs: Vec<f64> = (0..p.numel()).map(|i| p.get_linear(i)).collect();
    let mut out = Matrix::full(x.rows(), x.cols(), 0.0);
    for idx in 0..x.numel() {
        let xv = x.get_linear(idx);
        let mut acc = 0.0f64;
        for &c in &coeffs {
            acc = acc * xv + c;
        }
        out.set_linear(idx, acc);
    }
    Ok(out)
}

/// `conv(u, v)` — کانولوشن کامل؛ طول خروجی m+n−1.
pub fn conv(u: &Matrix, v: &Matrix) -> Result<Matrix, McError> {
    if (u.rows() != 1 && u.cols() != 1) || (v.rows() != 1 && v.cols() != 1) {
        return Err(num_error(
            mc_core::error::ids::SIZE_MISMATCH,
            "conv: inputs must be vectors.",
            "conv: ورودی‌ها باید بردار باشند.",
        ));
    }
    let a: Vec<f64> = (0..u.numel()).map(|i| u.get_linear(i)).collect();
    let b: Vec<f64> = (0..v.numel()).map(|i| v.get_linear(i)).collect();
    let (m, n) = (a.len(), b.len());
    let mut out = vec![0.0f64; m + n - 1];
    for (i, &ai) in a.iter().enumerate() {
        for (j, &bj) in b.iter().enumerate() {
            out[i + j] += ai * bj;
        }
    }
    // شکل خروجی: اگر هر دو سطری بودند 1×(m+n-1) وگرنه ستونی (رفتار متلب: shape بردار اول؟
    // متلب conv همیشه بردار می‌دهد؛ جهت = جهت u). ما جهت u را نگه می‌داریم.
    if u.rows() == 1 {
        Ok(Matrix::from_columns(1, out.len(), out))
    } else {
        Ok(Matrix::from_columns(out.len(), 1, out))
    }
}

/// خروجی `deconv` — [quotient, remainder] (چندخروجی متلب `[q,r] = deconv(u,v)`).
pub struct DeconvResult {
    pub quotient: Matrix,
    pub remainder: Matrix,
}

/// `deconv(u, v)` — تقسیم چندجمله‌ای: u = conv(v,q) + r با درجه r < درجه v.
///
/// شکل خروجی‌ها مثل متلب: هم‌شکل u (همان تعداد عنصر؛ r در ابتدا و صفر در انتها).
pub fn deconv(u: &Matrix, v: &Matrix) -> Result<DeconvResult, McError> {
    if (u.rows() != 1 && u.cols() != 1) || (v.rows() != 1 && v.cols() != 1) {
        return Err(num_error(
            mc_core::error::ids::SIZE_MISMATCH,
            "deconv: inputs must be vectors.",
            "deconv: ورودی‌ها باید بردار باشند.",
        ));
    }
    let a: Vec<f64> = (0..u.numel()).map(|i| u.get_linear(i)).collect();
    let b: Vec<f64> = (0..v.numel()).map(|i| v.get_linear(i)).collect();
    let m = a.len();
    let n = b.len();
    if n == 0 {
        return Err(num_error(
            mc_core::error::ids::SIZE_MISMATCH,
            "deconv: divisor must be non-empty.",
            "deconv: مقسوم‌علیه نباید خالی باشد.",
        ));
    }
    if n > m {
        // q = 0 اسکالر، r = u (قرارداد متلب deconv([1],[4 5]))
        return Ok(DeconvResult {
            quotient: shape_like(u, vec![0.0]),
            remainder: shape_like(u, a),
        });
    }
    // تقسیم طولانی (مقسوم b — ضرایب نزولی)
    let mut rem = a.clone();
    let mut q = vec![0.0f64; m];
    for i in 0..=(m - n) {
        let factor = rem[i] / b[0];
        q[i] = factor;
        for j in 0..n {
            rem[i + j] -= factor * b[j];
        }
    }
    // q طول m−n+1 (قرارداد متلب)؛ باقیمانده هم‌شکل u: اول m−n+1 صفر + دُم n−1 تایی
    q.truncate(m - n + 1);
    let mut remainder = vec![0.0f64; m];
    remainder[m - n + 1..].copy_from_slice(&rem[m - n + 1..]);
    Ok(DeconvResult {
        quotient: shape_like(u, q),
        remainder: shape_like(u, remainder),
    })
}

/// بردار با شکل هم‌جهت `u` از داده تخت.
fn shape_like(u: &Matrix, data: Vec<f64>) -> Matrix {
    if u.rows() == 1 {
        Matrix::from_columns(1, data.len(), data)
    } else {
        Matrix::from_columns(data.len(), 1, data)
    }
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

    fn assert_close(a: f64, b: f64, tol: f64) {
        assert!(
            (a - b).abs() <= tol * b.abs().max(1.0),
            "{a} vs {b} (tol {tol})"
        );
    }

    #[test]
    fn roots_simple_real_and_complex() {
        // p = [1 0 1] → x² + 1 = 0 → ±i
        let r = roots(&row(&[1.0, 0.0, 1.0])).unwrap_or_else(|e| panic!("{e:?}"));
        assert_eq!((r.rows(), r.cols()), (2, 1));
        assert_eq!(r.get(0, 0), (0.0, -1.0));
        assert_eq!(r.get(1, 0), (0.0, 1.0));
        // p = [1 -3 2] → (x−1)(x−2) → {1, 2}
        let r = roots(&row(&[1.0, -3.0, 2.0])).unwrap();
        assert_close(r.get(0, 0).0, 1.0, 1e-12);
        assert_close(r.get(1, 0).0, 2.0, 1e-12);
        // صفرهای پیشرو
        let r = roots(&row(&[0.0, 0.0, 1.0, -3.0, 2.0])).unwrap();
        assert_eq!(r.numel(), 2);
        // همه صفر → خالی
        let r = roots(&row(&[0.0, 0.0])).unwrap();
        assert_eq!(r.numel(), 0);
    }

    #[test]
    fn roots_via_polynomial_multiplication_chain() {
        // (x−1)(x−2)(x−3)(x−4) = x⁴−10x³+35x²−50x+24 → ریشه‌های 1..4
        let p = row(&[1.0, -10.0, 35.0, -50.0, 24.0]);
        let r = roots(&p).unwrap();
        let mut got: Vec<f64> = (0..4).map(|i| r.get(i, 0).0).collect();
        got.sort_by(|a, b| a.partial_cmp(b).unwrap());
        for (g, want) in got.iter().zip([1.0, 2.0, 3.0, 4.0]) {
            assert_close(*g, want, 1e-9);
        }
        for i in 0..4 {
            assert_close(r.get(i, 0).1, 0.0, 1e-9);
        }
    }

    #[test]
    fn polyfit_exact_recovery_quadratic() {
        // y = 2x² − x + 3 روی نقاط دلخواه → بازیابی دقیق (کلوزفورم)
        let x = col(&[-2.0, -1.0, 0.0, 1.0, 2.0]);
        let y = col(&[13.0, 6.0, 3.0, 4.0, 9.0]);
        let p = polyfit(&x, &y, 2).unwrap_or_else(|e| panic!("{e:?}"));
        assert_eq!(p.numel(), 3);
        assert_close(p.get(0, 0), 2.0, 1e-9);
        assert_close(p.get(1, 0), -1.0, 1e-9);
        assert_close(p.get(2, 0), 3.0, 1e-9);
    }

    #[test]
    fn polyfit_overdetermined_cubic_noiseless() {
        // ۱۰ نقطه از یک درجه-۳ → بازیابی در حد 1e-9
        let xs: Vec<f64> = (0..10).map(|i| i as f64 * 0.5).collect();
        let ys: Vec<f64> = xs
            .iter()
            .map(|&x| 0.5 * x * x * x - 2.0 * x * x + x - 7.0)
            .collect();
        let p = polyfit(&col(&xs), &col(&ys), 3).unwrap();
        assert_close(p.get(0, 0), 0.5, 1e-9);
        assert_close(p.get(1, 0), -2.0, 1e-9);
        assert_close(p.get(2, 0), 1.0, 1e-9);
        assert_close(p.get(3, 0), -7.0, 1e-9);
    }

    #[test]
    fn polyfit_rank_deficient_errors_honestly() {
        // ۳ نقطه با درجه ۳ → ۴ مجهول از ۳ معادله → رتبه ناقص
        let x = col(&[1.0, 2.0, 3.0]);
        let y = col(&[1.0, 4.0, 9.0]);
        let r = polyfit(&x, &y, 3);
        assert!(r.is_err(), "rank-deficient fit must error, not guess");
    }

    #[test]
    fn polyval_horner_matches_reference() {
        let p = row(&[2.0, -1.0, 3.0]); // 2x² − x + 3
        let x = row(&[0.0, 1.0, -1.0, 2.5]);
        let y = polyval(&p, &x).unwrap();
        assert_close(y.get(0, 0), 3.0, 1e-15);
        assert_close(y.get(0, 1), 4.0, 1e-15);
        assert_close(y.get(0, 2), 6.0, 1e-15);
        // 2(2.5)² − 2.5 + 3 = 12.5 − 2.5 + 3 = 13
        assert_close(y.get(0, 3), 13.0, 1e-15);
    }

    #[test]
    fn conv_and_deconv_round_trip() {
        let u = row(&[1.0, 2.0, 3.0]);
        let v = row(&[4.0, 5.0]);
        let c = conv(&u, &v).unwrap_or_else(|e| panic!("{e:?}"));
        assert_eq!(c.numel(), 4);
        assert_close(c.get(0, 0), 4.0, 1e-15);
        assert_close(c.get(0, 1), 13.0, 1e-15);
        assert_close(c.get(0, 2), 22.0, 1e-15);
        assert_close(c.get(0, 3), 15.0, 1e-15);
        // deconv(c, v) = u — طول q = m−n+1 = 2؟ نه: اینجا دقیقاً مقسوم است → q = u کامل (طول 3 = m−n+1 با m=4، n=2)
        let d = deconv(&c, &v).unwrap();
        assert_eq!(d.quotient.numel(), 3);
        assert_close(d.quotient.get(0, 0), 1.0, 1e-12);
        assert_close(d.quotient.get(0, 1), 2.0, 1e-12);
        assert_close(d.quotient.get(0, 2), 3.0, 1e-12);
        // باقیمانده صفر
        for i in 0..d.remainder.numel() {
            assert_close(d.remainder.get_linear(i), 0.0, 1e-12);
        }
        // deconv با باقیمانده واقعی: (x²+2x+3) ÷ (4x+5) → q=[0.25 0.1875] r=[0 0 2.0625]
        let d2 = deconv(&u, &v).unwrap();
        assert_eq!(d2.quotient.numel(), 2);
        assert_close(d2.quotient.get(0, 0), 0.25, 1e-12);
        assert_close(d2.quotient.get(0, 1), 0.1875, 1e-12);
        assert_close(d2.remainder.get_linear(0), 0.0, 1e-12);
        assert_close(d2.remainder.get_linear(1), 0.0, 1e-12);
        assert_close(d2.remainder.get_linear(2), 2.0625, 1e-12);
    }
}
