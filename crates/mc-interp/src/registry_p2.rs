//! رجیستری P2 — توابع عددی/آمار/RNG/حلگرها (سند ۰۴ §4-7، سند ۰۳ §12).
//!
//! ساختار:
//! - [`lookup`] → builtinهای تک‌خروجی P2 (به REGISTRY اصلی زنجیر می‌شود).
//! - [`lookup_multi`] → چندخروجی‌ها: `[T,Y] = ode45`، `[q,r] = deconv`،
//!   `[V,D] = eig` — مکانیزم جدید multi-assign برای builtin (سند ۰۳ §9).
//! - توابع کاربر: fzero/integral/ode هندل `@(x)...` را مستقیم فرامی‌خوانند
//!   (`AnonFn.call`) — همان قرارداد capture-by-value متلب (سند ۰۳ §9).

use mc_core::{ComplexMatrix, Matrix, McError, Value};
use mc_num as num;

use crate::complex_ops as cx;

/// امضای builtin چندخروجی — nargout = تعداد خروجی خواسته‌شده.
pub type MultiBuiltin = fn(&[Value], usize) -> Result<Vec<Value>, McError>;

/// یافتن builtin تک‌خروجی P2.
pub fn lookup(name: &str) -> Option<crate::registry::Builtin> {
    P2_REGISTRY
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, f)| *f)
}

/// یافتن builtin چندخروجی.
pub fn lookup_multi(name: &str) -> Option<MultiBuiltin> {
    P2_MULTI.iter().find(|(n, _)| *n == name).map(|(_, f)| *f)
}

static P2_REGISTRY: &[(&str, crate::registry::Builtin)] = &[
    // ---- RNG (سند ۰۴ §7) ----
    ("rng", bi_rng),
    ("rand", bi_rand),
    ("randn", bi_randn),
    ("randi", bi_randi),
    ("randperm", bi_randperm),
    // ---- چندجمله‌ای (سند ۰۴ §6 + §5.2) ----
    ("roots", bi_roots),
    ("polyfit", bi_polyfit),
    ("polyval", bi_polyval),
    ("conv", bi_conv),
    ("compan", bi_compan),
    // ---- درون‌یابی (سند ۰۴ §6) ----
    ("interp1", bi_interp1),
    ("interp2", bi_interp2),
    // ---- ریشه/انتگرال (سند ۰۴ §5) ----
    ("fzero", bi_fzero),
    ("integral", bi_integral),
    // ---- آمار (سند ۰۴ §7) ----
    ("median", bi_median),
    ("std", bi_std),
    ("var", bi_var),
    ("cov", bi_cov),
    ("corrcoef", bi_corrcoef),
    // ---- مختلط: عنصری مختلط-آگاه (جایگزین نسخهٔ P1) ----
    ("real", bi_real),
    ("imag", bi_imag),
    ("conj", bi_conj),
    ("abs", bi_abs),
    ("angle", bi_angle),
    ("isreal", bi_isreal),
    ("complex", bi_complex),
];

static P2_MULTI: &[(&str, MultiBuiltin)] = &[
    ("ode45", bi_ode45),
    ("ode23", bi_ode23),
    ("deconv", bi_deconv),
    ("eig", bi_eig),
    ("qr", bi_qr_multi),
    ("svd", bi_svd_multi),
    ("histcounts", bi_histcounts),
    ("histogram", bi_histogram),
];

// ---------- ابزارهای ورودی ----------

fn err(en: &str, fa: &str) -> McError {
    McError::new(mc_core::error::ids::PARSE_ERROR, 0, 0, en, fa)
}

fn scalar_val(v: f64) -> Value {
    Value::Double(Matrix::scalar(v))
}

/// اسکالر عددی از آرگومان.
fn need_scalar(args: &[Value], i: usize, name: &str) -> Result<f64, McError> {
    match args.get(i) {
        Some(Value::Double(m)) if m.is_scalar() => Ok(m.get(0, 0)),
        Some(Value::Complex(z)) if z.is_scalar() && z.is_all_real() => Ok(z.get(0, 0).0),
        _ => Err(err(
            &format!("{name}: argument {} must be a numeric scalar.", i + 1),
            &format!("{name}: آرگومان {} باید اسکالر عددی باشد.", i + 1),
        )),
    }
}

/// بردار تخت از آرگومان (هر جهتی).
fn need_vector(args: &[Value], i: usize, name: &str) -> Result<Vec<f64>, McError> {
    match args.get(i) {
        Some(Value::Double(m)) if m.rows() == 1 || m.cols() == 1 => {
            Ok((0..m.numel()).map(|k| m.get_linear(k)).collect())
        }
        Some(Value::Complex(z)) if (z.rows() == 1 || z.cols() == 1) && z.is_all_real() => {
            Ok(z.re_column_slice().to_vec())
        }
        _ => Err(err(
            &format!("{name}: argument {} must be a real vector.", i + 1),
            &format!("{name}: آرگومان {} باید بردار حقیقی باشد.", i + 1),
        )),
    }
}

/// ماتریس حقیقی از آرگومان.
fn need_matrix(args: &[Value], i: usize, name: &str) -> Result<Matrix, McError> {
    match args.get(i) {
        Some(Value::Double(m)) => Ok(m.clone()),
        _ => Err(err(
            &format!("{name}: argument {} must be a real matrix.", i + 1),
            &format!("{name}: آرگومان {} باید ماتریس حقیقی باشد.", i + 1),
        )),
    }
}

/// رشته کاراکتری از آرگومان.
fn need_char(args: &[Value], i: usize, name: &str) -> Result<String, McError> {
    match args.get(i) {
        Some(Value::Char(s)) => Ok(s.clone()),
        _ => Err(err(
            &format!("{name}: argument {} must be a string like 'pchip'.", i + 1),
            &format!("{name}: آرگومان {} باید رشته باشد مثل 'pchip'.", i + 1),
        )),
    }
}

/// هندل تابع کاربر از آرگومان.
fn need_fn<'a>(args: &'a [Value], i: usize, name: &str) -> Result<&'a mc_core::AnonFn, McError> {
    match args.get(i) {
        Some(Value::Anon(f)) => Ok(f),
        _ => Err(err(
            &format!(
                "{name}: argument {} must be a function handle like @(x)...",
                i + 1
            ),
            &format!("{name}: آرگومان {} باید هندل تابع باشد مثل @(x)...", i + 1),
        )),
    }
}

/// خروجی تابع کاربر → بردار f64 (ستون‌پیچ — scalar هم مجاز).
fn value_to_vec(v: &Value, name: &str) -> Result<Vec<f64>, McError> {
    match v {
        Value::Double(m) => Ok((0..m.numel()).map(|k| m.get_linear(k)).collect()),
        Value::Complex(z) if z.is_all_real() => Ok(z.re_column_slice().to_vec()),
        _ => Err(err(
            &format!("{name}: function must return a numeric value."),
            &format!("{name}: تابع کاربر باید مقدار عددی برگرداند."),
        )),
    }
}

