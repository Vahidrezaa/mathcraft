//! درون‌یابی — interp1 با روش‌های linear/nearest/pchip/spline (سند ۰۴ بخش ۶).
//!
//! قراردادهای متلب:
//! - x باید بردار یکنواخت باشد (صعودی یا نزولی — متلب هر دو را می‌پذیرد)؛
//!   درون‌سازه روی شبکه مرتب‌شده صعودی انجام می‌شود.
//! - خارج بازه: پیش‌فرض `NaN`؛ با پرچم `'extrap'` برون‌یابی روش.
//! - `'spline'` متلب = مکعبی **not-a-knot** (نه natural) — سیستم (n+1)-مجهولی کامل.

use mc_core::{Matrix, McError};

use crate::linalg::num_error;

/// روش درون‌یابی.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    Linear,
    Nearest,
    Pchip,
    Spline,
}

impl Method {
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "linear" => Some(Method::Linear),
            "nearest" | "nearest neighbour" | "nearest neighbor" => Some(Method::Nearest),
            "pchip" => Some(Method::Pchip),
            "spline" => Some(Method::Spline),
            _ => None,
        }
    }
}

/// ورودی استانداردشدهٔ interp1 — بردارهای شبکه/مقادیر صعودی + پرچم برون‌یابی.
pub struct Interp1Input {
    pub x: Vec<f64>,
    pub v: Vec<f64>,
    /// شکل خروجی (هم‌شکل xq).
    pub out_rows: usize,
    pub out_cols: usize,
    pub xq: Vec<f64>,
    pub method: Method,
    pub extrap: bool,
    /// مقدار جایگزین خارج از بازه (پرچم 'extrapval' متلب).
    pub extrapval: Option<f64>,
}

/// آماده‌سازی ورودی — اعتبارسنجی دوزبانه + مرتب‌سازی نزولی→صعودی.
pub fn prepare_interp1(
    x: &Matrix,
    v: &Matrix,
    xq: &Matrix,
    method: Method,
    extrap: bool,
    extrapval: Option<f64>,
) -> Result<Interp1Input, McError> {
    for (name, m) in [("X", x), ("V", v), ("XQ", xq)] {
        if m.rows() != 1 && m.cols() != 1 {
            return Err(num_error(
                mc_core::error::ids::SIZE_MISMATCH,
                format!("interp1: {name} must be a vector."),
                format!("interp1: {name} باید بردار باشد."),
            ));
        }
    }
    let mut xs: Vec<f64> = (0..x.numel()).map(|i| x.get_linear(i)).collect();
    let mut vs: Vec<f64> = (0..v.numel()).map(|i| v.get_linear(i)).collect();
    if xs.len() != vs.len() {
        return Err(num_error(
            mc_core::error::ids::SIZE_MISMATCH,
            format!(
                "interp1: X and V must have the same length ({} vs {}).",
                xs.len(),
                vs.len()
            ),
            format!(
                "interp1: X و V باید هم‌طول باشند ({} در برابر {}).",
                xs.len(),
                vs.len()
            ),
        ));
    }
    if xs.len() < 2 {
        return Err(num_error(
            mc_core::error::ids::SIZE_MISMATCH,
            "interp1: at least two grid points are required.",
            "interp1: دست‌کم دو نقطه شبکه لازم است.",
        ));
    }
    // یکنواختی اکید — NaN نقض می‌کند (رفتار متلب: x باید monotone باشد)
    let ascending = xs.windows(2).all(|w| w[0] < w[1]);
    let descending = xs.windows(2).all(|w| w[0] > w[1]);
    if !ascending && !descending {
        return Err(num_error(
            mc_core::error::ids::SIZE_MISMATCH,
            "interp1: grid points X must be strictly monotonic.",
            "interp1: نقاط شبکه X باید اکیداً یکنوا باشند.",
        ));
    }
    if descending {
        xs.reverse();
        vs.reverse();
    }
    let xqs: Vec<f64> = (0..xq.numel()).map(|i| xq.get_linear(i)).collect();
    Ok(Interp1Input {
        x: xs,
        v: vs,
        out_rows: xq.rows(),
        out_cols: xq.cols(),
        xq: xqs,
        method,
        extrap,
        extrapval,
    })
}

