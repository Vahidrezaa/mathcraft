//! تاریخ‌بندی histogram/histcounts — قاعدهٔ auto متلب (سند ۰۴ §7، ROADMAP 2.6).
//!
//! قراردادهای متلب (R2026b doc):
//! - شمارش: `[edges(k), edges(k+1))` و **آخرین بین بستهٔ راست** (شامل edges آخر)؛
//!   مقادیر بیرون edges شمرده نمی‌شوند؛ NaN همیشه و ±Inf (جز وقتی خود مرزند) نادیده.
//! - روش‌های عرض بین: scott `3.5·σ·n^(−1/3)`، fd `2·iqr·n^(−1/3)` (iqr=0 →
//!   `0.2·range·n^(−1/3)`)، sturges `ceil(1+log2 n)`، sqrt `ceil(√n)`،
//!   integers عرض ۱ با مرزهای نیم‌صحیح (سقف 65536 بین).
//! - **'auto'** (پیش‌فرض): متلب فرمول دقیق را مستند نکرده («عرض را طوری
//!   تنظیم می‌کند که مرزها روی اعداد خوش‌فرم بیفتند»). قرارداد این موتور —
//!   اسکات مقیاس‌شده با `2^(1/3)` + گرد عرض به نزدیک‌ترین
//!   `{1, 2, 2.5, 5, 10}×10^k` + گسترش مرزها به مضارب عرض خوش‌فرم تا داده را
//!   بپوشاند — در COMPATIBILITY §4.5 ثبت شده و با MATLAB واقعی کاربر **پین**
//!   می‌شود (الگوی بندهای قالب نمایش، گام 2.4).
//!
//! نویسنده‌های مرجع (iqr/quantile): درون‌یابی خطی نوع-7 (h = 1+(n−1)p —
//! سازگار با numpy.default / prctile متلب).

use mc_core::McError;

use crate::linalg::num_error;

/// روش انتخاب عرض بین (BinMethod متلب).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinMethod {
    Auto,
    Scott,
    Fd,
    Sturges,
    Sqrt,
    Integers,
}

impl BinMethod {
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "auto" => Some(BinMethod::Auto),
            "scott" => Some(BinMethod::Scott),
            "fd" | "freedman diaconis" | "freedman-diaconis" => Some(BinMethod::Fd),
            "sturges" => Some(BinMethod::Sturges),
            "sqrt" => Some(BinMethod::Sqrt),
            "integers" => Some(BinMethod::Integers),
            _ => None,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            BinMethod::Auto => "auto",
            BinMethod::Scott => "scott",
            BinMethod::Fd => "fd",
            BinMethod::Sturges => "sturges",
            BinMethod::Sqrt => "sqrt",
            BinMethod::Integers => "integers",
        }
    }
}

/// مقادیر قابل‌شمارش — NaN و ±Inf کنار می‌روند (رفتار متلب histcounts).
fn finite_values(x: &[f64]) -> Vec<f64> {
    x.iter().copied().filter(|v| v.is_finite()).collect()
}

/// انحراف معیار n−1 روی مقادیر متناهی (NaN از قبل کنار رفته).
fn std_finite(v: &[f64], n: usize) -> f64 {
    if n < 2 {
        return 0.0;
    }
    let mean = v.iter().sum::<f64>() / n as f64;
    let s: f64 = v.iter().map(|&x| (x - mean) * (x - mean)).sum();
    (s / (n - 1) as f64).sqrt()
}

/// چارک با درون‌یابی خطی نوع-7 — p ∈ [0,1].
fn quantile7(sorted: &[f64], p: f64) -> f64 {
    let n = sorted.len();
    if n == 1 {
        return sorted[0];
    }
    let h = (n as f64 - 1.0) * p;
    let lo = h.floor() as usize;
    let hi = (lo + 1).min(n - 1);
    let frac = h - lo as f64;
    sorted[lo] * (1.0 - frac) + sorted[hi] * frac
}

