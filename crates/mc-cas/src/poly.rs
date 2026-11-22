//! فرم نرمال چندجمله‌ای/گویای تعمیم‌یافته (گام 4.2/CAS-1 — سند ۰۵ §۳ سطح ۲).
//!
//! نمایش sparse چندمتغیره: هر «تک‌جمله‌ای» = نگاشت (ژنراتور → توان صحیح).
//! ژنراتورها همهٔ زیردرخت‌های غیرچندجمله‌ای‌اند (نماد، sin(u)، x^(1/2)،
//! …) و هرگز بازنویسی نمی‌شوند — پس گسترش هرگز دامنه را عوض نمی‌کند
//! (sqrt(x)² به x جمع نمی‌شود: برای x<0 نادرست است — سند ۰۵ §۳ گارد).
//!
//! GCD چندمتغیره: PRS بدوی (pseudo-remainder + تفکیک content در هر گام؛
//! بازگشتی روی متغیرها با لم گاوس: gcd(c, p) = gcd(c, content(p))).
//! همهٔ حساب ℚ دقیق (BigRational) — رشد ضرایب کنترل‌شده با content،
//! خطای صادقانه روی بودجه (نه هنگ بی‌پایان).
//!
//! محدودیت صادقانهٔ مستند: توان‌های هم‌پایه با نمای غیرصحیح (x^(1/2)·x^(3/2))
//! ژنراتورهای جدا هستند — ساده‌سازی کمتر، هرگز جواب غلط نه.

use std::collections::{BTreeMap, BTreeSet};

use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::{Signed, ToPrimitive, Zero};

use crate::sexpr;
use crate::{CasError, Sym};

/// سقف جمله‌ها در حاصل‌ضرب/توان — بالاتر از آن خطای صادقانه (سند ۰۵ §۱۲).
const MAX_TERMS: usize = 100_000;
/// سقف تکرار الگوریتم‌های تقسیم/gcd.
const MAX_GCD_STEPS: usize = 10_000;

pub type Mon = BTreeMap<String, i64>;

/// چندجمله‌ای sparse با ضرایب گویای دقیق (صفرها هرگز ذخیره نمی‌شوند).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Poly {
    pub terms: BTreeMap<Mon, BigRational>,
}

impl Poly {
    pub fn zero() -> Poly {
        Poly::default()
    }

    pub fn one() -> Poly {
        Poly::constant(BigRational::from(BigInt::from(1)))
    }

    pub fn constant(r: BigRational) -> Poly {
        let mut p = Poly::zero();
        if !r.is_zero() {
            p.terms.insert(Mon::new(), r);
        }
        p
    }

    pub fn is_zero(&self) -> bool {
        self.terms.is_empty()
    }

    pub fn is_constant(&self) -> bool {
        self.terms.iter().all(|(m, _)| m.is_empty())
    }

    pub fn constant_value(&self) -> Option<BigRational> {
        match self.terms.len() {
            0 => Some(BigRational::zero()),
            1 => self
                .terms
                .iter()
                .next()
                .filter(|(m, _)| m.is_empty())
                .map(|(_, c)| c.clone()),
            _ => None,
        }
    }

    /// ژنراتورهای حاضر (مرتب — قطعی).
    pub fn generators(&self) -> BTreeSet<String> {
        let mut out = BTreeSet::new();
        for m in self.terms.keys() {
            out.extend(m.keys().cloned());
        }
        out
    }

    pub fn add(&self, other: &Poly) -> Poly {
        let mut out = self.clone();
        for (m, c) in &other.terms {
            let e = out.terms.entry(m.clone()).or_default();
            *e += c;
        }
        out.terms.retain(|_, c| !c.is_zero());
        out
    }

    pub fn neg(&self) -> Poly {
        Poly {
            terms: self
                .terms
                .iter()
                .map(|(m, c)| (m.clone(), -c.clone()))
                .collect(),
        }
    }

    pub fn mul(&self, other: &Poly) -> Result<Poly, CasError> {
        let mut budget = 0usize;
        self.mul_with_budget(other, &mut budget)
    }

