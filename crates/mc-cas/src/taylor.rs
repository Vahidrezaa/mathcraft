//! بسط تیلور نمادین — گام 4.4/CAS-3 (سند ۰۵ §۷).
//!
//! `taylor(f, x, a, order)`: مشتق‌های نمادین تا مرتبهٔ order، جای‌گذاری
//! نقطهٔ a، ضرایب دقیق گویا (`f^(k)(a)/k!`)؛ سقف مطلق مرتبه ۲۰ (سند ۰۵ §۷)
//! و بودجهٔ گره برای مشتق‌های منفجرشونده. جملهٔ `O((x−a)^n)` نمایشی است
//! (LaTeX در رندر) — درختِ برگشتی خودِ چندجمله‌ای است.
//!
//! صداقت در شکست (اصل ۵ سند ۰۰): تکین در a یا مشتق خارج از جدول یا
//! حضور abs/sign (غیرتحلیلی) → `None`/خطای روشن — هرگز سریِ غلط نه؛
//! نمونهٔ درس‌آموز: بدون گارد abs، taylor(abs(x), x, 0, 2) با ضریب
//! sign(0)=0 «رشتهٔ صفر» می‌داد و |x| را «تحلیل‌پذیر» جا می‌زداشت.

use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::One;

use crate::diff::{diff_n, node_count};
use crate::int::contains;
use crate::simplify::simplify;
use crate::sym::{CasError, Sym};

/// سقف مطلق مرتبهٔ بسط (سند ۰۵ §۷).
pub const TAYLOR_MAX_ORDER: u32 = 20;

/// بودجهٔ گرهٔ سری خروجی — مشتق‌های تودرتو (مثل tan مرتبهٔ بالا) درخت
/// را نمایی می‌کنند؛ فراتر از بودجه = خطای صادقانه نه فروریختن.
const NODE_BUDGET: usize = 40_000;

/// آیا درخت تحلیل‌پذیر است؟ abs/sign روی دامنهٔ حقیقی ناپیوسته/غیر
/// تحلیلی‌اند (سند ۰۵ §۲ — Real فقط ورودی است).
fn has_abs_or_sign(e: &Sym) -> bool {
    match e {
        Sym::Fn(name, args) => {
            (&**name == "abs" || &**name == "sign") || args.iter().any(has_abs_or_sign)
        }
        Sym::Add(ts) | Sym::Mul(ts) => ts.iter().any(has_abs_or_sign),
        Sym::Pow(b, ex) => has_abs_or_sign(b) || has_abs_or_sign(ex),
        _ => false,
    }
}