/// گرد به نزدیک‌ترین «عدد خوش‌فرم» از {1, 2, 2.5, 5, 10}×10^k.
fn nice_number(h: f64) -> f64 {
    let d = 10f64.powf(h.abs().log10().floor());
    let m = h / d;
    let nice = [1.0f64, 2.0, 2.5, 5.0, 10.0];
    let mut best = nice[0];
    let mut best_d = (m - nice[0]).abs();
    for &c in &nice[1..] {
        let dd = (m - c).abs();
        if dd < best_d {
            best = c;
            best_d = dd;
        }
    }
    best * d
}

/// مرزها از عرض w شروع‌شده از مضرب w ≤ lo تا پوشش hi (مضارب خوش‌فرم).
pub fn edges_for_width(lo: f64, hi: f64, w: f64) -> Vec<f64> {
    let start = (lo / w).floor() * w;
    let nbins = (((hi - start) / w).ceil() as usize).max(1);
    (0..=nbins).map(|k| start + k as f64 * w).collect()
}

/// مرزهای یکنواخت nbins+1 روی [lo, hi] (شکل nbins متلب).
pub fn edges_for_nbins(lo: f64, hi: f64, nbins: usize) -> Vec<f64> {
    (0..=nbins)
        .map(|k| lo + (hi - lo) * (k as f64) / (nbins as f64))
        .collect()
}

/// انتخاب مرزها با BinMethod — روی مقادیر متناهی (xsorted صعودی).
///
/// دادهٔ خالی: مرز پیش‌فرض متلب `[0 1]`.
pub fn auto_edges(x: &[f64], method: BinMethod) -> Result<Vec<f64>, McError> {
    let v = finite_values(x);
    if v.is_empty() {
        return Ok(vec![0.0, 1.0]);
    }
    let mut s = v.clone();
    s.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Less));
    let lo = s[0];
    let hi = s[s.len() - 1];
    let n = v.len();
    if lo == hi {
        // دادهٔ ثابت — یک بین مرکزی (قرارداد مستند)
        return Ok(vec![lo - 0.5, hi + 0.5]);
    }
    let edges = match method {
        BinMethod::Sturges => {
            let nbins = ((1.0 + (n as f64).log2()).ceil() as usize).max(1);
            edges_for_nbins(lo, hi, nbins)
        }
        BinMethod::Sqrt => {
            let nbins = ((n as f64).sqrt().ceil() as usize).max(1);
            edges_for_nbins(lo, hi, nbins)
        }
        BinMethod::Integers => {
            // عرض ۱، مرزها روی نیم‌صحیح‌ها؛ سقف 65536 بین (doc متلب)
            let range = hi - lo;
            let w = if range > 65536.0 {
                (range / 65536.0).ceil()
            } else {
                1.0
            };
            let start = (lo / w).floor() * w - 0.5;
            let nbins = (((hi + 0.5 - start) / w).ceil() as usize).max(1);
            (0..=nbins).map(|k| start + k as f64 * w).collect()
        }
        BinMethod::Scott | BinMethod::Fd | BinMethod::Auto => {
            let h = match method {
                BinMethod::Fd => {
                    let q1 = quantile7(&s, 0.25);
                    let q3 = quantile7(&s, 0.75);
                    let h_fd = 2.0 * (q3 - q1) * (n as f64).powf(-1.0 / 3.0);
                    if h_fd > 0.0 {
                        h_fd
                    } else {
                        // iqr=0 → جایگزین doc متلب: 0.2·range·n^(−1/3)
                        0.2 * (hi - lo) * (n as f64).powf(-1.0 / 3.0)
                    }
                }
                _ => {
                    // scott و auto: σ با نرمال‌سازی n−1
                    let sigma = std_finite(&v, n);
                    let base = 3.5 * sigma * (n as f64).powf(-1.0 / 3.0);
                    if method == BinMethod::Auto {
                        base * 2.0f64.powf(1.0 / 3.0)
                    } else {
                        base
                    }
                }
            };
            if !h.is_finite() || h <= 0.0 {
                edges_for_nbins(lo, hi, 1)
            } else {
                // «تنظیم جزئی» متلب: عرض خوش‌فرم + مرزها روی مضارب آن
                edges_for_width(lo, hi, nice_number(h))
            }
        }
    };
    Ok(edges)
}

