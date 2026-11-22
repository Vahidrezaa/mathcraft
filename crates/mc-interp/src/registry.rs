//! رجیستری توابع Built-in — خانواده‌های «ریشه/ماتریسی/عنصری» سند ۰۳ §12 (P1).
//!
//! قواعد:
//! - هر builtin آرگومان‌های ارزیابی‌شده را می‌گیرد (بدون lazy) و nargout می‌داند
//!   (توابع چندخروجی کامل در مایل‌استون multi-assign فعال می‌شوند — سند ۰۷ P1).
//! - هیچ panic/unwrap در مسیر محصول — همه خطاها McError دوزبانه (سند ۰۲ §9).
//! - الگوریتم‌های سنگین به mc-num واگذار می‌شوند (سند ۰۴ §1: نقشه ماژول‌ها).

use mc_core::{budget, ids, Matrix, McError, Value};
use mc_num as num;

/// امضای توابع داخلی — آرگومان‌ها + تعداد خروجی خواسته‌شده.
pub type Builtin = fn(&[Value], usize) -> Result<Value, McError>;

/// یافتن builtin به نام — REGISTRY اصلی + P2 (سند ۰۴ §4-7).
pub fn lookup(name: &str) -> Option<Builtin> {
    if let Some(f) = REGISTRY.iter().find(|(n, _)| *n == name).map(|(_, f)| *f) {
        return Some(f);
    }
    crate::registry_p2::lookup(name)
}

/// آیا نام در رجیستری هست؟ — برای did-you-mean و پیام‌های آموزشی.
pub fn contains(name: &str) -> bool {
    lookup(name).is_some()
}

/// پیشنهاد نزدیک‌ترین builtin برای did-you-mean (سند ۰۸: `lenght` → `length`).
/// آستانه Levenshtein ≤ 2 — همان قرارداد env.rs.
pub fn suggest_builtin(typo: &str) -> Option<String> {
    use crate::env::levenshtein;
    REGISTRY
        .iter()
        .map(|(n, _)| *n)
        .filter(|n| *n != typo)
        .filter(|n| levenshtein(typo, n) <= 2)
        .min_by_key(|n| levenshtein(typo, n))
        .map(String::from)
}

static REGISTRY: &[(&str, Builtin)] = &[
    // ---- سازه‌های ماتریسی (سند ۰۳ §12 «ریشه و سازه‌های ماتریسی») ----
    ("zeros", bi_zeros),
    ("ones", bi_ones),
    ("eye", bi_eye),
    ("linspace", bi_linspace),
    ("meshgrid", bi_meshgrid),
    ("peaks", bi_peaks),
    ("logspace", bi_logspace),
    ("diag", bi_diag),
    ("tril", bi_tril),
    ("triu", bi_triu),
    ("size", bi_size),
    ("length", bi_length),
    ("numel", bi_numel),
    ("isempty", bi_isempty),
    ("isvector", bi_isvector),
    ("isscalar", bi_isscalar),
    ("repmat", bi_repmat),
    ("reshape", bi_reshape),
    // ---- فایل/دادهٔ I/O (گام 3.7 — سند ۱۴ §۵.۱) ----
    ("readmatrix", bi_readmatrix),
    ("readtable", bi_readtable),
    // ---- جبر خطی (سند ۰۴ §3) ----
    ("inv", bi_inv),
    ("det", bi_det),
    ("rank", bi_rank),
    ("trace", bi_trace),
    ("rref", bi_rref),
    ("norm", bi_norm),
    ("cond", bi_cond),
    ("chol", bi_chol),
    ("qr", bi_qr),
    ("svd", bi_svd),
    ("eig", bi_eig),
    ("dot", bi_dot),
    ("cross", bi_cross),
    ("kron", bi_kron),
    // ---- عنصری (سند ۰۳ §12 «توابع عنصری») ----
    ("sqrt", el_sqrt),
    ("exp", el_exp),
    ("log", el_log),
    ("log2", el_log2),
    ("log10", el_log10),
    ("sin", el_sin),
    ("cos", el_cos),
    ("tan", el_tan),
    ("asin", el_asin),
    ("acos", el_acos),
    ("atan", el_atan),
    ("atan2", bi_atan2),
    ("sinh", el_sinh),
    ("cosh", el_cosh),
    ("tanh", el_tanh),
    ("sign", el_sign),
    ("round", el_round),
    ("floor", el_floor),
    ("ceil", el_ceil),
    ("fix", el_fix),
    ("mod", bi_mod),
    ("rem", bi_rem),
    // ---- کاهشی/آمار پایه ----
    ("sum", bi_sum),
    ("prod", bi_prod),
    ("max", bi_max),
    ("min", bi_min),
    ("cumsum", bi_cumsum),
    ("cumprod", bi_cumprod),
    ("diff", bi_diff),
    ("mean", bi_mean),
    ("all", bi_all),
    ("any", bi_any),
    ("find", bi_find),
    ("sort", bi_sort),
    // ---- منطقی/مقایسه‌ای (سند ۰۳ §12) ----
    ("true", bi_true),
    ("false", bi_false),
    ("isnan", el_isnan),
    ("isinf", el_isinf),
    ("isfinite", el_isfinite),
    ("isequal", bi_isequal),
    // ---- نمایش/متنی و ثابت‌ها (ROADMAP 2.4 — آیتم ۸ ممیزی) ----
    ("class", bi_class),
    ("num2str", bi_num2str),
    // ---- char/رشته (ROADMAP 2.8 — بند ۶/۱۲ ممیزی خارجی) ----
    ("char", bi_char),
    ("double", bi_double),
    // ---- سمبولیک CAS-0→CAS-1 (گام 4.1/4.2 — سند ۰۵؛ syms گزاره است در eval.rs) ----
    ("sym", bi_sym),
    ("subs", bi_subs),
    // CAS-1: diff دو-چهره است — آرگومان اول Sym → مسیر نمادین (sym_ops)،
    // وگرنه تفاضل عددی همان پایین؛ simplify/expand/collect/latex نمادین‌اند
    ("simplify", bi_simplify),
    ("expand", bi_expand),
    ("collect", bi_collect),
    ("latex", bi_latex),
    // CAS-2 (گام 4.3 — سند ۰۵ §۵/§۸): int/solve فقط نمادین‌اند — چهرهٔ
    // عددی انتگرال/ریشه: integral / roots / fzero (registry_p2). int
    // شکست نامعین → گرهٔ unevaluated؛ solve همیشه list (cell) می‌دهد
    ("int", bi_int),
    ("solve", bi_solve),
    ("ischar", bi_ischar),
    ("blanks", bi_blanks),
    ("str2double", bi_str2double),
    ("intmax", bi_intmax),
    ("realmax", bi_realmax),
    ("realmin", bi_realmin),
    // disp/fprintf به‌عنوان گزاره در eval.rs هندل می‌شوند (خروجی مستقل از
    // «;» و بدون مقدار) — اینجا فقط برای خطای صادقانه در بافت عبارت:
    ("disp", bi_display_no_value),
    ("plot", bi_plot_no_value),
    ("scatter", bi_plot_no_value),
    ("bar", bi_plot_no_value),
    ("barh", bi_plot_no_value),
    ("stem", bi_plot_no_value),
    ("stairs", bi_plot_no_value),
    ("semilogx", bi_plot_no_value),
    ("semilogy", bi_plot_no_value),
    ("loglog", bi_plot_no_value),
    ("xlim", bi_plot_no_value),
    ("ylim", bi_plot_no_value),
    ("xticks", bi_plot_no_value),
    ("yticks", bi_plot_no_value),
    ("histogram", bi_plot_no_value),
    ("area", bi_plot_no_value),
    ("fill", bi_plot_no_value),
    ("polarplot", bi_plot_no_value),
    ("errorbar", bi_plot_no_value),
    ("fplot", bi_plot_no_value),
    ("subplot", bi_plot_no_value),
    ("legend", bi_plot_no_value),
    ("axis", bi_plot_no_value),
    ("grid", bi_plot_no_value),
    ("xlabel", bi_plot_no_value),
    ("ylabel", bi_plot_no_value),
    ("title", bi_plot_no_value),
    ("hold", bi_plot_no_value),
    ("figure", bi_plot_no_value),
    ("fprintf", bi_display_no_value),
];

/// هسته مشترک توابع عنصری ۱-آرگومانی — روی همه عناصر ماتریس.
fn elem_apply(args: &[Value], f: fn(f64) -> Result<f64, McError>) -> Result<Value, McError> {
    let m = need_matrix(args, 0)?;
    let mut out = Matrix::full(m.rows(), m.cols(), 0.0);
    for idx in 0..m.numel() {
        out.set_linear(idx, f(m.get_linear(idx))?);
    }
    Ok(Value::Double(out))
}

