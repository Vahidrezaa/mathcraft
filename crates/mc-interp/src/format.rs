//! نمایش اعداد و ماتریس‌ها به سبک متلب — موتور `format` (سند ۰۳ §11.4).
//!
//! لایهٔ نمایش FB-1 (P2 — بازخورد دستگاه کاربر 2026-10-21):
//! - انتساب: `{name} =` در یک سطر، بدنه از سطر بعد (رفتار متلب).
//! - بدنهٔ ماتریس: هر ردیف ماتریس یک سطر؛ ستون‌ها راست‌چین با عرض مشترک.
//! - عرض ستون = `max(6, بلندترین عضو قالب‌بندی‌شده + 4)` — کالیبره‌شده روی
//!   خروجی واقعی متلبِ کاربر (`     1` پنج فاصله + رقم) و مثال سند متلب
//!   (`    0.3333    0.1667` چهار فاصله + ۶ نویسه = میدان ۱۰تایی).
//! - بین خروجی‌های متوالی یک سطر خالی (قرارداد پنجرهٔ فرمان متلب) — بلوک
//!   با `\n` پایانی برمی‌گردد و renderer ها هر بلوک را جدا چاپ می‌کنند.
//! - مختلط: `1.0000 + 2.0000i` — اجزای ۴ اعشار (رفتار متلب برای مختلط).
//!
//! ROADMAP 2.4 (آیتم ۱۱ ممیزی): قاعدهٔ متلبِ «۴ اعشار» جایگزین «۵ رقم
//! معنادار با حذف صفر» شد — `0.1` دیگر `0.1` نیست، `0.1000` است؛ پنجرهٔ
//! fixed `1e-3 ≤ |x| < 1e3`، بیرون آن نماد علمی ۴ اعشار؛ ماتریس با عضو
//! بزرگ/ریز یک «ضریب مقیاس مشترک» `1.0e+03 *` می‌گیرد (فقط short).
//! چیدمان FB-1 دست‌نخورده ماند — فقط رشتهٔ هر عضو عوض می‌شود.

use mc_core::{ComplexMatrix, Matrix, Value};

/// حالت نمایش عددی — `format` command (ROADMAP 2.4 — آیتم ۱۱ ممیزی).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NumFormat {
    /// `format short` — پیش‌فرض: fixed ۴ اعشار در پنجره، بیرون: نماد علمی.
    #[default]
    Short,
    /// `format long` — ۱۵ رقم معنادار (اسکلت — سند ۰۳ §11.4).
    Long,
    /// `format short e` — همیشه نماد علمی ۴ اعشار.
    ShortE,
    /// `format long e` — همیشه نماد علمی ۱۵ رقم.
    LongE,
    /// `format short g` — ۵ رقم، بدون صفر انتهایی، بدون scale (اسکلت).
    ShortG,
    /// `format long g` — ۱۵ رقم، بدون صفر انتهایی (اسکلت).
    LongG,
}

/// مشخصهٔ نمایش کنسول. `compact`/`loose` پذیرفته می‌شوند ولی فاصله‌گذاری
/// بین بلوک‌ها در سطح renderer/UI است (اسکلت مستند — سند ۰۳ §11.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DisplaySpec {
    pub num: NumFormat,
}

/// حداقل عرض ستون و حاشیهٔ ستونی (قواعد چیدمان متلب — کالیبره روی خروجی کاربر).
const COLUMN_MIN_WIDTH: usize = 6;
const COLUMN_PAD: usize = 4;

/// توان ۱۰ِ کف — با اصلاح خطای گردشدن log10 (مثل 1000 → 2.999…).
fn exp10_floor(a: f64) -> i32 {
    let mut e = a.log10().floor() as i32;
    if a < 10f64.powi(e) {
        e -= 1;
    } else if a >= 10f64.powi(e + 1) {
        e += 1;
    }
    e
}

