//! پایپ‌لاین ساده‌سازی CAS-1 (گام 4.2 — سند ۰۵ §۳): expand/collect/simplify.
//!
//! - `expand`: توزیع کامل روی فرم گویا — **بدون** حذف gcd (پاریت متلب:
//!   expand فقط گسترش می‌دهد؛ ساده‌سازی کسرها کار simplify است).
//! - `collect`: گروه‌بندی بر حسب توان متغیر (ضریب هر توان = چندجمله‌ای
//!   بقیهٔ ژنراتورها)؛ خروجی با ترتیب کلیدی پایدار درخت (سند ۰۵ §۲).
//! - `simplify`: فرم نرمال گویا + حذف gcd + قواعد ساختاریِ «کل-دامنه» فقط:
//!   sin²+cos²→1، exp(a)·exp(b)→exp(a+b)، log(exp(u))→u (روی ℝ همیشه درست).
//!   قواعد نیم‌دامنه (exp(log u)→u، sqrt(u)²→u) **بدون assume اعمال
//!   نمی‌شوند** — دقیقاً جایی که CASهای آماتور دروغ می‌گویند (سند ۰۵ §۳).

use std::collections::BTreeMap;

use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::{One, Zero};

use crate::poly::{poly_to_sym, Poly, RatFun};
use crate::sexpr;
use crate::{CasError, Sym};

/// expand — توزیع ضرب‌ها روی جمع‌ها (بدون حذف gcd؛ پاریت متلب).
pub fn expand(e: &Sym) -> Result<Sym, CasError> {
    let rf = RatFun::from_sym(e)?;
    rf.to_sym()
}

/// collect(S, x) — گسترش + گروه‌بندی بر حسب توان نزولی x؛ ضرایب هر توان،
/// چندجمله‌ای بقیهٔ ژنراتورها هستند.
pub fn collect(e: &Sym, var: &str) -> Result<Sym, CasError> {
    let mut rf = RatFun::from_sym(e)?;
    rf.cancel()?;
    let gen_key = sexpr::to_sexp(&Sym::symbol(var));
    let num = collect_poly(&rf.num, &gen_key, var)?;
    if let Some(d) = rf.den.constant_value() {
        if d == BigRational::from(BigInt::from(1)) {
            return Ok(num);
        }
        return Sym::over(num, Sym::Rational(d));
    }
    let den = collect_poly(&rf.den, &gen_key, var)?;
    Sym::over(num, den)
}

/// گروه‌بندی یک چندجمله‌ای بر حسب توان ژنراتور var (نزولی).
fn collect_poly(p: &Poly, gen_key: &str, var: &str) -> Result<Sym, CasError> {
    let mut by_deg: BTreeMap<i64, Poly> = BTreeMap::new();
    for (m, c) in &p.terms {
        let d = m.get(gen_key).copied().unwrap_or(0);
        let mut rest = m.clone();
        rest.remove(gen_key);
        let slot = by_deg.entry(d).or_default();
        *slot.terms.entry(rest).or_default() += c;
    }
    let x = Sym::symbol(var);
    let mut acc: Option<Sym> = None;
    for (d, mut coef) in by_deg.into_iter().rev() {
        coef.terms.retain(|_, c| !c.is_zero());
        if coef.terms.is_empty() {
            continue;
        }
        let csym = poly_to_sym(&coef)?;
        let term = if d == 0 {
            csym
        } else {
            let xp = Sym::pow(x.clone(), Sym::int(d))?;
            Sym::times(csym, xp)
        };
        acc = Some(match acc {
            None => term,
            Some(a) => Sym::plus(a, term),
        });
    }
    Ok(acc.unwrap_or_else(|| Sym::int(0)))
}