// ---------- RNG (سند ۰۴ §7) ----------

/// rng(seed) — تکرارپذیری؛ خروجی ساختاری rng() متلب در P2-b می‌آید (ساختار لازم دارد).
fn bi_rng(args: &[Value], _n: usize) -> Result<Value, McError> {
    if args.is_empty() {
        return Err(err(
            "rng: seed form rng(seed) is supported; the state-struct form arrives in P2-b.",
            "rng: شکل rng(seed) پشتیبانی می‌شود؛ شکل ساختارِ وضعیت در P2-b می‌آید.",
        ));
    }
    let s = need_scalar(args, 0, "rng")?;
    if s < 0.0 || s.fract() != 0.0 || !s.is_finite() {
        return Err(err(
            "rng: seed must be a non-negative integer.",
            "rng: seed باید عدد صحیح نامنفی باشد.",
        ));
    }
    num::rng::reseed(s as u64);
    Ok(scalar_val(s))
}

/// rand() / rand(n) / rand(m, n) — یکنواخت [0,1).
fn bi_rand(args: &[Value], _n: usize) -> Result<Value, McError> {
    uniform_like(args, "rand", num::rng::uniform)
}

fn bi_randn(args: &[Value], _n: usize) -> Result<Value, McError> {
    uniform_like(args, "randn", num::rng::normal)
}

fn uniform_like(args: &[Value], name: &str, draw: fn() -> f64) -> Result<Value, McError> {
    let (rows, cols) = match args.len() {
        0 => (1usize, 1usize),
        1 => {
            let n = need_scalar(args, 0, name)?;
            if n < 0.0 || n.fract() != 0.0 {
                return Err(err(
                    &format!("{name}: size must be a non-negative integer."),
                    &format!("{name}: اندازه باید عدد صحیح نامنفی باشد."),
                ));
            }
            let n = n as usize;
            (n, n)
        }
        2 => {
            let m = need_scalar(args, 0, name)?;
            let n = need_scalar(args, 1, name)?;
            if m < 0.0 || n < 0.0 || m.fract() != 0.0 || n.fract() != 0.0 {
                return Err(err(
                    &format!("{name}: sizes must be non-negative integers."),
                    &format!("{name}: اندازه‌ها باید صحیح نامنفی باشند."),
                ));
            }
            (m as usize, n as usize)
        }
        _k => {
            return Err(err(
                &format!("{name}: 0 to 2 arguments supported in this milestone."),
                &format!("{name}: در این مایل‌استون ۰ تا ۲ آرگومان پشتیبانی می‌شود."),
            ))
        }
    };
    let mut out = Matrix::try_full(rows, cols, 0.0)?; // بودجهٔ حافظه (ROADMAP 2.5) — ابعاد از کاربر
    for idx in 0..out.numel() {
        out.set_linear(idx, draw());
    }
    Ok(Value::Double(out))
}

/// randi(imax [, n | m, n]) یا randi([imin imax], ...) — صحیح شامل دو سر.
fn bi_randi(args: &[Value], _n: usize) -> Result<Value, McError> {
    const NAME: &str = "randi";
    let (lo, hi) = match args.first() {
        Some(Value::Double(m)) if m.numel() == 2 => {
            let a = m.get_linear(0);
            let b = m.get_linear(1);
            (a, b)
        }
        Some(_) => {
            let imax = need_scalar(args, 0, NAME)?;
            (1.0, imax)
        }
        None => return Err(err("randi: needs a bound.", "randi: به سقف نیاز دارد.")),
    };
    if lo < 1.0 || hi < lo || hi.fract() != 0.0 || lo.fract() != 0.0 {
        return Err(err(
            "randi: bounds must be integers with imax ≥ imin ≥ 1.",
            "randi: مرزها باید صحیح باشند و imax ≥ imin ≥ 1.",
        ));
    }
    let (rows, cols) = match args.len() {
        1 => (1usize, 1usize),
        2 => {
            let n = need_scalar(args, 1, NAME)?;
            if n < 0.0 || n.fract() != 0.0 {
                return Err(err(
                    "randi: size must be a non-negative integer.",
                    "randi: اندازه باید صحیح نامنفی باشد.",
                ));
            }
            (n as usize, n as usize)
        }
        3 => {
            let m = need_scalar(args, 1, NAME)?;
            let n = need_scalar(args, 2, NAME)?;
            if m < 0.0 || n < 0.0 {
                return Err(err(
                    "randi: sizes must be non-negative integers.",
                    "randi: اندازه‌ها باید صحیح نامنفی باشند.",
                ));
            }
            (m as usize, n as usize)
        }
        _ => {
            return Err(err(
                "randi: up to 3 arguments supported.",
                "randi: حداکثر ۳ آرگومان.",
            ))
        }
    };
    let mut out = Matrix::try_full(rows, cols, 0.0)?; // بودجهٔ حافظه (ROADMAP 2.5) — ابعاد از کاربر
    for idx in 0..out.numel() {
        out.set_linear(idx, num::rng::uniform_int_in(lo as u64, hi as u64) as f64);
    }
    Ok(Value::Double(out))
}

/// randperm(n [, k]) — جایگشت ۱..n.
fn bi_randperm(args: &[Value], _n: usize) -> Result<Value, McError> {
    const NAME: &str = "randperm";
    let n = need_scalar(args, 0, NAME)?;
    let k = if args.len() > 1 {
        need_scalar(args, 1, NAME)?
    } else {
        n
    };
    if n < 0.0 || k < 0.0 || n.fract() != 0.0 || k.fract() != 0.0 || k > n {
        return Err(err(
            "randperm: needs integers with 0 ≤ k ≤ n.",
            "randperm: اعداد صحیح با 0 ≤ k ≤ n می‌خواهد.",
        ));
    }
    let perm = num::rng::permutation(n as usize, k as usize);
    Ok(Value::Double(Matrix::from_columns(1, perm.len(), perm)))
}

// ---------- چندجمله‌ای ----------

/// roots(p) — ماتریس همبند → ستون‌بردار مختلط n×1 (رفتار متلب).
fn bi_roots(args: &[Value], _n: usize) -> Result<Value, McError> {
    let p = need_matrix(args, 0, "roots")?;
    let z = num::poly::roots(&p)?;
    Ok(Value::Complex(z))
}

/// polyfit(x, y, n) — ضرایب نزولی، سطر 1×(n+1) (رفتار متلب).
fn bi_polyfit(args: &[Value], _n: usize) -> Result<Value, McError> {
    let x = need_matrix(args, 0, "polyfit")?;
    let y = need_matrix(args, 1, "polyfit")?;
    let d = need_scalar(args, 2, "polyfit")?;
    if d < 0.0 || d.fract() != 0.0 {
        return Err(err(
            "polyfit: degree must be a non-negative integer.",
            "polyfit: درجه باید صحیح نامنفی باشد.",
        ));
    }
    let p = num::poly::polyfit(&x, &y, d as usize)?;
    // سطری‌کردن (متلب سطر 1×(n+1) می‌دهد)
    Ok(Value::Double(Matrix::from_columns(
        1,
        p.numel(),
        (0..p.numel()).map(|k| p.get_linear(k)).collect(),
    )))
}

