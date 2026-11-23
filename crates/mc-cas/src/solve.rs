//! گام 4.3/CAS-2 — حل معادلات (سند ۰۵ §۸).
//!
//! `solve` همیشه فهرست برمی‌گرداند (پاریت متلب) و معادله را به شکل
//! `expr = 0` می‌پذیرد. مسیرها:
//!   * شکل‌های فراتر از جدول وارون‌ها (exp/log/sin/cos/tan با آرگومان خطی)
//!   * مسیر گویا: ریشه‌های صورت RatFun + گارد مخرج و دامنهٔ تعریف
//!     (ریشهٔ عقلانی با ارزیابی عددی اصلی — مثل ریشهٔ 0 در x·log(x) — حذف)
//!   * چندجمله‌ای: خطی، درجه ۲ (فرمول با گارد مختلط)، RRT + کاهش،
//!     کاردانو (مسیر رادیکال حقیقی؛ casus irreducibilis → عددی صادقانه)،
//!     بی‌درجه‌۲ (جایگذاری u = x²) و فراری با ریشهٔ رزولونت حقیقی
//!   * درجه ≥ ۵: پیام آموزشی Abel–Ruffini + اشارهٔ عددی roots()
//!   * دستگاه خطی: حذف گاوسی سمبولیک با کسرهای دقیق + پارامتری‌سازی
//!     فضای پوچی؛ دستگاه غیرخطی ۲×۲ با جایگذاری (سقف ترکیب)
//!
//! انضباط کیفیت: هر ریشهٔ برگردانده‌شده ورفای می‌شود — نمادین
//! (simplify(subs) = 0) یا عددی (|p(r)| کوچک نسبی). ریشهٔ رد‌شدهٔ تأیید
//! = خطای صادقانه، هرگز جواب غلطِ مطمئن‌نما. فرم‌های بستهٔ قضیه‌ای با
//! ضرایب پارامتری که هیچ‌کدام ممکن نیست، با اطمینان قضیه‌ای می‌مانند
//! (مستند در سند ۰۵ §۱۱).

use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::{Signed, Zero};

use crate::diff::diff;
use crate::int::{as_linear, contains, div_lin, sqrt_sym, try_rational_root, vkey};
use crate::poly::{poly_to_sym, Poly, RatFun};
use crate::simplify::simplify;
use crate::sym::{CasError, Sym};

/// سقف ترکیب جایگذاری در دستگاه غیرخطی ۲×۲.
const MAX_SUBST_PAIRS: usize = 8;

// ---------------------------------------------------------------------------
// گاوسی سمبولیک (زیرساخت مشترک solve و PF تجزیهٔ جزء به جزء)
// ---------------------------------------------------------------------------

/// نتیجهٔ حذف گاوسی روی ماتریس RatFun.
pub(crate) enum Gauss {
    /// راه یکتا (ردیف به ازای هر مجهول)
    Unique(Vec<RatFun>),
    /// حل خاص + پایهٔ فضای پوچی (ستون‌های آزاد)
    Parametric(Vec<RatFun>, Vec<Vec<RatFun>>),
    /// ناسازگار
    Inconsistent,
}

/// RREF روی ماتریس افزودهٔ n-مجهوله — دقیق با RatFun.
fn gauss_rref(a: &mut [Vec<RatFun>], b: &mut [RatFun], n: usize) -> Result<Gauss, CasError> {
    let m = a.len();
    if n == 0 || m == 0 {
        return Ok(Gauss::Inconsistent);
    }
    let mut rank = 0usize;
    for col in 0..n {
        // محور: اولین سطر با ضریب ناصفر
        let Some(piv) = (rank..m).find(|&r| !a[r][col].is_zero()) else {
            continue;
        };
        a.swap(rank, piv);
        b.swap(rank, piv);
        let pv = a[rank][col].clone();
        // نرمال‌سازی سطر محور: ضرب در pv⁻¹ = den/num (قبلاً فقط den
        // ضرب می‌شد و pv.num نادیده می‌ماند — جواب‌ها مقیاس غلط می‌گرفتند)
        let inv = RatFun {
            num: pv.den.clone(),
            den: pv.num.clone(),
        };
        for aj in a[rank].iter_mut() {
            *aj = aj.mul(&inv)?;
        }
        b[rank] = b[rank].mul(&inv)?;
        // حذف از همهٔ سطرهای دیگر
        for r in 0..m {
            if r == rank || a[r][col].is_zero() {
                continue;
            }
            let f = a[r][col].clone();
            let pivot_row = a[rank].clone();
            for (arow, pr) in a[r].iter_mut().zip(pivot_row.iter()) {
                let sub = pr.mul(&f)?;
                *arow = arow.sub(&sub)?;
            }
            let subb = b[rank].mul(&f)?;
            b[r] = b[r].sub(&subb)?;
        }
        rank += 1;
        if rank == m {
            break;
        }
    }
    // سازگاری: سطرهای صفر با rhs ناصفر
    for r in rank..m {
        if a[r].iter().all(|c| c.is_zero()) && !b[r].is_zero() {
            return Ok(Gauss::Inconsistent);
        }
    }
    if rank == n {
        // یکتا — استخراج از RREF
        let mut sol = vec![RatFun::zero(); n];
        for (r, arow) in a.iter().enumerate().take(rank) {
            if let Some(c) = (0..n).find(|&j| !arow[j].is_zero()) {
                sol[c] = b[r].clone();
            }
        }
        Ok(Gauss::Unique(sol))
    } else {
        // پارامتریک — ستون‌های بدون محور آزادند
        let piv_col_of: Vec<Option<usize>> = a
            .iter()
            .take(rank)
            .map(|arow| (0..n).find(|&j| !arow[j].is_zero()))
            .collect();
        let piv_cols: Vec<usize> = piv_col_of.iter().flatten().copied().collect();
        let free: Vec<usize> = (0..n).filter(|j| !piv_cols.contains(j)).collect();
        let mut particular = vec![RatFun::zero(); n];
        for (r, c) in piv_col_of.iter().enumerate() {
            if let Some(c) = c {
                particular[*c] = b[r].clone();
            }
        }
        let mut nullspace = vec![];
        for &f in &free {
            let mut v = vec![RatFun::zero(); n];
            v[f] = RatFun::from_poly(&Poly::one());
            for (r, c) in piv_col_of.iter().enumerate() {
                if let Some(c) = c {
                    v[*c] = a[r][f].clone().neg();
                }
            }
            nullspace.push(v);
        }
        Ok(Gauss::Parametric(particular, nullspace))
    }
}