/// نماد علمی متلب با `decimals` اعشار — `1.2345e+03` (توان حداقل ۲ رقم).
fn fmt_exp(x: f64, decimals: usize) -> String {
    if x.is_nan() {
        return "NaN".to_owned();
    }
    if x.is_infinite() {
        return if x > 0.0 { "Inf" } else { "-Inf" }.to_owned();
    }
    let sci = format!("{x:.decimals$e}");
    let (mantissa, exponent) = sci.split_once('e').unwrap_or((sci.as_str(), "0"));
    let exp_val: i32 = exponent.parse().unwrap_or(0);
    format!(
        "{mantissa}e{}{:02}",
        if exp_val < 0 { '-' } else { '+' },
        exp_val.abs()
    )
}

/// `format short` برای یک عضو — قاعدهٔ متلبِ ROADMAP 2.4:
/// پنجرهٔ fixed `1e-3 ≤ |x| < 1e3` → دقیقاً ۴ اعشار (`0.1000`، `123.4560`)،
/// مگر بعد از گردشدن صحیح شود (`999.99996` → `1000`)؛ بیرون پنجره →
/// نماد علمی ۴ اعشار (`1.2345e+03`). صفرهای انتهایی حذف نمی‌شوند.
pub fn format_short(x: f64) -> String {
    if x.is_nan() {
        return "NaN".to_owned();
    }
    if x.is_infinite() {
        return if x > 0.0 { "Inf" } else { "-Inf" }.to_owned();
    }
    if x == 0.0 {
        return "0".to_owned();
    }
    let e = exp10_floor(x.abs());
    if (-3..=2).contains(&e) {
        let s = format!("{x:.4}");
        // صحیحِ پس از گردشدن بدون اعشار چاپ می‌شود (رفتار متلب: `>> 2` → `2`)
        match s.strip_suffix(".0000") {
            Some(int_part) => int_part.to_owned(),
            None => s,
        }
    } else {
        fmt_exp(x, 4)
    }
}

/// `format long` — ۱۵ رقم معنادار (اسکلت ROADMAP 2.4): پنجرهٔ همان short،
/// اعشار = ۱۴−توان (`pi` → `3.14159265358979`، `0.1` → `0.100000000000000`).
fn fmt_fixed_sig(x: f64, sig: usize) -> String {
    if x.is_nan() {
        return "NaN".to_owned();
    }
    if x.is_infinite() {
        return if x > 0.0 { "Inf" } else { "-Inf" }.to_owned();
    }
    if x == 0.0 {
        return "0".to_owned();
    }
    let e = exp10_floor(x.abs());
    if (-3..=2).contains(&e) {
        let dec = (sig as i32 - 1 - e).max(0) as usize;
        let s = format!("{x:.dec$}");
        // فقط «صحیحِ پس از گردشدن» بدون اعشار — صفرهای معنادارِ long حفظ
        // می‌شوند (0.1 → 0.100000000000000، رفتار متلب)
        if let Some(dot) = s.find('.') {
            if s[dot + 1..].bytes().all(|b| b == b'0') {
                return s[..dot].to_owned();
            }
        }
        s
    } else {
        fmt_exp(x, sig - 1)
    }
}

/// `format short g` / `long g` — قاعدهٔ %g: fixed اگر `-4 ≤ e < sig` بود،
/// وگرنه نماد علمی؛ همیشه بدون صفرهای انتهایی؛ بدون ضریب مقیاس (اسکلت).
fn fmt_g(x: f64, sig: usize) -> String {
    if x.is_nan() {
        return "NaN".to_owned();
    }
    if x.is_infinite() {
        return if x > 0.0 { "Inf" } else { "-Inf" }.to_owned();
    }
    if x == 0.0 {
        return "0".to_owned();
    }
    let e = exp10_floor(x.abs());
    if (-4..sig as i32).contains(&e) {
        let dec = (sig as i32 - 1 - e).max(0) as usize;
        let s = format!("{x:.dec$}");
        s.trim_end_matches('0').trim_end_matches('.').to_owned()
    } else {
        let sci = format!("{x:.*e}", sig - 1);
        let (m, ev) = sci.split_once('e').unwrap_or((sci.as_str(), "0"));
        let ev: i32 = ev.parse().unwrap_or(0);
        let m = m.trim_end_matches('0').trim_end_matches('.');
        format!("{m}e{}{:02}", if ev < 0 { '-' } else { '+' }, ev.abs())
    }
}

