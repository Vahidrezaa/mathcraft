//! درخت عبارت سمبولیک + حساب دقیق + کانونی‌سازی سطح-۰ (سند ۰۵ §۲–۳).
//!
//! ساختار کانونی (Canonical Structure):
//! - `Add`/`Mul` همیشه تخت و مرتب با ترتیب کلیدی پایدار (خروجی repeatable).
//! - جمع‌ها به (ضریب گویا) + (بقیهٔ ترم‌ها) نرمال می‌شوند؛ توان‌های هم‌پایه
//!   ادغام می‌شوند — اما فقط وقتی نتیجه قطعاً درست است (گاردهای پایین).
//! - صفر جمع = حذف؛ `x¹=x`، `x⁰=۱`؛ ضرب در صفر فقط با عامل‌های «کل-دامنه»
//!   جمع می‌شود — `0*x^-1` جمع نمی‌شود چون در x=0 تعریف‌نشده است (قاعدهٔ
//!   گارد سند ۰۵ §۲ — جایی که CASهای آماتور صادق نیستند؛ ما صادق می‌مانیم).
//! - `Real` (تقریبی) هرگز با گویاهای دقیق مخلوط «مساوی» اعلام نمی‌شود؛
//!   در جمع/ضربِ عددی به f64 منتشر می‌شود (تقریب صادقانه).

use std::cmp::Ordering;
use std::fmt::Write as _;
use std::sync::Arc;

use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::{Signed, ToPrimitive, Zero};

/// سقف قدر مطلق توان صحیح برای حساب دقیق — بالاتر از این، خطای صادقانه
/// (نه هنگ/حافظهٔ بی‌پایان روی دستگاه).
const MAX_EXACT_EXPONENT: i64 = 100_000;

/// خطاهای صادقانهٔ لایهٔ سمبولیک — مفسر به پیام دوزبانه نگاشت می‌کند.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CasError {
    /// تقسیم بر صفرِ قطعی (مثل `0^-1` یا x/0) — ساخت درخت رد می‌شود.
    DivByZero,
    /// تبدیل به double ممکن نیست: نمادِ تعویض‌نشده در درخت مانده است.
    SymbolLeft(String),
    /// مقدار دقیق در بازهٔ f64 جا نمی‌شود.
    F64Overflow,
    /// عملی که در CAS-0 نمادین نمی‌شود (مسیر صادقانه — هرگز جواب غلط نه).
    Unsupported(&'static str),
}

/// درخت عبارت سمبولیک (سند ۰۵ §۲).
///
/// `Real` فقط ورودیِ کاربر/غیرگویاست: `sym(0.1)` چون 0.1 در مبنای ۲ دقیق
/// نیست `Real` می‌ماند و در کل درخت به‌عنوان «تقریبی» منتشر می‌شود —
/// هرگز خود را به‌عنوان 1/10 جا نمی‌زند (اصل صداقت، سند ۰۰ اصل ۵).
#[derive(Debug, Clone, PartialEq)]
pub enum Sym {
    /// عدد صحیح دقیق.
    Integer(BigInt),
    /// کسر دقیق — همیشه کوپریم (BigRational خودکار ساده می‌کند)؛
    /// `1/3` هرگز به float تبدیل نمی‌شود (تصمیم D-12).
    Rational(BigRational),
    /// فقط ورود کاربر/غیرگویای صریح.
    Real(f64),
    /// نماد آزاد: x, y, theta.
    Symbol(Arc<str>),
    /// جمع تختِ کانونی (≥۲ عضو، مرتب، ترم عددی آخر).
    Add(Vec<Sym>),
    /// ضرب تختِ کانونی (≥۲ عضو، مرتب، ضریب عددی اول).
    Mul(Vec<Sym>),
    /// توان — `x^-1` برای تقسیم نمادین نگه داشته می‌شود.
    Pow(Box<Sym>, Box<Sym>),
    /// تابع نام‌دار — در CAS-0 مسیر مفسر برای آرگومان سمبولیک خطای روشن
    /// می‌دهد؛ این.variant برای سازگاری طراحی سند ۰۵ و subs آماده است
    /// (CAS-1: sin/exp/log/...).
    Fn(Arc<str>, Vec<Sym>),
}

impl Sym {
    /// عدد صحیح کوچک — کمکی خوانا برای تست‌ها و فراخوانی‌های مفسر.
    pub fn int(i: i64) -> Sym {
        Sym::Integer(BigInt::from(i))
    }

    /// کسر دقیق — خودکار ساده (BigRational).
    pub fn rat(n: i64, d: i64) -> Result<Sym, CasError> {
        if d == 0 {
            return Err(CasError::DivByZero);
        }
        Ok(Sym::Rational(BigRational::new(
            BigInt::from(n),
            BigInt::from(d),
        )))
    }

    /// نماد آزاد.
    pub fn symbol(name: &str) -> Sym {
        Sym::Symbol(Arc::from(name))
    }

    /// ورود f64 به دنیای سمبولیک — صادقانه:
    /// - صحیح در بازهٔ i64 → `Integer` دقیق؛
    /// - کسر دودوییِ «مرتب» (≤۲۰ بیت کسری، مثل 2.5=5/2) → `Rational` دقیق؛
    /// - هرچیز دیگر (0.1، NaN، سرریزها) → `Real` تقریبی.
    pub fn from_f64(x: f64) -> Sym {
        if !x.is_finite() {
            return Sym::Real(x);
        }
        if x.fract() == 0.0 {
            if let Some(i) = x.to_i64() {
                return Sym::Integer(BigInt::from(i));
            }
            return Sym::Real(x); // |x| > i64 — دقت f64 می‌ماند
        }
        // کسر دودویی مرتب؟ تا ۲۰ بیت کسری (نیم، ربع، ... تا ۱/۱۰۴۸۵۷۶)
        let mut den: i64 = 1;
        let mut num = x;
        for _ in 0..20 {
            num *= 2.0;
            den *= 2;
            if num.fract() == 0.0 {
                if let Some(n) = num.to_i64() {
                    if n as f64 == num {
                        return Sym::Rational(BigRational::new(BigInt::from(n), BigInt::from(den)));
                    }
                }
                break;
            }
        }
        Sym::Real(x)
    }

    /// گره «عدد دقیق» است؟ (صحیح یا گویا — نه Real تقریبی)
    fn as_exact(&self) -> Option<BigRational> {
        match self {
            Sym::Integer(i) => Some(BigRational::from(i.clone())),
            Sym::Rational(r) => Some(r.clone()),
            _ => None,
        }
    }

    /// آیا این درخت هرگز `Real` ندارد؟ (برای جمع ضریب‌ها در Add — دو ترمِ
    /// دارای Real را ادغام نمی‌کنیم چون جمع f64 آن‌ها با ضریب‌زدن فرق دارد)
    fn is_exact_tree(&self) -> bool {
        match self {
            Sym::Integer(_) | Sym::Rational(_) | Sym::Symbol(_) => true,
            Sym::Real(_) => false,
            Sym::Add(v) | Sym::Mul(v) => v.iter().all(Self::is_exact_tree),
            Sym::Pow(b, e) => b.is_exact_tree() && e.is_exact_tree(),
            Sym::Fn(_, args) => args.iter().all(Self::is_exact_tree),
        }
    }