/// گاوسی با پذیرش راه پارامتری — راهِ خاص برای ضرایب نامعین (گام 4.4/CAS-3).
/// ناسازگار → None؛ یکتا/پارامتریک → راه خاص (بخش همگنِ باقی در حدسِ
/// توسعه‌یافته به ثابت‌های C جواب همگن منتقل می‌شود و ورفای نهایی می‌گیرد).
pub(crate) fn gaussian_particular(a: &[Vec<Sym>], b: &[Sym]) -> Result<Option<Vec<Sym>>, CasError> {
    let m = a.len();
    let n = a.first().map(|r| r.len()).unwrap_or(0);
    if n == 0 || m == 0 || b.len() != m {
        return Ok(None);
    }
    let mut ar: Vec<Vec<RatFun>> = vec![];
    let mut br: Vec<RatFun> = vec![];
    for (i, row) in a.iter().enumerate() {
        let mut rrow = vec![];
        for c in row {
            rrow.push(RatFun::from_sym(c)?);
        }
        ar.push(rrow);
        br.push(RatFun::from_sym(&b[i])?);
    }
    let g = gauss_rref(&mut ar, &mut br, n)?;
    match g {
        Gauss::Unique(sol) | Gauss::Parametric(sol, _) => {
            let mut out = vec![];
            for s in sol {
                out.push(s.to_sym()?);
            }
            Ok(Some(out))
        }
        Gauss::Inconsistent => Ok(None),
    }
}

/// گاوسی با پذیرش راه پارامتری — راهِ خاص برای ضرایب نامعین (گام 4.4/CAS-3).
/// ناسازگار → None؛ یکتا/پارامتریک → راه خاص (بخش همگنِ باقی در حدسِ
/// حل دستگاه مربعی با راه یکتا — زیرساخت PF در int.rs.
pub(crate) fn gaussian_solve_symbolic(
    a: &[Vec<Sym>],
    b: &[Sym],
) -> Result<Option<Vec<Sym>>, CasError> {
    let m = a.len();
    let n = a.first().map(|r| r.len()).unwrap_or(0);
    if n == 0 || m < n || b.len() != m {
        return Ok(None);
    }
    let mut ar: Vec<Vec<RatFun>> = vec![];
    let mut br: Vec<RatFun> = vec![];
    for (i, row) in a.iter().enumerate() {
        let mut rrow = vec![];
        for c in row {
            rrow.push(RatFun::from_sym(c)?);
        }
        ar.push(rrow);
        br.push(RatFun::from_sym(&b[i])?);
    }
    let g = gauss_rref(&mut ar, &mut br, n)?;
    match g {
        Gauss::Unique(sol) => {
            let mut out = vec![];
            for s in sol {
                out.push(s.to_sym()?);
            }
            Ok(Some(out))
        }
        _ => Ok(None),
    }
}

// ---------------------------------------------------------------------------
// حل تک‌معادله
// ---------------------------------------------------------------------------

/// حل `eq = 0` در متغیر `var` — همیشه فهرست ریشه‌های حقیقی.
pub fn solve_expr(eq: &Sym, var: &str) -> Result<Vec<Sym>, CasError> {
    let e = simplify(eq)?;
    if !contains(&e, var) {
        return Err(CasError::Unsupported(
            "the equation does not contain the requested variable",
        ));
    }
    // ۱) شکل‌های فراتر از جدول وارون‌ها
    if let Some(sol) = try_transcendental(&e, var)? {
        return Ok(sol);
    }
    // گارد صداقت: معادلهٔ فراتر از جدول که وارون‌پذیر نشد را با مسیر
    // «گویا در ژنراتورهای اُپک» حل نمی‌کنیم — ریشه‌های ناقص/کاذب می‌داد
    // (درس طراحی: exp·sin = 1 به [] می‌رسید در حالی که حل دارد).
    if has_var_fn(&e, var) {
        return Err(CasError::Unsupported(
            "transcendental equation outside the invertible table — use numeric solvers (roots / fzero)",
        ));
    }
    // ۲) مسیر گویا (توابع/توان‌های غیرصحیح به‌عنوان ژنراتور اُپک)
    let rf = RatFun::from_sym(&e)?;
    if rf.num.is_zero() {
        return Err(CasError::Unsupported(
            "the equation reduces to 0 = 0 — every value of the variable solves it",
        ));
    }
    if rf.den.is_zero() {
        return Err(CasError::DivByZero);
    }
    let num_sym = poly_to_sym(&rf.num)?;
    let den_sym = poly_to_sym(&rf.den)?;
    let orig = e.clone();
    let mut roots = poly_roots(&rf.num, var)?;
    // گارد مخرج + دامنهٔ تعریف عبارت اصلی (ارزیابی عددی ریشه‌های عددی)
    roots.retain(|r| root_ok(&orig, &den_sym, r, var));
    // ورفای هر ریشه — ریشهٔ رد‌شده = خطای صادقانه
    let mut out = vec![];
    for r in roots {
        match verify_root(&num_sym, &orig, &r, var) {
            Some(true) => out.push(r),
            Some(false) => {
                return Err(CasError::Unsupported(
                    "root verification failed — a produced root did not check out (please report)",
                ))
            }
            None => out.push(r), // فرم بستهٔ قضیه‌ای با ضرایب پارامتری
        }
    }
    Ok(out)
}

/// ریشه، مخرج را صفر نمی‌کند و عبارت اصلی را بی‌تعریف نمی‌گذارد.
fn root_ok(orig: &Sym, den_sym: &Sym, root: &Sym, var: &str) -> bool {
    // مخرج: نمادین
    let dsub = den_sym.substitute(var, root);
    if let Ok(z) = simplify(&dsub) {
        if is_exact_zero(&z) {
            return false;
        }
        if !is_exact_nonzero(&z) {
            // نامشخص — عددی اگر ممکن
            if let (Ok(_rv), Ok(dv)) = (root.to_f64(), z.to_f64()) {
                if dv.abs() < 1e-9 {
                    return false;
                }
            }
        }
    }
    // دامنهٔ تعریف اصلی: ارزیابی عددی ریشه‌های کاملاً عددی
    if root.free_symbols().is_empty() {
        if let (Ok(_rv), Ok(v)) = (root.to_f64(), orig.substitute(var, root).to_f64()) {
            if !v.is_finite() {
                return false;
            }
        }
    }
    true
}

/// آیا درخت تابعی دارد که آرگومانش var است (فراتر از جبر)?
fn has_var_fn(e: &Sym, var: &str) -> bool {
    match e {
        Sym::Fn(_, args) => {
            args.iter().any(|a| contains(a, var)) || args.iter().any(|a| has_var_fn(a, var))
        }
        Sym::Add(ts) => ts.iter().any(|t| has_var_fn(t, var)),
        Sym::Mul(fs) => fs.iter().any(|f| has_var_fn(f, var)),
        Sym::Pow(b, ex) => has_var_fn(b, var) || has_var_fn(ex, var),
        _ => false,
    }
}

fn is_exact_zero(z: &Sym) -> bool {
    match z {
        Sym::Integer(i) => i.is_zero(),
        Sym::Rational(r) => r.is_zero(),
        _ => false,
    }
}

fn is_exact_nonzero(z: &Sym) -> bool {
    match z {
        Sym::Integer(i) => !i.is_zero(),
        Sym::Rational(r) => !r.is_zero(),
        _ => false,
    }
}