/// simplify — فرم نرمال گویا (حذف gcd) + قواعد ساختاری کل-دامنه.
pub fn simplify(e: &Sym) -> Result<Sym, CasError> {
    let mut cur = rebuild_children(e);
    for _ in 0..3 {
        // ۱) فرم نرمال گویا + حذف gcd (Real → صادقانه همین‌قدر که ممکن است)
        let mut next = match RatFun::from_sym(&cur) {
            Ok(mut rf) => {
                rf.cancel()?;
                rf.to_sym()?
            }
            Err(CasError::Unsupported(_)) => cur.clone(),
            Err(other) => return Err(other),
        };
        // ۲) قواعد ساختاری کل-دامنه — بازگشتی روی درخت
        next = apply_rules(&next);
        if next == cur {
            return Ok(next);
        }
        cur = rebuild_children(&next);
    }
    Ok(cur)
}

/// بازساخت درخت از پایین با سازنده‌های کانونی (سطح-۰ دوباره اعمال می‌شود).
fn rebuild_children(e: &Sym) -> Sym {
    match e {
        Sym::Integer(_) | Sym::Rational(_) | Sym::Real(_) | Sym::Symbol(_) => e.clone(),
        Sym::Add(terms) => terms
            .iter()
            .map(rebuild_children)
            .reduce(Sym::plus)
            .unwrap_or_else(|| Sym::int(0)),
        Sym::Mul(factors) => factors
            .iter()
            .map(rebuild_children)
            .reduce(Sym::times)
            .unwrap_or_else(|| Sym::int(1)),
        Sym::Pow(b, ex) => {
            Sym::pow(rebuild_children(b), rebuild_children(ex)).unwrap_or_else(|_| e.clone())
        }
        Sym::Fn(name, args) => Sym::fn_of(name, args.iter().map(rebuild_children).collect())
            .unwrap_or_else(|_| e.clone()),
    }
}

/// قواعد ساختاری — فقط قواعد معتبر روی تمام دامنه (بالای فایل).
fn apply_rules(e: &Sym) -> Sym {
    match e {
        Sym::Add(terms) => {
            let rebuilt: Vec<Sym> = terms.iter().map(apply_rules).collect();
            if rebuilt != *terms {
                rebuilt
                    .into_iter()
                    .reduce(Sym::plus)
                    .unwrap_or_else(|| Sym::int(0))
            } else {
                rule_sin_cos_add(e).unwrap_or_else(|| e.clone())
            }
        }
        Sym::Mul(factors) => {
            let rebuilt: Vec<Sym> = factors.iter().map(apply_rules).collect();
            if rebuilt != *factors {
                rebuilt
                    .into_iter()
                    .reduce(Sym::times)
                    .unwrap_or_else(|| Sym::int(1))
            } else {
                rule_exp_mul(e).unwrap_or_else(|| e.clone())
            }
        }
        Sym::Pow(b, ex) => Sym::pow(apply_rules(b), apply_rules(ex)).unwrap_or_else(|_| e.clone()),
        Sym::Fn(name, args) => {
            let new_args: Vec<Sym> = args.iter().map(apply_rules).collect();
            rule_log_exp(name, &new_args).unwrap_or_else(|| e.clone())
        }
        _ => e.clone(),
    }
}

/// ترم برچسب‌خوردهٔ sin²/cos²: (ضریب، عامل‌های دیگر، آرگومان).
struct TrigTerm {
    coeff: BigRational,
    others: Vec<Sym>,
    u: Sym,
    is_sin: bool,
}