    /// آیا گره به‌طور قطعی روی همهٔ دامنه تعریف شده است؟ (گارد ضرب در صفر —
    /// سند ۰۵ §۲): عدد، نماد، جمع/ضربِ عامل‌های total، و توانِ صحیح نامنفی
    /// از پایهٔ total. `x^-1` (توان منفی) تعریف‌نشده می‌ماند.
    /// `Real` فقط در حالت متناهی total است: `0*Inf`/`0*NaN` در متلب NaN اند،
    /// پس فروختنِ `0*(…inf…)` به 0 صادقانه نیست (ممیزی ۲۰۲۶-۱۱-۲۱).
    fn is_total(&self) -> bool {
        match self {
            Sym::Integer(_) | Sym::Rational(_) | Sym::Symbol(_) => true,
            Sym::Real(x) => x.is_finite(),
            Sym::Pow(b, e) => matches!(&**e, Sym::Integer(i) if !i.is_negative()) && b.is_total(),
            Sym::Add(v) | Sym::Mul(v) => v.iter().all(Self::is_total),
            Sym::Fn(..) => false, // محافظه‌کار: sin کل-دامنه است، log نه — جدول در CAS-1
        }
    }

    // ---------- سازنده‌های کانونی (سطح-۰ — ساده‌سازی همیشه روشن) ----------

    /// جمع کانونی: تخت‌سازی، جمع ضرایب ترم‌های هم‌ساختار، حذف صفر، ترتیب پایدار.
    pub fn plus(a: Sym, b: Sym) -> Sym {
        // مسیر عددی: exact+exact دقیق؛ Real → f64 (تقریب صادقانه)
        if let (Some(x), Some(y)) = (a.as_exact(), b.as_exact()) {
            return rebuild_rational(x + y);
        }
        match (&a, &b) {
            (Sym::Real(x), other) | (other, Sym::Real(x)) => {
                if let Some(y) = other.as_exact() {
                    return Sym::Real(x + ratio_f64(&y));
                }
                if let Sym::Real(y) = other {
                    return Sym::Real(x + y);
                }
            }
            _ => {}
        }
        // تخت‌سازی Add(Add(..)) → یک لیست؛ جداسازی (ضریب، باقی) هر ترم
        let mut terms: Vec<(BigRational, Sym)> = Vec::new();
        let mut approx: Vec<Sym> = Vec::new(); // ترم‌های حاوی Real — ادغام نمی‌شوند
        for t in iter_add_terms(a).into_iter().chain(iter_add_terms(b)) {
            let (coeff, rest) = split_coeff(t);
            if coeff.is_zero() && matches!(rest, Sym::Integer(ref i) if i.is_zero()) {
                continue; // عدد صفرِ خالص حذف
            }
            if !rest.is_exact_tree() {
                // ترم تقریبی (مثل 0.1*x) — جمع ضریبی ناامن؛ صادقانه نگه می‌داریم
                let term = if rat_is_one(&coeff) {
                    rest
                } else {
                    Sym::times(rebuild_rational(coeff), rest)
                };
                approx.push(term);
                continue;
            }
            if let Some(slot) = terms.iter_mut().find(|(_, r)| *r == rest) {
                *slot = (slot.0.clone() + &coeff, rest.clone());
            } else {
                terms.push((coeff, rest));
            }
        }
        // ترم‌های تقریبی (حاوی Real) بدون بازگشت، مستقیم در جمع کانونی
        // می‌نشینند — درس ممیزی کیفیت ۲۰۲۶-۱۱-۲۱: حلقهٔ بازگشتی قبلی
        // `plus(acc, t)` با ورودیِ هم‌شکل دوباره وارد همان شاخه می‌شد و
        // عبارت‌هایی مثل `x + 0.1*x` سرریز پشته می‌داد (مسیر تست‌نشده).
        rebuild_add(terms, approx)
    }

    /// تفریق — via جمع با منفی‌کردن.
    pub fn minus(a: Sym, b: Sym) -> Sym {
        Sym::plus(a, Sym::negated(b))
    }

    /// منفی یکه — کانونی: `Mul(-1, x)`.
    pub fn negated(a: Sym) -> Sym {
        Sym::times(Sym::int(-1), a)
    }

    /// ضرب کانونی: تخت‌سازی، ضریب گویا، ادغام توان‌های هم‌پایه (با گارد)،
    /// گارد ضرب در صفر، ترتیب پایدار.
    pub fn times(a: Sym, b: Sym) -> Sym {
        if let (Some(x), Some(y)) = (a.as_exact(), b.as_exact()) {
            return rebuild_rational(x * y);
        }
        match (&a, &b) {
            (Sym::Real(x), other) | (other, Sym::Real(x)) => {
                if let Some(y) = other.as_exact() {
                    return Sym::Real(x * ratio_f64(&y));
                }
            }
            _ => {}
        }
        let mut coeff = BigRational::from(BigInt::from(1));
        let mut factors: Vec<Sym> = Vec::new();
        for f in iter_mul_factors(a).into_iter().chain(iter_mul_factors(b)) {
            if let Some(r) = f.as_exact() {
                coeff *= r;
                continue;
            }
            if let Sym::Real(_) = f {
                factors.push(f); // عامل تقریبی — درجا می‌ماند
                continue;
            }
            // ادغام توان هم‌پایه: x^2 * x^3 = x^5 — فقط با گارد (پایین)
            if let Sym::Pow(base, e) = &f {
                let pos = factors.iter().position(
                    |g| matches!(g, Sym::Pow(b2, e2) if b2 == base && as_small_int(e2).is_some()),
                );
                if let Some(i) = pos {
                    let e2 = match &factors[i] {
                        Sym::Pow(_, e2) => as_small_int(e2),
                        _ => None,
                    };
                    if let (Some(n2), Some(n1)) = (e2, as_small_int(e)) {
                        if let Some(sum) = n2.checked_add(n1) {
                            // ادغام امن فقط وقتی حاصل مثبت است یا پایه قطعاً ناصفر
                            if sum > 0 || base.as_exact().is_some_and(|r| !r.is_zero()) {
                                factors[i] = Sym::pow((**base).clone(), Sym::int(sum)).unwrap_or(f);
                                continue;
                            }
                        }
                    }
                    // ادغام ناممکن/ناامن → هر دو عامل می‌مانند (صادقانه)
                }
            }
            factors.push(f);
        }
        // گارد ضرب در صفر: فقط با عامل‌های قطعاً کل-دامنه (سند ۰۵ §۲)
        if coeff.is_zero() && factors.iter().all(|f| f.is_total()) {
            return Sym::int(0);
        }
        if factors.is_empty() {
            return rebuild_rational(coeff); // ضرب عددی خالص
        }
        // فاکتورهای ساختاریِ برابر → توان صحیح: x*x = x^2
        let mut merged: Vec<Sym> = Vec::new();
        for f in factors {
            let mut hit = false;
            for slot in merged.iter_mut() {
                if *slot == f {
                    *slot = Sym::pow(f.clone(), Sym::int(2)).unwrap_or(f.clone());
                    hit = true;
                    break;
                }
            }
            if !hit {
                merged.push(f);
            }
        }
        merged.sort_by(sym_sort);
        let mut out: Vec<Sym> = Vec::new();
        if coeff.is_zero() {
            out.push(Sym::int(0)); // 0 × X نامجموع (X بالقوه تعریف‌نشده)
        } else if !rat_is_one(&coeff) {
            out.push(rebuild_rational(coeff));
        }
        out.extend(merged);
        match out.len() {
            0 => Sym::int(1),
            1 => out.into_iter().next().unwrap_or_else(|| Sym::int(1)),
            _ => Sym::Mul(out),
        }
    }

