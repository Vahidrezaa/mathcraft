//! حد نمادین — گام 4.4/CAS-3 (سند ۰۵ §۶).
//!
//! پنج مسیر به ترتیب: (۱) جای‌گذاری مستقیم روی فرم ساده‌شده؛ (۲) گویای
//! دقیق — حذف (x−a) مشترک برای ۰/۰ و تحلیل علامت قطب و مقایسهٔ درجه در
//! ±∞ (همه دقیق گویا)؛ (۳) لُوپیتال با بودجهٔ ۳ برای ۰/۰ و ∞/∞ (شامل
//! تبدیل ۰·∞ به خارج‌قسمت)؛ (۴) تبدیل لگاریتمی توان‌ها `B^E →
//! exp(lim E·ln B)` — ثابت `e` در درخت ما نمادین است (`exp(1)` — مثل
//! π-گونه‌ها، حدس عددی جعل است)؛ (۵) کاوش عددی چندمقیاسی با تشخیص
//! همگرایی/واگرایی/نوسان — DNE پاسخِ درست است نه خطا (سند ۰۵ §۶ مسیر ۴).
//!
//! مسیرهای «تیلور → جملهٔ ثابت» و «لُوپیتال» سند در اجرا ادغام شدند
//! (لُوپیتال همان تطبیق جملهٔ رأس است). نتیجهٔ عددی همیشه `Real` است —
//! تقریبِ صادقانه، هرگز گویای قلبی.

use crate::diff::diff;
use crate::int::{contains, is_sym_zero};
use crate::simplify::simplify;
use crate::sym::{CasError, Sym};
use crate::verify::eval_at;

/// جهت حد.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Side {
    /// دوطرفه (پیش‌فرض متلب) — چپ و راست باید همگرا باشند وگرنه DNE.
    TwoSided,
    FromLeft,
    FromRight,
}

/// نقطهٔ حد.
#[derive(Clone, Debug)]
pub enum LimitPoint {
    /// نقطهٔ متناهی (می‌تواند نمادین باشد).
    At(Sym),
    PosInf,
    NegInf,
}

/// نتیجهٔ حد — DNE پاسخِ درست است نه خطا (نوسان/چپ-راست متفاوت).
#[derive(Clone, Debug, PartialEq)]
pub enum LimitOutcome {
    /// مقدار — دقیق (گویا/نمادین) یا تقریبی صادقانه (`Real`).
    Value(Sym),
    PosInf,
    NegInf,
    /// وجود ندارد + توضیح (متن موتور انگلیسی؛ لایهٔ بالا دوزبانه می‌کند).
    DoesNotExist(String),
}

/// حد f در x→point با جهت side.
pub fn limit_expr(
    f: &Sym,
    x: &str,
    point: &LimitPoint,
    side: Side,
) -> Result<LimitOutcome, CasError> {
    if !contains(f, x) {
        return Ok(LimitOutcome::Value(f.clone()));
    }
    match point {
        LimitPoint::At(a) => limit_at(f, x, a, side),
        LimitPoint::PosInf => limit_inf(f, x, 1.0),
        LimitPoint::NegInf => limit_inf(f, x, -1.0),
    }
}

// ---------------------------------------------------------------------------
// نقطهٔ متناهی
// ---------------------------------------------------------------------------

fn limit_at(f: &Sym, x: &str, a: &Sym, side: Side) -> Result<LimitOutcome, CasError> {
    // ۱) جای‌گذاری مستقیم روی فرم ساده‌شده — حذف gcd در simplify خودش
    //    ۰/0های جداشدنی گویا (مثل (x²−1)/(x−1)) را می‌بندد
    let g = simplify(f)?;
    if &g != f {
        if let Some(v) = direct_value(&g, x, a)? {
            return Ok(LimitOutcome::Value(v));
        }
    }
    if let Some(v) = direct_value(f, x, a)? {
        return Ok(LimitOutcome::Value(v));
    }
    // ۲) گویای دقیق: ۰/۰ باقیمانده (رادیکالی که gcd نمادین نمی‌بیند) و قطب
    if let Some(out) = rational_at(f, x, a, side)? {
        return Ok(out);
    }
    // ۳) لُوپیتال (بودجهٔ ۳) — ۰/۰ و ∞/∞ و ۰·∞ (با تبدیل خارج‌قسمت)
    if let Some(out) = lhopital(f, x, a, side, 3)? {
        return Ok(out);
    }
    // ۴) تبدیل لگاریتمی توان‌ها: B^E → exp(lim E·ln B)
    if let Some(out) = pow_log(f, x, a, side)? {
        return Ok(out);
    }
    // ۵) کاوش عددی چندمقیاسی + تشخیص نوسان
    numeric_at(f, x, a, side)
}

