//! بسته‌بندی هیستوگرام — یال‌ها/شمارش‌ها/نرمال‌سازی (MATLAB-lite، سند ۰۳).
//!
//! پیش‌فرض تعداد بسته: ۱۰ (رفع `hist` کلاسیک متلب — سند COMPATIBILITY).
//! نرمال‌سازی: Count / Pdf / Cdf (پارتی `histogram(...,'Normalization',…)`).

/// نرمال‌سازی هیستوگرام — قرارداد v2.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HistNorm {
    /// شمارش خام (پیش‌فرض).
    #[default]
    Count,
    /// چگالی: c/(n·عرض بسته) — سطح زیر میله‌ها ۱.
    Pdf,
    /// توزیع تجمعی: cumsum(c)/n — سقف ۱.
    Cdf,
}

/// تعداد بستهٔ پیش‌فرض (MATLAB-lite — `hist` کلاسیک).
pub const DEFAULT_NBINS: usize = 10;

/// یال‌های یکنواخت روی [mn, mx] — بازهٔ صفرعرض به ±0.5 گسترش می‌خورد.
pub fn uniform_edges(mn: f64, mx: f64, nbins: usize) -> Vec<f64> {
    let nbins = nbins.max(1);
    let (mn, mx) = if mx <= mn || !mn.is_finite() || !mx.is_finite() {
        (mn - 0.5, mn + 0.5)
    } else {
        (mn, mx)
    };
    (0..=nbins)
        .map(|i| mn + (mx - mn) * i as f64 / nbins as f64)
        .collect()
}

/// شمارش نقطه‌ها در یال‌های نیمه‌باز [e_i, e_{i+1}) — آخرین بسته بسته (شامل
/// ماکسیمم؛ رفتار متلب: آخرین مقدار در آخرین بسته می‌افتد).
pub fn counts(data: &[f64], edges: &[f64]) -> Vec<f64> {
    if edges.len() < 2 {
        return Vec::new();
    }
    let mut out = vec![0.0; edges.len() - 1];
    for &v in data {
        if !v.is_finite() {
            continue;
        }
        if v < edges[0] || v > edges[edges.len() - 1] {
            continue;
        }
        if v == edges[edges.len() - 1] {
            let last = out.len() - 1;
            out[last] += 1.0;
            continue;
        }
        // جست‌وجای دودویی روی یال‌ها
        match edges.binary_search_by(|e| e.partial_cmp(&v).unwrap_or(std::cmp::Ordering::Equal)) {
            Ok(i) if i < out.len() => out[i] += 1.0,
            Ok(i) => {
                let _ = i; // دقیقاً روی یال آخر بالا گرفیم
            }
            Err(i) if i > 0 && i <= out.len() => out[i - 1] += 1.0,
            _ => {}
        }
    }
    out
}

/// اعمال نرمال‌سازی روی شمارش‌ها — Count: همان مقادیر.
pub fn apply_norm(counts: &[f64], edges: &[f64], norm: HistNorm) -> Vec<f64> {
    match norm {
        HistNorm::Count => counts.to_vec(),
        HistNorm::Pdf => counts
            .iter()
            .enumerate()
            .map(|(i, &c)| {
                let w = edges.get(i + 1).unwrap_or(&1.0) - edges.get(i).unwrap_or(&0.0);
                let total: f64 = counts.iter().sum();
                if w > 0.0 && total > 0.0 {
                    c / (total * w)
                } else {
                    0.0
                }
            })
            .collect(),
        HistNorm::Cdf => {
            let total: f64 = counts.iter().sum();
            let mut acc = 0.0;
            counts
                .iter()
                .map(|&c| {
                    acc += c;
                    if total > 0.0 {
                        acc / total
                    } else {
                        0.0
                    }
                })
                .collect()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uniform_edges_spans_exact_range() {
        let e = uniform_edges(0.0, 1.0, 10);
        assert_eq!(e.len(), 11);
        assert_eq!(e[0], 0.0);
        assert!((e[10] - 1.0).abs() < 1e-12);
    }

    #[test]
    fn degenerate_range_widens_by_half() {
        let e = uniform_edges(3.0, 3.0, 4);
        assert_eq!(e[0], 2.5);
        assert_eq!(e[4], 3.5);
    }

    #[test]
    fn counts_half_open_last_inclusive() {
        let edges = vec![0.0, 1.0, 2.0, 3.0];
        let c = counts(&[0.0, 0.5, 1.0, 2.9, 3.0], &edges);
        assert_eq!(c, vec![2.0, 1.0, 2.0]); // 1.0 در بستهٔ دوم، 3.0 در آخری
        assert_eq!(counts(&[-0.1, 3.1], &edges), vec![0.0, 0.0, 0.0]);
        assert_eq!(counts(&[f64::NAN], &edges), vec![0.0, 0.0, 0.0]);
    }

    #[test]
    fn classic_ten_bin_default() {
        let data: Vec<f64> = (0..100).map(|i| i as f64).collect();
        let e = uniform_edges(0.0, 99.0, DEFAULT_NBINS);
        let c = counts(&data, &e);
        let sum: f64 = c.iter().sum();
        assert_eq!(sum, 100.0);
    }

    #[test]
    fn norm_pdf_integrates_to_one() {
        let data: Vec<f64> = (0..1000).map(|i| (i as f64 * 0.01).sin()).collect();
        let e = uniform_edges(-1.0, 1.0, 20);
        let c = counts(&data, &e);
        let pdf = apply_norm(&c, &e, HistNorm::Pdf);
        let integral: f64 = pdf
            .iter()
            .enumerate()
            .map(|(i, &p)| p * (e[i + 1] - e[i]))
            .sum();
        assert!((integral - 1.0).abs() < 1e-9, "integral {integral}");
    }

    #[test]
    fn norm_cdf_ends_at_one_and_is_monotone() {
        let c = vec![2.0, 3.0, 5.0];
        let e = vec![0.0, 1.0, 2.0, 3.0];
        let cdf = apply_norm(&c, &e, HistNorm::Cdf);
        assert_eq!(cdf, vec![0.2, 0.5, 1.0]);
    }
}
