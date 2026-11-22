//! مشتق‌گیری نمادین کامل (گام 4.2/CAS-1 — سند ۰۵ §۴).
//!
//! الگوریتم کلاسیک بازگشتی روی درخت — همهٔ خروجی‌ها از سازنده‌های
//! کانونی ساخته می‌شوند، پس کانونی‌سازی سطح-۰ خودکار اعمال می‌شود.
//!
//! فرم‌های خروجی متلب‌گونه (راستی‌آزمایی‌شده): diff(tan(u)) = (tan(u)^2+1)·u′،
//! diff(abs(u)) = sign(u)·u′ (دامنهٔ حقیقی)، توان عمومی
//! u^v → u^v·(v′·log(u) + v·u′/u) — x^x → x^x*(log(x) + 1).
//!
//! صداقت دامنه: هر فرم مشتق دقیقاً روی همان دامنه‌ای معتبر است که خودِ
//! عبارتِ مبنا تعریف‌شده است (u^(-1) در 0 تعریف‌نشده می‌ماند و مشتقش هم
//! همین‌طور) — هرگز ناحیهٔ تعریف را گسترش نمی‌دهیم.

use crate::{CasError, Sym};
use num_traits::Zero;

/// سقف مرتبهٔ مشتق — صادقانه، نه هنگِ بی‌پایان روی دستگاه.
pub const MAX_DERIV_ORDER: i64 = 10_000;
/// سقف گرهٔ درخت نتیجه پس از هر مرتبه.
const MAX_NODES: usize = 10_000;

/// مشتق مرتبهٔ n — n منفی/نامعتبر → خطای صادقانه؛ رشد درخت با بودجهٔ
/// گره کنترل می‌شود (مشتق‌های تکراریِ عبارت‌های نمایی‌رشد صادقانه
/// متوقف می‌شوند، نه قفل‌کردن دستگاه).
pub fn diff_n(expr: &Sym, var: &str, n: i64) -> Result<Sym, CasError> {
    if n < 0 {
        return Err(CasError::Unsupported(
            "derivative order must be a non-negative integer",
        ));
    }
    if n > MAX_DERIV_ORDER {
        return Err(CasError::Unsupported(
            "derivative order too large (limit 10000)",
        ));
    }
    let mut acc = expr.clone();
    for _ in 0..n {
        acc = diff(&acc, var)?;
        if node_count(&acc) > MAX_NODES {
            return Err(CasError::Unsupported(
                "derivative expression grew too large (node budget)",
            ));
        }
    }
    Ok(acc)
}

/// شمارش گره‌های درخت.
pub fn node_count(s: &Sym) -> usize {
    1 + match s {
        Sym::Integer(_) | Sym::Rational(_) | Sym::Real(_) | Sym::Symbol(_) => 0,
        Sym::Add(v) | Sym::Mul(v) => v.iter().map(node_count).sum::<usize>(),
        Sym::Pow(b, e) => node_count(b) + node_count(e),
        Sym::Fn(_, args) => args.iter().map(node_count).sum::<usize>(),
    }
}

