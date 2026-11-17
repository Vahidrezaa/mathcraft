//! توابع رسم گزاره‌ای (ROADMAP 3.2 — P3-a).
//!
//! الگوی `disp`/`rng` (eval.rs): فقط به‌صورت گزاره قابل فراخوانی‌اند — بدون
//! اکو، بدون `ans`، بدون مقدار برگشتی. فراخوانی داخل عبارت به رجیستری
//! می‌خورد که خطای روشن می‌دهد (`bi_plot_no_value`).
//!
//! خالص: هر تابع فقط دادهٔ ورودی را به [`PlotOp`] تبدیل و به سینک
//! `mc_plot::session` می‌فرستد؛ حالت (figure/hold/subplot) در کاهش‌دهندهٔ
//! `FigureState` جمع می‌شود (fold قابل‌بازپخش). واگرایی‌های MATLAB-lite در
//! COMPATIBILITY ثبت شده‌اند: یک استایل در هر فراخوانی، چندجفت‌سری در
//! `fill`، بدون `barh`/`loglog`/toggle-hold/`axis auto`/`DisplayName`.

use mc_core::{ids, Matrix, McError, Value};
use mc_plot::histogram::{self, HistNorm};
use mc_plot::linespec;
use mc_plot::session;
use mc_plot::spec::{ErrBand, PlotColorSpec, PlotOp, SeriesData, SeriesPair, StyleSpec};

/// نام‌های گزاره‌ای رسم — الگوی dict در exec_stmt (تقدم: متغیر/تابع کاربر).
pub const PLOT_STATEMENT_FNS: [&str; 18] = [
    "plot",
    "scatter",
    "bar",
    "histogram",
    "area",
    "fill",
    "polarplot",
    "errorbar",
    "fplot",
    "subplot",
    "legend",
    "axis",
    "grid",
    "xlabel",
    "ylabel",
    "title",
    "hold",
    "figure",
];

/// آیا نام، تابع رسم گزاره‌ای است؟
pub fn is_plot_statement_fn(name: &str) -> bool {
    PLOT_STATEMENT_FNS.contains(&name)
}

/// اجرای یک تابع رسم گزاره‌ای با آرگومان‌های ارزیابی‌شده.
pub fn apply(name: &str, args: &[Value], line: u32, col: u32) -> Result<(), McError> {
    match name {
        "plot" => bi_plot(args, line, col),
        "scatter" => bi_scatter(args, line, col),
        "bar" => bi_bar(args, line, col),
        "histogram" => bi_histogram(args, line, col),
        "area" => bi_area(args, line, col),
        "fill" => bi_fill(args, line, col),
        "polarplot" => bi_polarplot(args, line, col),
        "errorbar" => bi_errorbar(args, line, col),
        "fplot" => bi_fplot(args, line, col),
        "subplot" => bi_subplot(args, line, col),
        "legend" => bi_legend(args, line, col),
        "axis" => bi_axis(args, line, col),
        "grid" => bi_grid(args, line, col),
        "xlabel" => bi_xlabel(args, line, col),
        "ylabel" => bi_ylabel(args, line, col),
        "title" => bi_title(args, line, col),
        "hold" => bi_hold(args, line, col),
        "figure" => bi_figure(args, line, col),
        other => Err(perr(
            line,
            col,
            format!("unknown plot builtin `{other}`."),
            format!("توابع رسم ناشناخته `{other}`."),
        )),
    }
}

// ---------- خطاها و کمکی‌ها ----------

fn perr(line: u32, col: u32, en: String, fa: String) -> McError {
    McError::new(ids::TYPE_ERROR, line, col, en, fa)
}

/// وکتور عددی از مقدار (ستون/سطر/اسکالر؛ مختلط فقط اگر همه‌واقعی).
fn numeric_vec(v: &Value, name: &str, line: u32, col: u32) -> Result<Vec<f64>, McError> {
    match v {
        Value::Double(m) => Ok((0..m.numel()).map(|k| m.get_linear(k)).collect()),
        Value::Complex(z) if z.is_all_real() => Ok(z.re_column_slice().to_vec()),
        Value::Complex(_) => Err(perr(
            line,
            col,
            format!("{name}: complex values are not plottable (use real())."),
            format!("{name}: مقدار مختلط قابل رسم نیست (real() بگیرید)."),
        )),
        _ => Err(perr(
            line,
            col,
            format!("{name}: numeric argument expected."),
            format!("{name}: آرگومان عددی لازم است."),
        )),
    }
}