    /// توان صحیح نامنفی — با گارد بودجهٔ **انباشته** روی همهٔ گام‌ها
    /// (بودجهٔ هر-گامِ جدا، ضرب‌های متوالی را بدون سقف رها می‌کرد —
    /// درس اجرای اول: (1+x)^16384 به‌جای خطای سریع، دقیق حساب می‌شد).
    pub fn pow(&self, n: i64) -> Result<Poly, CasError> {
        if n < 0 {
            return Err(CasError::Unsupported("negative power of a polynomial"));
        }
        if n == 0 {
            return Ok(Poly::one());
        }
        if self.is_zero() {
            return Ok(Poly::zero());
        }
        let mut acc = Poly::one();
        let mut budget = 0usize;
        for _ in 0..n {
            acc = acc.mul_with_budget(self, &mut budget)?;
        }
        Ok(acc)
    }

    /// ضرب با بودجهٔ بیرونی (انباشته) — برای توان‌های تکرارشونده.
    fn mul_with_budget(&self, other: &Poly, budget: &mut usize) -> Result<Poly, CasError> {
        let mut out: BTreeMap<Mon, BigRational> = BTreeMap::new();
        for (ma, ca) in &self.terms {
            for (mb, cb) in &other.terms {
                let m = mon_mul(ma, mb, budget)?;
                *out.entry(m).or_default() += ca * cb;
            }
        }
        out.retain(|_, c| !c.is_zero());
        Ok(Poly { terms: out })
    }

    /// درجه در ژنراتور مشخص (۰ اگر حاضر نیست).
    pub fn degree_in(&self, gen: &str) -> i64 {
        self.terms
            .keys()
            .map(|m| m.get(gen).copied().unwrap_or(0))
            .max()
            .unwrap_or(0)
    }
}

/// تک‌جمله‌ای‌ضرب + شمارندهٔ بودجهٔ جمله‌ای.
fn mon_mul(a: &Mon, b: &Mon, budget: &mut usize) -> Result<Mon, CasError> {
    let mut out = Mon::new();
    for (g, ea) in a {
        let e = ea + b.get(g).copied().unwrap_or(0);
        if e != 0 {
            out.insert(g.clone(), e);
        }
    }
    for (g, eb) in b {
        if !a.contains_key(g) && *eb != 0 {
            out.insert(g.clone(), *eb);
        }
    }
    *budget = budget
        .checked_add(out.len())
        .ok_or(CasError::Unsupported("polynomial budget overflow"))?;
    if *budget > MAX_TERMS {
        return Err(CasError::Unsupported(
            "polynomial expansion too large (term budget)",
        ));
    }
    Ok(out)
}

// ---------- نگاه تک‌متغیره (main) ----------

/// نمایش تک‌متغیره در main: درجه → چندجمله‌ای ضرایب (درجه‌های خالی حذف).
fn univ(p: &Poly, main: &str) -> BTreeMap<i64, Poly> {
    let mut raw: BTreeMap<i64, BTreeMap<Mon, BigRational>> = BTreeMap::new();
    for (m, c) in &p.terms {
        let d = m.get(main).copied().unwrap_or(0);
        let mut rest = m.clone();
        rest.remove(main);
        *raw.entry(d).or_default().entry(rest).or_default() += c;
    }
    raw.into_iter()
        .filter_map(|(d, mut terms)| {
            terms.retain(|_, c| !c.is_zero());
            if terms.is_empty() {
                None
            } else {
                Some((d, Poly { terms }))
            }
        })
        .collect()
}

fn from_univ(u: &BTreeMap<i64, Poly>, main: &str) -> Poly {
    let mut out = Poly::zero();
    for (d, coef) in u {
        for (m, c) in &coef.terms {
            let mut full = m.clone();
            full.insert(main.to_string(), *d);
            *out.terms.entry(full).or_default() += c;
        }
    }
    out.terms.retain(|_, c| !c.is_zero());
    out
}

/// content نسبت به main: gcd چندجمله‌ایِ ضرایب (لم گاوس).
fn content_main(p: &Poly, main: &str) -> Result<Poly, CasError> {
    let u = univ(p, main);
    let mut g = Poly::zero();
    for coef in u.values() {
        g = if g.is_zero() {
            coef.clone()
        } else {
            poly_gcd(&g, coef)?
        };
    }
    Ok(g)
}