/// جای‌گذاری سازنده‌محور — هر خطای ساخت پس از جای‌گذاری (مثل `0^−1` در
/// exp(1/x) هنگام x→0) → None، برخلاف `substitute` خام که زیردرخت را
/// دست‌نخورده نگه می‌دارد و fold صفرِ `0·exp(1/0)` مقدارِ غلطِ ۰ می‌سازد
/// (درس حدِ e^x/x² در +∞).
fn safe_substitute(e: &Sym, x: &str, with: &Sym) -> Option<Sym> {
    match e {
        Sym::Symbol(s) if &**s == x => Some(with.clone()),
        Sym::Integer(_) | Sym::Rational(_) | Sym::Real(_) | Sym::Symbol(_) => Some(e.clone()),
        Sym::Add(ts) => ts
            .iter()
            .map(|t| safe_substitute(t, x, with))
            .collect::<Option<Vec<_>>>()
            .and_then(|v| v.into_iter().reduce(Sym::plus)),
        Sym::Mul(fs) => fs
            .iter()
            .map(|t| safe_substitute(t, x, with))
            .collect::<Option<Vec<_>>>()
            .and_then(|v| v.into_iter().reduce(Sym::times)),
        Sym::Pow(b, ex) => {
            Sym::pow(safe_substitute(b, x, with)?, safe_substitute(ex, x, with)?).ok()
        }
        Sym::Fn(name, args) => {
            let new_args = args
                .iter()
                .map(|t| safe_substitute(t, x, with))
                .collect::<Option<Vec<_>>>()?;
            Sym::fn_of(name, new_args).ok()
        }
    }
}

/// جای‌گذاری مستقیم — فقط اگر مقدار بدون x و متناهی (یا نمادینِ سالم) بدهد.
fn direct_value(f: &Sym, x: &str, a: &Sym) -> Result<Option<Sym>, CasError> {
    let v = match safe_substitute(f, x, a) {
        Some(v) => match simplify(&v) {
            Ok(sv) => sv,
            Err(_) => return Ok(None), // 0/0 و تکین — صادقانه رد
        },
        None => return Ok(None), // ساختِ نود تعریف‌نشده پس از جای‌گذاری
    };
    if contains(&v, x) {
        return Ok(None);
    }
    // ارزیابی عددی برای چک تعریف‌شدگی: log(0)-گونه‌ها NaN می‌دهند؛ نماد
    // مانده (نقطهٔ نمادین) قابل چک نیست ولی جای‌گذاری مشروع است.
    match v.to_f64() {
        Ok(t) if t.is_finite() => Ok(Some(v)),
        Ok(_) => Ok(None), // NaN/±Inf → تعریف‌نشده در نقطه
        Err(_) => Ok(Some(v)),
    }
}

/// مسیر گویا در نقطهٔ متناهی — ۰/۰ (حذف (x−a)) و قطب (علامت دو طرف).
fn rational_at(f: &Sym, x: &str, a: &Sym, side: Side) -> Result<Option<LimitOutcome>, CasError> {
    let Some((num, den)) = as_quotient(f)? else {
        return Ok(None);
    };
    let (Some(na), Some(da)) = (safe_substitute(&num, x, a), safe_substitute(&den, x, a)) else {
        return Ok(None); // تعریف‌نشده در نقطه — مسیر عددی
    };
    let num_zero = match simplify(&na) {
        Ok(v) => is_sym_zero(&v),
        Err(_) => return Ok(None),
    };
    let den_zero = match simplify(&da) {
        Ok(v) => is_sym_zero(&v),
        Err(_) => return Ok(None),
    };
    if !den_zero {
        // مخرج سالم — مسیر ۱ باید گرفته باشد؛ اینجا فقط صورتِ صفر
        return if num_zero {
            Ok(Some(LimitOutcome::Value(Sym::int(0))))
        } else {
            Ok(None)
        };
    }
    if num_zero {
        // ۰/۰ باقیمانده: تا ۸ بار تقسیم نمادین بر (x−a)
        let mut n = num.clone();
        let mut d = den.clone();
        for _ in 0..8 {
            let (n2, rn) = match poly_div_sym(&n, x, a) {
                Ok(v) => v,
                Err(_) => break, // چندجمله‌ای نیست — مسیر بعدی
            };
            let (d2, rd) = match poly_div_sym(&d, x, a) {
                Ok(v) => v,
                Err(_) => break,
            };
            if !is_sym_zero(&rn) || !is_sym_zero(&rd) {
                break;
            }
            n = n2;
            d = d2;
        }
        if let Ok(q) = Sym::over(n, d) {
            if let Some(v) = direct_value(&q, x, a)? {
                return Ok(Some(LimitOutcome::Value(v)));
            }
        }
        return Ok(None);
    }
    // قطب: علامت f در دو سمت نقطه — تحلیل عددی روی عبارت اصلی
    let (lp, rp) = pole_signs(f, x, a);
    let combine = |s: f64| -> LimitOutcome {
        if s > 0.0 {
            LimitOutcome::PosInf
        } else {
            LimitOutcome::NegInf
        }
    };
    let (l_out, r_out) = (lp.map(combine), rp.map(combine));
    Ok(Some(match side {
        Side::FromLeft => match l_out.or(r_out) {
            Some(o) => o,
            None => return Err(CasError::Unsupported("pole sign could not be determined")),
        },
        Side::FromRight => match r_out.or(l_out) {
            Some(o) => o,
            None => return Err(CasError::Unsupported("pole sign could not be determined")),
        },
        Side::TwoSided => match (l_out, r_out) {
            (Some(l), Some(r)) if l == r => l,
            (Some(l), Some(r)) => LimitOutcome::DoesNotExist(format!(
                "left-hand limit is {l:?} but right-hand limit is {r:?}"
            )),
            (Some(o), None) | (None, Some(o)) => o,
            (None, None) => return Err(CasError::Unsupported("pole sign could not be determined")),
        },
    }))
}