fn is_numeric(v: &Value) -> bool {
    matches!(v, Value::Double(_) | Value::Complex(_))
}

fn scalar_of(v: &Value, name: &str, line: u32, col: u32) -> Result<f64, McError> {
    match v {
        Value::Double(m) if m.numel() == 1 => Ok(m.get_linear(0)),
        Value::Complex(z) if z.is_all_real() && z.numel() == 1 => Ok(z.re_column_slice()[0]),
        _ => Err(perr(
            line,
            col,
            format!("{name}: a scalar number is expected here."),
            format!("{name}: این‌جا یک عدد اسکالر لازم است."),
        )),
    }
}

fn char_of(v: &Value, name: &str, line: u32, col: u32) -> Result<String, McError> {
    match v {
        Value::Char(s) => Ok(s.clone()),
        _ => Err(perr(
            line,
            col,
            format!("{name}: a character string is expected here."),
            format!("{name}: این‌جا یک رشته کاراکتری لازم است."),
        )),
    }
}

/// (ردیف، ستون، دادهٔ ستون‌مجنون) از مقدار عددی.
fn shape_of(v: &Value) -> Option<(usize, usize, Vec<f64>)> {
    match v {
        Value::Double(m) => Some((
            m.rows(),
            m.cols(),
            (0..m.numel()).map(|k| m.get_linear(k)).collect(),
        )),
        Value::Complex(z) if z.is_all_real() => {
            Some((z.rows(), z.cols(), z.re_column_slice().to_vec()))
        }
        _ => None,
    }
}

/// ستون‌های ماتریس به‌صورت وکتور — وکتور (1×n یا n×1) = «یک» سری
/// (پارتی متلب: plot(1:3) یک خط است، نه سه نقطهٔ جدا).
fn columns_of(v: &Value, name: &str, line: u32, col: u32) -> Result<Vec<Vec<f64>>, McError> {
    match shape_of(v) {
        Some((rows, cols, data)) => {
            if rows == 0 || cols == 0 {
                return Err(perr(
                    line,
                    col,
                    format!("{name}: empty data cannot be plotted."),
                    format!("{name}: دادهٔ خالی قابل رسم نیست."),
                ));
            }
            if rows == 1 || cols == 1 {
                return Ok(vec![data]);
            }
            Ok((0..cols)
                .map(|c| (0..rows).map(|r| data[r + c * rows]).collect())
                .collect())
        }
        None => numeric_vec(v, name, line, col).map(|v| vec![v]),
    }
}

/// خروجی xy_args: (x اختیاری، ستون‌های Y، ایندکس آرگومان بعدی).
type XyParsed = (Option<Vec<f64>>, Vec<Vec<f64>>, usize);

/// ورودی XY مشترک plot/scatter/bar/area.
fn xy_args(args: &[Value], name: &str, line: u32, col: u32) -> Result<XyParsed, McError> {
    if args.is_empty() || !is_numeric(&args[0]) {
        return Err(perr(
            line,
            col,
            format!("{name}: numeric data expected, e.g. {name}(x, y)."),
            format!("{name}: دادهٔ عددی لازم است، مثل {name}(x, y)."),
        ));
    }
    let (x, ycols, next) = if args.len() >= 2 && is_numeric(&args[1]) {
        let x = numeric_vec(&args[0], name, line, col)?;
        let ycols = columns_of(&args[1], name, line, col)?;
        (Some(x), ycols, 2)
    } else {
        let ycols = columns_of(&args[0], name, line, col)?;
        (None, ycols, 1)
    };
    if let Some(x) = &x {
        let n = ycols.first().map_or(0, Vec::len);
        if !x.is_empty() && x.len() != n {
            return Err(perr(
                line,
                col,
                format!(
                    "{name}: x has {} points but y has {} — lengths must match.",
                    x.len(),
                    n
                ),
                format!(
                    "{name}: طول x برابر {} است ولی y برابر {} — باید یکی باشند.",
                    x.len(),
                    n
                ),
            ));
        }
    }
    Ok((x, ycols, next))
}