/// polyval(p, x) — Horner عنصری.
fn bi_polyval(args: &[Value], _n: usize) -> Result<Value, McError> {
    let p = need_matrix(args, 0, "polyval")?;
    let x = need_matrix(args, 1, "polyval")?;
    Ok(Value::Double(num::poly::polyval(&p, &x)?))
}

/// conv(u, v) — کانولوشن کامل.
fn bi_conv(args: &[Value], _n: usize) -> Result<Value, McError> {
    let u = need_matrix(args, 0, "conv")?;
    let v = need_matrix(args, 1, "conv")?;
    Ok(Value::Double(num::poly::conv(&u, &v)?))
}

/// compan(p) — ماتریس همبند (برای آموزش dispatch، سند ۰۴ §5.2).
fn bi_compan(args: &[Value], _n: usize) -> Result<Value, McError> {
    let p = need_matrix(args, 0, "compan")?;
    let coeffs: Vec<f64> = (0..p.numel()).map(|k| p.get_linear(k)).collect();
    let trimmed: &[f64] = {
        let first = coeffs
            .iter()
            .position(|c| *c != 0.0 && !c.is_nan())
            .unwrap_or(coeffs.len());
        &coeffs[first..]
    };
    let n = trimmed.len().saturating_sub(1);
    if n == 0 {
        return Err(err(
            "compan: needs a polynomial of degree ≥ 1.",
            "compan: چندجمله‌ای با درجه دست‌کم ۱ می‌خواهد.",
        ));
    }
    let p0 = trimmed[0];
    let mut c = Matrix::full(n, n, 0.0);
    for j in 0..n {
        c.set(0, j, -trimmed[j + 1] / p0);
    }
    for i in 1..n {
        c.set(i, i - 1, 1.0);
    }
    Ok(Value::Double(c))
}

// ---------- درون‌یابی ----------

/// interp1(x, v, xq [, method] [, 'extrap']) — سند ۰۴ §6.
fn bi_interp1(args: &[Value], _n: usize) -> Result<Value, McError> {
    const NAME: &str = "interp1";
    if args.len() < 3 {
        return Err(err(
            "interp1: needs X, V and XQ.",
            "interp1: به X و V و XQ نیاز دارد.",
        ));
    }
    let x = need_matrix(args, 0, NAME)?;
    let v = need_matrix(args, 1, NAME)?;
    let xq = need_matrix(args, 2, NAME)?;
    let mut method = num::interp::Method::Linear;
    let mut extrap = false;
    let mut extrapval: Option<f64> = None;
    for a in &args[3..] {
        match a {
            Value::Char(s) => {
                if s == "extrap" {
                    extrap = true;
                } else if let Some(m) = num::interp::Method::from_name(s) {
                    method = m;
                } else {
                    return Err(err(
                        "interp1: unknown method (linear | nearest | pchip | spline).",
                        "interp1: روش ناشناخته (linear | nearest | pchip | spline).",
                    ));
                }
            }
            Value::Double(m) if m.is_scalar() && extrapval.is_none() => {
                extrapval = Some(m.get(0, 0));
            }
            _ => {
                return Err(err(
                    "interp1: extra arguments must be a method name, 'extrap' or a value.",
                    "interp1: آرگومان‌های اضافی باید نام روش، 'extrap' یا مقدار باشند.",
                ))
            }
        }
    }
    num::interp::require_min_points(x.numel(), method)?;
    let inp = num::interp::prepare_interp1(&x, &v, &xq, method, extrap, extrapval)?;
    Ok(Value::Double(num::interp::interp1_eval(&inp)))
}

/// interp2 — درون‌یابی دوبعدی linear/nearest (سند ۰۴ §6، ROADMAP 2.6).
///
/// فرم‌های متلب:
/// - `interp2(X, Y, V, Xq, Yq, ...)` — V ابعاد numel(Y)×numel(X)
/// - `interp2(V, Xq, Yq, ...)` — محورهای ضمنی X=1:n، Y=1:m
///
/// گزینه‌ها: `'linear'` (پیش‌فرض) / `'nearest'`، پرچم `'extrap'`، مقدار extrapval.
/// Xq سطری + Yq ستونی = گسترش meshgrid (خروجی numel(Yq)×numel(Xq))؛
/// در غیر این صورت Xq/Yq باید هم‌شکل باشند. spline/cubic دوبعدی = E7.
fn bi_interp2(args: &[Value], _n: usize) -> Result<Value, McError> {
    const NAME: &str = "interp2";
    if args.len() < 3 {
        return Err(err(
            "interp2: needs V, Xq, Yq (or X, Y, V, Xq, Yq).",
            "interp2: به V و Xq و Yq (یا X و Y و V و Xq و Yq) نیاز دارد.",
        ));
    }
    let opt_start;
    let (x, y, v, xq, yq): (Matrix, Matrix, Matrix, Matrix, Matrix) = if args.len() >= 5 {
        opt_start = 5;
        (
            need_matrix(args, 0, NAME)?,
            need_matrix(args, 1, NAME)?,
            need_matrix(args, 2, NAME)?,
            need_matrix(args, 3, NAME)?,
            need_matrix(args, 4, NAME)?,
        )
    } else {
        opt_start = 3;
        let v = need_matrix(args, 0, NAME)?;
        // محورهای ضمنی متلب: X = 1:n (ستون‌ها)، Y = 1:m (سطرها)
        let xs: Vec<f64> = (1..=v.cols()).map(|k| k as f64).collect();
        let ys: Vec<f64> = (1..=v.rows()).map(|k| k as f64).collect();
        (
            Matrix::from_columns(1, xs.len(), xs),
            Matrix::from_columns(1, ys.len(), ys),
            v,
            need_matrix(args, 1, NAME)?,
            need_matrix(args, 2, NAME)?,
        )
    };
    let mut method = num::interp::Method2::Linear;
    let mut extrap = false;
    let mut extrapval: Option<f64> = None;
    for a in &args[opt_start..] {
        match a {
            Value::Char(s) => {
                if s == "extrap" {
                    extrap = true;
                } else if let Some(m) = num::interp::Method2::from_name(s) {
                    method = m;
                } else {
                    return Err(err(
                        "interp2: unknown method (linear | nearest) — 2-D spline/cubic is out of scope for v1 (expansion E7).",
                        "interp2: روش ناشناخته (linear | nearest) — spline/cubic دوبعدی خارج از محدودهٔ v1 است (گسترش E7).",
                    ));
                }
            }
            Value::Double(m) if m.is_scalar() && extrapval.is_none() => {
                extrapval = Some(m.get(0, 0));
            }
            _ => {
                return Err(err(
                    "interp2: extra arguments must be a method name, 'extrap' or a value.",
                    "interp2: آرگومان‌های اضافی باید نام روش، 'extrap' یا مقدار باشند.",
                ))
            }
        }
    }
    let grid = num::interp::prepare_interp2(&x, &y, &v, method, extrap, extrapval)?;
    // شکل پرسش‌ها — سمانتیک متلب
    let (out_rows, out_cols, xq_flat, yq_flat) = if xq.rows() == 1
        && yq.cols() == 1
        && xq.cols() > 0
        && yq.rows() > 0
    {
        // سطری + ستونی → گسترش meshgrid: خروجی numel(Yq)×numel(Xq)
        // Q(r,c) = V در (Xq(c)، Yq(r)) — تولید جفت‌ها ستون‌محور تا با
        // set_linear ستون‌محور (کولون-محور) هم‌تراز بماند.
        let m = yq.rows();
        let n = xq.cols();
        let mut xs = Vec::with_capacity(m * n);
        let mut ys = Vec::with_capacity(m * n);
        for c in 0..n {
            for r in 0..m {
                xs.push(xq.get(0, c));
                ys.push(yq.get(r, 0));
            }
        }
        (m, n, xs, ys)
    } else if xq.rows() == yq.rows() && xq.cols() == yq.cols() {
        let xs: Vec<f64> = (0..xq.numel()).map(|k| xq.get_linear(k)).collect();
        let ys: Vec<f64> = (0..yq.numel()).map(|k| yq.get_linear(k)).collect();
        (xq.rows(), xq.cols(), xs, ys)
    } else {
        return Err(err(
                "interp2: Xq and Yq must be the same size, or Xq a row vector and Yq a column vector (meshgrid expansion).",
                "interp2: Xq و Yq باید هم‌شکل باشند، یا Xq سطری و Yq ستونی (گسترش meshgrid).",
            ));
    };
    let vals = num::interp::interp2_eval(&grid, &xq_flat, &yq_flat);
    let mut out = Matrix::full(out_rows, out_cols, f64::NAN);
    for (k, val) in vals.iter().enumerate() {
        out.set_linear(k, *val);
    }
    Ok(Value::Double(out))
}

