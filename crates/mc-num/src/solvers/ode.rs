//! ODE — ode45 (Dormand–Prince 5(4)) و ode23 (Bogacki–Shampine 3(2)) — سند ۰۴ §4.
//!
//! قراردادهای متلب (سند ۰۴ §4.1):
//! - کنترل خطا: نُرم RMS نسبت به `max(RelTol·|y|, AbsTol)`؛ گام جدید:
//!   `h *= 0.9·(1/err)^{1/p}` با سقف رشد ۵× و کف ۰.۲×.
//! - پیش‌فرض‌ها: RelTol=1e-3، AbsTol=1e-6، MaxStep=(tEnd−tStart)/10.
//! - گام اول با هیوریستیک Hairer (پروب اویلر دوبل).
//! - FSAL: DP54 مرحلهٔ هفتم (c=1) و BS32 مرحلهٔ چهارم — f گام بعدی رایگان.
//! - tspan دو عنصری → همه گام‌های پذیرفته‌شده؛ tspan چندعنصری → خروجی دقیق
//!   روی نقاط خواسته‌شده با درون‌یابی Hermite مکعبی بین گام‌های پذیرفته‌شده
//!   (انحراف مستند از درون‌یابی درجه-۴ DP — قرارداد dense-output کامل P2-b).
//! - Cancel همکاری‌گر: هر ۵۰ گام پرچم `AtomicBool` چک می‌شود (سند ۰۲ §3)؛
//!   خطا پیش از انتساب برمی‌گردد → workspace consistent (DoD فاز ۲).

use std::sync::atomic::{AtomicBool, Ordering};

use mc_core::{ids, McError};

use super::root::solver_error;

/// گزینه‌های odeset پشتیبانی‌شده (سند ۰۴ §4.1: RelTol/AbsTol/MaxStep/InitialStep).
#[derive(Debug, Clone, Copy)]
pub struct OdeOpts {
    pub reltol: f64,
    pub abstol: f64,
    pub max_step: f64,
    pub initial_step: Option<f64>,
}

impl Default for OdeOpts {
    fn default() -> Self {
        OdeOpts {
            reltol: 1e-3,
            abstol: 1e-6,
            max_step: f64::INFINITY, // موقعیت با (t1−t0)/10 پر می‌شود
            initial_step: None,
        }
    }
}

/// نتیجهٔ حلگر — مقادیر t و y (هر ردیف neq مؤلفه؛ مثل ستون‌های Y متلب).
#[derive(Debug, Clone)]
pub struct OdeResult {
    pub t: Vec<f64>,
    /// y[k][j] = مؤلفه j در زمان t[k]
    pub y: Vec<Vec<f64>>,
    pub n_f_evals: u32,
    pub n_accepted: u32,
    pub n_rejected: u32,
}

/// تابع سمت راست — لایهٔ registry هندل کاربر را می‌پیچد.
pub trait OdeFn {
    fn call(&mut self, t: f64, y: &[f64]) -> Result<Vec<f64>, McError>;
}

impl<T: FnMut(f64, &[f64]) -> Result<Vec<f64>, McError>> OdeFn for T {
    fn call(&mut self, t: f64, y: &[f64]) -> Result<Vec<f64>, McError> {
        self(t, y)
    }
}

/// نُرم RMS خطا: sqrt(mean((e/scale)²)) — قرارداد متلب.
fn err_norm(e: &[f64], y0: &[f64], y1: &[f64], reltol: f64, abstol: f64) -> f64 {
    let mut acc = 0.0;
    for i in 0..e.len() {
        let scale = abstol + reltol * y0[i].abs().max(y1[i].abs());
        acc += (e[i] / scale).powi(2);
    }
    (acc / e.len() as f64).sqrt()
}

