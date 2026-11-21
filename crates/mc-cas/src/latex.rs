//! نویسندهٔ LaTeX (گام 4.2/CAS-1 — سند ۰۵ §۱۰).
//!
//! نویسندهٔ اختصاصی ما: تقدم‌آگاه با پرانتز حداقلی (`\left(` `\right)`)،
//! `\frac` برای کسرهای گویا و خارج‌قسمت‌ها، `\sqrt`، توان‌ها همیشه در
//! آکولاد، نام‌های یونانی (`\theta` و …)، منفیِ یکهٔ تمیز، exp به شکل
//! `\mathrm{e}^{…}` (قرارداد متلب). رشتهٔ خروجی برای رندر سمت Dart
//! (flutter_math_fork — سند ۰۶) و `latex(S)` برای گزارش‌های درس است.
//!
//! خروجی قطعی است — تست‌های طلایی رشته‌ای پینش کرده‌اند. ابهام‌زدایی با
//! آکولاد خود توان انجام می‌شود (x^{\frac{1}{2}}) پس کسر در جایگاه توان
//! پرانتز نمی‌خواهد؛ فقط پایه‌های ساختاری (جمع/ضرب/توان/عدد منفی) پرانتز
//! می‌گیرند.

use std::fmt::Write as _;

use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::Signed;

use crate::sym_as_small_int;
use crate::Sym;

