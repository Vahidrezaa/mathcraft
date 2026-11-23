//! گام 4.4/CAS-3 فاز ۲ — `dsolve`: حل معادلهٔ دیفرانسیل معمولی (سند ۰۵ §۹).
//!
//! زیرمجموعهٔ دقیق سند ۰۵ §۹:
//!   * مرتبهٔ اول:
//!     - بی‌وابستگی به y → انتگرال‌گیری مستقیم؛
//!     - جداپذیر `y′ = f(x)·g(y)` → جداسازی + انتگرال دو طرف + وارون‌سازی؛
//!     - کامل `M dx + N dy = 0` با چک `M_y = N_x` و فاکتور انتگرال‌ساز
//!       سادهٔ `μ(x)` یا `μ(y)`؛
//!     - خطی `y′ + p(x)y = q(x)` با فاکتور انتگرال‌ساز.
//!   * خطی با ضرایب ثابت هر مرتبه: معادلهٔ مشخصه (ریشهٔ گویا + فرمول درجه ۲
//!     با گارد مختلط) → جواب عمومی (حقیقی/مختلط/تکراری) + ضرایب نامعین برای
//!     RHS ∈ {چندجمله‌ای، e^{ax}، sin/cos، حاصل‌ضرب‌ها} با گارد تشدید.
//!
//! **قرارداد ورودی (فاز ۲ — سطح موتور):** معادله در فرم `expr = 0` با گره‌های
//! کانونی `Fn("diff", [y(x), x])` و تو در تو برای مرتبه‌های بالاتر؛ مجهول،
//! تابع تک‌آرگومانِ ناشناختهٔ `y(x)` است. مشتقِ عبارت‌های حاوی `y(x)` پذیرفته
//! نیست — `diff` موتور روی تابع ناشناخته خطای صادقانه می‌دهد (درس CAS-1) و
//! سیم‌کشی مفسر در فاز ۳ مسئول ساخت گره‌های کانونی از دستور `Dy/D2y` است.
//!
//! **خروجی:** فهرست شاخه‌های صریح `y(x)` با ثابت‌های `C1, C2, …` (پاریت
//! چندشاخه‌ای — مثلاً `y′ = x/y` دو شاخه ± می‌دهد). هر شاخه پیش از بازگشت
//! ورفای می‌شود: نمادین (`simplify(residual) = 0`) یا عددی روی نمونه‌های x با
//! جایگذاری نمونه‌ای ثابت‌ها. شاخهٔ ورفای‌نشده هرگز برنمی‌گردد — صداقت در
//! شکست (سند ۰۰ اصل ۵). فرم‌های خارج از زیرمجموعه (Bernoulli/Riccati،
//! ضرایب متغیر) پیام صریح + اشارهٔ `ode45` می‌دهند.
//!
//! **یادداشت طبقه‌بندی (صادقانه):** کلاس‌های مرتبهٔ اول هم‌پوشانی دارند
//! (هر معادلهٔ دقیقی که F آن در y خطی است، خطی هم هست). ترتیب آشکارساز:
//! انتگرال مستقیم → جداپذیر → کامل/μ → خطی. شماره‌گذاری کلاس در مرجع‌ها
//! بر اساس آشکارسازِ فعال است، نه فرم ریاضی.

use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::{Signed, Zero};

use crate::diff::diff_n;
use crate::int::{contains, integrate, is_sym_zero};
use crate::poly::{Poly, RatFun};
use crate::sexpr::to_sexp;
use crate::simplify::simplify;
use crate::solve::solve_expr;
use crate::sym::{CasError, Sym};

/// ضریب تجزیهٔ پاسخ: (نوع تریگ، گویا) + درجه + ضریب.
type RespDec = ((bool, Option<BigRational>), i64, BigRational);

/// نام گرهٔ مشتق کانونی.
const DIFF_FN: &str = "diff";
/// پیشوند اتم مشتق (D1y، D2y، …).
const D_ATOM_PREFIX: &str = "D";

// ---------------------------------------------------------------------------
// تجزیهٔ معادلهٔ ODE — گره‌های مشتق → اتم‌های D1y/D2y/…
// ---------------------------------------------------------------------------

/// معادلهٔ ODE تجزیه‌شده.
pub(crate) struct ParsedOde {
    /// نام تابع مجهول («y»).
    pub(crate) f: String,
    /// متغیر مستقل («x»).
    pub(crate) var: String,
    /// مرتبهٔ بیشینهٔ مشتق حاضر در معادله.
    pub(crate) order: usize,
    /// معادلهٔ اتمیزه‌شده در فرم `expr = 0` — مجهول `y`، مشتق‌ها `D1y`… .
    pub(crate) norm: Sym,
}

/// نتیجهٔ یک‌بار پیمایش: نام مجهول، متغیر و نقشهٔ گرهٔ مشتق → اتم.
#[derive(Default)]
struct Scan {
    f: Option<String>,
    var: Option<String>,
    /// کلید S-اجر گرهٔ مشتق کانونی → نام اتم (D1y، …).
    d_atoms: Vec<(String, String)>,
}

/// ورودی ODE را تجزیه می‌کند: مجهول، متغیر، مرتبه و فرم اتمیزهٔ `expr = 0`.
pub(crate) fn parse_ode(eq: &Sym) -> Result<ParsedOde, CasError> {
    let e = simplify(eq)?;
    let mut scan = Scan::default();
    scan_node(&e, &mut scan)?;
    let f = scan
        .f
        .clone()
        .ok_or(CasError::Unsupported(
            "dsolve: no unknown single-argument function found — write the ODE in terms of y(x) and diff(y(x), x).",
        ))?;
    let var = scan.var.clone().ok_or(CasError::Unsupported(
        "dsolve: the independent variable of the unknown function is not a plain symbol.",
    ))?;
    if f == var {
        return Err(CasError::Unsupported(
            "dsolve: the unknown function name and the independent variable must differ.",
        ));
    }
    // اتم‌ها نباید با نمادهای آزادِ معادله تصادم کنند — وگرنه جایگذاری
    // معنا را عوض می‌کند؛ خطای صادقانه بهتر از جوابِ غلطِ مطمئن‌نما.
    let atom_names: Vec<String> = scan
        .d_atoms
        .iter()
        .map(|(_, a)| a.clone())
        .chain(std::iter::once(f.clone()))
        .collect();
    for s in e.free_symbols() {
        if atom_names.contains(&s) {
            return Err(CasError::Unsupported(
                "dsolve: a helper atom name collides with a free symbol of the equation — rename the function or the symbol.",
            ));
        }
    }
    let norm = rewrite(&e, &scan)?;
    // مرتبه = بیشینهٔ شمارهٔ اتم مشتق حاضر در فرم نهایی — اتم «D{k}{f}».
    let mut order = 0usize;
    for s in norm.free_symbols() {
        if let Some(num) = s.strip_prefix(D_ATOM_PREFIX) {
            if let Some(k) = num.strip_suffix(&f) {
                if let Ok(k) = k.parse::<usize>() {
                    if k > 0 {
                        order = order.max(k);
                    }
                }
            }
        }
    }
    if order == 0 {
        return Err(CasError::Unsupported(
            "dsolve: no derivative of the unknown function found — this is not a differential equation.",
        ));
    }
    Ok(ParsedOde {
        f,
        var,
        order,
        norm,
    })
}

/// پیمایش بازگشتی: مجهول و گره‌های مشتق را ثبت می‌کند (بدون بازنویسی).
fn scan_node(e: &Sym, scan: &mut Scan) -> Result<(), CasError> {
    match e {
        Sym::Symbol(_) | Sym::Integer(_) | Sym::Rational(_) | Sym::Real(_) => Ok(()),
        Sym::Add(ts) => {
            for t in ts {
                scan_node(t, scan)?;
            }
            Ok(())
        }
        Sym::Mul(fs) => {
            for t in fs {
                scan_node(t, scan)?;
            }
            Ok(())
        }
        Sym::Pow(b, ex) => {
            scan_node(b, scan)?;
            scan_node(ex, scan)
        }
        Sym::Fn(name, args) => {
            if name.as_ref() == DIFF_FN {
                return scan_diff(e, args, 1, scan);
            }
            // توابع جدولی (exp/log/sin/…) مجهول نیستند — فقط آرگومان‌ها.
            if crate::fntable::is_symbolic_fn(name) {
                for a in args {
                    scan_node(a, scan)?;
                }
                return Ok(());
            }
            // گرهٔ int/e-graph اُپک در معادلهٔ ODE پذیرفته نیست.
            if name.as_ref() == "int" {
                return Err(CasError::Unsupported(
                    "dsolve: unevaluated int() inside an ODE is not supported.",
                ));
            }
            let [a] = &args[..] else {
                return Err(CasError::Unsupported(
                    "dsolve: the unknown function must appear as single-argument applications like y(x).",
                ));
            };
            let Sym::Symbol(v) = a else {
                return Err(CasError::Unsupported(
                    "dsolve: the unknown function argument must be the independent variable symbol.",
                ));
            };
            match &scan.f {
                Some(prev) if prev != name.as_ref() => {
                    return Err(CasError::Unsupported(
                        "dsolve: more than one unknown function in one equation is ambiguous.",
                    ));
                }
                Some(_) => {}
                None => {
                    if crate::fntable::is_symbolic_fn(name) {
                        return Err(CasError::Unsupported(
                            "dsolve: built-in symbolic functions cannot be the unknown function.",
                        ));
                    }
                    scan.f = Some(name.to_string());
                    scan.var = Some(v.to_string());
                }
            }
            if scan.var.as_deref() != Some(v.as_ref()) {
                return Err(CasError::Unsupported(
                    "dsolve: the unknown function is applied to more than one independent variable.",
                ));
            }
            Ok(())
        }
    }
}