/// هیوریستیک گام اول Hairer (پروب اویلر دوبل) — نسخهٔ استاندارد.
#[allow(clippy::too_many_arguments)]
fn initial_step<F: OdeFn>(
    f: &mut F,
    t0: f64,
    y0: &[f64],
    f0: &[f64],
    direction: f64,
    max_step: f64,
    reltol: f64,
    abstol: f64,
) -> Result<f64, McError> {
    let neq = y0.len();
    let mut d0 = 0.0f64;
    let mut d1 = 0.0f64;
    for i in 0..neq {
        let scale = abstol + reltol * y0[i].abs();
        d0 += (y0[i] / scale).powi(2);
        d1 += (f0[i] / scale).powi(2);
    }
    let (d0, d1) = (d0.sqrt(), d1.sqrt());
    let mut h0 = if d0 < 1e-5 || d1 < 1e-5 {
        1e-6
    } else {
        0.01 * d0 / d1
    };
    h0 = h0.min(max_step);
    let y1: Vec<f64> = y0
        .iter()
        .zip(f0.iter())
        .map(|(y, fi)| y + h0 * fi)
        .collect();
    let f1 = f.call(t0 + direction * h0, &y1)?;
    let mut d2 = 0.0f64;
    for i in 0..neq {
        let scale = abstol + reltol * y0[i].abs();
        d2 += ((f1[i] - f0[i]) / scale).powi(2);
    }
    let d2 = d2.sqrt() / h0;
    let max_d = d1.max(d2);
    if max_d <= 1e-15 {
        h0 = 1e-6_f64.max(1e-3 * h0);
    } else {
        h0 = (0.01 / max_d).min(100.0 * h0).min(max_step);
    }
    h0 = h0.min((t0.abs() + max_step) * 1e-3 + 1e-9);
    Ok(direction * h0)
}

/// درون‌یابی Hermite مکعبی بین دو نقطهٔ پذیرفته‌شده.
fn hermite_dense(
    t0: f64,
    y0: &[f64],
    f0: &[f64],
    t1: f64,
    y1: &[f64],
    f1: &[f64],
    t: f64,
) -> Vec<f64> {
    let h = t1 - t0;
    let s = (t - t0) / h;
    let s2 = s * s;
    let s3 = s2 * s;
    let h00 = 2.0 * s3 - 3.0 * s2 + 1.0;
    let h10 = s3 - 2.0 * s2 + s;
    let h01 = -2.0 * s3 + 3.0 * s2;
    let h11 = s3 - s2;
    y0.iter()
        .zip(y1.iter())
        .zip(f0.iter())
        .zip(f1.iter())
        .map(|(((y0i, y1i), f0i), f1i)| h00 * y0i + h10 * h * f0i + h01 * y1i + h11 * h * f1i)
        .collect()
}

/// چرخهٔ حل تطبیقی مشترک — نتیجه + تاریخچهٔ f در گام‌های پذیرفته‌شده.
struct AdaptiveRun {
    t: Vec<f64>,
    y: Vec<Vec<f64>>,
    /// f(t_k, y_k) در هر گام پذیرفته‌شده — برای Hermite (FSAL رایگان می‌دهد)
    f_hist: Vec<Vec<f64>>,
    f_evals: u32,
    accepted: u32,
    rejected: u32,
}

