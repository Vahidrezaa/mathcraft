//! نمایش اعداد به سبک متلب — `format short` = ۵ رقم معنادار (سند ۰۳ بخش ۱۱ بند ۴).
//!
//! قواعد:
//! - صحیح‌ها بدون اعشار: `2`، `-17`
//! - NaN/Inf: `NaN`، `Inf`، `-Inf`
//! - صفر (شامل -0): `0` — انتخاب نمایش؛ محاسبات زیرین IEEE می‌مانند (سند ۰۳ بخش ۱۱ بند ۵)
//! - بازه ثابت: `1e-3 ≤ |x| < 1e5` با اعشار کافی برای ۵ رقم معنادار؛ بیرون بازه: نماد علمی MATLAB (`1.2346e+05`)

use mc_core::{Matrix, Value};

/// ۵ رقم معنادار (سند ۰۳ بخش ۱۱ بند ۴).
const SIG_DIGITS: i32 = 5;
const FIXED_MIN_EXP: i32 = -3;
const FIXED_MAX_EXP: i32 = 4;

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

/// نمایش یک مقدار برای کنسول:
/// - اسکالر → همان عدد
/// - ماتریس → لفظ متلب: `[1 2; 3 4]` (کارت جدولی کامل، نمای P3 است — سند ۰۶ بخش ۷.۱)
pub fn format_value(v: &Value) -> String {
    match v {
        Value::Double(m) => format_matrix(m),
        Value::Char(s) => format!("'{s}'"),
        Value::Anon(_) => "function_handle".to_owned(),
    }
}

fn format_matrix(m: &Matrix) -> String {
    if m.is_empty() {
        return "[]".to_owned();
    }
    if m.is_scalar() {
        return format_short(m.get(0, 0));
    }
    let mut rows: Vec<String> = Vec::with_capacity(m.rows());
    for r in 0..m.rows() {
        let cells: Vec<String> = (0..m.cols()).map(|c| format_short(m.get(r, c))).collect();
        rows.push(cells.join(" "));
    }
    format!("[{}]", rows.join("; "))
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
    fn matrix_literal_display() {
        let m = Matrix::from_columns(2, 2, vec![1.0, 3.0, 2.0, 4.0]);
        assert_eq!(format_value(&Value::Double(m)), "[1 2; 3 4]");
        assert_eq!(format_value(&Value::Double(Matrix::full(0, 0, 0.0))), "[]");
    }
}
