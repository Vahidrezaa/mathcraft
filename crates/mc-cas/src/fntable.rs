//! جدول مرکزی توابع ریاضی نمادین (گام 4.2/CAS-1 — سند ۰۵ §۴).
//!
//! یک منبع حقیقت برای: نام‌های پذیرفتنی در درخت `Fn`، ارزیابی عددی f64
//! (پاریتهٔ متلب — NaN صادقانه بیرون از دامنهٔ حقیقی، نه جواب غلط)، و
//! تاش‌های همانی روی آرگومان دقیق (sin(0)=0 و sqrt(4)=2 — متلب همین‌ها
//! را همان لحظهٔ ساخت جمع می‌کند).

use num_bigint::{BigInt, BigUint};
use num_rational::BigRational;
use num_traits::{One, Signed, Zero};

use crate::{CasError, Sym};

/// نام‌های پذیرفتنی — متلب‌گونه (log = لگاریتم طبیعی).
pub const SYMBOLIC_FNS: &[&str] = &[
    "sin", "cos", "tan", "asin", "acos", "atan", "sinh", "cosh", "tanh", "exp", "log", "log10",
    "log2", "sqrt", "abs", "sign",
];

pub fn is_symbolic_fn(name: &str) -> bool {
    SYMBOLIC_FNS.contains(&name)
}

/// ارزیابی عددی — None یعنی «نام ناشناخته» (نمی‌توان حدس زد).
/// پاریتهٔ دامنهٔ حقیقی: log/سختی‌های منفی → NaN (مختلط در فاز بعد).
pub fn eval_f64(name: &str, x: f64) -> Option<f64> {
    Some(match name {
        "sin" => x.sin(),
        "cos" => x.cos(),
        "tan" => x.tan(),
        "asin" => x.asin(),
        "acos" => x.acos(),
        "atan" => x.atan(),
        "sinh" => x.sinh(),
        "cosh" => x.cosh(),
        "tanh" => x.tanh(),
        "exp" => x.exp(),
        "log" => x.ln(),
        "log10" => x.log10(),
        "log2" => x.log2(),
        "sqrt" => x.sqrt(),
        "abs" => x.abs(),
        "sign" => {
            if x.is_nan() {
                f64::NAN
            } else if x > 0.0 {
                1.0
            } else if x < 0.0 {
                -1.0
            } else {
                0.0
            }
        }
        _ => return None,
    })
}

/// سازندهٔ کانونی `Fn` — دروازهٔ صداقت: نام/‌تعداد آرگومان نامعتبر → خطا؛
/// همانی‌های قطعی روی آرگومان دقیق همان لحظهٔ ساخت جمع می‌شوند
/// (sin(0)=0، cos(0)=1، exp(0)=1، log(1)=0، sqrt(4)=2، abs(-3)=3، sign(-2)=-1).
/// π-گونه‌ها (asin(1)، acos(0)) نمادین می‌مانند — CAS-1 ثابت π ندارد
/// و حدس‌زدن عددی جعل است.
pub fn fn_of(name: &str, args: Vec<Sym>) -> Result<Sym, CasError> {
    if !is_symbolic_fn(name) {
        return Err(CasError::Unsupported(
            "unknown symbolic function (not in the CAS-1 table)",
        ));
    }
    let [a] = &args[..] else {
        return Err(CasError::Unsupported(
            "symbolic functions take exactly one argument in CAS-1",
        ));
    };
    // آرگومان عددی غیرگویا (Real) → ارزیابی f64 تقریبی صادقانه؟ نه — نگه
    // می‌داریم؛ sin(0.1) نمادین می‌ماند و در double(subs(...)) ارزیابی می‌شود
    // (نمایش و subs عددی هر دو مسیر دارند؛ جعلِ 0.0998 به‌عنوان «دقیق» نه).
    if let Some(r) = a.as_exact() {
        if let Some(fold) = fold_exact(name, &r) {
            return Ok(fold);
        }
    }
    Ok(Sym::Fn(std::sync::Arc::from(name), vec![a.clone()]))
}

/// همانی‌های قطعی روی گویای دقیق — فقط مقادیری که خودشان دقیق‌اند.
fn fold_exact(name: &str, r: &BigRational) -> Option<Sym> {
    let one = BigInt::from(1);
    match name {
        "sin" | "tan" | "asin" | "atan" | "sinh" | "tanh" if r.is_zero() => Some(Sym::int(0)),
        "cos" | "cosh" if r.is_zero() => Some(Sym::int(1)),
        "exp" if r.is_zero() => Some(Sym::int(1)),
        "log" | "log10" | "log2" if r.numer() == &one && r.denom() == &one => Some(Sym::int(0)),
        "sqrt" => fold_sqrt(r),
        "abs" => {
            // بازگشت با فرم تمیز: مخرج ۱ → Integer (هم‌نوعِ ورودی)
            let v = r.abs();
            Some(rebuild(v))
        }
        "sign" => Some(if r.is_zero() {
            Sym::int(0)
        } else if r.is_negative() {
            Sym::int(-1)
        } else {
            Sym::int(1)
        }),
        "acos" if r.numer() == &one && r.denom() == &one => Some(Sym::int(0)),
        _ => None,
    }
}

