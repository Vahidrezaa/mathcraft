//! fzero — روش Brent با جست‌وجوی براکت خودکار (سند ۰۴ §5.1).
//!
//! - Brent = دوبخشی + امتیانی + درجه‌۲ وارون — همگرایی سوپرخطی با ضمانت براکت.
//! - معیار توقف: `|Δx| ≤ 2*eps*|x| + tol` با tol پیش‌فرض `eps`؛ حداکثر ۱۰۰۰ تکرار.
//! - نقطهٔ شروع اسکالر → جست‌وجوی براکت با گسترش هندسی ×۱.۷ (الگوی متلب)؛
//!   اگر براکت نیافت → خطای صادقانه «sign change not found».
//! - سقف فراخوانی تابع کاربر (فلاپ‌سنج) — از سرریز بودجهٔ حلگرها جلوگیری می‌کند.

use mc_core::{ids, McError};

/// خطای استاندارد حلگرها — لایهٔ interp موقعیت AST را اضافه می‌کند.
pub fn solver_error(id: &'static str, en: impl Into<String>, fa: impl Into<String>) -> McError {
    McError::new(id, 0, 0, en, fa)
}

/// نتیجهٔ fzero.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BrentResult {
    pub x: f64,
    pub fx: f64,
    pub iterations: u32,
    pub fcalls: u32,
}

const MAX_ITER: u32 = 1000;
const MAX_FCALLS: u32 = 100_000;

/// Brent با براکت داده‌شده — پیش‌نیاز: `f(a)·f(b) ≤ 0` (بررسی می‌شود).
pub fn fzero_bracket<F>(f: &mut F, mut a: f64, mut b: f64, tol: f64) -> Result<BrentResult, McError>
where
    F: FnMut(f64) -> Result<f64, McError>,
{
    let mut fcalls = 0u32;
    let mut fa = f(a)?;
    fcalls += 1;
    let mut fb = f(b)?;
    fcalls += 1;
    if fa == 0.0 {
        return Ok(BrentResult { x: a, fx: fa, iterations: 0, fcalls });
    }
    if fb == 0.0 {
        return Ok(BrentResult { x: b, fx: fb, iterations: 0, fcalls });
    }
    if fa.signum() == fb.signum() {
        return Err(solver_error(
            ids::PARSE_ERROR,
            format!(
                "fzero: endpoints must bracket a sign change (f(a) = {fa}, f(b) = {fb})."
            ),
            format!(
                "fzero: دو سر بازه باید تغییر علامت داشته باشند (f(a) = {fa}، f(b) = {fb})."
            ),
        ));
    }

    // متغیرهای Brent (قالب zbrent — Numerical Recipes)
    let (mut c, mut fc, mut d, mut e) = (b, fb, b - a, b - a);
    let mut iterations = 0u32;
    for _ in 0..MAX_ITER {
        iterations += 1;
        if fb.signum() == fc.signum() {
            c = a;
            fc = fa;
            d = b - a;
            e = b - a;
        }
        if fc.abs() < fb.abs() {
            a = b;
            b = c;
            c = a;
            fa = fb;
            fb = fc;
            fc = fa;
        }
        // معیار توقف سند ۰۴ §5.1
        let tol1 = 2.0 * f64::EPSILON * b.abs() + 0.5 * tol;
        let xm = 0.5 * (c - b);
        if xm.abs() <= tol1 || fb == 0.0 {
            return Ok(BrentResult { x: b, fx: fb, iterations, fcalls });
        }
        if e.abs() >= tol1 && fa.abs() > fb.abs() {
            // تلاش درجه‌۲ وارون / امتیانی
            let s = fb / fa;
            let (p, q) = if a == c {
                (2.0 * xm * s, 1.0 - s)
            } else {
                let q0 = fa / fc;
                let r = fb / fc;
                (s * (2.0 * xm * q0 * (q0 - r) - (b - a) * (r - 1.0)),
                 (q0 - 1.0) * (r - 1.0) * (s - 1.0))
            };
            let q = if p > 0.0 { -q } else { q };
            let p = p.abs();
            let min1 = 3.0 * xm * q - (tol1 * q).abs();
            let min2 = (e * q).abs();
            if 2.0 * p < min1.min(min2) {
                e = d;
                d = p / q;
            } else {
                d = xm;
                e = d;
            }
        } else {
            d = xm;
            e = d;
        }
        // جابه‌جایی براکت
        a = b;
        fa = fb;
        b += if d.abs() > tol1 { d } else { if d > 0.0 { tol1 } else { -tol1 } };
        fb = f(b)?;
        fcalls += 1;
        if fcalls > MAX_FCALLS {
            return Err(solver_error(
                ids::PARSE_ERROR,
                "fzero: function call budget exceeded.",
                "fzero: سقف فراخوانی تابع پر شد.",
            ));
        }
    }
    Err(solver_error(
        ids::PARSE_ERROR,
        format!("fzero: did not converge in {MAX_ITER} iterations."),
        format!("fzero: در {MAX_ITER} تکرار همگرا نشد."),
    ))
}