/// اجرای interp1 روی ورودی آماده — خروجی هم‌شکل XQ.
pub fn interp1_eval(inp: &Interp1Input) -> Matrix {
    let n = inp.x.len();
    let mut out = Matrix::full(inp.out_rows, inp.out_cols, f64::NAN);
    let values = match inp.method {
        Method::Linear => None, // محلی
        Method::Nearest => None,
        Method::Pchip => Some(pchip_slopes(&inp.x, &inp.v)),
        Method::Spline => Some(not_a_knot_second_derivs(&inp.x, &inp.v)),
    };
    for (idx, &t) in inp.xq.iter().enumerate() {
        let out_of_range = t < inp.x[0] || t > inp.x[n - 1];
        let val = if out_of_range {
            match inp.method {
                // رفتار متلب: spline و pchip به‌طور پیش‌فرض برون‌یابی می‌کنند
                // (ارزیابی مکعب انتهایی)؛ بقیه روش‌ها NaN می‌دهند.
                Method::Spline | Method::Pchip => {
                    evaluate(&inp.x, &inp.v, values.as_ref(), inp.method, t)
                }
                _ => match (inp.extrap, inp.extrapval) {
                    (false, Some(ev)) => ev,
                    (false, None) => f64::NAN,
                    (true, _) => evaluate(&inp.x, &inp.v, values.as_ref(), inp.method, t),
                },
            }
        } else {
            evaluate(&inp.x, &inp.v, values.as_ref(), inp.method, t)
        };
        out.set_linear(idx, val);
    }
    out
}

fn evaluate(x: &[f64], v: &[f64], pre: Option<&Vec<f64>>, method: Method, t: f64) -> f64 {
    match method {
        Method::Nearest => {
            // گرهٔ نزدیک‌تر؛ تساوی → گرهٔ چپ (رفتار متلب 'nearest')
            let i = search_left(x, t);
            let right = (i + 1 < x.len()).then(|| x[i + 1]);
            match right {
                Some(r) if (r - t).abs() < (t - x[i]).abs() => v[i + 1],
                _ => v[i],
            }
        }
        Method::Linear => {
            let i = search_left(x, t);
            if t == x[i] {
                return v[i];
            }
            if i + 1 >= x.len() {
                return v[x.len() - 1];
            }
            let h = x[i + 1] - x[i];
            let w = (t - x[i]) / h;
            v[i] * (1.0 - w) + v[i + 1] * w
        }
        Method::Pchip => {
            let slopes = match pre {
                Some(s) => s,
                None => unreachable!("pchip slopes are precomputed in interp1_eval"),
            };
            let i = find_interval(x, t);
            hermite_eval(x, v, slopes, i, t)
        }
        Method::Spline => {
            let m2 = match pre {
                Some(s) => s,
                None => unreachable!("spline second derivatives are precomputed in interp1_eval"),
            };
            spline_eval(x, v, m2, t)
        }
    }
}

/// آخرین بازهٔ [x[i], x[i+1]] شامل t — با جست‌وجوی دودویی (سند ۰۴ §6).
fn search_left(x: &[f64], t: f64) -> usize {
    match x.binary_search_by(|&xi| xi.partial_cmp(&t).unwrap_or(std::cmp::Ordering::Less)) {
        Ok(i) => i,
        Err(i) => i.saturating_sub(1).min(x.len() - 2),
    }
}