/// شبه‌باقیمانده (pseudo-remainder) در main — فقط + و × (بدون تقسیم ضرایب).
fn prem(a: &Poly, b: &Poly, main: &str) -> Result<Poly, CasError> {
    let (ua, ub) = (univ(a, main), univ(b, main));
    let (da, db) = (
        ua.keys().copied().max().unwrap_or(0),
        ub.keys().copied().max().unwrap_or(0),
    );
    if db == 0 || da < db {
        return Ok(a.clone()); // b در main ثابت یا a از پایین‌تر — طبق تعریف
    }
    let lead_b = ub
        .get(&db)
        .cloned()
        .unwrap_or_else(|| Poly::constant(BigRational::from(BigInt::from(1))));
    // work = a · lead_b^e — e دقیقاً به‌قدری که هیچ‌گاه توان منفی نیاید
    let mut work = a.clone();
    for _ in 0..(da - db + 1) {
        work = work.mul(&lead_b)?;
    }
    let mut uw = univ(&work, main);
    let mut steps = 0usize;
    while let Some(&dw) = uw.keys().next_back() {
        if dw < db {
            break;
        }
        steps += 1;
        if steps > MAX_GCD_STEPS {
            return Err(CasError::Unsupported("gcd step budget exceeded"));
        }
        let lead_w = uw
            .get(&dw)
            .cloned()
            .unwrap_or_else(|| Poly::constant(BigRational::from(BigInt::from(1))));
        // t1 = work·lead_b ; t2 = lead_w·b·x^(dw−db) → تفریق، رأس حذف می‌شود
        let mut t1 = univ(&from_univ(&uw, main).mul(&lead_b)?, main);
        let mut scaled: BTreeMap<i64, Poly> = BTreeMap::new();
        for (d, c) in &ub {
            scaled.insert(d + (dw - db), c.clone());
        }
        let sb = from_univ(&scaled, main).mul(&lead_w)?;
        for (d, c) in univ(&sb, main) {
            let e = t1.entry(d).or_default();
            *e = e.add(&c.neg());
        }
        t1.retain(|_, p| !p.is_zero());
        uw = t1;
    }
    Ok(from_univ(&uw, main))
}

/// gcd گویای دو ثابت — همیشه مثبت نرمال؛ gcd(a/b, c/d) = gcd(a·d, c·b)/(b·d).
fn const_gcd(a: &BigRational, b: &BigRational) -> BigRational {
    if a.is_zero() {
        return b.abs();
    }
    if b.is_zero() {
        return a.abs();
    }
    let (an, ad) = (a.numer().abs().clone(), a.denom().clone());
    let (bn, bd) = (b.numer().abs().clone(), b.denom().clone());
    let g = gcd_bigint(&(an * bd.clone()), &(bn * ad.clone()));
    BigRational::new(g, ad * bd)
}

fn gcd_bigint(a: &BigInt, b: &BigInt) -> BigInt {
    let (mut a, mut b) = (a.abs(), b.abs());
    while !b.is_zero() {
        let r = &a % &b;
        a = b;
        b = r;
    }
    a
}

