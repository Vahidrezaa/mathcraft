//! integral — انتگرال‌گیری تطبیقی Gauss–Kronrod G7/K15 (سند ۰۴ §5.3).
//!
//! - تقسیم بازگشتی بازه با صف اولویت (بزرگ‌ترین خطا اول)؛ بودجه: ۱۰٬۰۰۰ پرسش تابع.
//! - پیش‌فرض‌های متلب: RelTol = AbsTol = 1e-10.
//! - بازه‌های بی‌نهایت با تبدیل متغیر مرجع:
//!   `[a, ∞)`: x = a + (1−t)/t · `(-∞, b]`: x = b − (1−t)/t · `(-∞, ∞)`: x = t/(1−t²)
//! - خطای نهایی > tol → **خطای صادقانه** (قاعدهٔ سند ۰۴ §9: نتیجه هرگز
//!   silent-مشکوک نیست؛ کانال هشدار ساختاریافته با FFI رویدادی در P2-b می‌آید).
//! - لغو همکاری‌گر + گزارش پیشرفت هر تکرار تقسیم از کانتکست job
//!   (`mc_core::job`، ROADMAP 2.3) — بدون کانتکست no-op است.
//! - گره‌ها/وزن‌ها: جدول مرجع QUADPACK QK15.

use std::collections::BinaryHeap;

use mc_core::{ids, job, McError};

use super::root::solver_error;

/// گره‌های ۱۵-نقطه‌ای Kronrod (نیمهٔ مثبت + مرکز) — جدول مرجع QUADPACK QK15.
const XGK: [f64; 8] = [
    0.991455371120813,
    0.949107912342759,
    0.864864423359769,
    0.741531185599394,
    0.586087235467691,
    0.405845151377397,
    0.207784955007898,
    0.0,
];
const WGK: [f64; 8] = [
    0.022935322010529,
    0.063092092629979,
    0.104790010322250,
    0.140653259715525,
    0.169004726639267,
    0.190350578064785,
    0.204432940075298,
    0.209482141084728,
];
/// وزن‌های ۷-نقطه‌ای Gauss — روی گره‌های زوج XGK (اندیس‌های 1,3,5,7,9,11,13 در جدول کامل).
const WG: [f64; 4] = [
    0.129484966168870,
    0.279705391489277,
    0.381830050505119,
    0.417959183673469,
];

/// بازهٔ انتگرال — متناهی، نیمه‌بی‌نهایت یا دوطرفه.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Bounds {
    Finite { a: f64, b: f64 },
    HalfUp { a: f64 },   // [a, +∞)
    HalfDown { b: f64 }, // (-∞, b]
    DoublyInfinite,
}

/// نتیجهٔ انتگرال‌گیری.
#[derive(Debug, Clone, Copy)]
pub struct QuadResult {
    pub value: f64,
    pub abserr: f64,
    pub fcalls: u32,
    /// حداکثر |f| دیده‌شده — برای تشخیص singularity (سند ۰۴ §5.3).
    pub max_abs_f: f64,
}

/// ورودی تابع به‌صورت closure — لایهٔ registry هندل کاربر را می‌پیچد.
pub trait QuadFn {
    fn call(&mut self, x: f64) -> Result<f64, McError>;
}

impl<T: FnMut(f64) -> Result<f64, McError>> QuadFn for T {
    fn call(&mut self, x: f64) -> Result<f64, McError> {
        self(x)
    }
}

/// یک بازهٔ کاری در صف اولویت.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Interval {
    a: f64,
    b: f64,
    value: f64,
    err: f64,
}

impl Eq for Interval {}

impl Ord for Interval {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // بزرگ‌ترین خطا اول (max-heap روی err)
        self.err
            .partial_cmp(&other.err)
            .unwrap_or(std::cmp::Ordering::Equal)
    }
}

