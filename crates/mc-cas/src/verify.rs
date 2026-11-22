//! خودآزمایی عددی مشتق (گام 4.2/CAS-1 — DoD «۱۰۰٪ تست مشتق خودآزمایی»).
//!
//! هر مشتق نمادین، در نقاط نمونهٔ قطعی (LCG با بذرِ هشِ عبارت) با تفاضل
//! مرکزی + اکسترپولاسیون ریچاردسون مقایسه می‌شود. نقاط بیرون از دامنه
//! (NaN/Inf) صادقانه رد می‌شوند؛ حداقل نقاط معتبر لازم است وگرنه «تأیید
//! نشد» گزارش می‌شود — هرگز «معتبر» اعلام نمی‌شود بی‌شاهد.
//!
//! این ماژول هم زیربنای تست دائمی است (سند ۰۸) و هم ابزار CAS-2 برای
//! diff-verify خودکار نتایج انتگرال.

use crate::diff;
use crate::{CasError, Sym};

/// گزارش خودآزمایی.
#[derive(Debug, Clone, PartialEq)]
pub struct DiffCheck {
    /// نقاطی که هر سه مقدار (f، f′، تفاضل عددی) متناهی بودند.
    pub valid_points: usize,
    /// بدترین خطای نسبی روی نقاط معتبر.
    pub worst_rel_err: f64,
}

/// پذیرش: خطای نسبی ≤ TOL روی ≥ MIN_VALID نقطهٔ معتبر.
const TOL: f64 = 1e-6;
const MIN_VALID: usize = 3;

/// مشتق مرتبهٔ اول را در نقاط نمونه عددی راستی‌آزمایی می‌کند.
pub fn check_derivative(expr: &Sym, var: &str) -> Result<DiffCheck, CasError> {
    let d = diff::diff(expr, var)?;
    verify_pair(expr, &d, var)
}

/// جفت (تابع، مشتق) را می‌آزماید — زیرساخت diff-verify برای CAS-2.
pub fn verify_pair(f: &Sym, df: &Sym, var: &str) -> Result<DiffCheck, CasError> {
    let mut report = DiffCheck {
        valid_points: 0,
        worst_rel_err: 0.0,
    };
    // بذر قطعی از sexpr — تکرارپذیری کامل در CI
    let mut rng = Lcg::new(hash_str(&crate::sexpr::to_sexp(f)));
    for _ in 0..24 {
        if report.valid_points >= 6 {
            break;
        }
        let x = 0.35 + rng.next_f64() * 2.1; // دور از صفر و لبه‌ها
        let (fx, dfx) = (eval_at(f, var, x)?, eval_at(df, var, x)?);
        if !fx.is_finite() || !dfx.is_finite() {
            continue; // بیرون از دامنه — صادقانه رد
        }
        // تفاضل مرکزی دوسطحی + ریچاردسون
        let h = 1e-4;
        let (fp, fm) = (eval_at(f, var, x + h)?, eval_at(f, var, x - h)?);
        let (fp2, fm2) = (eval_at(f, var, x + h / 2.0)?, eval_at(f, var, x - h / 2.0)?);
        if !fp.is_finite() || !fm.is_finite() || !fp2.is_finite() || !fm2.is_finite() {
            continue;
        }
        let d1 = (fp - fm) / (2.0 * h);
        let d2 = (fp2 - fm2) / h;
        let rich = (4.0 * d2 - d1) / 3.0;
        if !rich.is_finite() {
            continue;
        }
        let rel = (rich - dfx).abs() / (1.0 + dfx.abs());
        report.valid_points += 1;
        report.worst_rel_err = report.worst_rel_err.max(rel);
    }
    Ok(report)
}

/// گزارش پذیرفته؟ (خطا ≤ TOL و کافی بودن شاهد)
pub fn accepted(r: &DiffCheck) -> bool {
    r.valid_points >= MIN_VALID && r.worst_rel_err <= TOL
}

pub(crate) fn eval_at(e: &Sym, var: &str, x: f64) -> Result<f64, CasError> {
    e.substitute(var, &Sym::Real(x)).to_f64()
}

/// FNV-1a — بذر قطعی.
fn hash_str(s: &str) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in s.as_bytes() {
        h ^= u64::from(*b);
        h = h.wrapping_mul(0x1000_0000_01b3);
    }
    h
}

/// LCG کوچک قطعی — همان الگو که در تست‌های طلایی نمودار استفاده شده.
struct Lcg(u64);