// ---------- ریشه/انتگرال ----------

/// fzero(f, x0) — x0 اسکالر یا براکت [a b].
fn bi_fzero(args: &[Value], _n: usize) -> Result<Value, McError> {
    const NAME: &str = "fzero";
    if args.len() < 2 {
        return Err(err(
            "fzero: needs a function handle and a start point or bracket.",
            "fzero: به هندل تابع و نقطهٔ شروع یا براکت نیاز دارد.",
        ));
    }
    let f = need_fn(args, 0, NAME)?.clone();
    let mut call = |x: f64| -> Result<f64, McError> {
        let v = (f.call)(&[scalar_val(x)])?;
        match v {
            Value::Double(m) if m.is_scalar() => Ok(m.get(0, 0)),
            Value::Complex(z) if z.is_scalar() && z.is_all_real() => Ok(z.get(0, 0).0),
            _ => Err(err(
                "fzero: function must return a scalar.",
                "fzero: تابع کاربر باید اسکالر برگرداند.",
            )),
        }
    };
    let r = match args.get(1) {
        Some(Value::Double(m)) if m.numel() == 2 => {
            let a = m.get_linear(0);
            let b = m.get_linear(1);
            num::solvers::fzero_bracket(&mut call, a, b, f64::EPSILON)?
        }
        Some(Value::Double(m)) if m.is_scalar() => {
            let x0 = m.get(0, 0);
            num::solvers::fzero_scalar(&mut call, x0, f64::EPSILON)?
        }
        _ => {
            return Err(err(
                "fzero: second argument must be a scalar or a 2-element bracket.",
                "fzero: آرگومان دوم باید اسکالر یا براکت دومعیاری باشد.",
            ))
        }
    };
    Ok(scalar_val(r.x))
}

/// integral(f, a, b [, 'RelTol', rt] [, 'AbsTol', at] [, 'Waypoints', wp])
/// — G7/K15 تطبیقی؛ بازه‌های بی‌نهایت با تبدیل متغیر (سند ۰۴ §5.3، ROADMAP 2.6).
fn bi_integral(args: &[Value], _n: usize) -> Result<Value, McError> {
    const NAME: &str = "integral";
    if args.len() < 3 {
        return Err(err(
            "integral: needs a function handle and two bounds.",
            "integral: به هندل تابع و دو مرز نیاز دارد.",
        ));
    }
    let f = need_fn(args, 0, NAME)?.clone();
    let a = need_scalar(args, 1, NAME)?;
    let b = need_scalar(args, 2, NAME)?;
    // گزینه‌های نامی (گام 2.6 — سطر integral جدول §4.5 COMPATIBILITY بسته شد)
    let mut reltol = 1e-10;
    let mut abstol = 1e-10;
    let mut waypoints: Option<Vec<f64>> = None;
    let mut i = 3;
    while i + 1 < args.len() {
        let key = need_char(args, i, NAME)?;
        match key.as_str() {
            "RelTol" => {
                let v = need_scalar(args, i + 1, NAME)?;
                if !v.is_finite() || v <= 0.0 {
                    return Err(err(
                        "integral: RelTol must be a positive finite scalar.",
                        "integral: RelTol باید اسکالر متناهیِ مثبت باشد.",
                    ));
                }
                reltol = v;
            }
            "AbsTol" => {
                let v = need_scalar(args, i + 1, NAME)?;
                if !v.is_finite() || v < 0.0 {
                    return Err(err(
                        "integral: AbsTol must be a non-negative finite scalar.",
                        "integral: AbsTol باید اسکالر متناهیِ نامنفی باشد.",
                    ));
                }
                abstol = v;
            }
            "Waypoints" => {
                let m = need_matrix(args, i + 1, NAME)?;
                let wp: Vec<f64> = (0..m.numel()).map(|k| m.get_linear(k)).collect();
                if wp.is_empty() {
                    return Err(err(
                        "integral: Waypoints must be a non-empty vector of finite numbers.",
                        "integral: Waypoints باید برداری غیرخالی از اعداد متناهی باشد.",
                    ));
                }
                waypoints = Some(wp);
            }
            _ => {
                return Err(err(
                    "integral: unsupported option (RelTol | AbsTol | Waypoints).",
                    "integral: گزینهٔ پشتیبانی‌نشده (RelTol | AbsTol | Waypoints).",
                ))
            }
        }
        i += 2;
    }
    if i < args.len() {
        return Err(err(
            "integral: options must come in name/value pairs.",
            "integral: گزینه‌ها باید جفت نام/مقدار باشند.",
        ));
    }
    let mut call = |x: f64| -> Result<f64, McError> {
        let v = (f.call)(&[scalar_val(x)])?;
        match v {
            Value::Double(m) if m.is_scalar() => Ok(m.get(0, 0)),
            Value::Complex(z) if z.is_scalar() && z.is_all_real() => Ok(z.get(0, 0).0),
            _ => Err(err(
                "integral: function must return a scalar.",
                "integral: تابع کاربر باید اسکالر برگرداند.",
            )),
        }
    };
    let bounds = num::solvers::bounds_from(a, b)?;
    let r = match &waypoints {
        None => num::solvers::integrate(&mut call, bounds, reltol, abstol)?,
        Some(wp) => num::solvers::integrate_waypoints(&mut call, bounds, wp, reltol, abstol)?,
    };
    Ok(scalar_val(r.value))
}