/// گرهٔ مشتق: `diff(u, x)` — تو در تو تا تابع مجهول؛ وگرنه خارج از قرارداد.
/// `node` خودِ گرهٔ diff است (برای کلید S-اجر اتم).
fn scan_diff(node: &Sym, args: &[Sym], depth: usize, scan: &mut Scan) -> Result<(), CasError> {
    if args.len() != 2 {
        return Err(CasError::Unsupported(
            "dsolve: derivative nodes take exactly two arguments: diff(y(x), x).",
        ));
    }
    let Sym::Symbol(v) = &args[1] else {
        return Err(CasError::Unsupported(
            "dsolve: the second argument of diff must be the independent variable symbol.",
        ));
    };
    if let Some(prev) = &scan.var {
        if prev != v.as_ref() {
            return Err(CasError::Unsupported(
                "dsolve: derivatives with respect to more than one variable are not supported.",
            ));
        }
    } else {
        scan.var = Some(v.to_string());
    }
    let ok = match &args[0] {
        Sym::Fn(name, inner) if name.as_ref() == DIFF_FN => {
            scan_diff(node, inner, depth + 1, scan)
        }
        // نقطهٔ پایانی: تابع مجهول — ثبت نام و متغیر.
        Sym::Fn(name, inner) if inner.len() == 1 => {
            let Sym::Symbol(fv) = &inner[0] else {
                return Err(CasError::Unsupported(
                    "dsolve: the unknown function argument must be the independent variable symbol.",
                ));
            };
            if scan.var.as_deref() != Some(fv.as_ref()) {
                return Err(CasError::Unsupported(
                    "dsolve: the unknown function is applied to more than one independent variable.",
                ));
            }
            match &scan.f {
                Some(prev) if prev != name.as_ref() => {
                    return Err(CasError::Unsupported(
                        "dsolve: more than one unknown function in one equation is ambiguous.",
                    ));
                }
                Some(_) => {}
                None => {
                    if crate::fntable::is_symbolic_fn(name) {
                        return Err(CasError::Unsupported(
                            "dsolve: built-in symbolic functions cannot be the unknown function.",
                        ));
                    }
                    scan.f = Some(name.to_string());
                }
            }
            Ok(())
        }
        _ => Err(CasError::Unsupported(
            "dsolve: only diff(y(x), x) and nested forms like diff(diff(y(x), x), x) are accepted — write the ODE in canonical derivative form.",
        )),
    };
    ok?;
    // ثبت اتم این گره مشتق (کلید یک‌به‌یک S-اجر).
    let atom = format!(
        "{D_ATOM_PREFIX}{depth}{}",
        scan.f.clone().unwrap_or_default()
    );
    let key = to_sexp(node);
    if !scan.d_atoms.iter().any(|(k, _)| *k == key) {
        scan.d_atoms.push((key, atom));
    }
    Ok(())
}

/// بازنویسی درخت: `y(x) → y` و گره‌های مشتق → اتم‌های D1y/… .
fn rewrite(e: &Sym, scan: &Scan) -> Result<Sym, CasError> {
    // گره مشتق؟
    if let Sym::Fn(name, _args) = e {
        if name.as_ref() == DIFF_FN {
            if let Some((_, atom)) = scan.d_atoms.iter().find(|(k, _)| *k == to_sexp(e)) {
                return Ok(Sym::symbol(atom));
            }
        }
    }
    match e {
        Sym::Fn(name, args) if args.len() == 1 => {
            if scan.f.as_deref() == Some(name.as_ref()) {
                if let Some(v) = scan.var.as_deref() {
                    if matches!(&args[0], Sym::Symbol(s) if s.as_ref() == v) {
                        return Ok(Sym::symbol(name));
                    }
                }
            }
            let a = rewrite(&args[0], scan)?;
            Ok(Sym::fn_of(name, vec![a])?)
        }
        Sym::Add(ts) => {
            let mut acc: Option<Sym> = None;
            for c in ts {
                let r = rewrite(c, scan)?;
                acc = Some(match acc {
                    None => r,
                    Some(a) => Sym::plus(a, r),
                });
            }
            Ok(acc.ok_or(CasError::Serde("empty n-ary node"))?)
        }
        Sym::Mul(fs) => {
            let mut acc: Option<Sym> = None;
            for c in fs {
                let r = rewrite(c, scan)?;
                acc = Some(match acc {
                    None => r,
                    Some(a) => Sym::times(a, r),
                });
            }
            Ok(acc.ok_or(CasError::Serde("empty n-ary node"))?)
        }
        Sym::Pow(b, ex) => Ok(Sym::pow(rewrite(b, scan)?, rewrite(ex, scan)?)?),
        other => Ok(other.clone()),
    }
}

// ---------------------------------------------------------------------------
// ورفای جواب — نمادین یا عددی روی نمونه‌ها
// ---------------------------------------------------------------------------

/// بررسی تساوی دو طرف با ثابت‌های C — نمادین یا عددی.
/// `lhs`/`rhs` ممکن است حاوی `C1…` و `x` باشند.
pub(crate) fn verify_equal(lhs: &Sym, rhs: &Sym, cs: &[String], x: &str) -> Result<bool, CasError> {
    let res = simplify(&Sym::minus(lhs.clone(), rhs.clone()))?;
    if is_sym_zero(&res) || to_sexp(&res) == "(int 0)" {
        return Ok(true);
    }
    // عددی: جایگذاری نمونه‌ای ثابت‌ها + نمونه‌های x — گارد دامنه/تکینگی.
    // بازهٔ پهن — جواب‌های log/exp به ثابت‌های بزرگ نیاز دارند
    // (مثلاً −log(C1 − e^x) با C1 ≈ −100 روی نمونه‌ها زنده می‌ماند).
    const C_CANDS: [i64; 14] = [1, 2, 3, 5, -1, 7, -3, 11, -7, -11, 13, -30, 50, -100];
    const X_SAMPLES: [f64; 7] = [-1.7, -0.6, 0.35, 0.9, 1.6, 2.4, 3.9];
    let n = cs.len().min(3);
    let mut idx = vec![0usize; n];
    for _ in 0..C_CANDS.len().pow(n as u32).min(24) {
        let mut lhs_c = lhs.clone();
        let mut rhs_c = rhs.clone();
        for (i, c) in cs.iter().take(n).enumerate() {
            let v = Sym::int(C_CANDS[idx[i]]);
            lhs_c = lhs_c.substitute(c, &v);
            rhs_c = rhs_c.substitute(c, &v);
        }
        let mut ok = true;
        for xs in X_SAMPLES {
            let xv = Sym::Real(xs);
            let l = lhs_c.substitute(x, &xv).to_f64();
            let r = rhs_c.substitute(x, &xv).to_f64();
            let (Ok(l), Ok(r)) = (l, r) else {
                ok = false;
                break;
            };
            if !l.is_finite() || !r.is_finite() {
                ok = false;
                break;
            }
            let scale = 1.0 + l.abs() + r.abs();
            if (l - r).abs() > 1e-6 * scale + 1e-9 {
                ok = false;
                break;
            }
        }
        if ok {
            return Ok(true);
        }
        // ترکیب بعدی ثابت‌ها
        for i in (0..n).rev() {
            idx[i] += 1;
            if idx[i] < C_CANDS.len() {
                break;
            }
            idx[i] = 0;
        }
    }
    Ok(false)
}

/// باقی‌ماندهٔ ODE برای جواب `sol`: هر اتم `Dk·f` → مشتق k-ام جواب، `f` → جواب.
pub(crate) fn residual_for(sol: &Sym, p: &ParsedOde) -> Result<Sym, CasError> {
    let mut out = p.norm.clone();
    // ترتیب: اول اتم‌های مرتبهٔ بالا (تا D1y پس از y بازنویسی نشود) —
    // اتم «D{k}{f}» با پسوند نام تابع شناخته می‌شود.
    let mut order: Vec<(String, usize)> = Vec::new();
    for s in out.free_symbols() {
        if let Some(num) = s.strip_prefix(D_ATOM_PREFIX) {
            if let Some(k) = num.strip_suffix(&p.f) {
                if let Ok(k) = k.parse::<usize>() {
                    if k >= 1 {
                        order.push((s.clone(), k));
                    }
                }
            }
        }
    }
    order.sort_by_key(|(_, k)| *k);
    for (atom, k) in order.iter().rev() {
        let dv = diff_n(sol, &p.var, *k as i64)?;
        out = out.substitute(atom, &dv);
    }
    Ok(out.substitute(&p.f, sol))
}

/// `dsolve` سطح موتور — فهرست شاخه‌های صریح جواب عمومی (سند ۰۵ §۹).
pub fn dsolve(eq: &Sym) -> Result<Vec<Sym>, CasError> {
    let p = parse_ode(eq)?;
    if p.order == 1 {
        dsolve_first_order(&p)
    } else {
        dsolve_const_coeff(&p)
    }
}

// ---------------------------------------------------------------------------
// مرتبهٔ اول — ترتیب آشکارساز: مستقیم → خطی → جداپذیر → کامل → μ(x) → μ(y)
// (کلاس‌ها هم‌پوشانند؛ اولین مسیری که شاخهٔ ورفای‌شده بدهد برنده است.)
// ---------------------------------------------------------------------------

/// نام ثابت‌های انتگرال‌گیری که dsolve خروجی‌اش رزرو می‌کند.
const RESERVED: [&str; 13] = [
    "C1", "C2", "C3", "C4", "C5", "C6", "u0", "u1", "u2", "u3", "u4", "u5", "u6",
];

/// زنجیرهٔ جمع — خالی → صفر، تک‌عضوی → خودش.
fn plus_chain(items: Vec<Sym>) -> Result<Sym, CasError> {
    let mut it = items.into_iter();
    let Some(first) = it.next() else {
        return Ok(Sym::int(0));
    };
    Ok(it.fold(first, Sym::plus))
}

/// گویای دقیق از عدد صحیح — Ratio<BigInt> از i32 پیاده نمی‌شود.
fn rq(n: i64) -> BigRational {
    BigRational::new(BigInt::from(n), BigInt::from(1))
}

fn dsolve_first_order(p: &ParsedOde) -> Result<Vec<Sym>, CasError> {
    let norm = simplify(&p.norm)?;
    let d1 = format!("{D_ATOM_PREFIX}1{}", p.f);
    // فرم خطی در مشتق: N·D1y + M = 0 — شرط ورود به همهٔ مسیرهای مرتبهٔ اول.
    let Some((n_coef, m_part)) = crate::int::as_linear(&norm, &d1) else {
        return Err(CasError::Unsupported(
            "dsolve: the first-order equation must be linear in y' — implicit forms beyond M + N·y' = 0 are outside the documented subset.",
        ));
    };
    let rhs = simplify(&Sym::over(Sym::negated(m_part.clone()), n_coef.clone())?)?;

    // مسیرها — هر کدام Vec خالی = «شاخهٔ ورفای‌شده نداد».
    let mut out = direct_int(&rhs, p)?;
    if out.is_empty() {
        out = linear_first(&rhs, p)?;
    }
    if out.is_empty() {
        out = separable_first(&rhs, p)?;
    }
    if out.is_empty() {
        // کامل بدون μ و سپس با μ سادهٔ x/ y.
        out = exact_from(&m_part, &n_coef, p)?;
        if out.is_empty() {
            out = exact_with_mu(&m_part, &n_coef, p, true)?;
        }
        if out.is_empty() {
            out = exact_with_mu(&m_part, &n_coef, p, false)?;
        }
    }
    if out.is_empty() {
        return Err(CasError::Unsupported(
            "dsolve: this first-order ODE is outside the documented subset (separable / linear / exact with simple integrating factor) — Bernoulli, Riccati and general variable-coefficient forms are not supported; use the numeric ode45 route.",
        ));
    }
    Ok(out)
}

/// شاخه‌های ورفای‌شدهٔ یک فرم بستهٔ کاندید — گیت نهایی همهٔ مسیرها.
fn verified_branches(branches: Vec<Sym>, p: &ParsedOde) -> Result<Vec<Sym>, CasError> {
    let cs: Vec<String> = branch_constants(&branches);
    let mut ok = Vec::new();
    for sol in branches {
        // شاخهٔ جواب عمومی باید دست‌کم یک ثابت C داشته باشد؛ راه‌های
        // بدون ثابت (فقط خاص) جواب عمومی نیستند و پذیرفته نمی‌شوند.
        if !sol.free_symbols().iter().any(|s| s.starts_with('C')) {
            continue;
        }
        let res = residual_for(&sol, p)?;
        if verify_equal(&res, &Sym::int(0), &cs, &p.var)? {
            // بدون simplify نهایی — توزیعِ صورت‌های مسیر خطی حفظ می‌شود
            // (simplify کل، جمله‌ها را دوباره در یک کسر ادغام می‌کند).
            ok.push(sol);
        }
    }
    Ok(ok)
}