/// درخت → رشتهٔ LaTeX.
pub fn to_latex(s: &Sym) -> String {
    let mut out = String::new();
    write_latex(&mut out, s, Prec::Top);
    out
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Prec {
    Top,
    Add,
    Mul,
    Pow,
    Atom,
}

fn write_latex(out: &mut String, s: &Sym, ctx: Prec) {
    match s {
        Sym::Integer(i) => {
            let _ = write!(out, "{i}");
        }
        Sym::Rational(r) => {
            let (n, d) = (r.numer(), r.denom());
            if d == &BigInt::from(1) {
                let _ = write!(out, "{n}");
            } else {
                let _ = write!(out, "\\frac{{{n}}}{{{d}}}");
            }
        }
        Sym::Real(x) => out.push_str(&real_latex(*x)),
        Sym::Symbol(n) => out.push_str(&symbol_latex(n)),
        Sym::Add(terms) => {
            if ctx > Prec::Add {
                out.push_str("\\left(");
            }
            for (i, t) in terms.iter().enumerate() {
                if i == 0 {
                    write_latex(out, t, Prec::Add);
                    continue;
                }
                // ترم منفی با « - » می‌چسبد (مثل نمایش خطی)
                let mut buf = String::new();
                write_latex(&mut buf, t, Prec::Add);
                if let Some(stripped) = buf.strip_prefix('-') {
                    out.push_str(" - ");
                    out.push_str(stripped);
                } else {
                    out.push_str(" + ");
                    out.push_str(&buf);
                }
            }
            if ctx > Prec::Add {
                out.push_str("\\right)");
            }
        }
        Sym::Mul(factors) => write_mul_latex(out, factors),
        Sym::Pow(b, e) => write_pow_latex(out, b, e, ctx),
        Sym::Fn(name, args) => write_fn_latex(out, name, args),
    }
}

/// ضرب — هرگز پرانتز بیرونی نمی‌خواهد (عامل‌های جمع‌دار خودشان
/// پرانتز می‌گیرند و ضرب در جایگاه توان با آکولاد توان جدا می‌شود).
/// ضریب عددی آغازین جدا می‌شود؛ شکل تقسیم (آخرین عامل b^-1) → \frac.
fn write_mul_latex(out: &mut String, factors: &[Sym]) {
    // جداکردن ضریب گویای آغازین
    let mut coeff: Option<BigRational> = None;
    let mut rest: Vec<&Sym> = Vec::new();
    for f in factors {
        if let Some(r) = f.as_exact() {
            if coeff.is_none() && rest.is_empty() {
                coeff = Some(r);
                continue;
            }
        }
        rest.push(f);
    }
    let neg = coeff.as_ref().is_some_and(|c| c.is_negative());
    if neg {
        out.push('-');
    }
    let mag = coeff.map(|c| c.abs());
    // شکل تقسیم: یک عامل Pow(b,-1) در هر جای ضرب — صورت = ضریب × بقیه
    if let Some(i) = rest
        .iter()
        .rposition(|f| matches!(f, Sym::Pow(_, e) if sym_as_small_int(e) == Some(-1)))
    {
        let (b, num_factors) = match rest[i] {
            Sym::Pow(b, _) => (
                b,
                rest.iter()
                    .enumerate()
                    .filter(|(j, _)| *j != i)
                    .map(|(_, f)| *f)
                    .collect::<Vec<&Sym>>(),
            ),
            _ => return,
        };
        out.push_str("\\frac{");
        let mut wrote = false;
        if let Some(c) = &mag {
            if *c != BigRational::from(BigInt::from(1)) {
                if c.denom() == &BigInt::from(1) {
                    let _ = write!(out, "{}", c.numer());
                } else {
                    let _ = write!(out, "\\frac{{{}}}{{{}}}", c.numer(), c.denom());
                }
                wrote = true;
            }
        }
        for f in &num_factors {
            if wrote {
                out.push_str("\\,");
            }
            write_latex(out, f, Prec::Mul);
            wrote = true;
        }
        if !wrote {
            out.push('1');
        }
        out.push_str("}{");
        write_latex(out, b, Prec::Top);
        out.push('}');
        return;
    }
    // مسیر عادی: ضریب عددی (مثبت‌شده — منفی بالا رفت) + عامل‌ها
    if let Some(c) = &mag {
        if *c != BigRational::from(BigInt::from(1)) {
            if c.denom() == &BigInt::from(1) {
                let _ = write!(out, "{}\\,", c.numer());
            } else {
                let _ = write!(out, "\\frac{{{}}}{{{}}}\\,", c.numer(), c.denom());
            }
        }
    }
    for (i, f) in rest.iter().enumerate() {
        if i > 0 {
            out.push_str("\\,");
        }
        write_latex(out, f, Prec::Mul);
    }
}

fn write_pow_latex(out: &mut String, b: &Sym, e: &Sym, ctx: Prec) {
    // توان -۱ تنها (بیرونِ ضرب): \frac{1}{b} — مثل نمایش خطی 1/x
    if ctx != Prec::Mul && sym_as_small_int(e) == Some(-1) {
        out.push_str("\\frac{1}{");
        write_latex(out, b, Prec::Top);
        out.push('}');
        return;
    }
    // پرانتز پایه: ساختارهای ترکیبی و اعداد منفی/کسری
    // (پایهٔ توانِ توان بدون آکولاد خطای double-superscript می‌دهد)
    let need_b = matches!(b, Sym::Add(_) | Sym::Mul(_) | Sym::Real(_) | Sym::Pow(..))
        || matches!(b, Sym::Integer(ref i) if *i < BigInt::from(0))
        || matches!(b, Sym::Rational(ref r) if *r.numer() < BigInt::from(0) || r.denom() != &BigInt::from(1));
    if need_b {
        out.push_str("\\left(");
        write_latex(out, b, Prec::Top);
        out.push_str("\\right)");
    } else {
        write_latex(out, b, Prec::Pow);
    }
    out.push_str("^{");
    write_latex(out, e, Prec::Atom);
    out.push('}');
}

fn write_fn_latex(out: &mut String, name: &str, args: &[Sym]) {
    let [a] = args else {
        // CAS-1 تک‌آرگومانی است — گزارهٔ unreachable برای درخت‌های خام
        let _ = write!(out, "\\mathrm{{{name}}}\\left(");
        for (i, g) in args.iter().enumerate() {
            if i > 0 {
                out.push_str(", ");
            }
            write_latex(out, g, Prec::Top);
        }
        out.push_str("\\right)");
        return;
    };
    match name {
        "sqrt" => {
            out.push_str("\\sqrt{");
            write_latex(out, a, Prec::Top);
            out.push('}');
        }
        "exp" => {
            // قرارداد متلب: \mathrm{e}^{…} (آرگومان در آکولاد — بدون پرانتز)
            out.push_str("\\mathrm{e}^{");
            write_latex(out, a, Prec::Atom);
            out.push('}');
        }
        "abs" => {
            out.push_str("\\left|");
            write_latex(out, a, Prec::Top);
            out.push_str("\\right|");
        }
        "sin" | "cos" | "tan" | "asin" | "acos" | "atan" | "sinh" | "cosh" | "tanh" | "log"
        | "log10" | "log2" | "sign" => {
            out.push_str(fn_name_latex(name));
            out.push_str("\\left(");
            write_latex(out, a, Prec::Top);
            out.push_str("\\right)");
        }
        other => {
            let _ = write!(out, "\\mathrm{{{other}}}\\left(");
            write_latex(out, a, Prec::Top);
            out.push_str("\\right)");
        }
    }
}

fn fn_name_latex(name: &str) -> &'static str {
    match name {
        "sin" => "\\sin",
        "cos" => "\\cos",
        "tan" => "\\tan",
        "asin" => "\\arcsin",
        "acos" => "\\arccos",
        "atan" => "\\arctan",
        "sinh" => "\\sinh",
        "cosh" => "\\cosh",
        "tanh" => "\\tanh",
        "log" => "\\ln",
        "log10" => "\\log_{10}",
        "log2" => "\\log_{2}",
        "sign" => "\\mathrm{sign}",
        _ => "\\mathrm{op}",
    }
}

