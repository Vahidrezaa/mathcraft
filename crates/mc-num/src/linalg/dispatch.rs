//! درخت dispatch اپراتور `\` و `/` (سند ۰۴ §3.2 — رفتار امضادار متلب).
//!
//! «یک درخت، دو مصرف»: هم اجرا و هم افزونه آموزشی `steps(A\b)` از همین تابع
//! استفاده می‌کنند (سند ۰۴ §3.2، سند ۰۳ §13) — پس هر شاخه با `DispatchStep`
//! ثبت می‌شود تا مسیر تصمیم‌گیری قابل نمایش باشد.
//!
//! ترتیب شاخه‌ها (دقیقاً طبق سند ۰۴ §3.2):
//! 1. اسکالر → `b./a`
//! 2. مثلثی (چک ساختاری) → حل مثلثی O(n²)
//! 3. سه‌قطری → الگوریتم Thomas O(n)
//! 4. مثلثیِ پرموته → پرموت سپس حل مثلثی
//! 5. مربعی متقارن: chol موفق → chol-solve؛ وگرنه LDLᵀ
//! 6. مربعی عمومی → LU پیموت جزئی (+ هشدار rcond کوچک در steps)
//! 7. مستطیلی m>n → QR پیموت‌دار = کمینه مربعات (+ ثبت کم‌رتبتی)
//! 8. مستطیلی m<n → کم‌نُرم (QR روی Aᵀ)

use crate::linalg::convert;
use crate::linalg::solve::ldlt_solve_pub;
use crate::linalg::{matmul, num_error, DispatchStep};
use faer::prelude::*;
use mc_core::error::ids;
use mc_core::matrix::Matrix;