/// تقسیم دقیق چندجمله‌ای (n = q·d) — تقسیم در میدان کسری ℚ(gens)[main]
/// با بررسی دقیق‌بودن در پایان (هرگز گرد نمی‌کنیم، نام‌دقیق = خطا).
pub fn pdiv_exact(n: &Poly, d: &Poly) -> Result<Poly, CasError> {
    if d.is_zero() {
        return Err(CasError::DivByZero);
    }
    if let Some(dc) = d.constant_value() {
        let mut out = Poly::zero();
        for (m, c) in &n.terms {
            out.terms.insert(m.clone(), c / &dc);
        }
        return Ok(out);
    }
    let gens = d.generators();
    let main = gens
        .iter()
        .next()
        .ok_or(CasError::Unsupported("empty generator set"))?
        .clone();
    let (un, ud) = (univ(n, &main), univ(d, &main));
    let dd = *ud
        .keys()
        .next_back()
        .ok_or(CasError::Unsupported("zero divisor degree"))?;
    let lead_d = RatFun::from_poly(
        ud.get(&dd)
            .ok_or(CasError::Unsupported("missing leading coefficient"))?,
    );
    let mut rem: BTreeMap<i64, RatFun> =
        un.iter().map(|(k, v)| (*k, RatFun::from_poly(v))).collect();
    let ud_rat: BTreeMap<i64, RatFun> =
        ud.iter().map(|(k, v)| (*k, RatFun::from_poly(v))).collect();
    let mut q: BTreeMap<i64, RatFun> = BTreeMap::new();
    let mut steps = 0usize;
    while let Some(&dr) = rem.keys().next_back() {
        if dr < dd {
            break;
        }
        steps += 1;
        if steps > MAX_GCD_STEPS {
            return Err(CasError::Unsupported("division step budget exceeded"));
        }
        let lead_r = rem
            .get(&dr)
            .cloned()
            .unwrap_or_else(|| RatFun::from_poly(&Poly::zero()));
        let f = lead_r.div(&lead_d)?; // در میدان کسری — همیشه ممکن
        q.insert(dr - dd, f.clone());
        for (k, c) in &ud_rat {
            let prod = f.mul(c)?;
            let e = rem
                .entry(k + (dr - dd))
                .or_insert_with(|| RatFun::from_poly(&Poly::zero()));
            *e = e.sub(&prod)?;
        }
        rem.retain(|_, v| !v.is_zero());
    }
    if !rem.is_empty() {
        return Err(CasError::Unsupported(
            "inexact polynomial division (remainder nonzero)",
        ));
    }
    // خارج‌قسمت باید چندجمله‌ای باشد (مخرج ۱) — وگرنه تقسیمِ چندجمله‌ای نبوده
    let mut out = Poly::zero();
    for (k, v) in q {
        for (m, c) in v.to_poly()?.terms {
            let mut full = m;
            full.insert(main.clone(), k);
            *out.terms.entry(full).or_default() += c;
        }
    }
    out.terms.retain(|_, c| !c.is_zero());
    Ok(out)
}

/// GCD چندمتغیره — PRS بدوی + بازگشت لم گاوس (بالای فایل).
pub fn poly_gcd(a: &Poly, b: &Poly) -> Result<Poly, CasError> {
    // گارد عمق بازگشت متقابل — درس CAS-2: باقیمانده‌های دارای abs/sign
    // تودرتو (از back-substitution جایگذاری) ژنراتورهای اُپک می‌سازند و
    // چرخهٔ pdiv_exact ↔ cancel ↔ poly_gcd می‌تواند بی‌کران شود (سرریز
    // پشته — مجموعهٔ مرجع ۱۲۰، مورد (x²+1)/(x−1)). فراتر از بودجه =
    // خطای صادقانه، هرگز فروریختن نه (اصل ۵ سند ۰۰).
    thread_local! {
        static GCD_DEPTH: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
    }
    const GCD_MAX_DEPTH: u32 = 64;
    let depth = GCD_DEPTH.with(|d| {
        let cur = d.get();
        d.set(cur + 1);
        cur
    });
    let _guard = DepthGuardReset;
    struct DepthGuardReset;
    impl Drop for DepthGuardReset {
        fn drop(&mut self) {
            GCD_DEPTH.with(|d| d.set(d.get().saturating_sub(1)));
        }
    }
    if depth > GCD_MAX_DEPTH {
        return Err(CasError::Unsupported("gcd recursion budget exceeded"));
    }
    poly_gcd_inner(a, b)
}