/// ورفای ریشه: Some(true/false) یا None (نامشخص — اعتماد قضیه‌ای).
fn verify_root(num_sym: &Sym, orig: &Sym, root: &Sym, var: &str) -> Option<bool> {
    // ۱) نمادین
    let sub = num_sym.substitute(var, root);
    if let Ok(z) = simplify(&sub) {
        if is_exact_zero(&z) {
            return Some(true);
        }
        if is_exact_nonzero(&z) {
            // هنوز می‌توان عددی هم آزمود — گام ۲
        }
    }
    // ۲) عددی (ریشهٔ کاملاً عددی)
    if root.free_symbols().is_empty() {
        if let (Ok(rv), Ok(v)) = (root.to_f64(), sub.to_f64()) {
            let scale = 1.0 + rv.abs().powi(4);
            return Some(v.abs() < 1e-6 * scale);
        }
    }
    let _ = orig;
    None
}

// ---------------------------------------------------------------------------
// ریشه‌های چندجمله‌ای
// ---------------------------------------------------------------------------

fn rat_of(s: &Sym) -> Option<BigRational> {
    match s {
        Sym::Integer(i) => Some(BigRational::from_integer(i.clone())),
        Sym::Rational(r) => Some(r.clone()),
        _ => None,
    }
}

/// ریشه‌های حقیقی چندجمله‌ای در `var` (ضرایب پارامتری فقط تا درجه ۲).
fn poly_roots(p: &Poly, var: &str) -> Result<Vec<Sym>, CasError> {
    let deg = p.degree_in(&vkey(var));
    match deg {
        0 => Ok(vec![]), // ثابت ناصفر — بی‌ریشه
        1 => {
            let a1 = coeff_sym(p, var, 1);
            let a0 = coeff_sym(p, var, 0);
            Ok(vec![Sym::over(Sym::negated(a0), a1)?])
        }
        2 => quadratic_roots(p, var),
        3 => cubic_roots(p, var),
        4 => quartic_roots(p, var),
        _ => Err(CasError::Unsupported(
            "degree 5+ has no general radical formula (Abel-Ruffini) — use roots() for numeric roots",
        )),
    }
}

fn coeff_sym(p: &Poly, var: &str, k: i64) -> Sym {
    let vk = vkey(var);
    let mut out = Poly::zero();
    for (m, c) in &p.terms {
        if m.get(&vk).copied().unwrap_or(0) == k {
            let mut mm = m.clone();
            mm.remove(&vk);
            let piece = Poly {
                terms: [(mm, c.clone())].into_iter().collect(),
            };
            out = out.add(&piece);
        }
    }
    RatFun {
        num: out,
        den: Poly::one(),
    }
    .to_sym()
    .unwrap_or_else(|_| Sym::int(0))
}

fn quadratic_roots(p: &Poly, var: &str) -> Result<Vec<Sym>, CasError> {
    let a2s = coeff_sym(p, var, 2);
    let a1s = coeff_sym(p, var, 1);
    let a0s = coeff_sym(p, var, 0);
    // Δ = a1² − 4a2a0
    let four_a2a0 = Sym::times(Sym::int(4), Sym::times(a2s.clone(), a0s.clone()));
    let delta = simplify(&Sym::minus(Sym::pow(a1s.clone(), Sym::int(2))?, four_a2a0))?;
    let two_a2 = Sym::plus(a2s.clone(), a2s.clone());
    let neg_a1 = Sym::negated(a1s.clone());
    if let Some(d) = rat_of(&delta) {
        if d < BigRational::zero() {
            return Err(CasError::Unsupported(
                "both roots are complex — symbolic complex numbers are a later phase; use roots() for numeric roots",
            ));
        }
        if d.is_zero() {
            return Ok(vec![Sym::over(neg_a1, two_a2)?]);
        }
        let s = sqrt_sym(&d)?;
        let mk = |s: Sym| -> Result<Sym, CasError> {
            Sym::over(Sym::plus(neg_a1.clone(), s), two_a2.clone())
        };
        Ok(vec![mk(s.clone())?, mk(Sym::negated(s))?])
    } else {
        // Δ نمادین — جفت رادیکالی (فرم قضیه‌ای؛ ورفای نمادین در solve_expr)
        let s = Sym::pow(delta, Sym::rat(1, 2)?)?;
        let mk = |sgn: i64| -> Result<Sym, CasError> {
            let num = if sgn == 1 {
                Sym::plus(neg_a1.clone(), s.clone())
            } else {
                Sym::minus(neg_a1.clone(), s.clone())
            };
            Sym::over(num, two_a2.clone())
        };
        Ok(vec![mk(1)?, mk(-1)?])
    }
}

/// ضرایب عددی (فقط مسیرهای درجه ۳/۴).
fn numeric_coeffs(p: &Poly, var: &str, deg: i64) -> Option<Vec<BigRational>> {
    let mut out = vec![];
    for k in (0..=deg).rev() {
        let c = coeff_sym(p, var, k);
        out.push(rat_of(&c)?);
    }
    Some(out)
}

fn cubic_roots(p: &Poly, var: &str) -> Result<Vec<Sym>, CasError> {
    // RRT اول — ریشه‌های گویا را بکش و درجه را پایین بیاور
    let (reduced, rational_roots) = rrt_reduce(p, var)?;
    let mut out = rational_roots;
    let rdeg = reduced.degree_in(&vkey(var));
    if rdeg == 0 {
        return Ok(out);
    }
    if rdeg > 1 && reduced.generators().iter().any(|g| g != &vkey(var)) {
        return Err(CasError::Unsupported(
            "cubic with symbolic coefficients is outside v1 — use roots() for numeric roots",
        ));
    }
    match rdeg {
        1 => {
            out.push(linear_root(&reduced, var)?);
            Ok(out)
        }
        2 => {
            out.extend(quadratic_roots(&reduced, var)?);
            Ok(out)
        }
        3 => {
            out.extend(cardano_real(&reduced, var)?);
            Ok(out)
        }
        _ => Err(CasError::Unsupported(
            "cubic reduction produced an unexpected degree",
        )),
    }
}

fn linear_root(p: &Poly, var: &str) -> Result<Sym, CasError> {
    let a1 = coeff_sym(p, var, 1);
    let a0 = coeff_sym(p, var, 0);
    Sym::over(Sym::negated(a0), a1)
}

/// کشیدن ریشه‌های گویا با RRT — (چندجمله‌ای کاهش‌یافته، ریشه‌ها).
fn rrt_reduce(p: &Poly, var: &str) -> Result<(Poly, Vec<Sym>), CasError> {
    let mut h = p.clone();
    let mut roots = vec![];
    let mut guard = 0;
    while h.degree_in(&vkey(var)) >= 1 {
        guard += 1;
        if guard > 8 {
            break;
        }
        let Some(r) = try_rational_root(&h, var) else {
            break;
        };
        let (q, rem) = div_lin(&h, &r, var);
        if !rem.is_zero() {
            break;
        }
        roots.push(Sym::Rational(r));
        h = q;
    }
    Ok((h, roots))
}