    /// تقسیم: عدد/صفر → خطا؛ x/عدد → کسر دقیق؛ مخرج نمادین → توان -۱.
    pub fn over(a: Sym, b: Sym) -> Result<Sym, CasError> {
        if let Some(d) = b.as_exact() {
            if d.is_zero() {
                return Err(CasError::DivByZero);
            }
            if let Some(n) = a.as_exact() {
                return Ok(rebuild_rational(n / d));
            }
            let inv = BigRational::from(BigInt::from(1)) / d;
            return Ok(Sym::times(a, rebuild_rational(inv)));
        }
        if let Sym::Real(d) = &b {
            if d == &0.0 {
                return Err(CasError::DivByZero);
            }
        }
        // مخرج نمادین: توان -۱ (تعریف‌نشده در صفرِ مخرج — گارد بالا)
        Ok(Sym::times(a, Sym::pow(b, Sym::int(-1))?))
    }

    /// توان کانونی: `x^0=1`، `x^1=x`، `(x^2)^3=x^6` (با گارد)، عدد^توان‌صحیح
    /// دقیق (با سقف توان — بالاتر از آن خطای صادقانه، نه هنگِ بی‌پایان).
    pub fn pow(base: Sym, exp: Sym) -> Result<Sym, CasError> {
        if let Some(n) = as_small_int(&exp) {
            if n.abs() > MAX_EXACT_EXPONENT {
                return Err(CasError::Unsupported(
                    "exponent too large for exact arithmetic (limit 100000)",
                ));
            }
            if let Some(b) = base.as_exact() {
                if b.is_zero() && n < 0 {
                    return Err(CasError::DivByZero);
                }
                return Ok(rebuild_rational(pow_ratio(&b, n)?));
            }
            if let Sym::Real(x) = &base {
                if *x == 0.0 && n < 0 {
                    return Err(CasError::DivByZero);
                }
                return Ok(Sym::Real(x.powi(n as i32)));
            }
            if n == 0 {
                return Ok(Sym::int(1)); // قرارداد جهانی x^0=1 (متلب همین است)
            }
            if n == 1 {
                return Ok(base);
            }
            if let Sym::Pow(b2, e2) = &base {
                // (x^a)^n → x^(a*n) فقط وقتی حاصل مثبت/امن است
                if let Some(m) = as_small_int(e2) {
                    if let Some(p) = m.checked_mul(n) {
                        if p.abs() <= MAX_EXACT_EXPONENT
                            && (p > 0 || b2.as_exact().is_some_and(|r| !r.is_zero()))
                        {
                            return Sym::pow((**b2).clone(), Sym::int(p));
                        }
                    }
                }
            }
            return Ok(Sym::Pow(Box::new(base), Box::new(exp)));
        }
        // توان غیرصحیح/نمادین روی پایهٔ عددی → تقریب صادقانهٔ f64.
        // صفرِ «واقعی» را از زیرریزِ تبدیل جدا می‌کنیم: (۱۰^-۴۰۰)^(-۱/۲)
        // جوابش ۱۰^۲۰۰ است، نه خطای تقسیم بر صفر (ممیزی ۲۰۲۶-۱۱-۲۱).
        if let (Some(b), Some(e)) = (base.as_exact(), exp.as_exact()) {
            let ef = ratio_f64(&e);
            if b.numer().is_zero() && ef < 0.0 {
                return Err(CasError::DivByZero);
            }
            return Ok(Sym::Real(pow_exact_approx(&b, &e)));
        }
        if let Sym::Real(bf) = &base {
            if let Some(e) = exp.as_exact() {
                let ef = ratio_f64(&e);
                if *bf == 0.0 && ef < 0.0 {
                    return Err(CasError::DivByZero);
                }
                return Ok(Sym::Real(bf.powf(ef)));
            }
        }
        Ok(Sym::Pow(Box::new(base), Box::new(exp)))
    }

    // ---------- subs و تبدیل به double ----------

    /// جای‌گذاری نام نماد با عبارت — بازساخت از سازنده‌های کانونی، پس
    /// ساده‌سازی سطح-۰ دوباره اعمال می‌شود (subs(x^2+3*x, x, 5/2) → 55/4).
    pub fn substitute(&self, name: &str, with: &Sym) -> Sym {
        match self {
            Sym::Integer(_) | Sym::Rational(_) | Sym::Real(_) => self.clone(),
            Sym::Symbol(s) if &**s == name => with.clone(),
            Sym::Symbol(_) => self.clone(),
            Sym::Add(terms) => terms
                .iter()
                .map(|t| t.substitute(name, with))
                .reduce(Sym::plus)
                .unwrap_or_else(|| Sym::int(0)),
            Sym::Mul(factors) => factors
                .iter()
                .map(|f| f.substitute(name, with))
                .reduce(Sym::times)
                .unwrap_or_else(|| Sym::int(1)),
            Sym::Pow(b, e) => {
                let nb = b.substitute(name, with);
                let ne = e.substitute(name, with);
                // جای‌گذاری محض خطا نمی‌دهد؛ توان نامعتبر (مثل 0^-1) صادقانه
                // به‌صورت درختِ ارزیابی‌نشده می‌ماند و در to_f64 خطا می‌دهد.
                Sym::pow(nb, ne).unwrap_or_else(|_| {
                    Sym::Pow(
                        Box::new(b.substitute(name, with)),
                        Box::new(e.substitute(name, with)),
                    )
                })
            }
            Sym::Fn(f, args) => Sym::Fn(
                f.clone(),
                args.iter().map(|a| a.substitute(name, with)).collect(),
            ),
        }
    }