// ---------- آمار ----------

/// کاهش ستونی (dim=1 متلب) — روی بردار → اسکالر، روی ماتریس → سطر 1×n.
fn column_reduce(m: &Matrix, name: &str, f: impl Fn(&[f64]) -> f64) -> Result<Value, McError> {
    if m.rows() == 1 || m.cols() == 1 {
        let vals: Vec<f64> = (0..m.numel()).map(|k| m.get_linear(k)).collect();
        return Ok(scalar_val(f(&vals)));
    }
    let mut out = Matrix::full(1, m.cols(), 0.0);
    for c in 0..m.cols() {
        let col: Vec<f64> = (0..m.rows()).map(|r| m.get(r, c)).collect();
        out.set(0, c, f(&col));
    }
    let _ = name;
    Ok(Value::Double(out))
}

fn bi_median(args: &[Value], _n: usize) -> Result<Value, McError> {
    let m = need_matrix(args, 0, "median")?;
    let omit = has_flag(args, 1, "omitnan");
    column_reduce(&m, "median", |v| num::stats::median_flagged(v, omit))
}

fn bi_std(args: &[Value], _n: usize) -> Result<Value, McError> {
    let m = need_matrix(args, 0, "std")?;
    let w = if args.len() > 1 && matches!(args.get(1), Some(Value::Double(mm)) if mm.is_scalar()) {
        need_scalar(args, 1, "std")? != 0.0
    } else {
        false
    };
    let omit = has_flag(args, 2, "omitnan") || has_flag(args, 1, "omitnan");
    column_reduce(&m, "std", |v| num::stats::std_flagged(v, w, omit))
}

fn bi_var(args: &[Value], _n: usize) -> Result<Value, McError> {
    let m = need_matrix(args, 0, "var")?;
    let w = if args.len() > 1 && matches!(args.get(1), Some(Value::Double(mm)) if mm.is_scalar()) {
        need_scalar(args, 1, "var")? != 0.0
    } else {
        false
    };
    let omit = has_flag(args, 2, "omitnan") || has_flag(args, 1, "omitnan");
    column_reduce(&m, "var", |v| num::stats::var_flagged(v, w, omit))
}

/// پرچم 'omitnan' در موقعیت داده‌شده.
fn has_flag(args: &[Value], i: usize, flag: &str) -> bool {
    matches!(args.get(i), Some(Value::Char(s)) if s == flag)
}

fn bi_cov(args: &[Value], _n: usize) -> Result<Value, McError> {
    if args.len() >= 2 && matches!(args.get(1), Some(Value::Double(_))) {
        let x = need_matrix(args, 0, "cov")?;
        let y = need_matrix(args, 1, "cov")?;
        Ok(Value::Double(num::stats::cov2(&x, &y)?))
    } else {
        let x = need_matrix(args, 0, "cov")?;
        Ok(Value::Double(num::stats::cov(&x)?))
    }
}

fn bi_corrcoef(args: &[Value], _n: usize) -> Result<Value, McError> {
    if args.len() >= 2 && matches!(args.get(1), Some(Value::Double(_))) {
        let x = need_matrix(args, 0, "corrcoef")?;
        let y = need_matrix(args, 1, "corrcoef")?;
        Ok(Value::Double(num::stats::corrcoef2(&x, &y)?))
    } else {
        let x = need_matrix(args, 0, "corrcoef")?;
        Ok(Value::Double(num::stats::corrcoef(&x)?))
    }
}

// ---------- مختلط: عنصری مختلط-آگاه ----------

/// عنصری روی Value (حقیقی یا مختلط) با خروجی حقیقی — real/imag/abs/angle
/// (رفتار متلب: این‌ها همیشه REAL برمی‌گردانند، حتی روی ورودی مختلط).
fn elem_apply_real(
    args: &[Value],
    fr: fn(f64) -> f64,
    fc: impl Fn(f64, f64) -> f64,
) -> Result<Value, McError> {
    const NAME: &str = "elementwise";
    match args.first() {
        Some(Value::Complex(z)) => {
            let mut out = Matrix::full(z.rows(), z.cols(), 0.0);
            for idx in 0..z.numel() {
                let (re, im) = z.get_linear(idx);
                out.set_linear(idx, fc(re, im));
            }
            Ok(Value::Double(out))
        }
        Some(Value::Double(m)) => {
            let mut out = Matrix::full(m.rows(), m.cols(), 0.0);
            for idx in 0..m.numel() {
                out.set_linear(idx, fr(m.get_linear(idx)));
            }
            Ok(Value::Double(out))
        }
        Some(Value::Char(s)) => {
            let mut ch = s.chars();
            match (ch.next(), ch.next()) {
                (Some(c), None) => Ok(scalar_val(fr(c as u32 as f64))),
                _ => Err(err(
                    "Only single-character strings work in numeric functions.",
                    "در توابع عددی فقط رشتهٔ تک‌کاراکتری کار می‌کند.",
                )),
            }
        }
        _ => Err(err(
            &format!("{NAME}: numeric argument expected."),
            &format!("{NAME}: آرگومان عددی انتظار می‌رود."),
        )),
    }
}

/// عنصری با خروجی مختلط — فقط conj (و آینده: توابع مختلط دیگر).
fn elem_apply_complex(
    args: &[Value],
    fr: fn(f64) -> f64,
    fc: impl Fn(f64, f64) -> (f64, f64),
) -> Result<Value, McError> {
    match args.first() {
        Some(Value::Complex(z)) => {
            let mut out = ComplexMatrix::full(z.rows(), z.cols(), 0.0, 0.0);
            for idx in 0..z.numel() {
                let (re, im) = z.get_linear(idx);
                let (ore, oim) = fc(re, im);
                out.set_linear(idx, ore, oim);
            }
            Ok(Value::Complex(out))
        }
        Some(Value::Double(m)) => {
            let mut out = Matrix::full(m.rows(), m.cols(), 0.0);
            for idx in 0..m.numel() {
                out.set_linear(idx, fr(m.get_linear(idx)));
            }
            Ok(Value::Double(out))
        }
        _ => Err(err(
            "elementwise: numeric argument expected.",
            "elementwise: آرگومان عددی انتظار می‌رود.",
        )),
    }
}

fn bi_real(args: &[Value], _n: usize) -> Result<Value, McError> {
    elem_apply_real(args, |x| x, |re, _| re)
}