/// کاردانو — فقط مسیر رادیکال حقیقی (Δ ≥ 0).
fn cardano_real(p: &Poly, var: &str) -> Result<Vec<Sym>, CasError> {
    let Some(c) = numeric_coeffs(p, var, 3) else {
        return Err(CasError::Unsupported(
            "cubic with symbolic coefficients is outside v1 — use roots() for numeric roots",
        ));
    };
    let (a3, a2, a1, a0) = (&c[0], &c[1], &c[2], &c[3]);
    if a3.is_zero() {
        return Err(CasError::Unsupported("cubic path got a non-cubic"));
    }
    // تبدیل به حالت فروکاسته: x = t − a2/(3a3) ؛ t³ + p·t + q = 0
    let three = BigRational::from_integer(BigInt::from(3));
    let shift = a2 / (a3 * &three);
    let p_dep = (&three * a1 * a3 - a2 * a2) / (&three * a3 * a3);
    let q_dep = (BigRational::from_integer(BigInt::from(2)) * a2 * a2 * a2
        - BigRational::from_integer(BigInt::from(9)) * a2 * a1 * a3
        + BigRational::from_integer(BigInt::from(27)) * a0 * a3)
        / (BigRational::from_integer(BigInt::from(27)) * a3 * a3 * a3);
    let quarter = BigRational::new(BigInt::from(1), BigInt::from(4));
    let twenty7th = BigRational::new(BigInt::from(1), BigInt::from(27));
    let disc = &q_dep * &q_dep * quarter + &p_dep * &p_dep * &p_dep * twenty7th;
    let t_root = if disc < BigRational::zero() {
        return Err(CasError::Unsupported(
            "casus irreducibilis — three real irrational roots need complex radicals; use roots() for numeric roots",
        ));
    } else if disc.is_zero() {
        // ریشه‌های مضاعف — گویای صرف: ساده = 3q/p ، مضاعف = −3q/(2p)
        if p_dep.is_zero() {
            Sym::int(0) // سه‌تایی (q هم صفر است)
        } else {
            let two = BigRational::from_integer(BigInt::from(2));
            let t_simple = &three * &q_dep / &p_dep;
            let _ = &two;
            Sym::Rational(t_simple)
        }
    } else {
        // t = cbrt(−q/2 + √Δ) + cbrt(−q/2 − √Δ)
        let half_q = q_dep.clone() / BigRational::from_integer(BigInt::from(2));
        let sqrt_d = sqrt_sym(&disc)?;
        let mk_cbrt = |rad: &Sym| -> Result<Sym, CasError> {
            // علامت را عددی بیرون می‌کشیم — پایهٔ منفی با توان کسری ساخته نمی‌شود
            if let Ok(v) = rad.to_f64() {
                if v < 0.0 {
                    let pos = Sym::negated(rad.clone());
                    return Ok(Sym::negated(Sym::pow(pos, Sym::rat(1, 3)?)?));
                }
            }
            Sym::pow(rad.clone(), Sym::rat(1, 3)?)
        };
        let u = Sym::plus(Sym::Rational(-half_q.clone()), sqrt_d.clone());
        let v = Sym::minus(Sym::Rational(-half_q), sqrt_d);
        Sym::plus(mk_cbrt(&u)?, mk_cbrt(&v)?)
    };
    // x = t − shift — به‌همراه ریشهٔ مضاعف وقتی Δ = 0
    let shift_sym = Sym::Rational(shift);
    let mut roots = vec![Sym::minus(t_root, shift_sym.clone())];
    if disc.is_zero() && !p_dep.is_zero() {
        let two = BigRational::from_integer(BigInt::from(2));
        let t_double = -(&three * &q_dep / &(two * &p_dep));
        roots.push(Sym::minus(Sym::Rational(t_double), shift_sym));
    }
    Ok(roots)
}

// ---------------------------------------------------------------------------
// درجه ۴ — بی‌درجه‌۲ + فراری (رزولونت مکعبی مشتقشده، نه حفظی)
// ---------------------------------------------------------------------------