/// sqrt گویای مربع کامل → گویای دقیق (sqrt(4)=2، sqrt(9/4)=3/2)؛ وگرنه None.
fn fold_sqrt(r: &BigRational) -> Option<Sym> {
    if r.is_negative() {
        return None; // مختلط — نمادین می‌ماند، ارزیابی عددی NaN صادقانه
    }
    // sqrt(n/d) = sqrt(n)/sqrt(d) — صورت و مخرج جدا (هرگز round نکن)
    let (n, d) = (r.numer().to_biguint()?, r.denom().to_biguint()?);
    let (sn, sd) = (isqrt(&n)?, isqrt(&d)?);
    if &sn * &sn == n && &sd * &sd == d {
        let q = BigRational::new(
            BigInt::from_biguint(num_bigint::Sign::Plus, sn),
            BigInt::from_biguint(num_bigint::Sign::Plus, sd),
        );
        return Some(rebuild(q));
    }
    None
}

/// گویا → برگ تمیز (مخرج ۱ → Integer) — همان قرارداد rebuild_rational سازنده‌ها.
fn rebuild(r: BigRational) -> Sym {
    if r.denom() == &BigInt::from(1) {
        Sym::Integer(r.to_integer())
    } else {
        Sym::Rational(r)
    }
}

/// ریشهٔ صحیح کف BigUint (نیوتن، از بالا همگرم — گام آخر همیشه کف است).
fn isqrt(n: &BigUint) -> Option<BigUint> {
    if n.is_zero() {
        return Some(BigUint::zero());
    }
    let two = BigUint::from(2u32);
    // شروع: توانِ ۲ بالاتر از n (شروع از بالا، شرط همگرمی نیوتن صحیح می‌ماند)
    let mut x = BigUint::one();
    while &x * &x <= *n {
        x *= &two.pow(64u32);
    }
    loop {
        // x ← (x + n/x) / 2 — نزولی تا کفِ ریشه
        let next = (&x + n / &x) / &two;
        if next >= x {
            break;
        }
        x = next;
    }
    Some(x)
}

// ============================================================
// تست‌ها
// ============================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_folds_exact() {
        assert_eq!(fn_of("sin", vec![Sym::int(0)]).expect("t"), Sym::int(0));
        assert_eq!(fn_of("cos", vec![Sym::int(0)]).expect("t"), Sym::int(1));
        assert_eq!(fn_of("exp", vec![Sym::int(0)]).expect("t"), Sym::int(1));
        assert_eq!(fn_of("log", vec![Sym::int(1)]).expect("t"), Sym::int(0));
        assert_eq!(fn_of("sqrt", vec![Sym::int(4)]).expect("t"), Sym::int(2));
        assert_eq!(
            fn_of("sqrt", vec![Sym::rat(9, 4).expect("t")]).expect("t"),
            Sym::rat(3, 2).expect("t")
        );
        assert_eq!(
            fn_of("abs", vec![Sym::rat(-3, 4).expect("t")]).expect("t"),
            Sym::rat(3, 4).expect("t")
        );
        assert_eq!(fn_of("sign", vec![Sym::int(-7)]).expect("t"), Sym::int(-1));
    }

    #[test]
    fn non_foldable_stays_symbolic() {
        // sin(1/2) نمادین می‌ماند (متلب هم همین می‌کند)
        let s = fn_of("sin", vec![Sym::rat(1, 2).expect("t")]).expect("t");
        assert_eq!(s.display(), "sin(1/2)");
        // sqrt(2) نمادین — 1.414… جعل «دقیق» نیست
        let q = fn_of("sqrt", vec![Sym::int(2)]).expect("t");
        assert_eq!(q.display(), "sqrt(2)");
        // sqrt(2+…) ترکیبی
        let inner = Sym::plus(Sym::int(1), Sym::symbol("x"));
        assert_eq!(
            fn_of("sqrt", vec![inner]).expect("t").display(),
            "sqrt(x + 1)"
        );
        // asin(1) = π/2 — CAS-1 ثابت π ندارد؛ نمادین صادقانه
        assert_eq!(
            fn_of("asin", vec![Sym::int(1)]).expect("t").display(),
            "asin(1)"
        );
    }

    #[test]
    fn unknown_and_arity_rejected() {
        assert!(fn_of("besselj", vec![Sym::int(1)]).is_err());
        assert!(fn_of("sin", vec![Sym::int(1), Sym::int(2)]).is_err());
        assert!(fn_of("sin", vec![]).is_err());
    }

    #[test]
    fn numeric_eval_matlab_parity() {
        assert!((eval_f64("sin", 0.5).expect("t") - 0.5f64.sin()).abs() < 1e-15);
        // بیرون از دامنهٔ حقیقی → NaN صادقانه (نه عدد غلط)
        assert!(eval_f64("asin", 2.0).expect("t").is_nan());
        assert!(eval_f64("log", -1.0).expect("t").is_nan());
        assert!(eval_f64("sqrt", -4.0).expect("t").is_nan());
        // sign متلب‌گونه
        assert_eq!(eval_f64("sign", -3.0), Some(-1.0));
        assert_eq!(eval_f64("sign", 0.0), Some(0.0));
        assert!(eval_f64("sign", f64::NAN).expect("t").is_nan());
        assert!(eval_f64("legendre", 1.0).is_none());
    }

    #[test]
    fn sqrt_of_huge_perfect_square() {
        // مربع کامل بزرگ — مسیر BigUint نیوتنی
        let n = BigInt::from(10).pow(60);
        let r = BigRational::from(n);
        let folded = fold_sqrt(&r).expect("t");
        assert_eq!(folded, Sym::Integer(BigInt::from(10).pow(30)));
    }
}