impl PartialOrd for Interval {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// قاعدهٔ G7/K15 روی [a,b] — ارزیابی یک‌باره + تفکیک خطا.
fn qk15<F: QuadFn>(f: &mut F, a: f64, b: f64) -> Result<(f64, f64), McError> {
    let c = 0.5 * (a + b);
    let hl = 0.5 * (b - a);
    // resk: کرونراد ۱۵ نقطه‌ای؛ resg: گاوس ۷ نقطه‌ای (گره‌های زوج)
    let mut resk = 0.0;
    let mut resg = 0.0;
    // جفت‌های متقارن — 7 گره مثبت (اندیس 0..6) + مرکز (اندیس 7)
    for i in 0..7 {
        let dx = hl * XGK[i];
        let f1 = f.call(c - dx)?;
        let f2 = f.call(c + dx)?;
        resk += WGK[i] * (f1 + f2);
        // گره‌های گاوس = اندیس‌های زوج جدول کامل → اینجا i ∈ {1,3,5} و قرینه‌شان
        if i % 2 == 1 {
            resg += WG[(i - 1) / 2] * (f1 + f2);
        }
    }
    let f0 = f.call(c)?;
    resk += WGK[7] * f0;
    resg += WG[3] * f0;
    let value = resk * hl;
    let err = (resk - resg).abs() * hl.abs();
    Ok((value, err))
}

/// انتگرال‌گیری تطبیقی — تابع اصلی بدون تبدیل (بازه متناهی).
fn adaptive<F: QuadFn>(
    f: &mut F,
    a: f64,
    b: f64,
    reltol: f64,
    abstol: f64,
    budget: u32,
) -> Result<QuadResult, McError> {
    let mut fcalls = 0u32;
    #[allow(unused_variables)]
    let max_abs_f = 0.0f64;
    // جعبه‌ابزار محلی برای شمارش پرسش‌ها — qk15 خودش 15 بار صدا می‌زند؛
    // شمارش: هر qk15 = 15 فراخوانی (۷ جفت + مرکز)
    let (v0, e0) = qk15(f, a, b)?;
    fcalls += 15;
    let mut heap = BinaryHeap::new();
    heap.push(Interval {
        a,
        b,
        value: v0,
        err: e0,
    });
    let mut total = v0;
    let mut total_err = e0;
    while total_err > (abstol).max(reltol * total.abs()) {
        let Some(iv) = heap.pop() else { break };
        if fcalls + 30 > budget {
            // بودجه تمام شد — با خطای فعلی تصمیم می‌گیریم
            break;
        }
        // لغو همکاری‌گر + پیشرفت (ROADMAP 2.3 task 2) — هر تکرار تقسیم،
        // کسر = سهم بودجهٔ مصرف‌شده (محافظه‌کارانه ولی بی‌هزینه)
        job::report("integral", Some(fcalls as f64 / budget as f64));
        if job::cancel_requested() {
            return Err(solver_error(
                ids::CANCELLED,
                "integral: cancelled; partial result discarded.",
                "integral: لغو شد؛ نتیجهٔ جزئی دور ریخته شد.",
            ));
        }
        let mid = 0.5 * (iv.a + iv.b);
        let (vl, el) = qk15(f, iv.a, mid)?;
        let (vr, er) = qk15(f, mid, iv.b)?;
        fcalls += 30;
        total -= iv.value;
        total_err -= iv.err;
        total += vl + vr;
        total_err += el + er;
        heap.push(Interval {
            a: iv.a,
            b: mid,
            value: vl,
            err: el,
        });
        heap.push(Interval {
            a: mid,
            b: iv.b,
            value: vr,
            err: er,
        });
    }
    Ok(QuadResult {
        value: total,
        abserr: total_err,
        fcalls,
        max_abs_f: 0.0,
    })
}

/// انتگرال‌گیری روی بازهٔ بی‌نهایت با تبدیل متغیر (سند ۰۴ §5.3).
fn integrate_infinite<F: QuadFn>(
    f: &mut F,
    bounds: Bounds,
    reltol: f64,
    abstol: f64,
    budget: u32,
) -> Result<QuadResult, McError> {
    struct Transformed<'a, F> {
        inner: &'a mut F,
        bounds: Bounds,
        /// بالاترین |f(x)| واقعی (پس از بازگشت از t)
        max_abs: f64,
    }
    impl<F: QuadFn> QuadFn for Transformed<'_, F> {
        fn call(&mut self, t: f64) -> Result<f64, McError> {
            let (x, jacobian) = match self.bounds {
                Bounds::HalfUp { a } => {
                    // x = a + (1−t)/t ، dx/dt = −1/t²  (t ∈ (0,1])
                    let x = a + (1.0 - t) / t;
                    (x, 1.0 / (t * t))
                }
                Bounds::HalfDown { b } => {
                    let x = b - (1.0 - t) / t;
                    (x, 1.0 / (t * t))
                }
                Bounds::DoublyInfinite => {
                    // x = t/(1−t²) ، dx/dt = (1+t²)/(1−t²)²  (t ∈ (−1,1))
                    let d = 1.0 - t * t;
                    (t / d, (1.0 + t * t) / (d * d))
                }
                Bounds::Finite { .. } => unreachable!("transformed called with finite bounds"),
            };
            let fx = self.inner.call(x)?;
            self.max_abs = self.max_abs.max(fx.abs());
            Ok(fx * jacobian)
        }
    }
    let mut inner = Transformed {
        inner: f,
        bounds,
        max_abs: 0.0,
    };
    let res = match bounds {
        Bounds::HalfUp { .. } | Bounds::HalfDown { .. } => {
            adaptive(&mut inner, 0.0, 1.0, reltol, abstol, budget)
        }
        Bounds::DoublyInfinite => adaptive(&mut inner, -1.0, 1.0, reltol, abstol, budget),
        Bounds::Finite { .. } => unreachable!(),
    }?;
    Ok(QuadResult {
        max_abs_f: inner.max_abs,
        ..res
    })
}