    /// تبدیل به f64 — فقط وقتی درخت همهٔ نمادها را از دست داده باشد.
    /// گویاها تا واپسین گام دقیق می‌مانند و یک بار تبدیل می‌شوند
    /// (subs عددی معمولاً کل درخت را به یک برگِ گویا می‌لاند).
    pub fn to_f64(&self) -> Result<f64, CasError> {
        match self {
            Sym::Real(x) => Ok(*x),
            Sym::Integer(i) => i
                .to_f64()
                .filter(|f| f.is_finite())
                .ok_or(CasError::F64Overflow),
            Sym::Rational(r) => r
                .to_f64()
                .filter(|f| f.is_finite())
                .ok_or(CasError::F64Overflow),
            Sym::Symbol(s) => Err(CasError::SymbolLeft(s.to_string())),
            Sym::Add(terms) => {
                let mut acc = 0.0f64;
                for t in terms {
                    acc += t.to_f64()?;
                }
                Ok(acc)
            }
            Sym::Mul(factors) => {
                let mut acc = 1.0f64;
                for f in factors {
                    acc *= f.to_f64()?;
                }
                Ok(acc)
            }
            Sym::Pow(b, e) => {
                // مسیر دقیق: پایه گویا و توان صحیح کوچک — بدون دور رفتن به f64
                if let (Some(bb), Some(n)) = (b.as_exact(), as_small_int(e)) {
                    if bb.is_zero() && n < 0 {
                        return Err(CasError::DivByZero);
                    }
                    return pow_ratio(&bb, n)?
                        .to_f64()
                        .filter(|f| f.is_finite())
                        .ok_or(CasError::F64Overflow);
                }
                Ok(b.to_f64()?.powf(e.to_f64()?))
            }
            Sym::Fn(..) => Err(CasError::Unsupported(
                "numeric evaluation of symbolic functions arrives in CAS-1",
            )),
        }
    }

    /// نمایش خطی (CAS-0) — قطعی و پایدار؛ LaTeX در CAS-1 (سند ۰۵ §۱۰).
    pub fn display(&self) -> String {
        let mut s = String::new();
        write_sym(&mut s, self, Prec::Top);
        s
    }
}

/// نمایش خطی — نام عمومی برای مفسر.
pub fn sym_display(s: &Sym) -> String {
    s.display()
}

// ---------- کمکی‌های داخلی ----------

/// ترتیب کلیدی پایدار — عددی صفر (در Add ترم عددی جداگانه آخر می‌افتد)،
/// بعد نماد، توان، تابع، جمع، ضرب؛ هم‌رده با متن نمایشی.
fn sym_sort(a: &Sym, b: &Sym) -> Ordering {
    fn rank(s: &Sym) -> u8 {
        match s {
            Sym::Integer(_) | Sym::Rational(_) | Sym::Real(_) => 0,
            Sym::Symbol(_) => 1,
            Sym::Pow(_, _) => 2,
            Sym::Fn(_, _) => 3,
            Sym::Add(_) => 4,
            Sym::Mul(_) => 5,
        }
    }
    match rank(a).cmp(&rank(b)) {
        Ordering::Equal => a.display().cmp(&b.display()),
        other => other,
    }
}

/// توان صحیح کوچک؟ (i64)
fn as_small_int(s: &Sym) -> Option<i64> {
    if let Sym::Integer(i) = s {
        return i.to_i64();
    }
    None
}

/// توان گویا با اسکالر i64 — دقیق؛ فراخواننده سقف و صفر بودن را چک کرده.
fn pow_ratio(r: &BigRational, n: i64) -> Result<BigRational, CasError> {
    if n < 0 && r.is_zero() {
        return Err(CasError::DivByZero);
    }
    Ok(r.pow(n as i32))
}

/// آیا گویا دقیقاً ۱ است؟
fn rat_is_one(r: &BigRational) -> bool {
    r.numer() == &BigInt::from(1) && r.denom() == &BigInt::from(1)
}

/// گویا → f64 (مسیر تقریبی Real). تبدیل num-rational در مقیاس بزرگ هم
/// درست است (۱۰^۴۰۰/۱۰^۴۰۰ → 1.0، سرریز → ±inf — با تست پین‌شده)، اما
/// fallback هرگز 0 غلط نمی‌دهد: اگر روزی None بیاید، NAN «نامشخص» می‌ماند
/// (ممیزی ۲۰۲۶-۱۱-۲۱ — قبلاً unwrap_or(0.0) بود).
fn ratio_f64(r: &BigRational) -> f64 {
    r.to_f64().unwrap_or(f64::NAN)
}

/// توانِ گویای دقیق با توان گویا — تقریب صادقانهٔ f64 بدون قربانی‌کردن
/// دامنه (ممیزی ۲۰۲۶-۱۱-۲۱): اگر پایه در بازهٔ f64 جا شود powf خودِ IEEE
/// کافی است؛ برای پایه‌های غول/ریز (۱۰^۴۰۰، ۱۰^-۴۰۰) که در تبدیل مستقیم
/// inf/0 می‌شوند در دامنهٔ لگاریتمی حساب می‌کنیم: (۱۰^۴۰۰)^(۱/۲) → ~1e200
/// (نه inf)، (۱۰^-۴۰۰)^(۱/۲) → ~1e-200 (نه 0). خطای نسبی ~۱e-14 است (سقفِ
/// کوانتیزه‌شدنِ خودِ ln|b| در f64) — برای `Real`ِ تقریبی صادقانه است.
/// سرریز/زیرریزِ «واقعی» همچنان ±inf/0 می‌ماند؛ پایهٔ منفی با توان غیرصحیح → NaN (مختلط، CAS-1+)
/// و با توان صحیح بزرگ‌تر از i64 علامت از زوج/فردِ توان می‌آید.
fn pow_exact_approx(b: &BigRational, e: &BigRational) -> f64 {
    let bf = ratio_f64(b);
    let ef = ratio_f64(e);
    let e_int = *e.denom() == BigInt::from(1);
    let b_neg = b.numer().sign() == num_bigint::Sign::Minus;
    if b_neg && !e_int {
        // پایهٔ منفی با توان غیرصحیح → مختلط؛ در CAS-0 NaN صادقانه (سند ۰۵ §۱۲)
        return f64::NAN;
    }
    // توان صحیح بزرگ‌تر از i64 از مسیر as_small_int جا می‌ماند و اینجاست؛
    // علامتش به زوج/فردِ توان بستگی دارد (پاریتهٔ متلب، مستقل از powf کتابخانه)
    let sign = if b_neg && e.numer().bit(0) { -1.0 } else { 1.0 };
    let mag = if bf.is_finite() && (bf != 0.0 || b.numer().is_zero()) {
        bf.abs().powf(ef)
    } else if b.numer().is_zero() {
        0.0 // 0^مثبت (توان منفی قبلاً DivByZero شد)
    } else {
        (ef * ln_abs_rational(b)).exp()
    };
    sign * mag
}