/// مشتق جزئی نسبت به نماد `var` — بقیهٔ نمادها ثابت می‌مانند.
pub fn diff(expr: &Sym, var: &str) -> Result<Sym, CasError> {
    // زیردرخت ثابت → 0 (پیش از هر ساخت — مسیر فشرده و صادقانه)
    if !contains_var(expr, var) {
        return Ok(Sym::int(0));
    }
    match expr {
        Sym::Integer(_) | Sym::Rational(_) | Sym::Real(_) | Sym::Symbol(_) => {
            if let Sym::Symbol(s) = expr {
                if &**s == var {
                    return Ok(Sym::int(1));
                }
            }
            Ok(Sym::int(0))
        }
        Sym::Add(terms) => {
            let mut acc: Option<Sym> = None;
            for t in terms {
                let d = diff(t, var)?;
                acc = Some(match acc {
                    None => d,
                    Some(a) => Sym::plus(a, d),
                });
            }
            Ok(acc.unwrap_or_else(|| Sym::int(0)))
        }
        Sym::Mul(factors) => {
            // قاعدهٔ ضرب: Σᵢ fᵢ′·Πⱼ≠ᵢ fⱼ
            let mut acc: Option<Sym> = None;
            for i in 0..factors.len() {
                let di = diff(&factors[i], var)?;
                if matches!(di, Sym::Integer(ref z) if z.is_zero()) {
                    continue; // عامل ثابت — جملهٔ صفر لازم نیست
                }
                let mut rest: Option<Sym> = None;
                for (j, f) in factors.iter().enumerate() {
                    if j != i {
                        rest = Some(match rest {
                            None => f.clone(),
                            Some(r) => Sym::times(r, f.clone()),
                        });
                    }
                }
                let prod = Sym::times(di, rest.unwrap_or_else(|| Sym::int(1)));
                acc = Some(match acc {
                    None => prod,
                    Some(a) => Sym::plus(a, prod),
                });
            }
            Ok(acc.unwrap_or_else(|| Sym::int(0)))
        }
        Sym::Pow(b, e) => diff_pow(b, e, var),
        Sym::Fn(name, args) => {
            // قضیهٔ بنیادی حسابان (گام 4.۳): d/dx از گرهٔ unevaluatedِ
            // خودِ int برابر خودِ انتگران‌گیرنده است — مشتق هرگز صفرِ
            // غلط نمی‌دهد؛ متغیر انتگرال غیر از var → خطای صادقانه.
            if &**name == "int" && args.len() == 2 {
                if let Sym::Symbol(v) = &args[1] {
                    if &**v == var {
                        return Ok(args[0].clone());
                    }
                }
                return Err(CasError::Unsupported(
                    "differentiating an integral with respect to another variable is not supported in v1",
                ));
            }
            let [a] = &args[..] else {
                return Err(CasError::Unsupported(
                    "symbolic functions take exactly one argument in CAS-1",
                ));
            };
            let da = diff(a, var)?;
            diff_fn(name, a, &da)
        }
    }
}

fn diff_pow(b: &Sym, e: &Sym, var: &str) -> Result<Sym, CasError> {
    // توان صحیح کوچک: n·u^(n-1)·u′
    if let Some(n) = as_small_int(e) {
        if n == 0 {
            return Ok(Sym::int(0)); // u^0 = 1 ثابت
        }
        let db = diff(b, var)?;
        let power = if n == 1 {
            Sym::int(1)
        } else {
            Sym::pow(b.clone(), Sym::int(n - 1))?
        };
        // n·u^(n-1)·u′ — برای n<0 همین فرم روی دامنهٔ خودِ u درست است
        return Ok(Sym::times(Sym::times(Sym::int(n), power), db));
    }
    let de = diff(e, var)?;
    let db = diff(b, var)?;
    let db_const = matches!(db, Sym::Integer(ref z) if z.is_zero());
    let de_const = matches!(de, Sym::Integer(ref z) if z.is_zero());
    if db_const && !de_const {
        // پایه ثابت: a^v → a^v·log(a)·v′
        let log_b = Sym::fn_of("log", vec![b.clone()])?;
        return Ok(Sym::times(
            Sym::times(Sym::pow(b.clone(), e.clone())?, log_b),
            de,
        ));
    }
    if !db_const && de_const {
        // نمای ثابت a (غیرصحیح): u^a → a·u^(a-1)·u′
        let am1 = Sym::minus(e.clone(), Sym::int(1));
        return Ok(Sym::times(
            Sym::times(e.clone(), Sym::pow(b.clone(), am1)?),
            db,
        ));
    }
    // توان عمومی: u^v·(v′·log(u) + v·u′/u) — گارد دامنه: log(u) فقط برای
    // u>0 معتبر است؛ همان دامنهٔ حقیقیِ u^v. متلب همین فرم را می‌دهد.
    let log_b = Sym::fn_of("log", vec![b.clone()])?;
    let du_over_u = Sym::over(db, b.clone())?;
    let inner = Sym::plus(Sym::times(de, log_b), Sym::times(e.clone(), du_over_u));
    Ok(Sym::times(Sym::pow(b.clone(), e.clone())?, inner))
}