/// sin(u)² + cos(u)² → 1 — با ضریب و عامل‌های مشترک:
/// c·r·sin(u)² + c·r·cos(u)² → c·r.
fn rule_sin_cos_add(e: &Sym) -> Option<Sym> {
    let Sym::Add(terms) = e else {
        return None;
    };
    let mut tags: Vec<(usize, TrigTerm)> = Vec::new();
    for (i, t) in terms.iter().enumerate() {
        let (coeff, rest) = split_term(t)?;
        let factors: Vec<Sym> = match &rest {
            Sym::Mul(v) => v.clone(),
            other => vec![other.clone()],
        };
        for (k, f) in factors.iter().enumerate() {
            let Sym::Pow(b, ex) = f else {
                continue;
            };
            if as_int(ex) != Some(2) {
                continue;
            }
            let Sym::Fn(name, args) = &**b else {
                continue;
            };
            let is_sin = match &**name {
                "sin" => true,
                "cos" => false,
                _ => continue,
            };
            let [u] = &args[..] else {
                continue;
            };
            let mut others = factors.clone();
            others.remove(k);
            tags.push((
                i,
                TrigTerm {
                    coeff: coeff.clone(),
                    others,
                    u: u.clone(),
                    is_sin,
                },
            ));
        }
    }
    for (i1, t1) in &tags {
        if !t1.is_sin {
            continue;
        }
        for (i2, t2) in &tags {
            if t2.is_sin || i1 == i2 {
                continue;
            }
            if t1.u == t2.u && t1.coeff == t2.coeff && t1.others == t2.others {
                let mut repl_factors = t1.others.clone();
                if !t1.coeff.is_one() {
                    repl_factors.insert(0, Sym::Rational(t1.coeff.clone()));
                }
                let repl = repl_factors
                    .into_iter()
                    .reduce(Sym::times)
                    .unwrap_or_else(|| Sym::int(1));
                let mut out: Vec<Sym> = Vec::new();
                for (i, t) in terms.iter().enumerate() {
                    if i == *i1 || i == *i2 {
                        continue;
                    }
                    out.push(t.clone());
                }
                out.push(repl);
                return Some(
                    out.into_iter()
                        .reduce(Sym::plus)
                        .unwrap_or_else(|| Sym::int(0)),
                );
            }
        }
    }
    None
}

/// exp(a)·exp(b)·rest → exp(a+b)·rest — جمع آرگومان‌ها (کل-دامنه).
fn rule_exp_mul(e: &Sym) -> Option<Sym> {
    let Sym::Mul(factors) = e else {
        return None;
    };
    let mut exp_args: Vec<Sym> = Vec::new();
    let mut rest: Vec<Sym> = Vec::new();
    for f in factors {
        if let Sym::Fn(name, args) = f {
            if &**name == "exp" && args.len() == 1 {
                exp_args.push(args[0].clone());
                continue;
            }
        }
        rest.push(f.clone());
    }
    if exp_args.len() < 2 {
        return None;
    }
    let sum = exp_args
        .into_iter()
        .reduce(Sym::plus)
        .unwrap_or_else(|| Sym::int(0));
    let merged = Sym::fn_of("exp", vec![sum]).ok()?;
    rest.push(merged);
    Some(
        rest.into_iter()
            .reduce(Sym::times)
            .unwrap_or_else(|| Sym::int(1)),
    )
}

/// log(exp(u)) → u — روی ℝ همیشه درست (exp>0). exp(log(u)) اعمال نمی‌شود
/// (نیازمند u>0 — گارد سند ۰۵ §۳).
fn rule_log_exp(name: &str, args: &[Sym]) -> Option<Sym> {
    if name != "log" || args.len() != 1 {
        return None;
    }
    if let Sym::Fn(inner, iargs) = &args[0] {
        if &**inner == "exp" && iargs.len() == 1 {
            return Some(iargs[0].clone());
        }
    }
    None
}

// ---------- کمکی‌ها ----------

/// تفکیک (ضریب گویا، بقیه) یک ترم جمع — هم‌ارز split_coeff سازنده‌ها.
fn split_term(t: &Sym) -> Option<(BigRational, Sym)> {
    if let Some(r) = t.as_exact() {
        return Some((r, Sym::int(1)));
    }
    if let Sym::Mul(factors) = t {
        let mut coeff = BigRational::from(BigInt::from(1));
        let mut rest: Vec<Sym> = Vec::new();
        for f in factors {
            if let Some(r) = f.as_exact() {
                coeff *= r;
            } else {
                rest.push(f.clone());
            }
        }
        let rsym = match rest.len() {
            0 => Sym::int(1),
            1 => rest.into_iter().next().unwrap_or_else(|| Sym::int(1)),
            _ => Sym::Mul(rest),
        };
        return Some((coeff, rsym));
    }
    Some((BigRational::from(BigInt::from(1)), t.clone()))
}