fn bi_imag(args: &[Value], _n: usize) -> Result<Value, McError> {
    elem_apply_real(args, |_| 0.0, |_, im| im)
}

fn bi_conj(args: &[Value], _n: usize) -> Result<Value, McError> {
    elem_apply_complex(args, |x| x, |re, im| (re, -im))
}

fn bi_abs(args: &[Value], _n: usize) -> Result<Value, McError> {
    elem_apply_real(args, |x| x.abs(), |re, im| (re * re + im * im).sqrt())
}

fn bi_angle(args: &[Value], _n: usize) -> Result<Value, McError> {
    elem_apply_real(
        args,
        |x| {
            if x < 0.0 {
                std::f64::consts::PI
            } else {
                0.0
            }
        },
        |re, im| im.atan2(re),
    )
}

fn bi_isreal(args: &[Value], _n: usize) -> Result<Value, McError> {
    match args.first() {
        Some(Value::Complex(z)) => Ok(scalar_val(if z.is_all_real() { 1.0 } else { 0.0 })),
        Some(Value::Double(_)) | Some(Value::Char(_)) => Ok(scalar_val(1.0)),
        _ => Ok(scalar_val(0.0)),
    }
}

/// complex(a, b) — ساخت مختلط از بخش‌ها؛ complex(z) = z مختلط‌سازی.
fn bi_complex(args: &[Value], _n: usize) -> Result<Value, McError> {
    if args.is_empty() {
        return Err(err(
            "complex: needs one or two numeric arguments.",
            "complex: به یک یا دو آرگومان عددی نیاز دارد.",
        ));
    }
    let re = cx::to_complex(&args[0], 0, 0)?;
    if args.len() == 1 {
        // complex(z): اگر حقیقی بود → موهومی صفر (پرچم مختلط)
        return Ok(Value::Complex(re));
    }
    let im = cx::to_complex(&args[1], 0, 0)?;
    let im_i = cx::c_elem_mul(&im, &ComplexMatrix::imaginary_unit())?;
    let z = cx::c_add(&re, &im_i)?;
    Ok(Value::Complex(z))
}

// ---------- histcounts/histogram (سند ۰۴ §7، ROADMAP 2.6) ----------

/// بردار تخت از هر ماتریس — برای شمارش (شکل اصلی برای خروجی bin حفظ می‌شود).
fn flat_any(m: &Matrix) -> Vec<f64> {
    (0..m.numel()).map(|k| m.get_linear(k)).collect()
}

fn row_value(vals: &[f64]) -> Value {
    Value::Double(Matrix::from_columns(1, vals.len(), vals.to_vec()))
}

/// خروجی‌های مشترک histcounts/histogram از مرزهای آماده.
fn hist_outputs(
    x: &[f64],
    shape: (usize, usize),
    edges: &[f64],
    nargout: usize,
    max_out: usize,
    name: &str,
) -> Result<Vec<Value>, McError> {
    if nargout > max_out {
        return Err(err(
            &format!("{name} produces at most {max_out} outputs."),
            &format!("«{name}» حداکثر {max_out} خروجی دارد."),
        ));
    }
    let (counts, bin) = num::stats::counts_for_edges(x, edges);
    let mut outs = vec![row_value(&counts)];
    if nargout >= 2 {
        outs.push(row_value(edges));
    }
    if nargout >= 3 {
        // اندیس بین هر عنصر — هم‌شکل X (0 = بیرون مرزها؛ قرارداد متلب)
        let mut b = Matrix::full(shape.0, shape.1, 0.0);
        for (k, val) in bin.iter().enumerate() {
            b.set_linear(k, *val);
        }
        outs.push(Value::Double(b));
    }
    Ok(outs)
}

/// [N, edges, bin] = histcounts(X [, nbins | edges])
/// — N سطر 1×nbins، edges سطر 1×(nbins+1)، bin هم‌شکل X.
fn bi_histcounts(args: &[Value], nargout: usize) -> Result<Vec<Value>, McError> {
    const NAME: &str = "histcounts";
    if args.is_empty() {
        return Err(err(
            "histcounts: needs a data vector or matrix.",
            "histcounts: به داده (بردار یا ماتریس) نیاز دارد.",
        ));
    }
    let xm = need_matrix(args, 0, NAME)?;
    let x = flat_any(&xm);
    let edges = match args.get(1) {
        None => num::stats::auto_edges(&x, num::stats::BinMethod::Auto)?,
        Some(Value::Double(m)) if m.is_scalar() => {
            let nb = m.get(0, 0);
            if nb < 1.0 || nb.fract() != 0.0 || !nb.is_finite() {
                return Err(err(
                    "histcounts: nbins must be a positive integer.",
                    "histcounts: nbins باید عدد صحیح مثبت باشد.",
                ));
            }
            let fin: Vec<f64> = x.iter().copied().filter(|v| v.is_finite()).collect();
            if fin.is_empty() {
                vec![0.0, 1.0]
            } else {
                let lo = fin.iter().copied().fold(f64::INFINITY, f64::min);
                let hi = fin.iter().copied().fold(f64::NEG_INFINITY, f64::max);
                if lo == hi {
                    vec![lo - 0.5, hi + 0.5]
                } else {
                    num::stats::edges_for_nbins(lo, hi, nb as usize)
                }
            }
        }
        Some(Value::Double(m)) => {
            let e = flat_any(m);
            num::stats::validate_edges(&e)?;
            e
        }
        Some(_) => {
            return Err(err(
                "histcounts: second argument must be a bin count or an edges vector.",
                "histcounts: آرگومان دوم باید تعداد بین یا بردار مرزها باشد.",
            ))
        }
    };
    hist_outputs(&x, (xm.rows(), xm.cols()), &edges, nargout, 3, NAME)
}