fn poly_gcd_inner(a: &Poly, b: &Poly) -> Result<Poly, CasError> {
    if a.is_zero() {
        return Ok(normalize(b));
    }
    if b.is_zero() {
        return Ok(normalize(a));
    }
    let mut gens: BTreeSet<String> = a.generators();
    gens.extend(b.generators());
    let Some(main) = gens.iter().next().cloned() else {
        // هر دو ثابت — gcd گویا
        let (x, y) = (
            a.constant_value().unwrap_or_else(BigRational::zero),
            b.constant_value().unwrap_or_else(BigRational::zero),
        );
        return Ok(Poly::constant(const_gcd(&x, &y)));
    };
    let (da, db) = (a.degree_in(&main), b.degree_in(&main));
    if da == 0 || db == 0 {
        // یکی در main ثابت است: gcd(c, p) = gcd(c, content_main(p))
        let (c, p) = if da == 0 { (a, b) } else { (b, a) };
        let cont = content_main(p, &main)?;
        return poly_gcd(c, &cont);
    }
    // PRS بدوی: تفکیک content، سپس چرخهٔ pseudo-remainder
    let ca = content_main(a, &main)?;
    let cb = content_main(b, &main)?;
    let g = poly_gcd(&ca, &cb)?;
    let mut pa = pdiv_exact(a, &ca)?;
    let mut pb = pdiv_exact(b, &cb)?;
    if pa.degree_in(&main) < pb.degree_in(&main) {
        std::mem::swap(&mut pa, &mut pb);
    }
    let mut steps = 0usize;
    loop {
        steps += 1;
        if steps > MAX_GCD_STEPS {
            return Err(CasError::Unsupported("gcd step budget exceeded"));
        }
        let r = prem(&pa, &pb, &main)?;
        if r.is_zero() {
            return Ok(normalize(&g.mul(&pb)?));
        }
        if r.is_constant() {
            return Ok(normalize(&g));
        }
        let cont_r = content_main(&r, &main)?;
        let rp = pdiv_exact(&r, &cont_r)?;
        if rp.degree_in(&main) == 0 {
            // باقیمانده در main ثابت شد → gcd(pa, pb) از content می‌آید (لم گاوس)
            let cont_pa = content_main(&pa, &main)?;
            let g2 = poly_gcd(&cont_pa, &rp)?;
            return Ok(normalize(&g.mul(&g2)?));
        }
        pa = pb;
        pb = rp;
    }
}

/// نرمال‌سازی: ضریبِ رأس (بزرگ‌ترین تک‌جمله‌ای — next_back) مثبت.
fn normalize(p: &Poly) -> Poly {
    let Some((_, lead)) = p.terms.iter().next_back() else {
        return Poly::zero();
    };
    let lead = lead.clone();
    if lead > BigRational::zero() {
        return p.clone();
    }
    let inv = -lead;
    Poly {
        terms: p.terms.iter().map(|(m, c)| (m.clone(), c / &inv)).collect(),
    }
}

// ---------- تابع گویا (نسبت دو چندجمله‌ای) ----------

/// تابع گویای کانونی — num/den؛ حذف gcd فقط با فراخوان صریح `cancel`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RatFun {
    pub num: Poly,
    pub den: Poly,
}

impl RatFun {
    pub fn from_poly(p: &Poly) -> RatFun {
        RatFun {
            num: p.clone(),
            den: Poly::one(),
        }
    }

    pub fn zero() -> RatFun {
        RatFun::from_poly(&Poly::zero())
    }

    pub fn is_zero(&self) -> bool {
        self.num.is_zero() && !self.den.is_zero()
    }

    /// درخت Sym → RatFun (گسترش). Real → خطای صادقانه (تقریبی، نه چندجمله‌ای).
    pub fn from_sym(e: &Sym) -> Result<RatFun, CasError> {
        match e {
            Sym::Integer(i) => Ok(RatFun::from_poly(&Poly::constant(
                BigRational::from(i.clone()),
            ))),
            Sym::Rational(r) => Ok(RatFun::from_poly(&Poly::constant(r.clone()))),
            Sym::Real(_) => Err(CasError::Unsupported(
                "approximate (real) coefficients are outside polynomial normalization; keep them symbolic or use exact rationals",
            )),
            Sym::Symbol(_) => Ok(RatFun::from_poly(&generator_poly(e))),
            Sym::Add(terms) => {
                let mut acc = RatFun::zero();
                for t in terms {
                    acc = acc.add(&RatFun::from_sym(t)?)?;
                }
                Ok(acc)
            }
            Sym::Mul(factors) => {
                let mut acc = RatFun::from_poly(&Poly::one());
                for f in factors {
                    acc = acc.mul(&RatFun::from_sym(f)?)?;
                }
                Ok(acc)
            }
            Sym::Pow(b, ex) => {
                if let Some(n) = small_int(ex) {
                    let base = RatFun::from_sym(b)?;
                    return if n >= 0 {
                        Ok(RatFun {
                            num: base.num.pow(n)?,
                            den: base.den.pow(n)?,
                        })
                    } else {
                        if base.num.is_zero() {
                            return Err(CasError::DivByZero);
                        }
                        Ok(RatFun {
                            num: base.den.pow(-n)?,
                            den: base.num.pow(-n)?,
                        })
                    };
                }
                // توان غیرصحیح/نمادین → کل گره ژنراتور است (بدون گسترش درون)
                Ok(RatFun::from_poly(&generator_poly(e)))
            }
            Sym::Fn(..) => {
                // آرگومان‌ها عمداً دست‌نخورده‌اند: گسترش داخل توابع (مثل
                // expand مثلثاتیِ مجموع‌ها) قلمرو لایهٔ قواعد است نه فرم
                // نرمال — سند ۰۵ §۳. ژنراتور = sexpr گره با آرگومان خام.
                Ok(RatFun::from_poly(&generator_poly(e)))
            }
        }
    }

