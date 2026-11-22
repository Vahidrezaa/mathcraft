//! گام 4.3/CAS-2 — موتور انتگرال‌گیری نمادین «Risch-lite» (سند ۰۵ §۵).
//!
//! پایپ‌لاین اجرایی روی هر جملهٔ جمع (با کش و بودجهٔ عمق):
//!   ۱. تجزیهٔ جمع + جداکردن ضریب x-آزاد
//!   ۲. مسیر گویا: تقسیم چندجمله‌ای (در میدان کسرهای پارامتری) +
//!      تجزیهٔ جزء به جزء دقیق — ریشه‌های گویا RRT روی مخرج عددی،
//!      مخرج پارامتری درجه ۱/۲ با فرمول درجه ۲ (لوگاریتمی/آرکتانژانت
//!      با تشخیص علامت دیسکریمینان). هم‌ارز Hermite reduction برای
//!      کلاس پوشش‌داده؛ Rothstein–Trager در v1 نیست: مخرج تجزیه‌ناپذیرِ
//!      درجه ≥ ۳ صادقانه None می‌شود، نه جواب غلطِ مطمئن‌نما.
//!   ۳. جایگذاری u-sub: کاندیدهای زیردرخت، آزمون g′ | h و u-تمیزی
//!   ۴. جدول فرم‌های استاندارد (مثلثاتی/هذلولی/وارون‌ها/حاصل‌ضرب‌ها/
//!      exp·sin چرخشی/لوگاریتم‌ها)
//!   ۵. جزء به جزء (by parts) با انتخاب LIATE و بودجهٔ عمق
//!   ۶. شکست صادقانه → None (فراخوان گرهٔ unevaluated می‌سازد)
//!
//! انضباط کیفیت (DoD سند ۰۵ §۵): هر نتیجهٔ برگردانده‌شده پیش از ارائه
//! راستی‌آزمایی می‌شود — نمادین (simplify(diff(F) − f) = 0) و در صورت
//! امکان عددی (verify_pair با تفاضل مرکزی + ریچاردسون). ردِ تأیید یعنی
//! None؛ هرگز جواب تأییدنشده.

use std::collections::HashMap;

use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::{Signed, Zero};

use crate::diff::{diff, node_count};
use crate::poly::{Mon, Poly, RatFun};
use crate::simplify::simplify;
use crate::solve;
use crate::sym::{as_small_int, CasError, Sym};
use crate::verify::{accepted, verify_pair};

/// بودجهٔ کل تودرتویی (u-sub + by-parts).
const MAX_DEPTH: u8 = 6;
/// سقف کاندیدهای جایگذاری در هر جمله.
const MAX_CANDIDATES: usize = 24;

// ---------------------------------------------------------------------------
// کمکی‌های درخت
// ---------------------------------------------------------------------------

/// آیا درخت حاوی نماد `x` است؟
pub(crate) fn contains(e: &Sym, x: &str) -> bool {
    match e {
        Sym::Symbol(s) => &**s == x,
        Sym::Integer(_) | Sym::Rational(_) | Sym::Real(_) => false,
        Sym::Add(terms) => terms.iter().any(|t| contains(t, x)),
        Sym::Mul(factors) => factors.iter().any(|f| contains(f, x)),
        Sym::Pow(b, ex) => contains(b, x) || contains(ex, x),
        Sym::Fn(_, args) => args.iter().any(|a| contains(a, x)),
    }
}

/// کلید تک‌جمله‌ای Poly برای متغیر — ژنراتورها با sexpr کامل نام‌گذاری
/// می‌شوند (قرارداد generator_poly) نه با نام سادهٔ نماد؛ همهٔ نگاه‌های
/// «چندجمله‌ای در x» از این کلید استفاده می‌کنند.
pub(crate) fn vkey(x: &str) -> String {
    crate::sexpr::to_sexp(&Sym::symbol(x))
}

/// جایگذاری ساختاری: هر رخدادِ دقیقِ `needle` با `repl`.
/// درخت‌ها کانونی‌اند پس برابری ساختاری قابل‌اطمینان است.
fn replace_sub(e: &Sym, needle: &Sym, repl: &Sym) -> Sym {
    if e == needle {
        return repl.clone();
    }
    match e {
        Sym::Add(terms) => fold_add(terms.iter().map(|t| replace_sub(t, needle, repl))),
        Sym::Mul(factors) => fold_mul(factors.iter().map(|f| replace_sub(f, needle, repl))),
        Sym::Pow(b, ex) => {
            match Sym::pow(replace_sub(b, needle, repl), replace_sub(ex, needle, repl)) {
                Ok(p) => p,
                // جایگذاری ساختار را نامعتبر کرد — ورودی اصلی برمی‌گردد
                Err(_) => e.clone(),
            }
        }
        Sym::Fn(name, args) => {
            let new_args: Vec<Sym> = args.iter().map(|a| replace_sub(a, needle, repl)).collect();
            match Sym::fn_of(name, new_args) {
                Ok(f) => f,
                Err(_) => e.clone(),
            }
        }
        _ => e.clone(),
    }
}

fn fold_add<I: Iterator<Item = Sym>>(mut it: I) -> Sym {
    let Some(first) = it.next() else {
        return Sym::int(0);
    };
    it.fold(first, Sym::plus)
}

fn fold_mul<I: Iterator<Item = Sym>>(mut it: I) -> Sym {
    let Some(first) = it.next() else {
        return Sym::int(1);
    };
    it.fold(first, Sym::times)
}

fn abs_of(u: &Sym) -> Result<Sym, CasError> {
    Sym::fn_of("abs", vec![u.clone()])
}

/// «واقعاً گویا در x»؟ — تابعی که آرگومانش x دارد یا توانِ غیرصحیح از
/// پایهٔ x-دار یا ضریب Real نه. (`RatFun::from_sym` توابع را ژنراتور
/// اُپک می‌گیرد؛ این تشخیص باید صریح باشد.)
fn is_rational_in(e: &Sym, x: &str) -> bool {
    match e {
        // خودِ نماد (چه متغیر چه پارامتر) همیشه گویا در x است — درس
        // باگ: s != x یعنی ∫x اصلاً وارد مسیر گویا نمی‌شد!
        Sym::Symbol(_) => true,
        Sym::Integer(_) | Sym::Rational(_) => true,
        Sym::Real(_) => false,
        Sym::Add(terms) => terms.iter().all(|t| is_rational_in(t, x)),
        Sym::Mul(factors) => factors.iter().all(|f| is_rational_in(f, x)),
        Sym::Pow(b, ex) => {
            if contains(b, x) {
                as_small_int(ex).is_some()
            } else {
                !contains(ex, x)
            }
        }
        Sym::Fn(_, args) => args.iter().all(|a| !contains(a, x)),
    }
}

/// جداکردن ضریب x-آزاد: (c, h) با f = c·h و h حاوی x.
fn split_const(f: &Sym, x: &str) -> (Sym, Sym) {
    if let Sym::Mul(factors) = f {
        let mut consts: Vec<Sym> = vec![];
        let mut rest: Vec<Sym> = vec![];
        for fac in factors {
            if contains(fac, x) {
                rest.push(fac.clone());
            } else {
                consts.push(fac.clone());
            }
        }
        if rest.is_empty() {
            return (Sym::int(1), f.clone());
        }
        let c = if consts.is_empty() {
            Sym::int(1)
        } else {
            fold_mul(consts.into_iter())
        };
        let h = if rest.len() == 1 {
            rest.remove(0)
        } else {
            fold_mul(rest.into_iter())
        };
        (c, h)
    } else {
        (Sym::int(1), f.clone())
    }
}

/// نماد تازه که با `taken` تصادم ندارد (u جایگذاری / پارامترهای PF).
fn fresh_sym(base: &str, taken: &[String]) -> Sym {
    for i in 0..64 {
        let name = format!("{base}{i}");
        if !taken.iter().any(|t| t == &name) {
            return Sym::symbol(&name);
        }
    }
    Sym::symbol(&format!("{base}_w"))
}

// ---------------------------------------------------------------------------
// موتور اصلی
// ---------------------------------------------------------------------------

/// انتگرال نامعین: `Some(F)` با dF/dx = f، یا `None` = شکست صادقانه.
pub fn integrate(f: &Sym, x: &str) -> Result<Option<Sym>, CasError> {
    let mut eng = Eng {
        cache: HashMap::new(),
        used: vec![],
    };
    let Some(raw) = eng.run(f, x, 0)? else {
        return Ok(None);
    };
    // انضباط DoD: هر نتیجه پیش از ارائه تأیید می‌شود.
    if verified(&raw, f, x) {
        Ok(Some(raw))
    } else {
        Ok(None)
    }
}