/// حل `A\b` — انتخاب خودکار الگوریتم طبق درخت سند ۰۴ §3.2.
///
/// خروجی: جواب X (a.cols? — برای مستطیلی، m×k) + رد گام‌های dispatch.
pub fn mldivide(
    a: &Matrix,
    b: &Matrix,
) -> Result<(Matrix, Vec<DispatchStep>), mc_core::error::McError> {
    let mut steps: Vec<DispatchStep> = Vec::new();

    // شاخه ۱ — اسکالر: متلب `A\b` با A اسکالر = b./A (سند ۰۴ §3.2).
    if a.is_scalar() {
        steps.push(DispatchStep::new(
            "SCALAR",
            "ورودی اسکالر — تقسیم عنصری b./a",
            "scalar input — elementwise b./a",
        ));
        let s = a.get(0, 0);
        // تقسیم بر صفر: IEEE → Inf/NaN بدون خطا (سند ۰۳ §11 — رفتار متلب).
        let mut out = Matrix::full(b.rows(), b.cols(), 0.0);
        for idx in 0..b.numel() {
            out.set_linear(idx, b.get_linear(idx) / s);
        }
        return Ok((out, steps));
    }

    // کنترل شکل: تعداد ردیف‌های A و b باید برابر باشد (سند ۰۲ §9).
    if a.rows() != b.rows() {
        return Err(num_error(
            ids::SIZE_MISMATCH,
            format!(
                "matrix division: rows of A ({}) must equal rows of B ({}).",
                a.rows(),
                b.rows()
            ),
            format!(
                "تقسیم ماتریسی: تعداد ردیف‌های A ({}) باید با ردیف‌های B ({}) برابر باشد.",
                a.rows(),
                b.rows()
            ),
        ));
    }

    // شاخه‌های ۲-۴ — ساختاری، فقط برای مربع (مثل متلب؛ مستطیلی هرگز مسیر مثلثی نمی‌رود).
    if a.rows() == a.cols() {
        // شاخه ۲ — مثلثی ساختاری (بالا/پایین با چک دقیق صفر — سند ۰۴ §3.2).
        match crate::linalg::is_triangular(a) {
            Some(true) => {
                steps.push(DispatchStep::new(
                    "TRIANGULAR",
                    "ماتریس مثلثی بالا — حل مثلثی O(n²)",
                    "upper triangular — O(n²) triangular solve",
                ));
                return Ok((crate::linalg::trisolve_upper(a, b)?, steps));
            }
            Some(false) => {
                steps.push(DispatchStep::new(
                    "TRIANGULAR",
                    "ماتریس مثلثی پایین — حل مثلثی O(n²)",
                    "lower triangular — O(n²) triangular solve",
                ));
                return Ok((crate::linalg::trisolve_lower(a, b)?, steps));
            }
            None => {}
        }

        // شاخه ۳ — سه‌قطری: Thomas O(n) — فقط n≥3 (برای n≤2 هر ماتریسی «سه‌قطری» است
        // و متلب مسیر band نمی‌رود؛ Thomas بدون پیموت برای n≤2 فایده ندارد).
        if a.rows() >= 3 && crate::linalg::is_tridiagonal(a) {
            steps.push(DispatchStep::new(
                "TRIDIAGONAL",
                "ماتریس سه‌قطری — الگوریتم Thomas O(n)",
                "tridiagonal — Thomas algorithm O(n)",
            ));
            match crate::linalg::thomas_solve(a, b) {
                Ok(x) => return Ok((x, steps)),
                Err(_) => {
                    // پیوت صفر در Thomas → سقوط به مسیر مربعی با ثبت صادقانه.
                    steps.push(DispatchStep::new(
                        "LU",
                        "Thomas با پیوت صفر شکست خورد — سقوط به LU پیموت جزئی",
                        "Thomas hit a zero pivot — falling back to partial-pivot LU",
                    ));
                }
            }
        }

        // شاخه ۴ — مثلثیِ پرموته: یافتن پرموتاسیون ردیفی که ماتریس را مثلثی کند.
        if let Some(perm) = crate::linalg::is_permuted_triangular(a) {
            let (upper, _) = detect_permuted_orientation(a, &perm);
            steps.push(DispatchStep::new(
                "PERMUTED_TRIANGULAR",
                "پرموتاسیون مثلثی شناسایی شد — پرموت سپس حل مثلثی O(n²)",
                "permuted triangular detected — permute then O(n²) solve",
            ));
            let ap = convert::permute_rows(a, &perm);
            let bp = convert::permute_rows(b, &perm);
            let x = if upper {
                crate::linalg::trisolve_upper(&ap, &bp)?
            } else {
                crate::linalg::trisolve_lower(&ap, &bp)?
            };
            return Ok((x, steps));
        }
    }

    let (m, n) = (a.rows(), a.cols());

    if m == n {
        // شاخه ۵ — مربعی متقارن: chol → LDLᵀ (سند ۰۴ §3.2 Hermitian).
        if convert::is_exactly_symmetric(a) {
            match crate::linalg::chol(a) {
                Ok(l) => {
                    steps.push(DispatchStep::new(
                        "CHOL",
                        "متقارن مثبت معین — حل با Cholesky (دو حل مثلثی)",
                        "symmetric positive definite — Cholesky solve (two triangular solves)",
                    ));
                    // L·Lᵀ·x = b  ⟹  y = حل L·y = b (پایین)؛ x = حل Lᵀ·x = y (بالا).
                    let lt = convert::transpose(&l);
                    let y = crate::linalg::trisolve_lower(&l, b)?;
                    let x = crate::linalg::trisolve_upper(&lt, &y)?;
                    return Ok((x, steps));
                }
                Err(_) => {
                    steps.push(DispatchStep::new(
                        "LDLT",
                        "متقارن اما مثبت معین نیست — تجزیه LDLᵀ",
                        "symmetric but not positive definite — LDLᵀ solve",
                    ));
                    match ldlt_solve_pub(a, b) {
                        Ok(x) => return Ok((x, steps)),
                        Err(_) => {
                            // LDLᵀ با پیوت صفر شکست خورد → سقوط به LU عمومی با ثبت گام.
                            steps.push(DispatchStep::new(
                                "LU",
                                "LDLᵀ با پیوت صفر شکست خورد — سقوط به LU پیموت جزئی",
                                "LDLᵀ hit a zero pivot — falling back to partial-pivot LU",
                            ));
                        }
                    }
                }
            }
        }

        // شاخه ۶ — مربعی عمومی: LU پیموت جزئی (سند ۰۴ §3.2).
        steps.push(DispatchStep::new(
            "LU",
            "مربعی عمومی — LU با پیموت جزئی (faer)",
            "square general — LU with partial pivoting (faer)",
        ));
        let x = lu_solve(a, b, Some(&mut steps))?;
        return Ok((x, steps));
    }

    // شاخه ۷ — مستطیلی بلند m>n: کمینه مربعات با QR پیموت ستونی (سند ۰۴ §3.2).
    if m > n {
        steps.push(DispatchStep::new(
            "QR_LS",
            "مستطیلی بلند m>n — کمینه مربعات با QR پیموت ستونی",
            "tall m>n — least squares via column-pivoted QR",
        ));
        return ls_via_qr(a, b, &mut steps).map(|x| (x, steps));
    }

    // شاخه ۸ — مستطیلی پهن m<n: راه‌حل کم-نُرم با شبه‌وارون SVD (سند ۰۴ §3.2 —
    // «راه‌حل کم-نُرم»؛ SVD پایدارترین راه است و کم‌رتبتی را هم مدیریت می‌کند).
    steps.push(DispatchStep::new(
        "MIN_NORM",
        "مستطیلی پهن m<n — راه‌حل کم-نُرم با شبه‌وارون (SVD)",
        "wide m<n — minimum-norm solution via SVD pseudoinverse",
    ));
    min_norm_via_svd(a, b, &mut steps).map(|x| (x, steps))
}