// ---- wrapperهای نام‌دار برای رجیستری static (fn pointer ها const-safe اند) ----
#[allow(dead_code)] // P1 نسخه؛ نسخهٔ مختلط-آگاه در registry_p2
fn el_abs(a: &[Value], n: usize) -> Result<Value, McError> {
    let _ = n;
    elem_apply(a, bi_abs)
}
fn el_sqrt(a: &[Value], n: usize) -> Result<Value, McError> {
    let _ = n;
    elem_apply(a, bi_sqrt)
}
fn el_exp(a: &[Value], n: usize) -> Result<Value, McError> {
    let _ = n;
    elem_apply(a, bi_exp)
}
fn el_log(a: &[Value], n: usize) -> Result<Value, McError> {
    let _ = n;
    elem_apply(a, bi_log)
}
fn el_log2(a: &[Value], n: usize) -> Result<Value, McError> {
    let _ = n;
    elem_apply(a, bi_log2)
}
fn el_log10(a: &[Value], n: usize) -> Result<Value, McError> {
    let _ = n;
    elem_apply(a, bi_log10)
}
fn el_sin(a: &[Value], n: usize) -> Result<Value, McError> {
    let _ = n;
    elem_apply(a, bi_sin)
}
fn el_cos(a: &[Value], n: usize) -> Result<Value, McError> {
    let _ = n;
    elem_apply(a, bi_cos)
}
fn el_tan(a: &[Value], n: usize) -> Result<Value, McError> {
    let _ = n;
    elem_apply(a, bi_tan)
}
fn el_asin(a: &[Value], n: usize) -> Result<Value, McError> {
    let _ = n;
    elem_apply(a, bi_asin)
}
fn el_acos(a: &[Value], n: usize) -> Result<Value, McError> {
    let _ = n;
    elem_apply(a, bi_acos)
}
fn el_atan(a: &[Value], n: usize) -> Result<Value, McError> {
    let _ = n;
    elem_apply(a, bi_atan)
}
fn el_sinh(a: &[Value], n: usize) -> Result<Value, McError> {
    let _ = n;
    elem_apply(a, bi_sinh)
}
fn el_cosh(a: &[Value], n: usize) -> Result<Value, McError> {
    let _ = n;
    elem_apply(a, bi_cosh)
}
fn el_tanh(a: &[Value], n: usize) -> Result<Value, McError> {
    let _ = n;
    elem_apply(a, bi_tanh)
}
fn el_sign(a: &[Value], n: usize) -> Result<Value, McError> {
    let _ = n;
    elem_apply(a, bi_sign)
}
fn el_round(a: &[Value], n: usize) -> Result<Value, McError> {
    let _ = n;
    elem_apply(a, bi_round)
}
fn el_floor(a: &[Value], n: usize) -> Result<Value, McError> {
    let _ = n;
    elem_apply(a, bi_floor)
}
fn el_ceil(a: &[Value], n: usize) -> Result<Value, McError> {
    let _ = n;
    elem_apply(a, bi_ceil)
}
fn el_fix(a: &[Value], n: usize) -> Result<Value, McError> {
    let _ = n;
    elem_apply(a, bi_fix)
}
#[allow(dead_code)] // P1 نسخه؛ نسخهٔ مختلط-آگاه در registry_p2
fn el_angle(a: &[Value], n: usize) -> Result<Value, McError> {
    let _ = n;
    elem_apply(a, bi_angle)
}
#[allow(dead_code)] // P1 نسخه؛ نسخهٔ مختلط-آگاه در registry_p2
fn el_real(a: &[Value], n: usize) -> Result<Value, McError> {
    let _ = n;
    elem_apply(a, Ok)
} // روی حقیقی = همانی
#[allow(dead_code)] // P1 نسخه؛ نسخهٔ مختلط-آگاه در registry_p2
fn el_imag(a: &[Value], n: usize) -> Result<Value, McError> {
    let _ = n;
    elem_apply(a, |_| Ok(0.0))
}
#[allow(dead_code)] // P1 نسخه؛ نسخهٔ مختلط-آگاه در registry_p2
fn el_conj(a: &[Value], n: usize) -> Result<Value, McError> {
    let _ = n;
    elem_apply(a, Ok)
}
fn el_isnan(a: &[Value], n: usize) -> Result<Value, McError> {
    let _ = n;
    elem_apply(a, |x| Ok(x.is_nan() as i32 as f64))
}
fn el_isinf(a: &[Value], n: usize) -> Result<Value, McError> {
    let _ = n;
    elem_apply(a, |x| Ok(x.is_infinite() as i32 as f64))
}
fn el_isfinite(a: &[Value], n: usize) -> Result<Value, McError> {
    let _ = n;
    elem_apply(a, |x| Ok(x.is_finite() as i32 as f64))
}
#[allow(dead_code)] // P1 نسخه؛ نسخهٔ مختلط-آگاه در registry_p2
fn el_isreal(a: &[Value], _n: usize) -> Result<Value, McError> {
    let _ = a;
    Ok(scalar_val(1.0))
}

// ---------- خطاهای کمکی ----------

fn arity_err(name: &str, want: &str, got: usize) -> McError {
    McError::new(
        ids::TYPE_ERROR,
        0,
        0,
        format!("{name}: expected {want} argument(s), got {got}."),
        format!("{name}: {want} آرگومان انتظار می‌رود، ولی {got} دریافت شد."),
    )
}

fn type_err(name: &str) -> McError {
    McError::new(
        ids::UNSUPPORTED,
        0,
        0,
        format!("{name}: unsupported argument type in this milestone."),
        format!("{name}: نوع آرگومان در این مایل‌استون پشتیبانی نمی‌شود."),
    )
}

fn need_matrix(args: &[Value], i: usize) -> Result<Matrix, McError> {
    match args.get(i) {
        Some(Value::Double(m)) => Ok(m.clone()),
        Some(_) => Err(McError::new(
            ids::TYPE_ERROR,
            0,
            0,
            "Numeric matrix expected.",
            "ماتریس عددی انتظار می‌رود.",
        )),
        None => Err(McError::new(
            ids::TYPE_ERROR,
            0,
            0,
            "Missing argument.",
            "آرگومان جا افتاده است.",
        )),
    }
}

fn need_scalar(args: &[Value], i: usize) -> Result<f64, McError> {
    match args.get(i).and_then(Value::as_scalar) {
        Some(v) => Ok(v),
        None => Err(McError::new(
            ids::TYPE_ERROR,
            0,
            0,
            "Scalar argument expected.",
            "آرگومان اسکالر انتظار می‌رود.",
        )),
    }
}

fn need_int_scalar(args: &[Value], i: usize) -> Result<usize, McError> {
    let v = need_scalar(args, i)?;
    if v < 0.0 || v.fract() != 0.0 {
        return Err(McError::new(
            ids::TYPE_ERROR,
            0,
            0,
            "Size must be a nonnegative integer.",
            "ابعاد باید عدد صحیح نامنفی باشند.",
        ));
    }
    Ok(v as usize)
}

fn scalar_val(v: f64) -> Value {
    Value::Double(Matrix::scalar(v))
}

// ---------- سازه‌های ماتریسی ----------

fn bi_zeros(args: &[Value], _n: usize) -> Result<Value, McError> {
    let (r, c) = dim2(args, "zeros")?;
    // بودجهٔ حافظه (ROADMAP 2.5): ابعاد از ورودی کاربر — try_full اجباری.
    Ok(Value::Double(Matrix::try_full(r, c, 0.0)?))
}

fn bi_ones(args: &[Value], _n: usize) -> Result<Value, McError> {
    let (r, c) = dim2(args, "ones")?;
    Ok(Value::Double(Matrix::try_full(r, c, 1.0)?))
}

fn bi_eye(args: &[Value], _n: usize) -> Result<Value, McError> {
    let (r, c) = dim2(args, "eye")?;
    let mut m = Matrix::try_full(r, c, 0.0)?;
    for i in 0..r.min(c) {
        m.set(i, i, 1.0);
    }
    Ok(Value::Double(m))
}

/// یک یا دو بعد — `zeros(n)` مربع، `zeros(m,n)`.
fn dim2(args: &[Value], name: &str) -> Result<(usize, usize), McError> {
    match args.len() {
        1 => {
            let n = need_int_scalar(args, 0)?;
            Ok((n, n))
        }
        2 => {
            let r = need_int_scalar(args, 0)?;
            let c = need_int_scalar(args, 1)?;
            Ok((r, c))
        }
        _ => Err(arity_err(name, "1 or 2", args.len())),
    }
}

/// `meshgrid(x)` / `meshgrid(x, y)` — سمانتیک تک‌خروجی متلب: فقط شبکهٔ X
/// برمی‌گردد (تعداد سطر = numel(y)، ستون = numel(x))؛ خروجی دوم
/// ([X, Y] = ...) با مایل‌استون multi-assign می‌آید (همان پیام صادقانهٔ
/// qr/svd). تا آن زمان surf(Z) شبکه را خودش می‌سازد.
fn bi_meshgrid(args: &[Value], _n: usize) -> Result<Value, McError> {
    if args.is_empty() || args.len() > 2 {
        return Err(arity_err("meshgrid", "1 or 2", args.len()));
    }
    let xv = match &args[0] {
        Value::Double(m) if m.rows() == 1 || m.cols() == 1 => m.as_column_slice().to_vec(),
        _ => return Err(type_err("meshgrid")),
    };
    let yv = if args.len() == 2 {
        match &args[1] {
            Value::Double(m) if m.rows() == 1 || m.cols() == 1 => m.as_column_slice().to_vec(),
            _ => return Err(type_err("meshgrid")),
        }
    } else {
        xv.clone()
    };
    if xv.is_empty() || yv.is_empty() {
        return Err(type_err("meshgrid"));
    }
    let rows = yv.len();
    let cols = xv.len();
    let mut m = Matrix::try_full(rows, cols, 0.0)?;
    for (i, _) in yv.iter().enumerate() {
        for (j, &x) in xv.iter().enumerate() {
            m.set(i, j, x);
        }
    }
    Ok(Value::Double(m))
}