/// بازنویسی توان‌های فرد مثلثاتی/هذلولی — اتحادهای کل-دامنهٔ درست برای هر u:
/// sin^{2k+1} → sin·(1−cos²)^k، cos^{2k+1} → cos·(1−sin²)^k،
/// sinh^{2k+1} → sinh·(cosh²−1)^k، cosh^{2k+1} → cosh·(1+sinh²)^k.
/// درس مجموعهٔ مرجع: بدون این، u-sub روی sin³ هرگز u-تمیز نمی‌شد.
fn rewrite_odd_powers(h: &Sym) -> Sym {
    if let Sym::Pow(b, ex) = h {
        if let (Sym::Fn(name, args), Some(n)) = (b.as_ref(), as_small_int(ex)) {
            if args.len() == 1 && n > 1 && n % 2 == 1 {
                let (f, g, g_minus_one) = match &**name {
                    "sin" => ("sin", "cos", false),
                    "cos" => ("cos", "sin", false),
                    "sinh" => ("sinh", "cosh", true),
                    "cosh" => ("cosh", "sinh", true),
                    _ => ("", "", false),
                };
                if !f.is_empty() {
                    let u = args[0].clone();
                    let k = (n - 1) / 2;
                    let built: Option<Sym> = (|| {
                        // sin² = 1 − cos²؛ sinh² = cosh² − 1 (اتحاد کل-دامنه)
                        let fu = Sym::fn_of(f, vec![u.clone()]).ok()?;
                        let gu = Sym::fn_of(g, vec![u.clone()]).ok()?;
                        let g2 = Sym::pow(gu, Sym::int(2)).ok()?;
                        let base = if g_minus_one {
                            Sym::minus(g2, Sym::int(1))
                        } else {
                            Sym::minus(Sym::int(1), g2)
                        };
                        let powed = Sym::pow(base, Sym::int(k)).ok()?;
                        Some(Sym::times(fu, powed))
                    })();
                    if let Some(res) = built {
                        return res;
                    }
                }
            }
        }
        return Sym::pow(rewrite_odd_powers(b), rewrite_odd_powers(ex))
            .unwrap_or_else(|_| h.clone());
    }
    match h {
        Sym::Add(ts) => fold_add(ts.iter().map(rewrite_odd_powers)),
        Sym::Mul(fs) => fold_mul(fs.iter().map(rewrite_odd_powers)),
        Sym::Fn(name, args) => Sym::Fn(name.clone(), args.iter().map(rewrite_odd_powers).collect()),
        other => other.clone(),
    }
}

/// توزیع ضرب روی جمع — فقط وقتی عضوی Add است؛ بودجهٔ ۸ جمله (درس مرجع:
/// exp(x)·(sin(x)+cos(x))) — پس از توزیع، مسیر جمع جمله‌ها را مستقل می‌گیرد.
fn distribute_mul_over_add(factors: &[Sym]) -> Option<Sym> {
    let (i, add) = factors
        .iter()
        .enumerate()
        .find(|(_, f)| matches!(f, Sym::Add(_)))?;
    let terms = match add {
        Sym::Add(v) => v,
        _ => return None,
    };
    if terms.len() > 8 {
        return None; // بودجه — انفجار توزیع صادقانه رد می‌شود
    }
    let mut out: Option<Sym> = None;
    for t in terms {
        let mut prod = t.clone();
        for (k, f) in factors.iter().enumerate() {
            if k != i {
                prod = Sym::times(prod, f.clone());
            }
        }
        out = Some(match out {
            None => prod,
            Some(a) => Sym::plus(a, prod),
        });
    }
    out
}

fn verified(f_res: &Sym, f: &Sym, x: &str) -> bool {
    // ۱) نمادین خام: simplify(diff(F) − f) = 0
    if let Ok(d) = diff(f_res, x) {
        let residue = Sym::minus(d, f.clone());
        if let Ok(z) = simplify(&residue) {
            if is_sym_zero(&z) {
                return true;
            }
        }
        // ۱ب) نمادین با نرمال‌سازی abs/sign — log|u| روی دامنهٔ u>0 با
        // log(u) هم‌ارز است؛ برای ورفای رسمِ پادمشتق روی یک ناحیهٔ باز
        // کافی است (رویهٔ استاندارد CASها با شاخه‌ها).
        let stripped = strip_abs_sign(&residue);
        if let Ok(z) = simplify(&stripped) {
            if is_sym_zero(&z) {
                return true;
            }
        }
    }
    // ۲) عددی: تفاضل مرکزی + ریچاردسون روی نقاط قطعی
    if let Ok(rep) = verify_pair(f_res, f, x) {
        if accepted(&rep) {
            return true;
        }
    }
    // ۳) عددی با نمونهٔ پارامترها — پادمشتق‌های پارامتر‌دار (a·log|…|)
    // را می‌توان با مقدارگذاری قطعی پارامترها راستی‌آزمایی کرد.
    if !f.free_symbols().is_empty() {
        let params: Vec<String> = f.free_symbols().into_iter().filter(|s| s != x).collect();
        // مقادیر کوچک ناصفر قطعی — از تصادم با ریشه‌های ساده پرهیز
        let sample: Vec<(String, Sym)> = params
            .iter()
            .enumerate()
            .map(|(i, p)| {
                (
                    p.clone(),
                    Sym::rat(3 + 2 * i as i64, 5).unwrap_or_else(|_| Sym::int(1)),
                )
            })
            .collect();
        let fs = substitute_all(f, &sample);
        let fres = substitute_all(f_res, &sample);
        if let Ok(rep) = verify_pair(&fres, &fs, x) {
            if accepted(&rep) {
                return true;
            }
        }
    }
    false
}

/// حذف abs/sign برای ورفای نمادین (روی دامنهٔ مثبت آرگومان).
fn strip_abs_sign(e: &Sym) -> Sym {
    match e {
        Sym::Fn(name, args) if &**name == "abs" && args.len() == 1 => args[0].clone(),
        Sym::Fn(name, args) if &**name == "sign" && args.len() == 1 => Sym::int(1),
        Sym::Fn(name, args) => {
            let new_args: Vec<Sym> = args.iter().map(strip_abs_sign).collect();
            Sym::Fn(name.clone(), new_args)
        }
        Sym::Add(ts) => fold_add(ts.iter().map(strip_abs_sign)),
        Sym::Mul(fs) => fold_mul(fs.iter().map(strip_abs_sign)),
        Sym::Pow(b, ex) => match Sym::pow(strip_abs_sign(b), strip_abs_sign(ex)) {
            Ok(p) => p,
            Err(_) => e.clone(),
        },
        _ => e.clone(),
    }
}

/// جایگذاری چند نماد هم‌زمان.
fn substitute_all(e: &Sym, subs: &[(String, Sym)]) -> Sym {
    let mut out = e.clone();
    for (name, val) in subs {
        out = out.substitute(name, val);
    }
    out
}

struct Eng {
    cache: HashMap<String, Option<Sym>>,
    /// نام‌های موقتِ مصرف‌شده (u جایگذاری) — ضد تصادم بین سطوح تودرتو.
    used: Vec<String>,
}

impl Eng {
    fn run(&mut self, f: &Sym, x: &str, depth: u8) -> Result<Option<Sym>, CasError> {
        if depth > MAX_DEPTH {
            return Ok(None);
        }
        let key = format!("{}\u{1}{}", x, crate::sexpr::to_sexp(f));
        if let Some(hit) = self.cache.get(&key) {
            return Ok(hit.clone());
        }
        let res = self.run_uncached(f, x, depth)?;
        self.cache.insert(key, res.clone());
        Ok(res)
    }

    fn run_uncached(&mut self, f: &Sym, x: &str, depth: u8) -> Result<Option<Sym>, CasError> {
        // ۰. ثابت در x → f·x
        if !contains(f, x) {
            return Ok(Some(Sym::times(f.clone(), Sym::symbol(x))));
        }
        // ۱. تجزیهٔ جمع — جمله به جمله؛ شکست یک جمله = شکست کل (صادقانه)
        if let Sym::Add(terms) = f {
            let mut acc: Option<Sym> = None;
            for t in terms {
                let Some(it) = self.run(t, x, depth)? else {
                    return Ok(None);
                };
                acc = Some(match acc {
                    None => it,
                    Some(a) => Sym::plus(a, it),
                });
            }
            return Ok(acc);
        }
        // ۲. ضریب ثابت بیرون
        let (c, h) = split_const(f, x);
        if !matches!(c, Sym::Integer(ref i) if *i == BigInt::from(1)) {
            let inner = self.run(&h, x, depth)?;
            return Ok(inner.map(|fi| Sym::times(c.clone(), fi)));
        }
        // ۲٫۵. بازنویسی توان‌های فرد مثلثاتی/هذلولی (درس مرجع: sin³/cos³):
        // sin^{2k+1} → sin·(1−cos²)^k — اتحاد کل-دامنه — تا u-sub با
        // u=cos تمیز شود. نتیجه با h اصلی ورفای می‌شود.
        let h_rw = rewrite_odd_powers(&h);
        if h_rw != h {
            if let Some(fr) = self.run(&h_rw, x, depth)? {
                if verified(&fr, &h, x) {
                    return Ok(Some(fr));
                }
            }
        }
        // ۳–۶. هر مرحله نتیجه‌اش همان‌جا ورفای می‌شود؛ ردِ تأیید یعنی
        // «این مرحله موفق نشد» و مرحلهٔ بعد فرصت می‌گیرد — درس sin²(x):
        // مسیر گویا آن را به‌شکل غلط «موفق» می‌پنداشت و جدول را می‌بست.
        if is_rational_in(&h, x) {
            if let Some(fr) = self.int_rational(&h, x, depth)? {
                if verified(&fr, &h, x) {
                    return Ok(Some(fr));
                }
            }
        }
        if let Some(fu) = self.int_usub(&h, x, depth)? {
            if verified(&fu, &h, x) {
                return Ok(Some(fu));
            }
        }
        if let Some(ft) = table_form(&h, x) {
            if verified(&ft, &h, x) {
                return Ok(Some(ft));
            }
        }
        // ۵٫۵. توزیع ضرب روی جمع (درس مرجع: exp(x)·(sin+cos)) — با بودجهٔ
        // تعداد جمله؛ پس از آن مسیر جمع، هر جمله را مستقل می‌گیرد
        if let Sym::Mul(factors) = &h {
            if factors.iter().any(|f| matches!(f, Sym::Add(_))) {
                if let Some(dist) = distribute_mul_over_add(factors) {
                    if let Some(fr) = self.run(&dist, x, depth)? {
                        if verified(&fr, &h, x) {
                            return Ok(Some(fr));
                        }
                    }
                }
            }
        }
        if let Some(fb) = self.int_by_parts(&h, x, depth)? {
            if verified(&fb, &h, x) {
                return Ok(Some(fb));
            }
        }
        Ok(None)
    }