/// علامت عددی f در دو سمت a — None یعنی آن سمت قابل ارزیابی نبود.
fn pole_signs(f: &Sym, x: &str, a: &Sym) -> (Option<f64>, Option<f64>) {
    let Some(t) = a.to_f64().ok().filter(|v| v.is_finite()) else {
        return (None, None);
    };
    let probe = |dir: f64| -> Option<f64> {
        for k in 3..9u32 {
            let at = t + dir * 10f64.powi(-(k as i32));
            if let Ok(v) = eval_at(f, x, at) {
                if v.is_finite() && v != 0.0 {
                    return Some(v.signum());
                }
            }
        }
        None
    };
    (probe(-1.0), probe(1.0))
}

/// خارج‌قسمت کانونی؟ `Mul(N, Pow(D, −1))` یا خودِ `Pow(D, −1)` → `(N, D)`.
fn as_quotient(f: &Sym) -> Result<Option<(Sym, Sym)>, CasError> {
    if let Sym::Mul(factors) = f {
        let mut rest: Vec<Sym> = Vec::new();
        let mut den: Option<Sym> = None;
        for fac in factors {
            if let Sym::Pow(b, e) = fac {
                if matches!(e.as_ref(), Sym::Integer(i) if *i == num_bigint::BigInt::from(-1))
                    && den.is_none()
                {
                    den = Some(b.as_ref().clone());
                    continue;
                }
            }
            rest.push(fac.clone());
        }
        if let Some(d) = den {
            let n = rest
                .into_iter()
                .reduce(Sym::times)
                .unwrap_or_else(|| Sym::int(1));
            return Ok(Some((n, d)));
        }
        return Ok(None);
    }
    // over(1, D) کانونی بدون Mul است: خودِ Pow(D, −1) → صورت ۱
    if let Sym::Pow(b, e) = f {
        if matches!(e.as_ref(), Sym::Integer(i) if *i == num_bigint::BigInt::from(-1)) {
            return Ok(Some((Sym::int(1), b.as_ref().clone())));
        }
    }
    Ok(None)
}

/// تقسیم چندجمله‌ای نمادین p در x بر (x − a): خارج‌قسمت + باقیمانده.
/// ضرایب با تفکیک دنباله‌ای (مشتق در ۰) جدا می‌شوند؛ غیرچندجمله‌ای → خطا.
fn poly_div_sym(p: &Sym, x: &str, a: &Sym) -> Result<(Sym, Sym), CasError> {
    let Some(n) = poly_degree_in(p, x) else {
        return Err(CasError::Unsupported(
            "not a polynomial in the limit variable",
        ));
    };
    if n < 1 {
        return Ok((Sym::int(0), p.clone()));
    }
    // ضرایب c_k با جدا کردن جمله‌ها از درجهٔ بالا به پایین
    let mut coeffs_desc: Vec<Sym> = Vec::new();
    let mut rem = p.clone();
    for k in (0..=n).rev() {
        let c = coeff_at(&rem, x, k)?;
        rem = simplify(&Sym::minus(
            rem.clone(),
            Sym::times(c.clone(), Sym::pow(Sym::symbol(x), Sym::int(k))?),
        ))?;
        coeffs_desc.push(c);
    }
    if contains(&rem, x) || !is_sym_zero(&rem) {
        return Err(CasError::Unsupported(
            "not a polynomial in the limit variable",
        ));
    }
    // coeffs_desc = [c_n, …, c_0] — تقسیم سینتی بر (x − a)
    let mut b: Vec<Sym> = Vec::new(); // [b_n, b_{n−1}, …, b_0] باقیمانده=b_0
    let mut carry = coeffs_desc[0].clone();
    b.push(carry.clone());
    for c in coeffs_desc.iter().skip(1) {
        carry = simplify(&Sym::plus(Sym::times(carry, a.clone()), c.clone()))?;
        b.push(carry.clone());
    }
    let rem_out = b.pop().unwrap_or_else(|| Sym::int(0));
    // خارج‌قسمت: b_n x^{n−1} + … + b_1 — b از بالا به پایین
    let mut out = Sym::int(0);
    for (i, c) in b.iter().rev().enumerate() {
        out = Sym::plus(
            out,
            Sym::times(c.clone(), Sym::pow(Sym::symbol(x), Sym::int(i as i64))?),
        );
    }
    Ok((simplify(&out)?, simplify(&rem_out)?))
}