/// قالب یک عضو عددی بر اساس حالت فعال.
pub fn format_elem(x: f64, mode: NumFormat) -> String {
    match mode {
        NumFormat::Short => format_short(x),
        NumFormat::Long => fmt_fixed_sig(x, 15),
        NumFormat::ShortE => fmt_exp(x, 4),
        NumFormat::LongE => fmt_exp(x, 15),
        NumFormat::ShortG => fmt_g(x, 5),
        NumFormat::LongG => fmt_g(x, 15),
    }
}

/// قالب عضو `num2str` — متلب: ۴ اعشار در پنجرهٔ گسترده‌تر از short
/// (|x| < 1e4)، همیشه بدون صفر انتهایی (اعشار و مانتیس). رفرنس‌های رفتاری:
/// num2str(pi) → '3.1416'، num2str(0.1) → '0.1'، num2str(1000) → '1000'،
/// num2str(1e10) → '1e+10'. (ریزقاعده‌ها برای پین با MATLAB واقعی علامت
/// خورده‌اند — COMPATIBILITY §4.8 ردیف آیتم ۸.)
pub fn format_num2str_elem(x: f64) -> String {
    if x.is_nan() {
        return "NaN".to_owned();
    }
    if x.is_infinite() {
        return if x > 0.0 { "Inf" } else { "-Inf" }.to_owned();
    }
    if x == 0.0 {
        return "0".to_owned();
    }
    let e = exp10_floor(x.abs());
    if (-3..=3).contains(&e) {
        format!("{x:.4}")
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_owned()
    } else {
        match fmt_exp(x, 4).split_once('e') {
            Some((m, tail)) => format!("{}e{tail}", m.trim_end_matches('0').trim_end_matches('.')),
            None => String::new(),
        }
    }
}

/// جزء مختلط — مثل متلب برای مختلط‌ها همیشه ۴ اعشار در بازهٔ fixed
/// (`1.0000`) و بیرون بازه نماد علمی (`1.2346e+05`).
/// ثابت‌های این مسیر عمداً مثل قبل مانده‌اند — مختلط‌ها از تغییرات موتور
/// format 2.4 مستثنی‌اند (چیدمان FB مستند؛ ROADMAP 2.4 فقط حقیقی‌ها).
const COMPLEX_SIG_DIGITS: i32 = 5;
const COMPLEX_FIXED_MIN_EXP: i32 = -3;
const COMPLEX_FIXED_MAX_EXP: i32 = 4;

fn format_short_complex_part(x: f64) -> String {
    if x.is_nan() {
        return "NaN".to_owned();
    }
    if x.is_infinite() {
        return if x > 0.0 { "Inf" } else { "-Inf" }.to_owned();
    }
    if x == 0.0 {
        return "0.0000".to_owned();
    }
    let exp = x.abs().log10().floor() as i32;
    if (COMPLEX_FIXED_MIN_EXP..=COMPLEX_FIXED_MAX_EXP).contains(&exp) {
        let decimals = (COMPLEX_SIG_DIGITS - 1 - exp).max(0) as usize;
        let decimals = decimals.max(4);
        format!("{x:.decimals$}")
    } else {
        let sci = format!("{x:.width$e}", width = (COMPLEX_SIG_DIGITS - 1) as usize);
        let (mantissa, exponent) = sci.split_once('e').unwrap_or((sci.as_str(), "0"));
        let exp_val: i32 = exponent.parse().unwrap_or(0);
        format!(
            "{mantissa}e{}{:02}",
            if exp_val < 0 { '-' } else { '+' },
            exp_val.abs()
        )
    }
}

/// قالب‌بندی یک عنصر مختلط — `1.0000 + 2.0000i`.
fn format_complex(re: f64, im: f64) -> String {
    let re_s = format_short_complex_part(re);
    let im_s = format_short_complex_part(im.abs());
    let sign = if im < 0.0 || (im == 0.0 && im.is_sign_negative()) {
        '-'
    } else {
        '+'
    };
    format!("{re_s} {sign} {im_s}i")
}