    // ---------------------- مسیر گویا ----------------------
    fn int_rational(&mut self, h: &Sym, x: &str, depth: u8) -> Result<Option<Sym>, CasError> {
        let rf = RatFun::from_sym(h)?;
        if rf.is_zero() {
            return Ok(Some(Sym::int(0)));
        }
        let den_x = rf.den.degree_in(&vkey(x));
        if den_x == 0 {
            // مخرج x-آزاد → قاعدهٔ توان روی صورت، ÷ مخرج
            let ip = int_poly_x(&rf.num, x)?;
            let inv_den = RatFun {
                num: Poly::one(),
                den: rf.den.clone(),
            }
            .to_sym()?;
            return Ok(Some(Sym::times(ip, inv_den)));
        }
        // تقسیم طولانی در میدان کسرهای پارامتری
        let (q_sym, rem) = pdiv_x_fraction(&rf.num, &rf.den, x)?;
        // درس مجموعهٔ مرجع ۱۲۰ (مورد ۲۹): خارج‌قسمت باید انتگرال بگیرد
        // نه خام بماند — ∫(q + r/d) = ∫q + ∫r/d. نسخهٔ اولیه q را خام
        // می‌گذاشت؛ ورفای درستِ آن را رد می‌کرد و موتور به مسیرهای معلق
        // (by-parts → abs تودرتو → gcd ژنراتورهای اُپک) می‌افتاد.
        let qp = RatFun::from_sym(&q_sym)?.to_poly()?;
        let mut out = int_poly_x(&qp, x)?;
        if !rem.is_zero() {
            let Some(rest) = self.int_proper(&rem.num, &rem.den, x, depth)? else {
                return Ok(None);
            };
            out = Sym::plus(out, rest);
        }
        Ok(Some(out))
    }

    /// کسر سرراست (deg_x r < deg_x den) → PF دقیق.
    fn int_proper(
        &mut self,
        r: &Poly,
        den: &Poly,
        x: &str,
        _depth: u8,
    ) -> Result<Option<Sym>, CasError> {
        let den_x = den.degree_in(&vkey(x));
        let den_pure = den.generators().iter().all(|g| g == &vkey(x));
        if den_pure {
            self.pf_numeric(r, den, x)
        } else if den_x == 1 {
            self.pf_linear(r, den, x)
        } else if den_x == 2 {
            self.pf_quad(r, den, x)
        } else {
            // مخرج پارامتری درجه ≥ ۳ در x — صادقانه رد
            Ok(None)
        }
    }

    /// مخرج خطی پارامتری: (r1·x + r0)/(a1·x + a0).
    /// تقسیم: r1/a1 + (r0 − r1·a0/a1)/(a1·x + a0) → جملهٔ لوگاریتمی.
    fn pf_linear(&mut self, r: &Poly, den: &Poly, x: &str) -> Result<Option<Sym>, CasError> {
        let a1 = coeff_of(den, x, 1);
        let a0 = coeff_of(den, x, 0);
        let r1 = coeff_of(r, x, 1);
        let r0 = coeff_of(r, x, 0);
        if a1.is_zero() {
            return Ok(None);
        }
        let one = Poly::one();
        let rf_a1 = RatFun {
            num: a1.clone(),
            den: one.clone(),
        };
        let rf_a0 = RatFun {
            num: a0.clone(),
            den: one.clone(),
        };
        let rf_r1 = RatFun {
            num: r1,
            den: one.clone(),
        };
        let rf_r0 = RatFun {
            num: r0,
            den: one.clone(),
        };
        let lin = Sym::plus(Sym::times(rf_a1.to_sym()?, Sym::symbol(x)), rf_a0.to_sym()?);
        let r1_over_a1 = rf_r1.div(&rf_a1)?;
        let quo = Sym::times(r1_over_a1.to_sym()?, Sym::symbol(x));
        // باقیماندهٔ مؤثر: r0 − (r1/a1)·a0
        let prod = r1_over_a1.mul(&rf_a0)?;
        let c_rem = rf_r0.sub(&prod)?;
        // ∫ c_rem/(a1·x + a0) = (c_rem/a1)·log|a1·x + a0|
        let coef = c_rem.div(&rf_a1)?.to_sym()?;
        let logf = Sym::fn_of("log", vec![abs_of(&lin)?])?;
        Ok(Some(Sym::plus(quo, Sym::times(coef, logf))))
    }

    /// مخرج درجه ۲ (عددی یا پارامتری): α·log|q| + β·∫dx/q.
    fn pf_quad(&mut self, r: &Poly, den: &Poly, x: &str) -> Result<Option<Sym>, CasError> {
        let a2 = coeff_of(den, x, 2);
        let a1 = coeff_of(den, x, 1);
        let a0 = coeff_of(den, x, 0);
        if a2.is_zero() {
            return Ok(None);
        }
        let qs = RatFun {
            num: den.clone(),
            den: Poly::one(),
        }
        .to_sym()?;
        let (r1, r0) = (coeff_of(r, x, 1), coeff_of(r, x, 0));
        let two_a2 = Poly::add(&a2, &a2);
        // α = r1/(2a2)؛ β = r0 − α·a1
        let alpha = RatFun {
            num: r1.clone(),
            den: two_a2.clone(),
        }
        .to_sym()?;
        let a1s = RatFun {
            num: a1.clone(),
            den: Poly::one(),
        }
        .to_sym()?;
        let beta = Sym::minus(
            RatFun {
                num: r0,
                den: Poly::one(),
            }
            .to_sym()?,
            Sym::times(alpha.clone(), a1s.clone()),
        );
        let log_part = Sym::times(alpha, Sym::fn_of("log", vec![abs_of(&qs)?])?);
        let Some(dxq) = int_dx_over_quad(&a2, &a1, &a0, x)? else {
            return Ok(None);
        };
        Ok(Some(Sym::plus(log_part, Sym::times(beta, dxq))))
    }

    /// مخرج عددی یک‌متغیره: RRT + دنبالهٔ درجه ۲ + PF با دستگاه خطی.
    fn pf_numeric(&mut self, r: &Poly, den: &Poly, x: &str) -> Result<Option<Sym>, CasError> {
        // مونیک‌سازی مخرج عددی (درس مورد ۳۰): ضریب رأس → بیرون —
        // 1/(4x²+4x+1) = (1/4)·1/(x+1/2)²؛ بدون آن PF با فاکتور مونیک
        // ناسازگار می‌شد و ورفای ۴ برابر خطا را رد می‌کرد
        let vk = vkey(x);
        let lead = coeff_of(den, x, den.degree_in(&vk))
            .constant_value()
            .filter(|l| !l.is_zero());
        if let Some(l) = lead {
            if l != BigRational::from_integer(BigInt::from(1)) {
                let r_monic = Poly {
                    terms: r.terms.iter().map(|(m, c)| (m.clone(), c / &l)).collect(),
                };
                let den_monic = Poly {
                    terms: den.terms.iter().map(|(m, c)| (m.clone(), c / &l)).collect(),
                };
                return self.pf_numeric(&r_monic, &den_monic, x);
            }
        }
        let Some(factors) = factor_univariate(den, x)? else {
            return Ok(None);
        };
        let Some(pieces) = pf_decompose(r, den, &factors, x)? else {
            return Ok(None);
        };

        integrate_pf_pieces(&pieces, x)
    }

    // ---------------------- جایگذاری u-sub ----------------------

    fn int_usub(&mut self, h: &Sym, x: &str, depth: u8) -> Result<Option<Sym>, CasError> {
        let mut cands: Vec<Sym> = vec![];
        collect_subexprs(h, x, &mut cands);
        // کاندید پیچیده‌تر اول — احتمال u-تمیزی بیشتر
        cands.sort_by_key(|g| std::cmp::Reverse(node_count(g)));
        cands.truncate(MAX_CANDIDATES);
        for g in cands {
            let dg = diff(&g, x)?;
            if matches!(dg, Sym::Integer(ref z) if z.is_zero()) {
                continue;
            }
            let Ok(q) = Sym::over(h.clone(), dg) else {
                continue;
            };
            // ساده‌سازی پیش از جایگذاری حیاتی است: h/g′ باید در RNF
            // عوامل مشترک را بکشد وگرنه x در مخرج می‌ماند و «u-تمیزی»
            // حتی برای کلاسیک‌ترین حالت (x·e^(−x²)) شکست می‌خورد.
            let Ok(q_simple) = simplify(&q) else {
                continue;
            };
            let mut taken = q_simple.free_symbols();
            taken.push(x.to_string());
            taken.extend(self.used.iter().cloned());
            let usym = fresh_sym("_u", &taken);
            if let Sym::Symbol(s) = &usym {
                let name = s.to_string();
                self.used.push(name.clone());
                let q2 = replace_sub(&q_simple, &g, &usym);
                if contains(&q2, x) {
                    continue;
                }
                if let Some(fu) = self.run(&q2, &name, depth + 1)? {
                    return Ok(Some(replace_sub(&fu, &usym, &g)));
                }
            }
        }
        Ok(None)
    }

    // ---------------------- جزء به جزء ----------------------