/// ضریب جملهٔ x^k در p — مشتق k باری در 0 تقسیم بر k!.
fn coeff_at(p: &Sym, x: &str, k: i64) -> Result<Sym, CasError> {
    let mut d = p.clone();
    for _ in 0..k {
        d = diff(&d, x)?;
    }
    simplify(&d.substitute(x, &Sym::int(0)))
}

/// درجهٔ x در عبارت چندجمله‌ای (ضرایب پارامتری مجاز) — غیرچندجمله‌ای → None.
fn poly_degree_in(e: &Sym, x: &str) -> Option<i64> {
    fn walk(e: &Sym, x: &str, cur: i64, best: &mut i64) {
        match e {
            Sym::Add(ts) | Sym::Mul(ts) => {
                for t in ts {
                    walk(t, x, cur, best);
                }
            }
            Sym::Pow(b, ex) => {
                if let Some(n) = small_int(ex) {
                    walk(b, x, cur * n, best);
                }
            }
            Sym::Symbol(s) if &**s == x => *best = (*best).max(cur),
            _ => {}
        }
    }
    if !is_poly(e, x) {
        return None;
    }
    let mut best: i64 = -1;
    walk(e, x, 1, &mut best);
    Some(best)
}

fn small_int(e: &Sym) -> Option<i64> {
    match e {
        Sym::Integer(i) => i.to_string().parse::<i64>().ok(),
        _ => None,
    }
}

fn is_poly(e: &Sym, _x: &str) -> bool {
    match e {
        Sym::Integer(_) | Sym::Rational(_) | Sym::Real(_) | Sym::Symbol(_) => true,
        Sym::Add(ts) | Sym::Mul(ts) => ts.iter().all(|t| is_poly(t, _x)),
        Sym::Pow(b, ex) => matches!(small_int(ex), Some(n) if n >= 0) && is_poly(b, _x),
        _ => false,
    }
}

// ---------------------------------------------------------------------------
// لُوپیتال
// ---------------------------------------------------------------------------

fn lhopital(
    f: &Sym,
    x: &str,
    a: &Sym,
    side: Side,
    budget: u8,
) -> Result<Option<LimitOutcome>, CasError> {
    if budget == 0 {
        return Ok(None);
    }
    // ۰·∞ → خارج‌قسمت: g·h با g→0 و h→∞ → g/(1/h) (یا وارونه)
    if let Sym::Mul(factors) = f {
        if factors.len() == 2 {
            let (g, h) = (&factors[0], &factors[1]);
            let near = probe_point(a, 1.0);
            if let (Ok(gv), Ok(hv)) = (eval_at(g, x, near), eval_at(h, x, near)) {
                let zero_inf =
                    (gv.abs() < 1e-9 && hv.abs() > 1e9) || (hv.abs() < 1e-9 && gv.abs() > 1e9);
                if zero_inf {
                    let (z, big) = if gv.abs() < 1e-9 { (g, h) } else { (h, g) };
                    if let Ok(inv) = Sym::over(Sym::int(1), big.clone()) {
                        let q = Sym::times(z.clone(), inv);
                        if let Some(out) = lhopital(&q, x, a, side, budget)? {
                            return Ok(Some(out));
                        }
                    }
                }
            }
        }
    }
    let Some((num, den)) = as_quotient(f)? else {
        return Ok(None);
    };
    // ۰/۰؟ — نمادین دقیق در خود نقطه (صورت و مخرج هر دو صفر)
    let na = safe_substitute(&num, x, a).and_then(|v| simplify(&v).ok());
    let da = safe_substitute(&den, x, a).and_then(|v| simplify(&v).ok());
    let zero_zero = matches!((&na, &da), (Some(n), Some(d)) if is_sym_zero(n) && is_sym_zero(d));
    // ∞/∞؟ — کرانی عددی در چند مقیاس درشت‌به‌ریز (1e-2 … 1e-7)؛ آستانهٔ
    // 1e6 تا exp-مانندها (سیگموئید) پیش از سرریز f64 گرفته شوند
    let inf_inf = (|| {
        for k in 2..8u32 {
            // probe_point خودش ×1e-6 می‌کند → برای فاصلهٔ 10^{-k} باید
            // 10^{6-k} بدهیم (درس سیگموئید: نمونه در 1e-10 سرریز می‌شد)
            let p = probe_point(a, 10f64.powi(6 - (k as i32)));
            if let (Ok(nn), Ok(dd)) = (eval_at(&num, x, p), eval_at(&den, x, p)) {
                if nn.is_finite() && dd.is_finite() && nn.abs() > 1e6 && dd.abs() > 1e6 {
                    return true;
                }
            }
        }
        false
    })();
    if !zero_zero && !inf_inf {
        return Ok(None);
    }
    let (dn, ddn) = (diff(&num, x)?, diff(&den, x)?);
    if is_sym_zero(&simplify(&dn)?) && is_sym_zero(&simplify(&ddn)?) {
        return Ok(None);
    }
    let Ok(q) = Sym::over(dn, ddn) else {
        return Ok(None);
    };
    if let Some(v) = direct_value(&q, x, a)? {
        return Ok(Some(LimitOutcome::Value(v)));
    }
    if let Some(out) = rational_at(&q, x, a, side)? {
        return Ok(Some(out));
    }
    if let Some(out) = lhopital(&q, x, a, side, budget - 1)? {
        return Ok(Some(out));
    }
    Ok(None)
}

