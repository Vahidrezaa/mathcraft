//! تیک‌های «زیبا» و قالب اعداد — سند ۰۴ §8.1: الگوریتم 1-2-5 در Rust تا
//! Dart فقط بکشد؛ فرمت نمایش `%.4g` با حذف صفرهای بی‌معنا (پارتی C).

/// حداکثر تعداد تیک اصلی هر محور (رفتار متلب ~۶–۸).
pub const MAX_TICKS: usize = 8;

/// تیک انتخاب‌شده — `label: None` یعنی تیک فرعی (minor) بدون برچسب.
#[derive(Debug, Clone, PartialEq)]
pub struct Tick {
    pub position: f64,
    pub label: Option<String>,
}

/// گام «زیبا» از خانوادهٔ 1-2-5: کوچک‌ترین 10^k×{1,2,5,10} که ≥ آرمان است.
pub fn nice_step(ideal: f64) -> f64 {
    if ideal <= 0.0 || !ideal.is_finite() {
        return 1.0;
    }
    let mag = 10f64.powf(ideal.log10().floor());
    for mult in [1.0, 2.0, 5.0, 10.0] {
        if mult * mag >= ideal - mag * 1e-9 {
            return mult * mag;
        }
    }
    10.0 * mag
}

/// تیک‌های خطی 1-2-5 در بازهٔ [min, max] — خروجی مرتب و بدون تکرار.
pub fn linear_ticks(min: f64, max: f64, max_ticks: usize) -> Vec<f64> {
    if !min.is_finite() || !max.is_finite() || max <= min {
        return vec![min];
    }
    let step = nice_step((max - min) / max_ticks.max(2) as f64);
    let first = (min / step).ceil();
    let last = (max / step).floor();
    let mut out = Vec::new();
    let mut k = first;
    while k <= last + 1e-9 {
        let v = k * step;
        if v.is_finite() {
            out.push(v);
        }
        k += 1.0;
    }
    out.dedup_by(|a, b| (*a - *b).abs() < step * 1e-9);
    out
}

/// تیک‌های لگاریتمی: اصلی‌ها توان‌های ۱۰ با برچسب `10^{k}` (TeX سبک برای
/// رندر superscript در Dart)؛ اگر کمتر از ۲ اصلی در بازه باشد، تیک‌های
/// فرعی ۲..۹× همان دهه اضافه می‌شوند.
pub fn log_ticks(min: f64, max: f64) -> Vec<Tick> {
    if !min.is_finite() || !max.is_finite() || max <= min || min <= 0.0 {
        return Vec::new();
    }
    let k0 = min.log10().ceil() as i32;
    let k1 = max.log10().floor() as i32;
    let mut majors: Vec<i32> = (k0..=k1).collect();
    if majors.len() < 2 {
        majors.clear();
    }
    if !majors.is_empty() {
        return majors
            .into_iter()
            .map(|k| Tick {
                position: 10f64.powi(k),
                label: Some(format!("10^{{{k}}}")),
            })
            .collect();
    }
    // بازهٔ باریک‌تر از یک دهه — فرعی‌های ۲..۹ دهه‌های پوشش‌داده
    let mut out = Vec::new();
    let mut dec = min.log10().floor() as i32;
    while dec <= max.log10().ceil() as i32 {
        for m in 2..10 {
            let v = f64::from(m) * 10f64.powi(dec);
            if v >= min && v <= max {
                out.push(Tick {
                    position: v,
                    label: None,
                });
            }
        }
        dec += 1;
    }
    out
}

/// فرمت `%.4g` با پارتی C: ۴ رقم معنادار؛ نماد علمی فقط وقتی توان < -4 یا
/// ≥ 4؛ حذف صفرهای انتهایی و نقطهٔ یتیم.
pub fn format_g4(v: f64) -> String {
    if v == 0.0 {
        return "0".to_owned();
    }
    if !v.is_finite() {
        return if v.is_nan() {
            "NaN".to_owned()
        } else if v > 0.0 {
            "Inf".to_owned()
        } else {
            "-Inf".to_owned()
        };
    }
    let neg = v < 0.0;
    let a = v.abs();
    // توان را از قالب علمی می‌خوانیم (مقاوم به خطای floor(log10(0.99..)))
    let exp: i32 = format!("{a:.3e}")
        .split('e')
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let body = if !(-4..4).contains(&exp) {
        let m = a / 10f64.powi(exp);
        let mut ms = format!("{m:.3}");
        strip_trailing_zeros(&mut ms);
        let sign = if exp < 0 { '-' } else { '+' };
        format!("{ms}e{sign}{:02}", exp.abs())
    } else {
        let prec = (3 - exp).max(0) as usize;
        let mut fs = format!("{a:.prec$}");
        strip_trailing_zeros(&mut fs);
        fs
    };
    if neg {
        format!("-{body}")
    } else {
        body
    }
}

