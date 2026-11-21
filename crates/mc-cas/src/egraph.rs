//! موتور e-graph پشت پرچم `egraph` (گام 4.2/CAS-1 — سند ۰۵ §۳ سطح ۱).
//!
//! اشباع برابری با کتابخانهٔ `egg`: همهٔ فرم‌های معادلِ قابل‌دستیابی از
//! قواعد، موازی در e-graph می‌مانند و در پایان بهترین (کوتاه‌ترین با
//! هزینهٔ AstSize) استخراج می‌شود — به دام مرحلهٔ میانی بد نمی‌افتد.
//!
//! **صداقت دامنه:** فقط قواعد معتبر روی تمام دامنه (یا با دامنهٔ دقیقاً
//! برابر) اینجا هستند — sin²+cos²→1، exp(a)·exp(b)→exp(a+b)،
//! log(exp(u))→u (روی ℝ)، sqrt(u)↔u^(1/2) (روی ℝ دامنه‌ها یکی‌اند).
//! قواعد نیم‌دامنه (exp(log u)→u، sqrt(u)²→u) ممنوع — همان مرز صداقتِ
//! لایهٔ قواعد. حساب ثابت‌ها (جمع/ضرب اعداد) قصداً نیست: برگ‌ها opaque
//! اند و فولد عددی مسیر سازنده‌های کانونی است، نه بازنویسی کور.
//!
//! بودجه (سند ۰۵ §۱۲ بند ۴): ورودی >۵۰۰ گره → خطای صادقانه؛ اجرای
//! اشباع با سقف تکرار ۳۰، سقف گره ۳۰هزار و مهلت ۱۰۰ms.

use std::fmt::Write as _;
use std::time::Duration;

use egg::{
    define_language, rewrite, AstSize, Extractor, Id, Language, RecExpr, Rewrite, Runner,
    Symbol as ESymbol,
};

use crate::{CasError, Sym};

define_language! {
    enum MathLang {
        "+" = Add([Id; 2]),
        "*" = Mul([Id; 2]),
        "^" = Pow([Id; 2]),
        "sin" = Sin(Id),
        "cos" = Cos(Id),
        "tan" = Tan(Id),
        "asin" = Asin(Id),
        "acos" = Acos(Id),
        "atan" = Atan(Id),
        "sinh" = Sinh(Id),
        "cosh" = Cosh(Id),
        "tanh" = Tanh(Id),
        "exp" = Exp(Id),
        "log" = Log(Id),
        "log10" = Log10(Id),
        "log2" = Log2(Id),
        "sqrt" = Sqrt(Id),
        "abs" = Abs(Id),
        "sign" = Sign(Id),
        // برگ عمومی: ثابت‌ها با توکن کانونی خود («3»، «1/3»، «r0x…»)،
        // نمادها با پیشوند s («sx») — تشابه توکن ناممکن است
        Leaf(ESymbol),
    }
}

/// قواعد — فقط کل-دامنه (بالای فایل).
fn rules() -> Vec<Rewrite<MathLang, ()>> {
    vec![
        rewrite!("add-zero"; "(+ ?a 0)" => "?a"),
        rewrite!("add-zero-r"; "(+ 0 ?a)" => "?a"),
        rewrite!("mul-one"; "(* ?a 1)" => "?a"),
        rewrite!("mul-one-r"; "(* 1 ?a)" => "?a"),
        rewrite!("pow-one"; "(^ ?a 1)" => "?a"),
        rewrite!("pow-zero"; "(^ ?a 0)" => "1"),
        rewrite!("add-comm"; "(+ ?a ?b)" => "(+ ?b ?a)"),
        rewrite!("mul-comm"; "(* ?a ?b)" => "(* ?b ?a)"),
        rewrite!("mul-square"; "(* ?a ?a)" => "(^ ?a 2)"),
        rewrite!("exp-merge"; "(* (exp ?a) (exp ?b))" => "(exp (+ ?a ?b))"),
        rewrite!("sin-cos"; "(+ (^ (sin ?u) 2) (^ (cos ?u) 2))" => "1"),
        rewrite!("sin-cos-r"; "(+ (^ (cos ?u) 2) (^ (sin ?u) 2))" => "1"),
        rewrite!("sin-cos-c"; "(+ (* ?c (^ (sin ?u) 2)) (* ?c (^ (cos ?u) 2)))" => "(* ?c 1)"),
        rewrite!("sin-cos-c-r"; "(+ (* ?c (^ (cos ?u) 2)) (* ?c (^ (sin ?u) 2)))" => "(* ?c 1)"),
        rewrite!("log-exp"; "(log (exp ?u))" => "?u"),
        rewrite!("sqrt-half"; "(sqrt ?u)" => "(^ ?u 1/2)"),
        rewrite!("half-sqrt"; "(^ ?u 1/2)" => "(sqrt ?u)"),
    ]
}