/// `peaks(n)` / `peaks` / `peaks(x, y)` — تابع نمونهٔ متلب (نمایشی surf).
/// پیش‌فرض n = 49 روی [-3, 3]؛ فرم مختصاتی برای x/y برداری.
fn bi_peaks(args: &[Value], _n: usize) -> Result<Value, McError> {
    let peaks_at = |x: f64, y: f64| -> f64 {
        3.0 * (1.0 - x).powi(2) * (-x * x - (y + 1.0).powi(2)).exp()
            - 10.0 * (x / 5.0 - x.powi(3) - y.powi(5)) * (-x * x - y * y).exp()
            + (1.0 / 3.0) * (-(x + 1.0).powi(2) - y * y).exp()
    };
    match args.len() {
        0 => {
            let n = 49;
            let (lo, hi) = (-3.0, 3.0);
            let step = (hi - lo) / (n as f64 - 1.0);
            let mut m = Matrix::try_full(n, n, 0.0)?;
            for i in 0..n {
                for j in 0..n {
                    let x = lo + j as f64 * step;
                    let y = lo + i as f64 * step;
                    m.set(i, j, peaks_at(x, y));
                }
            }
            Ok(Value::Double(m))
        }
        1 => {
            let n = need_int_scalar(args, 0)?;
            if !(2..=512).contains(&n) {
                return Err(McError::new(
                    ids::TYPE_ERROR,
                    0,
                    0,
                    "peaks: n must be between 2 and 512.".to_owned(),
                    "peaks: n باید بین ۲ و ۵۱۲ باشد.".to_owned(),
                ));
            }
            let (lo, hi) = (-3.0, 3.0);
            let step = (hi - lo) / (n as f64 - 1.0);
            let mut m = Matrix::try_full(n, n, 0.0)?;
            for i in 0..n {
                for j in 0..n {
                    let x = lo + j as f64 * step;
                    let y = lo + i as f64 * step;
                    m.set(i, j, peaks_at(x, y));
                }
            }
            Ok(Value::Double(m))
        }
        2 => {
            // peaks(x, y): x بردار ستون‌ها، y بردار سطرها → ماتریس numel(y)×numel(x)
            let xv = match &args[0] {
                Value::Double(m) if m.rows() == 1 || m.cols() == 1 => m.as_column_slice().to_vec(),
                _ => return Err(type_err("peaks")),
            };
            let yv = match &args[1] {
                Value::Double(m) if m.rows() == 1 || m.cols() == 1 => m.as_column_slice().to_vec(),
                _ => return Err(type_err("peaks")),
            };
            if xv.is_empty() || yv.is_empty() {
                return Err(type_err("peaks"));
            }
            let mut m = Matrix::try_full(yv.len(), xv.len(), 0.0)?;
            for (i, &y) in yv.iter().enumerate() {
                for (j, &x) in xv.iter().enumerate() {
                    m.set(i, j, peaks_at(x, y));
                }
            }
            Ok(Value::Double(m))
        }
        n => Err(arity_err("peaks", "0, 1 or 2", n)),
    }
}

fn bi_linspace(args: &[Value], _n: usize) -> Result<Value, McError> {
    if args.len() != 3 {
        return Err(arity_err("linspace", "3", args.len()));
    }
    let (a, b) = (need_scalar(args, 0)?, need_scalar(args, 1)?);
    let n = need_int_scalar(args, 2)?;
    if n == 0 {
        return Ok(Value::Double(Matrix::full(1, 0, 0.0)));
    }
    if n == 1 {
        return Ok(Value::Double(Matrix::full(1, 1, b)));
    }
    let mut m = Matrix::try_full(1, n, 0.0)?;
    for i in 0..n {
        // فرمول متلب: a + i*(b-a)/(n-1) با اصلاح پایانی (سند ۰۳ §7 روح).
        m.set(0, i, a + (i as f64) * (b - a) / ((n - 1) as f64));
    }
    m.set(0, n - 1, b);
    Ok(Value::Double(m))
}

fn bi_logspace(args: &[Value], _n: usize) -> Result<Value, McError> {
    if args.len() != 3 {
        return Err(arity_err("logspace", "3", args.len()));
    }
    let a = need_scalar(args, 0)?;
    let b = need_scalar(args, 1)?;
    let n = need_int_scalar(args, 2)?;
    let lin = bi_linspace(&[scalar_val(a), scalar_val(b), scalar_val(n as f64)], 1)?;
    if let Value::Double(m) = lin {
        let mut out = Matrix::full(m.rows(), m.cols(), 0.0);
        for idx in 0..m.numel() {
            out.set_linear(idx, 10f64.powf(m.get_linear(idx)));
        }
        Ok(Value::Double(out))
    } else {
        Err(type_err("logspace"))
    }
}

fn bi_diag(args: &[Value], _n: usize) -> Result<Value, McError> {
    if args.is_empty() || args.len() > 2 {
        return Err(arity_err("diag", "1 or 2", args.len()));
    }
    let k = if args.len() == 2 {
        need_scalar(args, 1)? as i64
    } else {
        0
    };
    match &args[0] {
        Value::Double(m) if m.rows() == 1 || m.cols() == 1 => {
            // بردار → ماتریس قطری
            let len = m.rows().max(m.cols());
            // بودجهٔ حافظه: k اسکالر کاربر است (k=1e18 مجاز نیست) — جمع/چک امن (ROADMAP 2.5).
            let n = len
                .checked_add(k.unsigned_abs() as usize)
                .ok_or_else(|| budget::out_of_memory_error(len, k.unsigned_abs() as usize))?;
            budget::check_elements(n, n)?;
            let mut out = Matrix::full(n, n, 0.0);
            for i in 0..len {
                let idx = (i as i64 + k) as usize;
                out.set(idx, idx, m.get_linear(i));
            }
            Ok(Value::Double(out))
        }
        Value::Double(m) => {
            // ماتریس → قطر k
            let (r, c) = (m.rows(), m.cols());
            let mut v = Vec::new();
            for i in 0..r.min(c) {
                let j = i as i64 + k;
                if j >= 0 && (j as usize) < c {
                    v.push(m.get(i, j as usize));
                }
            }
            let mut out = Matrix::full(v.len(), 1, 0.0);
            for (i, x) in v.into_iter().enumerate() {
                out.set(i, 0, x);
            }
            Ok(Value::Double(out))
        }
        _ => Err(type_err("diag")),
    }
}

fn bi_tril(args: &[Value], _n: usize) -> Result<Value, McError> {
    tri_extract(args, true)
}

fn bi_triu(args: &[Value], _n: usize) -> Result<Value, McError> {
    tri_extract(args, false)
}

fn tri_extract(args: &[Value], lower: bool) -> Result<Value, McError> {
    if args.is_empty() || args.len() > 2 {
        return Err(arity_err(
            if lower { "tril" } else { "triu" },
            "1 or 2",
            args.len(),
        ));
    }
    let m = need_matrix(args, 0)?;
    let k = if args.len() == 2 {
        need_scalar(args, 1)? as i64
    } else {
        0
    };
    let mut out = Matrix::full(m.rows(), m.cols(), 0.0);
    for r in 0..m.rows() {
        for c in 0..m.cols() {
            // قرارداد متلب: قطر k-اُم، k>0 بالای قطر اصلی. tril(A,k) = روی و
            // زیرِ قطر k-اُم: i−j ≥ −k؛ triu(A,k) = روی و بالای آن: i−j ≤ −k.
            // (فیکس رگرسیون ROADMAP 2.6 — قبلاً علامت k برعکس متلب بود)
            let d = (r as i64) - (c as i64);
            let keep = if lower { d >= -k } else { d <= -k };
            if keep {
                out.set(r, c, m.get(r, c));
            }
        }
    }
    Ok(Value::Double(out))
}

fn bi_size(args: &[Value], _n: usize) -> Result<Value, McError> {
    // ساختار ۱×۱ (ROADMAP 2.9) — size = [1 1]؛ ابعاد بالاتر 1
    if let Some(Value::Struct(_)) = args.first() {
        if args.len() == 2 {
            let dim = need_int_scalar(args, 1)?;
            return Ok(scalar_val(if dim == 0 { 2.0 } else { 1.0 }));
        }
        let mut out = Matrix::full(1, 2, 0.0);
        out.set(0, 0, 1.0);
        out.set(0, 1, 1.0);
        return Ok(Value::Double(out));
    }
    let m = need_matrix(args, 0)?;
    if args.len() == 2 {
        let dim = need_int_scalar(args, 1)?;
        return Ok(scalar_val(match dim {
            1 => m.rows() as f64,
            2 => m.cols() as f64,
            _ => 1.0, // ابعاد بالاتر از 2 در P1 یک‌بعدی‌اند
        }));
    }
    let mut out = Matrix::full(1, 2, 0.0);
    out.set(0, 0, m.rows() as f64);
    out.set(0, 1, m.cols() as f64);
    Ok(Value::Double(out))
}

fn bi_length(args: &[Value], _n: usize) -> Result<Value, McError> {
    match &args.first() {
        Some(Value::Double(m)) => Ok(scalar_val(m.rows().max(m.cols()) as f64)),
        Some(Value::Char(s)) => Ok(scalar_val(s.chars().count() as f64)), // یونیکد صحیح (سند ۰۳ §10)
        // ساختار ۱×۱ (ROADMAP 2.9)
        Some(Value::Struct(_)) => Ok(scalar_val(1.0)),
        _ => Err(type_err("length")),
    }
}

fn bi_numel(args: &[Value], _n: usize) -> Result<Value, McError> {
    match &args.first() {
        Some(Value::Double(m)) => Ok(scalar_val(m.numel() as f64)),
        Some(Value::Char(s)) => Ok(scalar_val(s.chars().count() as f64)),
        // ساختار ۱×۱ (ROADMAP 2.9)
        Some(Value::Struct(_)) => Ok(scalar_val(1.0)),
        _ => Err(type_err("numel")),
    }
}

fn bi_isempty(args: &[Value], _n: usize) -> Result<Value, McError> {
    // ساختار ۱×۱ هرگز خالی نیست (حتی بدون فیلد — اوراکل Octave 10.3)
    if matches!(args.first(), Some(Value::Struct(_))) {
        return Ok(scalar_val(0.0));
    }
    let m = need_matrix(args, 0)?;
    Ok(scalar_val(m.is_empty() as i32 as f64))
}

fn bi_isvector(args: &[Value], _n: usize) -> Result<Value, McError> {
    if matches!(args.first(), Some(Value::Struct(_))) {
        return Ok(scalar_val(1.0));
    }
    let m = need_matrix(args, 0)?;
    Ok(scalar_val((m.rows() == 1 || m.cols() == 1) as i32 as f64))
}

fn bi_isscalar(args: &[Value], _n: usize) -> Result<Value, McError> {
    if matches!(args.first(), Some(Value::Struct(_))) {
        return Ok(scalar_val(1.0));
    }
    let m = need_matrix(args, 0)?;
    Ok(scalar_val(m.is_scalar() as i32 as f64))
}