/// بدنهٔ نمایش یک مقدار — بدون پیشوند `name =`؛ ماتریس‌ها چندسطری‌اند.
/// برای بلوک‌های نمایشی متلب با `\n` پایانی برمی‌گردد (سطر خالی بین خروجی‌ها).
/// نسخهٔ بدون spec = حالت پیش‌فرض (short) — سازگاری با فراخوان‌های قدیمی.
pub fn format_value(v: &Value) -> String {
    format_value_spec(v, &DisplaySpec::default())
}

pub fn format_value_spec(v: &Value, spec: &DisplaySpec) -> String {
    let body = match v {
        Value::Double(m) => format_matrix(m, spec),
        Value::Complex(z) => format_complex_matrix(z, spec),
        Value::Char(s) => format!("'{s}'"),
        Value::Anon(_) => "function_handle".to_owned(),
    };
    format!("{body}\n")
}

/// متن کامل انتساب — `x =\n<body>\n`.
pub fn format_assignment(name: &str, v: &Value) -> String {
    format_assignment_spec(name, v, &DisplaySpec::default())
}

pub fn format_assignment_spec(name: &str, v: &Value, spec: &DisplaySpec) -> String {
    match v {
        Value::Anon(_) => format!("{name} = function_handle\n"),
        _ => format!("{name} =\n{}", format_value_spec(v, spec)),
    }
}

/// متن کامل `ans` — `ans =\n<body>\n`.
pub fn format_ans(v: &Value) -> String {
    format_ans_spec(v, &DisplaySpec::default())
}

pub fn format_ans_spec(v: &Value, spec: &DisplaySpec) -> String {
    match v {
        Value::Anon(_) => "ans = function_handle\n".to_owned(),
        _ => format!("ans =\n{}", format_value_spec(v, spec)),
    }
}

/// متن نمایش ماتریس خالی به سبک متلب (ROADMAP 2.4 — آیتم ۱۰ ممیزی):
/// 0×0 → `[]`؛ 1×0 → `1×0 empty double row vector`؛ 0×1 → ستونی؛
/// بقیه → `m×n empty double array`. تورفتگی ۵ فاصله هم‌راستا با بدنهٔ
/// اعداد FB-1 — برای پین‌شدن با خروجی واقعی متلب علامت‌گذاری شده
/// (COMPATIBILITY §4.8 — ردیف آیتم ۱۰).
fn empty_display_text(rows: usize, cols: usize) -> String {
    if rows == 0 && cols == 0 {
        return "[]".to_owned();
    }
    let kind = match (rows, cols) {
        (1, 0) => "row vector",
        (0, 1) => "column vector",
        _ => "array",
    };
    format!("     {rows}×{cols} empty double {kind}")
}

/// چیدمان ماتریس حقیقی — راست‌چین ستونی با عرض مشترک.
/// ROADMAP 2.4 (آیتم ۱۱): در حالت short اگر بزرگ‌ترین عضو ناصفر ≥ 1e3 یا
/// کل اعضای ناصفر < 1e3 باشند، ضریب مقیاس مشترک متلب (`1.0e+03 *`) چاپ
/// می‌شود و اعضا بر اثر تقسیم با ۴ اعشار اجباری نمایش داده می‌شوند.
fn format_matrix(m: &Matrix, spec: &DisplaySpec) -> String {
    if m.is_empty() {
        return empty_display_text(m.rows(), m.cols());
    }
    if m.is_scalar() {
        return pad_column(&[format_elem(m.get(0, 0), spec.num)]);
    }
    // ضریب مقیاس مشترک — فقط در short (متلب: در e/g اعمال نمی‌شود)
    let scaled: Option<(f64, String)> = if spec.num == NumFormat::Short {
        scale_factor_of(m).map(|(pow10, header)| (pow10, format!("   {header} *")))
    } else {
        None
    };
    let cell_of = |r: usize, c: usize| match &scaled {
        Some((p, _)) => format!("{:.4}", m.get(r, c) / p),
        None => format_elem(m.get(r, c), spec.num),
    };
    // عرض هر ستون از بلندترین عضو قالب‌بندی‌شده
    let mut widths = vec![COLUMN_MIN_WIDTH; m.cols()];
    for (c, w_c) in widths.iter_mut().enumerate() {
        let mut w = 0usize;
        for r in 0..m.rows() {
            w = w.max(cell_of(r, c).chars().count());
        }
        *w_c = (*w_c).max(w + COLUMN_PAD);
    }
    let mut lines = Vec::with_capacity(m.rows() + 1);
    if let Some((_, header)) = &scaled {
        lines.push(header.clone());
    }
    for r in 0..m.rows() {
        let mut line = String::new();
        for (c, w_c) in widths.iter().enumerate() {
            let cell = cell_of(r, c);
            let pad = w_c.saturating_sub(cell.chars().count());
            line.push_str(&" ".repeat(pad));
            line.push_str(&cell);
        }
        lines.push(line);
    }
    lines.join("\n")
}

