//! تجزیه‌های جبر خطی — facade روی faer (سند ۰۴ §3.1).
//!
//! جدول تجزیه‌های نسخه ۱ (سند ۰۴ §3.1): LU با pivot جزئی، Cholesky، QR با pivoting ستونی،
//! SVD کامل، Eigen متقارن/عمومی. همه خروجی‌ها به `Matrix` ستونی mc-core برمی‌گردند.

use crate::linalg::convert;
use crate::linalg::num_error;
use faer::prelude::*;
use faer::Mat;
use mc_core::error::ids;
use mc_core::matrix::Matrix;

/// [L,U,P] = lu(A) با پیموت جزئی ردیفی؛ قرارداد متلب: `P·A = L·U`.
///
/// - L: واحد-مثلثی پایین (m×min(m,n))، U: مثلثی بالا (min(m,n)×n)، P: پرموتاسیون m×m.
/// - برای A مربع، همان [L,U,P] کلاسیک متلب است (مثلاً lu([1 2;3 4])).
pub fn lu(a: &Matrix) -> Result<(Matrix, Matrix, Matrix), mc_core::error::McError> {
    let (m, n) = (a.rows(), a.cols());
    if a.is_empty() {
        // ابعاد صفر: L تهی m×0، U تهی 0×n، P همانی تهی m×m — بدون فراخوانی faer.
        let l = Matrix::full(m, 0, 0.0);
        let u = Matrix::full(0, n, 0.0);
        let p = convert::identity(m);
        return Ok((l, u, p));
    }
    let af = convert::to_faer(a);
    let lu = af.partial_piv_lu();
    let l = convert::from_faer(&lu.L().to_owned());
    let u = convert::from_faer(&lu.U().to_owned());
    // P(i, fwd[i]) = 1  ⟹  (P·A)[i] = A[fwd[i]]  ⟹  P·A = L·U (قرارداد solve در faer).
    let (fwd, _) = lu.P().arrays();
    let mut p = Matrix::full(m, m, 0.0);
    for (i, &src) in fwd.iter().enumerate() {
        p.set(i, src, 1.0);
    }
    Ok((l, u, p))
}

/// det(A) با LU پیموت جزئی — علامت از تعداد جابه‌جایی‌های پرموتاسیون (سند ۰۴ §3.1).
///
/// متلب: det([1 2;3 4]) = -2؛ det([]) = 1؛ ماتریس تکین دقیقاً → 0.
pub fn det(a: &Matrix) -> Result<f64, mc_core::error::McError> {
    convert::require_square(a, "det", "det")?;
    let n = a.rows();
    if n == 0 {
        return Ok(1.0);
    }
    if n == 1 {
        return Ok(a.get(0, 0));
    }
    let af = convert::to_faer(a);
    let lu = af.partial_piv_lu();
    // علامت پرموتاسیون σ با σ(i) = fwd[i]: det(P) = (-1)^{n - تعداد دورها}
    let (fwd, _) = lu.P().arrays();
    let sign = permutation_sign(fwd);
    let mut product = 1.0f64;
    for i in 0..n {
        product *= lu.U()[(i, i)];
    }
    Ok(sign * product)
}

/// علامت پرموتاسیون (۰-پایه) از تجزیه به دورها: هر دور به طول L، L-1 جابه‌جایی.
pub(crate) fn permutation_sign(perm: &[usize]) -> f64 {
    let n = perm.len();
    let mut seen = vec![false; n];
    let mut transpositions = 0usize;
    for start in 0..n {
        if seen[start] {
            continue;
        }
        let mut len = 0usize;
        let mut cur = start;
        while !seen[cur] {
            seen[cur] = true;
            cur = perm[cur];
            len += 1;
        }
        transpositions += len - 1;
    }
    if transpositions.is_multiple_of(2) {
        1.0
    } else {
        -1.0
    }
}