fn quartic_roots(p: &Poly, var: &str) -> Result<Vec<Sym>, CasError> {
    // RRT اول
    let (reduced, rational_roots) = rrt_reduce(p, var)?;
    let mut out = rational_roots;
    let rdeg = reduced.degree_in(&vkey(var));
    match rdeg {
        0 => return Ok(out),
        1 => {
            out.push(linear_root(&reduced, var)?);
            return Ok(out);
        }
        2 => {
            out.extend(quadratic_roots(&reduced, var)?);
            return Ok(out);
        }
        3 => {
            out.extend(cardano_real(&reduced, var)?);
            return Ok(out);
        }
        4 => {}
        _ => {
            return Err(CasError::Unsupported(
                "quartic with symbolic coefficients is outside v1 — use roots()",
            ))
        }
    }
    if reduced.generators().iter().any(|g| g != &vkey(var)) {
        return Err(CasError::Unsupported(
            "quartic with symbolic coefficients is outside v1 — use roots()",
        ));
    }
    let Some(c) = numeric_coeffs(&reduced, var, 4) else {
        return Err(CasError::Unsupported(
            "quartic with symbolic coefficients is outside v1 — use roots()",
        ));
    };
    let (a4, a3, a2, a1, a0) = (&c[0], &c[1], &c[2], &c[3], &c[4]);
    // بی‌درجه‌۲: توان‌های فرد صفر
    if a3.is_zero() && a1.is_zero() {
        // a4·u² + a2·u + a0 با u = x²
        let mut quad = Poly::zero();
        quad.terms.insert(monom(var, 2), a4.clone());
        quad.terms.insert(monom(var, 1), a2.clone());
        if !a0.is_zero() {
            quad.terms.insert(monom_empty(), a0.clone());
        }
        let u_roots = quadratic_roots(&quad, var)?;
        for u in u_roots {
            // x = ±√u — فقط حقیقی
            if let Some(ur) = rat_of(&u) {
                if ur < BigRational::zero() {
                    continue; // مختلط — حذف صادقانه
                }
                let s = sqrt_sym(&ur)?;
                out.push(s.clone());
                if !matches!(s, Sym::Integer(ref i) if i.is_zero()) {
                    out.push(Sym::negated(s));
                }
            }
        }
        return Ok(out);
    }
    // فراری: نرمال‌سازی x⁴ + b·x³ + c·x² + d·x + e
    let b = a3 / a4;
    let cc = a2 / a4;
    let d = a1 / a4;
    let e = a0 / a4;
    // فروکاسته با x = t − b/4: t⁴ + p·t² + q·t + r
    let p_dep = &cc
        - BigRational::from_integer(BigInt::from(3)) * &b * &b
            / BigRational::from_integer(BigInt::from(8));
    let q_dep = &d - &b * &cc / BigRational::from_integer(BigInt::from(2))
        + &b * &b * &b / BigRational::from_integer(BigInt::from(8));
    let r_dep = &e - &b * &d / BigRational::from_integer(BigInt::from(4))
        + &b * &b * &cc / BigRational::from_integer(BigInt::from(16))
        - BigRational::from_integer(BigInt::from(3)) * &b * &b * &b * &b
            / BigRational::from_integer(BigInt::from(256));
    if q_dep.is_zero() {
        // پس از فروکاست، بی‌درجه‌۲ در t
        let mut quad = Poly::zero();
        quad.terms
            .insert(monom(var, 2), BigRational::from_integer(BigInt::from(1)));
        quad.terms.insert(monom(var, 1), p_dep.clone());
        if !r_dep.is_zero() {
            quad.terms.insert(monom_empty(), r_dep.clone());
        }
        let t_roots = quadratic_roots(&quad, var)?;
        let shift = Sym::Rational(b / BigRational::from_integer(BigInt::from(4)));
        for t in t_roots {
            if let Some(tr) = rat_of(&t) {
                if tr < BigRational::zero() {
                    continue;
                }
                let s = sqrt_sym(&tr)?;
                out.push(Sym::minus(s.clone(), shift.clone()));
                if !matches!(s, Sym::Integer(ref i) if i.is_zero()) {
                    out.push(Sym::minus(Sym::negated(s), shift.clone()));
                }
            }
        }
        return Ok(out);
    }
    // رزولونت در z = α²:  z³ + 2p·z² + (p² − 4r)·z − q² = 0
    // (مشتق‌شده از هم‌نهادی (t²+αt+β)(t²−αt+γ))
    let two = BigRational::from_integer(BigInt::from(2));
    let four = BigRational::from_integer(BigInt::from(4));
    let mut resolvent = Poly::zero();
    resolvent
        .terms
        .insert(monom(var, 3), BigRational::from_integer(BigInt::from(1)));
    resolvent.terms.insert(monom(var, 2), &two * &p_dep);
    resolvent
        .terms
        .insert(monom(var, 1), &p_dep * &p_dep - four * &r_dep);
    let q2 = -(&q_dep * &q_dep);
    if !q2.is_zero() {
        resolvent.terms.insert(monom_empty(), q2);
    }
    let z_roots = cardano_real(&resolvent, var)?;
    // z باید ≥ 0 و عددی باشد
    let mut z_pos: Vec<BigRational> = vec![];
    for z in z_roots {
        if let Some(zr) = rat_of(&z) {
            if zr >= BigRational::zero() {
                z_pos.push(zr);
            }
        } else if let Ok(zf) = z.to_f64() {
            if zf >= 0.0 {
                // رادیکال — به‌صورت Sym نگه می‌داریم (پایین با α)
                z_pos.push(BigRational::zero()); // نشانه — در صورت نیاز
            }
        }
    }
    z_pos.retain(|z| !z.is_zero() || q_dep.is_zero());
    // فقط مسیر گویا (α = √z گویا) — رادیکال تودرتو v1 صادقانه رد
    let Some(z) = z_pos.iter().find(|z| !z.is_zero()).cloned() else {
        return Err(CasError::Unsupported(
            "quartic needs a nested-radical factorization — use roots() for numeric roots",
        ));
    };
    let alpha = match sqrt_sym(&z)? {
        Sym::Rational(r) => Sym::Rational(r),
        _ => {
            return Err(CasError::Unsupported(
                "quartic needs a nested-radical factorization — use roots() for numeric roots",
            ))
        }
    };
    // β = (p + z − q/α)/2 ، γ = (p + z + q/α)/2
    let q_over_a = &q_dep / rat_of(&alpha).unwrap_or_else(BigRational::zero);
    if rat_of(&alpha).map(|a| a.is_zero()).unwrap_or(true) {
        return Err(CasError::Unsupported("quartic factorization degenerated"));
    }
    let beta = (&p_dep + &z - &q_over_a) / &two;
    let gamma = (&p_dep + &z + &q_over_a) / &two;
    // دو معادلهٔ درجه ۲: t² + αt + β = 0 و t² − αt + γ = 0
    let mk_quad = |s1: i64, constt: &BigRational| -> Result<Vec<Sym>, CasError> {
        let mut quad = Poly::zero();
        quad.terms
            .insert(monom(var, 2), BigRational::from_integer(BigInt::from(1)));
        if s1 != 0 {
            let a_rat = rat_of(&alpha).unwrap_or_default();
            quad.terms.insert(
                monom(var, 1),
                a_rat * BigRational::from_integer(BigInt::from(s1)),
            );
        }
        if !constt.is_zero() {
            quad.terms.insert(monom_empty(), constt.clone());
        }
        quadratic_roots(&quad, var)
    };
    let shift = Sym::Rational(b / BigRational::from_integer(BigInt::from(4)));
    for t in mk_quad(1, &beta)?.into_iter().chain(mk_quad(-1, &gamma)?) {
        out.push(Sym::minus(t, shift.clone()));
    }
    Ok(out)
}

fn monom(var: &str, k: i64) -> crate::poly::Mon {
    let mut m = crate::poly::Mon::new();
    m.insert(vkey(var), k);
    m
}

fn monom_empty() -> crate::poly::Mon {
    crate::poly::Mon::new()
}

// ---------------------------------------------------------------------------
// شکل‌های فراتر از جدول وارون‌ها
// ---------------------------------------------------------------------------