/// نماد → نام یونانی یا خودِ نام.
fn symbol_latex(n: &str) -> String {
    let greek = match n {
        "alpha" => Some("\\alpha"),
        "beta" => Some("\\beta"),
        "gamma" => Some("\\gamma"),
        "delta" => Some("\\delta"),
        "epsilon" => Some("\\epsilon"),
        "zeta" => Some("\\zeta"),
        "eta" => Some("\\eta"),
        "theta" => Some("\\theta"),
        "iota" => Some("\\iota"),
        "kappa" => Some("\\kappa"),
        "lambda" => Some("\\lambda"),
        "mu" => Some("\\mu"),
        "nu" => Some("\\nu"),
        "xi" => Some("\\xi"),
        "pi" => Some("\\pi"),
        "rho" => Some("\\rho"),
        "sigma" => Some("\\sigma"),
        "tau" => Some("\\tau"),
        "phi" => Some("\\phi"),
        "varphi" => Some("\\varphi"),
        "chi" => Some("\\chi"),
        "psi" => Some("\\psi"),
        "omega" => Some("\\omega"),
        "Gamma" => Some("\\Gamma"),
        "Delta" => Some("\\Delta"),
        "Theta" => Some("\\Theta"),
        "Lambda" => Some("\\Lambda"),
        "Xi" => Some("\\Xi"),
        "Pi" => Some("\\Pi"),
        "Sigma" => Some("\\Sigma"),
        "Phi" => Some("\\Phi"),
        "Psi" => Some("\\Psi"),
        "Omega" => Some("\\Omega"),
        _ => None,
    };
    greek.map(str::to_string).unwrap_or_else(|| n.to_string())
}

fn real_latex(x: f64) -> String {
    if x.is_nan() {
        return "\\mathrm{NaN}".into();
    }
    if x == f64::INFINITY {
        return "\\infty".into();
    }
    if x == f64::NEG_INFINITY {
        return "-\\infty".into();
    }
    format!("{x}")
}

// ============================================================
// تست‌های طلایی (سند ۰۵ §۱۰)
// ============================================================
#[cfg(test)]
mod tests {
    use crate::diff;
    use crate::latex::to_latex;
    use crate::testkit::*;

    #[test]
    fn goldens_atoms_and_arithmetic() {
        assert_eq!(to_latex(&int(42)), "42");
        assert_eq!(to_latex(&rat(3, 4)), "\\frac{3}{4}");
        assert_eq!(to_latex(&rat(-3, 4)), "\\frac{-3}{4}");
        assert_eq!(to_latex(&sym("x")), "x");
        assert_eq!(to_latex(&sym("theta")), "\\theta");
        assert_eq!(to_latex(&Sym::Real(0.1)), "0.1");
        // جمع با ترم منفی تمیز
        let e = add(&[pow(&sym("x"), &int(2)), times(&[int(-3), sym("y")]), int(5)]);
        assert_eq!(to_latex(&e), "x^{2} - 3\\,y + 5");
    }