    fn int_by_parts(&mut self, h: &Sym, x: &str, depth: u8) -> Result<Option<Sym>, CasError> {
        let Sym::Mul(factors) = h else {
            return Ok(None);
        };
        let x_factors: Vec<&Sym> = factors.iter().filter(|f| contains(f, x)).collect();
        if x_factors.len() < 2 {
            return Ok(None);
        }
        // LIATE: log(0) > وارون‌ها(1) > جبری(2) > مثلثاتی(3) > نمایی(4)
        let class = |f: &Sym| -> u8 {
            if let Sym::Fn(name, _) = f {
                match &**name {
                    "log" | "log10" | "log2" => 0,
                    "atan" | "asin" | "acos" => 1,
                    "sin" | "cos" | "tan" | "sinh" | "cosh" | "tanh" => 3,
                    "exp" => 4,
                    _ => 2,
                }
            } else if let Sym::Pow(b, ex) = f {
                if contains(b, x) && as_small_int(ex).is_some() {
                    2
                } else {
                    4 // b^x
                }
            } else {
                2
            }
        };
        let pick = x_factors
            .iter()
            .enumerate()
            .min_by_key(|(i, f)| (class(f), std::cmp::Reverse(rat_deg_x(f, x)), *i))
            .map(|(i, _)| i);
        let Some(pick) = pick else {
            return Ok(None);
        };
        let u_sym = x_factors[pick].clone();
        let dv = fold_mul(
            x_factors
                .iter()
                .enumerate()
                .filter(|(i, _)| *i != pick)
                .map(|(_, f)| (*f).clone()),
        );
        let Some(v) = self.run(&dv, x, depth + 1)? else {
            return Ok(None);
        };
        let du = diff(&u_sym, x)?;
        if matches!(du, Sym::Integer(ref z) if z.is_zero()) {
            return Ok(None);
        }
        let rest = Sym::times(v.clone(), du);
        let Some(w) = self.run(&rest, x, depth + 1)? else {
            return Ok(None);
        };
        Ok(Some(Sym::minus(Sym::times(u_sym, v), w)))
    }
}

/// درجهٔ مؤثر به‌عنوان تابع گویا در x (برای اولویت LIATE).
fn rat_deg_x(f: &Sym, x: &str) -> i64 {
    match RatFun::from_sym(f) {
        Ok(rf) => rf.num.degree_in(&vkey(x)) - rf.den.degree_in(&vkey(x)),
        Err(_) => 0,
    }
}

// ---------------------------------------------------------------------------
// چندجمله‌ای در x با ضرایب پارامتری
// ---------------------------------------------------------------------------

/// ضریب درجهٔ k از x — چندجمله‌ای در بقیهٔ متغیرها.
fn coeff_of(p: &Poly, x: &str, k: i64) -> Poly {
    let vk = vkey(x);
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
    out
}

/// قاعدهٔ توان روی چندجمله‌ای با ضرایب دلخواه (توان x نامنفی).
fn int_poly_x(p: &Poly, x: &str) -> Result<Sym, CasError> {
    if p.is_zero() {
        return Ok(Sym::int(0));
    }
    let vk = vkey(x);
    let mut acc: Option<Sym> = None;
    for (m, c) in &p.terms {
        let k = m.get(&vk).copied().unwrap_or(0);
        if k < 0 {
            return Err(CasError::Unsupported(
                "negative powers belong to the denominator side of the rational form",
            ));
        }
        let mut params: Vec<Sym> = vec![Sym::Rational(c.clone())];
        for (g, e) in m {
            if g != &vk && *e != 0 {
                params.push(generator_sym(g)?);
            }
        }
        let factor = fold_mul(params.into_iter());
        let term = if k == 0 {
            Sym::times(factor, Sym::symbol(x))
        } else {
            let inv = BigRational::new(BigInt::from(1), BigInt::from(k + 1));
            let xpow = Sym::times(factor, Sym::pow(Sym::symbol(x), Sym::int(k + 1))?);
            Sym::times(xpow, Sym::Rational(inv))
        };
        acc = Some(match acc {
            None => term,
            Some(a) => Sym::plus(a, term),
        });
    }
    Ok(acc.unwrap_or_else(|| Sym::int(0)))
}

/// تقسیم طولانی num ÷ den در x، در میدان کسرهای پارامتری.
/// خروجی: (خارج‌قسمت Sym، باقیماندهٔ سرراست RatFun).
/// نامعادنی: r با مقیاس‌گذاری دقیق در حلقهٔ چندجمله‌ای‌ها پیش می‌رود —
/// هر گام: r ← (r·lead_d − den·x^shift·lead_r)/lead_d (کسری دقیق).
fn pdiv_x_fraction(num: &Poly, den: &Poly, x: &str) -> Result<(Sym, RatFun), CasError> {
    let dn = den.degree_in(&vkey(x));
    let vk = vkey(x);
    let mut r = RatFun {
        num: num.clone(),
        den: Poly::one(),
    };
    let mut q_terms: Vec<Sym> = vec![];
    let mut guard = 0;
    loop {
        if r.num.is_zero() || r.num.degree_in(&vkey(x)) < dn {
            break;
        }
        guard += 1;
        if guard > 64 {
            return Err(CasError::Unsupported("polynomial division budget exceeded"));
        }
        let rn = r.num.degree_in(&vkey(x));
        let lead_r = coeff_of(&r.num, x, rn); // x-آزاد
        let lead_d = coeff_of(den, x, dn); // x-آزاد و ناصفر
        let ratio = RatFun {
            num: lead_r.clone(),
            den: lead_d.clone(),
        };
        let shift = rn - dn;
        let xs = if shift == 0 {
            Sym::int(1)
        } else {
            Sym::pow(Sym::symbol(x), Sym::int(shift))?
        };
        q_terms.push(Sym::times(ratio.to_sym()?, xs));
        // r ← (r·lead_d − den·x^shift·lead_r) / lead_d
        let mut den_shift = Poly::one();
        if shift > 0 {
            let mut mon = Mon::new();
            mon.insert(vk.clone(), shift);
            den_shift = Poly {
                terms: [(mon, BigRational::from(BigInt::from(1)))]
                    .into_iter()
                    .collect(),
            };
        }
        let scaled = r.num.mul(&lead_d)?;
        let subtrahend = den.mul(&den_shift)?.mul(&lead_r)?;
        let new_num = Poly::add(&scaled, &Poly::neg(&subtrahend));
        let new_den = r.den.mul(&lead_d)?;
        r = RatFun {
            num: new_num,
            den: new_den,
        };
        r.cancel()?;
    }
    let q_sym = if q_terms.is_empty() {
        Sym::int(0)
    } else {
        fold_add(q_terms.into_iter())
    };
    // باقیماندهٔ «سرراست» = مقدارِ r (که چندجملهٔ باقیمانده است) تقسیم بر
    // مخرج اصلی — درس باگ اول: مخرج اصلی جا می‌افتاد و 1/x به 1 تبدیل می‌شد!
    let mut proper = RatFun {
        num: r.num,
        den: r.den.mul(den)?,
    };
    proper.cancel()?;
    Ok((q_sym, proper))
}

// ---------------------------------------------------------------------------
// تجزیهٔ مخرج عددی (RRT) + جزء به جزء
// ---------------------------------------------------------------------------

/// عامل مخرج یک‌متغیرهٔ عددی.
enum UFactor {
    /// (x − r) با ریشهٔ گویا
    Lin(BigRational),
    /// چندجمله‌ای درجه ۲ کاهش‌ناپذیر عددی (δ<0) — مسیر atan
    QuadIrred(Poly),
}

/// ریشهٔ گویای چندجمله‌ای یک‌متغیره با ضرایب گویا (RRT) — اولین یافته.
pub(crate) fn try_rational_root(p: &Poly, x: &str) -> Option<BigRational> {
    let vk = vkey(x);
    if p.generators().iter().any(|g| g != &vk) {
        return None; // فقط مسیر عددی
    }
    let deg = p.degree_in(&vk);
    if deg < 1 {
        return None;
    }
    let cst = coeff_of(p, x, 0).constant_value()?;
    if cst.is_zero() {
        return Some(BigRational::zero()); // ریشهٔ صفر
    }
    // مقیاس به ضرایب صحیح: p_int = L·p
    let mut lcm_den = BigInt::from(1);
    for c in p.terms.values() {
        let d = c.denom();
        lcm_lcm(&mut lcm_den, d);
    }
    let int_coeff = |k: i64| -> Option<BigInt> {
        let c = coeff_of(p, x, k).constant_value()?;
        let scaled = c * BigRational::from(lcm_den.clone());
        if !scaled.is_integer() {
            return None;
        }
        Some(scaled.to_integer())
    };
    let (Some(lead_i), Some(cst_i)) = (int_coeff(deg), int_coeff(0)) else {
        return None;
    };
    // بزرگ‌بودن اعداد → احتیاط صادقانه (انفجار مقسوم‌علیه‌ها)
    if lead_i.abs() > BigInt::from(10_000_000_000i64)
        || cst_i.abs() > BigInt::from(10_000_000_000i64)
    {
        return None;
    }
    let _vk = vkey(x);
    let mut cands: Vec<BigRational> = vec![];
    for pn in divisors(&cst_i) {
        for qd in divisors(&lead_i) {
            if pn.is_zero() {
                continue;
            }
            let r = BigRational::new(pn.clone(), qd.clone());
            cands.push(r.clone());
            cands.push(-r);
        }
    }
    if cands.len() > 512 {
        return None;
    }
    cands.sort();
    cands.dedup();
    cands
        .into_iter()
        .find(|r| eval_univariate(p, x, r).is_zero())
}

fn bigint_gcd(a: &BigInt, b: &BigInt) -> BigInt {
    let (mut a, mut b) = (a.abs(), b.abs());
    while !b.is_zero() {
        let r = &a % &b;
        a = b.clone();
        b = r;
    }
    a
}