/// استایل از آرگومان‌های انتهایی: LineSpec ('r--o') + LineWidth/Color +
/// پرچم 'stacked' (bar). برگشت: (استایل، stacked).
fn style_from_trailing(
    args: &[Value],
    skip: usize,
    name: &str,
    line: u32,
    col: u32,
) -> Result<(StyleSpec, bool), McError> {
    let mut style = StyleSpec::default();
    let mut stacked = false;
    let mut i = skip;
    while i < args.len() {
        match &args[i] {
            Value::Char(s) => match s.as_str() {
                "LineWidth" => {
                    i += 1;
                    let w = args
                        .get(i)
                        .and_then(|v| scalar_of(v, name, line, col).ok())
                        .ok_or_else(|| {
                            perr(
                                line,
                                col,
                                "LineWidth needs a positive number.".into(),
                                "LineWidth عددی مثبت لازم دارد.".into(),
                            )
                        })?;
                    if !(w.is_finite() && w > 0.0) {
                        return Err(perr(
                            line,
                            col,
                            "LineWidth must be a positive finite number.".into(),
                            "LineWidth باید عددی مثبت و متناهی باشد.".into(),
                        ));
                    }
                    style.width = Some(w as f32);
                }
                "Color" => {
                    i += 1;
                    let c = args.get(i).ok_or_else(|| {
                        perr(
                            line,
                            col,
                            "Color needs a value ('r' or [r g b]).".into(),
                            "Color مقدار لازم دارد ('r' یا [r g b]).".into(),
                        )
                    })?;
                    style.color = Some(color_value(c, name, line, col)?);
                }
                "stacked" => stacked = true,
                "DisplayName" => {
                    return Err(perr(
                        line,
                        col,
                        "DisplayName is not supported yet — use legend('label', ...).".into(),
                        "DisplayName هنوز پشتیبانی نمی‌شود — از legend('برچسب', ...) استفاده کنید."
                            .into(),
                    ));
                }
                other => {
                    let parts = linespec::parse(other).map_err(|e| {
                        perr(
                            line,
                            col,
                            format!("{name}: {e}"),
                            format!("{name}: رشتهٔ استایل شناخته نشد ({e})."),
                        )
                    })?;
                    if parts == linespec::LineSpecParts::default() {
                        return Err(perr(
                            line,
                            col,
                            format!("{name}: unknown option '{other}'."),
                            format!("{name}: گزینهٔ ناشناخته «{other}»."),
                        ));
                    }
                    style = linespec::merge_into(&style, parts);
                }
            },
            _ => {
                return Err(perr(
                    line,
                    col,
                    format!("{name}: trailing arguments must be a style string or options."),
                    format!("{name}: آرگومان‌های انتهایی باید رشتهٔ استایل یا گزینه باشند."),
                ));
            }
        }
        i += 1;
    }
    Ok((style, stacked))
}

/// مقدار رنگ: نام کوتاه/بلند متلب یا سه‌تایی [r g b] (0..1 متلب یا 0..255).
fn color_value(v: &Value, name: &str, line: u32, col: u32) -> Result<PlotColorSpec, McError> {
    if let Value::Char(s) = v {
        let c = s.chars().next();
        if s.chars().count() == 1 {
            if let Some(color) = c.and_then(linespec::color_char) {
                return Ok(color);
            }
        }
        let long = match s.as_str() {
            "red" => (255, 0, 0),
            "green" => (0, 255, 0),
            "blue" => (0, 0, 255),
            "cyan" => (0, 255, 255),
            "magenta" => (255, 0, 255),
            "yellow" => (255, 255, 0),
            "black" => (0, 0, 0),
            "white" => (255, 255, 255),
            _ => {
                return Err(perr(
                    line,
                    col,
                    format!("{name}: unknown color name '{s}'."),
                    format!("{name}: نام رنگ ناشناخته «{s}»."),
                ));
            }
        };
        return Ok(PlotColorSpec::Rgb {
            r: long.0,
            g: long.1,
            b: long.2,
        });
    }
    let t = numeric_vec(v, name, line, col)?;
    if t.len() == 3 && t.iter().all(|x| x.is_finite()) {
        let scale = |x: f64| -> u8 {
            // متلب: سه‌تایی 0..1؛ پذیرش 0..255 هم برای بخشندگی
            let v = if x <= 1.0 { x * 255.0 } else { x };
            v.clamp(0.0, 255.0).round() as u8
        };
        return Ok(PlotColorSpec::Rgb {
            r: scale(t[0]),
            g: scale(t[1]),
            b: scale(t[2]),
        });
    }
    Err(perr(
        line,
        col,
        format!("{name}: color must be a name or an [r g b] triple."),
        format!("{name}: رنگ باید نام یا سه‌تایی [r g b] باشد."),
    ))
}

fn x_or_default(x: &Option<Vec<f64>>, n: usize) -> Vec<f64> {
    match x {
        Some(v) if !v.is_empty() => v.clone(),
        _ => (1..=n).map(|k| k as f64).collect(),
    }
}