/// [N, edges] = histogram(X, 'BinMethod', m | 'BinWidth', w | 'BinLimits', [a b])
///
/// انحراف مستند (COMPATIBILITY §4.5): متلب آبجکت Histogram می‌سازد و رسم
/// می‌کند — رسم در P3 (گام 3.2) می‌آید؛ هستهٔ عددی همین‌جا ارائه می‌شود و
/// شکل خروجی [counts، edges] است.
fn bi_histogram(args: &[Value], nargout: usize) -> Result<Vec<Value>, McError> {
    const NAME: &str = "histogram";
    if args.is_empty() {
        return Err(err(
            "histogram: needs a data vector or matrix.",
            "histogram: به داده (بردار یا ماتریس) نیاز دارد.",
        ));
    }
    let xm = need_matrix(args, 0, NAME)?;
    let x = flat_any(&xm);
    let fin: Vec<f64> = x.iter().copied().filter(|v| v.is_finite()).collect();
    let mut method = num::stats::BinMethod::Auto;
    let mut bin_width: Option<f64> = None;
    let mut limits: Option<(f64, f64)> = None;
    // فرم دوم متلب: histogram(X, edges) — مرزهای صریح به‌جای گزینه‌ها
    let explicit_edges: Option<Vec<f64>> = match args.get(1) {
        Some(Value::Double(m)) if !m.is_scalar() => {
            let e = flat_any(m);
            num::stats::validate_edges(&e)?;
            Some(e)
        }
        Some(Value::Double(m)) if m.is_scalar() => {
            return Err(err(
                "histogram: a scalar bin count is not supported — use 'BinWidth' or 'BinLimits' options.",
                "histogram: تعداد بینِ اسکالر پشتیبانی نمی‌شود — از گزینه‌های «BinWidth» یا «BinLimits» استفاده کنید.",
            ))
        }
        _ => None,
    };
    let mut i = if explicit_edges.is_some() { 2 } else { 1 };
    while i + 1 < args.len() {
        let key = need_char(args, i, NAME)?;
        match key.as_str() {
            "BinMethod" => {
                let m = need_char(args, i + 1, NAME)?;
                match num::stats::BinMethod::from_name(&m) {
                    Some(bm) => method = bm,
                    None => {
                        return Err(err(
                            "histogram: unknown BinMethod (auto | scott | fd | sturges | sqrt | integers).",
                            "histogram: BinMethod ناشناخته (auto | scott | fd | sturges | sqrt | integers).",
                        ))
                    }
                }
            }
            "BinWidth" => {
                let w = need_scalar(args, i + 1, NAME)?;
                if !w.is_finite() || w <= 0.0 {
                    return Err(err(
                        "histogram: BinWidth must be a positive finite scalar.",
                        "histogram: BinWidth باید اسکالر متناهیِ مثبت باشد.",
                    ));
                }
                bin_width = Some(w);
            }
            "BinLimits" => {
                let m = need_matrix(args, i + 1, NAME)?;
                if m.numel() != 2 {
                    return Err(err(
                        "histogram: BinLimits must be a two-element vector [a b].",
                        "histogram: BinLimits باید بردار دومعیاری [a b] باشد.",
                    ));
                }
                let (a, b) = (m.get_linear(0), m.get_linear(1));
                if !a.is_finite() || !b.is_finite() || a >= b {
                    return Err(err(
                        "histogram: BinLimits must be finite with a < b.",
                        "histogram: BinLimits باید متناهی و با a < b باشد.",
                    ));
                }
                limits = Some((a, b));
            }
            _ => {
                return Err(err(
                    "histogram: unsupported option (BinMethod | BinWidth | BinLimits).",
                    "histogram: گزینهٔ پشتیبانی‌نشده (BinMethod | BinWidth | BinLimits).",
                ))
            }
        }
        i += 2;
    }
    if i < args.len() {
        return Err(err(
            "histogram: options must come in name/value pairs.",
            "histogram: گزینه‌ها باید جفت نام/مقدار باشند.",
        ));
    }
    let edges = match explicit_edges {
        Some(e) => e,
        None => match (bin_width, limits) {
            // عرض صریح — مرزها از مضرب w ≤ شروع تا پوششِ سقف محدوده
            (Some(w), Some((a, b))) => num::stats::edges_for_width(a, b, w),
            (Some(w), None) => {
                let lo = fin.iter().copied().fold(f64::INFINITY, f64::min);
                let hi = fin.iter().copied().fold(f64::NEG_INFINITY, f64::max);
                if lo.is_infinite() {
                    vec![0.0, 1.0]
                } else {
                    num::stats::edges_for_width(lo, hi, w)
                }
            }
            // محدودهٔ صریح بدون عرض — روش روی دادهٔ بریده‌شده به [a,b]، سپس برش
            // مرزها به محدوده (قرارداد مستند؛ متلب مرزها را به BinLimits می‌چسباند)
            (None, Some((a, b))) => {
                let clamped: Vec<f64> = fin.iter().copied().filter(|&v| v >= a && v <= b).collect();
                let e = num::stats::auto_edges(&clamped, method)?;
                let mut e: Vec<f64> = e.into_iter().filter(|&t| t >= a && t <= b).collect();
                if e.len() < 2 {
                    e = vec![a, b];
                }
                e
            }
            // روش خالص (پیش‌فرض auto) — مرزها روی خود داده
            (None, None) => num::stats::auto_edges(&x, method)?,
        },
    };
    hist_outputs(&x, (xm.rows(), xm.cols()), &edges, nargout, 2, NAME)
}

// ---------- چندخروجی‌ها ----------

/// [T, Y] = ode45(f, tspan, y0 [, 'RelTol', rt, 'AbsTol', at, ...])
/// nargout=1 → sol = [T Y] (n×(1+neq)) — رفتار متلب.
fn bi_ode45(args: &[Value], nargout: usize) -> Result<Vec<Value>, McError> {
    ode_driver(args, nargout, OdeMethod::Dp54)
}

fn bi_ode23(args: &[Value], nargout: usize) -> Result<Vec<Value>, McError> {
    ode_driver(args, nargout, OdeMethod::Bs32)
}

enum OdeMethod {
    Dp54,
    Bs32,
}