/// ln|گویا| برای اعداد خارج از بازهٔ f64 — از ۵۴ بیت بالای صورت و مخرج.
/// ln2 دو-جزئی fdlibm تا خطای `shift×ln2` حذف شود (~۱e-13 → حذف)؛ خطای
/// نسبی نهایی ~۱e-14 است که سقفِ کوانتیزه‌شدنِ خودِ ln|b| (~۹۲۱) در f64
/// است — برای مسیر تقریبیِ `Real` صادقانه کافی است.
fn ln_abs_rational(r: &BigRational) -> f64 {
    // ثابت‌های fdlibm — با پروب راستی‌آزمایی شدند: hi+lo == LN_2 دقیقاً
    // و فرم کوتاه همان بیتهای فرم بلند را دارد (تسک ۴۹)
    const LN_2_HI: f64 = 6.931_471_803_691_238e-1; // bits 0x3FE62E42FEE00000
    const LN_2_LO: f64 = 1.908_214_929_270_587_7e-10; // bits 0x3DEA39EF35793C76
    let n = r.numer().abs();
    let d = r.denom();
    let (nb, db) = (n.bits() as i64, d.bits() as i64);
    let sn = (nb - 54).max(0) as u64;
    let sd = (db - 54).max(0) as u64;
    let n_hi = (&n >> sn).to_f64().unwrap_or(f64::INFINITY);
    let d_hi = (d >> sd).to_f64().unwrap_or(1.0);
    let shift = (sn as i64 - sd as i64) as f64;
    (n_hi.ln() - d_hi.ln()) + shift.mul_add(LN_2_HI, shift * LN_2_LO)
}

/// ترم‌های جمع (تخت‌سازی) — برگ یا اعضای Add.
fn iter_add_terms(s: Sym) -> Vec<Sym> {
    match s {
        Sym::Add(v) => v,
        other => vec![other],
    }
}

/// عامل‌های ضرب (تخت‌سازی) — برگ یا اعضای Mul.
fn iter_mul_factors(s: Sym) -> Vec<Sym> {
    match s {
        Sym::Mul(v) => v,
        other => vec![other],
    }
}

/// جداسازی (ضریب گویا، باقیٔ ترم) برای جمع کانونی: `3*x^2` → (3, x^2).
fn split_coeff(t: Sym) -> (BigRational, Sym) {
    if let Some(r) = t.as_exact() {
        return (r, Sym::int(1));
    }
    if let Sym::Mul(factors) = &t {
        if let Some(first) = factors.first() {
            if let Some(r) = first.as_exact() {
                let rest: Vec<Sym> = factors[1..].to_vec();
                if rest.is_empty() {
                    return (r, Sym::int(1));
                }
                if rest.len() == 1 {
                    return (r, rest.into_iter().next().unwrap_or_else(|| Sym::int(1)));
                }
                return (r, Sym::Mul(rest));
            }
        }
    }
    (BigRational::from(BigInt::from(1)), t)
}

/// گویا → برگ تمیز (صحیح اگر مخرج ۱).
fn rebuild_rational(r: BigRational) -> Sym {
    if r.denom() == &BigInt::from(1) {
        Sym::Integer(r.to_integer())
    } else {
        Sym::Rational(r)
    }
}

/// بازساخت جمع از (ضریب، ترم)ها + ترم‌های تقریبی (حاوی `Real` — هرگز با
/// هیچ ترمی ادغام نمی‌شوند؛ جمعِ f64 دو ترم تقریبی با ضریب‌های دیگر
/// ضریب‌پذیر نیست و صادقانه جدا می‌مانند). ترم‌های عددی به یک ثابتِ
/// انتهایی جمع می‌شوند، ضرایبِ صحیحِ ۱ حذف می‌شوند و ترتیب پایدار است
/// (متلب: x^2 + 3*x + 5 — ترم عددی، دقیق یا Real تقریبی، آخر).
fn rebuild_add(terms: Vec<(BigRational, Sym)>, approx: Vec<Sym>) -> Sym {
    let mut parts: Vec<Sym> = Vec::new();
    let mut num = BigRational::from(BigInt::from(0));
    for (c, rest) in terms {
        if matches!(rest, Sym::Integer(ref i) if *i == BigInt::from(1)) {
            num += c; // ترم عددی
            continue;
        }
        if rat_is_one(&c) {
            parts.push(rest);
        } else {
            parts.push(Sym::times(rebuild_rational(c), rest));
        }
    }
    parts.extend(approx);
    if !num.is_zero() {
        parts.push(rebuild_rational(num));
    }
    // مرتب‌سازی پایدار: ترم‌های عددی (دقیق یا Real تقریبی) آخر (متلب‌گونه)
    let mut syms: Vec<Sym> = Vec::new();
    let mut consts: Vec<Sym> = Vec::new();
    for p in parts {
        if p.as_exact().is_some() || matches!(p, Sym::Real(_)) {
            consts.push(p);
        } else {
            syms.push(p);
        }
    }
    syms.sort_by(sym_sort);
    consts.sort_by(sym_sort);
    syms.append(&mut consts);
    match syms.len() {
        0 => Sym::int(0),
        1 => syms.into_iter().next().unwrap_or_else(|| Sym::int(0)),
        _ => Sym::Add(syms),
    }
}

// ---------- نمایش خطی ----------

/// تقدم زمینه در نمایش — برای پرانتزگذاری درست.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Prec {
    Top,
    Add,
    Mul,
    Pow,
    Atom,
}