// ---------- توابع ----------

fn bi_plot(args: &[Value], line: u32, col: u32) -> Result<(), McError> {
    let (x, ycols, next) = xy_args(args, "plot", line, col)?;
    let (style, _) = style_from_trailing(args, next, "plot", line, col)?;
    let n = ycols.first().map_or(0, Vec::len);
    let xs = x_or_default(&x, n);
    let series = ycols
        .into_iter()
        .map(|y| SeriesPair {
            data: SeriesData { x: xs.clone(), y },
            style: style.clone(),
        })
        .collect();
    session::emit(PlotOp::Line { series });
    Ok(())
}

fn bi_scatter(args: &[Value], line: u32, col: u32) -> Result<(), McError> {
    let (x, ycols, next) = xy_args(args, "scatter", line, col)?;
    let (style, _) = style_from_trailing(args, next, "scatter", line, col)?;
    let n = ycols.first().map_or(0, Vec::len);
    let xs = x_or_default(&x, n);
    let series = ycols
        .into_iter()
        .map(|y| SeriesPair {
            data: SeriesData { x: xs.clone(), y },
            style: style.clone(),
        })
        .collect();
    session::emit(PlotOp::Scatter { series });
    Ok(())
}

fn bi_bar(args: &[Value], line: u32, col: u32) -> Result<(), McError> {
    let (x, ycols, mut next) = xy_args(args, "bar", line, col)?;
    // عرض نسبی به‌صورت موقعیتی (bar(x, y, w))
    let mut width = None;
    if let Some(v) = args.get(next) {
        if is_numeric(v) && v_is_scalar(v) {
            let w = scalar_of(v, "bar", line, col)?;
            if !(w.is_finite() && w > 0.0 && w <= 1.0) {
                return Err(perr(
                    line,
                    col,
                    "bar width must be in (0, 1].".into(),
                    "عرض میله باید در بازهٔ (0, 1] باشد.".into(),
                ));
            }
            width = Some(w as f32);
            next += 1;
        }
    }
    let (style, stacked) = style_from_trailing(args, next, "bar", line, col)?;
    let n = ycols.first().map_or(0, Vec::len);
    let xs = x_or_default(&x, n);
    let series = ycols
        .into_iter()
        .map(|y| SeriesPair {
            data: SeriesData { x: xs.clone(), y },
            style: style.clone(),
        })
        .collect();
    session::emit(PlotOp::Bar {
        series,
        width,
        horizontal: false,
        stacked,
    });
    Ok(())
}

fn v_is_scalar(v: &Value) -> bool {
    match v {
        Value::Double(m) => m.numel() == 1,
        Value::Complex(z) => z.numel() == 1,
        _ => false,
    }
}