/// ثابت‌های C حاضر در شاخه‌ها (برای ورفای عددی).
fn branch_constants(branches: &[Sym]) -> Vec<String> {
    let mut cs = Vec::new();
    for b in branches {
        for s in b.free_symbols() {
            if RESERVED.contains(&s.as_str()) && !cs.contains(&s) {
                cs.push(s);
            }
        }
    }
    cs.sort();
    cs
}

/// مسیر ۱ — rhs بی‌وابستگی به y: انتگرال‌گیری مستقیم.
fn direct_int(rhs: &Sym, p: &ParsedOde) -> Result<Vec<Sym>, CasError> {
    if contains(rhs, &p.f) {
        return Ok(vec![]);
    }
    let fint = if is_sym_zero(rhs) {
        Sym::int(0)
    } else {
        match integrate(rhs, &p.var)? {
            Some(v) => v,
            None => return Ok(vec![]),
        }
    };
    let sol = simplify(&Sym::plus(fint, Sym::symbol("C1")))?;
    verified_branches(vec![sol], p)
}

/// μ = exp(∫p dv) با حذف abs از log(|u|) — چک دقیق‌بودنِ پس از آن، گیت
/// صداقت است: اگر μ کاندید فاکتور انتگرال‌ساز نبود، مسیر رد می‌شود.
fn mu_from(p_exp: &Sym, v: &str) -> Result<Option<Sym>, CasError> {
    let Some(pp) = integrate(p_exp, v)? else {
        return Ok(None);
    };
    // log(|u|) → log(u): μ = exp(Σ c·log u) = ∏ u^c — روی هر مؤلفهٔ
    // همبندِ دامنه (u > 0 یا u < 0) فاکتور انتگرال‌سازِ معتبر است؛ چک
    // نمادینیِ M_y = N_x در ادامه این را اثبات یا رد می‌کند.
    let pp = strip_abs_in_log(&pp);
    Ok(Some(simplify(&Sym::fn_of("exp", vec![pp])?)?))
}

/// بازنویسی بازگشتی log(abs(u)) → log(u) فقط در نمای log.
fn strip_abs_in_log(e: &Sym) -> Sym {
    match e {
        Sym::Fn(name, args) if name.as_ref() == "log" && args.len() == 1 => {
            if let Sym::Fn(iname, iargs) = &args[0] {
                if iname.as_ref() == "abs" && iargs.len() == 1 {
                    if let Ok(built) = Sym::fn_of("log", vec![iargs[0].clone()]) {
                        return built;
                    }
                }
            }
            e.clone()
        }
        Sym::Add(ts) => {
            let mut acc: Option<Sym> = None;
            for t in ts {
                let r = strip_abs_in_log(t);
                acc = Some(match acc {
                    None => r,
                    Some(a) => Sym::plus(a, r),
                });
            }
            acc.unwrap_or_else(|| Sym::int(0))
        }
        Sym::Mul(fs) => {
            let mut acc: Option<Sym> = None;
            for t in fs {
                let r = strip_abs_in_log(t);
                acc = Some(match acc {
                    None => r,
                    Some(a) => Sym::times(a, r),
                });
            }
            acc.unwrap_or_else(|| Sym::int(1))
        }
        _ => e.clone(),
    }
}

/// مسیر ۲ — خطی: rhs = p₁·y + q₁ → y′ + p·y = q با p = −p₁.
fn linear_first(rhs: &Sym, p: &ParsedOde) -> Result<Vec<Sym>, CasError> {
    let Some((p1, q1)) = crate::int::as_linear(rhs, &p.f) else {
        return Ok(vec![]);
    };
    let pv = simplify(&Sym::negated(p1))?;
    let Some(mu_raw) = mu_from(&pv, &p.var)? else {
        return Ok(vec![]);
    };
    // فروپاشی کل-دامنهٔ بی‌خطر نیست؛ کاندید ∏u^c با گیت ورفای نهایی
    // پذیرفته/رد می‌شود (روی هر مؤلفهٔ همبند دامنه معتبر است).
    let mu = collapse_mu(&mu_raw).unwrap_or(mu_raw);
    let mq = if is_sym_zero(&q1) {
        Sym::int(0)
    } else {
        match integrate(&simplify(&Sym::times(mu.clone(), q1))?, &p.var)? {
            Some(v) => v,
            None => return Ok(vec![]),
        }
    };
    // توزیع μ⁻¹ روی جمله‌ها + ثابت — هر جمله جدا ساده می‌شود تا خروجی
    // متنی شکل کتاب‌درسی بگیرد (e^x/2 + C1·e^(−x) نه یک کسر بزرگ).
    let mu_inv = Sym::pow(mu, Sym::int(-1))?;
    let mterms: Vec<Sym> = match simplify(&mq)? {
        Sym::Add(ts) => ts,
        o => vec![o],
    };
    let mut sol = Sym::int(0);
    for t in mterms.into_iter().chain(std::iter::once(Sym::symbol("C1"))) {
        let st = simplify(&Sym::times(t, mu_inv.clone()))?;
        if !is_sym_zero(&st) {
            sol = Sym::plus(sol, st);
        }
    }
    verified_branches(vec![sol], p)
}

/// وارون 1/g در فرم جدول‌پسند: exp(u) → exp(−u)، u^k → u^(−k)، بقیه Quotient.
fn reciprocal_form(g: &Sym) -> Sym {
    match g {
        Sym::Fn(name, args) if name.as_ref() == "exp" && args.len() == 1 => {
            Sym::fn_of("exp", vec![Sym::negated(args[0].clone())]).unwrap_or_else(|_| Sym::int(1))
        }
        Sym::Pow(b, e) => {
            Sym::pow((**b).clone(), Sym::negated((**e).clone())).unwrap_or_else(|_| Sym::int(1))
        }
        other => Sym::over(Sym::int(1), other.clone()).unwrap_or_else(|_| Sym::int(1)),
    }
}

/// مسیر ۳ — جداپذیر: rhs = f(x)·g(y) با آزمون pivot و چک نمادین.
fn separable_first(rhs: &Sym, p: &ParsedOde) -> Result<Vec<Sym>, CasError> {
    let Some((f, g)) = split_separable(rhs, &p.var, &p.f) else {
        return Ok(vec![]);
    };
    let hres = integrate(&reciprocal_form(&g), &p.f);
    let Some(h) = hres? else {
        return Ok(vec![]);
    };
    let fres = integrate(&f, &p.var);
    let Some(fx) = fres? else {
        return Ok(vec![]);
    };
    // H(y) = F(x) + C1 — وارون‌سازی: خطی-در-y، شکل‌های خاص، جدول solve.
    let eq = Sym::minus(h, Sym::plus(fx, Sym::symbol("C1")));
    let branches = invert_eq(&eq, &p.f)?;
    if branches.is_empty() {
        return Ok(vec![]);
    }
    verified_branches(branches, p)
}

/// آزمون تجزیهٔ rhs = f(x)·g(y) با pivot های عددی ساده + چک نمادین نهایی.
fn split_separable(rhs: &Sym, x: &str, y: &str) -> Option<(Sym, Sym)> {
    if is_sym_zero(&simplify(rhs).ok()?) {
        return None;
    }
    const XS: [i64; 4] = [0, 1, 2, -1];
    const YS: [i64; 4] = [1, 2, -1, 3];
    for x0 in XS {
        for y0 in YS {
            let g = simplify(&rhs.substitute(x, &Sym::int(x0))).ok()?;
            if is_sym_zero(&g) || contains(&g, x) {
                continue;
            }
            let gy0 = simplify(&g.substitute(y, &Sym::int(y0))).ok()?;
            if is_sym_zero(&gy0) {
                continue;
            }
            let fy0 = simplify(&rhs.substitute(y, &Sym::int(y0))).ok()?;
            // f = rhs(y→y0) · g(y→y0)⁻¹ — ضرب در وارون تا exp-mul کار کند.
            let ginv = Sym::pow(gy0, Sym::int(-1)).ok()?;
            let f = simplify(&Sym::times(fy0, ginv)).ok()?;
            if is_sym_zero(&f) || contains(&f, y) {
                continue;
            }
            // گیت نهایی: rhs ≡ f·g (نمادین).
            let prod = simplify(&Sym::times(f.clone(), g.clone())).ok()?;
            let diff = simplify(&Sym::minus(rhs.clone(), prod)).ok()?;
            if is_sym_zero(&diff) {
                return Some((f, g));
            }
        }
    }
    None
}

/// مسیر ۴ — کامل: M + N·y′ = 0 با M_y = N_x → F(x,y) = C1.
fn exact_from(m: &Sym, n: &Sym, p: &ParsedOde) -> Result<Vec<Sym>, CasError> {
    let dmy = simplify(&diff_n(m, &p.f, 1)?)?;
    let dnx = simplify(&diff_n(n, &p.var, 1)?)?;
    let gap = simplify(&Sym::minus(dmy, dnx))?;
    if !is_sym_zero(&gap) {
        return Ok(vec![]);
    }
    exact_potential(m, n, p)
}

/// ساخت پتانسیل F از M، N (پیش‌شرط: M_y = N_x — فراخوان چک می‌کند) و وارون‌سازی.
fn exact_potential(m: &Sym, n: &Sym, p: &ParsedOde) -> Result<Vec<Sym>, CasError> {
    let f0r = integrate(m, &p.var);
    let Some(f0) = f0r? else {
        return Ok(vec![]);
    };
    let dphi = simplify(&Sym::minus(n.clone(), diff_n(&f0, &p.f, 1)?))?;
    let phi = if is_sym_zero(&dphi) {
        Sym::int(0)
    } else {
        match integrate(&dphi, &p.f)? {
            Some(v) => v,
            None => {
                return Ok(vec![]);
            }
        }
    };
    let ff = simplify(&Sym::plus(f0, phi))?;
    invert_potential(ff, p)
}

/// وارون‌سازی F(x,y) = C1 — اول خطی-در-y دستی، بعد جدول وارون‌های solve.
fn invert_potential(ff: Sym, p: &ParsedOde) -> Result<Vec<Sym>, CasError> {
    let eq = Sym::minus(ff, Sym::symbol("C1"));
    let branches = invert_eq(&eq, &p.f)?;
    if branches.is_empty() {
        return Ok(vec![]);
    }
    verified_branches(branches, p)
}