/// الگوی k·fn(L) + C با L خطی در var و k، C x-آزاد.
fn try_transcendental(e: &Sym, var: &str) -> Result<Option<Vec<Sym>>, CasError> {
    let terms: Vec<&Sym> = match e {
        Sym::Add(ts) => ts.iter().collect(),
        _ => vec![e],
    };
    if terms.len() > 2 {
        return Ok(None);
    }
    let mut x_terms: Vec<&Sym> = vec![];
    let mut c_terms: Vec<&Sym> = vec![];
    for t in &terms {
        if contains(t, var) {
            x_terms.push(t);
        } else {
            c_terms.push(t);
        }
    }
    if x_terms.len() != 1 {
        return Ok(None);
    }
    let t = x_terms[0];
    // k·fn(L)
    let (k, fname, arg) = match t {
        Sym::Fn(n, args) if args.len() == 1 => (Sym::int(1), n.clone(), args[0].clone()),
        Sym::Mul(factors) => {
            let mut fn_fac: Option<(std::sync::Arc<str>, Sym)> = None;
            let mut consts: Vec<Sym> = vec![];
            for f in factors {
                if let Sym::Fn(n, args) = f {
                    if args.len() == 1 && contains(f, var) {
                        if fn_fac.is_some() {
                            return Ok(None); // دو تابع — خارج از الگو
                        }
                        fn_fac = Some((n.clone(), args[0].clone()));
                        continue;
                    }
                }
                if contains(f, var) {
                    return Ok(None); // ساختار غیر از k·fn(L)
                }
                consts.push(f.clone());
            }
            let Some((n, arg)) = fn_fac else {
                return Ok(None);
            };
            let k = if consts.is_empty() {
                Sym::int(1)
            } else {
                let mut it = consts.into_iter();
                let first = it.next().unwrap_or_else(|| Sym::int(1));
                it.fold(first, Sym::times)
            };
            (k, n, arg)
        }
        _ => return Ok(None),
    };
    if !matches!(&*fname, "exp" | "log" | "sin" | "cos" | "tan") {
        return Ok(None);
    }
    let Some((a, b)) = as_linear(&arg, var) else {
        return Ok(None);
    };
    let c = if c_terms.is_empty() {
        Sym::int(0)
    } else {
        let mut it = c_terms.into_iter();
        let first = it.next().cloned().unwrap_or_else(|| Sym::int(0));
        it.cloned().fold(first, Sym::plus)
    };
    // انتقال ثابت: k·fn(L) = −C → fn(L) = −C/k
    let rhs = Sym::over(Sym::negated(c.clone()), k.clone())?;
    let solve_lin = |inner: Sym| -> Result<Sym, CasError> {
        // a·x + b = inner → x = (inner − b)/a
        Sym::over(Sym::minus(inner, b.clone()), a.clone())
    };
    match &*fname {
        "exp" => {
            if is_exact_zero(&c) {
                return Ok(Some(vec![])); // exp(L) = 0 بی‌حل حقیقی
            }
            if let Some(rv) = rat_of(&rhs) {
                if rv <= BigRational::zero() {
                    return Ok(Some(vec![])); // لگاریتمِ نامثبت — بی‌حل حقیقی
                }
            }
            Ok(Some(vec![solve_lin(Sym::fn_of("log", vec![rhs])?)?]))
        }
        "log" => {
            let inner = Sym::fn_of("exp", vec![rhs])?;
            Ok(Some(vec![solve_lin(inner)?]))
        }
        "sin" | "cos" => {
            // |c| > 1 عددی → بی‌حل حقیقی
            if let Some(rv) = rat_of(&rhs) {
                if rv.abs() > BigRational::from_integer(BigInt::from(1)) {
                    return Ok(Some(vec![]));
                }
            }
            let pi = Sym::symbol("pi");
            let k_par = Sym::symbol("k");
            let two_pi_k = Sym::times(Sym::int(2), Sym::times(pi.clone(), k_par.clone()));
            if &*fname == "sin" {
                // asin(c) + 2πk یا π − asin(c) + 2πk
                let base = Sym::fn_of("asin", vec![rhs.clone()])?;
                let s1 = Sym::plus(base.clone(), two_pi_k.clone());
                let s2 = Sym::plus(Sym::minus(pi, base), two_pi_k);
                Ok(Some(vec![solve_lin(s1)?, solve_lin(s2)?]))
            } else {
                // ±acos(c) + 2πk
                let base = Sym::fn_of("acos", vec![rhs.clone()])?;
                let s1 = Sym::plus(base.clone(), two_pi_k.clone());
                let s2 = Sym::plus(Sym::negated(base), two_pi_k);
                Ok(Some(vec![solve_lin(s1)?, solve_lin(s2)?]))
            }
        }
        "tan" => {
            let pi = Sym::symbol("pi");
            let k_par = Sym::symbol("k");
            let base = Sym::fn_of("atan", vec![rhs])?;
            let family = Sym::plus(base, Sym::times(pi, k_par));
            Ok(Some(vec![solve_lin(family)?]))
        }
        _ => Ok(None),
    }
}

// ---------------------------------------------------------------------------
// دستگاه‌ها
// ---------------------------------------------------------------------------

/// حل دستگاه معادلات (expr = 0 هر کدام) در متغیرهای داده‌شده.
/// خروجی: یک Sym به ازای هر متغیر (حل یکتا یا پارامتریک با _t).
pub fn solve_system(eqs: &[Sym], vars: &[String]) -> Result<Vec<Sym>, CasError> {
    if eqs.is_empty() || vars.is_empty() {
        return Err(CasError::Unsupported(
            "solve needs at least one equation and one variable",
        ));
    }
    // استخراج ضرایب خطی
    let mut rows: Vec<Vec<Sym>> = vec![];
    let mut rhs: Vec<Sym> = vec![];
    let mut nonlinear = false;
    for eq in eqs {
        let e = simplify(eq)?;
        let mut row: Vec<Sym> = vec![];
        let mut lin = vec![];
        for v in vars {
            let c = simplify(&diff(&e, v)?)?;
            if vars.iter().any(|vv| contains(&c, vv)) {
                nonlinear = true;
            }
            row.push(c.clone());
            lin.push(Sym::times(c, Sym::symbol(v)));
        }
        let c0 = simplify(&Sym::minus(
            e.clone(),
            lin.into_iter().fold(Sym::int(0), Sym::plus),
        ))?;
        if vars.iter().any(|vv| contains(&c0, vv)) {
            nonlinear = true;
        }
        rows.push(row);
        rhs.push(Sym::negated(c0));
    }
    if nonlinear {
        return solve_nonlinear(eqs, vars);
    }
    // گاوسی سمبولیک
    let n = vars.len();
    let mut ar: Vec<Vec<RatFun>> = vec![];
    let mut br: Vec<RatFun> = vec![];
    for (i, row) in rows.iter().enumerate() {
        let mut rrow = vec![];
        for c in row {
            rrow.push(RatFun::from_sym(c)?);
        }
        ar.push(rrow);
        br.push(RatFun::from_sym(&rhs[i])?);
    }
    match gauss_rref(&mut ar, &mut br, n)? {
        Gauss::Inconsistent => Err(CasError::Unsupported(
            "the linear system is inconsistent — it has no solution",
        )),
        Gauss::Unique(sol) => {
            let out: Vec<Sym> = sol
                .iter()
                .map(|s| s.to_sym().unwrap_or_else(|_| Sym::int(0)))
                .collect();
            verify_system(eqs, vars, &out)?;
            Ok(out)
        }
        Gauss::Parametric(part, nullspace) => {
            let mut taken: Vec<String> = eqs.iter().flat_map(|e| e.free_symbols()).collect();
            taken.extend(vars.iter().cloned());
            let params: Vec<Sym> = (0..nullspace.len())
                .map(|i| fresh_param("_t", i, &taken))
                .collect();
            let mut out = vec![];
            for (i, p) in part.iter().enumerate() {
                let mut acc = p.to_sym()?;
                for (k, nv) in nullspace.iter().enumerate() {
                    let coeff = nv[i].to_sym()?;
                    acc = Sym::plus(acc, Sym::times(params[k].clone(), coeff));
                }
                out.push(acc);
            }
            verify_system(eqs, vars, &out)?;
            Ok(out)
        }
    }
}

fn fresh_param(base: &str, i: usize, taken: &[String]) -> Sym {
    for k in 0..64 {
        let name = format!("{base}{}", i + k);
        if !taken.contains(&name) {
            return Sym::symbol(&name);
        }
    }
    Sym::symbol(&format!("{base}_w"))
}