fn bi_repmat(args: &[Value], _n: usize) -> Result<Value, McError> {
    if args.len() != 3 {
        return Err(arity_err("repmat", "3", args.len()));
    }
    let m = need_matrix(args, 0)?;
    let r = need_int_scalar(args, 1)?;
    let c = need_int_scalar(args, 2)?;
    // بودجهٔ حافظه: ضرب ابعاد کاربر می‌تواند سرریز کند — ضرب امن (ROADMAP 2.5).
    let or = m
        .rows()
        .checked_mul(r)
        .ok_or_else(|| budget::out_of_memory_error(m.rows().saturating_mul(r), c))?;
    let oc = m
        .cols()
        .checked_mul(c)
        .ok_or_else(|| budget::out_of_memory_error(or, m.cols().saturating_mul(c)))?;
    budget::check_elements(or, oc)?;
    let mut out = Matrix::full(or, oc, 0.0);
    for br in 0..r {
        for bc in 0..c {
            for sr in 0..m.rows() {
                for sc in 0..m.cols() {
                    out.set(br * m.rows() + sr, bc * m.cols() + sc, m.get(sr, sc));
                }
            }
        }
    }
    Ok(Value::Double(out))
}

fn bi_reshape(args: &[Value], _n: usize) -> Result<Value, McError> {
    if args.len() != 3 {
        return Err(arity_err("reshape", "3", args.len()));
    }
    let m = need_matrix(args, 0)?;
    let r = need_int_scalar(args, 1)?;
    let c = need_int_scalar(args, 2)?;
    if r * c != m.numel() {
        return Err(McError::new(
            ids::SIZE_MISMATCH,
            0,
            0,
            format!(
                "reshape: cannot reshape {old} elements into {r}x{c}.",
                old = m.numel()
            ),
            format!(
                "reshape: نمی‌توان {old} عنصر را به {r}×{c} بازچینی کرد.",
                old = m.numel()
            ),
        ));
    }
    // column-major: ترتیب خطی حفظ می‌شود (سند ۰۴ §2)
    let data: Vec<f64> = (0..m.numel()).map(|i| m.get_linear(i)).collect();
    Ok(Value::Double(Matrix::from_columns(r, c, data)))
}

// ---------- جبر خطی (mc-num — سند ۰۴ §3) ----------

fn bi_inv(args: &[Value], _n: usize) -> Result<Value, McError> {
    Ok(Value::Double(num::inv(&need_matrix(args, 0)?)?))
}

fn bi_det(args: &[Value], _n: usize) -> Result<Value, McError> {
    Ok(scalar_val(num::det(&need_matrix(args, 0)?)?))
}

fn bi_rank(args: &[Value], _n: usize) -> Result<Value, McError> {
    let m = need_matrix(args, 0)?;
    Ok(scalar_val(num::rank(&m, None) as f64))
}

fn bi_trace(args: &[Value], _n: usize) -> Result<Value, McError> {
    let m = need_matrix(args, 0)?;
    if m.rows() != m.cols() {
        return Err(McError::new(
            ids::SIZE_MISMATCH,
            0,
            0,
            "trace: matrix must be square.",
            "trace: ماتریس باید مربع باشد.",
        ));
    }
    Ok(scalar_val((0..m.rows()).map(|i| m.get(i, i)).sum()))
}

fn bi_rref(args: &[Value], _n: usize) -> Result<Value, McError> {
    let (r, _) = num::rref(&need_matrix(args, 0)?);
    Ok(Value::Double(r))
}

fn bi_norm(args: &[Value], _n: usize) -> Result<Value, McError> {
    let m = need_matrix(args, 0)?;
    let p = match args.get(1) {
        Some(Value::Char(s)) => Some(s.as_str()),
        Some(Value::Double(d)) if d.is_scalar() => {
            // norm(v, p) با p عددی — متلب هم عدد می‌پذیرد (1/2/Inf) هم رشته 'inf'/'fro'
            match d.get(0, 0) {
                2.0 => None,
                1.0 => Some("1"),
                x if x.is_infinite() && x > 0.0 => Some("inf"),
                _ => {
                    return Err(McError::new(
                        ids::UNSUPPORTED,
                        0,
                        0,
                        "norm: only p = 1, 2, inf, fro in this milestone.",
                        "norm: در این مایل‌استون فقط p = 1، 2، inf، fro.",
                    ))
                }
            }
        }
        None => None,
        _ => return Err(type_err("norm")),
    };
    Ok(scalar_val(num::norm(&m, p)?))
}

fn bi_cond(args: &[Value], _n: usize) -> Result<Value, McError> {
    Ok(scalar_val(num::cond(&need_matrix(args, 0)?)?))
}

fn bi_chol(args: &[Value], _n: usize) -> Result<Value, McError> {
    Ok(Value::Double(num::chol(&need_matrix(args, 0)?)?))
}

fn bi_qr(args: &[Value], nout: usize) -> Result<Value, McError> {
    let m = need_matrix(args, 0)?;
    if nout > 1 {
        // [Q,R] = qr — چندخروجی در registry_p2 (ROADMAP 2.6) پیاده شد.
        return multi_out_soon("qr");
    }
    // متلب qr(A) تک‌خروجی: فقط R (مثلثی بالایی) — COMPATIBILITY §۳.
    let (_, r) = num::qr(&m)?;
    Ok(Value::Double(r))
}

fn bi_svd(args: &[Value], nout: usize) -> Result<Value, McError> {
    let m = need_matrix(args, 0)?;
    if nout > 1 {
        // [U,S,V] = svd — چندخروجی در registry_p2 (ROADMAP 2.6) پیاده شد.
        return multi_out_soon("svd");
    }
    let (_, s, _) = num::svd(&m)?;
    // متلب svd(A) تک‌خروجی: بردار ستونی σها به‌ترتیب نزولی (نه ماتریس قطری) —
    // همین رفتار. ماتریس S فقط در شکل سه‌خروجی [U,S,V] می‌آید.
    let mut out = Matrix::full(s.len(), 1, 0.0);
    for (i, v) in s.iter().enumerate() {
        out.set(i, 0, *v);
    }
    Ok(Value::Double(out))
}

fn bi_eig(args: &[Value], nout: usize) -> Result<Value, McError> {
    let m = need_matrix(args, 0)?;
    if nout > 1 {
        return multi_out_soon("eig");
    }
    let (vals, _) = if mc_num::is_exactly_symmetric(&m) {
        num::eig_sym(&m)?
    } else {
        // عمومی: مقادیر مختلط با P2 — پیام صادقانه (سند ۰۴ §3.1 مسیر دوگانه)
        return Err(McError::new(
            ids::UNSUPPORTED,
            0,
            0,
            "eig of non-symmetric matrices returns complex values — arrives with P2 complex support. Try eig on a symmetric matrix.",
            "eig ماتریس نامتقارن مقدار مختلط می‌دهد — با پشتیبانی مختلط P2 می‌آید. روی ماتریس متقارن امتحان کنید.",
        ));
    };
    let mut out = Matrix::full(vals.len(), 1, 0.0);
    for (i, v) in vals.iter().enumerate() {
        out.set(i, 0, *v);
    }
    Ok(Value::Double(out))
}

fn multi_out_soon(name: &str) -> Result<Value, McError> {
    Err(McError::new(
        ids::UNSUPPORTED,
        0,
        0,
        format!("{name} with multiple outputs ([a,b]=f()) arrives with the next P1 milestone."),
        format!("{name} چندخروجی ([a,b]=f()) در مایل‌استون بعدی P1 می‌آید."),
    ))
}

fn bi_dot(args: &[Value], _n: usize) -> Result<Value, McError> {
    let a = need_matrix(args, 0)?;
    let b = need_matrix(args, 1)?;
    if a.numel() != b.numel() {
        return Err(McError::new(
            ids::SIZE_MISMATCH,
            0,
            0,
            "dot: vectors must have the same length.",
            "dot: طول بردارها باید برابر باشد.",
        ));
    }
    Ok(scalar_val(
        (0..a.numel())
            .map(|i| a.get_linear(i) * b.get_linear(i))
            .sum(),
    ))
}

fn bi_cross(args: &[Value], _n: usize) -> Result<Value, McError> {
    let a = need_matrix(args, 0)?;
    let b = need_matrix(args, 1)?;
    if a.numel() != 3 || b.numel() != 3 {
        return Err(McError::new(
            ids::SIZE_MISMATCH,
            0,
            0,
            "cross: both inputs must be 3-element vectors.",
            "cross: هر دو ورودی باید بردار ۳-عنصری باشند.",
        ));
    }
    let g = |m: &Matrix, i: usize| m.get_linear(i);
    let mut out = Matrix::full(3, 1, 0.0);
    out.set(0, 0, g(&a, 1) * g(&b, 2) - g(&a, 2) * g(&b, 1));
    out.set(1, 0, g(&a, 2) * g(&b, 0) - g(&a, 0) * g(&b, 2));
    out.set(2, 0, g(&a, 0) * g(&b, 1) - g(&a, 1) * g(&b, 0));
    Ok(Value::Double(out))
}

fn bi_kron(args: &[Value], _n: usize) -> Result<Value, McError> {
    let a = need_matrix(args, 0)?;
    let b = need_matrix(args, 1)?;
    let (ar, ac, br, bc) = (a.rows(), a.cols(), b.rows(), b.cols());
    // بودجهٔ حافظه: kron(a,b) از اندازهٔ دو ماتریس موجود فراتر می‌رود — چک امن (ROADMAP 2.5).
    let or = ar
        .checked_mul(br)
        .ok_or_else(|| budget::out_of_memory_error(ar, br))?;
    let oc = ac
        .checked_mul(bc)
        .ok_or_else(|| budget::out_of_memory_error(ac, bc))?;
    budget::check_elements(or, oc)?;
    let mut out = Matrix::full(or, oc, 0.0);
    for i in 0..ar {
        for j in 0..ac {
            let v = a.get(i, j);
            for bi in 0..br {
                for bj in 0..bc {
                    out.set(i * br + bi, j * bc + bj, v * b.get(bi, bj));
                }
            }
        }
    }
    Ok(Value::Double(out))
}