fn bi_histogram(args: &[Value], line: u32, col: u32) -> Result<(), McError> {
    if args.is_empty() || !is_numeric(&args[0]) {
        return Err(perr(
            line,
            col,
            "histogram(y): numeric data expected.".into(),
            "histogram(y): دادهٔ عددی لازم است.".into(),
        ));
    }
    let data = numeric_vec(&args[0], "histogram", line, col)?;
    let data: Vec<f64> = data.into_iter().filter(|x| x.is_finite()).collect();
    if data.is_empty() {
        return Err(perr(
            line,
            col,
            "histogram: no finite data to bin.".into(),
            "histogram: دادهٔ متناهی برای بسته‌بندی نیست.".into(),
        ));
    }
    let mut nbins: Option<usize> = None;
    let mut edges_opt: Option<Vec<f64>> = None;
    let mut norm = HistNorm::Count;
    let mut i = 1;
    while i < args.len() {
        match &args[i] {
            Value::Double(m) if m.numel() == 1 && nbins.is_none() && edges_opt.is_none() => {
                let n = m.get_linear(0);
                if !(n >= 1.0 && n.fract() == 0.0 && n <= 1000.0) {
                    return Err(perr(
                        line,
                        col,
                        "histogram: bin count must be an integer in 1..=1000.".into(),
                        "histogram: تعداد بسته باید عدد صحیح 1..=1000 باشد.".into(),
                    ));
                }
                nbins = Some(n as usize);
            }
            Value::Double(_) | Value::Complex(_) if edges_opt.is_none() => {
                let e = numeric_vec(&args[i], "histogram", line, col)?;
                if e.len() < 2 || e.windows(2).any(|w| w[1] <= w[0]) {
                    return Err(perr(
                        line,
                        col,
                        "histogram: edges must be a strictly increasing vector.".into(),
                        "histogram: یال‌ها باید برداری صعودی اکید باشند.".into(),
                    ));
                }
                edges_opt = Some(e);
            }
            Value::Char(s) if s == "Normalization" => {
                i += 1;
                let v = args.get(i).ok_or_else(|| {
                    perr(
                        line,
                        col,
                        "Normalization needs a value.".into(),
                        "Normalization مقدار لازم دارد.".into(),
                    )
                })?;
                norm = match char_of(v, "histogram", line, col)?.as_str() {
                    "count" => HistNorm::Count,
                    "pdf" => HistNorm::Pdf,
                    "cdf" => HistNorm::Cdf,
                    other => {
                        return Err(perr(
                            line,
                            col,
                            format!("histogram: unknown normalization '{other}' (count/pdf/cdf)."),
                            format!("histogram: نرمال‌سازی ناشناخته «{other}» (count/pdf/cdf)."),
                        ));
                    }
                };
            }
            _ => {
                return Err(perr(
                    line,
                    col,
                    "histogram: unexpected argument.".into(),
                    "histogram: آرگومان غیرمنتظره.".into(),
                ));
            }
        }
        i += 1;
    }
    let (edges, counts) = match &edges_opt {
        Some(e) => {
            let c = histogram::counts(&data, e);
            (e.clone(), c)
        }
        None => {
            let mn = data.iter().copied().fold(f64::INFINITY, f64::min);
            let mx = data.iter().copied().fold(f64::NEG_INFINITY, f64::max);
            let e = histogram::uniform_edges(mn, mx, nbins.unwrap_or(histogram::DEFAULT_NBINS));
            let c = histogram::counts(&data, &e);
            (e, c)
        }
    };
    let counts = histogram::apply_norm(&counts, &edges, norm);
    session::emit(PlotOp::Histogram {
        edges,
        counts,
        norm,
        style: StyleSpec::default(),
    });
    Ok(())
}

fn bi_area(args: &[Value], line: u32, col: u32) -> Result<(), McError> {
    let (x, ycols, next) = xy_args(args, "area", line, col)?;
    let (style, _) = style_from_trailing(args, next, "area", line, col)?;
    let n = ycols.first().map_or(0, Vec::len);
    let xs = x_or_default(&x, n);
    let series = ycols
        .into_iter()
        .map(|y| SeriesPair {
            data: SeriesData { x: xs.clone(), y },
            style: style.clone(),
        })
        .collect();
    session::emit(PlotOp::Area {
        series,
        baseline: 0.0,
    });
    Ok(())
}

fn bi_fill(args: &[Value], line: u32, col: u32) -> Result<(), McError> {
    // fill(x1,y1, x2,y2, ... [, style]) — زوج‌ها؛ استایل مشترک (MATLAB-lite)
    let mut nums: Vec<Vec<f64>> = Vec::new();
    let mut style_idx = None;
    for (i, a) in args.iter().enumerate() {
        if is_numeric(a) {
            nums.push(numeric_vec(a, "fill", line, col)?);
        } else if matches!(a, Value::Char(_)) && nums.len().is_multiple_of(2) && style_idx.is_none()
        {
            style_idx = Some(i);
            break;
        } else {
            return Err(perr(
                line,
                col,
                "fill: expected (x, y) pairs then an optional style string.".into(),
                "fill: زوج‌های (x, y) و سپس رشتهٔ استایل اختیاری لازم است.".into(),
            ));
        }
    }
    if nums.is_empty() || !nums.len().is_multiple_of(2) {
        return Err(perr(
            line,
            col,
            "fill: arguments must be (x, y) pairs.".into(),
            "fill: آرگومان‌ها باید زوج‌های (x, y) باشند.".into(),
        ));
    }
    let (style, _) = match style_idx {
        Some(i) => style_from_trailing(args, i, "fill", line, col)?,
        None => (StyleSpec::default(), false),
    };
    let mut polys = Vec::new();
    let mut k = 0;
    while k + 1 < nums.len() {
        let (x, y) = (&nums[k], &nums[k + 1]);
        if x.len() != y.len() || x.is_empty() {
            return Err(perr(
                line,
                col,
                "fill: each polygon needs x and y of equal nonzero length.".into(),
                "fill: هر چندضلعی x و y هم‌طول و ناتهی لازم دارد.".into(),
            ));
        }
        polys.push(SeriesData {
            x: x.clone(),
            y: y.clone(),
        });
        k += 2;
    }
    session::emit(PlotOp::Fill { polys, style });
    Ok(())
}