#[allow(clippy::too_many_arguments)]
fn solve_adaptive<F: OdeFn, const N_STAGE: usize>(
    f: &mut F,
    tspan: (f64, f64),
    y0: Vec<f64>,
    opts: &OdeOpts,
    c: [f64; N_STAGE],
    a: &[&[f64]],
    b: [f64; N_STAGE],
    b_hat: [f64; N_STAGE],
    exponent: f64,
    stop: Option<&AtomicBool>,
) -> Result<AdaptiveRun, McError> {
    let (t0, t1) = tspan;
    let neq = y0.len();
    if neq == 0 {
        return Err(solver_error(
            ids::SIZE_MISMATCH,
            "ode solver: initial condition must have at least one component.",
            "حلگر ODE: شرط اولیه دست‌کم یک مؤلفه می‌خواهد.",
        ));
    }
    if t1 == t0 {
        // نکته: f64::signum در Rust برای +0.0 مقدار 1.0 می‌دهد (نه 0.0 مثل C) —
        // پس برابری را مستقیم چک می‌کنیم.
        return Err(solver_error(
            ids::SIZE_MISMATCH,
            "ode solver: tspan endpoints must differ.",
            "حلگر ODE: دو سر tspan باید متفاوت باشند.",
        ));
    }
    let direction = (t1 - t0).signum();
    if direction == 0.0 {
        return Err(solver_error(
            ids::SIZE_MISMATCH,
            "ode solver: tspan endpoints must differ.",
            "حلگر ODE: دو سر tspan باید متفاوت باشند.",
        ));
    }
    if let Some(flag) = stop {
        if flag.load(Ordering::Relaxed) {
            return Err(solver_error(
                ids::PARSE_ERROR,
                "ODE solver was cancelled; workspace is unchanged.",
                "حلگر ODE لغو شد؛ فضای کاری تغییر نکرده است.",
            ));
        }
    }
    let max_step = if opts.max_step.is_infinite() {
        (t1 - t0).abs() / 10.0
    } else {
        opts.max_step.abs()
    };
    let mut t = t0;
    let mut y = y0.clone();
    let mut k = vec![vec![0.0f64; neq]; N_STAGE];
    k[0] = f.call(t0, &y0)?;
    let mut f_evals = 1u32;
    let mut accepted = 0u32;
    let mut rejected = 0u32;
    let mut ts: Vec<f64> = vec![t0];
    let mut ys: Vec<Vec<f64>> = vec![y0.clone()];
    let mut f_hist: Vec<Vec<f64>> = vec![k[0].clone()];
    let mut h = match opts.initial_step {
        Some(h) => direction * h.abs(),
        None => initial_step(
            f,
            t0,
            &y0,
            &k[0],
            direction,
            max_step,
            opts.reltol,
            opts.abstol,
        )?,
    };
    let mut steps_since_cancel = 0u32;
    loop {
        let h_remaining = t1 - t;
        let mut h_step = h;
        if h_step.abs() > h_remaining.abs() {
            h_step = h_remaining;
        }
        if h_step.abs() > max_step {
            h_step = direction * max_step;
        }
        if h_step == 0.0 {
            break;
        }
        // مراحل — فقط مراحل لازم: k[s] = f(t + c[s]h, y + h·Σ a[s-1][j]·k[j])
        for s in 1..N_STAGE {
            let mut ysum = y.clone();
            for (j, k_j) in k.iter().take(s).enumerate() {
                let a_sj = a[s - 1][j];
                if a_sj != 0.0 {
                    for i in 0..neq {
                        ysum[i] += h_step * a_sj * k_j[i];
                    }
                }
            }
            k[s] = f.call(t + c[s] * h_step, &ysum)?;
            f_evals += 1;
        }
        let mut y_new = y.clone();
        let mut err_vec = vec![0.0f64; neq];
        for (s, &b_s) in b.iter().enumerate() {
            if b_s != 0.0 {
                for i in 0..neq {
                    y_new[i] += h_step * b_s * k[s][i];
                }
            }
            let bh = b_hat[s] - b_s;
            if bh != 0.0 {
                for i in 0..neq {
                    err_vec[i] += h_step * bh * k[s][i];
                }
            }
        }
        let err = err_norm(&err_vec, &y, &y_new, opts.reltol, opts.abstol);
        if err <= 1.0 || h_step.abs() <= 16.0 * f64::EPSILON * t.abs().max(1e-300) {
            accepted += 1;
            t += h_step;
            y = y_new;
            ts.push(t);
            ys.push(y.clone());
            // FSAL: مرحلهٔ آخر (c=1) = f(t+h, y_new) → k[0] گام بعد
            if c[N_STAGE - 1] == 1.0 {
                f_hist.push(k[N_STAGE - 1].clone());
                k[0] = k[N_STAGE - 1].clone();
            } else {
                let f_new = f.call(t, &y)?;
                f_evals += 1;
                f_hist.push(f_new.clone());
                k[0] = f_new;
            }
            if t == t1 {
                break;
            }
            let factor = if err == 0.0 {
                5.0
            } else {
                (0.9 * err.powf(-exponent)).clamp(0.2, 5.0)
            };
            h = h_step * factor;
            steps_since_cancel += 1;
            if steps_since_cancel >= 50 {
                steps_since_cancel = 0;
                if let Some(flag) = stop {
                    if flag.load(Ordering::Relaxed) {
                        return Err(solver_error(
                            ids::PARSE_ERROR,
                            "ODE solver was cancelled; workspace is unchanged.",
                            "حلگر ODE لغو شد؛ فضای کاری تغییر نکرده است.",
                        ));
                    }
                }
            }
        } else {
            rejected += 1;
            let factor = (0.9 * err.powf(-exponent)).clamp(0.1, 1.0);
            h = h_step * factor;
        }
        // سقف ایمنی: vdp1000 روی [0,3000] (مرجع سند ۰۷) ~۱M گام × ۶ ارزیابی می‌خواهد
        if f_evals > 30_000_000 {
            return Err(solver_error(
                ids::PARSE_ERROR,
                "ODE solver exceeded the function evaluation budget.",
                "حلگر ODE از سقف ارزیابی تابع گذشت.",
            ));
        }
    }
    Ok(AdaptiveRun {
        t: ts,
        y: ys,
        f_hist,
        f_evals,
        accepted,
        rejected,
    })
}