fn write_sym(out: &mut String, s: &Sym, ctx: Prec) {
    match s {
        Sym::Integer(i) => {
            let _ = write!(out, "{i}");
        }
        Sym::Rational(r) => {
            let (n, d) = (r.numer(), r.denom());
            if d == &BigInt::from(1) {
                let _ = write!(out, "{n}");
            } else if ctx == Prec::Atom {
                // در جایگاه توان: x^(1/2) — پرانتز لازم
                let _ = write!(out, "({n}/{d})");
            } else {
                // جایگاه جمع/ضرب با جداکنندهٔ '*' روشن است: 1/3*x
                let _ = write!(out, "{n}/{d}");
            }
        }
        Sym::Real(x) => {
            let _ = write!(out, "{x}");
        }
        Sym::Symbol(n) => out.push_str(n),
        Sym::Add(terms) => {
            if ctx > Prec::Add {
                out.push('(');
            }
            for (i, t) in terms.iter().enumerate() {
                if i == 0 {
                    write_sym(out, t, Prec::Add);
                    continue;
                }
                // ترم منفی با « - » می‌چسبد (متلب‌گونه: x - 3*y) — علامت از
                // نمایش ترم خوانده می‌شود (ممیزی ۲۰۲۶-۱۱-۲۱؛ قبلاً «x + -3*y»)
                let mut buf = String::new();
                write_sym(&mut buf, t, Prec::Add);
                if let Some(stripped) = buf.strip_prefix('-') {
                    out.push_str(" - ");
                    out.push_str(stripped);
                } else {
                    out.push_str(" + ");
                    out.push_str(&buf);
                }
            }
            if ctx > Prec::Add {
                out.push(')');
            }
        }
        Sym::Mul(factors) => {
            if ctx > Prec::Mul {
                out.push('(');
            }
            write_mul(out, factors);
            if ctx > Prec::Mul {
                out.push(')');
            }
        }
        Sym::Pow(b, e) => {
            // توان -۱ → 1/base (نمایش متلب‌گونه) — فقط وقتی خودش ریشهٔ
            // عبارت است؛ داخل ضرب عامل به‌صورت x^-1 می‌ماند (write_mul خودش
            // شکل y/x را می‌سازد).
            if ctx != Prec::Mul {
                if let Some(n) = as_small_int(e) {
                    if n == -1 {
                        out.push_str("1/");
                        if matches!(&**b, Sym::Symbol(_)) {
                            write_sym(out, b, Prec::Mul);
                        } else {
                            out.push('(');
                            write_sym(out, b, Prec::Top);
                            out.push(')');
                        }
                        return;
                    }
                }
            }
            let need_b = matches!(&**b, Sym::Add(_) | Sym::Mul(_) | Sym::Real(_))
                || matches!(&**b, Sym::Integer(ref i) if *i < BigInt::from(0))
                || matches!(&**b, Sym::Rational(ref r) if *r.numer() < BigInt::from(0));
            if need_b {
                out.push('(');
                write_sym(out, b, Prec::Top);
                out.push(')');
            } else {
                write_sym(out, b, Prec::Pow);
            }
            out.push('^');
            let need_e = matches!(&**e, Sym::Add(_) | Sym::Mul(_) | Sym::Real(_));
            if need_e {
                out.push('(');
                write_sym(out, e, Prec::Top);
                out.push(')');
            } else {
                write_sym(out, e, Prec::Atom);
            }
        }
        Sym::Fn(f, args) => {
            let _ = write!(out, "{f}(");
            for (i, a) in args.iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                write_sym(out, a, Prec::Top);
            }
            out.push(')');
        }
    }
}

/// نمایش ضرب — ضریب ۱- اول → علامت منفی بیرون (-x نه -1*x)؛
/// شکل تقسیم: آخرین عامل Pow(b,-1) بدون ضریب عددی در بقیه → `rest/b`
/// (y/x متلب‌گونه)؛ در غیر این‌ صورت عامل‌ها با '*' جدا می‌شوند.
fn write_mul(out: &mut String, factors: &[Sym]) {
    let mut start = 0;
    if factors.len() > 1 {
        if let Some(first) = factors.first() {
            if first.as_exact() == Some(BigRational::from(BigInt::from(-1))) {
                out.push('-');
                start = 1;
            }
        }
    }
    // شکل تقسیم؟ y*x^-1 → y/x (بدون ضریب عددی در عامل‌های قبل)
    if let (Some(Sym::Pow(b, e)), rest) = (factors.last(), &factors[start..factors.len() - 1]) {
        if as_small_int(e) == Some(-1)
            && !rest.is_empty()
            && rest
                .iter()
                .all(|f| !matches!(f, Sym::Integer(_) | Sym::Rational(_) | Sym::Real(_)))
        {
            for (i, f) in rest.iter().enumerate() {
                if i > 0 {
                    out.push('*');
                }
                write_sym(out, f, Prec::Mul);
            }
            out.push('/');
            if matches!(&**b, Sym::Symbol(_)) {
                write_sym(out, b, Prec::Mul);
            } else {
                out.push('(');
                write_sym(out, b, Prec::Top);
                out.push(')');
            }
            return;
        }
    }
    for (i, f) in factors.iter().enumerate().skip(start) {
        if i > start {
            out.push('*');
        }
        write_sym(out, f, Prec::Mul);
    }
}

// ============================================================
// تست‌های واحد mc-cas (گام 4.1 — سند: ≥۱۵ تست در کل پروژه)
// ============================================================
#[cfg(test)]
mod tests {
    use super::*;

    fn x() -> Sym {
        Sym::symbol("x")
    }

    #[test]
    fn add_zero_drops() {
        assert_eq!(Sym::plus(x(), Sym::int(0)).display(), "x");
        assert_eq!(Sym::plus(Sym::int(0), x()).display(), "x");
    }

    #[test]
    fn add_collects_like_terms() {
        assert_eq!(Sym::plus(x(), x()).display(), "2*x");
        let a = Sym::times(Sym::int(2), x());
        let b = Sym::times(Sym::int(3), x());
        assert_eq!(Sym::plus(a, b).display(), "5*x");
    }

    #[test]
    fn add_exact_rationals() {
        // D-12: 1/3 + 1/6 = 1/2 — دقیق، هرگز 0.49999…
        let a = Sym::rat(1, 3).expect("test rat");
        let b = Sym::rat(1, 6).expect("test rat");
        assert_eq!(Sym::plus(a, b).display(), "1/2");
        // (1/3)*3 = 1 دقیق (نه 0.999…)
        let c = Sym::rat(1, 3).expect("test rat");
        assert_eq!(Sym::times(c, Sym::int(3)).display(), "1");
    }

    #[test]
    fn add_sorts_deterministically() {
        let x2 = Sym::pow(x(), Sym::int(2)).expect("test pow");
        let e1 = Sym::plus(Sym::times(Sym::int(3), x()), x2.clone());
        let e2 = Sym::plus(x2, Sym::times(Sym::int(3), x()));
        // ترتیب پایدار فارغ از ترتیب ورودی — متلب‌گونه: توان قبل از ضریب
        assert_eq!(e1.display(), "x^2 + 3*x");
        assert_eq!(e1.display(), e2.display());
    }

    #[test]
    fn add_numeric_term_goes_last() {
        let y = Sym::plus(Sym::plus(Sym::times(Sym::int(3), x()), Sym::int(5)), x());
        assert_eq!(y.display(), "4*x + 5");
    }

    #[test]
    fn mul_zero_guard() {
        // 0*x = 0 (نماد کل-دامنه است)
        assert_eq!(Sym::times(Sym::int(0), x()).display(), "0");
        // 0*x^-1 جمع نمی‌شود — در x=0 تعریف‌نشده (گارد سند ۰۵ §۲)
        let xi = Sym::pow(x(), Sym::int(-1)).expect("test pow");
        assert_eq!(Sym::times(Sym::int(0), xi).display(), "0*x^-1");
    }

    #[test]
    fn mul_merges_powers_positive() {
        let a = Sym::pow(x(), Sym::int(2)).expect("test pow");
        let b = Sym::pow(x(), Sym::int(3)).expect("test pow");
        assert_eq!(Sym::times(a, b).display(), "x^5");
        assert_eq!(Sym::times(x(), x()).display(), "x^2");
    }