fn lcm_lcm(acc: &mut BigInt, d: &BigInt) {
    let g = bigint_gcd(acc, d);
    if !g.is_zero() {
        let q = &*acc / &g;
        *acc = q * d;
    }
}

/// مقسوم‌علیه‌های مثبت — تجزیه با آزمون تقسیم (اعداد کلاس‌درس کوچک‌اند).
fn divisors(n: &BigInt) -> Vec<BigInt> {
    let mut n = n.abs();
    if n.is_zero() {
        return vec![BigInt::from(1)];
    }
    if n > BigInt::from(10_000_000_000_000i64) {
        return vec![BigInt::from(1)]; // پاس صادقانهٔ انفجار
    }
    let mut out: Vec<BigInt> = vec![BigInt::from(1)];
    let mut d = BigInt::from(2);
    while &d * &d <= n {
        while (&n % &d).is_zero() {
            n = &n / &d;
            let cur = out.clone();
            for v in cur {
                out.push(v * &d);
            }
        }
        d += 1;
    }
    if n > BigInt::from(1) {
        let cur = out.clone();
        for v in cur {
            out.push(v * &n);
        }
    }
    out
}

/// ارزیابی Horner دقیق یک‌متغیره (مسیر عددی — پارامتر ندارد).
fn eval_univariate(p: &Poly, x: &str, r: &BigRational) -> BigRational {
    let deg = p.degree_in(&vkey(x));
    let mut acc = BigRational::zero();
    for k in (0..=deg).rev() {
        let c = coeff_of(p, x, k)
            .constant_value()
            .unwrap_or_else(BigRational::zero);
        acc = acc * r + c;
    }
    acc
}

/// تقسیم دقیق بر (x − r) — خارج‌قسمت و باقیمانده (مسیر عددی).
pub(crate) fn div_lin(p: &Poly, r: &BigRational, x: &str) -> (Poly, BigRational) {
    let vk = vkey(x);
    let deg = p.degree_in(&vk);
    let mut coeffs: Vec<BigRational> = (0..=deg)
        .map(|k| {
            coeff_of(p, x, k)
                .constant_value()
                .unwrap_or_else(BigRational::zero)
        })
        .collect();
    // ضریب بیرونی (توان‌های x ممکن است مقیاس L داشته باشند — همان ضرایب)
    let lead = coeffs.pop().unwrap_or_else(BigRational::zero);
    let mut q_coeffs = vec![lead.clone()];
    let mut carry = lead;
    while let Some(c) = coeffs.pop() {
        carry = carry * r + c;
        q_coeffs.push(carry.clone());
    }
    let rem = q_coeffs.pop().unwrap_or_else(BigRational::zero);
    let mut q = Poly::zero();
    for (i, c) in q_coeffs.iter().rev().enumerate() {
        if !c.is_zero() {
            let mut mon = Mon::new();
            mon.insert(vk.clone(), i as i64);
            q.terms.insert(mon, c.clone());
        }
    }
    (q, rem)
}

/// تجزیهٔ مخرج عددی: ریشه‌های گویا (با تکرار) + دنبالهٔ درجه ≤ ۲.
fn factor_univariate(den: &Poly, x: &str) -> Result<Option<Vec<(UFactor, i64)>>, CasError> {
    let mut factors: Vec<(UFactor, i64)> = vec![];
    let mut h = den.clone();
    let mut guard = 0;
    while h.degree_in(&vkey(x)) >= 1 {
        guard += 1;
        if guard > 32 {
            return Ok(None);
        }
        let Some(r) = try_rational_root(&h, x) else {
            break;
        };
        let (q, rem) = div_lin(&h, &r, x);
        if !rem.is_zero() {
            break; // ناسازگار — ریشهٔ کاذب
        }
        // شمارش تکرار
        let mut mult = 1;
        let mut cur = q;
        while !cur.is_zero() {
            let (q2, rem2) = div_lin(&cur, &r, x);
            if rem2.is_zero() {
                mult += 1;
                cur = q2;
            } else {
                break;
            }
        }
        factors.push((UFactor::Lin(r), mult));
        h = cur;
    }
    match h.degree_in(&vkey(x)) {
        0 => Ok(Some(factors)),
        2 => {
            let a2 = coeff_of(&h, x, 2)
                .constant_value()
                .unwrap_or_else(BigRational::zero);
            let a1 = coeff_of(&h, x, 1)
                .constant_value()
                .unwrap_or_else(BigRational::zero);
            let a0 = coeff_of(&h, x, 0)
                .constant_value()
                .unwrap_or_else(BigRational::zero);
            let delta = &a1 * &a1 - BigRational::from_integer(BigInt::from(4)) * &a2 * &a0;
            if delta < BigRational::zero() {
                factors.push((UFactor::QuadIrred(h), 1));
                Ok(Some(factors))
            } else {
                // δ=0 یعنی ریشهٔ مضاعف گویا — RRT باید می‌گرفت؛ δ>0 غیر
                // مربع کامل یعنی ریشه‌های رادیکال حقیقی که PF در ℚ[x]
                // نمی‌گیرد — هر دو صادقانه رد
                Ok(None)
            }
        }
        _ => Ok(None), // تجزیه‌ناپذیر درجه ≥ ۳ — صادقانه رد
    }
}

/// √r فقط اگر مربع کامل گویا باشد.
fn sqrt_rat(r: &BigRational) -> Option<BigRational> {
    let is_sq = |n: &BigInt| -> Option<BigInt> {
        if n.is_negative() {
            return None;
        }
        // جذر صحیح با نیوتن
        let mut x = n.clone();
        if x.is_zero() {
            return Some(x);
        }
        let two = BigInt::from(2);
        let mut y = (&x + &two) / &two;
        while y < x {
            x = y.clone();
            y = (&x + n / &x) / &two;
        }
        if &x * &x == *n {
            Some(x)
        } else {
            None
        }
    };
    match (is_sq(r.numer()), is_sq(r.denom())) {
        (Some(a), Some(b)) => Some(BigRational::new(a, b)),
        _ => None,
    }
}

/// √r به‌صورت Sym: مربع کامل → گویا؛ وگرنه گرهٔ رادیکال (r > 0).
pub(crate) fn sqrt_sym(r: &BigRational) -> Result<Sym, CasError> {
    match sqrt_rat(r) {
        Some(s) => Ok(Sym::Rational(s)),
        None => Sym::pow(Sym::Rational(r.clone()), Sym::rat(1, 2)?),
    }
}

fn is_sym_zero(s: &Sym) -> bool {
    match s {
        Sym::Integer(i) => i.is_zero(),
        Sym::Rational(r) => r.is_zero(),
        _ => false,
    }
}

/// قطعهٔ PF پس از حل دستگاه.
enum PfTerm {
    LinPow {
        root: Sym,
        coeff: Sym,
        j: i64,
    },
    QuadPart {
        a2: Sym,
        a1: Sym,
        a0: Sym,
        alpha: Sym,
        beta: Sym,
    },
}