/// وارون‌سازی eq(y) = 0 برای شاخه‌های جواب: خطی-در-y (با گارد صفر دقیق)،
/// سپس شکل‌های خاص exp/log/atan با آرگومان خطی (حتی با ضریب سمبلیک)،
/// سپس جدول وارون‌های solve.
fn invert_eq(eq: &Sym, y: &str) -> Result<Vec<Sym>, CasError> {
    // اجتماع کاندیدها از هر سه منبع — ورفای نهاییِ verified_branches
    // شاخه‌های غلط را حذف می‌کند (مثلاً وارون‌سازی خطیِ کاذب وقتی y فقط
    // داخل log/exp است).
    let mut branches = match crate::int::as_linear(eq, y) {
        Some((a, b)) => {
            let a_s = simplify(&a)?;
            if is_sym_zero(&a_s) {
                vec![]
            } else {
                vec![simplify(&Sym::over(Sym::negated(b), a_s)?)?]
            }
        }
        None => vec![],
    };
    branches.extend(invert_special(eq, y)?);
    if branches.is_empty() {
        branches = solve_expr(eq, y).unwrap_or_default();
    }
    Ok(branches)
}

/// تفکیک ترم c·F(u): ضریب سمبلیکِ بی-y و گرهٔ F با آرگومان حاوی y.
/// بازگشت (ضریب، نام تابع، آرگومان) — غیر از این شکل None.
fn split_fn_term(t: &Sym, y: &str) -> Option<(Sym, String, Sym)> {
    let factors: Vec<Sym> = match t {
        Sym::Mul(fs) => fs.clone(),
        other => vec![other.clone()],
    };
    let mut coef: Vec<Sym> = Vec::new();
    let mut fnode: Option<(String, Sym)> = None;
    for f in &factors {
        match f {
            Sym::Fn(name, args) if args.len() == 1 && contains(&args[0], y) => {
                if fnode.is_some() {
                    return None;
                }
                fnode = Some((name.to_string(), args[0].clone()));
            }
            Sym::Fn(name, args) if args.len() == 1 => {
                coef.push(Sym::fn_of(name, args[0..1].to_vec()).ok()?);
            }
            other => {
                if contains(other, y) {
                    return None;
                }
                coef.push(other.clone());
            }
        }
    }
    let (name, u) = fnode?;
    // ترم تابعِ خالص (بدون ضریب) → ضریب ۱ نه صفر.
    let c = if coef.is_empty() {
        Sym::int(1)
    } else {
        plus_chain(coef).ok()?
    };
    Some((c, name, u))
}

/// وارون‌سازی خاص: eq = c·F(a·y) + R با R بی-y → وارون جدولی F.
/// exp → log، log → exp، log(|y|) → دو شاخه ±exp، atan → tan.
/// درستی هر شاخه با ورفای نهایی جداگانه اثبات یا رد می‌شود.
fn invert_special(eq: &Sym, y: &str) -> Result<Vec<Sym>, CasError> {
    let e = simplify(eq)?;
    let terms = match &e {
        Sym::Add(ts) => ts.clone(),
        other => vec![other.clone()],
    };
    let mut yterm: Option<(Sym, String, Sym)> = None;
    let mut rest: Vec<Sym> = Vec::new();
    for t0 in &terms {
        // ترم‌های صفرِ زائد خروجی integrate (مثل 0·log(...)) را بی‌اثر کن.
        let t = simplify(t0)?;
        if is_sym_zero(&t) {
            continue;
        }
        let t = &t;
        if !contains(t, y) {
            rest.push(t.clone());
            continue;
        }
        if yterm.is_some() {
            return Ok(vec![]);
        }
        match split_fn_term(t, y) {
            Some(trip) => yterm = Some(trip),
            None => {
                return Ok(vec![]);
            }
        }
    }
    let Some((c, name, u)) = yterm else {
        return Ok(vec![]);
    };
    if is_sym_zero(&simplify(&c)?) {
        return Ok(vec![]);
    }
    let r = plus_chain(rest)?;
    // −R/c
    let neg_rc = match Sym::over(Sym::negated(r), c.clone()) {
        Ok(v) => simplify(&v)?,
        Err(_) => return Ok(vec![]),
    };
    let mut out: Vec<Sym> = Vec::new();
    match name.as_str() {
        "exp" => {
            let Some((a, inter)) = crate::int::as_linear(&u, y) else {
                return Ok(vec![]);
            };
            if !is_sym_zero(&simplify(&inter)?) {
                return Ok(vec![]);
            }
            // a·y = log(−R/c) → y = log(−R/c)/a
            if let Ok(lg) = Sym::fn_of("log", vec![neg_rc]) {
                if let Ok(br) = Sym::over(lg, a) {
                    out.push(simplify(&br)?);
                }
            }
        }
        "log" => {
            // log(|y|) → |y| = exp(−R/c) → دو شاخه ±
            if let Sym::Fn(iname, iargs) = &u {
                if iname.as_ref() == "abs" && iargs.len() == 1 {
                    if let Sym::Symbol(s) = &iargs[0] {
                        if s.as_ref() == y {
                            if let Ok(ex) = Sym::fn_of("exp", vec![neg_rc]) {
                                out.push(simplify(&ex)?);
                                out.push(simplify(&Sym::negated(ex))?);
                            }
                            return Ok(out);
                        }
                    }
                }
            }
            let Some((a, inter)) = crate::int::as_linear(&u, y) else {
                return Ok(vec![]);
            };
            if !is_sym_zero(&simplify(&inter)?) {
                return Ok(vec![]);
            }
            // a·y = exp(−R/c) → y = exp(−R/c)/a
            if let Ok(ex) = Sym::fn_of("exp", vec![neg_rc]) {
                if let Ok(br) = Sym::over(ex, a) {
                    out.push(simplify(&br)?);
                }
            }
        }
        "atan" => {
            let Some((a, inter)) = crate::int::as_linear(&u, y) else {
                return Ok(vec![]);
            };
            if !is_sym_zero(&simplify(&inter)?) {
                return Ok(vec![]);
            }
            // a·y = tan(−R/c) → y = tan(−R/c)/a
            if let Ok(tg) = Sym::fn_of("tan", vec![neg_rc]) {
                if let Ok(br) = Sym::over(tg, a) {
                    out.push(simplify(&br)?);
                }
            }
        }
        _ => {}
    }
    Ok(out)
}

/// ضرب M در μ — برای μ = u⁻¹ تقسیم دقیق (gcd ضعیفِ چندمتغیره روی
/// ژنراتورهای اُپک، y·y⁻¹ را لغو نمی‌کند)؛ وگرنه ضرب + ساده‌سازی.
fn mul_by_mu(m: &Sym, mu: &Sym) -> Result<Sym, CasError> {
    if let Sym::Pow(b, e) = mu {
        // توان −1 گویا یا صحیح (کاندید فروپاشی، توان را Rational می‌سازد)
        let minus_one = matches!(&**e, Sym::Integer(z) if *z == BigInt::from(-1))
            || matches!(&**e, Sym::Rational(r) if *r == -rq(1));
        if minus_one {
            let mp = RatFun::from_sym(m).ok();
            let bp = RatFun::from_sym(b).ok();
            if let (Some(mp), Some(bp)) = (mp, bp) {
                if !bp.num.is_zero() {
                    let mut names: Vec<String> = Vec::new();
                    for k in bp.num.generators().into_iter().chain(mp.num.generators()) {
                        if let Some(nm) = k.strip_prefix("(sym ").and_then(|s| s.strip_suffix(')'))
                        {
                            if !names.iter().any(|x| x == nm) {
                                names.push(nm.to_string());
                            }
                        }
                    }
                    for g in names {
                        if let Some(q) = exact_div_in(&mp.num, &bp.num, &g) {
                            return Ok(q);
                        }
                    }
                }
            }
        }
    }
    simplify(&Sym::times(m.clone(), mu.clone()))
}

/// مسیر ۵/۶ — کامل با فاکتور انتگرال‌ساز سادهٔ μ(x) یا μ(y) (سند ۰۵ §۹).
fn exact_with_mu(m: &Sym, n: &Sym, p: &ParsedOde, mu_x: bool) -> Result<Vec<Sym>, CasError> {
    let dmy = simplify(&diff_n(m, &p.f, 1)?)?;
    let dnx = simplify(&diff_n(n, &p.var, 1)?)?;
    // μ(x): (M_y − N_x)/N باید فقط تابع x باشد؛ μ(y): (N_x − M_y)/M فقط y.
    // نسبت: اول تقسیم دقیق چندجمله‌ای (gcd چندمتغیره روی ژنراتورهای اُپک
    // لغو می‌کند)، بعد مسیر معمول ساده‌سازی.
    let (num, den, v, other) = if mu_x {
        (Sym::minus(dmy, dnx), n.clone(), p.var.clone(), p.f.clone())
    } else {
        (Sym::minus(dnx, dmy), m.clone(), p.f.clone(), p.var.clone())
    };
    let ratio = match ratio_exact(&num, &den) {
        Some(r) => simplify(&r)?,
        None => simplify(&Sym::over(num, den)?)?,
    };
    if contains(&ratio, &other) {
        return Ok(vec![]);
    }
    let Some(mu_raw) = mu_from(&ratio, &v)? else {
        return Ok(vec![]);
    };
    // کاندیدها: فروپاشی ∏u^c اول، بعد فرم exp — گیت دقیق‌بودن تصمیم می‌گیرد.
    let mut cands = Vec::new();
    if let Some(c) = collapse_mu(&mu_raw) {
        cands.push(c);
    }
    cands.push(mu_raw);
    for mu in cands {
        let m2 = mul_by_mu(m, &mu)?;
        let n2 = mul_by_mu(n, &mu)?;
        // گیت صداقت: μ کاندید باید واقعاً فاکتور انتگرال‌ساز باشد.
        let dmy2 = simplify(&diff_n(&m2, &p.f, 1)?)?;
        let dnx2 = simplify(&diff_n(&n2, &p.var, 1)?)?;
        let gate = simplify(&Sym::minus(dmy2, dnx2))?;
        if !is_sym_zero(&gate) {
            continue;
        }
        let out = exact_potential(&m2, &n2, p)?;
        if !out.is_empty() {
            return Ok(out);
        }
    }
    Ok(vec![])
}

/// نسبت دقیق a/b وقتی صورت یا مخرج به‌طور دقیق تقسیم‌پذیر است —
/// مسیر پشتیبان وقتی gcd چندمتغیرهٔ RatFun روی ژنراتورهای اُپک لغو می‌کند.
fn ratio_exact(a: &Sym, b: &Sym) -> Option<Sym> {
    let ra = RatFun::from_sym(a).ok()?;
    let rb = RatFun::from_sym(b).ok()?;
    if rb.num.is_zero() || ra.num.is_zero() {
        return None;
    }
    let n = ra.num.mul(&rb.den).ok()?;
    let d = ra.den.mul(&rb.num).ok()?;
    // تقسیم دقیق در هر ژنراتور کاندید (pdiv_exact خارج‌قسمت کسری
    // نمی‌دهد؛ ضرایب Poly گویا هستند — اینجا RatFun-محور انجام می‌شود).
    // coeff_of فقط نام نماد می‌پذیرد — کلیدهای «(sym v)» → نام v.
    let mut names: Vec<String> = Vec::new();
    for k in d.generators().into_iter().chain(n.generators()) {
        if let Some(nm) = k.strip_prefix("(sym ").and_then(|s| s.strip_suffix(')')) {
            if !names.iter().any(|x| x == nm) {
                names.push(nm.to_string());
            }
        }
    }
    for g in names {
        if let Some(q) = exact_div_in(&n, &d, &g) {
            return Some(q);
        }
    }
    None
}