/// ضریب مقیاس متلب: اگر بزرگ‌ترین |عضو ناصفر| ≥ 1e3 یا کل اعضای ناصفر
/// < 1e3 باشند → (10^k، متن `1.0e+03`). NaN/Inf و صفر در محاسبه شرکت نمی‌کنند.
/// حد آستانه و متن سربرگ برای پین‌شدن با خروجی واقعی متلب کاربر علامت‌گذاری شده
/// (COMPATIBILITY §4.8 — ردیف آیتم ۱۱).
fn scale_factor_of(m: &Matrix) -> Option<(f64, String)> {
    let mut maxabs = 0f64;
    for i in 0..m.numel() {
        let x = m.get_linear(i);
        if x.is_finite() {
            maxabs = maxabs.max(x.abs());
        }
    }
    if maxabs == 0.0 || (1e-3..1e3).contains(&maxabs) {
        return None;
    }
    let e = exp10_floor(maxabs);
    let header = format!("1.0e{}{:02}", if e < 0 { '-' } else { '+' }, e.abs());
    Some((10f64.powi(e), header))
}

/// چیدمان ماتریس مختلط — ستون‌ها راست‌چین؛ اجزای ۴ اعشار.
/// مختلط‌ها در گام 2.4 از موتور format مستثنی‌اند (چیدمان FB مستند) —
/// spec پذیرفته می‌شود ولی نادیده گرفته می‌شود.
fn format_complex_matrix(z: &ComplexMatrix, _spec: &DisplaySpec) -> String {
    if z.is_empty() {
        return empty_display_text(z.rows(), z.cols());
    }
    if z.is_scalar() {
        let (re, im) = z.get(0, 0);
        return pad_column(&[format_complex(re, im)]);
    }
    let mut widths = vec![COLUMN_MIN_WIDTH; z.cols()];
    for (c, w_c) in widths.iter_mut().enumerate() {
        let mut w = 0usize;
        for r in 0..z.rows() {
            let (re, im) = z.get(r, c);
            w = w.max(format_complex(re, im).chars().count());
        }
        *w_c = (*w_c).max(w + COLUMN_PAD);
    }
    let mut lines = Vec::with_capacity(z.rows());
    for r in 0..z.rows() {
        let mut line = String::new();
        for (c, w_c) in widths.iter().enumerate() {
            let (re, im) = z.get(r, c);
            let cell = format_complex(re, im);
            let pad = w_c.saturating_sub(cell.chars().count());
            line.push_str(&" ".repeat(pad));
            line.push_str(&cell);
        }
        lines.push(line);
    }
    lines.join("\n")
}