// ---------- عنصری دوآرگومانی ----------

fn bi_atan2(args: &[Value], _n: usize) -> Result<Value, McError> {
    let a = need_matrix(args, 0)?;
    let b = need_matrix(args, 1)?;
    broadcast2(&a, &b, "atan2", |y, x| y.atan2(x)).map(Value::Double)
}

fn bi_mod(args: &[Value], _n: usize) -> Result<Value, McError> {
    let a = need_matrix(args, 0)?;
    let b = need_matrix(args, 1)?;
    broadcast2(&a, &b, "mod", |x, y| {
        if y == 0.0 {
            x
        } else {
            x - y * (x / y).floor()
        }
    })
    .map(Value::Double)
}

fn bi_rem(args: &[Value], _n: usize) -> Result<Value, McError> {
    let a = need_matrix(args, 0)?;
    let b = need_matrix(args, 1)?;
    broadcast2(&a, &b, "rem", |x, y| {
        if y == 0.0 {
            x
        } else {
            x - y * (x / y).trunc()
        }
    })
    .map(Value::Double)
}

/// بروزکست متلب (سند ۰۲ §5): اسکالر با هر شکلی، هم‌شکل‌ها، وگرنه خطا.
pub fn broadcast2(
    a: &Matrix,
    b: &Matrix,
    name: &str,
    f: impl Fn(f64, f64) -> f64,
) -> Result<Matrix, McError> {
    if a.is_scalar() {
        let x = a.get(0, 0);
        let mut out = Matrix::full(b.rows(), b.cols(), 0.0);
        for idx in 0..b.numel() {
            out.set_linear(idx, f(x, b.get_linear(idx)));
        }
        return Ok(out);
    }
    if b.is_scalar() {
        let y = b.get(0, 0);
        let mut out = Matrix::full(a.rows(), a.cols(), 0.0);
        for idx in 0..a.numel() {
            out.set_linear(idx, f(a.get_linear(idx), y));
        }
        return Ok(out);
    }
    if (a.rows(), a.cols()) == (b.rows(), b.cols()) {
        let mut out = Matrix::full(a.rows(), a.cols(), 0.0);
        for idx in 0..a.numel() {
            out.set_linear(idx, f(a.get_linear(idx), b.get_linear(idx)));
        }
        return Ok(out);
    }
    Err(McError::new(
        ids::SIZE_MISMATCH,
        0,
        0,
        format!(
            "{name}: sizes must agree or one must be scalar, got {ar}x{ac} and {br}x{bc}.",
            ar = a.rows(),
            ac = a.cols(),
            br = b.rows(),
            bc = b.cols()
        ),
        format!(
            "{name}: ابعاد باید هم‌خوان باشند یا یکی اسکالر؛ دریافت شد {ar}×{ac} و {br}×{bc}.",
            ar = a.rows(),
            ac = a.cols(),
            br = b.rows(),
            bc = b.cols()
        ),
    ))
}

// ---------- کاهشی ----------

/// کاهش در امتداد ستون‌ها — سمانتیک متلب: بردار → اسکالر، ماتریس → سطر.
/// کاهش در راستای dim (سمانتیک متلب): dim 0=پیش‌فرض (اولین بعدِ غیرتک‌عضوی)،
/// 1=در راستای ستون‌ها (خروجی 1×n)، 2=در راستای ردیف‌ها (خروجی m×1).
/// شکل خروجی = ورودی با بعدِ کاهش‌یافته→۱؛ بردارِ هم‌راستا = اسکالر و
/// بردارِ عمود = خودش (جمعِ تک‌عضو) — هر دو خودکار از فرمول شکل درمی‌آید.
fn reduce_dim(m: &Matrix, init: f64, f: impl Fn(f64, f64) -> f64, dim: u8) -> Value {
    let along_cols = match dim {
        0 => m.rows() > 1,
        1 => true,
        _ => false,
    };
    if along_cols {
        let mut out = Matrix::full(1, m.cols(), init);
        for c in 0..m.cols() {
            let mut acc = init;
            for r in 0..m.rows() {
                acc = f(acc, m.get(r, c));
            }
            out.set(0, c, acc);
        }
        Value::Double(out)
    } else {
        let mut out = Matrix::full(m.rows(), 1, init);
        for r in 0..m.rows() {
            let mut acc = init;
            for c in 0..m.cols() {
                acc = f(acc, m.get(r, c));
            }
            out.set(r, 0, acc);
        }
        Value::Double(out)
    }
}

/// خواندن آرگومان dim اختیاری (sum/mean/...): 0=پیش‌فرض، 1، 2.
fn parse_dim_arg(args: &[Value], fname: &str) -> Result<u8, McError> {
    match args.get(1) {
        None => Ok(0),
        Some(Value::Double(d)) if d.is_scalar() => {
            let k = d.get(0, 0);
            if k.fract() != 0.0 || !(1.0..=2.0).contains(&k) {
                return Err(McError::new(
                    ids::UNSUPPORTED,
                    0,
                    0,
                    format!("{fname}: only dim 1/2 in this milestone."),
                    format!("{fname}: در این مایل‌استون فقط dim 1 یا 2."),
                ));
            }
            Ok(k as u8)
        }
        _ => Err(type_err(fname)),
    }
}

fn bi_sum(args: &[Value], _n: usize) -> Result<Value, McError> {
    let m = need_matrix(args, 0)?;
    let dim = parse_dim_arg(args, "sum")?;
    Ok(reduce_dim(&m, 0.0, |a, b| a + b, dim))
}

fn bi_prod(args: &[Value], _n: usize) -> Result<Value, McError> {
    let m = need_matrix(args, 0)?;
    let dim = parse_dim_arg(args, "prod")?;
    Ok(reduce_dim(&m, 1.0, |a, b| a * b, dim))
}

fn bi_max(args: &[Value], _n: usize) -> Result<Value, McError> {
    let m = need_matrix(args, 0)?;
    let dim = parse_dim_arg(args, "max")?;
    // NaN نادیده (رفتار متلب — سند ۰۳ §11 بند 5)
    Ok(reduce_dim(
        &m,
        f64::NEG_INFINITY,
        |a, b| {
            if b.is_nan() {
                a
            } else if a.is_nan() {
                b
            } else {
                a.max(b)
            }
        },
        dim,
    ))
}

fn bi_min(args: &[Value], _n: usize) -> Result<Value, McError> {
    let m = need_matrix(args, 0)?;
    let dim = parse_dim_arg(args, "min")?;
    Ok(reduce_dim(
        &m,
        f64::INFINITY,
        |a, b| {
            if b.is_nan() {
                a
            } else if a.is_nan() {
                b
            } else {
                a.min(b)
            }
        },
        dim,
    ))
}

fn bi_mean(args: &[Value], _n: usize) -> Result<Value, McError> {
    let m = need_matrix(args, 0)?;
    let dim = parse_dim_arg(args, "mean")?;
    let along_cols = match dim {
        0 => m.rows() > 1,
        1 => true,
        _ => false,
    };
    let n = if along_cols {
        m.rows() as f64
    } else {
        m.cols() as f64
    };
    match reduce_dim(&m, 0.0, |a, b| a + b, dim) {
        Value::Double(s) => {
            let mut out = s;
            for idx in 0..out.numel() {
                out.set_linear(idx, out.get_linear(idx) / n);
            }
            Ok(Value::Double(out))
        }
        _ => Err(type_err("mean")),
    }
}

fn cum_scan(m: &Matrix, prod: bool) -> Value {
    let mut out = Matrix::full(m.rows(), m.cols(), 0.0);
    // بردار سطری: پیمایش در راستای ردیف (اولین بعدِ غیرتک‌عضوی متلب)
    if m.rows() == 1 {
        let mut acc = if prod { 1.0 } else { 0.0 };
        for c in 0..m.cols() {
            acc = if prod {
                acc * m.get(0, c)
            } else {
                acc + m.get(0, c)
            };
            out.set(0, c, acc);
        }
        return Value::Double(out);
    }
    for c in 0..m.cols() {
        let mut acc = if prod { 1.0 } else { 0.0 };
        for r in 0..m.rows() {
            acc = if prod {
                acc * m.get(r, c)
            } else {
                acc + m.get(r, c)
            };
            out.set(r, c, acc);
        }
    }
    Value::Double(out)
}

fn bi_cumsum(args: &[Value], _n: usize) -> Result<Value, McError> {
    Ok(cum_scan(&need_matrix(args, 0)?, false))
}

fn bi_cumprod(args: &[Value], _n: usize) -> Result<Value, McError> {
    Ok(cum_scan(&need_matrix(args, 0)?, true))
}

fn bi_diff(args: &[Value], _n: usize) -> Result<Value, McError> {
    // CAS-1 (گام 4.2): آرگومان اول سمبولیک → مشتق نمادین (پاریت متلب:
    // diff عددی/نمادین زیر یک نام، دیسپچ با نوع آرگومان اول)
    if let Some(Value::Sym(_)) = args.first() {
        return crate::sym_ops::diff_sym(args, 0, 0);
    }
    // چهرهٔ عددی — پاریت متلب: diff(X)، diff(X,k)، diff(X,k,dim).
    // درس این گام: آرگومان k قبلاً بی‌صدا نادیده گرفته می‌شد (جواب غلطِ
    // مطمئن‌نما) — حالا مرتبهٔ تکرار و dim صریح صادقانه اعمال می‌شوند.
    let (order, dim) = match args.len() {
        1 => (1u64, None),
        2 => (diff_order_arg(args, 1)?, None),
        3 => (diff_order_arg(args, 1)?, Some(diff_dim_arg(args, 2)?)),
        n => return Err(arity_err("diff", "1..3", n)),
    };
    let mut m = need_matrix(args, 0)?;
    for _ in 0..order {
        m = diff_once(&m, dim);
    }
    Ok(Value::Double(m))
}