fn as_int(s: &Sym) -> Option<i64> {
    if let Sym::Integer(i) = s {
        use num_traits::ToPrimitive;
        return i.to_i64();
    }
    None
}

// ============================================================
// تست‌ها — مجموعهٔ مرجع ۵۰ عبارت (DoD سند ۰۵ §۱۱)
// ============================================================
#[cfg(test)]
mod tests {
    use crate::simplify::{collect, expand, simplify};
    use crate::testkit::*;

    fn sx(e: &Sym) -> String {
        simplify(e).expect("t").display()
    }

    #[test]
    fn reference_set_50_expressions() {
        // مجموعهٔ مرجع ساده‌سازی — ۵۰ عبارت با نتیجهٔ قطعی.
        // انتظارهای نمایشی دقیقاً ترتیب کلیدی پایدارِ درخت‌اند (سند ۰۵ §۲).
        let cases: Vec<(Sym, String)> = vec![
            // جمع/ضرب ضرایب و همانی‌ها (۱–۸)
            (add(&[sym("x"), int(0)]), "x".into()),
            (
                add(&[times(&[int(2), sym("x")]), times(&[int(3), sym("x")])]),
                "5*x".into(),
            ),
            (times(&[sym("x"), int(1)]), "x".into()),
            (pow(&sym("x"), &int(1)), "x".into()),
            (pow(&sym("x"), &int(0)), "1".into()),
            (add(&[rat(1, 3), rat(1, 6)]), "1/2".into()),
            (times(&[rat(1, 3), int(3)]), "1".into()),
            (times(&[int(2), rat(1, 4)]), "1/2".into()),
            // حذف gcd کسرهای گویا (۹–۱۶)
            (
                over(
                    &add(&[pow(&sym("x"), &int(2)), int(-1)]),
                    &add(&[sym("x"), int(-1)]),
                )
                .expect("t"),
                "x + 1".into(),
            ),
            (
                over(
                    &add(&[pow(&sym("x"), &int(2)), times(&[int(-2), sym("x")]), int(1)]),
                    &add(&[sym("x"), int(-1)]),
                )
                .expect("t"),
                "x - 1".into(),
            ),
            (
                over(&times(&[sym("x"), sym("y")]), &sym("x")).expect("t"),
                "y".into(),
            ),
            (
                over(
                    &add(&[times(&[sym("x"), sym("y")]), sym("x")]),
                    &add(&[sym("y"), int(1)]),
                )
                .expect("t"),
                "x".into(),
            ),
            (
                over(&int(1), &add(&[sym("x"), int(-1)])).expect("t"),
                "1/(x - 1)".into(),
            ),
            (
                add(&[
                    over(&int(1), &add(&[sym("x"), int(-1)])).expect("t"),
                    over(&int(1), &add(&[sym("x"), int(1)])).expect("t"),
                ]),
                "2*x/(x^2 - 1)".into(),
            ),
            (times(&[sym("x"), pow(&sym("x"), &int(-2))]), "1/x".into()),
            (
                over(&pow(&sym("x"), &int(3)), &pow(&sym("x"), &int(2))).expect("t"),
                "x".into(),
            ),
            // مثلثاتی کل-دامنه (۱۷–۲۰)
            (
                add(&[
                    pow(&f("sin", &[sym("x")]), &int(2)),
                    pow(&f("cos", &[sym("x")]), &int(2)),
                ]),
                "1".into(),
            ),
            (
                add(&[
                    times(&[int(3), pow(&f("sin", &[sym("x")]), &int(2))]),
                    times(&[int(3), pow(&f("cos", &[sym("x")]), &int(2))]),
                ]),
                "3".into(),
            ),
            (
                add(&[
                    pow(&f("sin", &[times(&[int(2), sym("x")])]), &int(2)),
                    pow(&f("cos", &[times(&[int(2), sym("x")])]), &int(2)),
                    sym("x"),
                ]),
                "x + 1".into(),
            ),
            (
                add(&[
                    pow(&f("sin", &[sym("x")]), &int(2)),
                    pow(&f("cos", &[sym("y")]), &int(2)),
                ]),
                "cos(y)^2 + sin(x)^2".into(),
            ),
            // exp/log (۲۱–۲۶)
            (
                times(&[
                    f("exp", &[sym("x")]),
                    f("exp", &[times(&[int(-1), sym("x")])]),
                ]),
                "1".into(),
            ),
            (
                times(&[f("exp", &[sym("x")]), f("exp", &[sym("y")])]),
                "exp(x + y)".into(),
            ),
            (
                f("log", &[f("exp", &[times(&[int(3), sym("x")])])]),
                "3*x".into(),
            ),
            (f("log", &[int(1)]), "0".into()),
            (f("exp", &[int(0)]), "1".into()),
            (f("sin", &[int(0)]), "0".into()),
            // توان‌ها و تاش‌های دقیق (۲۷–۳۳)
            (pow(&f("sqrt", &[sym("x")]), &int(2)), "sqrt(x)^2".into()),
            (
                times(&[f("sqrt", &[sym("x")]), f("sqrt", &[sym("x")])]),
                "sqrt(x)^2".into(),
            ),
            (pow(&int(4), &rat(1, 2)), "2".into()),
            (f("sqrt", &[int(4)]), "2".into()),
            (f("sqrt", &[rat(9, 4)]), "3/2".into()),
            (f("abs", &[int(-3)]), "3".into()),
            (f("sign", &[int(-5)]), "-1".into()),
            // گسترش و بازچینش (۳۴–۴۰)
            (
                times(&[add(&[sym("x"), int(1)]), add(&[sym("x"), int(2)])]),
                "x^2 + 3*x + 2".into(),
            ),
            (
                times(&[add(&[sym("x"), sym("y")]), add(&[sym("x"), neg(&sym("y"))])]),
                "x^2 - y^2".into(),
            ),
            (
                pow(&add(&[sym("x"), int(2)]), &int(2)),
                "x^2 + 4*x + 4".into(),
            ),
            (
                times(&[add(&[sym("x"), int(1)]), add(&[sym("x"), int(-1)])]),
                "x^2 - 1".into(),
            ),
            (
                add(&[
                    times(&[sym("y"), pow(&sym("x"), &int(2))]),
                    times(&[sym("y"), pow(&sym("x"), &int(2))]),
                    sym("x"),
                ]),
                "x + 2*y*x^2".into(),
            ),
            (
                add(&[
                    over(&sym("x"), &add(&[sym("x"), int(1)])).expect("t"),
                    over(&int(1), &add(&[sym("x"), int(1)])).expect("t"),
                ]),
                "1".into(),
            ),
            (
                over(
                    &add(&[pow(&sym("x"), &int(2)), int(1)]),
                    &add(&[sym("x"), int(1)]),
                )
                .expect("t"),
                "(x^2 + 1)/(x + 1)".into(),
            ),
            // چندجمله‌ای چندمتغیره (۴۱–۴۵)
            (
                times(&[add(&[sym("x"), int(1)]), add(&[sym("y"), int(1)])]),
                "x + y + x*y + 1".into(),
            ),
            (
                times(&[times(&[sym("x"), sym("y")]), add(&[sym("x"), int(2)])]),
                "2*x*y + y*x^2".into(),
            ),
            (
                pow(&add(&[times(&[int(2), sym("x")]), int(1)]), &int(3)),
                "12*x^2 + 6*x + 8*x^3 + 1".into(),
            ),
            (
                over(&add(&[times(&[int(2), sym("x")]), int(2)]), &int(2)).expect("t"),
                "x + 1".into(),
            ),
            (
                over(&times(&[int(6), sym("x")]), &times(&[int(4), sym("x")])).expect("t"),
                "3/2".into(),
            ),
            // Real تقریبی صادقانه (۴۶–۴۷)
            (times(&[Sym::Real(0.1), int(10)]), "1".into()),
            (add(&[Sym::Real(0.1), int(0)]), "0.1".into()),
            // عدم‌تقلب دامنه (۴۸–۵۰) — لایهٔ سازنده 0*x^-1 را حفظ می‌کند
            // (تست sym)؛ simplify صریح با قرارداد استاندارد CAS (متلب:
            // 0/x = 0، تکین‌شدنیِ حذف‌پذیر) به 0 می‌رسد — سیاست دولایه
            // مستند در سند ۰۵ §۳.
            (times(&[int(0), pow(&sym("x"), &int(-1))]), "0".into()),
            (f("sqrt", &[int(2)]), "sqrt(2)".into()),
            (f("asin", &[int(1)]), "asin(1)".into()),
        ];
        let mut failures = Vec::new();
        for (i, (input, want)) in cases.iter().enumerate() {
            let got = sx(input);
            if &got != want {
                failures.push(format!(
                    "#{}: {} → {} (منتظر {})",
                    i + 1,
                    input.display(),
                    got,
                    want
                ));
            }
        }
        assert!(
            failures.is_empty(),
            "مجموعهٔ مرجع ساده‌سازی شکست خورد:\n{}",
            failures.join("\n")
        );
        assert_eq!(cases.len(), 50, "مجموعه باید دقیقاً ۵۰ مورد باشد");
    }