/// fzero با نقطهٔ شروع اسکالر — جست‌وجوی براکت خودکار با گسترش ×۱.۷.
pub fn fzero_scalar<F>(f: &mut F, x0: f64, tol: f64) -> Result<BrentResult, McError>
where
    F: FnMut(f64) -> Result<f64, McError>,
{
    let f0 = f(x0)?;
    if f0 == 0.0 {
        return Ok(BrentResult { x: x0, fx: f0, iterations: 0, fcalls: 1 });
    }
    // الگوی متلب: گسترش هندسی از نقاط مجاور — ۶۰ گام ×۱.۷ (~۰.۱ تا ۱e13 برابر)
    let mut dx = (x0.abs() * 0.01).max(0.1);
    for _ in 0..60 {
        for &cand in &[x0 - dx, x0 + dx] {
            let fc = f(cand)?;
            if fc == 0.0 {
                return Ok(BrentResult { x: cand, fx: fc, iterations: 0, fcalls: 1 });
            }
            if fc.signum() != f0.signum() {
                return fzero_bracket(f, x0, cand, tol);
            }
        }
        dx *= 1.7;
    }
    Err(solver_error(
        ids::PARSE_ERROR,
        format!(
            "fzero: cannot detect a sign change around x0 = {x0}; provide a bracket [a b] \
             with f(a)·f(b) < 0."
        ),
        format!(
            "fzero: تغییر علامت اطراف x0 = {x0} پیدا نشد؛ براکت [a b] با f(a)·f(b) < 0 بدهید."
        ),
    )
    .with_suggestion("try a bracket: fzero(f, [2 4])"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64, tol: f64) -> bool {
        (a - b).abs() <= tol
    }

    #[test]
    fn brent_simple_polynomial() {
        // x² − 2 = 0 در [0, 2] → √2
        let mut f = |x: f64| Ok::<f64, McError>(x * x - 2.0);
        let r = fzero_bracket(&mut f, 0.0, 2.0, f64::EPSILON).unwrap_or_else(|e| panic!("{e:?}"));
        assert!(close(r.x, std::f64::consts::SQRT_2, 1e-12), "x = {}", r.x);
        assert!(close(r.fx, 0.0, 1e-12));
        // براکت بدون تغییر علامت (هر دو مثبت روی [2,3]) → خطای صادقانه
        assert!(fzero_bracket(&mut f, 2.0, 3.0, f64::EPSILON).is_err());
    }

    #[test]
    fn scalar_start_auto_bracket() {
        // cos(x) = 0 نزدیک ۱.۵ → π/2
        let mut f = |x: f64| Ok::<f64, McError>(x.cos());
        let r = fzero_scalar(&mut f, 1.5, f64::EPSILON).unwrap();
        assert!(close(r.x, std::f64::consts::FRAC_PI_2, 1e-12), "x = {}", r.x);
        // بدون تغییر علامت اطراف: x² + 1
        let mut g = |x: f64| Ok::<f64, McError>(x * x + 1.0);
        assert!(fzero_scalar(&mut g, 5.0, f64::EPSILON).is_err());
    }

    #[test]
    fn brent_on_transcendental() {
        // x·e^x − 1 = 0 → جواب Омега ≈ 0.567143290409784
        let mut f = |x: f64| Ok::<f64, McError>(x * x.exp() - 1.0);
        let r = fzero_bracket(&mut f, 0.0, 1.0, f64::EPSILON).unwrap();
        assert!(close(r.x, 0.567143290409784, 1e-13), "x = {}", r.x);
    }
}