/// آرگومان مرتبهٔ عددی — صحیح نامنفی.
fn diff_order_arg(args: &[Value], i: usize) -> Result<u64, McError> {
    match args.get(i) {
        Some(Value::Double(m))
            if m.is_scalar() && m.get(0, 0) >= 0.0 && m.get(0, 0).fract() == 0.0 =>
        {
            Ok(m.get(0, 0) as u64)
        }
        _ => Err(McError::new(
            ids::TYPE_ERROR,
            0,
            0,
            "diff: the order must be a non-negative integer.",
            "diff: مرتبه باید عدد صحیح نامنفی باشد.",
        )),
    }
}

/// آرگومان بُعد صریح — ۱ یا ۲.
fn diff_dim_arg(args: &[Value], i: usize) -> Result<usize, McError> {
    match args.get(i) {
        Some(Value::Double(m)) if m.is_scalar() && (m.get(0, 0) == 1.0 || m.get(0, 0) == 2.0) => {
            Ok(m.get(0, 0) as usize)
        }
        _ => Err(McError::new(
            ids::TYPE_ERROR,
            0,
            0,
            "diff: dim must be 1 or 2.",
            "diff: dim باید ۱ یا ۲ باشد.",
        )),
    }
}

/// یک گام تفاضل — پیش‌فرض متلب: اولین بُعد غیرتک‌عضوی؛ dim صریح معتبر است.
fn diff_once(m: &Matrix, dim: Option<usize>) -> Matrix {
    let along_cols = match dim {
        Some(d) => d == 2,
        None => m.rows() == 1, // بردار سطری → ستونی؛ وگرنه سطری (مثل متلب)
    };
    if along_cols {
        let n = m.cols();
        if n < 2 {
            return Matrix::full(m.rows(), 0, 0.0);
        }
        let mut out = Matrix::full(m.rows(), n - 1, 0.0);
        for c in 0..n - 1 {
            for r in 0..m.rows() {
                out.set(r, c, m.get(r, c + 1) - m.get(r, c));
            }
        }
        return out;
    }
    if m.rows() < 2 {
        return Matrix::full(0, m.cols(), 0.0);
    }
    let mut out = Matrix::full(m.rows() - 1, m.cols(), 0.0);
    for c in 0..m.cols() {
        for r in 0..m.rows() - 1 {
            out.set(r, c, m.get(r + 1, c) - m.get(r, c));
        }
    }
    out
}

fn bi_all(args: &[Value], _n: usize) -> Result<Value, McError> {
    let m = need_matrix(args, 0)?;
    let dim = parse_dim_arg(args, "all")?;
    Ok(reduce_dim(
        &m,
        1.0,
        |a, b| {
            if b != 0.0 && !b.is_nan() {
                a.min(b)
            } else {
                0.0
            }
        },
        dim,
    ))
}

fn bi_any(args: &[Value], _n: usize) -> Result<Value, McError> {
    let m = need_matrix(args, 0)?;
    let dim = parse_dim_arg(args, "any")?;
    Ok(reduce_dim(
        &m,
        0.0,
        |a, b| {
            if b != 0.0 && !b.is_nan() {
                1.0
            } else {
                a
            }
        },
        dim,
    ))
}

fn bi_find(args: &[Value], _n: usize) -> Result<Value, McError> {
    let m = need_matrix(args, 0)?;
    // ترتیب column-major — سمانتیک متلب (سند ۰۴ §2)
    let hits: Vec<f64> = (0..m.numel())
        .filter(|&i| m.get_linear(i) != 0.0 && !m.get_linear(i).is_nan())
        .map(|i| (i + 1) as f64)
        .collect();
    let n = hits.len();
    let mut out = Matrix::full(n, 1, 0.0);
    for (i, v) in hits.into_iter().enumerate() {
        out.set(i, 0, v);
    }
    Ok(Value::Double(out))
}

fn bi_sort(args: &[Value], _n: usize) -> Result<Value, McError> {
    let m = need_matrix(args, 0)?;
    // پرچم جهت — متلب: 'ascend' پیش‌فرض، 'descend' نزولی؛ هر پرچم دیگر = خطای صادقانه
    // (پاسخ اشتباه ساکت ممنوع — سند ۰۷ Global DoD ۵)
    let descending = match args.len() {
        1 => false,
        2 => {
            let flag = arg_char(&args[1], "sort")?;
            match flag.as_str() {
                "ascend" => false,
                "descend" => true,
                other => {
                    return Err(McError::new(
                        ids::UNSUPPORTED,
                        0,
                        0,
                        format!("sort: unsupported flag '{other}' (only 'ascend'/'descend')."),
                        format!("sort: پرچم «{other}» پشتیبانی نمی‌شود (فقط 'ascend'/'descend')."),
                    ))
                }
            }
        }
        n => return Err(arity_err("sort", "1..2", n)),
    };
    // قانون «اولین بعدِ غیرتک‌عضوی» متلب: ماتریس چندهمه‌ردیف → هر ستون (dim 1)؛
    // بردار سطری/اسکالر → همان ردیف (dim 2). NaN در پایان (سند ۰۳ §11 بند 5) —
    // در حالت نزولی هم NaN آخر می‌ماند (رفتار متلب).
    let along_rows = m.rows() <= 1; // dim 2 برای سطری/اسکالر
    let mut out = Matrix::full(m.rows(), m.cols(), 0.0);
    if along_rows {
        let mut row: Vec<f64> = (0..m.cols()).map(|c| m.get(0, c)).collect();
        sort_with_nan(&mut row, descending);
        for (c, val) in row.iter().enumerate() {
            out.set(0, c, *val);
        }
    } else {
        for c in 0..m.cols() {
            let mut col: Vec<f64> = (0..m.rows()).map(|r| m.get(r, c)).collect();
            sort_with_nan(&mut col, descending);
            for (r, val) in col.iter().enumerate() {
                out.set(r, c, *val);
            }
        }
    }
    Ok(Value::Double(out))
}

/// مرتب‌سازی با قرارداد NaN-در-پایان متلب (هر دو جهت).
fn sort_with_nan(v: &mut [f64], descending: bool) {
    if descending {
        v.sort_by(|a, b| match (a.is_nan(), b.is_nan()) {
            (true, true) => std::cmp::Ordering::Equal,
            (true, false) => std::cmp::Ordering::Greater,
            (false, true) => std::cmp::Ordering::Less,
            _ => b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal),
        });
    } else {
        v.sort_by(|a, b| match (a.is_nan(), b.is_nan()) {
            (true, true) => std::cmp::Ordering::Equal,
            (true, false) => std::cmp::Ordering::Greater,
            (false, true) => std::cmp::Ordering::Less,
            _ => a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal),
        });
    }
}

/// استخراج رشته کاراکتری از آرگومان ('descend' یا "descend") — برای پرچم‌ها.
fn arg_char(v: &Value, fname: &str) -> Result<String, McError> {
    match v {
        Value::Char(s) => Ok(s.clone()),
        _ => Err(McError::new(
            ids::TYPE_ERROR,
            0,
            0,
            format!("{fname}: flag must be a char row vector like 'descend'."),
            format!("{fname}: پرچم باید رشته کاراکتری مثل 'descend' باشد."),
        )),
    }
}

// ---------- نمایش/متنی و ثابت‌ها (ROADMAP 2.4 — آیتم ۸) ----------

/// کلاس مقدار به سبک متلب — مختلط هم «double» است (رفتار متلب).
fn bi_class(args: &[Value], _n: usize) -> Result<Value, McError> {
    if args.len() != 1 {
        return Err(arity_err("class", "1", args.len()));
    }
    let name = match &args[0] {
        Value::Double(_) | Value::Complex(_) => "double",
        Value::Char(_) => "char",
        Value::Anon(_) => "function_handle",
        Value::Struct(_) => "struct",
        Value::Sym(_) => "sym",
        Value::List(_) => "cell",
    };
    Ok(Value::Char(name.to_owned()))
}

/// `char(x)` — عددی → رشتهٔ کاراکتری (ROADMAP 2.8 — بند ۶/۱۲). قرارداد گرد
/// کردن به سمت صفر از سند رسمی متلب؛ دامنهٔ codepoint کامل یونیکد بدون
/// surrogate — بیرون از دامنه خطای صادقانه (متلب به 0/65535 اشباع می‌کند —
/// انحراف مستند). فقط ورودی تک‌ردیفه؛ ستونی/دوبعدی = char دوبعدی → E4.
fn bi_char(args: &[Value], _n: usize) -> Result<Value, McError> {
    if args.len() != 1 {
        return Err(arity_err("char", "1", args.len()));
    }
    if let Value::Char(s) = &args[0] {
        return Ok(Value::Char(s.clone()));
    }
    if matches!(&args[0], Value::Complex(z) if !z.is_all_real()) {
        return Err(McError::new(
            ids::TYPE_ERROR,
            0,
            0,
            "char: complex values have no character codes.",
            "char: مقادیر مختلط کد کاراکتری ندارند.",
        ));
    }
    if matches!(&args[0], Value::Anon(_)) {
        return Err(McError::new(
            ids::TYPE_ERROR,
            0,
            0,
            "char: function handles have no character form.",
            "char: function_handle شکل کاراکتری ندارد.",
        ));
    }
    let m = match &args[0] {
        Value::Double(m) => m.clone(),
        Value::Complex(z) => z.real_part(),
        // نوع‌های دیگر بالاتر هندل شدند
        _ => {
            return Err(McError::new(
                ids::UNSUPPORTED,
                0,
                0,
                "char: unsupported input type.",
                "char: نوع ورودی پشتیبانی نمی‌شود.",
            ))
        }
    };
    if m.numel() == 0 {
        return Ok(Value::Char(String::new()));
    }
    if m.rows() != 1 {
        return Err(McError::new(
            ids::UNSUPPORTED,
            0,
            0,
            "char: column/2-D inputs produce 2-D char arrays; expansion E4.",
            "char: ورودی ستونی/دوبعدی آرایهٔ char دوبعدی می‌سازد؛ انبساط E4 است.",
        ));
    }
    let mut out = String::with_capacity(m.cols());
    for i in 0..m.cols() {
        out.push(Value::char_from_code(m.get(0, i), 0, 0)?);
    }
    Ok(Value::Char(out))
}