/// inv(A): حل LU روی ماتریس همانی (سند ۰۴ §3.1) — مربع اجباری.
///
/// پیوت دقیقاً صفر → خطای `SINGULAR_MATRIX` (سند ۰۴ §9: نتیجه هرگز silent-مشکوک نیست).
pub fn inv(a: &Matrix) -> Result<Matrix, mc_core::error::McError> {
    convert::require_square(a, "inv", "inv")?;
    let n = a.rows();
    if n == 0 {
        return Ok(Matrix::full(0, 0, 0.0));
    }
    let af = convert::to_faer(a);
    let lu = af.partial_piv_lu();
    if (0..n).any(|i| lu.U()[(i, i)] == 0.0) {
        return Err(num_error(
            ids::SINGULAR_MATRIX,
            "Matrix is singular to working precision (zero pivot in LU); inv is undefined.",
            "ماتریس در دقت کاری تکین است (پیوت صفر در LU)؛ inv تعریف نمی‌شود.",
        )
        .with_suggestion("check rcond/cond"));
    }
    let eye = Mat::<f64>::identity(n, n);
    let x = lu.solve(eye);
    Ok(convert::from_faer(&x))
}

/// [Q,R] = qr(A) — QR کامل متلب: Q همانی‌چرخش m×m، R مثلثی/ذوزنقه‌ای m×n.
pub fn qr(a: &Matrix) -> Result<(Matrix, Matrix), mc_core::error::McError> {
    let (m, n) = (a.rows(), a.cols());
    if a.is_empty() {
        return Ok((convert::identity(m), Matrix::full(m, n, 0.0)));
    }
    let af = convert::to_faer(a);
    let qr = af.qr();
    let q = convert::from_faer(&qr.compute_Q());
    // faer فقط بلوک بالایی R را نگه می‌دارد (min(m,n)×n)؛ R کامل متلب m×n است.
    let r = expand_r(qr.R(), m, n);
    Ok((q, r))
}

/// [Q,R,E] = qr(A) با pivoting ستونی — قرارداد متلب: `A·E = Q·R` (E پرموتاسیون ستونی n×n).
pub fn qr_pivoted(a: &Matrix) -> Result<(Matrix, Matrix, Matrix), mc_core::error::McError> {
    let (m, n) = (a.rows(), a.cols());
    if a.is_empty() {
        return Ok((
            convert::identity(m),
            Matrix::full(m, n, 0.0),
            convert::identity(n),
        ));
    }
    let af = convert::to_faer(a);
    let qr = af.col_piv_qr();
    let q = convert::from_faer(&qr.compute_Q());
    let r = expand_r(qr.R(), m, n);
    // E(fwd[i], i) = 1  ⟹  ستون i از A·E برابر ستون fwd[i] از A (قرارداد permute_cols در faer).
    let (fwd, _) = qr.P().arrays();
    let mut e = Matrix::full(n, n, 0.0);
    for (i, &src) in fwd.iter().enumerate() {
        e.set(src, i, 1.0);
    }
    Ok((q, r, e))
}

/// بلوک بالایی R در faer (min(m,n)×n) را به R کامل m×n متلب گسترش می‌دهد (زیرش صفر).
fn expand_r(r_top: faer::MatRef<'_, f64>, m: usize, n: usize) -> Matrix {
    let size = m.min(n);
    let mut out = Matrix::full(m, n, 0.0);
    for c in 0..n {
        for r_i in 0..size {
            out.set(r_i, c, r_top[(r_i, c)]);
        }
    }
    out
}

/// chol(A): ضریب پایین‌مثلثی L با `L·Lᵀ = A` — معادل chol(A,'lower') متلب.
///
/// A باید متقارن مثبت معین باشد؛ در غیر این صورت خطای `NOT_POSITIVE_DEFINITE`
/// (پیام متلب: «Matrix must be positive definite»).
pub fn chol(a: &Matrix) -> Result<Matrix, mc_core::error::McError> {
    convert::require_square(a, "chol", "chol")?;
    if a.is_empty() {
        return Ok(Matrix::full(0, 0, 0.0));
    }
    if !convert::is_exactly_symmetric(a) {
        return Err(not_spd());
    }
    let af = convert::to_faer(a);
    match af.llt(faer::Side::Lower) {
        // MATLAB/Octave convention: chol returns the UPPER triangular R with R'·R = A
        // (caught by the Octave differential job — numpy's cholesky is lower and the
        // first oracle draft silently followed it). faer llt gives L; return Lᵀ = R.
        Ok(llt) => Ok(convert::transpose(&convert::from_faer(&llt.L().to_owned()))),
        Err(_) => Err(not_spd()),
    }
}