/// دستگاه غیرخطی — فقط ۲×۲ با جایگذاری؛ فقط جفت‌حل یکتا.
fn solve_nonlinear(eqs: &[Sym], vars: &[String]) -> Result<Vec<Sym>, CasError> {
    if eqs.len() != 2 || vars.len() != 2 {
        return Err(CasError::Unsupported(
            "nonlinear systems beyond two equations in two unknowns are outside v1",
        ));
    }
    // همهٔ جفت‌های (معادله، متغیر حل‌شده) — درس x²+y−5 با x−1:
    // فقط (eq1,x) دنبالهٔ بی‌متغیر می‌داد؛ (eq2,x) جواب را می‌یابد.
    let mut pairs: Vec<(Sym, Sym)> = vec![];
    for (ei, vi) in [(0usize, 0usize), (1, 0), (0, 1), (1, 1)] {
        if !contains(&eqs[ei], &vars[vi]) {
            continue;
        }
        let Ok(r1s) = solve_expr(&eqs[ei], &vars[vi]) else {
            continue;
        };
        let (oi, ovi) = (1 - ei, 1 - vi);
        for r in r1s.iter().take(4) {
            let e2 = simplify(&eqs[oi].substitute(&vars[vi], r))?;
            if contains(&e2, &vars[ovi]) {
                let Ok(r2s) = solve_expr(&e2, &vars[ovi]) else {
                    continue;
                };
                for s2 in r2s.iter().take(4) {
                    let pair = if vi == 0 {
                        (r.clone(), s2.clone())
                    } else {
                        (s2.clone(), r.clone())
                    };
                    pairs.push(pair);
                    if pairs.len() >= MAX_SUBST_PAIRS {
                        break;
                    }
                }
            } else if is_exact_zero(&e2) {
                // معادلهٔ دیگر بی‌متغیر و صفر — متغیر دوم آزاد نیست مشخص؛
                // این شاخه بی‌جفت است، جفت از شاخهٔ دیگر می‌آید
            }
        }
        if !pairs.is_empty() {
            break;
        }
    }
    // ورفای جفت‌ها روی هر دو معادله
    let mut good: Vec<(Sym, Sym)> = vec![];
    for (r, s) in pairs {
        let mut ok = true;
        for eq in eqs {
            let v = eq.substitute(&vars[0], &r).substitute(&vars[1], &s);
            let sv = simplify(&v);
            match sv {
                Ok(z) if is_exact_zero(&z) => {}
                Ok(_) => {
                    // نمادین صفر نشد — عددی
                    if let (Ok(rv), Ok(sv2), Ok(svv)) = (r.to_f64(), s.to_f64(), v.to_f64()) {
                        let scale = 1.0 + rv.abs() + svv.abs();
                        if !sv2.is_finite() || sv2.abs() > 1e-6 * scale {
                            ok = false;
                        }
                    } else {
                        ok = false;
                    }
                }
                Err(_) => ok = false,
            }
        }
        if ok {
            good.push((r, s));
        }
    }
    match good.len() {
        0 => Ok(vec![]),
        1 => Ok(vec![good[0].0.clone(), good[0].1.clone()]),
        _ => Err(CasError::Unsupported(
            "multiple solution pairs — v1 returns only unique system solutions; solve each variable separately",
        )),
    }
}