    #[test]
    fn expand_no_gcd_cancel_matlab_parity() {
        // expand فقط گسترش می‌دهد — (x^2-1)/(x-1) دست‌نخورده می‌ماند
        let e = over(
            &add(&[pow(&sym("x"), &int(2)), int(-1)]),
            &add(&[sym("x"), int(-1)]),
        )
        .expect("t");
        assert_eq!(expand(&e).expect("t").display(), "(x^2 - 1)/(x - 1)");
        // اما توزیع کامل
        let p = times(&[add(&[sym("x"), int(1)]), add(&[sym("x"), int(2)])]);
        assert_eq!(expand(&p).expect("t").display(), "x^2 + 3*x + 2");
    }

    #[test]
    fn collect_groups_by_powers() {
        // x^2*y + x*y + 3 بر حسب x — ترتیب کلیدی پایدار درخت
        let e = add(&[
            times(&[sym("y"), pow(&sym("x"), &int(2))]),
            times(&[sym("y"), sym("x")]),
            int(3),
        ]);
        assert_eq!(collect(&e, "x").expect("t").display(), "x*y + y*x^2 + 3");
        // (x+1)*(x+2) بر حسب x
        let p = times(&[add(&[sym("x"), int(1)]), add(&[sym("x"), int(2)])]);
        assert_eq!(collect(&p, "x").expect("t").display(), "x^2 + 3*x + 2");
        // بر حسب y وقتی y فقط ضریب است
        let q = add(&[times(&[sym("y"), sym("x")]), sym("x")]);
        assert_eq!(collect(&q, "y").expect("t").display(), "x + x*y");
    }

    #[test]
    fn simplify_real_tree_best_effort() {
        // درخت تقریبی — بدون فرم نرمال، ولی قواعد ساختاری باز کار می‌کنند
        let e = times(&[f("exp", &[Sym::Real(0.1)]), f("exp", &[sym("x")])]);
        assert_eq!(sx(&e), "exp(x + 0.1)");
    }
}