    #[test]
    fn mul_symbolic_division_form() {
        // x/3 → 1/3*x (کسر دقیق در ضریب)
        let d = Sym::over(x(), Sym::int(3)).expect("test div");
        assert_eq!(d.display(), "1/3*x");
        // y/x → y*x^-1 (توان -۱ نمایش 1/x می‌گیرد)
        let y = Sym::symbol("y");
        let q = Sym::over(y, x()).expect("test div");
        assert_eq!(q.display(), "y/x");
    }

    #[test]
    fn pow_identities() {
        assert_eq!(Sym::pow(x(), Sym::int(0)).expect("t").display(), "1");
        assert_eq!(Sym::pow(x(), Sym::int(1)).expect("t").display(), "x");
        let b = Sym::pow(x(), Sym::int(2)).expect("t");
        assert_eq!(Sym::pow(b, Sym::int(3)).expect("t").display(), "x^6");
    }

    #[test]
    fn pow_exact_numeric() {
        assert_eq!(
            Sym::pow(Sym::int(2), Sym::int(10)).expect("t").display(),
            "1024"
        );
        let h = Sym::rat(1, 2).expect("t");
        assert_eq!(Sym::pow(h, Sym::int(-2)).expect("t").display(), "4");
        // توان کسری → تقریب صادقانهٔ Real
        assert_eq!(
            Sym::pow(Sym::int(4), Sym::rat(1, 2).expect("t")).expect("t"),
            Sym::Real(2.0)
        );
    }

    #[test]
    fn pow_errors_honest() {
        assert_eq!(
            Sym::pow(Sym::int(0), Sym::int(-1)),
            Err(CasError::DivByZero)
        );
        assert_eq!(
            Sym::pow(Sym::int(2), Sym::int(10_000_000)),
            Err(CasError::Unsupported(
                "exponent too large for exact arithmetic (limit 100000)"
            ))
        );
    }

    #[test]
    fn div_by_exact_zero_errors() {
        assert_eq!(Sym::over(x(), Sym::int(0)), Err(CasError::DivByZero));
        assert_eq!(Sym::over(x(), Sym::Real(0.0)), Err(CasError::DivByZero));
    }

    #[test]
    fn from_f64_honest() {
        assert_eq!(Sym::from_f64(2.5).display(), "5/2");
        assert_eq!(Sym::from_f64(0.25).display(), "1/4");
        assert_eq!(Sym::from_f64(3.0).display(), "3");
        assert_eq!(Sym::from_f64(-2.5).display(), "-5/2");
        // غیردودویی → Real تقریبی (هرگز خود را 1/10 جا نمی‌زند)
        assert_eq!(Sym::from_f64(0.1), Sym::Real(0.1));
        assert!(matches!(Sym::from_f64(f64::NAN), Sym::Real(v) if v.is_nan()));
    }

    #[test]
    fn real_propagates_approx() {
        let a = Sym::from_f64(0.1);
        assert_eq!(Sym::plus(a.clone(), Sym::int(1)), Sym::Real(1.1));
        // 2*0.1 → Real(0.2) تقریبی — دقیق جعل نمی‌شود
        assert_eq!(Sym::times(Sym::int(2), a), Sym::Real(0.2));
    }

    #[test]
    fn substitute_dod_roundtrip() {
        // DoD گام 4.1: x^2 + 3*x با x=5/2 → 55/4 → 13.75 دقیق
        let x2 = Sym::pow(x(), Sym::int(2)).expect("t");
        let y = Sym::plus(x2, Sym::times(Sym::int(3), x()));
        assert_eq!(y.display(), "x^2 + 3*x");
        let subbed = y.substitute("x", &Sym::from_f64(2.5));
        assert_eq!(subbed.display(), "55/4");
        assert_eq!(subbed.to_f64(), Ok(13.75));
    }

    #[test]
    fn substitute_symbolic_stays_tree() {
        let x2 = Sym::pow(x(), Sym::int(2)).expect("t");
        let y = Sym::plus(x2, Sym::times(Sym::int(3), x()));
        let t = y.substitute("x", &Sym::symbol("t"));
        assert_eq!(t.display(), "t^2 + 3*t");
        assert_eq!(t.to_f64(), Err(CasError::SymbolLeft("t".into())));
    }

    #[test]
    fn substitute_exact_cancel() {
        // (1/3 + x) با x=2/3 → 1 دقیق
        let y = Sym::plus(Sym::rat(1, 3).expect("t"), x());
        let s = y.substitute("x", &Sym::rat(2, 3).expect("t"));
        assert_eq!(s.display(), "1");
    }

    #[test]
    fn to_f64_overflow_honest() {
        let big = Sym::pow(Sym::int(10), Sym::int(400)).expect("t");
        assert_eq!(big.to_f64(), Err(CasError::F64Overflow));
        // توان ۰ منفیِ ارزیابی‌نشده (باقی‌مانده از subs) → خطای تقسیم بر صفر
        let bad = Sym::pow(Sym::int(0), Sym::int(-1))
            .unwrap_or_else(|_| Sym::Pow(Box::new(Sym::int(0)), Box::new(Sym::int(-1))));
        assert_eq!(bad.to_f64(), Err(CasError::DivByZero));
    }

    #[test]
    fn fn_display_and_subs() {
        let f = Sym::Fn(Arc::from("sin"), vec![x()]);
        assert_eq!(f.display(), "sin(x)");
        let g = f.substitute("x", &Sym::int(2));
        assert_eq!(g.display(), "sin(2)");
    }

    #[test]
    fn neg_displays_cleanly() {
        assert_eq!(Sym::negated(x()).display(), "-x");
        assert_eq!(
            Sym::plus(Sym::negated(x()), Sym::times(Sym::int(2), x())).display(),
            "x"
        );
        assert_eq!(Sym::negated(Sym::times(Sym::int(3), x())).display(), "-3*x");
    }

    #[test]
    fn parens_where_needed() {
        let s = Sym::symbol("s");
        // (x+1)*s
        let inner = Sym::plus(x(), Sym::int(1));
        assert_eq!(Sym::times(inner, s).display(), "s*(x + 1)"); // ترتیب کانونی: نماد قبل از جمع
                                                                 // (x+1)^2
        let p = Sym::pow(Sym::plus(x(), Sym::int(1)), Sym::int(2)).expect("t");
        assert_eq!(p.display(), "(x + 1)^2");
    }

    // ---------- ممیزی کیفیت ۲۰۲۶-۱۱-۲۱ (تسک ۴۹) ----------