fn bi_polarplot(args: &[Value], line: u32, col: u32) -> Result<(), McError> {
    let (theta, rcols, next) = xy_args(args, "polarplot", line, col)?;
    let (style, _) = style_from_trailing(args, next, "polarplot", line, col)?;
    let n = rcols.first().map_or(0, Vec::len);
    let th = x_or_default(&theta, n);
    let series = rcols
        .into_iter()
        .map(|r| SeriesPair {
            data: SeriesData {
                x: th.clone(),
                y: r,
            },
            style: style.clone(),
        })
        .collect();
    session::emit(PlotOp::Polar { series });
    Ok(())
}

fn bi_errorbar(args: &[Value], line: u32, col: u32) -> Result<(), McError> {
    // errorbar(y, e) | errorbar(x, y, e) | errorbar(x, y, yneg, ypos)
    // | errorbar(x, y, xneg, xpos, yneg, ypos)
    let (x, ycols, _) = xy_args(args, "errorbar", line, col)?;
    if ycols.len() != 1 {
        return Err(perr(
            line,
            col,
            "errorbar: a single y vector is supported (one call per series).".into(),
            "errorbar: یک بردار y در هر فراخوانی پشتیبانی می‌شود.".into(),
        ));
    }
    let y = ycols.into_iter().next().unwrap_or_default();
    let n = y.len();
    let x = x_or_default(&x, n);
    let (x_err, y_err): (Option<ErrBand>, Option<ErrBand>) = match args.len() {
        2 | 3 => {
            // (y, e) یا (x, y, e): متقارن y
            let e = numeric_vec(&args[args.len() - 1], "errorbar", line, col)?;
            if e.len() == n && e.iter().all(|v| v.is_finite() && *v >= 0.0) {
                (
                    None,
                    Some(ErrBand {
                        neg: e.clone(),
                        pos: e,
                    }),
                )
            } else {
                return Err(perr(
                    line,
                    col,
                    "errorbar: error lengths must be >= 0 and match the data length.".into(),
                    "errorbar: طول‌های خطا باید ≥ 0 و هم‌طول داده باشند.".into(),
                ));
            }
        }
        4 => {
            // (x, y, yneg, ypos)
            let neg = numeric_vec(&args[2], "errorbar", line, col)?;
            let pos = numeric_vec(&args[3], "errorbar", line, col)?;
            if neg.len() == n && pos.len() == n {
                (None, Some(ErrBand { neg, pos }))
            } else {
                return Err(perr(
                    line,
                    col,
                    "errorbar: error vectors must match the data length.".into(),
                    "errorbar: وکتورهای خطا باید هم‌طول داده باشند.".into(),
                ));
            }
        }
        6 => {
            let xn = numeric_vec(&args[2], "errorbar", line, col)?;
            let xp = numeric_vec(&args[3], "errorbar", line, col)?;
            let yn = numeric_vec(&args[4], "errorbar", line, col)?;
            let yp = numeric_vec(&args[5], "errorbar", line, col)?;
            if xn.len() == n && xp.len() == n && yn.len() == n && yp.len() == n {
                (
                    Some(ErrBand { neg: xn, pos: xp }),
                    Some(ErrBand { neg: yn, pos: yp }),
                )
            } else {
                return Err(perr(
                    line,
                    col,
                    "errorbar: error vectors must match the data length.".into(),
                    "errorbar: وکتورهای خطا باید هم‌طول داده باشند.".into(),
                ));
            }
        }
        _ => {
            return Err(perr(
                line,
                col,
                "errorbar: supported forms are (y,e), (x,y,e), (x,y,yneg,ypos), (x,y,xneg,xpos,yneg,ypos).".into(),
                "errorbar: شکل‌های مجاز: (y,e) و (x,y,e) و (x,y,yneg,ypos) و (x,y,xneg,xpos,yneg,ypos).".into(),
            ));
        }
    };
    session::emit(PlotOp::ErrorBar {
        series: vec![mc_plot::spec::ErrorPair {
            data: SeriesData { x, y },
            x_err,
            y_err,
            style: StyleSpec::default(),
        }],
    });
    Ok(())
}