fn ode_driver(args: &[Value], nargout: usize, method: OdeMethod) -> Result<Vec<Value>, McError> {
    const NAME: &str = "ode";
    let fname = match method {
        OdeMethod::Dp54 => "ode45",
        OdeMethod::Bs32 => "ode23",
    };
    if args.len() < 3 {
        return Err(err(
            &format!("{fname}: needs f, tspan and y0."),
            &format!("{fname}: به f و tspan و y0 نیاز دارد."),
        ));
    }
    let f = need_fn(args, 0, NAME)?.clone();
    let tspan = need_vector(args, 1, NAME)?;
    if tspan.len() < 2 {
        return Err(err(
            &format!("{fname}: tspan must have at least two elements."),
            &format!("{fname}: tspan دست‌کم دو عنصر می‌خواهد."),
        ));
    }
    let y0v = need_matrix(args, 2, NAME)?;
    if y0v.rows() != 1 && y0v.cols() != 1 {
        return Err(err(
            &format!("{fname}: y0 must be a vector."),
            &format!("{fname}: y0 باید بردار باشد."),
        ));
    }
    let y0: Vec<f64> = (0..y0v.numel()).map(|k| y0v.get_linear(k)).collect();

    // گزینه‌های name/value از آرگومان چهارم (انحراف مستند از odeset — سند ۰۴ §4)
    let mut opts = num::solvers::OdeOpts::default();
    let mut i = 3;
    while i + 1 < args.len() {
        #[allow(unused_assignments)]
        let key = need_char(args, i, fname)?;
        let val = need_scalar(args, i + 1, fname)?;
        match key.as_str() {
            "RelTol" => opts.reltol = val,
            "AbsTol" => opts.abstol = val,
            "MaxStep" => opts.max_step = val,
            "InitialStep" => opts.initial_step = Some(val),
            _ => {
                return Err(err(
                    &format!(
                        "{fname}: unsupported option '{key}' (RelTol | AbsTol | MaxStep | InitialStep)."
                    ),
                    &format!(
                        "{fname}: گزینهٔ پشتیبانی‌نشدهٔ «{key}» (RelTol | AbsTol | MaxStep | InitialStep)."
                    ),
                ))
            }
        }
        i += 2;
    }
    if i < args.len() {
        return Err(err(
            &format!("{fname}: options must come in name/value pairs."),
            &format!("{fname}: گزینه‌ها باید جفت نام/مقدار باشند."),
        ));
    }

    // پوشش تابع کاربر — OdeFn
    let f2 = f.clone();
    let mut rhs = move |t: f64, y: &[f64]| -> Result<Vec<f64>, McError> {
        let col = Matrix::from_columns(y.len(), 1, y.to_vec());
        let out = (f2.call)(&[scalar_val(t), Value::Double(col)])?;
        let v = value_to_vec(&out, fname)?;
        if v.len() != y.len() {
            return Err(err(
                &format!(
                    "{fname}: function returned {} components, expected {}.",
                    v.len(),
                    y.len()
                ),
                &format!(
                    "{fname}: تابع {} مؤلفه برگرداند، {} انتظار می‌رفت.",
                    v.len(),
                    y.len()
                ),
            ));
        }
        Ok(v)
    };

    // tspan چندعنصری → خروجی دقیق روی نقاط (سمانتیک متلب)
    let (tspan_pair, tout): ((f64, f64), Option<Vec<f64>>) = if tspan.len() == 2 {
        ((tspan[0], tspan[1]), None)
    } else {
        ((tspan[0], tspan[tspan.len() - 1]), Some(tspan.clone()))
    };

    // پرچم لغو از کانتکست job (ROADMAP 2.3، سند ۰۲ §3–4): mc-ffi پیش از
    // ارزیابی کانتکست نصب می‌کند؛ بدون نشست فعال (تست‌ها/CLI) None می‌ماند.
    // Arc تا پایان تابع زنده است → استقراض Option<&AtomicBool> امن است.
    let job_ctx = mc_core::job::current();
    let stop = job_ctx.as_ref().map(|j| &j.cancel);

    let res = match method {
        OdeMethod::Dp54 => num::solvers::ode45(
            &mut rhs,
            &[tspan_pair.0, tspan_pair.1],
            y0,
            tout.as_deref(),
            &opts,
            stop,
        )?,
        OdeMethod::Bs32 => num::solvers::ode23(
            &mut rhs,
            &[tspan_pair.0, tspan_pair.1],
            y0,
            tout.as_deref(),
            &opts,
            stop,
        )?,
    };

    let n = res.t.len();
    let neq = res.y.first().map(|y| y.len()).unwrap_or(0);
    let t_col = Matrix::from_columns(n, 1, res.t.clone());
    let mut y_mat = Matrix::full(n, neq, 0.0);
    for (k, row) in res.y.iter().enumerate() {
        for (j, val) in row.iter().enumerate() {
            y_mat.set(k, j, *val);
        }
    }
    match nargout {
        1 => {
            // sol = [T Y] — n×(1+neq)
            let mut sol = Matrix::full(n, 1 + neq, 0.0);
            for k in 0..n {
                sol.set(k, 0, res.t[k]);
                for j in 0..neq {
                    sol.set(k, 1 + j, y_mat.get(k, j));
                }
            }
            Ok(vec![Value::Double(sol)])
        }
        _ => Ok(vec![Value::Double(t_col), Value::Double(y_mat)]),
    }
}

/// [q, r] = deconv(u, v)؛ تک‌خروجی = q.
fn bi_deconv(args: &[Value], nargout: usize) -> Result<Vec<Value>, McError> {
    let u = need_matrix(args, 0, "deconv")?;
    let v = need_matrix(args, 1, "deconv")?;
    let d = num::poly::deconv(&u, &v)?;
    if nargout == 1 {
        Ok(vec![Value::Double(d.quotient)])
    } else {
        Ok(vec![Value::Double(d.quotient), Value::Double(d.remainder)])
    }
}

/// [V, D] = eig(A) — V مختلط ستونی، D قطری مختلط؛ تک‌خروجی = ستون مقادیر.
fn bi_eig(args: &[Value], nargout: usize) -> Result<Vec<Value>, McError> {
    let a = need_matrix(args, 0, "eig")?;
    let (values, vectors) = num::eig_complex(&a)?;
    // مقادیر: ستون مختلط n×1 (متلب مقادیر را به‌صورت ستون می‌دهد)
    let n = values.len();
    let mut d_re = vec![0.0; n * n];
    let mut d_im = vec![0.0; n * n];
    for (i, (re, im)) in values.iter().enumerate() {
        d_re[i * n + i] = *re;
        d_im[i * n + i] = *im;
    }
    match nargout {
        1 => Ok(vec![Value::Complex(ComplexMatrix::from_columns(
            n,
            1,
            values.iter().map(|(r, _)| *r).collect(),
            values.iter().map(|(_, i)| *i).collect(),
        ))]),
        _ => Ok(vec![
            Value::Complex(vectors),
            Value::Complex(ComplexMatrix::from_columns(n, n, d_re, d_im)),
        ]),
    }
}

/// [Q, R] = qr(A) — قرارداد کامل متلب: Q یکانی m×m، R مثلثی بالایی m×n.
/// تک‌خروجی (هندل @qr / فراخوانی برهنه) = فقط R مثل متلب؛ مسیر اصلی تک‌خروجی
/// (registry) هم همین را می‌دهد (COMPATIBILITY §۳).
fn bi_qr_multi(args: &[Value], nargout: usize) -> Result<Vec<Value>, McError> {
    let a = need_matrix(args, 0, "qr")?;
    let (q, r) = num::qr(&a)?;
    if nargout == 1 {
        Ok(vec![Value::Double(r)])
    } else {
        Ok(vec![Value::Double(q), Value::Double(r)])
    }
}

/// [U, S, V] = svd(A) — U یکانی m×m، S قطری m×n، V یکانی n×n؛ σها نزولی.
/// تک‌خروجی = بردار ستونی σها (رفتار متلب s = svd(A)).
fn bi_svd_multi(args: &[Value], nargout: usize) -> Result<Vec<Value>, McError> {
    let a = need_matrix(args, 0, "svd")?;
    let (u, s, v) = num::svd(&a)?;
    if nargout == 1 {
        let mut col = Matrix::full(s.len(), 1, 0.0);
        for (i, x) in s.iter().enumerate() {
            col.set(i, 0, *x);
        }
        return Ok(vec![Value::Double(col)]);
    }
    let (m, n) = (a.rows(), a.cols());
    let mut sm = Matrix::full(m, n, 0.0);
    for (k, x) in s.iter().enumerate() {
        sm.set(k, k, *x);
    }
    Ok(vec![Value::Double(u), Value::Double(sm), Value::Double(v)])
}