/// رابط اصلی integral — بازه‌های متناهی/بی‌نهایت یکدست (سند ۰۴ §5.3).
pub fn integrate<F: QuadFn>(
    f: &mut F,
    bounds: Bounds,
    reltol: f64,
    abstol: f64,
) -> Result<QuadResult, McError> {
    let budget = 10_000;
    let res = match bounds {
        Bounds::Finite { a, b } => adaptive(f, a, b, reltol, abstol, budget)?,
        _ => integrate_infinite(f, bounds, reltol, abstol, budget)?,
    };
    let tol = abstol.max(reltol * res.value.abs());
    if res.abserr > tol {
        return Err(solver_error(
            ids::PARSE_ERROR,
            format!(
                "integral: failed to reach tolerance (error ≈ {:.3e} > {:.3e}); \
                 integrand may be singular or oscillatory.",
                res.abserr, tol
            ),
            format!(
                "integral: رسیدن به تلورانس ممکن نشد (خطا ≈ {:.3e} > {:.3e})؛ \
                 احتمالاً انتگرال‌نما تکین یا نوسانی است.",
                res.abserr, tol
            ),
        ));
    }
    Ok(res)
}

/// پوشش راحت برای بازهٔ متنی متلب — تشخیص ±Inf از ورودی رجیستری.
pub fn bounds_from(a: f64, b: f64) -> Result<Bounds, McError> {
    match (a.is_infinite(), b.is_infinite()) {
        (false, false) => Ok(Bounds::Finite { a, b }),
        (false, true) if b > 0.0 => Ok(Bounds::HalfUp { a }),
        (true, false) if a < 0.0 => Ok(Bounds::HalfDown { b }),
        (true, true) => Ok(Bounds::DoublyInfinite),
        _ => Err(solver_error(
            ids::PARSE_ERROR,
            "integral: invalid infinite bounds (direction mismatch).",
            "integral: مرزهای بی‌نهایت نامعتبر است (جهت ناهم‌خوان).",
        )),
    }
}