/// tspan دو عنصری → گام‌ها؛ چندعنصری → درون‌یابی روی نقاط خواسته‌شده.
fn finish_output(run: AdaptiveRun, tout: Option<&[f64]>) -> Result<OdeResult, McError> {
    let stats = (run.f_evals, run.accepted, run.rejected);
    let Some(points) = tout else {
        return Ok(OdeResult {
            t: run.t,
            y: run.y,
            n_f_evals: run.f_evals,
            n_accepted: run.accepted,
            n_rejected: run.rejected,
        });
    };
    let n = run.t.len();
    if n < 2 {
        return Ok(OdeResult {
            t: points.to_vec(),
            y: points.iter().map(|_| vec![f64::NAN]).collect(),
            n_f_evals: run.f_evals,
            n_accepted: run.accepted,
            n_rejected: run.rejected,
        });
    }
    let (t_lo, t_hi) = (run.t[0], run.t[n - 1]);
    let mut ts = Vec::with_capacity(points.len());
    let mut ys = Vec::with_capacity(points.len());
    let mut cursor = 0usize;
    for &tp in points {
        if tp < t_lo.min(t_hi) || tp > t_lo.max(t_hi) {
            return Err(solver_error(
                ids::SIZE_MISMATCH,
                format!("ode solver: output point {tp} lies outside tspan."),
                format!("حلگر ODE: نقطهٔ خروجی {tp} بیرون از tspan است."),
            ));
        }
        // پیشروی مکان‌نما تا بازهٔ شامل tp
        while cursor + 2 < n && (run.t[cursor + 1] - tp) * (run.t[cursor] - tp) > 0.0 {
            cursor += 1;
        }
        let (ta, ya) = (run.t[cursor], &run.y[cursor]);
        let (tb, yb) = (run.t[cursor + 1], &run.y[cursor + 1]);
        // شیب‌ها از تاریخچه (FSAL) — ارزیابی مجدد نمی‌شود
        let fa = &run.f_hist[cursor];
        let fb = &run.f_hist[cursor + 1];
        ts.push(tp);
        ys.push(hermite_dense(ta, ya, fa, tb, yb, fb, tp));
    }
    Ok(OdeResult {
        t: ts,
        y: ys,
        n_f_evals: stats.0,
        n_accepted: stats.1,
        n_rejected: stats.2,
    })
}