fn diff_fn(name: &str, u: &Sym, du: &Sym) -> Result<Sym, CasError> {
    // فرم‌های متلب‌گونه — u′ همیشه در جای ضریب آخر (نمایش تمیز)
    let f = |n: &str, a: &Sym| Sym::fn_of(n, vec![a.clone()]);
    let wrap = |body: Sym| Ok(Sym::times(body, du.clone()));
    match name {
        "sin" => wrap(f("cos", u)?),
        "cos" => Ok(Sym::times(
            Sym::times(Sym::int(-1), f("sin", u)?),
            du.clone(),
        )),
        "tan" => wrap(Sym::plus(Sym::int(1), Sym::pow(f("tan", u)?, Sym::int(2))?)),
        "asin" => wrap(Sym::over(
            Sym::int(1),
            Sym::fn_of(
                "sqrt",
                vec![Sym::minus(Sym::int(1), Sym::pow(u.clone(), Sym::int(2))?)],
            )?,
        )?),
        "acos" => Ok(Sym::times(
            Sym::times(
                Sym::int(-1),
                Sym::over(
                    Sym::int(1),
                    Sym::fn_of(
                        "sqrt",
                        vec![Sym::minus(Sym::int(1), Sym::pow(u.clone(), Sym::int(2))?)],
                    )?,
                )?,
            ),
            du.clone(),
        )),
        "atan" => wrap(Sym::over(
            Sym::int(1),
            Sym::plus(Sym::int(1), Sym::pow(u.clone(), Sym::int(2))?),
        )?),
        "sinh" => wrap(f("cosh", u)?),
        "cosh" => wrap(f("sinh", u)?),
        "tanh" => wrap(Sym::minus(
            Sym::int(1),
            Sym::pow(f("tanh", u)?, Sym::int(2))?,
        )),
        "exp" => wrap(f("exp", u)?),
        // log و خانواده و sqrt: صورتِ کسر خودِ u′ است — بدون wrap
        // (درس اجرای اول: wrap دوبار du می‌گرفت و مشتق log(x^2+1) را
        // 4x²/(x²+1) می‌داد؛ خودآزمایی عددی گرفتش)
        "log" => Ok(Sym::over(du.clone(), u.clone())?),
        "log10" => Ok(Sym::over(
            du.clone(),
            Sym::times(u.clone(), Sym::fn_of("log", vec![Sym::int(10)])?),
        )?),
        "log2" => Ok(Sym::over(
            du.clone(),
            Sym::times(u.clone(), Sym::fn_of("log", vec![Sym::int(2)])?),
        )?),
        "sqrt" => Ok(Sym::over(
            du.clone(),
            Sym::times(Sym::int(2), f("sqrt", u)?),
        )?),
        // دامنهٔ حقیقی: (d/dx)|u| = sign(u)·u′ (در 0 مشتق ندارد؛ فرم
        // نمادین همان قرارداد متلب است — سند ۰۵ §۴ گارد)
        "abs" => wrap(f("sign", u)?),
        // sign پارچه‌ای ثابت است — متلب مشتقش را 0 می‌دهد
        "sign" => Ok(Sym::int(0)),
        _ => Err(CasError::Unsupported(
            "unknown symbolic function (not in the CAS-1 table)",
        )),
    }
}

fn as_small_int(s: &Sym) -> Option<i64> {
    if let Sym::Integer(i) = s {
        use num_traits::ToPrimitive;
        return i.to_i64();
    }
    None
}