    pub fn add(&self, other: &RatFun) -> Result<RatFun, CasError> {
        // a/b + c/d = (a·d + c·b)/(b·d)
        let n = self.num.mul(&other.den)?.add(&other.num.mul(&self.den)?);
        let d = self.den.mul(&other.den)?;
        Ok(RatFun { num: n, den: d })
    }

    pub fn sub(&self, other: &RatFun) -> Result<RatFun, CasError> {
        self.add(&other.neg())
    }

    pub fn neg(&self) -> RatFun {
        RatFun {
            num: self.num.neg(),
            den: self.den.clone(),
        }
    }

    pub fn mul(&self, other: &RatFun) -> Result<RatFun, CasError> {
        Ok(RatFun {
            num: self.num.mul(&other.num)?,
            den: self.den.mul(&other.den)?,
        })
    }

    pub fn div(&self, other: &RatFun) -> Result<RatFun, CasError> {
        if other.num.is_zero() {
            return Err(CasError::DivByZero);
        }
        Ok(RatFun {
            num: self.num.mul(&other.den)?,
            den: self.den.mul(&other.num)?,
        })
    }

    /// حذف gcd صورت/مخرج + مخرجِ رأس مثبت. صورت صفر → 0/1.
    pub fn cancel(&mut self) -> Result<(), CasError> {
        if self.num.is_zero() {
            self.den = Poly::one();
            return Ok(());
        }
        if self.den.is_zero() {
            return Err(CasError::DivByZero);
        }
        if let Some(d) = self.den.constant_value() {
            // مخرج ثابت → ضریب به صورت (کسر گویا تمیز)
            self.num = Poly {
                terms: self
                    .num
                    .terms
                    .iter()
                    .map(|(m, c)| (m.clone(), c / &d))
                    .collect(),
            };
            self.den = Poly::one();
            return Ok(());
        }
        let g = poly_gcd(&self.num, &self.den)?;
        if let Some(gc) = g.constant_value() {
            // gcd ثابت → فقط ضریب گویای مشترک
            if !gc.is_zero() {
                self.num = Poly {
                    terms: self
                        .num
                        .terms
                        .iter()
                        .map(|(m, c)| (m.clone(), c / &gc))
                        .collect(),
                };
                self.den = Poly {
                    terms: self
                        .den
                        .terms
                        .iter()
                        .map(|(m, c)| (m.clone(), c / &gc))
                        .collect(),
                };
            }
        } else {
            self.num = pdiv_exact(&self.num, &g)?;
            self.den = pdiv_exact(&self.den, &g)?;
        }
        // مخرج رأس مثبت (قرارداد نمایش کسری) — «رأس» = بزرگ‌ترین تک‌جمله‌ای
        // (next_back)، نه کوچک‌ترین: درس اجرای اول، x^2-1 با رأسِ ثابتِ -1
        // منفی خوانده می‌شد و 2x/(x^2-1) به شکل -2x/(1-x^2) درمی‌آمد
        let lead = self
            .den
            .terms
            .iter()
            .next_back()
            .map(|(_, c)| c.clone())
            .unwrap_or_else(|| BigRational::from(BigInt::from(1)));
        if lead < BigRational::zero() {
            self.num = self.num.neg();
            self.den = self.den.neg();
        }
        Ok(())
    }