fn bi_fplot(args: &[Value], line: u32, col: u32) -> Result<(), McError> {
    // fplot(f, [a b]) | fplot(f) → [-5, 5] — نمونه‌برداری ۴۰۰ نقطه‌ای
    let f = match args.first() {
        Some(Value::Anon(f)) => f.clone(),
        _ => {
            return Err(perr(
                line,
                col,
                "fplot(f, [a b]): first argument must be a function handle @(x)...".into(),
                "fplot(f, [a b]): آرگومان اول باید هندل تابع @(x)... باشد.".into(),
            ));
        }
    };
    let (a, b) = if let Some(v) = args.get(1) {
        let r = numeric_vec(v, "fplot", line, col)?;
        if r.len() == 2 && r[0] < r[1] && r[0].is_finite() && r[1].is_finite() {
            (r[0], r[1])
        } else {
            return Err(perr(
                line,
                col,
                "fplot: domain must be an increasing pair [a b].".into(),
                "fplot: دامنه باید جفت صعودی [a b] باشد.".into(),
            ));
        }
    } else {
        (-5.0, 5.0)
    };
    const SAMPLES: usize = 400;
    let mut xs = Vec::with_capacity(SAMPLES);
    let mut ys = Vec::with_capacity(SAMPLES);
    for k in 0..SAMPLES {
        let x = a + (b - a) * k as f64 / (SAMPLES - 1) as f64;
        let out = (f.call)(&[Value::Double(Matrix::full(1, 1, x))])?;
        let y = scalar_of(&out, "fplot", line, col)?;
        xs.push(x);
        ys.push(y);
    }
    session::emit(PlotOp::Line {
        series: vec![SeriesPair {
            data: SeriesData { x: xs, y: ys },
            style: StyleSpec::default(),
        }],
    });
    Ok(())
}

fn bi_subplot(args: &[Value], line: u32, col: u32) -> Result<(), McError> {
    if args.len() != 3 {
        return Err(perr(
            line,
            col,
            "subplot(m, n, p): exactly three arguments expected.".into(),
            "subplot(m, n, p): دقیقاً سه آرگومان لازم است.".into(),
        ));
    }
    let vals = [
        scalar_of(&args[0], "subplot", line, col)?,
        scalar_of(&args[1], "subplot", line, col)?,
        scalar_of(&args[2], "subplot", line, col)?,
    ];
    let ints: Vec<u32> = vals
        .iter()
        .map(|v| (*v as u64).min(u32::MAX as u64) as u32)
        .collect();
    if vals.iter().any(|v| v.fract() != 0.0 || *v < 1.0) {
        return Err(perr(
            line,
            col,
            "subplot: m, n, p must be positive integers.".into(),
            "subplot: m و n و p باید اعداد صحیح مثبت باشند.".into(),
        ));
    }
    let (m, n, p) = (ints[0], ints[1], ints[2]);
    if m > 16 || n > 16 {
        return Err(perr(
            line,
            col,
            "subplot: grid larger than 16x16 is not supported.".into(),
            "subplot: شبکهٔ بزرگ‌تر از 16x16 پشتیبانی نمی‌شود.".into(),
        ));
    }
    if p < 1 || p > m * n {
        return Err(perr(
            line,
            col,
            format!("subplot: p must be in 1..={} (got {p}).", m * n),
            format!("subplot: p باید در 1..={} باشد (نه {p}).", m * n),
        ));
    }
    session::emit(PlotOp::SubplotSelect {
        rows: m,
        cols: n,
        index: p,
    });
    Ok(())
}

fn bi_legend(args: &[Value], line: u32, col: u32) -> Result<(), McError> {
    if args.is_empty() {
        return Err(perr(
            line,
            col,
            "legend: expected labels like legend('a', 'b') or legend('off').".into(),
            "legend: برچسب‌ها لازم است مثل legend('a', 'b') یا legend('off').".into(),
        ));
    }
    if args.len() == 1 {
        if let Value::Char(s) = &args[0] {
            if s == "off" {
                session::emit(PlotOp::Legend { entries: None });
                return Ok(());
            }
        }
    }
    let mut entries = Vec::new();
    for a in args {
        entries.push(char_of(a, "legend", line, col)?);
    }
    session::emit(PlotOp::Legend {
        entries: Some(entries),
    });
    Ok(())
}