/// ode45 — Dormand–Prince 5(4) تطبیقی (سند ۰۴ §4.1).
pub fn ode45<F: OdeFn>(
    f: &mut F,
    tspan: &[f64],
    y0: Vec<f64>,
    tout: Option<&[f64]>,
    opts: &OdeOpts,
    stop: Option<&AtomicBool>,
) -> Result<OdeResult, McError> {
    let (t0, t1) = match tspan {
        [a, b] => (*a, *b),
        _ => {
            return Err(solver_error(
                ids::SIZE_MISMATCH,
                "ode45: tspan must have exactly two elements.",
                "ode45: tspan باید دقیقاً دو عنصر داشته باشد.",
            ))
        }
    };
    // Butcher Dormand–Prince 5(4) — ۷ مرحله (FSAL)، ترتیب ۵ در برابر ۴
    const C: [f64; 7] = [0.0, 1.0 / 5.0, 3.0 / 10.0, 4.0 / 5.0, 8.0 / 9.0, 1.0, 1.0];
    const A: [&[f64]; 6] = [
        &[1.0 / 5.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        &[3.0 / 40.0, 9.0 / 40.0, 0.0, 0.0, 0.0, 0.0],
        &[44.0 / 45.0, -56.0 / 15.0, 32.0 / 9.0, 0.0, 0.0, 0.0],
        &[
            19372.0 / 6561.0,
            -25360.0 / 2187.0,
            64448.0 / 6561.0,
            -212.0 / 729.0,
            0.0,
            0.0,
        ],
        &[
            9017.0 / 3168.0,
            -355.0 / 33.0,
            46732.0 / 5247.0,
            49.0 / 176.0,
            -5103.0 / 18656.0,
            0.0,
        ],
        &[
            35.0 / 384.0,
            0.0,
            500.0 / 1113.0,
            125.0 / 192.0,
            -2187.0 / 6784.0,
            11.0 / 84.0,
        ],
    ];
    const B: [f64; 7] = [
        35.0 / 384.0,
        0.0,
        500.0 / 1113.0,
        125.0 / 192.0,
        -2187.0 / 6784.0,
        11.0 / 84.0,
        0.0,
    ];
    const B_HAT: [f64; 7] = [
        5179.0 / 57600.0,
        0.0,
        7571.0 / 16695.0,
        393.0 / 640.0,
        -92097.0 / 339200.0,
        187.0 / 2100.0,
        1.0 / 40.0,
    ];
    let run = solve_adaptive::<F, 7>(f, (t0, t1), y0, opts, C, &A, B, B_HAT, 1.0 / 5.0, stop)?;
    finish_output(run, tout)
}

/// ode23 — Bogacki–Shampine 3(2) (سند ۰۴ §4.1 — برای دقت پایین سریع‌تر).
pub fn ode23<F: OdeFn>(
    f: &mut F,
    tspan: &[f64],
    y0: Vec<f64>,
    tout: Option<&[f64]>,
    opts: &OdeOpts,
    stop: Option<&AtomicBool>,
) -> Result<OdeResult, McError> {
    let (t0, t1) = match tspan {
        [a, b] => (*a, *b),
        _ => {
            return Err(solver_error(
                ids::SIZE_MISMATCH,
                "ode23: tspan must have exactly two elements.",
                "ode23: tspan باید دقیقاً دو عنصر داشته باشد.",
            ))
        }
    };
    // BS32: ۴ مرحله؛ k4 = f(t+h, y3) با y3 = گام مرتبه ۳ → FSAL
    const C: [f64; 4] = [0.0, 0.5, 0.75, 1.0];
    const A: [&[f64]; 3] = [
        &[0.5, 0.0, 0.0],
        &[0.0, 0.75, 0.0],
        &[2.0 / 9.0, 1.0 / 3.0, 4.0 / 9.0],
    ];
    const B: [f64; 4] = [2.0 / 9.0, 1.0 / 3.0, 4.0 / 9.0, 0.0];
    const B_HAT: [f64; 4] = [7.0 / 24.0, 1.0 / 4.0, 1.0 / 3.0, 1.0 / 8.0];
    let run = solve_adaptive::<F, 4>(f, (t0, t1), y0, opts, C, &A, B, B_HAT, 1.0 / 3.0, stop)?;
    finish_output(run, tout)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ode45_exponential_decay() {
        // y' = −y، y(0)=1 → y(2) = e^{−2}
        let mut f = |_t: f64, y: &[f64]| Ok(vec![-y[0]]);
        let opts = OdeOpts {
            reltol: 1e-8,
            abstol: 1e-10,
            ..Default::default()
        };
        let r = ode45(&mut f, &[0.0, 2.0], vec![1.0], None, &opts, None)
            .unwrap_or_else(|e| panic!("{e:?}"));
        let last = r.y.last().unwrap_or_else(|| panic!("empty"))[0];
        let want = (-2.0_f64).exp();
        assert!((last - want).abs() < 1e-6, "got {last} want {want}");
        assert!(r.n_accepted > 0);
        assert_eq!(r.t.len(), r.y.len());
        assert_eq!(r.t.len(), r.y.len());
    }

    #[test]
    fn ode45_sec_reference() {
        // y' = y·tan(t)، y(0)=1 → y(t) = sec(t) → y(1) ≈ 1.8508157176809255
        let mut f = |t: f64, y: &[f64]| Ok(vec![y[0] * t.tan()]);
        let opts = OdeOpts {
            reltol: 1e-9,
            abstol: 1e-12,
            ..Default::default()
        };
        let r = ode45(&mut f, &[0.0, 1.0], vec![1.0], None, &opts, None).unwrap();
        let last = r.y.last().unwrap_or_else(|| panic!("empty"))[0];
        let want = 1.0 / 1.0_f64.cos();
        assert!((last - want).abs() < 1e-7, "got {last} want {want}");
    }

    #[test]
    fn ode45_harmonic_oscillator_period() {
        // y'' = −y — پس از 10π واحد (۵ دوره کامل) به نقطهٔ شروع برمی‌گردد
        let mut f = |_t: f64, y: &[f64]| Ok(vec![y[1], -y[0]]);
        let opts = OdeOpts {
            reltol: 1e-9,
            abstol: 1e-12,
            ..Default::default()
        };
        let r = ode45(
            &mut f,
            &[0.0, 10.0 * std::f64::consts::PI],
            vec![1.0, 0.0],
            None,
            &opts,
            None,
        )
        .unwrap();
        let last = r.y.last().unwrap_or_else(|| panic!("empty")).clone();
        assert!((last[0] - 1.0).abs() < 1e-5, "pos {}", last[0]);
        assert!(last[1].abs() < 1e-5, "vel {}", last[1]);
    }

    #[test]
    fn ode45_tspan_points_interpolated() {
        // y' = −y با خروجی روی ۱۱ نقطه — Hermite باید روی حل تحلیلی بنشیند
        let mut f = |_t: f64, y: &[f64]| Ok(vec![-y[0]]);
        let opts = OdeOpts {
            reltol: 1e-8,
            abstol: 1e-10,
            ..Default::default()
        };
        let ts: Vec<f64> = (0..=10).map(|i| i as f64 * 0.2).collect();
        let r = ode45(&mut f, &[0.0, 2.0], vec![1.0], Some(&ts), &opts, None).unwrap();
        assert_eq!(r.t.len(), 11);
        for (i, &t) in ts.iter().enumerate() {
            let want = (-t).exp();
            assert!(
                (r.y[i][0] - want).abs() < 1e-7,
                "at t={t}: got {} want {want}",
                r.y[i][0]
            );
        }
    }

    #[test]
    fn ode23_default_accuracy_like_matlab() {
        // y' = −y با پیش‌فرض‌های متلب (RelTol 1e-3) — دقت ~1e-3 نسبی
        let mut f = |_t: f64, y: &[f64]| Ok(vec![-y[0]]);
        let r = ode23(
            &mut f,
            &[0.0, 1.0],
            vec![1.0],
            None,
            &OdeOpts::default(),
            None,
        )
        .unwrap_or_else(|e| panic!("{e:?}"));
        let last = r.y.last().unwrap_or_else(|| panic!("empty"))[0];
        let want = (-1.0_f64).exp();
        assert!((last - want).abs() < 5e-3, "got {last} want {want}");
        // تلورانس سخت‌گیرتر → دقیق‌تر
        let opts = OdeOpts {
            reltol: 1e-8,
            abstol: 1e-10,
            ..Default::default()
        };
        let r2 = ode23(&mut f, &[0.0, 1.0], vec![1.0], None, &opts, None).unwrap();
        let last2 = r2.y.last().unwrap_or_else(|| panic!("empty"))[0];
        assert!((last2 - want).abs() < 1e-6, "got {last2} want {want}");
    }

    #[test]
    fn cancel_flag_stops_solver() {
        let mut f = |_t: f64, y: &[f64]| Ok(vec![-y[0]]);
        let stop = AtomicBool::new(true);
        let r = ode45(
            &mut f,
            &[0.0, 1.0],
            vec![1.0],
            None,
            &OdeOpts::default(),
            Some(&stop),
        );
        assert!(r.is_err(), "cancelled run must error");
    }

    #[test]
    fn bad_tspan_errors() {
        let mut f = |_t: f64, y: &[f64]| Ok(vec![-y[0]]);
        assert!(ode45(
            &mut f,
            &[1.0, 1.0],
            vec![1.0],
            None,
            &OdeOpts::default(),
            None
        )
        .is_err());
        assert!(ode45(&mut f, &[0.0], vec![1.0], None, &OdeOpts::default(), None).is_err());
    }

    #[test]
    fn output_point_outside_tspan_errors() {
        let mut f = |_t: f64, y: &[f64]| Ok(vec![-y[0]]);
        let r = ode45(
            &mut f,
            &[0.0, 1.0],
            vec![1.0],
            Some(&[0.5, 1.5]),
            &OdeOpts::default(),
            None,
        );
        assert!(r.is_err());
    }
}