/// درخت → متن sexpr خودِ egg.
fn to_egg_text(s: &Sym, out: &mut String) {
    match s {
        Sym::Integer(i) => out.push_str(&i.to_string()),
        Sym::Rational(r) => {
            out.push_str(&r.numer().to_string());
            out.push('/');
            out.push_str(&r.denom().to_string());
        }
        Sym::Real(x) => {
            let _ = write!(out, "r{:016x}", x.to_bits());
        }
        Sym::Symbol(n) => {
            out.push('s');
            out.push_str(n);
        }
        Sym::Add(terms) => nested_binary(out, "+", terms),
        Sym::Mul(factors) => nested_binary(out, "*", factors),
        Sym::Pow(b, e) => {
            out.push_str("(^ ");
            to_egg_text(b, out);
            out.push(' ');
            to_egg_text(e, out);
            out.push(')');
        }
        Sym::Fn(name, args) => {
            out.push('(');
            out.push_str(name);
            for a in args {
                out.push(' ');
                to_egg_text(a, out);
            }
            out.push(')');
        }
    }
}

/// (+ a b c) → (+ a (+ b c)) — دوبایتی‌سازی چپ‌گرا.
fn nested_binary(out: &mut String, op: &str, items: &[Sym]) {
    let mut buf = String::new();
    for (i, it) in items.iter().enumerate().rev() {
        if i == items.len() - 1 {
            buf = it_to_text(it);
        } else {
            buf = format!("({op} {} {buf})", it_to_text(it));
        }
    }
    out.push_str(&buf);
}

fn it_to_text(s: &Sym) -> String {
    let mut out = String::new();
    to_egg_text(s, &mut out);
    out
}

/// برگ توکن → Sym («3»، «1/3»، «r0x…»، «sx»).
fn leaf_sym(tok: &str) -> Result<Sym, CasError> {
    if let Some(name) = tok.strip_prefix('s') {
        return Ok(Sym::symbol(name));
    }
    if let Some(hex) = tok.strip_prefix('r') {
        let bits = u64::from_str_radix(hex, 16)
            .map_err(|_| CasError::Serde("bad real token in e-graph"))?;
        return Ok(Sym::Real(f64::from_bits(bits)));
    }
    if let Some(slash) = tok.find('/') {
        let (n, d) = tok.split_at(slash);
        let den = d[1..]
            .parse::<i64>()
            .map_err(|_| CasError::Serde("bad rational token in e-graph"))?;
        return Sym::rat(
            n.parse::<i64>()
                .map_err(|_| CasError::Serde("bad rational token in e-graph"))?,
            den,
        );
    }
    tok.parse::<i64>()
        .map(|i| Sym::int(i))
        .map_err(|_| CasError::Serde("bad number token in e-graph"))
}

/// RecExpr → درخت کانونی (بازساخت با سازنده‌ها).
fn sym_from_recexpr(expr: &RecExpr<MathLang>) -> Result<Sym, CasError> {
    let nodes = expr.as_ref();
    let mut built: Vec<Sym> = Vec::with_capacity(nodes.len());
    // فرزند i — همیشه کلون (گره‌ها می‌توانند والدِ مشترک باشند)
    let ch = |built: &Vec<Sym>, i: &Id| -> Result<Sym, CasError> {
        built
            .get(usize::from(*i))
            .cloned()
            .ok_or(CasError::Serde("dangling child id in e-graph"))
    };
    for node in nodes {
        let s = match node {
            MathLang::Add([a, b]) => Sym::plus(ch(&built, a)?, ch(&built, b)?),
            MathLang::Mul([a, b]) => Sym::times(ch(&built, a)?, ch(&built, b)?),
            MathLang::Pow([a, b]) => Sym::pow(ch(&built, a)?, ch(&built, b)?)?,
            MathLang::Sin(a) => Sym::fn_of("sin", vec![ch(&built, a)?])?,
            MathLang::Cos(a) => Sym::fn_of("cos", vec![ch(&built, a)?])?,
            MathLang::Tan(a) => Sym::fn_of("tan", vec![ch(&built, a)?])?,
            MathLang::Asin(a) => Sym::fn_of("asin", vec![ch(&built, a)?])?,
            MathLang::Acos(a) => Sym::fn_of("acos", vec![ch(&built, a)?])?,
            MathLang::Atan(a) => Sym::fn_of("atan", vec![ch(&built, a)?])?,
            MathLang::Sinh(a) => Sym::fn_of("sinh", vec![ch(&built, a)?])?,
            MathLang::Cosh(a) => Sym::fn_of("cosh", vec![ch(&built, a)?])?,
            MathLang::Tanh(a) => Sym::fn_of("tanh", vec![ch(&built, a)?])?,
            MathLang::Exp(a) => Sym::fn_of("exp", vec![ch(&built, a)?])?,
            MathLang::Log(a) => Sym::fn_of("log", vec![ch(&built, a)?])?,
            MathLang::Log10(a) => Sym::fn_of("log10", vec![ch(&built, a)?])?,
            MathLang::Log2(a) => Sym::fn_of("log2", vec![ch(&built, a)?])?,
            MathLang::Sqrt(a) => Sym::fn_of("sqrt", vec![ch(&built, a)?])?,
            MathLang::Abs(a) => Sym::fn_of("abs", vec![ch(&built, a)?])?,
            MathLang::Sign(a) => Sym::fn_of("sign", vec![ch(&built, a)?])?,
            MathLang::Leaf(sym) => leaf_sym(&sym.to_string())?,
        };
        built.push(s);
    }
    built
        .pop()
        .ok_or(CasError::Serde("empty e-graph expression"))
}