/// PF با ضرایب نامعین روی عوامل تجزیه‌شده — دستگاه خطی + گاوسی دقیق.
fn pf_decompose(
    r: &Poly,
    den: &Poly,
    factors: &[(UFactor, i64)],
    x: &str,
) -> Result<Option<Vec<PfTerm>>, CasError> {
    let _ = den;
    // چندجمله‌ای هر عامل متمایز (LinRad در ℚ[x] نمی‌ماند → رد صادقانه)
    let mut factor_polys: Vec<Poly> = vec![];
    for (f, _) in factors {
        match f {
            UFactor::Lin(root) => {
                // x − root
                let mut p = Poly::zero();
                p.terms
                    .insert(unit_mon(x), BigRational::from_integer(BigInt::from(1)));
                if !root.is_zero() {
                    p.terms.insert(Mon::new(), -root.clone());
                }
                factor_polys.push(p);
            }
            UFactor::QuadIrred(q) => factor_polys.push(q.clone()),
        }
    }
    // پایه‌های PF: برای عامل خطی i با تکرار m و توان j:
    // g_{i,j} = f_i^(m−j) · Π_{k≠i} f_k^m_k  (در ℚ[x])
    let mut basis: Vec<Poly> = vec![];
    let mut meta: Vec<(usize, i64)> = vec![];
    for (i, (f, mult)) in factors.iter().enumerate() {
        if !matches!(f, UFactor::Lin(_)) {
            continue; // قطعهٔ درجه ۲ با جفت مجهول (B, C) پایین‌تر
        }
        for j in 1..=*mult {
            let mut g = Poly::one();
            g = g.mul(&factor_polys[i].pow(mult - j)?)?;
            for (k, fp) in factor_polys.iter().enumerate() {
                if k != i {
                    g = g.mul(&fp.pow(factors[k].1)?)?;
                }
            }
            basis.push(g);
            meta.push((i, j));
        }
    }
    // قطعهٔ درجه ۲ کاهش‌ناپذیر: مجهول‌های B (برای x·g) و C (برای g)
    if let Some(pos) = factors
        .iter()
        .position(|(f, _)| matches!(f, UFactor::QuadIrred(_)))
    {
        let mut g = Poly::one();
        for (k, fp) in factor_polys.iter().enumerate() {
            if k != pos {
                g = g.mul(&fp.pow(factors[k].1)?)?;
            }
        }
        let gx = g.mul(&Poly {
            terms: [(unit_mon(x), BigRational::from_integer(BigInt::from(1)))]
                .into_iter()
                .collect(),
        })?;
        basis.push(gx);
        basis.push(g);
    }
    if basis.is_empty() {
        return Ok(None);
    }
    // دستگاه: برای هر درجهٔ d از x: Σ u_i·[x^d]g_i = [x^d]r — تا
    // بیشینهٔ درجهٔ پایه‌ها (نه فقط درجهٔ صورت؛ وگرنه قطعهٔ درجه ۲
    // همیشه کمبود معادله می‌دید و PF بی‌دلیل رد می‌شد)
    let vk = vkey(x);
    let dmax = basis.iter().map(|g| g.degree_in(&vk)).max().unwrap_or(0);
    let n_unk = basis.len();
    let mut rows: Vec<Vec<Sym>> = vec![];
    let mut rhs: Vec<Sym> = vec![];
    for d in 0..=dmax {
        let row: Vec<Sym> = basis
            .iter()
            .map(|g| sym_from_poly(&coeff_of(g, x, d)))
            .collect();
        if row.iter().any(|c| !is_sym_zero(c)) {
            rows.push(row);
            rhs.push(sym_from_poly(&coeff_of(r, x, d)));
        }
    }
    if rows.len() < n_unk {
        return Ok(None); // کمبود معادله — ساختار غیرمنتظر
    }
    let Some(sol) = solve::gaussian_solve_symbolic(&rows, &rhs)? else {
        return Ok(None);
    };
    // ساخت قطعات
    let mut out: Vec<PfTerm> = vec![];
    for (ui, (fi, j)) in meta.iter().enumerate() {
        let UFactor::Lin(root) = &factors[*fi].0 else {
            return Ok(None);
        };
        out.push(PfTerm::LinPow {
            root: Sym::Rational(root.clone()),
            coeff: sol[ui].clone(),
            j: *j,
        });
    }
    if let Some(pos) = factors
        .iter()
        .position(|(f, _)| matches!(f, UFactor::QuadIrred(_)))
    {
        if let UFactor::QuadIrred(q) = &factors[pos].0 {
            let a2 = sym_from_poly(&coeff_of(q, x, 2));
            let a1 = sym_from_poly(&coeff_of(q, x, 1));
            let a0 = sym_from_poly(&coeff_of(q, x, 0));
            let b_sym = sol[meta.len()].clone();
            let c_sym = sol[meta.len() + 1].clone();
            // بازسازی α و β از B·x + C
            let two_a2 = Sym::plus(a2.clone(), a2.clone());
            let alpha = Sym::over(b_sym, two_a2.clone())?;
            let beta = Sym::minus(c_sym, Sym::times(alpha.clone(), a1.clone()));
            out.push(PfTerm::QuadPart {
                a2,
                a1,
                a0,
                alpha,
                beta,
            });
        }
    }
    Ok(Some(out))
}

fn unit_mon(x: &str) -> Mon {
    let mut m = Mon::new();
    m.insert(vkey(x), 1);
    m
}

/// بازسازی درختِ یک ژنراتور نام‌گذاری‌شده با sexpr.
fn generator_sym(sexp: &str) -> Result<Sym, CasError> {
    crate::sexpr::from_sexp(sexp).map_err(|_| CasError::Serde("bad generator sexpr in polynomial"))
}

fn sym_from_poly(p: &Poly) -> Sym {
    if let Some(c) = p.constant_value() {
        Sym::Rational(c)
    } else {
        RatFun {
            num: p.clone(),
            den: Poly::one(),
        }
        .to_sym()
        .unwrap_or_else(|_| Sym::int(0))
    }
}

/// انتگرال قطعات PF.
fn integrate_pf_pieces(pieces: &[PfTerm], x: &str) -> Result<Option<Sym>, CasError> {
    let mut acc: Option<Sym> = None;
    for p in pieces {
        let term = match p {
            PfTerm::LinPow { root, coeff, j } => {
                let lin = Sym::minus(Sym::symbol(x), root.clone());
                if *j == 1 {
                    Sym::times(coeff.clone(), Sym::fn_of("log", vec![abs_of(&lin)?])?)
                } else {
                    let new_exp = Sym::int(1 - j);
                    let inv = Sym::Rational(BigRational::new(BigInt::from(1), BigInt::from(1 - j)));
                    Sym::times(coeff.clone(), Sym::times(Sym::pow(lin, new_exp)?, inv))
                }
            }
            PfTerm::QuadPart {
                a2,
                a1,
                a0,
                alpha,
                beta,
            } => {
                let q = fold_add(
                    [
                        Sym::times(a2.clone(), Sym::pow(Sym::symbol(x), Sym::int(2))?),
                        Sym::times(a1.clone(), Sym::symbol(x)),
                        a0.clone(),
                    ]
                    .into_iter(),
                );
                let log_part = Sym::times(alpha.clone(), Sym::fn_of("log", vec![abs_of(&q)?])?);
                // β·∫dx/q — با aهای عددی
                let a2r = RatFun::from_sym(a2).ok().and_then(|rf| rf.to_poly().ok());
                let a1r = RatFun::from_sym(a1).ok().and_then(|rf| rf.to_poly().ok());
                let a0r = RatFun::from_sym(a0).ok().and_then(|rf| rf.to_poly().ok());
                let (Some(a2r), Some(a1r), Some(a0r)) = (a2r, a1r, a0r) else {
                    return Ok(None);
                };
                let Some(dxq) = int_dx_over_quad(&a2r, &a1r, &a0r, x)? else {
                    return Ok(None);
                };
                Sym::plus(log_part, Sym::times(beta.clone(), dxq))
            }
        };
        acc = Some(match acc {
            None => term,
            Some(a) => Sym::plus(a, term),
        });
    }
    Ok(acc)
}

/// ∫dx/(a2·x² + a1·x + a0) — تشخیص مسیر با علامت D = 4a2a0 − a1².
fn int_dx_over_quad(a2: &Poly, a1: &Poly, a0: &Poly, x: &str) -> Result<Option<Sym>, CasError> {
    let four = Poly::constant(BigRational::from_integer(BigInt::from(4)));
    let d_poly = Poly::add(&four.mul(a2)?.mul(a0)?, &Poly::neg(&a1.mul(a1)?));
    let two_a2 = Poly::add(a2, a2);
    let lin = Sym::plus(
        Sym::times(sym_from_poly(&two_a2), Sym::symbol(x)),
        sym_from_poly(a1),
    );
    if let Some(d) = d_poly.constant_value() {
        if d > BigRational::zero() {
            // 2/√D·atan((2a2·x + a1)/√D)
            let s = sqrt_sym(&d)?;
            let ratio = Sym::over(lin, s.clone())?;
            let atan = Sym::fn_of("atan", vec![ratio])?;
            let coef = Sym::over(Sym::int(2), s)?;
            Ok(Some(Sym::times(coef, atan)))
        } else if d == BigRational::zero() {
            // ریشهٔ مضاعف: −2/(2a2·x + a1)
            Ok(Some(Sym::over(Sym::int(-2), lin)?))
        } else {
            // D < 0 → دو ریشهٔ حقیقی: (log|x−r1| − log|x−r2|)/√(−D)
            let s = sqrt_sym(&(-d))?;
            let two_a2s = sym_from_poly(&two_a2);
            let neg_a1 = sym_from_poly(&Poly::neg(a1));
            let mk = |sgn: i64| -> Result<Sym, CasError> {
                let num = if sgn == 1 {
                    Sym::plus(neg_a1.clone(), s.clone())
                } else {
                    Sym::minus(neg_a1.clone(), s.clone())
                };
                Sym::over(num, two_a2s.clone())
            };
            let (r1, r2) = (mk(1)?, mk(-1)?);
            let l1 = Sym::fn_of("log", vec![abs_of(&Sym::minus(Sym::symbol(x), r1))?])?;
            let l2 = Sym::fn_of("log", vec![abs_of(&Sym::minus(Sym::symbol(x), r2))?])?;
            let diff = Sym::minus(l1, l2);
            Ok(Some(Sym::over(diff, s)?))
        }
    } else {
        // D نمادین — فقط الگوی مربع/مربعِ منفی تشخیص داده می‌شود
        let ds = sym_from_poly(&d_poly);
        if let Some(u) = as_square(&ds) {
            let ratio = Sym::over(lin, u.clone())?;
            let atan = Sym::fn_of("atan", vec![ratio])?;
            let coef = Sym::over(Sym::int(2), u)?;
            Ok(Some(Sym::times(coef, atan)))
        } else if let Some(u) = as_neg_square(&ds) {
            let two_a2s = sym_from_poly(&two_a2);
            let neg_a1 = sym_from_poly(&Poly::neg(a1));
            let mk = |sgn: i64| -> Result<Sym, CasError> {
                let num = if sgn == 1 {
                    Sym::plus(neg_a1.clone(), u.clone())
                } else {
                    Sym::minus(neg_a1.clone(), u.clone())
                };
                Sym::over(num, two_a2s.clone())
            };
            let (r1, r2) = (mk(1)?, mk(-1)?);
            let l1 = Sym::fn_of("log", vec![abs_of(&Sym::minus(Sym::symbol(x), r1))?])?;
            let l2 = Sym::fn_of("log", vec![abs_of(&Sym::minus(Sym::symbol(x), r2))?])?;
            Ok(Some(Sym::over(Sym::minus(l1, l2), u)?))
        } else {
            Ok(None) // علامت دیسکریمینان نامشخص — صادقانه رد
        }
    }
}