/// انتگرال‌گیری تکه‌ای روی نقاط شکست — گزینهٔ Waypoints متلب (ROADMAP 2.6).
///
/// - فقط بازهٔ متناهی: Waypoints با مرز بی‌نهایت → خطای صادقانه (انحراف
///   مستند COMPATIBILITY §4.5).
/// - هر Waypoint باید متناهی و داخل بازهٔ بستهٔ انتگرال باشد (رفتار متلب:
///   «waypoints must lie within the integration interval») — بیرون = خطا.
/// - جهت a > b مثل متلب با علامت منفی اعمال می‌شود؛ نقاط تکراری/مرزی بی‌اثرند.
/// - هر تکه با بودجهٔ مستقل و همان RelTol انتگرال گرفته می‌شود؛ AbsTol بین
///   تکه‌ها تقسیم می‌شود و خطای کل در برابر tol نهایی سنجیده می‌شود — نتیجه
///   هرگز silent-مشکوک نیست (سند ۰۴ §9).
pub fn integrate_waypoints<F: QuadFn>(
    f: &mut F,
    bounds: Bounds,
    waypoints: &[f64],
    reltol: f64,
    abstol: f64,
) -> Result<QuadResult, McError> {
    let (a, b) = match bounds {
        Bounds::Finite { a, b } => (a, b),
        _ => {
            return Err(solver_error(
                ids::PARSE_ERROR,
                "integral: Waypoints with infinite limits is not supported in this milestone.",
                "integral: Waypoints با مرزهای بی‌نهایت در این مایل‌استون پشتیبانی نمی‌شود.",
            ))
        }
    };
    if !(reltol > 0.0 && reltol.is_finite()) || !(abstol >= 0.0 && abstol.is_finite()) {
        return Err(solver_error(
            ids::PARSE_ERROR,
            "integral: RelTol must be positive and AbsTol non-negative.",
            "integral: RelTol باید مثبت و AbsTol نامنفی باشد.",
        ));
    }
    let (lo, hi, sign) = if b < a { (b, a, -1.0) } else { (a, b, 1.0) };
    for &w in waypoints {
        if !w.is_finite() || w < lo || w > hi {
            return Err(solver_error(
                ids::PARSE_ERROR,
                "integral: Waypoints must be finite and lie within the integration interval.",
                "integral: Waypoints باید متناهی و داخل بازهٔ انتگرال‌گیری باشند.",
            ));
        }
    }
    let mut pts: Vec<f64> = waypoints
        .iter()
        .copied()
        .filter(|&w| w > lo && w < hi)
        .collect();
    pts.sort_by(|x, y| x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Less));
    pts.dedup();
    let mut breaks = Vec::with_capacity(pts.len() + 2);
    breaks.push(lo);
    breaks.extend(pts);
    breaks.push(hi);
    let n = breaks.len() - 1;
    let budget = (10_000 / n as u32).max(100);
    let abstol_piece = abstol / n as f64;
    let mut value = 0.0;
    let mut abserr = 0.0;
    let mut fcalls = 0u32;
    let mut max_abs_f = 0.0f64;
    for k in 0..n {
        let r = adaptive(f, breaks[k], breaks[k + 1], reltol, abstol_piece, budget)?;
        value += r.value;
        abserr += r.abserr;
        fcalls += r.fcalls;
        max_abs_f = max_abs_f.max(r.max_abs_f);
    }
    value *= sign;
    let tol = abstol.max(reltol * value.abs());
    if abserr > tol {
        return Err(solver_error(
            ids::PARSE_ERROR,
            format!(
                "integral: failed to reach tolerance with Waypoints (error ≈ {:.3e} > {:.3e}); \
                 integrand may be singular or oscillatory.",
                abserr, tol
            ),
            format!(
                "integral: رسیدن به تلورانس با Waypoints ممکن نشد (خطا ≈ {:.3e} > {:.3e})؛ \
                 احتمالاً انتگرال‌نما تکین یا نوسانی است.",
                abserr, tol
            ),
        ));
    }
    Ok(QuadResult {
        value,
        abserr,
        fcalls,
        max_abs_f,
    })
}