/// کم-نُرم با SVD: `X = V·S⁺·Uᵀ·B` — ستون‌های با σ≤tol حذف می‌شوند.
fn min_norm_via_svd(
    a: &Matrix,
    b: &Matrix,
    steps: &mut Vec<DispatchStep>,
) -> Result<Matrix, mc_core::error::McError> {
    let (m, n) = (a.rows(), a.cols());
    let (u, s, v) = crate::linalg::svd(a)?;
    let smax = s.first().copied().unwrap_or(0.0);
    let tol = (m.max(n) as f64) * f64::EPSILON * smax;
    let r = s.iter().filter(|&&v| v > tol).count();
    if r < s.len() {
        steps.push(DispatchStep::new(
            "WARN",
            &format!("ماتریس کم‌رتبه است (رتبه مؤثر {r} از {n}) — جواب کم‌نُرم داده می‌شود"),
            &format!("rank-deficient (effective rank {r} of {n}) — minimum-norm answer returned"),
        ));
    }
    // UT_B = Uᵀ·B (m×k) — فقط r سطر اول لازم است.
    let mut xb = Matrix::full(n, b.cols(), 0.0);
    for c in 0..b.cols() {
        // برای هر ستون از B: z_i = (u_iᵀ·b)/σ_i برای i<r؛ سپس X = Σ z_i·v_i.
        let mut z = vec![0.0f64; r];
        for (i, zi) in z.iter_mut().enumerate().take(r) {
            let mut acc = 0.0;
            for rr in 0..m {
                acc += u.get(rr, i) * b.get(rr, c);
            }
            *zi = acc / s[i];
        }
        for (i, &zi) in z.iter().enumerate() {
            for rr in 0..n {
                xb.set(rr, c, xb.get(rr, c) + zi * v.get(rr, i));
            }
        }
    }
    Ok(xb)
}

/// حل `B/A` = `(Aᵀ\Bᵀ)ᵀ` (سمانتیک متلب؛ سند ۰۴ §3.2).
pub fn mrdivide(
    b: &Matrix,
    a: &Matrix,
) -> Result<(Matrix, Vec<DispatchStep>), mc_core::error::McError> {
    let bt = convert::transpose(b);
    let at = convert::transpose(a);
    let (x, steps) = mldivide(&at, &bt)?;
    Ok((convert::transpose(&x), steps))
}

/// جهت مثلثی‌بودن پس از پرموت — `true` یعنی مثلثی بالا.
fn detect_permuted_orientation(a: &Matrix, perm: &[usize]) -> (bool, Matrix) {
    let ap = convert::permute_rows(a, perm);
    (crate::linalg::is_triangular(&ap) == Some(true), ap)
}