/// آیا e مربع کامل است؟ (u² یا c·u² با c مربع مثبت یا گویای مثبت مربع)
fn as_square(e: &Sym) -> Option<Sym> {
    match e {
        Sym::Pow(b, ex) if as_small_int(ex) == Some(2) => Some(b.as_ref().clone()),
        Sym::Rational(r) if *r > BigRational::zero() => sqrt_rat(r).map(Sym::Rational),
        Sym::Mul(factors) => {
            let mut numeric: Option<BigRational> = None;
            let mut square: Option<Sym> = None;
            let mut rest = 0usize;
            for f in factors {
                match f {
                    Sym::Integer(i) => {
                        let one = BigRational::from_integer(BigInt::from(1));
                        let c = numeric.unwrap_or(one.clone());
                        numeric = Some(c * BigRational::from_integer(i.clone()));
                    }
                    Sym::Rational(r) => {
                        let one = BigRational::from_integer(BigInt::from(1));
                        let c = numeric.unwrap_or(one.clone());
                        numeric = Some(c * r.clone());
                    }
                    Sym::Pow(b, ex) if as_small_int(ex) == Some(2) && square.is_none() => {
                        square = Some(b.as_ref().clone());
                    }
                    _ => rest += 1,
                }
            }
            if rest == 0 && square.is_some() {
                let c = numeric.unwrap_or_else(|| BigRational::from_integer(BigInt::from(1)));
                if c > BigRational::zero() {
                    let s = sqrt_rat(&c)?;
                    return Some(Sym::times(Sym::Rational(s), square?));
                }
            }
            None
        }
        _ => None,
    }
}