    #[test]
    fn goldens_products_and_quotients() {
        // ضریب صحیح
        assert_eq!(to_latex(&times(&[int(3), sym("x")])), "3\\,x");
        // ضریب گویا
        let e = times(&[rat(1, 3), sym("x")]);
        assert_eq!(to_latex(&e), "\\frac{1}{3}\\,x");
        // کسر
        let q = over(&sym("y"), &sym("x")).expect("t");
        assert_eq!(to_latex(&q), "\\frac{y}{x}");
        // کسر با ضریب: 2x/(x+1)
        let q2 = over(&times(&[int(2), sym("x")]), &add(&[sym("x"), int(1)])).expect("t");
        assert_eq!(to_latex(&q2), "\\frac{2\\,x}{x + 1}");
        // ضرب سمبل‌ها
        let m = times(&[sym("x"), sym("y")]);
        assert_eq!(to_latex(&m), "x\\,y");
        // کسر مخرج ترکیبی
        let q3 = over(&int(1), &add(&[pow(&sym("x"), &int(2)), int(-1)])).expect("t");
        assert_eq!(to_latex(&q3), "\\frac{1}{x^{2} - 1}");
    }

    #[test]
    fn goldens_powers() {
        assert_eq!(to_latex(&pow(&sym("x"), &int(3))), "x^{3}");
        // پرانتز حداقلی برای پایهٔ جمع
        assert_eq!(
            to_latex(&pow(&add(&[sym("x"), int(1)]), &int(2))),
            "\\left(x + 1\\right)^{2}"
        );
        // نمای گویا — آکولاد خود توان کافی است (بدون پرانتز)
        assert_eq!(to_latex(&pow(&sym("x"), &rat(1, 2))), "x^{\\frac{1}{2}}");
        // توان -۱ تنها → کسر
        assert_eq!(to_latex(&pow(&sym("x"), &int(-1))), "\\frac{1}{x}");
        // توان -۱ داخل ضرب → شکل تقسیم
        let e = times(&[sym("y"), pow(&sym("x"), &int(-1))]);
        assert_eq!(to_latex(&e), "\\frac{y}{x}");
        // پایهٔ توان‌دار — پرانتز برای جلوگیری از double-superscript
        // (نما نمادین است تا سازنده (x^2)^n را جمع نکند)
        let nested = pow(&pow(&sym("x"), &int(2)), &sym("n"));
        assert_eq!(to_latex(&nested), "\\left(x^{2}\\right)^{n}");
    }

    #[test]
    fn goldens_functions() {
        assert_eq!(to_latex(&f("sin", &[sym("x")])), "\\sin\\left(x\\right)");
        assert_eq!(
            to_latex(&f("cos", &[times(&[int(2), sym("x")])])),
            "\\cos\\left(2\\,x\\right)"
        );
        assert_eq!(
            to_latex(&f("asin", &[sym("x")])),
            "\\arcsin\\left(x\\right)"
        );
        assert_eq!(to_latex(&f("log", &[sym("x")])), "\\ln\\left(x\\right)");
        assert_eq!(
            to_latex(&f("log10", &[sym("x")])),
            "\\log_{10}\\left(x\\right)"
        );
        assert_eq!(
            to_latex(&f("sqrt", &[add(&[pow(&sym("x"), &int(2)), int(1)])])),
            "\\sqrt{x^{2} + 1}"
        );
        assert_eq!(to_latex(&f("abs", &[sym("x")])), "\\left|x\\right|");
        // exp متلب‌گونه — منفی یکه چسبیده در نمای توان
        assert_eq!(
            to_latex(&f("exp", &[neg(&pow(&sym("x"), &int(2)))])),
            "\\mathrm{e}^{-x^{2}}"
        );
    }

    #[test]
    fn golden_dod_demo() {
        // دموی DoD: diff(exp(-x^2)*x^3) با خروجی زیبا
        let e = times(&[
            f("exp", &[neg(&pow(&sym("x"), &int(2)))]),
            pow(&sym("x"), &int(3)),
        ]);
        let dd = diff::diff(&e, "x").expect("t");
        assert_eq!(
            to_latex(&dd),
            "-2\\,x^{4}\\,\\mathrm{e}^{-x^{2}} + 3\\,x^{2}\\,\\mathrm{e}^{-x^{2}}"
        );
    }

    #[test]
    fn goldens_special_reals() {
        assert_eq!(to_latex(&Sym::Real(f64::INFINITY)), "\\infty");
        assert_eq!(to_latex(&Sym::Real(f64::NEG_INFINITY)), "-\\infty");
        assert_eq!(to_latex(&Sym::Real(f64::NAN)), "\\mathrm{NaN}");
    }
}