fn contains_var(s: &Sym, var: &str) -> bool {
    match s {
        Sym::Integer(_) | Sym::Rational(_) | Sym::Real(_) => false,
        Sym::Symbol(n) => &**n == var,
        Sym::Add(v) | Sym::Mul(v) => v.iter().any(|t| contains_var(t, var)),
        Sym::Pow(b, e) => contains_var(b, var) || contains_var(e, var),
        Sym::Fn(_, args) => args.iter().any(|a| contains_var(a, var)),
    }
}

/// متغیر پیش‌فرض به سبک symvar متلب: نزدیک‌ترین نام تک‌حرفی به «x»
/// (تساوی → نزدیک‌تر به «z»). نام چندحرفی و نبودِ نامزد → خطای صادقانه
/// با درخواست متغیر صریح.
pub fn default_var(expr: &Sym) -> Result<String, CasError> {
    let mut best: Option<(String, (i32, i32))> = None;
    for name in expr.free_symbols() {
        let cs: Vec<char> = name.chars().collect();
        if cs.len() != 1 {
            continue; // نامزد تک‌حرفی — قاعدهٔ مستند
        }
        let c = cs[0] as i32;
        if !('a' as i32..='z' as i32).contains(&c) {
            continue;
        }
        let key = ((c - 'x' as i32).abs(), ('z' as i32 - c).abs());
        if best.as_ref().is_none_or(|(_, k)| key < *k) {
            best = Some((name, key));
        }
    }
    best.map(|(n, _)| n).ok_or(CasError::Unsupported(
        "no single-letter symbolic variable to pick as default; pass the variable explicitly, e.g. diff(f, x)",
    ))
}