fn not_spd() -> mc_core::error::McError {
    num_error(
        ids::NOT_POSITIVE_DEFINITE,
        "Matrix must be symmetric positive definite.",
        "ماتریس باید متقارنِ مثبت معین باشد.",
    )
    .with_suggestion("ensure A is symmetric with positive eigenvalues (see eig_sym)")
}

/// [U,S,V] = svd(A) — SVD کامل: U همانی‌چرخش m×m، مقادیر تکین نزولی ≥ 0، V همانی‌چرخش n×n.
pub fn svd(a: &Matrix) -> Result<(Matrix, Vec<f64>, Matrix), mc_core::error::McError> {
    let (m, n) = (a.rows(), a.cols());
    if a.is_empty() {
        return Ok((convert::identity(m), Vec::new(), convert::identity(n)));
    }
    let af = convert::to_faer(a);
    let svd = af.svd().map_err(|_| {
        num_error(
            ids::UNSUPPORTED,
            "SVD did not converge for this matrix.",
            "SVD برای این ماتریس همگرا نشد.",
        )
    })?;
    let u = convert::from_faer(&svd.U().to_owned());
    let v = convert::from_faer(&svd.V().to_owned());
    let s: Vec<f64> = svd.S().column_vector().iter().copied().collect();
    Ok((u, s, v))
}

/// [λ,V] = eig_sym(A) برای A متقارن حقیقی — مقادیر ویژه **صعودی** (رفتار متلب/LAPACK dsyev)
/// و بردارهای ویژه نرمال‌شده (ستون‌های V، برابر A·V[:,k] = λ_k·V[:,k]).
///
/// ورودی نامتقارن → خطا با راهنمای رفتن به `eig` (که خودش برای متقارن‌ها همین‌جا می‌رود).
pub fn eig_sym(a: &Matrix) -> Result<(Vec<f64>, Matrix), mc_core::error::McError> {
    convert::require_square(a, "eig_sym", "eig_sym")?;
    let n = a.rows();
    if n == 0 {
        return Ok((Vec::new(), Matrix::full(0, 0, 0.0)));
    }
    if !convert::is_exactly_symmetric(a) {
        return Err(num_error(
            ids::UNSUPPORTED,
            "eig_sym requires an exactly symmetric matrix; for general matrices use eig \
             (complex support arrives in P2).",
            "eig_sym به تقارن دقیق نیاز دارد؛ برای ماتریس‌های عمومی از eig استفاده کنید \
             (پشتیبانی مختلط در P2 می‌آید).",
        ));
    }
    let af = convert::to_faer(a);
    let eig = af.self_adjoint_eigen(faer::Side::Lower).map_err(|_| {
        num_error(
            ids::UNSUPPORTED,
            "symmetric eigen decomposition did not converge.",
            "تجزیه ویژه متقارن همگرا نشد.",
        )
    })?;
    let values: Vec<f64> = eig.S().column_vector().iter().copied().collect();
    let vectors = convert::from_faer(&eig.U().to_owned());
    Ok((values, vectors))
}