/// تقسیم دقیق n بر d در ژنراتور g با ضرایب RatFun — خارج‌قسمت کسری مجاز؛
/// نامدقیق (باقی‌ماندهٔ ناصفر) → None.
fn exact_div_in(n: &Poly, d: &Poly, g: &str) -> Option<Sym> {
    let vk = crate::int::vkey(g);
    let dd = d.degree_in(&vk);
    if dd == 0 {
        return None;
    }
    let rf = |p: &Poly, k: i64| -> RatFun {
        RatFun {
            num: crate::int::coeff_of(p, g, k),
            den: Poly::one(),
        }
    };
    let lc = rf(d, dd);
    let dmax = n.degree_in(&vk);
    let mut rem: std::collections::BTreeMap<i64, RatFun> = (0..=dmax)
        .map(|k| (k, rf(n, k)))
        .filter(|(_, v)| !v.is_zero())
        .collect();
    let mut q: Vec<(i64, RatFun)> = Vec::new();
    let mut steps = 0usize;
    while let Some(&dk) = rem.keys().next_back() {
        if dk < dd {
            break;
        }
        steps += 1;
        if steps > 60 {
            return None;
        }
        let lead = rem.get(&dk)?.clone();
        let qk = lead.div(&lc).ok()?;
        let shift = dk - dd;
        for k in 0..=dd {
            let c = rf(d, k);
            if c.is_zero() {
                continue;
            }
            let prod = c.mul(&qk).ok()?;
            let e = rem.entry(k + shift).or_insert_with(RatFun::zero);
            *e = e.sub(&prod).ok()?;
        }
        rem.remove(&dk);
        rem.retain(|_, v| !v.is_zero());
        q.push((shift, qk));
    }
    if !rem.is_empty() {
        return None;
    }
    let mut acc = Sym::int(0);
    for (k, qk) in q {
        if qk.is_zero() {
            continue;
        }
        let cs = qk.to_sym().ok()?;
        let term = if k == 0 {
            cs
        } else {
            Sym::times(cs, Sym::pow(Sym::symbol(g), Sym::int(k)).ok()?)
        };
        acc = Sym::plus(acc, term);
    }
    simplify(&acc).ok()
}

/// کاندید فروپاشی μ: exp(Σ c·log u) → ∏ u^c — فقط وقتی همهٔ جمله‌ها
/// همین شکل را دارند؛ درستی روی هر مؤلفهٔ همبند دامنه، با گیت چک می‌شود.
fn collapse_mu(mu: &Sym) -> Option<Sym> {
    let Sym::Fn(name, args) = mu else {
        return None;
    };
    if name.as_ref() != "exp" || args.len() != 1 {
        return None;
    }
    let terms: Vec<Sym> = match &args[0] {
        Sym::Add(ts) => ts.clone(),
        other => vec![other.clone()],
    };
    let mut factors: Vec<Sym> = Vec::new();
    for t in &terms {
        let (c, u) = split_coeff_log(t)?;
        let e = Sym::Rational(c);
        factors.push(Sym::pow(u, e).ok()?);
    }
    factors.into_iter().reduce(Sym::times)
}