/// شمارش روی مرزهای صریح — `[e(k), e(k+1))` با آخرین بین بستهٔ راست.
/// خروجی: counts و (اختیاری) اندیس بین هر عنصر — بیرون مرزها = 0.
pub fn counts_for_edges(x: &[f64], edges: &[f64]) -> (Vec<f64>, Vec<f64>) {
    let nbins = edges.len().saturating_sub(1);
    let mut counts = vec![0.0f64; nbins];
    let mut bin = Vec::with_capacity(x.len());
    for &v in x {
        if v.is_nan() {
            bin.push(0.0);
            continue;
        }
        // ±Inf فقط وقتی شمرده می‌شود که خودِ مرز باشند (doc متلب)
        let idx = if edges.len() < 2 {
            0
        } else {
            find_bin(edges, v)
        };
        if idx == 0 {
            bin.push(0.0);
        } else {
            counts[idx - 1] += 1.0;
            bin.push(idx as f64);
        }
    }
    (counts, bin)
}

/// اندیس بین ۱-پایه (0 = بیرون مرزها) — جست‌وجوی دودویی روی مرزهای صعودی.
fn find_bin(edges: &[f64], v: f64) -> usize {
    let last = edges.len() - 1;
    if v < edges[0] || v > edges[last] {
        return 0;
    }
    // آخرین بین بستهٔ راست: v == edges[last] → بین آخر
    if v == edges[last] {
        return last;
    }
    match edges.binary_search_by(|&e| e.partial_cmp(&v).unwrap_or(std::cmp::Ordering::Less)) {
        Ok(k) => {
            // v دقیقاً روی مرز k → بینِ سمت راست (چپِ بسته، رفتار متلب
            // «edges(k) ≤ x < edges(k+1)») → اندیس ۱-پایه k+1
            (k + 1).min(last)
        }
        Err(k) => k.max(1).min(last),
    }
}