fn bi_axis(args: &[Value], line: u32, col: u32) -> Result<(), McError> {
    if args.len() != 1 {
        return Err(perr(
            line,
            col,
            "axis: one argument expected ([a b c d] or 'equal'/'tight'/'on'/'off').".into(),
            "axis: یک آرگومان لازم است ([a b c d] یا 'equal'/'tight'/'on'/'off').".into(),
        ));
    }
    match &args[0] {
        Value::Char(s) => match s.as_str() {
            "equal" => session::emit(PlotOp::AxisEqual { on: true }),
            "tight" => session::emit(PlotOp::AxisTight),
            "on" => session::emit(PlotOp::AxisVisible { on: true }),
            "off" => session::emit(PlotOp::AxisVisible { on: false }),
            "auto" | "normal" => {
                return Err(perr(
                    line,
                    col,
                    "axis('auto') is not supported yet — set limits explicitly.".into(),
                    "axis('auto') هنوز پشتیبانی نمی‌شود — حدود را صریح بدهید.".into(),
                ));
            }
            other => {
                return Err(perr(
                    line,
                    col,
                    format!("axis: unknown mode '{other}' (equal/tight/on/off)."),
                    format!("axis: حالت ناشناخته «{other}» (equal/tight/on/off)."),
                ));
            }
        },
        v => {
            let r = numeric_vec(v, "axis", line, col)?;
            if r.len() == 4 && r.iter().all(|x| x.is_finite()) && r[0] < r[1] && r[2] < r[3] {
                session::emit(PlotOp::AxisLimits {
                    min_x: r[0],
                    max_x: r[1],
                    min_y: r[2],
                    max_y: r[3],
                });
            } else {
                return Err(perr(
                    line,
                    col,
                    "axis: limits must be [xmin xmax ymin ymax] with xmin<xmax, ymin<ymax.".into(),
                    "axis: حدود باید [xmin xmax ymin ymax] با ترتیب صعودی باشد.".into(),
                ));
            }
        }
    }
    Ok(())
}

fn on_off(args: &[Value], name: &str, line: u32, col: u32) -> Result<bool, McError> {
    if args.len() != 1 {
        return Err(perr(
            line,
            col,
            format!("{name}: exactly one argument expected ('on'/'off')."),
            format!("{name}: دقیقاً یک آرگومان لازم است ('on'/'off')."),
        ));
    }
    match char_of(&args[0], name, line, col)?.as_str() {
        "on" => Ok(true),
        "off" => Ok(false),
        other => Err(perr(
            line,
            col,
            format!("{name}: expected 'on' or 'off', got '{other}'."),
            format!("{name}: 'on' یا 'off' لازم است، نه «{other}»."),
        )),
    }
}

fn bi_grid(args: &[Value], line: u32, col: u32) -> Result<(), McError> {
    let on = on_off(args, "grid", line, col)?;
    session::emit(PlotOp::Grid { on });
    Ok(())
}

fn bi_hold(args: &[Value], line: u32, col: u32) -> Result<(), McError> {
    // بدون آرگومان = toggle نیست (MATLAB-lite): on/off صریح لازم است
    let on = on_off(args, "hold", line, col)?;
    session::emit(PlotOp::Hold { on });
    Ok(())
}

fn one_label(args: &[Value], name: &str, line: u32, col: u32) -> Result<String, McError> {
    if args.len() != 1 {
        return Err(perr(
            line,
            col,
            format!("{name}: exactly one string argument expected."),
            format!("{name}: دقیقاً یک آرگومان رشته‌ای لازم است."),
        ));
    }
    char_of(&args[0], name, line, col)
}

fn bi_xlabel(args: &[Value], line: u32, col: u32) -> Result<(), McError> {
    let text = one_label(args, "xlabel", line, col)?;
    session::emit(PlotOp::XLabel { text });
    Ok(())
}

fn bi_ylabel(args: &[Value], line: u32, col: u32) -> Result<(), McError> {
    let text = one_label(args, "ylabel", line, col)?;
    session::emit(PlotOp::YLabel { text });
    Ok(())
}

fn bi_title(args: &[Value], line: u32, col: u32) -> Result<(), McError> {
    let text = one_label(args, "title", line, col)?;
    session::emit(PlotOp::Title { text });
    Ok(())
}

fn bi_figure(args: &[Value], line: u32, col: u32) -> Result<(), McError> {
    let id = match args.len() {
        0 => None,
        1 => {
            let n = scalar_of(&args[0], "figure", line, col)?;
            if n.fract() != 0.0 || !(1.0..=64.0).contains(&n) {
                return Err(perr(
                    line,
                    col,
                    "figure: identifier must be an integer in 1..=64.".into(),
                    "figure: شناسه باید عدد صحیح 1..=64 باشد.".into(),
                ));
            }
            Some(n as u64)
        }
        _ => {
            return Err(perr(
                line,
                col,
                "figure: at most one argument expected (figure or figure(n)).".into(),
                "figure: حداکثر یک آرگومان (figure یا figure(n)).".into(),
            ));
        }
    };
    session::emit(PlotOp::ActivateFigure { id });
    Ok(())
}