// ============================================================
// تست‌ها — واحدهای قاعده‌به‌قاعده + فرم‌های نمایش متلب‌گونه
// ============================================================
#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit::*;

    fn d(e: &Sym) -> Sym {
        diff(e, "x").expect("t")
    }

    #[test]
    fn polynomial_and_power_rules() {
        assert_eq!(d(&sym("x")).display(), "1");
        assert_eq!(d(&int(5)).display(), "0");
        assert_eq!(d(&pow(&sym("x"), &int(3))).display(), "3*x^2");
        // 3x^2 + 2x + 7 → 6x + 2
        let p = add(&[
            times(&[int(3), pow(&sym("x"), &int(2))]),
            times(&[int(2), sym("x")]),
            int(7),
        ]);
        assert_eq!(d(&p).display(), "6*x + 2");
        // توان منفی: x^-2 → -2*x^-3
        assert_eq!(d(&pow(&sym("x"), &int(-2))).display(), "-2*x^-3");
    }

    #[test]
    fn product_rule() {
        // x*sin(x) → sin(x) + x*cos(x)
        let e = times(&[sym("x"), f("sin", &[sym("x")])]);
        assert_eq!(d(&e).display(), "sin(x) + x*cos(x)");
    }

    #[test]
    fn quotient_form() {
        // x/sin(x) → قاعدهٔ ضرب روی x·sin(x)^-1: 1/sin(x) − x·sin(x)^-2·cos(x)
        // (هم‌ارزِ کسر ترکیبی — خودآزمایی عددیِ verify این را تأیید می‌کند)
        let e = over(&sym("x"), &f("sin", &[sym("x")])).expect("t");
        assert_eq!(d(&e).display(), "1/(sin(x)) - x*sin(x)^-2*cos(x)");
    }

    #[test]
    fn trig_table_matlab_forms() {
        assert_eq!(d(&f("sin", &[sym("x")])).display(), "cos(x)");
        assert_eq!(d(&f("cos", &[sym("x")])).display(), "-sin(x)");
        assert_eq!(d(&f("tan", &[sym("x")])).display(), "tan(x)^2 + 1");
        assert_eq!(d(&f("exp", &[sym("x")])).display(), "exp(x)");
        assert_eq!(d(&f("log", &[sym("x")])).display(), "1/x");
        assert_eq!(d(&f("sqrt", &[sym("x")])).display(), "1/(2*sqrt(x))");
        // زنجیره: cos(2x) → -2*sin(2x)
        assert_eq!(
            d(&f("cos", &[times(&[int(2), sym("x")])])).display(),
            "-2*sin(2*x)"
        );
        // atan: 1/(1+x^2)
        assert_eq!(d(&f("atan", &[sym("x")])).display(), "1/(x^2 + 1)");
        // tanh: 1 - tanh² (فرم کانونی: ترم نمادی اول، ثابت آخر)
        assert_eq!(d(&f("tanh", &[sym("x")])).display(), "-tanh(x)^2 + 1");
        // abs (دامنهٔ حقیقی): sign(x)
        assert_eq!(d(&f("abs", &[sym("x")])).display(), "sign(x)");
    }

    #[test]
    fn general_power_rule() {
        // x^x → x^x*(log(x) + 1) — فرم متلب
        let e = pow(&sym("x"), &sym("x"));
        assert_eq!(d(&e).display(), "x^x*(log(x) + 1)");
        // پایهٔ ثابت: 2^x → 2^x*log(2)
        let e2 = pow(&int(2), &sym("x"));
        assert_eq!(d(&e2).display(), "2^x*log(2)");
        // نمای ثابت گویا: x^(1/2) → 1/2*x^(-1/2) (کسر در توان پرانتز می‌گیرد)
        let e3 = pow(&sym("x"), &rat(1, 2));
        assert_eq!(d(&e3).display(), "1/2*x^(-1/2)");
    }

    #[test]
    fn demo_expression_dod() {
        // دموی DoD: diff(exp(-x^2)*x^3) — ضرب + زنجیره
        let e = times(&[
            f("exp", &[neg(&pow(&sym("x"), &int(2)))]),
            pow(&sym("x"), &int(3)),
        ]);
        let dd = d(&e);
        assert_eq!(dd.display(), "-2*x^4*exp(-x^2) + 3*x^2*exp(-x^2)");
    }

    #[test]
    fn partial_and_other_symbols() {
        // ∂/∂x وقتی y حاضر است — y ثابت
        let e = times(&[sym("x"), pow(&sym("y"), &int(2))]);
        assert_eq!(diff(&e, "x").expect("t").display(), "y^2");
        // ∂/∂y → 2*x*y
        assert_eq!(diff(&e, "y").expect("t").display(), "2*x*y");
    }

    #[test]
    fn nth_derivative() {
        // d²/dx²(x^3) = 6x
        let e = pow(&sym("x"), &int(3));
        assert_eq!(diff_n(&e, "x", 2).expect("t").display(), "6*x");
        assert_eq!(diff_n(&e, "x", 0).expect("t").display(), "x^3");
        assert!(diff_n(&e, "x", -1).is_err());
    }

    #[test]
    fn default_var_symvar_rule() {
        // نزدیک‌ترین به x: y → x? نه؛ میان y و t → y? قاعده: نزدیک‌تر به x
        // (|c−x|، تساوی → نزدیک‌تر به z)
        let e1 = times(&[sym("x"), sym("y")]);
        assert_eq!(default_var(&e1).expect("t"), "x");
        let e2 = add(&[sym("y"), sym("t")]);
        assert_eq!(default_var(&e2).expect("t"), "y");
        // تساوی فاصله: u (فاصله 2) و z (فاصله 2) → نزدیک‌تر به z برنده
        let e3 = add(&[sym("u"), sym("z")]);
        assert_eq!(default_var(&e3).expect("t"), "z");
        // فقط نام چندحرفی → خطای صادقانه
        let e4 = sym("theta");
        assert!(default_var(&e4).is_err());
    }
}