fn strip_trailing_zeros(s: &mut String) {
    if s.contains('.') {
        while s.ends_with('0') {
            s.pop();
        }
        if s.ends_with('.') {
            s.pop();
        }
    }
}

/// تیک‌های کامل یک محور: خطی یا لگاریتمی بسته به پرچم `log`، با برچسب g4.
pub fn axis_ticks(min: f64, max: f64, log: bool) -> Vec<Tick> {
    if log {
        return log_ticks(min, max);
    }
    linear_ticks(min, max, MAX_TICKS)
        .into_iter()
        .map(|p| Tick {
            position: p,
            label: Some(format_g4(p)),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nice_step_follows_1_2_5_ladder() {
        assert_eq!(nice_step(0.3), 0.5); // 0.3 → 0.5
        assert_eq!(nice_step(0.7), 1.0);
        assert_eq!(nice_step(1.1), 2.0);
        assert_eq!(nice_step(3.0), 5.0);
        assert_eq!(nice_step(6.0), 10.0);
        assert_eq!(nice_step(17.0), 20.0);
        // ورودی غیرمثبت → ۱ (دفاعی)
        assert_eq!(nice_step(0.0), 1.0);
        assert_eq!(nice_step(-3.0), 1.0);
    }

    #[test]
    fn linear_ticks_cover_unit_interval() {
        let t = linear_ticks(0.0, 1.0, 8);
        assert_eq!(t.first(), Some(&0.0));
        assert_eq!(t.last(), Some(&1.0));
        assert!(t.len() >= 5 && t.len() <= 9, "got {t:?}");
        assert!(t.windows(2).all(|w| w[1] > w[0]));
    }

    #[test]
    fn linear_ticks_zero_width_returns_single() {
        assert_eq!(linear_ticks(2.0, 2.0, 8), vec![2.0]);
        assert_eq!(linear_ticks(5.0, 1.0, 8), vec![5.0]);
    }

    #[test]
    fn format_g4_matches_c_printf_semantics() {
        assert_eq!(format_g4(0.0), "0");
        assert_eq!(format_g4(0.5), "0.5");
        assert_eq!(format_g4(0.001), "0.001");
        assert_eq!(format_g4(0.0001), "0.0001");
        assert_eq!(format_g4(0.00001), "1e-05");
        assert_eq!(format_g4(1000.0), "1000");
        assert_eq!(format_g4(1234.5), "1234"); // 4 رقم معنادار
        assert_eq!(format_g4(12345.0), "1.234e+04");
        assert_eq!(format_g4(100000.0), "1e+05");
        assert_eq!(format_g4(-2.5), "-2.5");
        assert_eq!(format_g4(-12345.0), "-1.234e+04");
        assert_eq!(format_g4(9999.0), "9999");
        assert_eq!(format_g4(f64::NAN), "NaN");
        assert_eq!(format_g4(f64::INFINITY), "Inf");
    }

    #[test]
    fn log_ticks_major_decades_with_tex_labels() {
        let t = log_ticks(0.01, 100.0);
        let labels: Vec<&str> = t.iter().filter_map(|x| x.label.as_deref()).collect();
        assert_eq!(labels, ["10^{-2}", "10^{-1}", "10^{0}", "10^{1}", "10^{2}"]);
        assert_eq!(t[0].position, 0.01);
        assert_eq!(t[4].position, 100.0);
    }

    #[test]
    fn log_ticks_narrow_range_falls_back_to_minors() {
        let t = log_ticks(2.0, 8.0);
        assert!(t.iter().all(|x| x.label.is_none()), "all minor: {t:?}");
        assert!(t.iter().any(|x| x.position == 3.0));
        assert!(t.iter().any(|x| x.position == 5.0));
        // بازهٔ نامعتبر → خالی
        assert!(log_ticks(-1.0, 10.0).is_empty());
    }

    #[test]
    fn axis_ticks_dispatches_by_log_flag() {
        let lin = axis_ticks(0.0, 10.0, false);
        assert!(lin.iter().all(|t| t.label.is_some()));
        assert_eq!(lin[0].label.as_deref(), Some("0"));
        let lg = axis_ticks(1.0, 1000.0, true);
        assert_eq!(lg.len(), 4); // 10^0..10^3
    }
}