/// `double(x)` — char → ردیف کدها؛ double/مختلط (کلاس double) → خودش
/// (ROADMAP 2.8 — «interplay» char/double).
/// sym(x) — پل عدد→سمبولیک (گام 4.1): دودویی مرتب → گویای دقیق، وگرنه
/// Real تقریبی صادقانه (sym(0.1) هرگز 1/10 جا نمی‌زند). sym('x') هم نماد
/// می‌سازد (مکمل syms برای نام‌های دینامیک).
fn bi_sym(args: &[Value], _n: usize) -> Result<Value, McError> {
    if args.len() != 1 {
        return Err(arity_err("sym", "1", args.len()));
    }
    match &args[0] {
        Value::Char(name) => {
            if name.is_empty() || !name.chars().all(|c| c.is_alphanumeric() || c == '_') {
                return Err(McError::new(
                    mc_core::error::ids::TYPE_ERROR,
                    0,
                    0,
                    "sym: the name must be a valid identifier.",
                    "sym: نام باید شناسهٔ معتبر باشد.",
                ));
            }
            Ok(Value::Sym(std::sync::Arc::new(mc_cas::Sym::symbol(name))))
        }
        Value::Sym(_) => Ok(args[0].clone()),
        Value::Double(m) if m.is_scalar() => Ok(Value::Sym(std::sync::Arc::new(
            mc_cas::Sym::from_f64(m.get(0, 0)),
        ))),
        Value::Double(m) => Err(McError::new(
            mc_core::error::ids::UNSUPPORTED,
            0,
            0,
            format!(
                "sym: only scalars in CAS-0 (got {}x{}).",
                m.rows(),
                m.cols()
            ),
            format!(
                "sym: در CAS-0 فقط اسکالر (دریافت شد {}×{}).",
                m.rows(),
                m.cols()
            ),
        )),
        Value::Complex(_) => Err(McError::new(
            mc_core::error::ids::UNSUPPORTED,
            0,
            0,
            "sym: complex symbolic values are not supported yet — a later CAS phase.",
            "sym: مقادیر سمبولیک مختلط هنوز پشتیبانی نمی‌شود — فاز بعدی CAS.",
        )),
        _ => Err(McError::new(
            mc_core::error::ids::TYPE_ERROR,
            0,
            0,
            "sym: expected a number, a symbol name, or a symbolic value.",
            "sym: عدد، نام نماد یا مقدار سمبولیک لازم است.",
        )),
    }
}

/// subs(expr, old, new) — جای‌گذاری نماد (گام 4.1): subs(y, x, 2.5)؛
/// بازساخت از سازنده‌های کانونی ⇒ ساده‌سازی سطح-۰ دوباره اعمال می‌شود.
fn bi_subs(args: &[Value], _n: usize) -> Result<Value, McError> {
    if args.len() != 3 {
        return Err(arity_err("subs", "3", args.len()));
    }
    crate::sym_ops::subs_core(&args[0], &args[1], &args[2], 0, 0)
}

/// `simplify(S)` (گام 4.2/CAS-1) — فرم نرمال گویا + قواعد کل-دامنه.
fn bi_simplify(args: &[Value], _n: usize) -> Result<Value, McError> {
    if args.len() != 1 {
        return Err(arity_err("simplify", "1", args.len()));
    }
    crate::sym_ops::simplify_core(&args[0], 0, 0)
}

/// `expand(S)` — توزیع کامل بدون حذف gcd (پاریت متلب).
fn bi_expand(args: &[Value], _n: usize) -> Result<Value, McError> {
    if args.len() != 1 {
        return Err(arity_err("expand", "1", args.len()));
    }
    crate::sym_ops::expand_core(&args[0], 0, 0)
}

/// `collect(S)` / `collect(S, x)` — گروه‌بندی بر حسب توان متغیر.
fn bi_collect(args: &[Value], _n: usize) -> Result<Value, McError> {
    match args.len() {
        1 => crate::sym_ops::collect_core(&args[0], None, 0, 0),
        2 => crate::sym_ops::collect_core(&args[0], Some(&args[1]), 0, 0),
        n => Err(arity_err("collect", "1..2", n)),
    }
}

/// `latex(S)` — رشتهٔ TeX خام (سند ۰۵ §۱۰).
fn bi_latex(args: &[Value], _n: usize) -> Result<Value, McError> {
    if args.len() != 1 {
        return Err(arity_err("latex", "1", args.len()));
    }
    crate::sym_ops::latex_core(&args[0], 0, 0)
}

/// `int(S[, x][, x, a, b])` (گام 4.3/CAS-2) — فقط نمادین؛ عددیش integral است.
fn bi_int(args: &[Value], _n: usize) -> Result<Value, McError> {
    crate::sym_ops::int_sym(args, 0, 0)
}

/// `solve(S[, var])` / `solve([S…], [var…])` (گام 4.3/CAS-2) — فقط
/// نمادین؛ خروجی همیشه لیست (سند ۰۵ §۸).
fn bi_solve(args: &[Value], _n: usize) -> Result<Value, McError> {
    crate::sym_ops::solve_sym(args, 0, 0)
}

fn bi_double(args: &[Value], _n: usize) -> Result<Value, McError> {
    if args.len() != 1 {
        return Err(arity_err("double", "1", args.len()));
    }
    match &args[0] {
        // سمبولیک → عدد (CAS-0): کامل عددی باشد وگرنه خطای صادقانه
        Value::Sym(_) => crate::sym_ops::double_sym(&args[0], 0, 0),
        Value::Double(m) => Ok(Value::Double(m.clone())),
        Value::Complex(z) => Ok(Value::Complex(z.clone())),
        Value::Char(s) => Ok(Value::Double(Value::char_codes_row(s))),
        Value::Anon(_) => Err(McError::new(
            ids::TYPE_ERROR,
            0,
            0,
            "double: function handles have no numeric form.",
            "double: function_handle شکل عددی ندارد.",
        )),
        Value::Struct(_) => Err(McError::new(
            ids::TYPE_ERROR,
            0,
            0,
            "double: structs have no numeric form; extract fields with s.f first.",
            "double: ساختار شکل عددی ندارد؛ ابتدا فیلد را با s.f بیرون بکشید.",
        )),
        Value::List(_) => Err(McError::new(
            ids::TYPE_ERROR,
            0,
            0,
            "double: cell (list) has no numeric form; extract elements first (planned).",
            "double: cell (لیست) شکل عددی ندارد؛ ابتدا اعضا را جدا کنید (در برنامه).",
        )),
    }
}

/// `ischar(x)` — فقط کلاس char (ROADMAP 2.8).
fn bi_ischar(args: &[Value], _n: usize) -> Result<Value, McError> {
    if args.len() != 1 {
        return Err(arity_err("ischar", "1", args.len()));
    }
    Ok(scalar_val(matches!(args[0], Value::Char(_)) as i32 as f64))
}

/// `blanks(n)` — n فاصله (ROADMAP 2.8)؛ چک بودجه قبل از تخصیص (گام 2.5).
fn bi_blanks(args: &[Value], _n: usize) -> Result<Value, McError> {
    if args.len() != 1 {
        return Err(arity_err("blanks", "1", args.len()));
    }
    let n = match &args[0] {
        Value::Double(m) if m.is_scalar() => m.get(0, 0),
        _ => {
            return Err(McError::new(
                ids::TYPE_ERROR,
                0,
                0,
                "blanks: n must be a numeric scalar.",
                "blanks: n باید عدد اسکالر باشد.",
            ))
        }
    };
    if n < 0.0 || n.fract() != 0.0 {
        return Err(McError::new(
            ids::TYPE_ERROR,
            0,
            0,
            "blanks: n must be a non-negative integer.",
            "blanks: n باید عدد صحیح نامنفی باشد.",
        ));
    }
    let n = n as usize;
    budget::check_elements(1, n)?;
    Ok(Value::Char(" ".repeat(n)))
}

/// `str2double(s)` — رشته → عدد؛ شکست = NaN (رفتار متلب — ROADMAP 2.8).
/// می‌پذیرد: عدد اعشاری/علمی، Inf/NaN (بدون حساسیت بزرگی حروف)، فاصله‌های اطراف.
fn bi_str2double(args: &[Value], _n: usize) -> Result<Value, McError> {
    if args.len() != 1 {
        return Err(arity_err("str2double", "1", args.len()));
    }
    let s = match &args[0] {
        Value::Char(s) => s.as_str(),
        _ => {
            return Err(McError::new(
                ids::TYPE_ERROR,
                0,
                0,
                "str2double: input must be a char row vector like '1.5'.",
                "str2double: ورودی باید رشتهٔ کاراکتری مثل '1.5' باشد.",
            ))
        }
    };
    let t = s.trim();
    let v = if t.is_empty() {
        f64::NAN
    } else {
        match t.to_ascii_lowercase().as_str() {
            "inf" | "+inf" => f64::INFINITY,
            "-inf" => f64::NEG_INFINITY,
            "nan" => f64::NAN,
            _ => t.parse::<f64>().unwrap_or(f64::NAN),
        }
    };
    Ok(scalar_val(v))
}