fn probe_point(a: &Sym, dir: f64) -> f64 {
    match a.to_f64() {
        Ok(t) if t.is_finite() => t + dir * 1e-6,
        _ => 1e-6 * dir,
    }
}

// ---------------------------------------------------------------------------
// تبدیل لگاریتمی توان‌ها
// ---------------------------------------------------------------------------

fn pow_log(f: &Sym, x: &str, a: &Sym, side: Side) -> Result<Option<LimitOutcome>, CasError> {
    let (b, e) = match f {
        Sym::Pow(b, e) => (b.as_ref().clone(), e.as_ref().clone()),
        _ => return Ok(None),
    };
    // پایه باید سمت حد مثبت باشد (لگاریتم حقیقی)
    let near = probe_point(a, 1.0);
    let Ok(bv) = eval_at(&b, x, near) else {
        return Ok(None);
    };
    if !(bv.is_finite() && bv > 0.0) {
        return Ok(None);
    }
    // lim B^E = exp(lim E·ln B)
    let Some(ln_b) = Sym::fn_of("log", vec![b]).ok() else {
        return Ok(None);
    };
    let inner = Sym::times(e, ln_b);
    let out = limit_expr(&inner, x, &LimitPoint::At(a.clone()), side)?;
    Ok(Some(match out {
        LimitOutcome::Value(v) => LimitOutcome::Value(Sym::fn_of("exp", vec![v])?),
        LimitOutcome::PosInf => LimitOutcome::PosInf,
        LimitOutcome::NegInf => LimitOutcome::Value(Sym::int(0)),
        LimitOutcome::DoesNotExist(w) => LimitOutcome::DoesNotExist(w),
    }))
}

// ---------------------------------------------------------------------------
// مسیر عددی
// ---------------------------------------------------------------------------

