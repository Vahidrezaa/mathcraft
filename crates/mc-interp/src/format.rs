//! نمایش اعداد و ماتریس‌ها به سبک متلب — `format short` (سند ۰۳ §11.4).
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

use mc_core::{ComplexMatrix, Matrix, Value};

/// ۵ رقم معنادار (سند ۰۳ بخش ۱۱ بند ۴).
const SIG_DIGITS: i32 = 5;
const FIXED_MIN_EXP: i32 = -3;
const FIXED_MAX_EXP: i32 = 4;

/// حداقل عرض ستون و حاشیهٔ ستونی (قواعد چیدمان متلب — کالیبره روی خروجی کاربر).
const COLUMN_MIN_WIDTH: usize = 6;
const COLUMN_PAD: usize = 4;

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
    let exp = x.abs().log10().floor() as i32;
    if (FIXED_MIN_EXP..=FIXED_MAX_EXP).contains(&exp) {
        let decimals = (SIG_DIGITS - 1 - exp).max(0) as usize;
        let s = format!("{x:.decimals$}");
        let s = s.trim_end_matches('0').trim_end_matches('.');
        s.to_owned()
    } else {
        // نماد علمی MATLAB: 1.2346e+05
        let sci = format!("{x:.width$e}", width = (SIG_DIGITS - 1) as usize);
        let (mantissa, exponent) = sci.split_once('e').unwrap_or((sci.as_str(), "0"));
        let exp_val: i32 = exponent.parse().unwrap_or(0);
        format!(
            "{mantissa}e{}{:02}",
            if exp_val < 0 { '-' } else { '+' },
            exp_val.abs()
        )
    }
}

/// جزء مختلط — مثل متلب برای مختلط‌ها همیشه ۴ اعشار در بازهٔ fixed
/// (`1.0000`) و بیرون بازه نماد علمی (`1.2346e+05`).
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
    if (FIXED_MIN_EXP..=FIXED_MAX_EXP).contains(&exp) {
        let decimals = (SIG_DIGITS - 1 - exp).max(0) as usize;
        let decimals = decimals.max(4);
        format!("{x:.decimals$}")
    } else {
        let sci = format!("{x:.width$e}", width = (SIG_DIGITS - 1) as usize);
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
pub fn format_value(v: &Value) -> String {
    let body = match v {
        Value::Double(m) => format_matrix(m),
        Value::Complex(z) => format_complex_matrix(z),
        Value::Char(s) => format!("'{s}'"),
        Value::Anon(_) => "function_handle".to_owned(),
    };
    format!("{body}\n")
}

/// متن کامل انتساب — `x =\n<body>\n`.
pub fn format_assignment(name: &str, v: &Value) -> String {
    match v {
        Value::Anon(_) => format!("{name} = function_handle\n"),
        _ => format!("{name} =\n{}", format_value(v)),
    }
}

/// متن کامل `ans` — `ans =\n<body>\n`.
pub fn format_ans(v: &Value) -> String {
    match v {
        Value::Anon(_) => "ans = function_handle\n".to_owned(),
        _ => format!("ans =\n{}", format_value(v)),
    }
}

/// چیدمان ماتریس حقیقی — راست‌چین ستونی با عرض مشترک.
fn format_matrix(m: &Matrix) -> String {
    if m.is_empty() {
        return "[]".to_owned();
    }
    if m.is_scalar() {
        return pad_column(&[format_short(m.get(0, 0))]);
    }
    // عرض هر ستون از بلندترین عضو قالب‌بندی‌شده
    let mut widths = vec![COLUMN_MIN_WIDTH; m.cols()];
    for (c, w_c) in widths.iter_mut().enumerate() {
        let mut w = 0usize;
        for r in 0..m.rows() {
            w = w.max(format_short(m.get(r, c)).chars().count());
        }
        *w_c = (*w_c).max(w + COLUMN_PAD);
    }
    let mut lines = Vec::with_capacity(m.rows());
    for r in 0..m.rows() {
        let mut line = String::new();
        for (c, w_c) in widths.iter().enumerate() {
            let cell = format_short(m.get(r, c));
            let pad = w_c.saturating_sub(cell.chars().count());
            line.push_str(&" ".repeat(pad));
            line.push_str(&cell);
        }
        lines.push(line);
    }
    lines.join("\n")
}

/// چیدمان ماتریس مختلط — ستون‌ها راست‌چین؛ اجزای ۴ اعشار.
fn format_complex_matrix(z: &ComplexMatrix) -> String {
    if z.is_empty() {
        return "[]".to_owned();
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
    fn five_significant_digits() {
        // قرارداد سند ۰۳ بخش ۱۱: ۵ رقم معنادار
        assert_eq!(format_short(1.0 / 3.0), "0.33333");
        assert_eq!(format_short(123.456), "123.46");
        assert_eq!(format_short(9.87654321), "9.8765");
        assert_eq!(format_short(0.001), "0.001");
    }

    #[test]
    fn scientific_style_like_matlab() {
        assert_eq!(format_short(1.0e5), "1.0000e+05");
        assert_eq!(format_short(123456.0), "1.2346e+05");
        assert_eq!(format_short(0.0001), "1.0000e-04");
        assert_eq!(format_short(1.23456e-7), "1.2346e-07");
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
        assert_eq!(format_matrix(&m), "     1     2\n     3     4");
    }

    #[test]
    fn decimal_columns_wide_fields() {
        // اجزای ۵رقمی → میدان ۱۱تایی (مثال سبک سند متلب با قاعدهٔ +4)
        let m = Matrix::from_columns(2, 2, vec![1.0 / 3.0, 1.0 / 7.0, 1.0 / 6.0, 1.0 / 8.0]);
        let out = format_matrix(&m);
        let rows: Vec<&str> = out.split('\n').collect();
        assert_eq!(rows[0], "    0.33333    0.16667");
        assert_eq!(rows[1], "    0.14286      0.125");
    }

    #[test]
    fn column_right_justifies_mixed_widths() {
        // [1; 100] → عرض ستون max(6, 3+4)=7: "      1" / "    100"
        let m = Matrix::from_columns(2, 1, vec![1.0, 100.0]);
        assert_eq!(format_matrix(&m), "      1\n    100");
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
}