/// بازهٔ درون‌یابی برای Hermite (t دقیقاً در گرهٔ آخر → بازهٔ قبل).
fn find_interval(x: &[f64], t: f64) -> usize {
    let n = x.len();
    if t >= x[n - 1] {
        return n - 2;
    }
    match x.binary_search_by(|&xi| xi.partial_cmp(&t).unwrap_or(std::cmp::Ordering::Less)) {
        Ok(i) => i.min(n - 2),
        Err(i) => i.saturating_sub(1).min(n - 2),
    }
}

/// ارزیابی Hermite مکعبی روی بازهٔ i با شیب‌های داده‌شده.
fn hermite_eval(x: &[f64], v: &[f64], d: &[f64], i: usize, t: f64) -> f64 {
    let h = x[i + 1] - x[i];
    let s = (t - x[i]) / h;
    let s2 = s * s;
    let s3 = s2 * s;
    // بسط‌های پایه Hermite
    let h00 = 2.0 * s3 - 3.0 * s2 + 1.0;
    let h10 = s3 - 2.0 * s2 + s;
    let h01 = -2.0 * s3 + 3.0 * s2;
    let h11 = s3 - s2;
    h00 * v[i] + h10 * h * d[i] + h01 * v[i + 1] + h11 * h * d[i + 1]
}

// ---------- Pchip — Fritsch–Carlson با حفظ مونوتونی (سند ۰۴ §6) ----------

/// شیب‌های Fritsch–Carlson — بدون overshoot روی داده مونوتون (حیاتی برای داده آزمایشگاه).
pub fn pchip_slopes(x: &[f64], v: &[f64]) -> Vec<f64> {
    let n = x.len();
    let mut h = vec![0.0f64; n - 1];
    let mut delta = vec![0.0f64; n - 1];
    for i in 0..n - 1 {
        h[i] = x[i + 1] - x[i];
        delta[i] = (v[i + 1] - v[i]) / h[i];
    }
    let mut d = vec![0.0f64; n];
    // داخلی: میانگین وزنی همساز Forman–Sloan (فرمول pchip متلب)
    for i in 1..n - 1 {
        if delta[i - 1] * delta[i] <= 0.0 {
            d[i] = 0.0; // تغییر علامت یا صفر → شیب صفر (بدون overshoot)
        } else {
            let w1 = 2.0 * h[i] + h[i - 1];
            let w2 = h[i] + 2.0 * h[i - 1];
            d[i] = (w1 + w2) / (w1 / delta[i - 1] + w2 / delta[i]);
        }
    }
    // لبه‌ها: فرمول یک‌طرفهٔ pchip متلب (جهت‌گیری با نقطهٔ دوم)
    d[0] = edge_slope(h[0], h[1], delta[0], delta[1], d[1]);
    d[n - 1] = edge_slope(h[n - 2], h[n - 3], delta[n - 2], delta[n - 3], d[n - 2]);
    d
}

/// شیب لبهٔ pchip متلب (سه‌نقطه‌ای — MATLBpchipslope).
fn edge_slope(h0: f64, h1: f64, del0: f64, del1: f64, d1: f64) -> f64 {
    if del0 * del1 <= 0.0 {
        return 0.0;
    }
    // فرمول دقیق pchip متلب برای نقاط انتهایی
    let mut d0 = ((2.0 * h0 + h1) * del0 - h0 * del1) / (h0 + h1);
    if d0 * del0 <= 0.0 {
        d0 = 0.0;
    } else if del0 * del1 <= 0.0 && d0.abs() > 3.0 * del0.abs() {
        d0 = 3.0 * del0;
    }
    let _ = d1;
    d0
}

// ---------- Spline مکعبی not-a-knot (پیش‌فرض 'spline' متلب) ----------