/// حل مربعی با LU (faer) + ثبت هشدار نزدیکی تکینگی در steps (سند ۰۴ §3.2).
fn lu_solve(
    a: &Matrix,
    b: &Matrix,
    mut steps: Option<&mut Vec<DispatchStep>>,
) -> Result<Matrix, mc_core::error::McError> {
    let af = convert::to_faer(a);
    let bf = convert::to_faer(b);
    let lu = af.partial_piv_lu();
    // نمای تقریبی rcond: نسبت قطر U (پروکسی ارزان — سند ۰۴ §3.2 «هشدار rcond کوچک»).
    let u = lu.U();
    let mut umin = f64::INFINITY;
    let mut umax = 0.0f64;
    for i in 0..u.ncols().min(u.nrows()) {
        let v = u[(i, i)].abs();
        if v > 0.0 {
            umin = umin.min(v);
            umax = umax.max(v);
        } else {
            umin = 0.0;
            break;
        }
    }
    if let Some(st) = steps.as_mut() {
        if umin == 0.0 {
            st.push(DispatchStep::new(
                "WARN",
                "ماتریس دقیقاً تکین — پیوت صفر در U",
                "exactly singular — zero pivot in U",
            ));
        } else {
            let rcond = umin / umax;
            if rcond < 1e-12 {
                st.push(DispatchStep::new(
                    "WARN",
                    "ماتریس نزدیک تکین (rcond ≈ 1e-12-) — نتیجه ممکن است ناپایدار باشد",
                    "matrix close to singular (rcond < 1e-12) — result may be unstable",
                ));
            }
        }
    }
    if umin == 0.0 {
        return Err(num_error(
            ids::SINGULAR_MATRIX,
            "Matrix is exactly singular (zero pivot in LU); use cond/rank to inspect.",
            "ماتریس دقیقاً تکین است (پیوت صفر در LU)؛ برای بازرسی از cond/rank استفاده کنید.",
        ));
    }
    let xf = lu.solve(bf);
    Ok(convert::from_faer(&xf))
}

