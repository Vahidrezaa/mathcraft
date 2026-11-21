//! سری‌سازی S-اجرِ کانونی درخت Sym (گام 4.2/CAS-1 — سند ۰۵ §۲).
//!
//! سه مصرف: کلید ژنراتورهای چندجمله‌ایِ تعمیم‌یافته (نمای یک‌به‌یکِ زیردرخت)،
//! بار KIND_SYM در MCWS (سند ۰۵ §۱۲) و تبادل با موتور e-graph (پشت پرچم).
//! قالب قطعی و بدون ابهام است — دو درخت کانونی یکسان همیشه رشتهٔ برابر
//! می‌دهند و هر رشته فقط یک درخت می‌سازد.

use std::fmt::Write as _;
use std::sync::Arc;

use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::{Num, Zero};

use crate::{CasError, Sym};

/// درخت → S-اجر کانونی.
pub fn to_sexp(s: &Sym) -> String {
    let mut out = String::new();
    write_sexp(&mut out, s);
    out
}

fn write_sexp(out: &mut String, s: &Sym) {
    match s {
        Sym::Integer(i) => {
            out.push_str("(int ");
            out.push_str(&i.to_string());
            out.push(')');
        }
        Sym::Rational(r) => {
            out.push_str("(rat ");
            out.push_str(&r.numer().to_string());
            out.push(' ');
            out.push_str(&r.denom().to_string());
            out.push(')');
        }
        Sym::Real(x) => {
            out.push_str("(real 0x");
            let _ = write!(out, "{:016x}", x.to_bits());
            out.push(')');
        }
        Sym::Symbol(n) => {
            out.push_str("(sym ");
            out.push_str(n);
            out.push(')');
        }
        Sym::Add(terms) => {
            out.push_str("(+");
            for t in terms {
                out.push(' ');
                write_sexp(out, t);
            }
            out.push(')');
        }
        Sym::Mul(factors) => {
            out.push_str("(*");
            for f in factors {
                out.push(' ');
                write_sexp(out, f);
            }
            out.push(')');
        }
        Sym::Pow(b, e) => {
            out.push_str("(^ ");
            write_sexp(out, b);
            out.push(' ');
            write_sexp(out, e);
            out.push(')');
        }
        Sym::Fn(f, args) => {
            out.push_str("(fn ");
            out.push_str(f);
            for a in args {
                out.push(' ');
                write_sexp(out, a);
            }
            out.push(')');
        }
    }
}

/// S-اجر → درخت — خطا روی قالب خراب/ناشناخته (هرگز حدس نمی‌زنیم).
pub fn from_sexp(src: &str) -> Result<Sym, CasError> {
    let tokens = tokenize(src)?;
    let mut pos = 0usize;
    let s = parse_expr(&tokens, &mut pos)?;
    if pos != tokens.len() {
        return Err(CasError::Serde("trailing tokens in serialized expression"));
    }
    Ok(s)
}