/// دومین مشتق‌های گره‌ای M_i — سیستم کامل (n+1)×(n+1) با شرط not-a-knot.
/// حل با حذف گاوس دنس (n معمولاً کوچک؛ سادگی و درستی مقدم بر سرعت).
pub fn not_a_knot_second_derivs(x: &[f64], v: &[f64]) -> Vec<f64> {
    let n1 = x.len(); // تعداد گره‌ها
    let n = n1 - 1; // تعداد بازه‌ها
    let mut h = vec![0.0f64; n];
    for i in 0..n {
        h[i] = x[i + 1] - x[i];
    }
    // مجهولات: M_0..M_{n} (دومین مشتق) — معادلات:
    // داخلی i=1..n-1: h[i-1]M[i-1] + 2(h[i-1]+h[i])M[i] + h[i]M[i+1] = 6(Δ_i − Δ_{i-1})
    // not-a-knot در x_1: (h[0]+h[1])M[1] + h[1]M[2]... فرم استاندارد:
    //   h[1]·M[0] + (h[0]+h[1])·M[1] − h[0]·M[2] = 0  ← پیوستگی S''' در x_1
    //   −h[n-1]·M[n-2] + (h[n-2]+h[n-1])·M[n-1] + h[n-2]·M[n] = 0 ← در x_{n-1}
    let dim = n1;
    let mut a = vec![0.0f64; dim * dim];
    let mut rhs = vec![0.0f64; dim];
    let mut delta = vec![0.0f64; n];
    for i in 0..n {
        delta[i] = (v[i + 1] - v[i]) / h[i];
    }
    // ردیف ۰ — not-a-knot چپ (x_1): پیوستگی S''' بین بازه‌های ۰ و ۱
    // (M₂−M₁)/h₁ = (M₁−M₀)/h₀ ⟹ h₁·M₀ − (h₀+h₁)·M₁ + h₀·M₂ = 0
    // ردیف ۰ — اندیس‌های صریح (clippy: 0*dim را عملیات صفر می‌بیند)
    a[0] = h[1];
    a[1] = -(h[0] + h[1]);
    a[2] = h[0];
    // ردیف‌های داخلی (i = 1..n-1) — معادلهٔ گرهٔ i
    for i in 1..n {
        let r = i * dim;
        a[r + i - 1] = h[i - 1];
        a[r + i] = 2.0 * (h[i - 1] + h[i]);
        a[r + i + 1] = h[i];
        rhs[i] = 6.0 * (delta[i] - delta[i - 1]);
    }
    // ردیف آخر — not-a-knot راست (x_{n-1}):
    // (Mₙ−Mₙ₋₁)/hₙ₋₁ = (Mₙ₋₁−Mₙ₋₂)/hₙ₋₂ ⟹ hₙ₋₁·Mₙ₋₂ − (hₙ₋₂+hₙ₋₁)·Mₙ₋₁ + hₙ₋₂·Mₙ = 0
    let last = dim - 1;
    a[last * dim + (n - 2)] = h[n - 1];
    a[last * dim + (n - 1)] = -(h[n - 2] + h[n - 1]);
    a[last * dim + n] = h[n - 2];
    // حذف گاوس با pivot جزئی
    let mut mm = a;
    for col in 0..dim {
        // pivot
        let mut piv = col;
        let mut best = mm[col * dim + col].abs();
        for r in col + 1..dim {
            let cand = mm[r * dim + col].abs();
            if cand > best {
                best = cand;
                piv = r;
            }
        }
        if best == 0.0 {
            // شبکه یکنوا با نقاط تکراری — قبلاً رد شده؛ اینجا غیرقابل‌رسیدن
            break;
        }
        if piv != col {
            for c in 0..dim {
                mm.swap(col * dim + c, piv * dim + c);
            }
            rhs.swap(col, piv);
        }
        let pivot = mm[col * dim + col];
        for r in col + 1..dim {
            let factor = mm[r * dim + col] / pivot;
            if factor != 0.0 {
                for c in col..dim {
                    mm[r * dim + c] -= factor * mm[col * dim + c];
                }
                rhs[r] -= factor * rhs[col];
            }
        }
    }
    // جایگذاری به عقب
    let mut sol = vec![0.0f64; dim];
    for i in (0..dim).rev() {
        let mut acc = rhs[i];
        for c in i + 1..dim {
            acc -= mm[i * dim + c] * sol[c];
        }
        sol[i] = acc / mm[i * dim + i];
    }
    sol
}