/// ورفای حل دستگاه: جایگذاری در همهٔ معادلات باید صفر شود.
fn verify_system(eqs: &[Sym], vars: &[String], sol: &[Sym]) -> Result<(), CasError> {
    for eq in eqs {
        let mut v = eq.clone();
        for (i, var) in vars.iter().enumerate() {
            v = v.substitute(var, &sol[i]);
        }
        let sv = simplify(&v)?;
        if is_exact_zero(&sv) {
            continue;
        }
        // نمادین صفر نشد — عددی اگر ممکن
        if sv.free_symbols().is_empty() {
            if let Ok(nv) = sv.to_f64() {
                if nv.abs() < 1e-6 {
                    continue;
                }
            }
        }
        return Err(CasError::Unsupported(
            "system solution verification failed (please report)",
        ));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// تست‌ها
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit::{int, sym};

    fn roots_display(expr: &Sym, var: &str) -> String {
        let rs = solve_expr(expr, var).expect("solve");
        rs.iter()
            .map(|r| r.display())
            .collect::<Vec<_>>()
            .join(" ; ")
    }

    #[test]
    fn linear_symbolic() {
        // 3x + 6 = 0 → x = −2
        let e = Sym::plus(Sym::times(Sym::int(3), sym("x")), Sym::int(6));
        assert_eq!(roots_display(&e, "x"), "-2");
    }

    #[test]
    fn linear_with_params() {
        // a·x + b = 0 → x = −b/a
        let e = Sym::plus(Sym::times(sym("a"), sym("x")), sym("b"));
        let d = roots_display(&e, "x");
        assert!(d.contains("b"), "got {d}");
    }

    #[test]
    fn quadratic_rational_roots() {
        // x² − 5x + 6 = 0 → 2 و 3
        let e = Sym::plus(
            Sym::pow(sym("x"), Sym::int(2)).expect("pow"),
            Sym::times(Sym::int(-5), sym("x")),
        );
        let e = Sym::plus(e, Sym::int(6));
        let d = roots_display(&e, "x");
        assert!(d.contains("2") && d.contains("3"), "got {d}");
    }

    #[test]
    fn quadratic_radical_roots() {
        // x² − 2 = 0 → ±√2
        let e = Sym::minus(Sym::pow(sym("x"), Sym::int(2)).expect("pow"), Sym::int(2));
        let rs = solve_expr(&e, "x").expect("solve");
        assert_eq!(rs.len(), 2);
        for r in &rs {
            let v = r.to_f64().expect("f64");
            assert!((v.abs() - 2f64.sqrt()).abs() < 1e-12, "got {v}");
        }
    }

    #[test]
    fn quadratic_complex_honest_error() {
        // x² + 1 = 0 → مختلط — خطای صادقانه
        let e = Sym::plus(Sym::pow(sym("x"), Sym::int(2)).expect("pow"), Sym::int(1));
        assert!(solve_expr(&e, "x").is_err());
    }

    #[test]
    fn rational_guard_removes_pole() {
        // (x² − 1)/(x − 1) = 0 → فقط x = −1 (ریشهٔ 1 مخرج را صفر می‌کند)
        let num = Sym::minus(Sym::pow(sym("x"), Sym::int(2)).expect("pow"), Sym::int(1));
        let e = Sym::over(num, Sym::minus(sym("x"), Sym::int(1))).expect("over");
        let rs = solve_expr(&e, "x").expect("solve");
        assert_eq!(rs.len(), 1);
        assert_eq!(rs[0].display(), "-1");
    }

    #[test]
    fn domain_guard_log() {
        // x·log(x) = 0 → فراتر از جدول وارون‌ها (الگوی k·fn(L) نیست) →
        // خطای صادقانه، نه ریشهٔ 0 که دامنه را می‌شکند
        let e = Sym::times(sym("x"), Sym::fn_of("log", vec![sym("x")]).expect("log"));
        assert!(solve_expr(&e, "x").is_err());
    }

    #[test]
    fn transcendental_exp() {
        // 2·exp(3x + 1) − 4 = 0 → x = (log(2) − 1)/3
        let inner = Sym::plus(Sym::times(Sym::int(3), sym("x")), Sym::int(1));
        let e = Sym::minus(
            Sym::times(Sym::int(2), Sym::fn_of("exp", vec![inner]).expect("exp")),
            Sym::int(4),
        );
        let rs = solve_expr(&e, "x").expect("solve");
        assert_eq!(rs.len(), 1);
        // ورفای عددی: 2·exp(3x+1) = 4
        let v = rs[0].to_f64().expect("f64");
        assert!((2.0 * (3.0 * v + 1.0).exp() - 4.0).abs() < 1e-9);
    }

    #[test]
    fn transcendental_log() {
        // log(2x + 3) − 1 = 0 → x = (e − 3)/2
        let inner = Sym::plus(Sym::times(Sym::int(2), sym("x")), Sym::int(3));
        let e = Sym::minus(Sym::fn_of("log", vec![inner]).expect("log"), Sym::int(1));
        let rs = solve_expr(&e, "x").expect("solve");
        assert_eq!(rs.len(), 1);
        let v = rs[0].to_f64().expect("f64");
        assert!((v - ((std::f64::consts::E - 3.0) / 2.0)).abs() < 1e-9);
    }

    #[test]
    fn transcendental_sin_family() {
        // sin(x) − 1/2 = 0 → دو شاخه با پارامتر k
        let e = Sym::minus(
            Sym::fn_of("sin", vec![sym("x")]).expect("sin"),
            Sym::rat(1, 2).expect("rat"),
        );
        let rs = solve_expr(&e, "x").expect("solve");
        assert_eq!(rs.len(), 2);
        // نماد pi و k باید در فرمول‌ها باشند
        assert!(rs[0].free_symbols().iter().any(|s| s == "k"));
    }

    #[test]
    fn cubic_rational_reduction() {
        // x³ − 6x² + 11x − 6 = 0 → 1، 2، 3
        let x2 = Sym::pow(sym("x"), Sym::int(2)).expect("pow");
        let x3 = Sym::pow(sym("x"), Sym::int(3)).expect("pow");
        let e = Sym::plus(
            x3,
            Sym::plus(
                Sym::times(Sym::int(-6), x2),
                Sym::plus(Sym::times(Sym::int(11), sym("x")), Sym::int(-6)),
            ),
        );
        let d = roots_display(&e, "x");
        assert!(
            d.contains('1') && d.contains('2') && d.contains('3'),
            "got {d}"
        );
    }

    #[test]
    fn quartic_biquadratic() {
        // x⁴ − 2x² − 3 = 0 → ±√3
        let x2 = Sym::pow(sym("x"), Sym::int(2)).expect("pow");
        let x4 = Sym::pow(sym("x"), Sym::int(4)).expect("pow");
        let e = Sym::plus(x4, Sym::plus(Sym::times(Sym::int(-2), x2), Sym::int(-3)));
        let rs = solve_expr(&e, "x").expect("solve");
        assert_eq!(rs.len(), 2, "got {rs:?}");
        for r in &rs {
            let v = r.to_f64().expect("f64");
            assert!((v.abs() - 3f64.sqrt()).abs() < 1e-12);
        }
    }

    #[test]
    fn degree5_abel_ruffini_honest() {
        let x5 = Sym::pow(sym("x"), Sym::int(5)).expect("pow");
        let e = Sym::minus(x5, Sym::int(1));
        let err = solve_expr(&e, "x").unwrap_err();
        assert!(matches!(err, CasError::Unsupported(_)));
    }

    #[test]
    fn linear_system_unique() {
        // x + y − 3 = 0 ، x − y − 1 = 0 → x = 2، y = 1
        let e1 = Sym::plus(Sym::plus(sym("x"), sym("y")), Sym::int(-3));
        let e2 = Sym::minus(Sym::minus(sym("x"), sym("y")), Sym::int(1));
        let sol = solve_system(&[e1, e2], &["x".into(), "y".into()]).expect("sys");
        assert_eq!(sol[0].display(), "2");
        assert_eq!(sol[1].display(), "1");
    }

    #[test]
    fn linear_system_parametric() {
        // x + y − 1 = 0 (زیرتعیین) → y = 1 − x با پارامتر
        let e1 = Sym::plus(Sym::plus(sym("x"), sym("y")), Sym::int(-1));
        let sol = solve_system(std::slice::from_ref(&e1), &["x".into(), "y".into()]).expect("sys");
        // ورفای: جایگذاری در معادله صفر شود
        let v = e1.substitute("x", &sol[0]).substitute("y", &sol[1]);
        assert!(is_exact_zero(&simplify(&v).expect("simplify")));
    }

    #[test]
    fn linear_system_inconsistent() {
        // x − 1 = 0 ، x − 2 = 0
        let e1 = Sym::minus(sym("x"), Sym::int(1));
        let e2 = Sym::minus(sym("x"), Sym::int(2));
        assert!(solve_system(&[e1, e2], &["x".into()]).is_err());
    }

    #[test]
    fn nonlinear_2x2_substitution() {
        // x + y − 3 = 0 ، x·y − 2 = 0 → جفت‌ها (2,1) و (1,2) — چندجفتی →
        // خطای صادقانه؛ اما معادلهٔ طراحی‌شده با جفت یکتا:
        // x + y − 5 = 0 ، x − y − 1 = 0 خطی است؛ غیرخطیِ یکتا:
        // x² + y − 5 = 0 ، x − 1 = 0 → (1, 4)
        let e1 = Sym::plus(
            Sym::pow(sym("x"), Sym::int(2)).expect("pow"),
            Sym::minus(sym("y"), Sym::int(5)),
        );
        let e2 = Sym::minus(sym("x"), Sym::int(1));
        let sol = solve_system(&[e1, e2], &["x".into(), "y".into()]).expect("sys");
        assert_eq!(sol[0].display(), "1");
        assert_eq!(sol[1].display(), "4");
    }

    #[test]
    fn gaussian_unique_core() {
        // x + y = 3 ، x − y = 1
        let a = vec![
            vec![Sym::int(1), Sym::int(1)],
            vec![Sym::int(1), Sym::int(-1)],
        ];
        let b = vec![Sym::int(3), Sym::int(1)];
        let sol = gaussian_solve_symbolic(&a, &b)
            .expect("gauss")
            .expect("unique");
        assert_eq!(sol[0].display(), "2");
        assert_eq!(sol[1].display(), "1");
    }

    #[test]
    fn equation_without_var_honest_error() {
        let e = Sym::plus(int(2), int(3));
        assert!(solve_expr(&e, "x").is_err());
    }
}