/// آیا e قرینهٔ مربع است؟ (−u²)
fn as_neg_square(e: &Sym) -> Option<Sym> {
    let neg_one = BigInt::from(-1);
    match e {
        Sym::Mul(factors) => {
            if !factors
                .iter()
                .any(|f| matches!(f, Sym::Integer(i) if *i == neg_one))
            {
                return None;
            }
            let rest: Vec<Sym> = factors
                .iter()
                .filter(|f| !matches!(f, Sym::Integer(i) if *i == neg_one))
                .cloned()
                .collect();
            as_square(&fold_mul(rest.into_iter()))
        }
        Sym::Rational(r) if *r < BigRational::zero() => sqrt_rat(&(-r.clone())).map(Sym::Rational),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// جایگذاری — جمع‌آوری کاندیدها
// ---------------------------------------------------------------------------

fn collect_subexprs(e: &Sym, x: &str, out: &mut Vec<Sym>) {
    fn push_cand(n: &Sym, x: &str, out: &mut Vec<Sym>) {
        if contains(n, x) && !matches!(n, Sym::Symbol(_)) && !out.contains(n) {
            out.push(n.clone());
        }
    }
    match e {
        Sym::Add(terms) => {
            for t in terms {
                collect_subexprs(t, x, out);
                push_cand(t, x, out);
            }
        }
        Sym::Mul(factors) => {
            for f in factors {
                collect_subexprs(f, x, out);
                push_cand(f, x, out);
            }
        }
        Sym::Pow(b, ex) => {
            collect_subexprs(b, x, out);
            collect_subexprs(ex, x, out);
            push_cand(b, x, out);
            push_cand(e, x, out);
        }
        Sym::Fn(_, args) => {
            for a in args {
                collect_subexprs(a, x, out);
                push_cand(a, x, out);
            }
            push_cand(e, x, out);
        }
        _ => {}
    }
}

// ---------------------------------------------------------------------------
// جدول فرم‌های استاندارد
// ---------------------------------------------------------------------------

/// خطی در x: t = a·x + b (a x-آزاد و ناصفر، b x-آزاد).
pub(crate) fn as_linear(t: &Sym, x: &str) -> Option<(Sym, Sym)> {
    let rf = RatFun::from_sym(t).ok()?;
    if rf.den.degree_in(&vkey(x)) != 0 {
        return None;
    }
    let n = rf.num.degree_in(&vkey(x));
    if n > 1 {
        return None;
    }
    let a = RatFun {
        num: coeff_of(&rf.num, x, 1),
        den: rf.den.clone(),
    }
    .to_sym()
    .ok()?;
    let b = RatFun {
        num: coeff_of(&rf.num, x, 0),
        den: rf.den,
    }
    .to_sym()
    .ok()?;
    let a_zero = matches!(&a, Sym::Integer(z) if z.is_zero())
        || matches!(&a, Sym::Rational(z) if z.is_zero());
    if a_zero {
        return None;
    }
    Some((a, b))
}

/// جدول فرم‌های استاندارد — فقط آرگومانِ «خودِ متغیر» یا الگوهای
/// ساختاری ویژه (توان گویا، حاصل‌ضرب‌های مثلثاتی/نمایی).
fn table_form(h: &Sym, x: &str) -> Option<Sym> {
    let is_var = |u: &Sym| matches!(u, Sym::Symbol(s) if &**s == x);
    if let Sym::Fn(name, args) = h {
        if args.len() == 1 && is_var(&args[0]) {
            let u = args[0].clone();
            let built = match &**name {
                "exp" => u,
                "sin" => Sym::negated(Sym::fn_of("cos", vec![u.clone()]).ok()?),
                "cos" => Sym::fn_of("sin", vec![u.clone()]).ok()?,
                "tan" => Sym::negated(
                    Sym::fn_of(
                        "log",
                        vec![abs_of(&Sym::fn_of("cos", vec![u.clone()]).ok()?).ok()?],
                    )
                    .ok()?,
                ),
                "tanh" => {
                    Sym::fn_of("log", vec![Sym::fn_of("cosh", vec![u.clone()]).ok()?]).ok()?
                }
                "sinh" => Sym::fn_of("cosh", vec![u.clone()]).ok()?,
                "cosh" => Sym::fn_of("sinh", vec![u.clone()]).ok()?,
                "log" => Sym::minus(
                    Sym::times(u.clone(), Sym::fn_of("log", vec![u.clone()]).ok()?),
                    u.clone(),
                ),
                "log10" | "log2" => {
                    let base = Sym::fn_of(
                        "log",
                        vec![Sym::int(if &**name == "log10" { 10 } else { 2 })],
                    )
                    .ok()?;
                    let inner = Sym::minus(
                        Sym::times(u.clone(), Sym::fn_of("log", vec![u.clone()]).ok()?),
                        u.clone(),
                    );
                    Sym::over(inner, base).ok()?
                }
                "atan" => Sym::minus(
                    Sym::times(u.clone(), Sym::fn_of("atan", vec![u.clone()]).ok()?),
                    Sym::over(
                        Sym::fn_of(
                            "log",
                            vec![Sym::plus(
                                Sym::int(1),
                                Sym::pow(u.clone(), Sym::int(2)).ok()?,
                            )],
                        )
                        .ok()?,
                        Sym::int(2),
                    )
                    .ok()?,
                ),
                "asin" => Sym::plus(
                    Sym::times(u.clone(), Sym::fn_of("asin", vec![u.clone()]).ok()?),
                    Sym::fn_of(
                        "sqrt",
                        vec![Sym::minus(
                            Sym::int(1),
                            Sym::pow(u.clone(), Sym::int(2)).ok()?,
                        )],
                    )
                    .ok()?,
                ),
                "acos" => Sym::minus(
                    Sym::times(u.clone(), Sym::fn_of("acos", vec![u.clone()]).ok()?),
                    Sym::fn_of(
                        "sqrt",
                        vec![Sym::minus(
                            Sym::int(1),
                            Sym::pow(u.clone(), Sym::int(2)).ok()?,
                        )],
                    )
                    .ok()?,
                ),
                _ => return None,
            };
            return Some(built);
        }
        return None;
    }
    // توان گویا از خودِ متغیر: ∫x^r = x^(r+1)/(r+1) (r ≠ −1)
    if let Sym::Pow(b, ex) = h {
        if is_var(b) {
            if let Sym::Rational(r) = ex.as_ref() {
                let one = BigRational::from_integer(BigInt::from(1));
                let rp1 = r + one;
                if !rp1.is_zero() {
                    let new_exp = Sym::Rational(rp1.clone());
                    let inv = Sym::Rational(BigRational::from_integer(BigInt::from(1)) / rp1);
                    return Some(Sym::times(Sym::pow(b.as_ref().clone(), new_exp).ok()?, inv));
                }
            }
        }
        // b^x با b x-آزاد: b^x/log(b)
        if is_var(ex) && !contains(b, x) {
            let lb = Sym::fn_of("log", vec![b.as_ref().clone()]).ok()?;
            return Sym::over(h.clone(), lb).ok();
        }
        // sin²/cos² خودِ متغیر
        if let Sym::Fn(f, fargs) = b.as_ref() {
            if fargs.len() == 1 && is_var(&fargs[0]) && as_small_int(ex) == Some(2) {
                let u = fargs[0].clone();
                let two_u = Sym::times(Sym::int(2), u.clone());
                let sin2 = Sym::fn_of("sin", vec![two_u.clone()]).ok()?;
                let half = Sym::Rational(BigRational::new(BigInt::from(1), BigInt::from(2)));
                let quarter = Sym::Rational(BigRational::new(BigInt::from(1), BigInt::from(4)));
                return match &**f {
                    "sin" => Some(Sym::minus(Sym::times(half, u), Sym::times(quarter, sin2))),
                    "cos" => Some(Sym::plus(Sym::times(half, u), Sym::times(quarter, sin2))),
                    // اتحادهای هذلولی (درس مرجع ۸۶/۸۷): پادمشتق sinh² =
                    // sinh(2u)/4 − u/2 و cosh² = sinh(2u)/4 + u/2
                    // (cosh 2u = 1 + 2·sinh²u → ∫sinh² = ∫(cosh 2u − 1)/2)
                    "sinh" => {
                        let sinh2 = Sym::fn_of("sinh", vec![two_u]).ok()?;
                        Some(Sym::minus(Sym::times(quarter, sinh2), Sym::times(half, u)))
                    }
                    "cosh" => {
                        let sinh2 = Sym::fn_of("sinh", vec![two_u]).ok()?;
                        Some(Sym::plus(Sym::times(quarter, sinh2), Sym::times(half, u)))
                    }
                    _ => None,
                };
            }
        }
        return None;
    }
    // حاصل‌ضرب‌ها — مثلثاتی با فرکانس‌های مختلف، exp·sin/cos
    if let Sym::Mul(factors) = h {
        return table_product(factors, x);
    }
    None
}

/// حاصل‌ضرب‌های دو تابعی: sin·cos / sin·sin / cos·cos (تبدیل به جمع)
/// و exp·sin / exp·cos (فرم چرخشی).
fn table_product(factors: &[Sym], x: &str) -> Option<Sym> {
    if factors.len() != 2 {
        return None;
    }
    let (f1, f2) = (&factors[0], &factors[1]);
    let fn1 = (fname(f1), farg(f1));
    let fn2 = (fname(f2), farg(f2));
    let (Some(n1), a1) = fn1 else { return None };
    let (Some(n2), a2) = fn2 else { return None };
    let (l1, l2) = (as_linear(&a1, x)?, as_linear(&a2, x)?);
    // exp·(sin|cos) — فرکانس‌های عددی
    if (n1 == "exp" && (n2 == "sin" || n2 == "cos"))
        || (n2 == "exp" && (n1 == "sin" || n1 == "cos"))
    {
        let (ea, trig) = if n1 == "exp" {
            (a1.clone(), (n2, a2.clone()))
        } else {
            (a2.clone(), (n1, a1.clone()))
        };
        let (ta, tb) = (as_linear(&ea, x)?, as_linear(&trig.1, x)?);
        let (alpha, _b1) = ta;
        let (beta, _b2) = tb;
        // ضرایب باید عددی باشند (گارد مخرجِ a²+b² = 0)
        let to_rat = |s: &Sym| -> Option<BigRational> {
            match s {
                Sym::Integer(i) => Some(BigRational::from_integer(i.clone())),
                Sym::Rational(r) => Some(r.clone()),
                _ => None,
            }
        };
        let (Some(ar), Some(br)) = (to_rat(&alpha), to_rat(&beta)) else {
            return None;
        };
        let denom = &ar * &ar + &br * &br;
        if denom.is_zero() {
            return None;
        }
        let exp_term = Sym::fn_of("exp", vec![ea.clone()]).ok()?;
        let s = Sym::fn_of("sin", vec![trig.1.clone()]).ok()?;
        let c = Sym::fn_of("cos", vec![trig.1.clone()]).ok()?;
        let num = if trig.0 == "sin" {
            Sym::minus(Sym::times(alpha.clone(), s), Sym::times(beta.clone(), c))
        } else {
            Sym::plus(Sym::times(alpha.clone(), c), Sym::times(beta.clone(), s))
        };
        let div = Sym::over(num, Sym::Rational(denom)).ok()?;
        return Some(Sym::times(exp_term, div));
    }
    // تبدیل حاصل‌ضرب به جمع — هر دو مثلثاتی
    let trig = |n: &str| matches!(n, "sin" | "cos");
    if !(trig(n1) && trig(n2)) {
        return None;
    }
    let (a1s, b1s) = l1;
    let (a2s, b2s) = l2;
    // درس علامت: ضرب کانونیِ Mul ترتیب عامل‌ها را عوض می‌کند؛ در
    // sin·cos باید «شیب سینوس − شیب کسینوس» باشد (پاد対‌متقارن).
    let (s_sin, s_cos, b_sin, b_cos) = if n1 == "sin" {
        (a1s.clone(), a2s.clone(), b1s.clone(), b2s.clone())
    } else {
        (a2s.clone(), a1s.clone(), b2s.clone(), b1s.clone())
    };
    let sum = Sym::plus(a1s.clone(), a2s.clone());
    let diff = Sym::minus(s_sin, s_cos);
    let sum_l = Sym::plus(
        Sym::times(sum.clone(), Sym::symbol(x)),
        Sym::plus(b1s.clone(), b2s.clone()),
    );
    let diff_l = Sym::plus(
        Sym::times(diff.clone(), Sym::symbol(x)),
        Sym::minus(b_sin, b_cos),
    );
    let half = Sym::Rational(BigRational::new(BigInt::from(1), BigInt::from(2)));
    let mk = |name: &str, arg: &Sym| -> Option<Sym> { Sym::fn_of(name, vec![arg.clone()]).ok() };
    // انتگرال هر جمله: ±cos/سین تقسیم بر شیب
    let int_trig = |name: &str, arg: &Sym, slope: &Sym| -> Option<Sym> {
        // ∫sin(A)dx = −cos(A)/a ؛ ∫cos(A)dx = sin(A)/a
        let inner = match name {
            "sin" => Sym::negated(mk("cos", arg)?),
            _ => mk("sin", arg)?,
        };
        Sym::over(inner, slope.clone()).ok()
    };
    let s_sum = sum;
    let s_diff = diff;
    let (i1, i2) = match (n1, n2) {
        ("sin", "cos") | ("cos", "sin") => (
            int_trig("sin", &diff_l, &s_diff)?,
            int_trig("sin", &sum_l, &s_sum)?,
        ),
        ("sin", "sin") => (
            int_trig("cos", &diff_l, &s_diff)?,
            int_trig("cos", &sum_l, &s_sum)?,
        ),
        ("cos", "cos") => (
            int_trig("cos", &sum_l, &s_sum)?,
            int_trig("cos", &diff_l, &s_diff)?,
        ),
        _ => return None,
    };
    // sin·sin و cos·cos ضریب ½ دارند؛ sin·cos هم ½
    Some(Sym::times(half, Sym::plus(i1, i2)))
}

fn fname(f: &Sym) -> Option<&str> {
    match f {
        Sym::Fn(n, args) if args.len() == 1 => Some(n),
        _ => None,
    }
}

fn farg(f: &Sym) -> Sym {
    match f {
        Sym::Fn(_, args) => args[0].clone(),
        _ => Sym::int(0),
    }
}

// ---------------------------------------------------------------------------
// انتگرال معین
// ---------------------------------------------------------------------------

/// انتگرال معین با ارزیابی دقیق + پویش پیوستگی + cross-check عددی.
pub fn integrate_definite(f: &Sym, x: &str, a: &Sym, b: &Sym) -> Result<Sym, CasError> {
    let f_ant = integrate(f, x)?.ok_or(CasError::Unsupported(
        "no symbolic antiderivative found — for a numeric answer use integral()",
    ))?;
    let fb = f_ant.substitute(x, b);
    let fa = f_ant.substitute(x, a);
    let d = simplify(&Sym::minus(fb, fa))?;
    // بررسی‌های عددی فقط وقتی همه چیز عددی است
    let f_numeric = f.free_symbols().iter().all(|s| s == x);
    if f_numeric && a.to_f64().is_ok() && b.to_f64().is_ok() {
        let (av, bv) = (a.to_f64()?, b.to_f64()?);
        if av == bv {
            return Ok(Sym::int(0));
        }
        let (lo, hi, sign) = if av < bv {
            (av, bv, 1.0)
        } else {
            (bv, av, -1.0)
        };
        continuity_scan(f, x, lo, hi)?;
        if let Ok(dv) = d.to_f64() {
            if !dv.is_finite() {
                return Err(CasError::Unsupported(
                    "antiderivative is not finite at the interval endpoints (improper integral)",
                ));
            }
            let nv = simpson_composite(f, x, lo, hi)?;
            let scale = 1.0 + dv.abs() + nv.abs();
            if (nv * sign - dv).abs() > 1e-6 * scale {
                return Err(CasError::Unsupported(
                    "numeric cross-check mismatch — antiderivative may be discontinuous in the interval",
                ));
            }
        }
    }
    Ok(d)
}

/// پویش ۵۱۳ نقطه‌ای: تعریف‌نشدگی داخل بازه → خطای صادقانه.
fn continuity_scan(f: &Sym, x: &str, lo: f64, hi: f64) -> Result<(), CasError> {
    let n = 512;
    let step = (hi - lo) / n as f64;
    for i in 0..=n {
        let t = lo + step * i as f64;
        let v = f.substitute(x, &Sym::Real(t)).to_f64();
        match v {
            Ok(v) if v.is_finite() => {}
            _ => {
                return Err(CasError::Unsupported(
                    "the integrand is undefined at some point inside the interval",
                ))
            }
        }
    }
    Ok(())
}

/// سیمپسون مرکب با n=۲۰۴۸ (قطعی، بدون بازگشت) — NaN → خطا.
fn simpson_composite(f: &Sym, x: &str, lo: f64, hi: f64) -> Result<f64, CasError> {
    let n = 2048u64;
    let h = (hi - lo) / n as f64;
    let ev = |t: f64| -> Result<f64, CasError> {
        let v = f.substitute(x, &Sym::Real(t)).to_f64()?;
        if !v.is_finite() {
            return Err(CasError::Unsupported(
                "the integrand is undefined at some point inside the interval",
            ));
        }
        Ok(v)
    };
    let mut acc = ev(lo)? + ev(hi)?;
    for i in 1..n {
        let t = lo + h * i as f64;
        acc += if i % 2 == 1 { 4.0 } else { 2.0 } * ev(t)?;
    }
    Ok(acc * h / 3.0)
}

// ---------------------------------------------------------------------------
// تست‌ها
// ---------------------------------------------------------------------------