/// [λ,V] = eig(A) — eig عمومی حقیقی (سند ۰۴ §3.1: مسیر دوگانه متقارن/عمومی).
///
/// - ورودی متقارن → خودکار به `eig_sym` مسیر می‌شود (مقادیر صعودی).
/// - عمومی: فرم شرور حقیقی (real Schur) در faer؛ اگر زوج مختلطی وجود داشته باشد،
///   در P1 نوع Value مختلط نداریم → خطای `UNSUPPORTED` با پیام آموزشی مشخص.
/// - اگر همه مقادیر ویژه حقیقی باشند: مقادیر به ترتیب Schur (as-is، مثل متلب) و
///   بردارهای ویژه حقیقی برگردانده می‌شوند.
pub fn eig(a: &Matrix) -> Result<(Vec<f64>, Matrix), mc_core::error::McError> {
    convert::require_square(a, "eig", "eig")?;
    if convert::is_exactly_symmetric(a) {
        return eig_sym(a);
    }
    let n = a.rows();
    if n == 0 {
        return Ok((Vec::new(), Matrix::full(0, 0, 0.0)));
    }
    let af = convert::to_faer(a);
    // evd_real: فرم شرور حقیقی → S_re/S_im و بردارهای ویژه حقیقی (بلوک‌های ۲×۲ → جفت مختلط)
    let e = af.eigen().map_err(|_| {
        num_error(
            ids::UNSUPPORTED,
            "eigen decomposition did not converge for this matrix.",
            "تجزیه ویژه برای این ماتریس همگرا نشد.",
        )
    })?;
    let values_re: Vec<f64> = e.S().column_vector().iter().map(|c| c.re).collect();
    let values_im: Vec<f64> = e.S().column_vector().iter().map(|c| c.im).collect();
    if values_im.iter().any(|im| *im != 0.0) {
        return Err(num_error(
            ids::UNSUPPORTED,
            "complex eigenvalues arrive with P2 complex support; eig of symmetric matrices \
             is available.",
            "مقادیر ویژه مختلط با پشتیبانی مختلط P2 می‌آیند؛ eig ماتریس‌های متقارن در دسترس است.",
        )
        .with_suggestion("use eig_sym for symmetric matrices"));
    }
    // همه مقادیر ویژه حقیقی‌اند → ستون‌های U حقیقی‌اند (بخش موهومی دقیقاً صفر).
    let mut vectors = Matrix::full(n, n, 0.0);
    for c in 0..n {
        for r in 0..n {
            vectors.set(r, c, e.U()[(r, c)].re);
        }
    }
    Ok((values_re, vectors))
}

/// eig کامل مختلط (P2 — سند ۰۳ §12): مقادیر و بردارهای ویژه مختلط برای ماتریس عمومی.
///
/// - متقارن → همان `eig_sym` (مقادیر صعودی، بردارهای حقیقی در پوشِ مختلط).
/// - عمومی → فرم شرور حقیقی faer؛ زوج‌های مختلط به‌صورت (re، im) برگردانده می‌شوند.
/// - مصرف‌ها: `roots` (ماتریس همبند) و `[V,D] = eig` (رجیستری P2).
pub fn eig_complex(
    a: &Matrix,
) -> Result<(Vec<(f64, f64)>, mc_core::ComplexMatrix), mc_core::error::McError> {
    convert::require_square(a, "eig", "eig")?;
    let n = a.rows();
    if n == 0 {
        return Ok((Vec::new(), mc_core::ComplexMatrix::full(0, 0, 0.0, 0.0)));
    }
    if convert::is_exactly_symmetric(a) {
        let (vals, vecs) = eig_sym(a)?;
        return Ok((
            vals.into_iter().map(|v| (v, 0.0)).collect(),
            mc_core::ComplexMatrix::from_real(&vecs),
        ));
    }
    let af = convert::to_faer(a);
    let e = af.eigen().map_err(|_| {
        num_error(
            ids::UNSUPPORTED,
            "eigen decomposition did not converge for this matrix.",
            "تجزیه ویژه برای این ماتریس همگرا نشد.",
        )
    })?;
    let values: Vec<(f64, f64)> = e.S().column_vector().iter().map(|c| (c.re, c.im)).collect();
    let mut re = vec![0.0f64; n * n];
    let mut im = vec![0.0f64; n * n];
    for c in 0..n {
        for r in 0..n {
            re[c * n + r] = e.U()[(r, c)].re;
            im[c * n + r] = e.U()[(r, c)].im;
        }
    }
    Ok((values, mc_core::ComplexMatrix::from_columns(n, n, re, im)))
}