/// تفکیک c·log(u) یا log(u) → (c، u) — غیر از این شکل، None.
fn split_coeff_log(t: &Sym) -> Option<(BigRational, Sym)> {
    match t {
        Sym::Fn(name, args) if name.as_ref() == "log" && args.len() == 1 => {
            Some((rq(1), args[0].clone()))
        }
        Sym::Mul(fs) => {
            let mut c = rq(1);
            let mut lu: Option<Sym> = None;
            for f in fs {
                match f {
                    Sym::Fn(name, args) if name.as_ref() == "log" && args.len() == 1 => {
                        if lu.is_some() {
                            return None;
                        }
                        lu = Some(args[0].clone());
                    }
                    other => {
                        c *= other.as_exact()?;
                    }
                }
            }
            Some((c, lu?))
        }
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// خطی با ضرایب ثابت — هر مرتبه (سند ۰۵ §۹): معادلهٔ مشخصه + ضرایب نامعین
// ---------------------------------------------------------------------------

/// ریشهٔ معادلهٔ مشخصه — گویا یا مختلط (α ± iβ) یا رادیکال حقیقی نمادین.
#[derive(Clone, Debug, PartialEq)]
enum RootVal {
    Rat(BigRational),
    /// رادیکال حقیقی با نمایش سمبلیک (disc > 0 غیر مربع کامل).
    Rad(Sym),
    /// زوج مختلط α ± iβ با β > 0 (نمایش cos/sin).
    Cx(BigRational, Sym),
}

/// جملهٔ RHS برای ضرایب نامعین: c·x^d·e^{ax}·{1|cos(bx)|sin(bx)}.
#[derive(Clone, Debug)]
struct QTerm {
    c: BigRational,
    d: i64,
    ea: Option<BigRational>,
    tr: Option<(bool, BigRational)>,
}

fn dsolve_const_coeff(p: &ParsedOde) -> Result<Vec<Sym>, CasError> {
    let norm = simplify(&p.norm)?;
    let n = p.order;
    // اتم‌ها: y و Dky — ضرایب باید ثابت گویا باشند.
    let mut atoms: Vec<String> = vec![p.f.clone()];
    for k in 1..=n {
        atoms.push(format!("{D_ATOM_PREFIX}{k}{}", p.f));
    }
    let mut coefs: Vec<Option<BigRational>> = vec![None; n + 1]; // [y, D1y, …, Dny]
    let mut q_terms: Vec<Sym> = Vec::new();
    let terms = match &norm {
        Sym::Add(ts) => ts.clone(),
        other => vec![other.clone()],
    };
    for t in &terms {
        let (mono, coef) = split_atom_term(t, &atoms, p)?;
        match mono {
            Some(k) => {
                if coefs[k].is_some() {
                    return Err(CasError::Unsupported(
                        "dsolve: repeated monomial in the constant-coefficient decomposition — internal canonicalization error.",
                    ));
                }
                coefs[k] = Some(coef);
            }
            None => q_terms.push(t.clone()),
        }
    }
    if coefs[n].is_none() {
        return Err(CasError::Unsupported(
            "dsolve: the coefficient of the highest derivative vanished — inconsistent order.",
        ));
    }
    let q = simplify(&Sym::negated(plus_chain(q_terms)?))?;
    // معادلهٔ مشخصه: Σ coef_k·r^k (coef_k برای k ناینجا صفر).
    let mut cp = Sym::int(0);
    for (k, c) in coefs.iter().enumerate() {
        if let Some(c) = c {
            let term = Sym::times(
                Sym::Rational(c.clone()),
                Sym::pow(Sym::symbol("r"), Sym::int(k as i64))?,
            );
            cp = Sym::plus(cp, term);
        }
    }
    let roots = char_roots(&cp, n)?;
    // جواب همگن با ثابت‌های C1… در ترتیب ساخت.
    let mut c_count = 0usize;
    let mut hbases: Vec<Sym> = Vec::new();
    for (rv, mult) in &roots {
        match rv {
            RootVal::Rat(r) => {
                for j in 0..*mult {
                    hbases.push(exp_basis(&Sym::Rational(r.clone()), j, &p.var)?);
                    c_count += 1;
                }
            }
            RootVal::Rad(s) => {
                for j in 0..*mult {
                    hbases.push(exp_basis(s, j, &p.var)?);
                    c_count += 1;
                }
            }
            RootVal::Cx(a, b) => {
                for j in 0..*mult {
                    // β=1 → آرگومان x خالص (نمایش بدون 1·x)
                    let bx = if matches!(b, Sym::Rational(r) if *r == rq(1)) {
                        Sym::symbol(&p.var)
                    } else {
                        Sym::times(b.clone(), Sym::symbol(&p.var))
                    };
                    hbases.push(trig_basis(a, j, "cos", &bx, &p.var)?);
                    hbases.push(trig_basis(a, j, "sin", &bx, &p.var)?);
                    c_count += 2;
                }
            }
        }
    }
    // ثابت‌ها: C1… پس از y_h به ترتیب پایه‌ها.
    let mut y_h = Sym::int(0);
    for (i, b) in hbases.iter().enumerate() {
        y_h = Sym::plus(
            y_h,
            Sym::times(Sym::symbol(&format!("C{}", i + 1)), b.clone()),
        );
    }
    // جواب خصوصی با ضرایب نامعین (اگر q ≠ 0).
    let y_p = if is_sym_zero(&q) {
        None
    } else {
        Some(undetermined(&q, &coefs, &roots, &p.var)?)
    };
    let sol = match y_p {
        Some(yp) => simplify(&Sym::plus(y_h, yp))?,
        None => simplify(&y_h)?,
    };
    // گیت نهایی: residual نمادین یا عددی با جایگذاری ثابت‌ها.
    let cs: Vec<String> = (1..=c_count).map(|i| format!("C{i}")).collect();
    let res = residual_for(&sol, p)?;
    if !verify_equal(&res, &Sym::int(0), &cs, &p.var)? {
        return Err(CasError::Unsupported(
            "dsolve: candidate solution failed verification — honest failure, no unverified output.",
        ));
    }
    Ok(vec![sol])
}

/// تفکیک یک ترم Add: مونومیل اتم (Some(k) برای y یا Dky) + ضریب گویا،
/// یا ترم بی‌اتم (None) برای q. مونومیل مخلوط/درجه بالا → خطای صادقانه.
fn split_atom_term(
    t: &Sym,
    atoms: &[String],
    p: &ParsedOde,
) -> Result<(Option<usize>, BigRational), CasError> {
    let factors: Vec<Sym> = match t {
        Sym::Mul(fs) => fs.clone(),
        other => vec![other.clone()],
    };
    let mut mono: Option<usize> = None;
    let mut coef_parts: Vec<Sym> = Vec::new();
    for f in &factors {
        match f {
            Sym::Symbol(s) => {
                if let Some(k) = atoms.iter().position(|a| a == s.as_ref()) {
                    if mono.is_some() {
                        return Err(CasError::Unsupported(
                            "dsolve: product of two derivatives of the unknown — the equation is not linear; outside the documented subset.",
                        ));
                    }
                    mono = Some(k);
                    continue;
                }
                coef_parts.push(f.clone());
            }
            Sym::Pow(b, _e) => {
                if let Sym::Symbol(s) = &**b {
                    if atoms.iter().any(|a| a == s.as_ref()) {
                        return Err(CasError::Unsupported(
                            "dsolve: powers of the unknown or its derivatives — nonlinear ODE, outside the documented subset.",
                        ));
                    }
                }
                coef_parts.push(f.clone());
            }
            _ => coef_parts.push(f.clone()),
        }
    }
    let coef_sym = if coef_parts.is_empty() {
        Sym::int(1)
    } else {
        plus_chain(coef_parts)?
    };
    let Some(c) = coef_sym.as_exact() else {
        if mono.is_some() {
            return Err(CasError::Unsupported(
                "dsolve: variable (x-dependent) coefficients — outside the constant-coefficient subset; use the numeric ode45 route.",
            ));
        }
        // ترم q با ضریب غیرگویا — بی‌اشکال (خود ترم ذخیره می‌شود).
        return Ok((None, BigRational::zero()));
    };
    let _ = p;
    Ok((mono, c))
}

/// پایهٔ e^{rx}·x^j (r=0 → چندجمله‌ای خالص).
fn exp_basis(r: &Sym, j: usize, var: &str) -> Result<Sym, CasError> {
    let xj = if j == 0 {
        Sym::int(1)
    } else {
        Sym::pow(Sym::symbol(var), Sym::int(j as i64))?
    };
    let rzero =
        matches!(r, Sym::Integer(z) if z.is_zero()) || matches!(r, Sym::Rational(z) if z.is_zero());
    if rzero {
        Ok(xj)
    } else {
        Ok(Sym::times(
            xj,
            Sym::fn_of("exp", vec![Sym::times(r.clone(), Sym::symbol(var))])?,
        ))
    }
}

/// پایهٔ x^j·e^{αx}·trig(βx).
fn trig_basis(
    a: &BigRational,
    j: usize,
    which: &str,
    bx: &Sym,
    var: &str,
) -> Result<Sym, CasError> {
    let xj = if j == 0 {
        Sym::int(1)
    } else {
        Sym::pow(Sym::symbol(var), Sym::int(j as i64))?
    };
    let tg = Sym::fn_of(which, vec![bx.clone()])?;
    let mut acc = Sym::times(xj, tg);
    if !a.is_zero() {
        acc = Sym::times(
            acc,
            Sym::fn_of(
                "exp",
                vec![Sym::times(Sym::Rational(a.clone()), Sym::symbol(var))],
            )?,
        );
    }
    Ok(acc)
}

/// ریشه‌های معادلهٔ مشخصه — گویا با کاهش، درجه ۲ با فرمول (گارد مختلط).
fn char_roots(cp: &Sym, _order: usize) -> Result<Vec<(RootVal, usize)>, CasError> {
    let rf = RatFun::from_sym(cp)?;
    let mut poly = rf.num;
    let mut out: Vec<(RootVal, usize)> = Vec::new();
    // کاهش ریشهٔ گویا تا درجه ۲.
    loop {
        let deg = max_deg(&poly, "r");
        if deg <= 2 {
            break;
        }
        let Some(r) = crate::int::try_rational_root(&poly, "r") else {
            return Err(CasError::Unsupported(
                "dsolve: characteristic equation of degree ≥ 3 without a rational root — outside the documented subset (course-level characteristic roots); use the numeric ode45 route.",
            ));
        };
        let (q, _rem) = crate::int::div_lin(&poly, &r, "r");
        push_root(&mut out, RootVal::Rat(r));
        poly = q;
    }
    let deg = max_deg(&poly, "r");
    if deg == 0 {
        return Ok(out);
    }
    // ضریب‌های گویای درجهٔ باقیمانده (۱ یا ۲).
    let (a, b) = quad_coeffs(&poly)?;
    if deg == 1 {
        // b·r + a = 0
        if b.is_zero() {
            return Err(CasError::Unsupported(
                "dsolve: degenerate characteristic equation.",
            ));
        }
        push_root(&mut out, RootVal::Rat(-&a / &b));
        return Ok(out);
    }
    if a.is_zero() {
        return Err(CasError::Unsupported(
            "dsolve: degenerate quadratic in the characteristic equation.",
        ));
    }
    // a·r² + b·r + c با c ضریب ثابت — از quad_coeffs: (r² ضریب، r ضریب، ثابت)
    let (qa, qb, qc) = (a, b, quad_const(&poly)?);
    let disc = &(&qb * &qb) - &(rq(4) * &qa * &qc);
    if disc.is_zero() {
        let r = -&qb / &(rq(2) * &qa);
        // ریشهٔ مضاعف ریشهٔ دوم — چندگانگی ۲ (درس y'''+... و (r+1)^2).
        push_root(&mut out, RootVal::Rat(r.clone()));
        push_root(&mut out, RootVal::Rat(r));
        return Ok(out);
    }
    if disc.is_positive() {
        // sqrt دقیق — مربع کامل → گویا؛ وگرنه Real (تصمیم مستند سازندهٔ
        // pow: توان غیرصحیح روی پایهٔ عددی → تقریب صادقانهٔ f64) و ورفای
        // عددیِ نهایی درستی را پوشش می‌دهد.
        match crate::int::sqrt_sym(&disc)? {
            Sym::Rational(s) => {
                let two_a = rq(2) * &qa;
                push_root(&mut out, RootVal::Rat((-&qb + &s) / &two_a));
                push_root(&mut out, RootVal::Rat((-&qb - &s) / &two_a));
            }
            s => {
                let two_a = rq(2) * &qa;
                let half = Sym::Rational(rq(1) / &two_a);
                let mid = Sym::Rational(-&qb / &two_a);
                let r1 = Sym::plus(mid.clone(), Sym::times(half.clone(), s.clone()));
                let r2 = Sym::minus(mid, Sym::times(half, s));
                push_root(&mut out, RootVal::Rad(simplify(&r1)?));
                push_root(&mut out, RootVal::Rad(simplify(&r2)?));
            }
        }
        return Ok(out);
    }
    // disc < 0 → زوج مختلط α ± iβ با β = √(−disc)/(2|a|) — مربع کامل →
    // گویا (تشدید قابل‌تشخیص)؛ وگرنه Real با ورفای عددی.
    let alpha = -&qb / &(rq(2) * &qa);
    let two_a_abs = (rq(2) * &qa).abs();
    let beta = match crate::int::sqrt_sym(&(-&disc))? {
        Sym::Rational(s) => Sym::Rational(s.abs() / two_a_abs),
        Sym::Real(v) => {
            use num_traits::ToPrimitive;
            Sym::Real(v.abs() / two_a_abs.to_f64().unwrap_or(1.0))
        }
        other => other,
    };
    push_root(&mut out, RootVal::Cx(alpha, beta));
    Ok(out)
}

fn push_root(out: &mut Vec<(RootVal, usize)>, r: RootVal) {
    if let Some((_, m)) = out.iter_mut().find(|(rv2, _)| *rv2 == r) {
        *m += 1;
    } else {
        out.push((r, 1));
    }
}

fn max_deg(p: &crate::poly::Poly, v: &str) -> i64 {
    p.degree_in(&crate::int::vkey(v))
}

/// ضریب r² و r از Poly درجه ≤ 2 (a·r² + b·r + c) — این تابع (a_r2, b_r).
fn quad_coeffs(p: &crate::poly::Poly) -> Result<(BigRational, BigRational), CasError> {
    let c2 = crate::int::coeff_of(p, "r", 2);
    let c1 = crate::int::coeff_of(p, "r", 1);
    let ex = |pp: &crate::poly::Poly| -> Result<BigRational, CasError> {
        let s = crate::poly::poly_to_sym(pp)?;
        s.as_exact().ok_or(CasError::Unsupported(
            "dsolve: non-rational characteristic coefficient.",
        ))
    };
    Ok((ex(&c2)?, ex(&c1)?))
}

/// ضریب ثابت c از a·r² + b·r + c.
fn quad_const(p: &crate::poly::Poly) -> Result<BigRational, CasError> {
    let c0 = crate::int::coeff_of(p, "r", 0);
    let s = crate::poly::poly_to_sym(&c0)?;
    s.as_exact().ok_or(CasError::Unsupported(
        "dsolve: non-rational characteristic coefficient.",
    ))
}

// ---------------------------------------------------------------------------
// ضرایب نامعین — جواب خصوصی برای q ∈ {چندجمله‌ای، e^ax، sin/cos، ضرب‌ها}
// ---------------------------------------------------------------------------

/// تجزیهٔ یک ترم به c·x^d·e^{ax}·{1|cos(bx)|sin(bx)} — فرم خارج از
/// زیرمجموعه → خطای صادقانه (سند ۰۵ §۹: ضرایب نامعین فقط این خانواده‌ها).
fn classify_q(t: &Sym, var: &str) -> Result<QTerm, CasError> {
    let factors: Vec<Sym> = match t {
        Sym::Mul(fs) => fs.clone(),
        other => vec![other.clone()],
    };
    let mut c = rq(1);
    let mut d: i64 = 0;
    let mut ea: Option<BigRational> = None;
    let mut tr: Option<(bool, BigRational)> = None;
    for f in &factors {
        match f {
            Sym::Integer(_) | Sym::Rational(_) => {
                let v = f
                    .as_exact()
                    .ok_or(CasError::Unsupported("dsolve: bad rational factor."))?;
                c *= v;
            }
            Sym::Symbol(s) if s.as_ref() == var => d += 1,
            Sym::Pow(b, e) => {
                let Sym::Symbol(s) = &**b else {
                    return Err(unsupported_q_shape());
                };
                if s.as_ref() != var {
                    return Err(unsupported_q_shape());
                }
                let k = as_small_int(e).ok_or(CasError::Unsupported(
                    "dsolve: fractional/negative powers of x in the RHS are outside the undetermined-coefficients subset.",
                ))?;
                if k < 1 {
                    return Err(unsupported_q_shape());
                }
                d += k;
            }
            Sym::Fn(name, args) if args.len() == 1 => {
                match name.as_ref() {
                    "exp" => {
                        if ea.is_some() {
                            return Err(unsupported_q_shape());
                        }
                        let (slope, inter) =
                            crate::int::as_linear(&args[0], var).ok_or_else(unsupported_q_shape)?;
                        if !is_sym_zero(&inter) {
                            return Err(CasError::Unsupported(
                                "dsolve: exp(a·x + c) with c ≠ 0 — normalize the RHS first; outside the documented subset.",
                            ));
                        }
                        ea = Some(sym_to_rat(&slope)?);
                    }
                    "cos" | "sin" => {
                        if tr.is_some() {
                            return Err(unsupported_q_shape());
                        }
                        let (freq, inter) =
                            crate::int::as_linear(&args[0], var).ok_or_else(unsupported_q_shape)?;
                        if !is_sym_zero(&inter) {
                            return Err(CasError::Unsupported(
                                "dsolve: sin/cos(b·x + c) with c ≠ 0 — normalize the RHS first; outside the documented subset.",
                            ));
                        }
                        let mut b = sym_to_rat(&freq)?;
                        let mut is_sin = name.as_ref() == "sin";
                        if b.is_negative() {
                            // cos(−bx) = cos(bx) و sin(−bx) = −sin(bx)
                            b = -&b;
                            if is_sin {
                                c = -&c;
                            }
                            is_sin = !is_sin;
                        }
                        tr = Some((is_sin, b));
                    }
                    _ => return Err(unsupported_q_shape()),
                }
            }
            _ => return Err(unsupported_q_shape()),
        }
    }
    Ok(QTerm { c, d, ea, tr })
}

fn unsupported_q_shape() -> CasError {
    CasError::Unsupported(
        "dsolve: RHS shape outside the undetermined-coefficients subset (polynomial, e^{ax}, cos/sin(bx) and their products with rational coefficients) — use the numeric ode45 route.",
    )
}

fn sym_to_rat(s: &Sym) -> Result<BigRational, CasError> {
    s.as_exact().ok_or(CasError::Unsupported(
        "dsolve: non-rational coefficient in the RHS family.",
    ))
}

fn as_small_int(e: &Sym) -> Option<i64> {
    if let Sym::Integer(i) = e {
        use num_traits::ToPrimitive;
        return i.to_i64();
    }
    None
}

/// چندگانگی ریشهٔ متناظر خانواده — برای تشدید حدس.
fn resonance(
    ea: &Option<BigRational>,
    tr: &Option<(bool, BigRational)>,
    roots: &[(RootVal, usize)],
) -> usize {
    let a = ea.clone().unwrap_or_else(|| rq(0));
    let b = tr.as_ref().map(|(_, b)| b.clone()).unwrap_or_else(|| rq(0));
    if b.is_zero() {
        return roots
            .iter()
            .filter(|(rv, _)| matches!(rv, RootVal::Rat(r) if *r == a))
            .map(|(_, m)| *m)
            .sum();
    }
    roots
        .iter()
        .filter(|(rv, _)| matches!(rv, RootVal::Cx(al, be) if *al == a && sym_rat(be).map(|v| v == b).unwrap_or(false)))
        .map(|(_, m)| *m)
        .sum()
}

/// β نمادین → گویا وقتی Rational است (برای مقایسهٔ تشدید).
fn sym_rat(s: &Sym) -> Option<BigRational> {
    match s {
        Sym::Rational(r) => Some(r.clone()),
        _ => None,
    }
}

/// ضرایب نامعین — گروه‌بندی پهن برای حدس (cos/sin هم‌بسامد جفت می‌سازند)
/// و کلید باریک (نوع تریگ + درجه) برای معادلات ضرایب.
fn undetermined(
    q: &Sym,
    coefs: &[Option<BigRational>],
    roots: &[(RootVal, usize)],
    var: &str,
) -> Result<Sym, CasError> {
    let qs = simplify(q)?;
    let terms = match &qs {
        Sym::Add(ts) => ts.clone(),
        other => vec![other.clone()],
    };
    let items: Vec<QTerm> = terms
        .iter()
        .map(|t| classify_q(t, var))
        .collect::<Result<_, _>>()?;
    // خانواده‌های پهن: (ea، بسامد |b|، بیشینهٔ درجه)
    let mut fams: Vec<(Option<BigRational>, Option<BigRational>, i64)> = Vec::new();
    for it in &items {
        let b = it.tr.as_ref().map(|(_, b)| b.clone());
        if let Some(f) = fams.iter_mut().find(|f| f.0 == it.ea && f.1 == b) {
            f.2 = f.2.max(it.d);
        } else {
            fams.push((it.ea.clone(), b, it.d));
        }
    }
    // اپراتور خطی: L(y) = Σ coef_k·y^{(k)}.
    let op = |e: &Sym| -> Result<Sym, CasError> {
        let mut acc = Sym::int(0);
        for (k, c) in coefs.iter().enumerate() {
            if let Some(c) = c {
                let dv = if k == 0 {
                    e.clone()
                } else {
                    diff_n(e, var, k as i64)?
                };
                acc = Sym::plus(acc, Sym::times(Sym::Rational(c.clone()), dv));
            }
        }
        simplify(&acc)
    };
    let mut y_p = Sym::int(0);
    for (ea, fb, dmax) in &fams {
        let tr = fb.clone().map(|b| (false, b)); // فقط برای resonance
        let s = resonance(ea, &tr, roots);
        // گسترش تطبیقی: دستگاه بی‌راه → درجهٔ حدس را زیاد کن (تشدید عمیق).
        let mut d_try = *dmax;
        let mut sol_vals: Option<Vec<Sym>> = None;
        for _ in 0..4 {
            let tr_arg = fb.clone().map(|b| (false, b));
            let bases = family_bases(ea, &tr_arg, s, d_try, var)?;
            let m = bases.len();
            if m == 0 {
                break;
            }
            // پاسخ هر پایه + تجزیهٔ باریک: (ea، نوع تریگ، درجه) → ضریب.
            let mut resp: Vec<Vec<RespDec>> = Vec::new();
            let mut foreign = false;
            for b in &bases {
                let r = op(b)?;
                let rt = match &r {
                    Sym::Add(ts) => ts.clone(),
                    other => vec![other.clone()],
                };
                let mut dec: Vec<RespDec> = Vec::new();
                for t in &rt {
                    let qt = classify_q(t, var)?;
                    // گارد خانوادهٔ پهن: پاسخ باید در همان (ea، |b|) بماند.
                    let term_b = qt.tr.as_ref().map(|(_, b)| b.clone());
                    if qt.ea != *ea || term_b != *fb {
                        foreign = true;
                    }
                    let is_sin = qt.tr.map(|(is, _)| is).unwrap_or(false);
                    dec.push(((is_sin, term_b), qt.d, qt.c));
                }
                if foreign {
                    break;
                }
                resp.push(dec);
            }
            if foreign {
                // ساده‌ساز توان خانواده را نگه نداشت — صادقانه بیرون.
                return Err(unsupported_q_shape());
            }
            // کلیدهای باریک سطرها.
            let narrow: Vec<(bool, Option<BigRational>)> = match fb {
                None => vec![(false, None)],
                Some(b) => vec![(false, Some(b.clone())), (true, Some(b.clone()))],
            };
            let degs = (s as i64) + d_try;
            let mut a_mat: Vec<Vec<Sym>> = Vec::new();
            let mut b_vec: Vec<Sym> = Vec::new();
            for (is_sin, bval) in &narrow {
                for deg in 0..=degs {
                    let mut row: Vec<Sym> = vec![Sym::int(0); m];
                    for (j, dec) in resp.iter().enumerate() {
                        let v: BigRational = dec
                            .iter()
                            .filter(|((is2, b2), d, _)| {
                                *is2 == *is_sin && b2.as_ref() == bval.as_ref() && *d == deg
                            })
                            .map(|(_, _, c)| c.clone())
                            .sum();
                        row[j] = Sym::Rational(v);
                    }
                    let rhs_v: BigRational = items
                        .iter()
                        .filter(|it| {
                            let is2 = it.tr.as_ref().map(|(is, _)| *is).unwrap_or(false);
                            let b2 = it.tr.as_ref().map(|(_, b)| b.clone());
                            is2 == *is_sin
                                && b2.as_ref() == bval.as_ref()
                                && it.ea == *ea
                                && it.d == deg
                        })
                        .map(|it| it.c.clone())
                        .sum();
                    a_mat.push(row);
                    b_vec.push(Sym::Rational(rhs_v));
                }
            }
            match crate::solve::gaussian_particular(&a_mat, &b_vec)? {
                Some(vals) => {
                    sol_vals = Some(vals);
                    break;
                }
                None => d_try += 1,
            }
        }
        let Some(vals) = sol_vals else {
            return Err(CasError::Unsupported(
                "dsolve: undetermined-coefficients system had no solution — honest failure.",
            ));
        };
        let tr_arg = fb.clone().map(|b| (false, b));
        let bases = family_bases(ea, &tr_arg, s, d_try, var)?;
        for (v, b) in vals.into_iter().zip(bases) {
            if !is_sym_zero(&v) {
                y_p = Sym::plus(y_p, Sym::times(v, b));
            }
        }
    }
    simplify(&y_p)
}

/// پایه‌های حدس خانواده: x^{s+j}·[e^{ax}]·{1 | cos(bx)+sin(bx)} برای j ≤ dmax.
fn family_bases(
    ea: &Option<BigRational>,
    tr: &Option<(bool, BigRational)>,
    s: usize,
    dmax: i64,
    var: &str,
) -> Result<Vec<Sym>, CasError> {
    let mut out = Vec::new();
    match tr {
        None => {
            let a = ea.clone().unwrap_or_else(|| rq(0));
            for j in 0..=dmax {
                out.push(exp_basis(
                    &Sym::Rational(a.clone()),
                    (s as i64 + j) as usize,
                    var,
                )?);
            }
        }
        Some((_, b)) => {
            let a = ea.clone().unwrap_or_else(|| rq(0));
            let bx = if *b == rq(1) {
                Sym::symbol(var)
            } else {
                Sym::times(Sym::Rational(b.clone()), Sym::symbol(var))
            };
            for j in 0..=dmax {
                out.push(trig_basis(&a, (s as i64 + j) as usize, "cos", &bx, var)?);
                out.push(trig_basis(&a, (s as i64 + j) as usize, "sin", &bx, var)?);
            }
        }
    }
    Ok(out)
}

// ============================================================
// تست‌ها
// ============================================================
#[cfg(test)]
mod tests {
    use super::*;
    use crate::sym_display;
    use std::sync::Arc;

    /// گرهٔ y(x) — تابع ناشناخته با سازندهٔ مستقیم (fn_of جدول CAS-1 را
    /// اعتبارسنجی می‌کند و «y» را نمی‌پذیرد؛ همین قرارداد در سیم‌کشی فاز ۳).
    fn fy() -> Sym {
        Sym::Fn(Arc::from("y"), vec![Sym::symbol("x")])
    }
    /// مشتق n-ام کانونی: diff^n(y(x), x).
    fn dy(n: usize) -> Sym {
        let mut inner = fy();
        for _ in 0..n {
            inner = Sym::Fn(Arc::from("diff"), vec![inner, Sym::symbol("x")]);
        }
        inner
    }
    fn s(n: &str) -> Sym {
        Sym::symbol(n)
    }
    fn i(n: i64) -> Sym {
        Sym::int(n)
    }
    fn add(items: Vec<Sym>) -> Sym {
        items.into_iter().reduce(Sym::plus).expect("add")
    }
    fn mul(items: Vec<Sym>) -> Sym {
        items.into_iter().reduce(Sym::times).expect("mul")
    }
    fn f(name: &str, a: Sym) -> Sym {
        Sym::fn_of(name, vec![a]).expect("fn")
    }
    fn run(eq: Sym) -> String {
        match dsolve(&eq) {
            Ok(v) => v.iter().map(sym_display).collect::<Vec<_>>().join("  OR  "),
            Err(e) => format!("ERR {e:?}"),
        }
    }

    #[test]
    fn smoke_linear() {
        println!(
            "linear y'=x*y   -> {}",
            run(Sym::minus(dy(1), mul(vec![s("x"), fy()])))
        );
        println!(
            "linear y'+y=e^x -> {}",
            run(add(vec![dy(1), fy(), Sym::negated(f("exp", s("x")))]))
        );
        println!(
            "linear y'-2y=4  -> {}",
            run(add(vec![
                dy(1),
                Sym::negated(mul(vec![i(2), fy()])),
                Sym::negated(i(4))
            ]))
        );
        println!(
            "linear y'+y=x   -> {}",
            run(add(vec![dy(1), fy(), Sym::negated(s("x"))]))
        );
        println!(
            "linear y'+y/x=x -> {}",
            run(add(vec![
                dy(1),
                mul(vec![fy(), Sym::pow(s("x"), i(-1)).expect("p")]),
                Sym::negated(s("x"))
            ]))
        );
    }

    #[test]
    fn smoke_separable() {
        println!(
            "sep y'=e^(x+y)  -> {}",
            run(Sym::minus(dy(1), f("exp", Sym::plus(s("x"), fy()))))
        );
        println!(
            "sep y'=y^2      -> {}",
            run(Sym::minus(dy(1), Sym::pow(fy(), i(2)).expect("p")))
        );
        println!(
            "sep y'=x*y^2    -> {}",
            run(Sym::minus(
                dy(1),
                mul(vec![s("x"), Sym::pow(fy(), i(2)).expect("p")])
            ))
        );
        println!(
            "sep y'=x*e^y    -> {}",
            run(Sym::minus(dy(1), mul(vec![s("x"), f("exp", fy())])))
        );
        println!(
            "sep y'=x*(1+y^2)-> {}",
            run(Sym::minus(
                dy(1),
                mul(vec![
                    s("x"),
                    add(vec![i(1), Sym::pow(fy(), i(2)).expect("p")])
                ])
            ))
        );
        println!(
            "sep y'=cos(x)e^y-> {}",
            run(Sym::minus(
                dy(1),
                mul(vec![f("cos", s("x")), f("exp", fy())])
            ))
        );
    }

    #[test]
    fn smoke_exact_mu() {
        println!(
            "exact 2xy+1+x^2y' -> {}",
            run(add(vec![
                mul(vec![i(2), s("x"), fy()]),
                i(1),
                mul(vec![Sym::pow(s("x"), i(2)).expect("p"), dy(1)])
            ]))
        );
        println!(
            "exact ycosx+1     -> {}",
            run(add(vec![
                mul(vec![fy(), f("cos", s("x"))]),
                i(1),
                mul(vec![f("sin", s("x")), dy(1)])
            ]))
        );
        // μ(y): آشکارساز و نسبت در سطح واحد سبز است (dbg3)؛ مورد e2e
        // با log/exp در F به فاز ۳ موکول می‌شود — محدودیت مستند.
        println!("mu-y detector      -> unit-verified (see dbg3)");
    }

    #[test]
    fn smoke_const_coeff() {
        println!("cc y''+y=0     -> {}", run(add(vec![dy(2), fy()])));
        println!("cc y''-y=0     -> {}", run(Sym::minus(dy(2), fy())));
        println!(
            "cc y''+2y'+y=0 -> {}",
            run(add(vec![dy(2), mul(vec![i(2), dy(1)]), fy()]))
        );
        println!("cc y^3=0      -> {}", run(dy(3)));
        println!("cc y''+y'+y=0  -> {}", run(add(vec![dy(2), dy(1), fy()])));
        println!(
            "cc y''+4y'+13y -> {}",
            run(add(vec![
                dy(2),
                mul(vec![i(4), dy(1)]),
                mul(vec![i(13), fy()])
            ]))
        );
        println!(
            "cc y''-3y'+2y  -> {}",
            run(add(vec![
                dy(2),
                Sym::negated(mul(vec![i(3), dy(1)])),
                mul(vec![i(2), fy()])
            ]))
        );
    }

    #[test]
    fn smoke_undetermined() {
        println!(
            "uc y''-y=x        -> {}",
            run(add(vec![dy(2), Sym::negated(fy()), Sym::negated(s("x"))]))
        );
        println!(
            "uc y''+y=e^x      -> {}",
            run(add(vec![dy(2), fy(), Sym::negated(f("exp", s("x")))]))
        );
        println!(
            "uc y''+y=sin x    -> {}",
            run(add(vec![dy(2), fy(), Sym::negated(f("sin", s("x")))]))
        );
        println!(
            "uc y''-4y=e^2x    -> {}",
            run(add(vec![
                dy(2),
                Sym::negated(mul(vec![i(4), fy()])),
                Sym::negated(f("exp", mul(vec![i(2), s("x")])))
            ]))
        );
        println!(
            "uc y''+2y'=x+e^x  -> {}",
            run(add(vec![
                dy(2),
                mul(vec![i(2), dy(1)]),
                Sym::negated(s("x")),
                Sym::negated(f("exp", s("x")))
            ]))
        );
        println!(
            "uc y''-2y'+y=excos-> {}",
            run(add(vec![
                dy(2),
                Sym::negated(mul(vec![i(2), dy(1)])),
                fy(),
                Sym::negated(mul(vec![f("exp", s("x")), f("cos", s("x"))]))
            ]))
        );
        println!(
            "uc y''+2y'+y=e^-x -> {}",
            run(add(vec![
                dy(2),
                mul(vec![i(2), dy(1)]),
                fy(),
                Sym::negated(f("exp", Sym::negated(s("x"))))
            ]))
        );
        println!(
            "uc y''+y=x^2      -> {}",
            run(add(vec![
                dy(2),
                fy(),
                Sym::negated(Sym::pow(s("x"), i(2)).expect("p"))
            ]))
        );
        println!(
            "uc y''+y=cos2x    -> {}",
            run(add(vec![
                dy(2),
                fy(),
                Sym::negated(f("cos", mul(vec![i(2), s("x")])))
            ]))
        );
    }

    #[test]
    fn smoke_honest_failures() {
        // Bernoulli: y' + y = y² → خارج از زیرمجموعه، خطای صادقانه.
        let bern = add(vec![
            dy(1),
            fy(),
            Sym::negated(Sym::pow(fy(), i(2)).expect("p")),
        ]);
        match dsolve(&bern) {
            Err(e) => println!("bernoulli -> ERR (درست): {e:?}"),
            Ok(v) => panic!("bernoulli نباید حل می‌شد: {v:?}"),
        }
        // معادلهٔ سادهٔ مرتبهٔ اول — باید سبز باشد.
        let simple = Sym::minus(dy(1), fy());
        let out = run(simple);
        println!("plain y'=y      -> {out}");
        assert!(!out.starts_with("ERR"));
        // دو مجهول — ابهام صادقانه.
        let two = add(vec![dy(1), Sym::Fn(Arc::from("z"), vec![s("x")])]);
        match dsolve(&two) {
            Err(e) => println!("two unknowns -> ERR (درست): {e:?}"),
            Ok(v) => panic!("دو مجهول نباید حل می‌شد: {v:?}"),
        }
    }
}

#[cfg(test)]
mod dbg_tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn dbg_integrate_reciprocal() {
        let y = Sym::symbol("y");
        let emy = Sym::fn_of("exp", vec![Sym::negated(y.clone())]).expect("f");
        println!("int exp(-y)   -> {:?}", integrate(&emy, "y"));
        let ex = Sym::fn_of("exp", vec![Sym::symbol("x")]).expect("f");
        println!("int exp(x)    -> {:?}", integrate(&ex, "x"));
        // حالت e^(x+y): g بعد از جایگذاری x→0
        let fy = Sym::Fn(Arc::from("y"), vec![Sym::symbol("x")]);
        let g = Sym::fn_of("exp", vec![Sym::plus(Sym::int(0), fy.clone())]).expect("f");
        let gs = simplify(&g).expect("s");
        println!("g simplified  -> {gs:?}");
        let rec = reciprocal_form(&gs);
        println!("reciprocal    -> {rec:?}");
        println!("int rec 'y'   -> {:?}", integrate(&rec, "y"));
        // و H برای 1/(1+y^2)
        let g2 = Sym::plus(Sym::int(1), Sym::pow(fy.clone(), Sym::int(2)).expect("p"));
        let r2 = reciprocal_form(&simplify(&g2).expect("s"));
        println!("rec 1+y^2     -> {r2:?}");
        println!("int rec2 'y'  -> {:?}", integrate(&r2, "y"));
    }
}

#[cfg(test)]
mod dbg2 {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn dbg_invert_special_exp() {
        // eq = −exp(−y) − (C1 + e^(x+1)/e) — دقیقاً شکل e^(x+y)
        let fy = Sym::Fn(Arc::from("y"), vec![Sym::symbol("x")]);
        let emy = Sym::fn_of("exp", vec![Sym::negated(fy.clone())]).expect("f");
        let ex1 = Sym::fn_of("exp", vec![Sym::plus(Sym::symbol("x"), Sym::int(1))]).expect("f");
        let einv = Sym::pow(
            Sym::fn_of("exp", vec![Sym::int(1)]).expect("f"),
            Sym::int(-1),
        )
        .expect("p");
        let rest = Sym::negated(Sym::plus(Sym::symbol("C1"), Sym::times(einv, ex1)));
        let eq = Sym::plus(Sym::negated(emy.clone()), rest);
        let es = simplify(&eq).expect("s");
        println!("eq simplified: {es:?}");
        let r = invert_special(&es, "y").expect("inv");
        println!("special branches: {r:?}");
        // همان eq برای x·e^y که کار می‌کند
        let eq2 = simplify(&Sym::plus(
            Sym::negated(emy),
            Sym::negated(Sym::plus(
                Sym::symbol("C1"),
                Sym::times(
                    Sym::Rational(num_rational::BigRational::new(
                        num_bigint::BigInt::from(1),
                        num_bigint::BigInt::from(2),
                    )),
                    Sym::pow(Sym::symbol("x"), Sym::int(2)).expect("p"),
                ),
            )),
        ))
        .expect("s");
        println!("eq2: {eq2:?}");
        println!("special2: {:?}", invert_special(&eq2, "y").expect("inv"));
    }
}

#[cfg(test)]
mod unit_mu {
    use super::*;
    use crate::sym_display;

    /// تقسیم دقیق و نسبت μ(y) روی جفت واقعی — M = 2x·y·log(y) + y.
    /// M/y = 2x·log(y) + 1 و (N_x − M_y)/M = −1/y (ماشین‌کاری μ).
    #[test]
    fn exact_div_and_mu_ratio() {
        let x = Sym::symbol("x");
        let y = Sym::symbol("y");
        let l = Sym::fn_of("log", vec![y.clone()]).expect("l");
        let m = Sym::plus(
            Sym::times(Sym::times(Sym::times(Sym::int(2), x.clone()), y.clone()), l),
            y.clone(),
        );
        let mp = RatFun::from_sym(&m).expect("mp");
        let bp = RatFun::from_sym(&Sym::symbol("y")).expect("bp");
        let q = exact_div_in(&mp.num, &bp.num, "y").expect("تقسیم دقیق باید می‌شد");
        let qs = simplify(&q).expect("qs");
        assert_eq!(sym_display(&qs), "2*x*log(y) + 1");
        let n = Sym::pow(x.clone(), Sym::int(2)).expect("p");
        let dmy = simplify(&diff_n(&m, "y", 1).expect("a")).expect("b");
        let dnx = simplify(&diff_n(&n, "x", 1).expect("c")).expect("d");
        let num = Sym::minus(dnx, dmy);
        let ratio = ratio_exact(&num, &m).expect("نسبت μ(y)");
        let rs = simplify(&ratio).expect("rs");
        assert_eq!(sym_display(&rs), "-1/y");
    }
}