/// ارزیابی spline از دومین مشتق‌های گره‌ای.
fn spline_eval(x: &[f64], v: &[f64], m2: &[f64], t: f64) -> f64 {
    let i = find_interval(x, t);
    let h = x[i + 1] - x[i];
    let a = (x[i + 1] - t) / h;
    let b = (t - x[i]) / h;
    // فرمول استاندارد spline مکعبی بر پایهٔ دومین مشتق
    a * v[i] + b * v[i + 1] + ((a * a * a - a) * m2[i] + (b * b * b - b) * m2[i + 1]) * h * h / 6.0
}

/// کمینهٔ نقطه لازم برای هر روش — not-a-knot دست‌کم ۴ گره می‌خواهد.
pub fn require_min_points(n: usize, method: Method) -> Result<(), McError> {
    let min = match method {
        Method::Spline => 4,
        _ => 2,
    };
    if n < min {
        return Err(num_error(
            mc_core::error::ids::SIZE_MISMATCH,
            format!("interp1: method needs at least {min} points, got {n}."),
            format!("interp1: این روش دست‌کم {min} نقطه می‌خواهد، {n} داده شد."),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(v: &[f64]) -> Matrix {
        Matrix::from_columns(1, v.len(), v.to_vec())
    }

    fn at(m: &Matrix) -> f64 {
        m.get(0, 0)
    }

    fn run(x: &[f64], v: &[f64], tq: f64, method: Method) -> f64 {
        let inp = prepare_interp1(&row(x), &row(v), &row(&[tq]), method, false, None)
            .unwrap_or_else(|e| panic!("{e:?}"));
        at(&interp1_eval(&inp))
    }

    #[test]
    fn linear_interior_and_extrapolation_default_nan() {
        let x = [0.0, 1.0, 2.0, 3.0];
        let v = [0.0, 2.0, 4.0, 6.0];
        assert!((run(&x, &v, 1.5, Method::Linear) - 3.0).abs() < 1e-15);
        assert!((run(&x, &v, 0.25, Method::Linear) - 0.5).abs() < 1e-15);
        assert!(run(&x, &v, -1.0, Method::Linear).is_nan());
        assert!(run(&x, &v, 3.5, Method::Linear).is_nan());
        // نزولی → معکوس می‌شود و همان جواب
        assert!(
            (run(
                &[3.0, 2.0, 1.0, 0.0],
                &[6.0, 4.0, 2.0, 0.0],
                1.5,
                Method::Linear
            ) - 3.0)
                .abs()
                < 1e-15
        );
    }

    #[test]
    fn nearest_rounds_to_closest_node() {
        let x = [0.0, 1.0, 2.0];
        let v = [10.0, 20.0, 30.0];
        assert!((run(&x, &v, 0.4, Method::Nearest) - 10.0).abs() < 1e-15);
        assert!((run(&x, &v, 0.6, Method::Nearest) - 20.0).abs() < 1e-15);
        assert!((run(&x, &v, 1.0, Method::Nearest) - 20.0).abs() < 1e-15);
    }

    #[test]
    fn pchip_is_monotone_preserving_on_monotone_data() {
        // داده اکیداً صعودی → pchip نباید overshoot کند (فراتر از بازهٔ مقادیر)
        let x = [0.0, 1.0, 2.0, 3.0, 4.0];
        let v = [0.0, 0.1, 5.0, 5.1, 5.2];
        let inp = prepare_interp1(
            &row(&x),
            &row(&v),
            &row(&[0.5, 1.5, 2.5]),
            Method::Pchip,
            false,
            None,
        )
        .unwrap();
        let out = interp1_eval(&inp);
        for i in 0..3 {
            let t = out.get_linear(i);
            assert!(
                t >= v[i] - 1e-12 && t <= v[i + 1] + 1e-12,
                "overshoot at {i}: {t}"
            );
        }
        // در گره‌ها دقیقاً مقدار گره
        let inp = prepare_interp1(
            &row(&x),
            &row(&v),
            &row(&[1.0, 3.0]),
            Method::Pchip,
            false,
            None,
        )
        .unwrap();
        let out = interp1_eval(&inp);
        assert!((out.get_linear(0) - 0.1).abs() < 1e-12);
        assert!((out.get_linear(1) - 5.1).abs() < 1e-12);
    }

    #[test]
    fn spline_not_a_knot_matches_reference_cubic() {
        // spline متلب (not-a-knot) روی دادهٔ چندجمله‌ای درجه ۳ باید دقیقاً همان چندجمله‌ای را بازتولید کند
        // p(x) = x³ − 2x + 1 روی ۵ نقطه
        let f = |t: f64| t * t * t - 2.0 * t + 1.0;
        let x = [-2.0, -1.0, 0.0, 1.0, 2.0];
        let v: Vec<f64> = x.iter().map(|&t| f(t)).collect();
        // بین گره‌ها هم دقیق (چندجمله‌ای درجه ۳ در هر بازه = خودش)
        for &t in &[-1.5, -0.25, 0.5, 1.75] {
            let got = run(&x, &v, t, Method::Spline);
            assert!(
                (got - f(t)).abs() < 1e-10,
                "not-a-knot must reproduce cubic exactly at {t}: {got} vs {}",
                f(t)
            );
        }
    }

    #[test]
    fn spline_pchip_extrapolate_by_default() {
        // رفتار متلب: 'spline' و 'pchip' پیش‌فرض برون‌یابی می‌کنند؛ linear NaN می‌دهد
        let x = [0.0, 1.0, 2.0, 3.0];
        let v = [0.0, 1.0, 4.0, 9.0];
        let e = run(&x, &v, 3.5, Method::Spline);
        assert!(e.is_finite(), "spline extrapolates by default");
        let p = run(&x, &v, 3.5, Method::Pchip);
        assert!(p.is_finite(), "pchip extrapolates by default");
        assert!(run(&x, &v, 3.5, Method::Linear).is_nan());
        assert!(run(&x, &v, -0.5, Method::Nearest).is_nan());
        // روی دادهٔ چندجمله‌ای درجه ۳ برون‌یابی spline هم دقیق است
        let f = |t: f64| t * t * t - 2.0 * t + 1.0;
        let x3 = [-2.0, -1.0, 0.0, 1.0, 2.0];
        let v3: Vec<f64> = x3.iter().map(|&t| f(t)).collect();
        let got = run(&x3, &v3, 2.5, Method::Spline);
        assert!(
            (got - f(2.5)).abs() < 1e-9,
            "spline extrap: {got} vs {}",
            f(2.5)
        );
    }

    #[test]
    fn method_names_parse() {
        assert_eq!(Method::from_name("linear"), Some(Method::Linear));
        assert_eq!(Method::from_name("pchip"), Some(Method::Pchip));
        assert_eq!(Method::from_name("spline"), Some(Method::Spline));
        assert_eq!(Method::from_name("nearest"), Some(Method::Nearest));
        assert_eq!(Method::from_name("cubic"), None);
    }

    #[test]
    fn non_monotonic_grid_errors() {
        let inp = prepare_interp1(
            &row(&[0.0, 2.0, 1.0]),
            &row(&[1.0, 2.0, 3.0]),
            &row(&[1.0]),
            Method::Linear,
            false,
            None,
        );
        assert!(inp.is_err());
    }

    #[test]
    fn spline_needs_four_points() {
        assert!(require_min_points(3, Method::Spline).is_err());
        assert!(require_min_points(4, Method::Spline).is_ok());
        assert!(require_min_points(2, Method::Linear).is_ok());
    }
}
