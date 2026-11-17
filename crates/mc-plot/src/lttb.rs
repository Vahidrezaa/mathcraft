//! LTTB — downsampling نگه‌دارندهٔ شکل منحنی (سند ۰۴ §8.2).
//!
//! آستانهٔ انتقال: سری با بیش از [`THRESHOLD`] نقطه قبل از عبور از FFI به
//! ≤ [`THRESHOLD`] نقطه کاسته می‌شود. دادهٔ کامل در Rust می‌ماند (سند ۰۲ §7)
//! و هر re-decimation برای viewport جدید از دادهٔ خام انجام می‌شود.

/// سقف نقاط هر سری در انتقال به UI (سند ۰۴ §8.2).
pub const THRESHOLD: usize = 4000;

/// ایندکس‌های انتخاب‌شدهٔ LTTB — الگوریتم کلاسیک:
/// اولین و آخرین نقطه همیشه می‌مانند؛ در هر bucket نقطه‌ای انتخاب می‌شود
/// که مساحت مثلث (آخرین انتخاب‌شده، داوطلب، میانگین bucket بعدی) را بیشینه
/// کند. غیرمتناهی‌ها (NaN/Inf) مساحت صفر می‌گیرند و لبه‌ها را می‌بُرند.
pub fn select_indices(x: &[f64], y: &[f64], threshold: usize) -> Vec<usize> {
    let n = x.len();
    debug_assert_eq!(n, y.len());
    if n == 0 {
        return Vec::new();
    }
    if n <= threshold.max(2) {
        return (0..n).collect();
    }
    let threshold = threshold.max(2);
    let mut out = Vec::with_capacity(threshold);
    out.push(0);
    // bucket ها بین نقطهٔ اول و آخر؛ هر bucket یک انتخاب (اول/آخرر جدا INCLUDED)
    let buckets = threshold - 2;
    let bucket_size = (n - 2) as f64 / buckets as f64;
    let mut selected = 0usize;
    for b in 0..buckets {
        let range_start = 1 + ((b as f64) * bucket_size).floor() as usize;
        let range_end = 1 + (((b + 1) as f64) * bucket_size).floor() as usize;
        let range_end = range_end.min(n - 1);
        // میانگین bucket بعدی (آخرین bucket → نقطهٔ آخر)
        let (avg_x, avg_y) = if b + 1 < buckets {
            let next_start = 1 + (((b + 1) as f64) * bucket_size).floor() as usize;
            let next_end = (1 + (((b + 2) as f64) * bucket_size).floor() as usize).min(n - 1);
            let mut sx = 0.0;
            let mut sy = 0.0;
            let mut cnt = 0.0;
            for i in next_start..next_end.max(next_start + 1) {
                sx += x[i];
                sy += y[i];
                cnt += 1.0;
            }
            (sx / cnt, sy / cnt)
        } else {
            (x[n - 1], y[n - 1])
        };
        let ax = x[selected];
        let ay = y[selected];
        let mut best = range_start;
        let mut best_area = f64::NEG_INFINITY;
        for i in range_start..range_end.max(range_start + 1) {
            let area = ((x[i] - ax) * (avg_y - ay) - (y[i] - ay) * (avg_x - ax)).abs();
            if area.is_finite() && area > best_area {
                best_area = area;
                best = i;
            }
        }
        out.push(best);
        selected = best;
    }
    out.push(n - 1);
    out.dedup();
    out
}

/// کاستن یک سری به آستانه — خروجی: (x, y) انتخاب‌شده.
/// سری با n ≤ threshold دست‌نخورده برمی‌گردد (هویتی).
pub fn decimate(x: &[f64], y: &[f64], threshold: usize) -> (Vec<f64>, Vec<f64>) {
    let idx = select_indices(x, y, threshold);
    (
        idx.iter().map(|&i| x[i]).collect(),
        idx.iter().map(|&i| y[i]).collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// مساحت مطلق زیر منحنی (تراپز روی |y|) — متر کیفیت سند ۰۴ §8.2؛
    /// مطلق چون سیگنال‌های صفر-میانه (chirp/sinc) در مساحت امضادار حذف می‌شوند.
    fn auc(x: &[f64], y: &[f64]) -> f64 {
        x.windows(2)
            .zip(y.windows(2))
            .map(|(wx, wy)| (wx[1] - wx[0]) * (wy[0].abs() + wy[1].abs()) / 2.0)
            .sum()
    }

    #[test]
    fn identity_at_or_below_threshold() {
        let x: Vec<f64> = (0..100).map(|i| i as f64).collect();
        let y: Vec<f64> = x.iter().map(|v| (v / 10.0).sin()).collect();
        let (dx, dy) = decimate(&x, &y, 100);
        assert_eq!(dx, x);
        assert_eq!(dy, y);
    }

    #[test]
    fn output_respects_threshold_and_keeps_endpoints() {
        let n = 50_000;
        let x: Vec<f64> = (0..n).map(|i| i as f64 / 100.0).collect();
        let y: Vec<f64> = x.iter().map(|v| (2.0 * v).sin()).collect();
        let (dx, dy) = decimate(&x, &y, THRESHOLD);
        assert!(dx.len() <= THRESHOLD, "decimated to {}", dx.len());
        assert!(dx.len() >= THRESHOLD.saturating_sub(2));
        assert_eq!(dx.len(), dy.len());
        assert_eq!(dx.first(), x.first());
        assert_eq!(dx.last(), x.last());
    }

    #[test]
    fn area_error_under_one_percent_on_smooth_references() {
        // مجموعه‌های مرجع هموار سند ۰۴ §8.2 — خطای ناحیه پس از decimation
        let n = 200_000;
        let cases: Vec<(&str, Vec<f64>)> = vec![
            (
                "chirp",
                (0..n)
                    .map(|i| {
                        let t = i as f64 / n as f64;
                        (t * 40.0 * std::f64::consts::PI).sin()
                    })
                    .collect(),
            ),
            (
                "gaussian-pdf",
                (0..n)
                    .map(|i| {
                        let t = (i as f64 / n as f64) * 10.0 - 5.0;
                        (-(t * t) / 2.0).exp() / (2.0 * std::f64::consts::PI).sqrt()
                    })
                    .collect(),
            ),
            (
                "sinc",
                (0..n)
                    .map(|i| {
                        let t = (i as f64 / n as f64) * 20.0 - 10.0;
                        if t.abs() < 1e-9 {
                            1.0
                        } else {
                            t.sin() / t
                        }
                    })
                    .collect(),
            ),
        ];
        for (name, y) in cases {
            let x: Vec<f64> = (0..n).map(|i| i as f64).collect();
            let (dx, dy) = decimate(&x, &y, THRESHOLD);
            let a_full = auc(&x, &y).abs();
            let a_dec = auc(&dx, &dy).abs();
            let rel = (a_full - a_dec).abs() / a_full.max(1e-12);
            assert!(
                rel < 0.01,
                "{name}: area error {rel:.4} exceeds 1% (full {a_full:.3} vs dec {a_dec:.3})"
            );
        }
    }

    #[test]
    fn monotone_series_keeps_extremes() {
        // منحنی صعودی: نمونه‌گیری باید min/max را نگه دارد (اول/آخر)
        let n = 20_000;
        let x: Vec<f64> = (0..n).map(|i| i as f64).collect();
        let y: Vec<f64> = x.iter().map(|v| v * 0.5).collect();
        let (_, dy) = decimate(&x, &y, 100);
        assert_eq!(*dy.first().unwrap(), 0.0);
        assert_eq!(*dy.last().unwrap(), (n - 1) as f64 * 0.5);
    }
}