/// یک ستون تکی — راست‌چین با حداقل عرض (اسکالر/ستون‌بردار).
fn pad_column(cells: &[String]) -> String {
    let w = cells
        .iter()
        .map(|c| c.chars().count())
        .max()
        .unwrap_or(0)
        .max(COLUMN_MIN_WIDTH - COLUMN_PAD)
        + COLUMN_PAD;
    cells
        .iter()
        .map(|c| {
            let pad = w.saturating_sub(c.chars().count());
            format!("{}{}", " ".repeat(pad), c)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integers_and_zero() {
        assert_eq!(format_short(2.0), "2");
        assert_eq!(format_short(-17.0), "-17");
        assert_eq!(format_short(0.0), "0");
        assert_eq!(format_short(-0.0), "0");
    }

    #[test]
    fn special_values() {
        assert_eq!(format_short(f64::NAN), "NaN");
        assert_eq!(format_short(f64::INFINITY), "Inf");
        assert_eq!(format_short(f64::NEG_INFINITY), "-Inf");
    }

    #[test]
    fn matlab_four_decimal_rule() {
        // ROADMAP 2.4 — آیتم ۱۱: قاعدهٔ ۴ اعشار متلب، بدون حذف صفر انتهایی
        // (قبلاً «۵ رقم معنادار با حذف صفر» بود — تغییر عمدی سند ۰۳ §11.4)
        assert_eq!(format_short(0.1), "0.1000");
        assert_eq!(format_short(1.0 / 3.0), "0.3333");
        assert_eq!(format_short(123.456), "123.4560");
        assert_eq!(format_short(1.5), "1.5000");
        assert_eq!(format_short(-0.5), "-0.5000");
        assert_eq!(format_short(0.001), "0.0010");
        assert_eq!(format_short(999.5), "999.5000");
        // صحیحِ پس از گردشدن بدون اعشار چاپ می‌شود
        assert_eq!(format_short(0.99996), "1");
        assert_eq!(format_short(999.99996), "1000");
    }

    #[test]
    fn window_edges_scientific() {
        // مرز پنجرهٔ fixed: |x| ≥ 1e3 یا |x| < 1e-3 → نماد علمی ۴ اعشار
        assert_eq!(format_short(1234.5), "1.2345e+03");
        assert_eq!(format_short(1.0e10), "1.0000e+10");
        assert_eq!(format_short(0.0005), "5.0000e-04");
        assert_eq!(format_short(1.23456e-7), "1.2346e-07");
        assert_eq!(format_short(1.7976931348623157e308), "1.7977e+308");
    }

    #[test]
    fn common_scale_factor_matlab_style() {
        // [1000 2000] → سربرگ مقیاس + اجزای تقسیم‌شده با ۴ اعشار اجباری
        let m = Matrix::from_columns(1, 2, vec![1000.0, 2000.0]);
        assert_eq!(
            format_matrix(&m, &DisplaySpec::default()),
            "   1.0e+03 *\n    1.0000    2.0000"
        );
        // ریزها: [1e-5 2e-5] → 1.0e-05 *
        let tiny = Matrix::from_columns(1, 2, vec![1e-5, 2e-5]);
        assert_eq!(
            format_matrix(&tiny, &DisplaySpec::default()),
            "   1.0e-05 *\n    1.0000    2.0000"
        );
        // بزرگ‌ترین عضو داخل پنجره است → بدون scale (هر عضو با قاعدهٔ خودش)
        let mixed = Matrix::from_columns(1, 2, vec![0.5, 1e-5]);
        let out = format_matrix(&mixed, &DisplaySpec::default());
        assert!(
            !out.contains('*'),
            "mixed window matrix must not scale: {out}"
        );
    }

    #[test]
    fn display_modes_long_and_e_and_g() {
        // اسکلت ROADMAP 2.4 — آیتم ۱۱: long/short e/long e/short g/long g
        let pi = std::f64::consts::PI;
        assert_eq!(format_elem(pi, NumFormat::Long), "3.14159265358979");
        assert_eq!(format_elem(0.1, NumFormat::Long), "0.100000000000000");
        assert_eq!(format_elem(pi, NumFormat::ShortE), "3.1416e+00");
        assert_eq!(format_elem(pi, NumFormat::LongE), "3.141592653589793e+00");
        assert_eq!(format_elem(0.1, NumFormat::ShortG), "0.1");
        assert_eq!(format_elem(1.0e10, NumFormat::ShortG), "1e+10");
        assert_eq!(
            format_elem(1.0 / 3.0, NumFormat::LongG),
            "0.333333333333333"
        );
    }

    #[test]
    fn scientific_style_like_matlab() {
        assert_eq!(format_short(1.0e5), "1.0000e+05");
        assert_eq!(format_short(123456.0), "1.2346e+05");
        assert_eq!(format_short(0.0001), "1.0000e-04");
    }

    #[test]
    fn assignment_block_matlab_layout() {
        // خروجی واقعی متلبِ کاربر (۵×۵ — دستگاه 2026-10-21):
        // x =\n     1\n     2\n     3\n     4\n     5\n
        let x = Matrix::from_columns(5, 1, vec![1.0, 2.0, 3.0, 4.0, 5.0]);
        assert_eq!(
            format_assignment("x", &Value::Double(x)),
            "x =\n     1\n     2\n     3\n     4\n     5\n"
        );
        // residual =\n     0\n
        let res = Matrix::scalar(0.0);
        assert_eq!(
            format_assignment("residual", &Value::Double(res)),
            "residual =\n     0\n"
        );
    }

    #[test]
    fn matrix_two_columns_aligned() {
        // [1 2; 3 4] → هر ستون عرض 6: "     1     2"
        let m = Matrix::from_columns(2, 2, vec![1.0, 3.0, 2.0, 4.0]);
        assert_eq!(
            format_matrix(&m, &DisplaySpec::default()),
            "     1     2\n     3     4"
        );
    }

    #[test]
    fn decimal_columns_wide_fields() {
        // قاعدهٔ ۴ اعشار جدید — همهٔ اعضا ۶ نویسه‌ای → میدان ۱۰تایی
        let m = Matrix::from_columns(2, 2, vec![1.0 / 3.0, 1.0 / 7.0, 1.0 / 6.0, 1.0 / 8.0]);
        let out = format_matrix(&m, &DisplaySpec::default());
        let rows: Vec<&str> = out.split('\n').collect();
        assert_eq!(rows[0], "    0.3333    0.1667");
        assert_eq!(rows[1], "    0.1429    0.1250");
    }

    #[test]
    fn column_right_justifies_mixed_widths() {
        // [1; 100] → عرض ستون max(6, 3+4)=7: "      1" / "    100"
        let m = Matrix::from_columns(2, 1, vec![1.0, 100.0]);
        assert_eq!(
            format_matrix(&m, &DisplaySpec::default()),
            "      1\n    100"
        );
    }

    #[test]
    fn complex_display_matlab_style() {
        let z = ComplexMatrix::scalar(1.0, 2.0);
        assert_eq!(format_value(&Value::Complex(z)), "    1.0000 + 2.0000i\n");
        let z2 = ComplexMatrix::scalar(0.0, -1.0);
        assert_eq!(format_value(&Value::Complex(z2)), "    0.0000 - 1.0000i\n");
    }

    #[test]
    fn ans_block() {
        assert_eq!(
            format_ans(&Value::Double(Matrix::scalar(2.0))),
            "ans =\n     2\n"
        );
    }

    #[test]
    fn empty_and_char_and_anon() {
        // 0×0 مثل قبل [] (بند ۱۰: بقیهٔ شکل‌ها متن متلب می‌گیرند)
        assert_eq!(
            format_value(&Value::Double(Matrix::full(0, 0, 0.0))),
            "[]\n"
        );
        assert_eq!(format_value(&Value::Char("abc".to_owned())), "'abc'\n");
        // anon از format_ans/assignment مسیر جدا دارد
        assert_eq!(
            format_assignment("f", &Value::Char("x".into())),
            "f =\n'x'\n"
        );
    }

    #[test]
    fn empty_shape_display_matlab_style() {
        // ROADMAP 2.4 — آیتم ۱۰: متن خالی‌های متلب
        assert_eq!(
            format_value(&Value::Double(Matrix::full(1, 0, 0.0))),
            "     1×0 empty double row vector\n"
        );
        assert_eq!(
            format_value(&Value::Double(Matrix::full(0, 1, 0.0))),
            "     0×1 empty double column vector\n"
        );
        assert_eq!(
            format_value(&Value::Double(Matrix::full(3, 0, 0.0))),
            "     3×0 empty double array\n"
        );
        assert_eq!(
            format_value(&Value::Double(Matrix::full(0, 4, 0.0))),
            "     0×4 empty double array\n"
        );
    }
}