/// بسط تیلور f حول x=a تا مرتبهٔ order (شامل): `Σ f^(k)(a)/k!·(x−a)^k`.
/// a می‌تواند نمادین باشد (مثل `taylor(sin(x), x, a, 2)`). نقطهٔ تکین یا
/// مشتق‌گیری ناممکن → `Ok(None)` صادقانه؛ فراتر از سقف/بودجه → خطا.
pub fn taylor(f: &Sym, x: &str, a: &Sym, order: u32) -> Result<Option<Sym>, CasError> {
    if order > TAYLOR_MAX_ORDER {
        return Err(CasError::Unsupported(
            "taylor order above the absolute cap of 20",
        ));
    }
    if has_abs_or_sign(f) {
        // |x| و sign در 0 تحلیلی نیستند — سریِ صفرِ غلط می‌داد
        return Ok(None);
    }
    let mut fact = BigRational::one();
    let mut acc: Option<Sym> = None;
    for k in 0..=order {
        if k > 0 {
            fact *= BigRational::from_integer(BigInt::from(k as i64));
        }
        let d = diff_n(f, x, k as i64)?;
        let c = match simplify(&d.substitute(x, a)) {
            Ok(c) => c,
            // 0/0 یا تقسیم بر صفر هنگام جای‌گذاری → تکین در a
            Err(CasError::DivByZero) => return Ok(None),
            Err(other) => return Err(other),
        };
        if contains(&c, x) {
            return Ok(None);
        }
        let coeff = Sym::Rational(BigRational::one() / fact.clone());
        let term = if k == 0 {
            Sym::times(c, coeff)
        } else {
            let dx = Sym::minus(Sym::symbol(x), a.clone());
            let p = Sym::pow(dx, Sym::int(k as i64))?;
            Sym::times(Sym::times(c, p), coeff)
        };
        acc = Some(match acc {
            None => term,
            Some(prev) => Sym::plus(prev, term),
        });
        if let Some(prev) = &acc {
            if node_count(prev) > NODE_BUDGET {
                return Err(CasError::Unsupported(
                    "taylor series exceeds the node budget",
                ));
            }
        }
    }
    Ok(acc)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sym_display;

    fn sym(n: &str) -> Sym {
        Sym::symbol(n)
    }

    /// جملهٔ c·x^k کانونی — c = num/den
    fn term(num: i64, den: i64, k: i64) -> Sym {
        let co = Sym::Rational(BigRational::new(BigInt::from(num), BigInt::from(den)));
        if k == 0 {
            co
        } else {
            Sym::times(Sym::pow(sym("x"), Sym::int(k)).expect("p"), co)
        }
    }

    #[test]
    fn exp_sin_cos_at_zero() {
        // برابری ساختاری — ترتیب نمایش کانونی (ثابت آخر، سند ۰۵ §۲) مهم نیست
        let x = sym("x");
        let e = Sym::fn_of("exp", vec![x.clone()]).expect("e");
        let s = taylor(&e, "x", &Sym::int(0), 4).expect("ok").expect("some");
        let want = [
            term(1, 1, 0),
            term(1, 1, 1),
            term(1, 2, 2),
            term(1, 6, 3),
            term(1, 24, 4),
        ]
        .into_iter()
        .reduce(Sym::plus)
        .expect("fold");
        assert_eq!(s, want);

        let sn = Sym::fn_of("sin", vec![x.clone()]).expect("s");
        let s2 = taylor(&sn, "x", &Sym::int(0), 3)
            .expect("ok")
            .expect("some");
        let want2 = Sym::minus(
            sym("x"),
            Sym::times(
                Sym::pow(sym("x"), Sym::int(3)).expect("p"),
                Sym::Rational(BigRational::new(BigInt::from(1), BigInt::from(6))),
            ),
        );
        assert_eq!(s2, want2);

        let c = Sym::fn_of("cos", vec![x]).expect("c");
        let s3 = taylor(&c, "x", &Sym::int(0), 4).expect("ok").expect("some");
        let want3 = Sym::plus(Sym::minus(Sym::int(1), term(1, 2, 2)), term(1, 24, 4));
        assert_eq!(s3, want3);
    }

    #[test]
    fn geometric_and_log() {
        let x = sym("x");
        // 1/(1−x) = 1 + x + x² + x³
        let g = Sym::over(Sym::int(1), Sym::minus(Sym::int(1), x.clone())).expect("o");
        let s = taylor(&g, "x", &Sym::int(0), 3).expect("ok").expect("some");
        let want = [term(1, 1, 0), term(1, 1, 1), term(1, 1, 2), term(1, 1, 3)]
            .into_iter()
            .reduce(Sym::plus)
            .expect("fold");
        assert_eq!(s, want);
        // log(1+x) = x − x²/2 + x³/3 − x⁴/4
        let l = Sym::fn_of("log", vec![Sym::plus(Sym::int(1), x)]).expect("l");
        let s2 = taylor(&l, "x", &Sym::int(0), 4).expect("ok").expect("some");
        let want2 = [term(1, 1, 1), term(-1, 2, 2), term(1, 3, 3), term(-1, 4, 4)]
            .into_iter()
            .reduce(Sym::plus)
            .expect("fold");
        assert_eq!(s2, want2);
    }

    #[test]
    fn shifted_and_symbolic_point() {
        let x = sym("x");
        // e^x حول a=1: exp(1)·(1 + (x−1) + (x−1)²/2)
        let e = Sym::fn_of("exp", vec![x.clone()]).expect("e");
        let s = taylor(&e, "x", &Sym::int(1), 2).expect("ok").expect("some");
        let d = sym_display(&s);
        assert!(d.contains("exp(1)"), "باید exp(1) داشته باشد: {d}");
        assert!(d.contains("x - 1"), "باید (x−1) داشته باشد: {d}");
        // نقطهٔ نمادین: sin حول a → sin(a) + cos(a)(x−a) − sin(a)/2·(x−a)²
        let a = sym("a");
        let sn = Sym::fn_of("sin", vec![x.clone()]).expect("s");
        let s2 = taylor(&sn, "x", &a, 2).expect("ok").expect("some");
        let d2 = sym_display(&s2);
        assert!(
            d2.contains("sin(a)") && d2.contains("cos(a)"),
            "نقطهٔ نمادین: {d2}"
        );
    }

    #[test]
    fn singular_and_abs_honest_none() {
        let x = sym("x");
        // تکین: 1/x در 0 → None
        let f = Sym::over(Sym::int(1), x.clone()).expect("o");
        assert!(taylor(&f, "x", &Sym::int(0), 3).expect("ok").is_none());
        // غیرتحلیلی: |x| در 0 → None (نه سریِ صفرِ غلط)
        let absf = Sym::fn_of("abs", vec![x]).expect("a");
        assert!(taylor(&absf, "x", &Sym::int(0), 2).expect("ok").is_none());
        // سقف مطلق
        let e = Sym::fn_of("exp", vec![sym("x")]).expect("e");
        assert!(taylor(&e, "x", &Sym::int(0), 21).is_err());
    }
}