fn bi_num2str(args: &[Value], _n: usize) -> Result<Value, McError> {
    if args.len() != 1 {
        return Err(arity_err("num2str", "1", args.len()));
    }
    match &args[0] {
        // سمبولیک → رشتهٔ نمایش خطی
        Value::Sym(_) => Ok(Value::Char(crate::sym_ops::sym_value_to_string(&args[0]))),
        Value::List(_) => Err(McError::new(
            ids::TYPE_ERROR,
            0,
            0,
            "num2str: cell (list) has no string form.",
            "num2str: cell (لیست) شکل رشته‌ای ندارد.",
        )),
        Value::Char(s) => Ok(Value::Char(s.clone())),
        Value::Double(m) if m.is_scalar() => {
            Ok(Value::Char(crate::format::format_num2str_elem(m.get(0, 0))))
        }
        Value::Double(m) if m.rows() == 1 || m.cols() == 1 => {
            // بردار: اعضا با یک فاصله — ترازبندی ستونی متلب با مدل char دو‌بعدی
            // (گام 2.8 آیتم ۶/۱۲) می‌آید؛ انحراف مستند (COMPATIBILITY §4.8)
            let parts: Vec<String> = (0..m.numel())
                .map(|i| crate::format::format_num2str_elem(m.get_linear(i)))
                .collect();
            Ok(Value::Char(parts.join(" ")))
        }
        Value::Double(_) => Err(McError::new(
            ids::UNSUPPORTED,
            0,
            0,
            "num2str: 2-D inputs produce 2-D char arrays; expansion E4 (row vectors convert fine).",
            "num2str: ورودی دوبعدی آرایهٔ char دوبعدی می‌سازد؛ انبساط E4 (بردارهای سطری تبدیل می‌شوند).",
        )),
        Value::Complex(_) => Err(McError::new(
            ids::UNSUPPORTED,
            0,
            0,
            "num2str: complex input is not supported yet.",
            "num2str: ورودی مختلط هنوز پشتیبانی نمی‌شود.",
        )),
        Value::Anon(_) => Err(McError::new(
            ids::TYPE_ERROR,
            0,
            0,
            "num2str: function handles have no numeric text form.",
            "num2str: function_handle شکل متنی عددی ندارد.",
        )),
        Value::Struct(_) => Err(McError::new(
            ids::TYPE_ERROR,
            0,
            0,
            "num2str: structs have no numeric text form; extract fields with s.f first.",
            "num2str: ساختار شکل متنی عددی ندارد؛ ابتدا فیلد را با s.f بیرون بکشید.",
        )),
    }
}

/// سقف انواع صحیح متلب — مقدار به‌صورت double برمی‌گردد (موتور هنوز نوع
/// صحیح ندارد — انحراف مستند COMPATIBILITY §4.8؛ نمایش رقمی یکسان است).
fn int_max_of(cls: &str) -> Option<f64> {
    match cls {
        "int8" => Some(127.0),
        "int16" => Some(32767.0),
        "int32" => Some(2147483647.0),
        "int64" => Some(9223372036854775807.0),
        "uint8" => Some(255.0),
        "uint16" => Some(65535.0),
        "uint32" => Some(4294967295.0),
        "uint64" => Some(18446744073709551615.0),
        _ => None,
    }
}

fn bi_intmax(args: &[Value], _n: usize) -> Result<Value, McError> {
    let cls = match args.len() {
        0 => "int32".to_owned(), // پیش‌فرض متلب
        1 => arg_char(&args[0], "intmax")?,
        n => return Err(arity_err("intmax", "0..1", n)),
    };
    match int_max_of(&cls) {
        Some(v) => Ok(scalar_val(v)),
        None => Err(McError::new(
            ids::TYPE_ERROR,
            0,
            0,
            format!("intmax: unknown integer class '{cls}' (int8..int64, uint8..uint64)."),
            format!("intmax: کلاس صحیح ناشناخته «{cls}» (int8 تا int64، uint8 تا uint64)."),
        )),
    }
}

fn bi_realmax(args: &[Value], _n: usize) -> Result<Value, McError> {
    if let Some(a) = args.first() {
        let cls = arg_char(a, "realmax")?;
        if cls != "double" {
            return Err(McError::new(
                ids::UNSUPPORTED,
                0,
                0,
                "realmax: only the 'double' class is supported.",
                "realmax: فقط کلاس 'double' پشتیبانی می‌شود.",
            ));
        }
    }
    Ok(scalar_val(f64::MAX))
}

fn bi_realmin(args: &[Value], _n: usize) -> Result<Value, McError> {
    if let Some(a) = args.first() {
        let cls = arg_char(a, "realmin")?;
        if cls != "double" {
            return Err(McError::new(
                ids::UNSUPPORTED,
                0,
                0,
                "realmin: only the 'double' class is supported.",
                "realmin: فقط کلاس 'double' پشتیبانی می‌شود.",
            ));
        }
    }
    Ok(scalar_val(f64::MIN_POSITIVE))
}

/// disp/fprintf در بافت عبارت (مثل `x = disp(1)`) خطای صادقانه می‌دهند —
/// خروجی کنسولی مقدار برنمی‌گردانند (رفتار متلب).
fn bi_display_no_value(args: &[Value], _n: usize) -> Result<Value, McError> {
    let _ = args;
    Err(McError::new(
        ids::TYPE_ERROR,
        0,
        0,
        "disp/fprintf produce console output and cannot be used inside an expression.",
        "disp/fprintf خروجی کنسول می‌دهند و داخل عبارت قابل استفاده نیستند.",
    ))
}

/// توابع رسم گزاره‌ای‌اند (الگوی disp — P3-a): اثرشان PlotOp است نه مقدار؛
/// فراخوانی داخل عبارت/انتساب خطای روشن می‌دهد (MATLAB-lite ثبت‌شده).
fn bi_plot_no_value(_args: &[Value], _n: usize) -> Result<Value, McError> {
    Err(McError::new(
        ids::TYPE_ERROR,
        0,
        0,
        "plotting functions are statement-only and cannot be used inside an expression.",
        "توابع رسم فقط گزاره‌ای‌اند و داخل عبارت قابل استفاده نیستند.",
    ))
}

// ---------- منطقی ----------

fn bi_true(_args: &[Value], _n: usize) -> Result<Value, McError> {
    Ok(scalar_val(1.0))
}

fn bi_false(_args: &[Value], _n: usize) -> Result<Value, McError> {
    Ok(scalar_val(0.0))
}

fn bi_isequal(args: &[Value], _n: usize) -> Result<Value, McError> {
    if args.len() < 2 {
        return Err(arity_err("isequal", "2", args.len()));
    }
    let ok = args.windows(2).all(|w| w[0] == w[1]);
    Ok(scalar_val(ok as i32 as f64))
}

// ---------- عنصری (پیاده‌سازی‌ها) ----------

fn bi_abs(x: f64) -> Result<f64, McError> {
    Ok(x.abs())
}
fn bi_sqrt(x: f64) -> Result<f64, McError> {
    // sqrt(-x) مختلط است — متلب مختلط می‌دهد؛ P1: خطای صریح آموزشی
    if x < 0.0 {
        return Err(McError::new(
            ids::UNSUPPORTED,
            0,
            0,
            "sqrt of a negative number is complex — arrives with P2 complex support.",
            "جذر عدد منفی مختلط است — با پشتیبانی مختلط P2 می‌آید.",
        ));
    }
    Ok(x.sqrt())
}
fn bi_exp(x: f64) -> Result<f64, McError> {
    Ok(x.exp())
}
fn bi_log(x: f64) -> Result<f64, McError> {
    Ok(x.ln()) // ln(0) = -Inf، ln منفی = NaN (IEEE — سند ۰۳ §11)
}
fn bi_log2(x: f64) -> Result<f64, McError> {
    Ok(x.log2())
}
fn bi_log10(x: f64) -> Result<f64, McError> {
    Ok(x.log10())
}
fn bi_sin(x: f64) -> Result<f64, McError> {
    Ok(x.sin())
}
fn bi_cos(x: f64) -> Result<f64, McError> {
    Ok(x.cos())
}
fn bi_tan(x: f64) -> Result<f64, McError> {
    Ok(x.tan())
}
fn bi_asin(x: f64) -> Result<f64, McError> {
    Ok(x.asin())
}
fn bi_acos(x: f64) -> Result<f64, McError> {
    Ok(x.acos())
}
fn bi_atan(x: f64) -> Result<f64, McError> {
    Ok(x.atan())
}
fn bi_sinh(x: f64) -> Result<f64, McError> {
    Ok(x.sinh())
}
fn bi_cosh(x: f64) -> Result<f64, McError> {
    Ok(x.cosh())
}
fn bi_tanh(x: f64) -> Result<f64, McError> {
    Ok(x.tanh())
}
fn bi_sign(x: f64) -> Result<f64, McError> {
    if x.is_nan() {
        return Ok(f64::NAN);
    }
    Ok(if x > 0.0 {
        1.0
    } else if x < 0.0 {
        -1.0
    } else {
        0.0
    })
}
fn bi_round(x: f64) -> Result<f64, McError> {
    // round متلب: نیمه‌ها «دور از صفر» — نه banker's
    Ok(if x >= 0.0 {
        (x + 0.5).floor()
    } else {
        (x - 0.5).ceil()
    })
}
fn bi_floor(x: f64) -> Result<f64, McError> {
    Ok(x.floor())
}
fn bi_ceil(x: f64) -> Result<f64, McError> {
    Ok(x.ceil())
}
fn bi_fix(x: f64) -> Result<f64, McError> {
    Ok(x.trunc())
}
fn bi_angle(x: f64) -> Result<f64, McError> {
    // روی محور حقیقی: منفی → π، غیرمنفی → 0
    Ok(if x < 0.0 { std::f64::consts::PI } else { 0.0 })
}

// ---------- فایل/دادهٔ I/O (گام 3.7 — ROADMAP §3.7) ----------

/// `A = readmatrix('data.csv')` — متن محدودشدهٔ ثبت‌شده در VFS نشست.
fn bi_readmatrix(args: &[Value], line: usize) -> Result<Value, McError> {
    crate::io_builtins::readmatrix_fn(args, line)
}

/// `readtable` — ارجاع صادقانه به readmatrix (نوع table = E4؛ پیمان ضد-انحراف).
fn bi_readtable(args: &[Value], line: usize) -> Result<Value, McError> {
    crate::io_builtins::readtable_fn(args, line)
}