    /// چندجمله‌ای خالص؟ (مخرج ثابت) — خارج‌قسمت با ضریب اعمال‌شده.
    pub fn to_poly(&self) -> Result<Poly, CasError> {
        if let Some(d) = self.den.constant_value() {
            return Ok(Poly {
                terms: self
                    .num
                    .terms
                    .iter()
                    .map(|(m, c)| (m.clone(), c / &d))
                    .collect(),
            });
        }
        Err(CasError::Unsupported(
            "result is a rational function, not a polynomial",
        ))
    }

    /// RatFun → درخت Sym کانونی (سازنده‌ها دوباره سطح-۰ را می‌گذارند).
    pub fn to_sym(&self) -> Result<Sym, CasError> {
        if let Some(d) = self.den.constant_value() {
            let n = poly_to_sym(&self.num)?;
            if d == BigRational::from(BigInt::from(1)) {
                return Ok(n);
            }
            return Sym::over(n, Sym::Rational(d));
        }
        let n = poly_to_sym(&self.num)?;
        let d = poly_to_sym(&self.den)?;
        Sym::over(n, d)
    }
}

fn small_int(s: &Sym) -> Option<i64> {
    if let Sym::Integer(i) = s {
        return i.to_i64();
    }
    None
}

/// ژنراتورِ کلِ یک گره غیرچندجمله‌ای.
fn generator_poly(e: &Sym) -> Poly {
    let mut m = Mon::new();
    m.insert(sexpr::to_sexp(e), 1);
    let mut p = Poly::zero();
    p.terms.insert(m, BigRational::from(BigInt::from(1)));
    p
}

/// چندجمله‌ای → درخت (ضرایب × ژنراتورها به‌توان‌ها؛ ترتیب کانونی خودکار).
pub fn poly_to_sym(p: &Poly) -> Result<Sym, CasError> {
    if p.is_zero() {
        return Ok(Sym::int(0));
    }
    let mut acc: Option<Sym> = None;
    for (m, c) in &p.terms {
        let mut term = Sym::Rational(c.clone());
        for (g, e) in m {
            let gen = sexpr::from_sexp(g)?;
            let f = if *e == 1 {
                gen
            } else {
                Sym::pow(gen, Sym::int(*e))?
            };
            term = Sym::times(term, f);
        }
        acc = Some(match acc {
            None => term,
            Some(a) => Sym::plus(a, term),
        });
    }
    Ok(acc.unwrap_or_else(|| Sym::int(0)))
}

// ============================================================
// تست‌ها
// ============================================================
#[cfg(test)]
mod tests {
    use crate::testkit::*;

    use super::*;

    fn poly_of(e: &Sym) -> Poly {
        let mut rf = RatFun::from_sym(e).expect("t");
        rf.cancel().expect("t");
        rf.to_poly().expect("t")
    }

    fn sym_of(p: &Poly) -> Sym {
        poly_to_sym(p).expect("t")
    }

    #[test]
    fn expand_distributes() {
        // (x+1)*(x+2) = x^2 + 3x + 2
        let e = times(&[add(&[sym("x"), int(1)]), add(&[sym("x"), int(2)])]);
        assert_eq!(sym_of(&poly_of(&e)).display(), "x^2 + 3*x + 2");
        // (x+2)^2 = x^2+4x+4
        let e2 = pow(&add(&[sym("x"), int(2)]), &int(2));
        assert_eq!(sym_of(&poly_of(&e2)).display(), "x^2 + 4*x + 4");
    }

    #[test]
    fn expand_multivariate() {
        // (x+y)*(x-y) = x^2 - y^2
        let e = times(&[add(&[sym("x"), sym("y")]), add(&[sym("x"), neg(&sym("y"))])]);
        assert_eq!(sym_of(&poly_of(&e)).display(), "x^2 - y^2");
    }