fn numeric_at(f: &Sym, x: &str, a: &Sym, side: Side) -> Result<LimitOutcome, CasError> {
    let Some(av) = a.to_f64().ok().filter(|t| t.is_finite()) else {
        return Ok(LimitOutcome::DoesNotExist(
            "the point is symbolic — numeric exploration is not applicable".into(),
        ));
    };
    let explore = |dir: f64| -> Result<LimitOutcome, CasError> {
        let mut vals: Vec<f64> = Vec::new();
        let mut last_inf: Option<f64> = None;
        for k in 2..16u32 {
            let at = av + dir * 10f64.powi(-(k as i32));
            match eval_at(f, x, at) {
                Ok(v) if v.is_finite() => vals.push(v),
                Ok(v) if v.is_infinite() => last_inf = Some(v),
                _ => {}
            }
        }
        if vals.is_empty() {
            if let Some(s) = last_inf {
                return Ok(if s > 0.0 {
                    LimitOutcome::PosInf
                } else {
                    LimitOutcome::NegInf
                });
            }
            return Ok(LimitOutcome::DoesNotExist(
                "the function is undefined on this side of the point".into(),
            ));
        }
        // دمِ نزدیک‌ترین مقادیر
        let tail: Vec<f64> = vals.iter().rev().take(4).cloned().collect();
        // واگرایی: |v| بزرگ و رشدکننده به سمت نقطه
        if tail.iter().all(|v| v.abs() > 1e6)
            && tail.windows(2).all(|w| w[0].abs() > w[1].abs() * 0.9)
        {
            let s = tail[0].signum();
            return Ok(if s > 0.0 {
                LimitOutcome::PosInf
            } else {
                LimitOutcome::NegInf
            });
        }
        // همگرایی: سه مقادیر آخر در تلورانس نسبی؛ فقط دو مقدار finite
        // مانده (اشباع‌شدگی مثل e^x/(e^x+1) در ۰⁺) با تلورانس سخت‌گیرانه‌تر
        if vals.len() >= 3 {
            let n = vals.len();
            let (v1, v2, v3) = (vals[n - 3], vals[n - 2], vals[n - 1]);
            let tol = |p: f64, q: f64| (p - q).abs() <= 1e-9 * (1.0 + p.abs().max(q.abs()));
            if tol(v1, v2) && tol(v2, v3) {
                return Ok(LimitOutcome::Value(Sym::Real(v3)));
            }
        } else if vals.len() == 2 {
            let (v1, v2) = (vals[0], vals[1]);
            if (v1 - v2).abs() <= 1e-12 * (1.0 + v1.abs().max(v2.abs())) {
                return Ok(LimitOutcome::Value(Sym::Real(v2)));
            }
        }
        // نوسان: دامنهٔ دم بزرگ و علامت‌های متناوب
        let lo = tail.iter().cloned().fold(f64::INFINITY, f64::min);
        let hi = tail.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let alternating = tail.windows(2).all(|w| w[0] * w[1] < 0.0);
        if alternating && hi - lo > 1e-6 {
            return Ok(LimitOutcome::DoesNotExist(format!(
                "numeric exploration: oscillation between roughly {lo:.6} and {hi:.6}"
            )));
        }
        Ok(LimitOutcome::DoesNotExist(
            "numeric exploration found no convergence".into(),
        ))
    };
    match side {
        Side::FromLeft => explore(-1.0),
        Side::FromRight => explore(1.0),
        Side::TwoSided => {
            let l = explore(-1.0)?;
            let r = explore(1.0)?;
            Ok(match (l, r) {
                (LimitOutcome::Value(a1), LimitOutcome::Value(a2)) => {
                    let (v1, v2) = (a1.to_f64(), a2.to_f64());
                    match (v1, v2) {
                        (Ok(p), Ok(q)) if (p - q).abs() <= 1e-7 * (1.0 + p.abs().max(q.abs())) => {
                            LimitOutcome::Value(a1)
                        }
                        (Ok(p), Ok(q)) => LimitOutcome::DoesNotExist(format!(
                            "left and right numeric limits differ: {p:?} vs {q:?}"
                        )),
                        _ => LimitOutcome::DoesNotExist(
                            "left and right numeric limits are not comparable".into(),
                        ),
                    }
                }
                (LimitOutcome::Value(_), _) | (_, LimitOutcome::Value(_)) => {
                    LimitOutcome::DoesNotExist("one side converges, the other does not".into())
                }
                (o1, o2) if o1 == o2 => o1,
                (o1, o2) => LimitOutcome::DoesNotExist(format!(
                    "left-hand outcome {o1:?} vs right-hand {o2:?}"
                )),
            })
        }
    }
}

// ---------------------------------------------------------------------------
// ±∞
// ---------------------------------------------------------------------------

fn limit_inf(f: &Sym, x: &str, dir: f64) -> Result<LimitOutcome, CasError> {
    // گویای دقیق: مقایسهٔ درجه‌ها (ضرایب پارامتری مجاز — علامت باید معلوم شود)
    if let Some(out) = rational_inf(f, x, dir)? {
        return Ok(out);
    }
    // تبدیل t = 1/x → حد در ۰ (سمت راست برای +∞، چپ برای −∞)
    if let Ok(inv) = Sym::over(Sym::int(1), Sym::symbol(x)) {
        let g = simplify(&f.substitute(x, &inv))?;
        let side = if dir > 0.0 {
            Side::FromRight
        } else {
            Side::FromLeft
        };
        return limit_expr(&g, x, &LimitPoint::At(Sym::int(0)), side);
    }
    // عددی: x = dir·10^k
    numeric_inf(f, x, dir)
}

fn rational_inf(f: &Sym, x: &str, dir: f64) -> Result<Option<LimitOutcome>, CasError> {
    let Some((num, den)) = as_quotient(f)? else {
        return Ok(None);
    };
    let (Some(dn), Some(dd)) = (poly_degree_in(&num, x), poly_degree_in(&den, x)) else {
        return Ok(None);
    };
    if dn == 0 && dd == 0 {
        return Ok(None); // هر دو ثابت — مسیر مستقیم
    }
    if dd > dn {
        return Ok(Some(LimitOutcome::Value(Sym::int(0))));
    }
    let Some(ratio) = Sym::over(coeff_at(&num, x, dn)?, coeff_at(&den, x, dd)?).ok() else {
        return Ok(None);
    };
    if dd == dn {
        return Ok(Some(LimitOutcome::Value(simplify(&ratio)?)));
    }
    // dn > dd → قطب در بی‌نهایت — علامت نسبت رأس (× (−1)^(dn−dd) برای −∞)
    let s = match simplify(&ratio)?.to_f64() {
        Ok(v) if v.is_finite() && v != 0.0 => v.signum(),
        _ => return Ok(None), // علامت پارامتری نامعلوم — مسیر عددی/صادقانه
    };
    let mut sign = s;
    if dir < 0.0 && (dn - dd) % 2 == 1 {
        sign = -sign;
    }
    Ok(Some(if sign > 0.0 {
        LimitOutcome::PosInf
    } else {
        LimitOutcome::NegInf
    }))
}