/// کمینه مربعات با QR پیموت ستونی + ثبت کم‌رتبتی (سند ۰۴ §3.2 شاخه m>n).
fn ls_via_qr(
    a: &Matrix,
    b: &Matrix,
    steps: &mut Vec<DispatchStep>,
) -> Result<Matrix, mc_core::error::McError> {
    let (m, n) = (a.rows(), a.cols());
    let (q, r, e) = crate::linalg::qr_pivoted(a)?;

    // آستانه کم‌رتبتی متلب: max(m,n)*eps*|R(1,1)|.
    let r11 = if n > 0 { r.get(0, 0).abs() } else { 0.0 };
    let tol = (m.max(n) as f64) * f64::EPSILON * r11;
    let effective = (0..n).filter(|&i| r.get(i, i).abs() > tol).count();
    if effective < n {
        steps.push(DispatchStep::new(
            "WARN",
            &format!("ماتریس کم‌رتبه است (رتبه مؤثر {effective} از {n}) — جواب کم‌نُرم داده می‌شود"),
            &format!(
                "rank-deficient (effective rank {effective} of {n}) — minimum-norm answer returned"
            ),
        ));
    }

    // R·z = Qᵀ·b برای n مؤثر اول؛ zهای باقی صفر (راه‌حل کم‌نُرم).
    let qtb = matmul(&convert::transpose(&q), b)?;
    let mut z = Matrix::full(n, b.cols(), 0.0);
    // حل مثلثی بالای R·z = Qᵀ·b — پس‌رو: از k=effective-1 به پایین،
    // z_k = (y_k − Σ_{j>k} R(k,j)·z_j) / R(k,k).
    for c in 0..b.cols() {
        for rk_idx in (0..effective).rev() {
            let mut acc = qtb.get(rk_idx, c);
            for j in (rk_idx + 1)..effective {
                acc -= r.get(rk_idx, j) * z.get(j, c);
            }
            let rk = r.get(rk_idx, rk_idx);
            if rk == 0.0 {
                return Err(num_error(
                    ids::SINGULAR_MATRIX,
                    "Least-squares solve failed: zero pivot in R.",
                    "حل کمینه مربعات شکست خورد: پیوت صفر در R.",
                ));
            }
            z.set(rk_idx, c, acc / rk);
        }
    }
    // X = E·z (ستون‌ها را به ترتیب اصلی برمی‌گرداند).
    matmul(&e, &z)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::linalg::convert::max_abs_diff;

    fn mat(rows: usize, cols: usize, data: &[f64]) -> Matrix {
        Matrix::from_columns(rows, cols, data.to_vec())
    }

    #[test]
    fn scalar_branch_is_elementwise() {
        let a = Matrix::scalar(2.0);
        let b = mat(2, 1, &[4.0, 10.0]);
        let (x, steps) = mldivide(&a, &b).unwrap();
        assert_eq!(steps[0].branch, "SCALAR");
        assert!(max_abs_diff(&x, &mat(2, 1, &[2.0, 5.0])) < 1e-12);
    }

    #[test]
    fn upper_triangular_branch() {
        // [2 1; 0 4] \ [5; 8] → x2=2، x1=(5-2)/2=1.5
        let a = mat(2, 2, &[2.0, 0.0, 1.0, 4.0]);
        let b = mat(2, 1, &[5.0, 8.0]);
        let (x, steps) = mldivide(&a, &b).unwrap();
        assert_eq!(steps[0].branch, "TRIANGULAR");
        assert!(max_abs_diff(&x, &mat(2, 1, &[1.5, 2.0])) < 1e-12);
    }

    #[test]
    fn lower_triangular_branch() {
        let a = mat(2, 2, &[2.0, 0.0, 0.0, 4.0]);
        let b = mat(2, 1, &[4.0, 12.0]);
        let (x, steps) = mldivide(&a, &b).unwrap();
        assert_eq!(steps[0].branch, "TRIANGULAR");
        assert!(max_abs_diff(&x, &mat(2, 1, &[2.0, 3.0])) < 1e-12);
    }

    #[test]
    fn tridiagonal_branch_matches_lu() {
        // سه‌قطری 5×5 قطری‌مسلط — Thomas باید هم‌جواب LU باشد.
        let n = 5;
        let mut a = Matrix::full(n, n, 0.0);
        for i in 0..n {
            a.set(i, i, 4.0);
            if i + 1 < n {
                a.set(i, i + 1, 1.0);
                a.set(i + 1, i, -1.0);
            }
        }
        let b = mat(n, 1, &(1..=n).map(|v| v as f64).collect::<Vec<_>>());
        let (x1, steps) = mldivide(&a, &b).unwrap();
        assert_eq!(steps[0].branch, "TRIDIAGONAL");
        let (x2, _) = mldivide(&a, &b).unwrap();
        assert!(max_abs_diff(&x1, &x2) == 0.0);
        // A·x = b
        let ax = crate::linalg::norm_func::matmul(&a, &x1).unwrap();
        assert!(max_abs_diff(&ax, &b) < 1e-10);
    }

    #[test]
    fn permuted_triangular_branch() {
        // lower مثلثی با ردیف‌های جابه‌جاشده (و نه سه‌قطری): A = [[2,3,0],[4,5,6],[1,0,0]]
        // پرموت [2,0,1] آن را به L پایین‌مثلثی [[1,0,0],[2,3,0],[4,5,6]] تبدیل می‌کند.
        let a = mat(3, 3, &[2.0, 4.0, 1.0, 3.0, 5.0, 0.0, 0.0, 6.0, 0.0]);
        let b = mat(3, 1, &[1.0, 2.0, 3.0]);
        let (x, steps) = mldivide(&a, &b).unwrap();
        assert_eq!(steps[0].branch, "PERMUTED_TRIANGULAR");
        let ax = crate::linalg::matmul(&a, &x).unwrap();
        assert!(max_abs_diff(&ax, &b) < 1e-12);
    }

    #[test]
    fn symmetric_spd_branch_uses_chol() {
        // SPD: [4 2; 2 3] \ [1; 2]
        let a = mat(2, 2, &[4.0, 2.0, 2.0, 3.0]);
        let b = mat(2, 1, &[1.0, 2.0]);
        let (x, steps) = mldivide(&a, &b).unwrap();
        assert_eq!(steps[0].branch, "CHOL");
        let ax = crate::linalg::norm_func::matmul(&a, &x).unwrap();
        assert!(max_abs_diff(&ax, &b) < 1e-12);
    }

    #[test]
    fn symmetric_indefinite_branch_uses_ldlt() {
        // متقارن نامعین (نه پرموته-مثلثی): [1 2; 2 -1] \ [1; 2] → x = [1; 0]
        let a = mat(2, 2, &[1.0, 2.0, 2.0, -1.0]);
        let b = mat(2, 1, &[1.0, 2.0]);
        let (x, steps) = mldivide(&a, &b).unwrap();
        assert!(steps.iter().any(|s| s.branch == "LDLT" || s.branch == "LU"));
        assert!(max_abs_diff(&x, &mat(2, 1, &[1.0, 0.0])) < 1e-12);
    }

    #[test]
    fn general_square_branch_uses_lu() {
        let a = mat(3, 3, &[1.0, 2.0, 3.0, 0.0, 1.0, 4.0, 5.0, 6.0, 0.0]);
        let b = mat(3, 1, &[1.0, 1.0, 1.0]);
        let (x, steps) = mldivide(&a, &b).unwrap();
        assert!(steps.iter().any(|s| s.branch == "LU"));
        let ax = crate::linalg::norm_func::matmul(&a, &x).unwrap();
        assert!(max_abs_diff(&ax, &b) < 1e-10);
    }

    #[test]
    fn tall_rectangular_is_least_squares() {
        // A = [1 0; 0 1; 1 1] (بلند)، b=[1;2;3] → LS جواب [1;2] (A·[1;2]=[1;2;3])
        let a = mat(3, 2, &[1.0, 0.0, 1.0, 0.0, 1.0, 1.0]);
        let b = mat(3, 1, &[1.0, 2.0, 3.0]);
        let (x, steps) = mldivide(&a, &b).unwrap();
        assert!(steps.iter().any(|s| s.branch == "QR_LS"));
        assert!(max_abs_diff(&x, &mat(2, 1, &[1.0, 2.0])) < 1e-10);
    }

    #[test]
    fn wide_rectangular_is_minimum_norm() {
        // A = [1 0 0; 0 1 0] (پهن)، b=[1;2] → کم-نُرم [1;2;0]
        let a = mat(2, 3, &[1.0, 0.0, 0.0, 1.0, 0.0, 0.0]);
        let b = mat(2, 1, &[1.0, 2.0]);
        let (x, steps) = mldivide(&a, &b).unwrap();
        assert!(steps.iter().any(|s| s.branch == "MIN_NORM"));
        assert!(max_abs_diff(&x, &mat(3, 1, &[1.0, 2.0, 0.0])) < 1e-10);
    }

    #[test]
    fn row_mismatch_is_size_error() {
        let a = mat(2, 2, &[1.0; 4]);
        let b = mat(3, 1, &[1.0; 3]);
        assert_eq!(mldivide(&a, &b).unwrap_err().id, ids::SIZE_MISMATCH);
    }

    #[test]
    fn exactly_singular_is_error_with_new_id() {
        let a = mat(2, 2, &[1.0, 2.0, 2.0, 4.0]);
        let b = mat(2, 1, &[1.0, 2.0]);
        let err = mldivide(&a, &b).unwrap_err();
        assert_eq!(err.id, ids::SINGULAR_MATRIX);
    }

    #[test]
    fn mrdivide_matches_transpose_identity() {
        // B/A = (Aᵀ\Bᵀ)ᵀ — مثال: [2 0]/[2] = [1 0]
        let b = mat(1, 2, &[2.0, 0.0]);
        let a = Matrix::scalar(2.0);
        let (x, _) = mrdivide(&b, &a).unwrap();
        assert!(max_abs_diff(&x, &mat(1, 2, &[1.0, 0.0])) < 1e-12);
    }
}