/// ساده‌سازی با اشباع برابری — خروجی هم‌ارزِ در-دامنهٔ ورودی (قواعد
/// کل-دامنه فقط). پرچم ساخت: `--features egraph`.
pub fn simplify_egraph(e: &Sym) -> Result<Sym, CasError> {
    let text = it_to_text(e);
    let expr: RecExpr<MathLang> = text
        .parse()
        .map_err(|_| CasError::Serde("e-graph could not parse the canonical expression"))?;
    if expr.as_ref().len() > 500 {
        return Err(CasError::Unsupported(
            "expression larger than 500 nodes; e-graph simplification skipped (doc 05 §12)",
        ));
    }
    let runner: Runner<MathLang, ()> = Runner::default()
        .with_iter_limit(30)
        .with_node_limit(30_000)
        .with_time_limit(Duration::from_millis(100))
        .with_expr(&expr)
        .run(&rules());
    let root = runner
        .roots
        .first()
        .copied()
        .ok_or(CasError::Serde("e-graph produced no root"))?;
    let (_cost, best) = Extractor::new(&runner.egraph, AstSize).find_best(root);
    sym_from_recexpr(&best)
}

// ============================================================
// تست‌ها (فقط با --features egraph)
// ============================================================
#[cfg(test)]
mod tests {
    use crate::egraph::simplify_egraph;
    use crate::testkit::*;

    #[test]
    fn roundtrip_preserves_tree() {
        // بدون قاعدهٔ قابل‌اجرا، همان درخت برمی‌گردد (کانونی‌سازی دوباره)
        let e = add(&[pow(&sym("x"), &int(2)), times(&[int(3), sym("x")])]);
        assert_eq!(simplify_egraph(&e).expect("t").display(), "x^2 + 3*x");
    }

    #[test]
    fn sin_cos_square_to_one() {
        let e = add(&[
            pow(&f("sin", &[sym("x")]), &int(2)),
            pow(&f("cos", &[sym("x")]), &int(2)),
        ]);
        assert_eq!(simplify_egraph(&e).expect("t").display(), "1");
        // با ضریب برابر از طریق استخراج: 2sin²+2cos² → 2
        let e2 = add(&[
            times(&[int(2), pow(&f("sin", &[sym("x")]), &int(2))]),
            times(&[int(2), pow(&f("cos", &[sym("x")]), &int(2))]),
        ]);
        assert_eq!(simplify_egraph(&e2).expect("t").display(), "2");
    }

    #[test]
    fn exp_merge_and_log_exp() {
        let e = times(&[f("exp", &[sym("x")]), f("exp", &[sym("y")])]);
        assert_eq!(simplify_egraph(&e).expect("t").display(), "exp(x + y)");
        let e2 = f("log", &[f("exp", &[times(&[int(3), sym("x")])])]);
        assert_eq!(simplify_egraph(&e2).expect("t").display(), "3*x");
    }

    #[test]
    fn negative_and_real_leaves_survive() {
        // توان منفی و Real تقریبی — توکن‌ها بی‌اتلاف رفت‌وبرگشت می‌کنند
        let e = times(&[Sym::Real(0.1), pow(&sym("x"), &int(-2))]);
        let back = simplify_egraph(&e).expect("t");
        assert_eq!(back.display(), "0.1*x^-2");
    }

    #[test]
    fn budget_guard_honest() {
        // >۵۰۰ گره → خطای صادقانه (سند ۰۵ §۱۲ بند ۴) — نمادهای متمایز
        // تا جمع ضرایب درخت را کوچک نکند
        let mut e = Sym::symbol("a0");
        for i in 1..600 {
            e = Sym::plus(e, Sym::symbol(&format!("a{i}")));
        }
        assert!(matches!(simplify_egraph(&e), Err(CasError::Unsupported(_))));
    }
}