fn numeric_inf(f: &Sym, x: &str, dir: f64) -> Result<LimitOutcome, CasError> {
    let mut vals: Vec<f64> = Vec::new();
    for k in 1..15u32 {
        let at = dir * 10f64.powi(k as i32);
        if let Ok(v) = eval_at(f, x, at) {
            if v.is_finite() {
                vals.push(v);
            }
        }
    }
    let n = vals.len();
    if n >= 3 {
        let (v1, v2, v3) = (vals[n - 3], vals[n - 2], vals[n - 1]);
        let tol = |p: f64, q: f64| (p - q).abs() <= 1e-8 * (1.0 + p.abs().max(q.abs()));
        if tol(v1, v2) && tol(v2, v3) {
            return Ok(LimitOutcome::Value(Sym::Real(v3)));
        }
    } else if n == 2
        && (vals[0] - vals[1]).abs() <= 1e-12 * (1.0 + vals[0].abs().max(vals[1].abs()))
    {
        return Ok(LimitOutcome::Value(Sym::Real(vals[1])));
    }
    if n >= 2 && vals.iter().rev().take(3).all(|v| v.abs() > 1e6) {
        let s = vals[n - 1].signum();
        return Ok(if s > 0.0 {
            LimitOutcome::PosInf
        } else {
            LimitOutcome::NegInf
        });
    }
    Ok(LimitOutcome::DoesNotExist(
        "numeric exploration at infinity found no convergence".into(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sym_display;

    fn sym(n: &str) -> Sym {
        Sym::symbol(n)
    }

    fn lim(f: &Sym, a: Sym) -> LimitOutcome {
        limit_expr(f, "x", &LimitPoint::At(a), Side::TwoSided).expect("lim")
    }

    fn lim_inf(f: &Sym, dir: f64) -> LimitOutcome {
        let p = if dir > 0.0 {
            LimitPoint::PosInf
        } else {
            LimitPoint::NegInf
        };
        limit_expr(f, "x", &p, Side::TwoSided).expect("lim")
    }

    fn v(s: &LimitOutcome) -> String {
        match s {
            LimitOutcome::Value(t) => sym_display(t),
            other => panic!("Value نبود: {other:?}"),
        }
    }

    #[test]
    fn direct_and_rational_zero_over_zero() {
        let x = sym("x");
        // (x²−9)/(x−3) در 3 → 0/0 → مسیر مستقیم با simplify (حذف gcd) → 6
        let f = Sym::over(
            Sym::minus(Sym::pow(x.clone(), Sym::int(2)).expect("p"), Sym::int(9)),
            Sym::minus(x.clone(), Sym::int(3)),
        )
        .expect("o");
        assert_eq!(v(&lim(&f, Sym::int(3))), "6");
        // جداشدنی که gcd گویا نمی‌بیند: ((x−1/2)²)/(x−1/2) در 1/2 → 0
        let h = Sym::over(
            Sym::pow(
                Sym::minus(x.clone(), Sym::rat(1, 2).expect("r")),
                Sym::int(2),
            )
            .expect("p"),
            Sym::minus(x.clone(), Sym::rat(1, 2).expect("r")),
        )
        .expect("o");
        assert_eq!(v(&lim(&h, Sym::rat(1, 2).expect("r"))), "0");
    }

    #[test]
    fn pole_and_sides() {
        let x = sym("x");
        let f = Sym::over(Sym::int(1), Sym::pow(x.clone(), Sym::int(2)).expect("p")).expect("o");
        assert_eq!(lim(&f, Sym::int(0)), LimitOutcome::PosInf);
        let g = Sym::over(Sym::int(1), x.clone()).expect("o");
        assert!(matches!(
            lim(&g, Sym::int(0)),
            LimitOutcome::DoesNotExist(_)
        ));
        let r = limit_expr(&g, "x", &LimitPoint::At(Sym::int(0)), Side::FromRight).expect("lim");
        assert_eq!(r, LimitOutcome::PosInf);
        let l = limit_expr(&g, "x", &LimitPoint::At(Sym::int(0)), Side::FromLeft).expect("lim");
        assert_eq!(l, LimitOutcome::NegInf);
    }

    #[test]
    fn lhopital_classics() {
        let x = sym("x");
        let sin = Sym::fn_of("sin", vec![x.clone()]).expect("s");
        let f = Sym::over(sin, x.clone()).expect("o");
        assert_eq!(v(&lim(&f, Sym::int(0))), "1");
        let e = Sym::fn_of("exp", vec![x.clone()]).expect("e");
        let g = Sym::over(Sym::minus(e, Sym::int(1)), x.clone()).expect("o");
        assert_eq!(v(&lim(&g, Sym::int(0))), "1");
        let l = Sym::fn_of("log", vec![Sym::plus(Sym::int(1), x.clone())]).expect("l");
        let h = Sym::over(l, x).expect("o");
        assert_eq!(v(&lim(&h, Sym::int(0))), "1");
    }

    #[test]
    fn pow_log_euler() {
        let x = sym("x");
        // (1 + 1/x)^x → exp(1) — e نمادین است (مثل π-گونه‌ها)
        let base = Sym::plus(Sym::int(1), Sym::over(Sym::int(1), x.clone()).expect("o"));
        let f = Sym::pow(base, x.clone()).expect("p");
        assert_eq!(v(&lim_inf(&f, 1.0)), "exp(1)");
        // (1 + x)^(1/x) در 0 → exp(1)
        let base2 = Sym::plus(Sym::int(1), x.clone());
        let f2 = Sym::pow(base2, Sym::over(Sym::int(1), x).expect("o")).expect("p");
        assert_eq!(v(&lim(&f2, Sym::int(0))), "exp(1)");
    }

    #[test]
    fn infinity_rational() {
        let x = sym("x");
        let f = Sym::over(
            Sym::plus(
                Sym::times(Sym::int(3), Sym::pow(x.clone(), Sym::int(2)).expect("p")),
                Sym::int(1),
            ),
            Sym::plus(Sym::pow(x.clone(), Sym::int(2)).expect("p"), Sym::int(5)),
        )
        .expect("o");
        assert_eq!(v(&lim_inf(&f, 1.0)), "3");
        // (−x²)/1 در +∞ → NegInf
        let g = Sym::over(
            Sym::times(Sym::int(-1), Sym::pow(x.clone(), Sym::int(2)).expect("p")),
            Sym::int(1),
        )
        .expect("o");
        assert_eq!(lim_inf(&g, 1.0), LimitOutcome::NegInf);
        // x/x² در +∞ → 0
        let h = Sym::over(x.clone(), Sym::pow(x, Sym::int(2)).expect("p")).expect("o");
        assert_eq!(v(&lim_inf(&h, 1.0)), "0");
    }

    #[test]
    fn oscillation_honest_dne() {
        let x = sym("x");
        let inner = Sym::over(Sym::int(1), x.clone()).expect("o");
        let f = Sym::fn_of("sin", vec![inner]).expect("s");
        assert!(matches!(
            lim(&f, Sym::int(0)),
            LimitOutcome::DoesNotExist(_)
        ));
        // x·sin(1/x) → 0 (فشار) — عددی: Real(≈0)
        let g = Sym::times(
            x.clone(),
            Sym::fn_of("sin", vec![Sym::over(Sym::int(1), x).expect("o")]).expect("s"),
        );
        match lim(&g, Sym::int(0)) {
            LimitOutcome::Value(t) => {
                let fv = t.to_f64().expect("real");
                assert!(fv.abs() < 1e-8, "باید ≈۰ باشد: {fv}");
            }
            other => panic!("x·sin(1/x) باید همگرا شود: {other:?}"),
        }
    }

    #[test]
    fn exponential_growth_at_infinity() {
        let x = sym("x");
        // e^x/x² در +∞ → +∞ (لُوپیتال دوبار یا عددی)
        let e = Sym::fn_of("exp", vec![x.clone()]).expect("e");
        let f = Sym::over(e, Sym::pow(x, Sym::int(2)).expect("p")).expect("o");
        let out = lim_inf(&f, 1.0);
        assert!(
            matches!(out, LimitOutcome::PosInf),
            "e^x/x² باید +∞ باشد: {out:?}"
        );
        // x/e^x در +∞ → 0 (تقریب صادقانهٔ عددی)
        let g = Sym::over(sym("x"), Sym::fn_of("exp", vec![sym("x")]).expect("e")).expect("o");
        match lim_inf(&g, 1.0) {
            LimitOutcome::Value(t) => {
                let fv = t.to_f64().expect("real");
                assert!(fv.abs() < 1e-6, "x/e^x باید →۰ باشد: {fv}");
            }
            other => panic!("x/e^x باید همگرا شود: {other:?}"),
        }
    }
}