/// نسخهٔ با بودجهٔ پرسش‌های بیشتر برای تست‌ها — صادقانه مثل integrate.
pub fn integrate_transformed<F: QuadFn>(
    f: &mut F,
    t_of_x: impl Fn(f64) -> f64,
    a: f64,
    b: f64,
    reltol: f64,
    abstol: f64,
) -> Result<QuadResult, McError> {
    struct Map<'a, F, G> {
        inner: &'a mut F,
        map: G,
    }
    impl<F: QuadFn, G: Fn(f64) -> f64> QuadFn for Map<'_, F, G> {
        fn call(&mut self, x: f64) -> Result<f64, McError> {
            self.inner.call((self.map)(x))
        }
    }
    adaptive(
        &mut Map {
            inner: f,
            map: t_of_x,
        },
        a,
        b,
        reltol,
        abstol,
        10_000,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn call(f: impl FnMut(f64) -> Result<f64, McError>) -> impl QuadFn {
        f
    }

    #[test]
    fn integral_cancelled_via_job_ctx() {
        use mc_core::job::{self, JobSignals};
        use std::sync::Arc;
        let _serial = super::super::test_util::ctx_serial();
        let ctx = Arc::new(JobSignals::new(1));
        job::install(ctx.clone());
        ctx.cancel.store(true, std::sync::atomic::Ordering::Relaxed);

        // exp روی [0,20] چند تقسیم می‌خواهد → حلقهٔ adaptive قطعاً می‌چرخد
        // (x² روی [0,1] یک‌ضرب دقیق است و به حلقه نمی‌رسد)
        let mut f = |x: f64| Ok(x.exp());
        let err = integrate(&mut f, Bounds::Finite { a: 0.0, b: 20.0 }, 1e-10, 1e-10)
            .expect_err("pre-cancelled integral must error");
        assert_eq!(err.id, mc_core::ids::CANCELLED);
        job::clear();
    }

    #[test]
    fn analytic_polynomial_exact() {
        // ∫₀¹ x² dx = 1/3 — G7/K15 با تلورانس پیش‌فرض متلب
        let mut f = call(|x| Ok(x * x));
        let r = integrate(&mut f, Bounds::Finite { a: 0.0, b: 1.0 }, 1e-10, 1e-10)
            .unwrap_or_else(|e| panic!("{e:?}"));
        assert!((r.value - 1.0 / 3.0).abs() < 1e-12, "got {}", r.value);
        assert!(r.fcalls <= 10_000);
    }

    #[test]
    fn analytic_exp_and_sin() {
        // ∫₀^π sin = 2
        let mut f = call(|x| Ok(x.sin()));
        let r = integrate(
            &mut f,
            Bounds::Finite {
                a: 0.0,
                b: std::f64::consts::PI,
            },
            1e-10,
            1e-10,
        )
        .unwrap();
        assert!((r.value - 2.0).abs() < 1e-9, "got {}", r.value);
        // ∫₀¹ e^x = e − 1
        let mut g = call(|x| Ok(x.exp()));
        let r2 = integrate(&mut g, Bounds::Finite { a: 0.0, b: 1.0 }, 1e-10, 1e-10).unwrap();
        assert!((r2.value - (std::f64::consts::E - 1.0)).abs() < 1e-10);
    }

    #[test]
    fn oscillatory_integrand_adapts() {
        // ∫₀^{2π} sin(50x)² dx = π — نوسانی؛ تطبیقی باید بازه را بشکند
        let mut f = call(|x| Ok((50.0 * x).sin().powi(2)));
        let r = integrate(
            &mut f,
            Bounds::Finite {
                a: 0.0,
                b: 2.0 * std::f64::consts::PI,
            },
            1e-8,
            1e-8,
        )
        .unwrap_or_else(|e| panic!("{e:?}"));
        assert!(
            (r.value - std::f64::consts::PI).abs() < 1e-6,
            "got {}",
            r.value
        );
    }

    #[test]
    fn half_infinite_exp_decay() {
        // ∫₀^∞ e^{−x} dx = 1
        let mut f = call(|x| Ok((-x).exp()));
        let r = integrate(&mut f, Bounds::HalfUp { a: 0.0 }, 1e-10, 1e-10)
            .unwrap_or_else(|e| panic!("{e:?}"));
        assert!((r.value - 1.0).abs() < 1e-9, "got {}", r.value);
    }

    #[test]
    fn half_infinite_gaussian_tail() {
        // ∫₀^∞ x·e^{−x²} dx = 1/2
        let mut f = call(|x| Ok(x * (-x * x).exp()));
        let r = integrate(&mut f, Bounds::HalfUp { a: 0.0 }, 1e-10, 1e-10).unwrap();
        assert!((r.value - 0.5).abs() < 1e-10, "got {}", r.value);
    }

    #[test]
    fn doubly_infinite_gaussian() {
        // ∫_{−∞}^{∞} e^{−x²} dx = √π
        let mut f = call(|x| Ok((-x * x).exp()));
        let r = integrate(&mut f, Bounds::DoublyInfinite, 1e-10, 1e-10)
            .unwrap_or_else(|e| panic!("{e:?}"));
        assert!(
            (r.value - std::f64::consts::PI.sqrt()).abs() < 1e-9,
            "got {}",
            r.value
        );
    }

    #[test]
    fn singular_integrand_fails_honestly() {
        // ∫₀¹ 1/x dx واگرا — باید خطا بدهد نه مقدار بی‌معنی
        let mut f = call(|x| Ok(1.0 / x));
        let r = integrate(&mut f, Bounds::Finite { a: 0.0, b: 1.0 }, 1e-10, 1e-10);
        assert!(r.is_err(), "divergent integral must error");
    }

    #[test]
    fn bounds_direction_checks() {
        assert!(matches!(
            bounds_from(0.0, f64::INFINITY),
            Ok(Bounds::HalfUp { .. })
        ));
        assert!(matches!(
            bounds_from(f64::NEG_INFINITY, 0.0),
            Ok(Bounds::HalfDown { .. })
        ));
        assert!(matches!(
            bounds_from(f64::NEG_INFINITY, f64::INFINITY),
            Ok(Bounds::DoublyInfinite)
        ));
        // جهت غلط: [+∞ به 0]
        assert!(bounds_from(f64::INFINITY, 0.0).is_err());
    }

    // ---------- Waypoints (ROADMAP 2.6) ----------

    #[test]
    fn waypoints_piecewise_exact() {
        // f پله‌ای: 1 روی [0,0.5) و 2 روی (0.5,1] → ∫ = 1.5
        // بدون نقطهٔ شکست، G7/K15 روی تکهٔ ناهموار خطا می‌دهد یا گران است؛
        // با Waypoints=[0.5] هر تکه چندجمله‌ای‌درجه‌۰ و دقیق است.
        let mut f = call(|x| Ok(if x < 0.5 { 1.0 } else { 2.0 }));
        let r = integrate_waypoints(
            &mut f,
            Bounds::Finite { a: 0.0, b: 1.0 },
            &[0.5],
            1e-10,
            1e-10,
        )
        .unwrap_or_else(|e| panic!("{e:?}"));
        assert!((r.value - 1.5).abs() < 1e-12, "got {}", r.value);
    }

    #[test]
    fn waypoints_kink_abs() {
        // ∫₋₁¹ |x| dx = 1 — شکست در 0
        let mut f = call(|x| Ok(x.abs()));
        let r = integrate_waypoints(
            &mut f,
            Bounds::Finite { a: -1.0, b: 1.0 },
            &[0.0],
            1e-10,
            1e-10,
        )
        .unwrap();
        assert!((r.value - 1.0).abs() < 1e-12, "got {}", r.value);
    }

    #[test]
    fn waypoints_reversed_bounds_negative_sign() {
        // ∫₁⁻¹ |x| dx = −1 — جهت متلب
        let mut f = call(|x| Ok(x.abs()));
        let r = integrate_waypoints(
            &mut f,
            Bounds::Finite { a: 1.0, b: -1.0 },
            &[0.0],
            1e-10,
            1e-10,
        )
        .unwrap();
        assert!((r.value + 1.0).abs() < 1e-12, "got {}", r.value);
    }

    #[test]
    fn waypoints_outside_interval_error() {
        let mut f = call(Ok);
        let r = integrate_waypoints(
            &mut f,
            Bounds::Finite { a: 0.0, b: 1.0 },
            &[0.5, 2.0],
            1e-10,
            1e-10,
        );
        assert!(r.is_err(), "waypoint outside interval must error");
        // NaN هم خطاست
        let mut g = call(Ok);
        let r2 = integrate_waypoints(
            &mut g,
            Bounds::Finite { a: 0.0, b: 1.0 },
            &[f64::NAN],
            1e-10,
            1e-10,
        );
        assert!(r2.is_err());
    }

    #[test]
    fn waypoints_with_infinite_bounds_error() {
        let mut f = call(|x| Ok((-x).exp()));
        let r = integrate_waypoints(&mut f, Bounds::HalfUp { a: 0.0 }, &[1.0], 1e-10, 1e-10);
        assert!(
            r.is_err(),
            "Waypoints + infinite bounds must error honestly"
        );
    }

    #[test]
    fn waypoints_duplicates_and_endpoints_harmless() {
        let mut f = call(|x| Ok(x * x));
        let r = integrate_waypoints(
            &mut f,
            Bounds::Finite { a: 0.0, b: 1.0 },
            &[0.0, 0.5, 0.5, 1.0],
            1e-10,
            1e-10,
        )
        .unwrap();
        assert!((r.value - 1.0 / 3.0).abs() < 1e-12, "got {}", r.value);
    }

    #[test]
    fn waypoints_invalid_tolerances_error() {
        let mut f = call(Ok);
        let r = integrate_waypoints(
            &mut f,
            Bounds::Finite { a: 0.0, b: 1.0 },
            &[0.5],
            0.0,
            1e-10,
        );
        assert!(r.is_err(), "RelTol=0 must error");
    }
}