    #[test]
    fn expand_keeps_generators_honest() {
        // sqrt(x)^2 ژنراتور می‌ماند (هرگز x نمی‌شود — گارد دامنه)
        let s = f("sqrt", &[sym("x")]);
        let e = times(&[s.clone(), s.clone()]);
        assert_eq!(sym_of(&poly_of(&e)).display(), "sqrt(x)^2");
        // sin(x)+sin(x) = 2*sin(x)
        let e2 = add(&[f("sin", &[sym("x")]), f("sin", &[sym("x")])]);
        assert_eq!(sym_of(&poly_of(&e2)).display(), "2*sin(x)");
    }

    #[test]
    fn rational_cancel_classic() {
        // (x^2-1)/(x-1) = x+1 — gcd چندمتغیره واقعی
        let num = add(&[pow(&sym("x"), &int(2)), int(-1)]);
        let den = add(&[sym("x"), int(-1)]);
        let e = over(&num, &den).expect("t");
        let mut rf = RatFun::from_sym(&e).expect("t");
        rf.cancel().expect("t");
        assert_eq!(rf.to_sym().expect("t").display(), "x + 1");
    }

    #[test]
    fn rational_cancel_multivariate() {
        // (x*y + x)/(y+1) = x
        let num = add(&[times(&[sym("x"), sym("y")]), sym("x")]);
        let den = add(&[sym("y"), int(1)]);
        let e = over(&num, &den).expect("t");
        let mut rf = RatFun::from_sym(&e).expect("t");
        rf.cancel().expect("t");
        assert_eq!(rf.to_sym().expect("t").display(), "x");
    }

    #[test]
    fn gcd_coprime_is_unit() {
        // gcd(x, y) = 1
        let px = RatFun::from_sym(&sym("x")).expect("t").num;
        let py = RatFun::from_sym(&sym("y")).expect("t").num;
        let g = poly_gcd(&px, &py).expect("t");
        assert!(g.is_constant());
        // gcd(x^2-2x+1, x-1) = x-1
        let a = poly_of(&add(&[
            pow(&sym("x"), &int(2)),
            times(&[int(-2), sym("x")]),
            int(1),
        ]));
        let b = poly_of(&add(&[sym("x"), int(-1)]));
        let g2 = poly_gcd(&a, &b).expect("t");
        assert_eq!(sym_of(&g2).display(), "x - 1");
    }

    #[test]
    fn gcd_constants() {
        // gcd(6x, 4x) = 2x
        let a = poly_of(&times(&[int(6), sym("x")]));
        let b = poly_of(&times(&[int(4), sym("x")]));
        let g = poly_gcd(&a, &b).expect("t");
        assert_eq!(sym_of(&g).display(), "2*x");
    }

    #[test]
    fn negative_power_becomes_denominator() {
        // x*(x+1)^-1 → x/(x+1)
        let e = times(&[sym("x"), pow(&add(&[sym("x"), int(1)]), &int(-1))]);
        let mut rf = RatFun::from_sym(&e).expect("t");
        rf.cancel().expect("t");
        assert_eq!(rf.to_sym().expect("t").display(), "x/(x + 1)");
    }

    #[test]
    fn real_rejected_honestly() {
        let e = times(&[Sym::Real(0.1), sym("x")]);
        assert!(RatFun::from_sym(&e).is_err());
    }

    #[test]
    fn zero_power_convention() {
        assert_eq!(Poly::zero().pow(0).expect("t"), Poly::one());
        assert_eq!(Poly::zero().pow(3).expect("t"), Poly::zero());
    }

    #[test]
    fn expansion_budget_honest() {
        // (1+x)^16384 باید بودجه را بشکند نه حافظه را
        let e = pow(&add(&[int(1), sym("x")]), &int(16_384));
        assert!(matches!(
            RatFun::from_sym(&e),
            Err(CasError::Unsupported(_))
        ));
    }

    #[test]
    fn rational_sum_normalizes() {
        // 1/(x-1) + 1/(x+1) = 2x/(x^2-1)
        let a = over(&int(1), &add(&[sym("x"), int(-1)])).expect("t");
        let b = over(&int(1), &add(&[sym("x"), int(1)])).expect("t");
        let sum = Sym::plus(a, b);
        let mut rf = RatFun::from_sym(&sum).expect("t");
        rf.cancel().expect("t");
        assert_eq!(rf.to_sym().expect("t").display(), "2*x/(x^2 - 1)");
    }
}