    #[test]
    fn plus_approx_terms_terminate() {
        // قبلاً سرریز پشته: plus(acc, t) با ترم تقریبی دوباره وارد همان
        // شاخه می‌شد (بازگشت بی‌پایان) — مسیر تست‌نشدهٔ CAS-0
        let t = Sym::times(Sym::from_f64(0.1), x());
        assert_eq!(Sym::plus(x(), t).display(), "x + 0.1*x");
        // همان شاخه از سمت دیگر: x + 0.1
        assert_eq!(Sym::plus(x(), Sym::from_f64(0.1)).display(), "x + 0.1");
        // دو ترم تقریبی مجزا ادغام نمی‌شوند (جمع f64 ضرایب ضریب‌پذیر نیست)
        let a = Sym::times(Sym::from_f64(0.1), x());
        let b = Sym::times(Sym::from_f64(0.2), x());
        assert_eq!(Sym::plus(a, b).display(), "0.1*x + 0.2*x");
        // با ثابت دقیق: ترم عددی آخر (متلب‌گونه)
        let y = Sym::plus(
            Sym::plus(x(), Sym::int(5)),
            Sym::times(Sym::from_f64(0.1), x()),
        );
        assert_eq!(y.display(), "x + 0.1*x + 5");
    }

    #[test]
    fn zero_times_infinite_symbolic_stays_honest() {
        // 0*(inf*x): جمع‌شدن به 0 غلط است — متلب 0*Inf = NaN می‌دهد
        let inf_x = Sym::times(Sym::from_f64(f64::INFINITY), x());
        let z = Sym::times(Sym::int(0), inf_x);
        assert_ne!(z, Sym::int(0));
        // پس از جایگذاری x، ارزیابی صادقانه NaN است نه 0
        let zz = z.substitute("x", &Sym::int(1));
        assert!(matches!(zz.to_f64(), Ok(v) if v.is_nan()));
    }

    #[test]
    fn zero_times_total_expression_folds() {
        // (x+1)^2 روی همهٔ دامنه تعریف است → 0*(x+1)^2 = 0
        // (قبلاً فقط پایهٔ نمادِ خالص total شمرده می‌شد — محافظه‌کاری بی‌دلیل)
        let p = Sym::pow(Sym::plus(x(), Sym::int(1)), Sym::int(2)).expect("t");
        assert_eq!(Sym::times(Sym::int(0), p).display(), "0");
        // توان منفی → کل-دامنه نیست → نمی‌چیند
        let q = Sym::pow(Sym::plus(x(), Sym::int(1)), Sym::int(-1)).expect("t");
        assert_ne!(Sym::times(Sym::int(0), q), Sym::int(0));
    }

    #[test]
    fn huge_rational_conversions_pinned() {
        // پینِ درستیِ num-rational 0.4.2 در مقیاس بزرگ — رگرسیون وابستگی
        // باید اینجا قرمز شود، نه در دست کاربر (ممیزی ۲۰۲۶-۱۱-۲۱)
        let p = |n: u32| BigInt::from(10).pow(n);
        let one = BigRational::new(p(400), p(400));
        assert_eq!(Sym::Rational(one).to_f64(), Ok(1.0));
        let ten = BigRational::new(p(400), p(399));
        assert_eq!(Sym::Rational(ten).to_f64(), Ok(10.0));
        let slightly = BigRational::new(p(400) + BigInt::from(1), p(400));
        assert_eq!(Sym::Rational(slightly).to_f64(), Ok(1.0));
        // Real + گویای غول → جمع صادقانهٔ IEEE (inf) — نه 0ِ fallback قدیمی
        // (۱٫۰ نه — از خودش Integer دقیق می‌سازد و مسیر exact+exact می‌رود؛
        //  ۰٫۱ می‌ماند Real و همین مسیر تقریبی را می‌آزماید)
        let huge = Sym::Rational(BigRational::from(p(400)));
        assert_eq!(
            Sym::plus(Sym::from_f64(0.1), huge),
            Sym::Real(f64::INFINITY)
        );
    }

    #[test]
    fn pow_out_of_range_base_honest() {
        // (۱۰^۴۰۰)^(۱/۲): قبلاً inf غلط (تبدیل زودهنگامِ پایه به f64)؛ ~1e200
        let base = Sym::Integer(BigInt::from(10).pow(400));
        let half = Sym::rat(1, 2).expect("t");
        let r = Sym::pow(base.clone(), half.clone()).expect("t");
        assert!(matches!(r, Sym::Real(v) if (v - 1e200).abs() < 1e187));
        // (۱۰^-۴۰۰)^(۱/۲): قبلاً 0 غلط (زیرریز تبدیل)؛ ~1e-200
        let tiny = Sym::Rational(BigRational::new(BigInt::from(1), BigInt::from(10).pow(400)));
        let r2 = Sym::pow(tiny.clone(), half).expect("t");
        assert!(matches!(r2, Sym::Real(v) if (v - 1e-200).abs() < 1e-187));
        // سرریزِ واقعی همچنان inf صادقانه: (۱۰^۴۰۰)^(۳/۲) = ۱۰^۶۰۰
        let three_half = Sym::rat(3, 2).expect("t");
        let r3 = Sym::pow(base, three_half).expect("t");
        assert!(matches!(r3, Sym::Real(v) if v == f64::INFINITY));
        // (۱۰^-۴۰۰)^(-۱/۲) = ۱۰^۲۰۰ — قبلاً DivByZero غلط (صفرِ زیرریز!)
        let neg_half = Sym::rat(-1, 2).expect("t");
        let r4 = Sym::pow(tiny, neg_half).expect("t");
        assert!(matches!(r4, Sym::Real(v) if (v - 1e200).abs() < 1e187));
    }

    #[test]
    fn pow_negative_base_parity_honest() {
        // (-۸)^(۱/۳) مختلط است → NaN صادقانه در CAS-0 (سند ۰۵ §۱۲)
        let cbrt = Sym::pow(Sym::int(-8), Sym::rat(1, 3).expect("t")).expect("t");
        assert!(matches!(cbrt, Sym::Real(v) if v.is_nan()));
        // توان صحیحِ بزرگ‌تر از i64: (-۲)^(۱۰^۲۰) زوج → +inf؛ +۱ (فرد) → -inf
        let even = Sym::Integer(BigInt::from(10).pow(20));
        let odd = Sym::Integer(BigInt::from(10).pow(20) + BigInt::from(1));
        let re = Sym::pow(Sym::int(-2), even).expect("t");
        assert!(matches!(re, Sym::Real(v) if v == f64::INFINITY));
        let ro = Sym::pow(Sym::int(-2), odd).expect("t");
        assert!(matches!(ro, Sym::Real(v) if v == f64::NEG_INFINITY));
    }

    #[test]
    fn add_negative_terms_display_matlab_style() {
        // ممیزی ۲۰۲۶-۱۱-۲۱: قبلاً «x + -3*y» — اکنون متلب‌گونه
        let y3 = Sym::times(Sym::int(-3), Sym::symbol("y"));
        assert_eq!(Sym::plus(x(), y3).display(), "x - 3*y");
        // ثابت منفیِ انتهایی
        let z = Sym::plus(Sym::plus(x(), Sym::int(2)), Sym::int(-5));
        assert_eq!(z.display(), "x - 3");
        // ترم تقریبی منفی هم همین‌طور
        let a = Sym::times(Sym::from_f64(-0.1), x());
        assert_eq!(Sym::plus(x(), a).display(), "x - 0.1*x");
    }
}