/// اعتبارسنجی مرزهای صریح — صعودی اکید، دست‌کم دو مرز.
pub fn validate_edges(edges: &[f64]) -> Result<(), McError> {
    if edges.len() < 2 {
        return Err(num_error(
            mc_core::error::ids::SIZE_MISMATCH,
            "histogram: edges must have at least two elements.",
            "histogram: مرزها دست‌کم دو عنصر می‌خواهند.",
        ));
    }
    if edges
        .windows(2)
        .any(|w| w[0].partial_cmp(&w[1]) != Some(std::cmp::Ordering::Less))
    {
        return Err(num_error(
            mc_core::error::ids::SIZE_MISMATCH,
            "histogram: edges must be strictly increasing.",
            "histogram: مرزها باید اکیداً صعودی باشند.",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_explicit_edges_right_closed_last_bin() {
        let x = [0.5, 0.8, 1.2, 1.9, 2.3, 3.0];
        let edges = [0.0, 1.0, 2.0, 3.0];
        let (counts, _) = counts_for_edges(&x, &edges);
        assert_eq!(counts, vec![2.0, 2.0, 2.0], "last bin closed on the right");
    }

    #[test]
    fn counts_outside_edges_and_nan_ignored() {
        let x = [-1.0, 0.5, f64::NAN, 5.0];
        let (counts, bin) = counts_for_edges(&x, &[0.0, 1.0]);
        assert_eq!(counts, vec![1.0]);
        assert_eq!(bin, vec![0.0, 1.0, 0.0, 0.0]);
    }

    #[test]
    fn counts_interior_boundary_goes_right() {
        // متلب: x == edges(k) در بین k (سمت راست) می‌افتد؛ مرز آخر بستهٔ راست
        let (counts, _) = counts_for_edges(&[0.0, 1.0, 2.0], &[0.0, 1.0, 2.0]);
        assert_eq!(counts, vec![1.0, 2.0]);
    }

    #[test]
    fn inf_counted_only_when_edge() {
        let x = [f64::INFINITY, 0.5];
        let (_, _) = counts_for_edges(&x, &[0.0, 1.0]); // Inf بیرون → 0
        let (counts, _) = counts_for_edges(&x, &[0.0, f64::INFINITY]);
        assert_eq!(counts, vec![2.0], "Inf edge closes the last bin");
    }

    #[test]
    fn auto_rule_pinned_example() {
        // قاعدهٔ مستند ما: اسکات×2^(1/3) → گرد خوش‌فرم → مضارب پوشش‌دهنده
        // x = [1 2 3 4 5]: σ(n−1) = 1.5811، h = 2^(1/3)·3.5·σ·5^(−1/3) ≈ 4.077
        // → گرد به 5 → start = 0، پوشش تا 5 → مرزها [0 5]
        let edges = auto_edges(&[1.0, 2.0, 3.0, 4.0, 5.0], BinMethod::Auto).unwrap();
        assert_eq!(edges, vec![0.0, 5.0]);
    }

    #[test]
    fn sturges_and_sqrt_counts() {
        let x: Vec<f64> = (1..=32).map(|k| k as f64).collect();
        // sturges: ceil(1 + log2(32)) = 6 بین
        let e = auto_edges(&x, BinMethod::Sturges).unwrap();
        assert_eq!(e.len() - 1, 6);
        assert_eq!(e[0], 1.0);
        assert_eq!(e[e.len() - 1], 32.0);
        // sqrt: ceil(√32) = 6 بین
        let e2 = auto_edges(&x, BinMethod::Sqrt).unwrap();
        assert_eq!(e2.len() - 1, 6);
    }

    #[test]
    fn scott_known_width() {
        // x = [1 2 3 4 5]: scott h = 3.5·1.5811·5^(−1/3) ≈ 3.236 → گرد به 2.5
        // → start = 0، پوشش تا 5 → [0 2.5 5]
        let e = auto_edges(&[1.0, 2.0, 3.0, 4.0, 5.0], BinMethod::Scott).unwrap();
        assert_eq!(e, vec![0.0, 2.5, 5.0]);
    }

    #[test]
    fn integers_rule_half_integer_edges() {
        // دادهٔ صحیح 1..4 → مرزها 0.5 1.5 2.5 3.5 4.5
        let x = [1.0, 2.0, 3.0, 4.0];
        let e = auto_edges(&x, BinMethod::Integers).unwrap();
        assert_eq!(e, vec![0.5, 1.5, 2.5, 3.5, 4.5]);
    }

    #[test]
    fn constant_data_single_bin() {
        let e = auto_edges(&[3.0, 3.0, 3.0], BinMethod::Auto).unwrap();
        assert_eq!(e, vec![2.5, 3.5]);
    }

    #[test]
    fn empty_data_default_edges() {
        let e = auto_edges(&[], BinMethod::Auto).unwrap();
        assert_eq!(e, vec![0.0, 1.0]);
    }

    #[test]
    fn fd_zero_iqr_fallback() {
        // نیمی از داده یکسان → iqr کوچک؛ حد وسط‌ها چک می‌شود که صفر نشود
        let mut x: Vec<f64> = vec![5.0; 8];
        x.extend([1.0, 9.0]);
        let e = auto_edges(&x, BinMethod::Fd).unwrap();
        assert!(e.len() >= 2);
        assert!(e.windows(2).all(|w| w[0] < w[1]));
    }

    #[test]
    fn edges_validation() {
        assert!(validate_edges(&[0.0]).is_err());
        assert!(validate_edges(&[0.0, 1.0, 1.0]).is_err());
        assert!(validate_edges(&[0.0, 1.0, 2.0]).is_ok());
    }
}