/// نشانه‌ا: پرانتز باز/بسته یا اتم (هر توالی نا-فضاهی).
fn tokenize(src: &str) -> Result<Vec<String>, CasError> {
    let mut out = Vec::new();
    let mut cur = String::new();
    for c in src.chars() {
        if c == '(' || c == ')' {
            if !cur.is_empty() {
                out.push(std::mem::take(&mut cur));
            }
            out.push(c.to_string());
        } else if c.is_whitespace() {
            if !cur.is_empty() {
                out.push(std::mem::take(&mut cur));
            }
        } else {
            cur.push(c);
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    if out.is_empty() {
        return Err(CasError::Serde("empty serialized expression"));
    }
    Ok(out)
}

fn parse_expr(tokens: &[String], pos: &mut usize) -> Result<Sym, CasError> {
    let head = tokens
        .get(*pos)
        .ok_or(CasError::Serde("unexpected end of serialized expression"))?;
    if head != "(" {
        return Err(CasError::Serde("expected '(' in serialized expression"));
    }
    *pos += 1;
    let op = tokens
        .get(*pos)
        .ok_or(CasError::Serde("unexpected end of serialized expression"))?
        .clone();
    *pos += 1;
    let s = match op.as_str() {
        "int" => {
            let a = atom(tokens, pos)?;
            Sym::Integer(
                BigInt::from_str_radix(&a, 10)
                    .map_err(|_| CasError::Serde("bad integer in serialized expression"))?,
            )
        }
        "rat" => {
            let n = atom(tokens, pos)?;
            let d = atom(tokens, pos)?;
            let den = BigInt::from_str_radix(&d, 10).map_err(|_| {
                CasError::Serde("bad rational denominator in serialized expression")
            })?;
            if den.is_zero() {
                return Err(CasError::Serde("zero denominator in serialized expression"));
            }
            let num = BigInt::from_str_radix(&n, 10)
                .map_err(|_| CasError::Serde("bad rational numerator in serialized expression"))?;
            Sym::Rational(BigRational::new(num, den))
        }
        "real" => {
            let a = atom(tokens, pos)?;
            let hex = a
                .strip_prefix("0x")
                .ok_or(CasError::Serde("bad real bits in serialized expression"))?;
            let bits = u64::from_str_radix(hex, 16)
                .map_err(|_| CasError::Serde("bad real bits in serialized expression"))?;
            Sym::Real(f64::from_bits(bits))
        }
        "sym" => {
            let a = atom(tokens, pos)?;
            Sym::Symbol(Arc::from(a.as_str()))
        }
        "+" | "*" => {
            let mut items = Vec::new();
            while tokens.get(*pos).map(|t| t.as_str()) != Some(")") {
                items.push(parse_expr(tokens, pos)?);
            }
            if items.len() < 2 {
                return Err(CasError::Serde(
                    "canonical +/- nodes need at least two children",
                ));
            }
            if op == "+" {
                let mut acc = items.remove(0);
                for t in items {
                    acc = Sym::plus(acc, t);
                }
                acc
            } else {
                let mut acc = items.remove(0);
                for f in items {
                    acc = Sym::times(acc, f);
                }
                acc
            }
        }
        "^" => {
            let b = parse_expr(tokens, pos)?;
            let e = parse_expr(tokens, pos)?;
            Sym::pow(b, e)?
        }
        "fn" => {
            let name = atom(tokens, pos)?;
            let mut args = Vec::new();
            while tokens.get(*pos).map(|t| t.as_str()) != Some(")") {
                args.push(parse_expr(tokens, pos)?);
            }
            if args.is_empty() {
                return Err(CasError::Serde("fn node needs at least one argument"));
            }
            Sym::fn_of(&name, args)?
        }
        other => {
            let _ = other;
            return Err(CasError::Serde(
                "unknown node kind in serialized expression",
            ));
        }
    };
    expect_close(tokens, pos)?;
    Ok(s)
}

/// اتمِ بعدی (بدون پرانتز).
fn atom(tokens: &[String], pos: &mut usize) -> Result<String, CasError> {
    let t = tokens
        .get(*pos)
        .ok_or(CasError::Serde("unexpected end of serialized expression"))?;
    if t == "(" || t == ")" {
        return Err(CasError::Serde("expected an atom in serialized expression"));
    }
    *pos += 1;
    Ok(t.clone())
}

fn expect_close(tokens: &[String], pos: &mut usize) -> Result<(), CasError> {
    match tokens.get(*pos).map(|t| t.as_str()) {
        Some(")") => {
            *pos += 1;
            Ok(())
        }
        _ => Err(CasError::Serde("expected ')' in serialized expression")),
    }
}

// ============================================================
// تست‌ها
// ============================================================
#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit::*;

    #[test]
    fn roundtrip_simple() {
        for (name, s) in [
            ("int", &Sym::int(-42)),
            ("rat", &Sym::rat(3, 4).expect("t")),
            ("real", &Sym::Real(0.1)),
            ("symbol", &Sym::symbol("theta")),
        ] {
            let sexp = to_sexp(s);
            let back = from_sexp(&sexp).expect("t");
            assert_eq!(&back, s, "roundtrip {name}");
        }
    }

    #[test]
    fn roundtrip_tree() {
        let cases = vec![
            add(&[pow(&sym("x"), &int(2)), times(&[int(3), sym("x")])]),
            f("exp", &[neg(&sym("x"))]),
            f("sin", &[sym("x")]),
            over(&sym("y"), &sym("x")).expect("t"),
        ];
        for s in cases {
            let sexp = to_sexp(&s);
            assert_eq!(from_sexp(&sexp).expect("t"), s, "roundtrip {sexp}");
        }
    }

    #[test]
    fn roundtrip_deep_and_big() {
        // درخت عمیق + اعداد بزرگ — KIND_SYM باید عقبل از ازای اعداد زنده بماند
        let big = Sym::Integer(BigInt::from(10).pow(400));
        let s = times(&[big, pow(&add(&[sym("x"), rat(1, 3)]), &int(7))]);
        let back = from_sexp(&to_sexp(&s)).expect("t");
        assert_eq!(back, s);
        // NaN هم دقیق رفته‌برمی‌گردد (بیتی، نه متنی — مقایسهٔ بیتی چون
        // PartialEq برای NaN همیشه نادرست است)
        let nan = Sym::Real(f64::NAN);
        match from_sexp(&to_sexp(&nan)).expect("t") {
            Sym::Real(v) => assert_eq!(v.to_bits(), f64::NAN.to_bits()),
            other => panic!("NaN رفت‌وبرگشت نشد: {other:?}"),
        }
    }

    #[test]
    fn sexpr_is_unambiguous_key() {
        // دو درخت متفاوت هرگز کلید برابر نمی‌گیرند
        let a = times(&[int(2), sym("x")]);
        let b = pow(&sym("x"), &int(2));
        assert_ne!(to_sexp(&a), to_sexp(&b));
        assert_eq!(to_sexp(&a), "(* (int 2) (sym x))");
        assert_eq!(to_sexp(&b), "(^ (sym x) (int 2))");
    }

    #[test]
    fn malformed_rejected() {
        assert!(from_sexp("").is_err());
        assert!(from_sexp("(int)").is_err());
        assert!(from_sexp("(rat 1 0)").is_err());
        assert!(from_sexp("(bogus 1)").is_err());
        assert!(from_sexp("(int 5) (int 6)").is_err());
        assert!(from_sexp("(+ (int 5)").is_err());
    }
}