impl Lcg {
    fn new(seed: u64) -> Lcg {
        Lcg(seed | 1)
    }

    fn next_f64(&mut self) -> f64 {
        // رقم‌های بالا را برمی‌داریم (کیفیت کافی برای نمونه‌گیری نمونه)
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (self.0 >> 11) as f64 / (1u64 << 53) as f64
    }
}

// ============================================================
// تست‌ها
// ============================================================
#[cfg(test)]
mod tests {
    use crate::diff;
    use crate::testkit::*;
    use crate::verify;

    #[test]
    fn derivative_self_check_corpus_100pct() {
        // DoD: ۱۰۰٪ خودآزمایی — مجموعهٔ پوشش‌دهندهٔ همهٔ قواعد
        let corpus: Vec<Sym> = vec![
            sym("x"),
            pow(&sym("x"), &int(3)),
            add(&[
                times(&[int(3), pow(&sym("x"), &int(2))]),
                times(&[int(2), sym("x")]),
                int(7),
            ]),
            pow(&sym("x"), &int(-2)),
            times(&[sym("x"), f("sin", &[sym("x")])]),
            over(&sym("x"), &f("sin", &[sym("x")])).expect("t"),
            f("sin", &[sym("x")]),
            f("cos", &[times(&[int(2), sym("x")])]),
            f("tan", &[sym("x")]),
            f("asin", &[over(&sym("x"), &int(2)).expect("t")]),
            f("acos", &[over(&sym("x"), &int(3)).expect("t")]),
            f("atan", &[sym("x")]),
            f("sinh", &[sym("x")]),
            f("cosh", &[times(&[sym("x"), int(2)])]),
            f("tanh", &[sym("x")]),
            f("exp", &[sym("x")]),
            f("exp", &[neg(&pow(&sym("x"), &int(2)))]),
            times(&[
                f("exp", &[neg(&pow(&sym("x"), &int(2)))]),
                pow(&sym("x"), &int(3)),
            ]),
            f("log", &[sym("x")]),
            f("log10", &[sym("x")]),
            f("log2", &[sym("x")]),
            f("sqrt", &[sym("x")]),
            f("sqrt", &[add(&[pow(&sym("x"), &int(2)), int(1)])]),
            f("abs", &[add(&[sym("x"), int(2)])]),
            pow(&sym("x"), &sym("x")),
            pow(&int(2), &sym("x")),
            pow(&sym("x"), &rat(3, 2)),
            pow(&add(&[sym("x"), int(1)]), &int(5)),
            times(&[
                pow(&add(&[sym("x"), int(1)]), &int(2)),
                pow(&add(&[sym("x"), int(2)]), &int(3)),
            ]),
            over(&int(1), &add(&[pow(&sym("x"), &int(2)), int(1)])).expect("t"),
            add(&[
                f("sin", &[sym("x")]),
                times(&[int(3), f("cos", &[sym("x")])]),
            ]),
            times(&[rat(1, 3), pow(&sym("x"), &int(4))]),
            pow(&f("sin", &[sym("x")]), &int(2)),
            f("log", &[add(&[pow(&sym("x"), &int(2)), int(1)])]),
        ];
        let mut failures: Vec<String> = Vec::new();
        for e in &corpus {
            let d = diff::diff(e, "x").expect("t");
            let rep = verify::verify_pair(e, &d, "x").expect("t");
            if !verify::accepted(&rep) {
                failures.push(format!(
                    "f={} → d={} : valid={} worst={:.3e}",
                    e.display(),
                    d.display(),
                    rep.valid_points,
                    rep.worst_rel_err
                ));
            }
        }
        assert!(
            failures.is_empty(),
            "خودآزمایی مشتق شکست خورد:\n{}",
            failures.join("\n")
        );
    }

    #[test]
    fn out_of_domain_reported_not_guessed() {
        // sqrt(x) مشتقش در نزدیکی 0 بدرفتار است اما نمونه‌ها ≥0.35 سالم‌اند؛
        // یک تابع همیشه-ناکامل: log(x-10) در بازهٔ نمونه → بدون شاهد معتبر
        let e = f("log", &[add(&[sym("x"), int(-10)])]);
        let d = diff::diff(&e, "x").expect("t");
        let rep = verify::verify_pair(&e, &d, "x").expect("t");
        assert!(rep.valid_points < 3);
        assert!(!verify::accepted(&rep));
    }
}
