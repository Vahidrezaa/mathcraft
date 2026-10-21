//! RNG — PCG64 برای یکنواخت + Ziggurat برای نرمال (سند ۰۴ بخش ۷).
//!
//! قراردادهای سند ۰۴:
//! - `rng(seed)` تکرارپذیر — تکالیف درسی deterministic می‌شوند.
//! - **سازگاری بیت‌به‌بیت با Mersenne Twister متلب هدف نیست** (مستند در
//!   COMPATIBILITY): دنبالهٔ مستقل MathCraft؛ نتایج آماری معادل‌اند.
//! - `randn` با **Ziggurat** (Marsaglia–Tsang) — جدول در اولین استفاده
//!   **برنامه‌ای** ساخته می‌شود: نوارهای هم‌جرم روی CDF دقیق + دم حلقه‌ای؛
//!   صحت با تست moment و χ² goodness-of-fit تضمین می‌شود (سند ۰۴ §7).
//!
//! حالت سراسری: مفسر تک‌نخی است (موتور پشت Mutex — سند ۰۲ §2)؛
//! state در `Mutex` استاتیک نگه داشته می‌شود و `rng(seed)` آن را ریست می‌کند.

use std::sync::{Mutex, OnceLock};

// ---------- PCG-XSH-RR 128/64 (O'Neill — قرارداد رسمی PCG) ----------

const PCG_MULTIPLIER: u128 = 0x2360_ED05_1FC5_DA0D;

/// مولد PCG64 — state/stream از قرارداد رسمی PCG.
#[derive(Debug, Clone)]
pub struct Pcg64 {
    state: u128,
    inc: u128,
}

impl Pcg64 {
    /// ساخت با seed و stream id — initialization پیرو PCG reference.
    pub fn new(seed: u64, stream: u64) -> Self {
        let mut rng = Pcg64 {
            state: 0,
            inc: ((stream as u128) << 1) | 1,
        };
        rng.step();
        rng.state ^= seed as u128;
        rng.step();
        rng
    }

    fn step(&mut self) {
        self.state = self
            .state
            .wrapping_mul(PCG_MULTIPLIER)
            .wrapping_add(self.inc);
    }

    /// 64 بیت خروجی — XSH-RR: xorshift-high + rotate-low.
    pub fn next_u64(&mut self) -> u64 {
        self.step();
        let word = ((self.state >> 64) as u64) ^ (self.state as u64);
        let rot = (self.state >> 122) as u32;
        word.rotate_right(rot)
    }

    /// یکنواخت [0,1) با 53 بیت تصادف — دقت double مثل rand متلب.
    pub fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 * (1.0 / 9_007_199_254_740_992.0)
    }

    /// یکنواخت صحیح شامل دو سر [low, high] — unbiased با رد لِمِر (Lemire).
    pub fn next_u64_in(&mut self, low: u64, high: u64) -> u64 {
        debug_assert!(low <= high, "next_u64_in: low > high");
        let range = high - low + 1;
        if range == 0 {
            // سرریز یعنی کل فضای u64 — بدون رد
            return self.next_u64();
        }
        let threshold = range.wrapping_neg() % range;
        loop {
            let x = self.next_u64();
            if x >= threshold {
                return low + (x % range);
            }
        }
    }
}

// ---------- Ziggurat نرمال — ۲۵۵ نوار هم‌جرم + دم حلقه‌ای ----------
//
// طراحی rigorously درست (بدون جدول دستی، بدون ریسک انفجار بازگشتی):
//
// ۱. جرم نیمهٔ مثبت نرمال M = √(2π)/2 به ۲۵۶ شکاف هم‌جرم M/256 تقسیم می‌شود.
// ۲. شکاف‌های ۰..=254 نوارهای متناهی [bounds[i], bounds[i+1]] با مرزهایی که
//    دو نیم‌بختی روی CDF محلی پیدا می‌کند (∫ با Gauss–Legendre ۲۴ نقطه).
// ۳. شکاف ۲۵۵ دم [R, ∞) است؛ R از «∫_R^∞ f = M/256» با دو نیم‌بختی می‌آید.
// ۴. نمونه‌گیری:
//    - انتخاب شکاف با u·M روی جرم‌های تجمعی واقعی (وزن = جرم پذیرفته‌شده —
//      این کلید صحت است: مخلوط Σ (a_i/M)·(f·1_i/a_i) ≡ f).
//    - نوار متناهی: (x, y) یکنواخت در [x_{i−1}, x_i]×[0, f(x_{i−1})]، پذیرش y < f(x)
//      → چگالی پذیرفته = برش دقیق f روی نوار.
//    - دم حلقه‌ای: x = √(R² − 2ln u₁)، پذیرش u₂·x < R — پوش c·q ≥ f با نسبت
//      دقیقاً x/R (قابل‌اثبات) و پذیرش ≈ T·R/f(R) ≈ 0.85.
// ۵. علامت یکنواخت ±1 → نیمه‌نرمال به نرمال کامل.

const ZIG_LEVELS: usize = 255; // تعداد نوارهای متناهی (شکاف ۲۵۵ = دم)
const HALF_NORM_CONST: f64 = 0.5; // ∫₀^∞ f = 0.5 — جرم نیمهٔ مثبت نرمالِ نرمالیزه

fn normal_pdf(x: f64) -> f64 {
    (-0.5 * x * x).exp() / (2.0 * std::f64::consts::PI).sqrt()
}

/// گره‌ها/وزن‌های Gauss–Legendre n نقطه — Newton روی ریشه‌های P_n
/// (الگوریتم مرجع gauleg). در زمان اجرا ساخته می‌شود تا هیچ جدول دستی
/// (ریسک تایپو) وجود نداشته باشد.
fn gauss_legendre(n: usize) -> (Vec<f64>, Vec<f64>) {
    let mut x = vec![0.0f64; n];
    let mut w = vec![0.0f64; n];
    let m = (n + 1) / 2;
    for i in 0..m {
        // حدس اولیه ریشه i-ام (قرارداد gauleg)
        let mut z = (std::f64::consts::PI * (i as f64 + 0.75) / (n as f64 + 0.5)).cos();
        let mut p1 = 0.0f64;
        let mut pp = 0.0f64;
        for _ in 0..100 {
            p1 = 1.0;
            let mut p2 = 0.0;
            for j in 0..n {
                let p3 = p2;
                p2 = p1;
                p1 = ((2.0 * j as f64 + 1.0) * z * p2 - (j as f64) * p3) / (j as f64 + 1.0);
            }
            // مشتق P_n(z)
            pp = (n as f64) * (z * p1 - p2) / (z * z - 1.0);
            let dz = -p1 / pp;
            z += dz;
            if dz.abs() < 1e-15 {
                break;
            }
        }
        x[i] = -z;
        x[n - 1 - i] = z;
        w[i] = 2.0 / ((1.0 - z * z) * pp * pp);
        w[n - 1 - i] = w[i];
    }
    (x, w)
}

/// ∫_a^b f(x)dx با GL 24 نقطه (خطای ~1e-16 برای f نرم).
fn integrate_pdf(a: f64, b: f64) -> f64 {
    static GL: OnceLock<(Vec<f64>, Vec<f64>)> = OnceLock::new();
    let (gx, gw) = GL.get_or_init(|| gauss_legendre(24));
    let c = 0.5 * (a + b);
    let hl = 0.5 * (b - a);
    let mut acc = 0.0;
    for (i, &g) in gx.iter().enumerate() {
        acc += gw[i] * normal_pdf(c + hl * g);
    }
    acc * hl
}

/// جدول ziggurat — یک‌بار در اولین استفاده ساخته می‌شود.
struct ZigTable {
    /// مرزهای صعودی: bounds[0]=0 … bounds[255]=R (شروع دم)
    bounds: [f64; ZIG_LEVELS + 1],
    /// جرم تجمعی واقعی نوارهای متناهی: cum[0]=0 … cum[255]=Σa_i
    cum: [f64; ZIG_LEVELS + 1],
    /// جرم کل = M (√(2π)/2) — دم = M − Σa_i (مکمل، دقیق)
    total: f64,
    tail_start: f64,
}

fn zig_table() -> &'static ZigTable {
    static TABLE: OnceLock<ZigTable> = OnceLock::new();
    TABLE.get_or_init(|| {
        let slot = HALF_NORM_CONST / (ZIG_LEVELS + 1) as f64; // M/256
        let mut bounds = [0.0f64; ZIG_LEVELS + 1];
        for i in 1..=ZIG_LEVELS {
            // حل ∫_{bounds[i-1]}^x f = slot با دو نیم‌بختی (f اکیداً نزولی → جواب یکتا)
            let lo = bounds[i - 1];
            // سقف امن: چون f نزولی است، عرض w کافی دارد w·f(lo) ≥ slot → ۴× حاشیه
            let mut a = lo;
            let mut b = lo + 4.0 * slot / normal_pdf(lo).max(1e-300);
            for _ in 0..200 {
                let mid = 0.5 * (a + b);
                if integrate_pdf(lo, mid) < slot {
                    a = mid;
                } else {
                    b = mid;
                }
                if b - a < 1e-15 * b.max(1.0) {
                    break;
                }
            }
            bounds[i] = 0.5 * (a + b);
        }
        let tail_start = bounds[ZIG_LEVELS];
        let mut cum = [0.0f64; ZIG_LEVELS + 1];
        for i in 0..ZIG_LEVELS {
            cum[i + 1] = cum[i] + integrate_pdf(bounds[i], bounds[i + 1]);
        }
        ZigTable {
            bounds,
            cum,
            total: HALF_NORM_CONST,
            tail_start,
        }
    })
}

// ---------- حالت سراسری نشست ----------

#[derive(Debug)]
struct GlobalRng {
    uniform: Pcg64,
    normal: Pcg64,
}

static GLOBAL: Mutex<Option<GlobalRng>> = Mutex::new(None);

fn with_global<T>(f: impl FnOnce(&mut GlobalRng) -> T) -> T {
    let mut guard = GLOBAL
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let g = guard.get_or_insert_with(|| GlobalRng {
        uniform: Pcg64::new(0, 0xDA3E_39CB_94B9_5BDB),
        normal: Pcg64::new(0x853C_49E6_748F_EA9B, 0xDA3E_39CB_94B9_5BDB),
    });
    f(g)
}

/// `rng(seed)` — ریست دنباله‌ها با seed تازه؛ تکرارپذیری کامل (سند ۰۴ §7).
pub fn reseed(seed: u64) {
    with_global(|g| {
        g.uniform = Pcg64::new(seed, 0xDA3E_39CB_94B9_5BDB);
        g.normal = Pcg64::new(seed ^ 0x853C_49E6_748F_EA9B, 0x9E37_79B9_7F4A_7C15);
    });
}

/// یکنواخت [0,1) از دنبالهٔ نشست (`rand`).
pub fn uniform() -> f64 {
    with_global(|g| g.uniform.next_f64())
}

/// نرمال استاندارد N(0,1) با Ziggurat (`randn`).
pub fn normal() -> f64 {
    with_global(|g| g.normal_zig())
}

impl GlobalRng {
    fn normal_zig(&mut self) -> f64 {
        let table = zig_table();
        loop {
            let bits = self.normal.next_u64();
            let sign = if bits & 1 == 0 { 1.0 } else { -1.0 };
            // انتخاب شکاف با جرم واقعی: u ∈ [0, total) → جست‌وجوی تجمعی
            let u_sel = self.normal.next_f64() * table.total;
            if u_sel >= table.cum[ZIG_LEVELS] {
                // دم حلقه‌ای: x = √(R² − 2ln u₁) ؛ پذیرش u₂·x < R (نسبت پوش x/R)
                // هر تلاش پیشنهاد تازهٔ حلقه‌ای است — پذیرش مستقل از نوار ⇒ دقیق
                let r = table.tail_start;
                loop {
                    let u1 = 1.0 - self.normal.next_f64(); // (0,1]
                    let x = (r * r - 2.0 * u1.ln()).sqrt();
                    if self.normal.next_f64() * x < r {
                        return sign * x;
                    }
                }
            }
            // نوار متناهی: جست‌وجوی تجمعی → بازهٔ [bounds[idx], bounds[idx+1]]
            // نوار «یک‌بار» انتخاب می‌شود و ردِ y «درون همان نوار» تکرار می‌گردد —
            // ردِ کلاسیک درون‌نواری دقیق است (ناحیهٔ پذیرش = مساحت زیر f در نوار).
            // اگر به‌جای این، در هر تلاش نوار از نو انتخاب شود، چون نسبت جرمِ
            // پذیرفته (a_k) به مساحت مستطیل (V_k) بین نوارها متفاوت است،
            // نوارهای پهنِ دم کم‌نمونه‌برداری می‌شوند (بایاس سیستماتیک دم).
            let idx = match table.cum.binary_search_by(|probe| {
                probe.partial_cmp(&u_sel).unwrap_or(std::cmp::Ordering::Less)
            }) {
                Ok(i) => i.min(ZIG_LEVELS - 1),
                Err(i) => (i - 1).min(ZIG_LEVELS - 1),
            };
            let lo = table.bounds[idx];
            let width = table.bounds[idx + 1] - lo;
            let top = normal_pdf(lo);
            loop {
                let x = lo + self.normal.next_f64() * width;
                if self.normal.next_f64() * top < normal_pdf(x) {
                    return sign * x;
                }
            }
        }
    }
}

/// یکنواخت صحیح شامل دو سر [low, high] از دنبالهٔ نشست (`randi`).
pub fn uniform_int_in(low: u64, high: u64) -> u64 {
    with_global(|g| g.uniform.next_u64_in(low, high))
}

/// جایگشت تصادفی 1..=n، برش k عضو اول — Fisher–Yates با PCG (`randperm`).
/// خروجی مقادیر 1-پایه به‌صورت f64 (لایهٔ بالا ماتریس می‌سازد).
pub fn permutation(n: usize, k: usize) -> Vec<f64> {
    let mut perm: Vec<u64> = (1..=n as u64).collect();
    with_global(|g| {
        for i in 0..n {
            let j = g.uniform.next_u64_in(i as u64, (n - 1) as u64) as usize;
            perm.swap(i, j);
        }
    });
    perm.truncate(k);
    perm.into_iter().map(|v| v as f64).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// تست‌های state-دار باید سریال اجرا شوند (هارنس Rust موازی است و
    /// همه از یک state سراسری استفاده می‌کنند — تداخل دنباله‌ها).
    fn test_lock() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
        LOCK.lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// erf تقریبی Abramowitz–Stegun 7.1.26 — فقط برای انتظارات GoF در تست‌ها.
    fn erf_approx(x: f64) -> f64 {
        let t = 1.0 / (1.0 + 0.327_591_1 * x.abs());
        let y = 1.0
            - t * (0.254_829_592
                + t * (-0.284_496_736
                    + t * (1.421_413_741 + t * (-1.453_152_027 + t * 1.061_405_429))))
                * (-x * x).exp();
        if x >= 0.0 { y } else { -y }
    }

    #[test]
    fn pcg_seed_reproducibility() {
        let mut a = Pcg64::new(42, 0xDA3E_39CB_94B9_5BDB);
        let mut b = Pcg64::new(42, 0xDA3E_39CB_94B9_5BDB);
        for _ in 0..1000 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
        let mut c = Pcg64::new(43, 0xDA3E_39CB_94B9_5BDB);
        assert_ne!(a.next_u64(), c.next_u64());
    }

    #[test]
    fn ziggurat_table_invariants() {
        let t = zig_table();
        // مرزها اکیداً صعودی از 0
        for i in 0..ZIG_LEVELS {
            assert!(t.bounds[i] < t.bounds[i + 1], "boundary not increasing at {i}");
        }
        assert_eq!(t.bounds[0], 0.0);
        // جرم‌های تجمعی اکیداً صعودی و کمتر از کل
        for i in 0..ZIG_LEVELS {
            assert!(t.cum[i] < t.cum[i + 1], "cumulative mass not increasing at {i}");
        }
        assert!(t.cum[ZIG_LEVELS] < t.total, "tail mass must be positive");
        assert!(t.total - t.cum[ZIG_LEVELS] > 1e-6, "tail too small");
        // سازگاری درونی: انتگرال کل روی [0, R] باید = Σ جرم نوارها
        let whole = integrate_pdf(0.0, t.tail_start);
        assert!(
            (whole - t.cum[ZIG_LEVELS]).abs() < 1e-10,
            "strip masses vs whole: {whole} vs {}",
            t.cum[ZIG_LEVELS]
        );
        // گاوس-لژاندر: مجموع وزن‌ها = 2 و تقارن
        let (gx, gw) = gauss_legendre(24);
        let total: f64 = gw.iter().sum();
        assert!((total - 2.0).abs() < 1e-12, "GL weights sum: {total}");
        for i in 0..12 {
            assert!((gx[i] + gx[23 - i]).abs() < 1e-14);
        }
    }

    #[test]
    fn uniform_in_range_and_bounds() {
        reseed(7);
        for _ in 0..10_000 {
            let v = uniform();
            assert!((0.0..1.0).contains(&v));
        }
        let mut rng = Pcg64::new(1, 1);
        for _ in 0..1000 {
            let x = rng.next_u64_in(3, 9);
            assert!((3..=9).contains(&x));
        }
        assert_eq!(rng.next_u64_in(5, 5), 5);
    }

    #[test]
    fn ziggurat_moments_and_support() {
        let _guard = test_lock();
        reseed(123);
        let n = 1_000_000u64;
        let mut sum = 0.0;
        let mut sum2 = 0.0;
        let mut max = f64::NEG_INFINITY;
        for _ in 0..n {
            let x = normal();
            sum += x;
            sum2 += x * x;
            max = max.max(x);
        }
        let mean = sum / n as f64;
        let var = sum2 / n as f64 - mean * mean;
        assert!(mean.abs() < 0.005, "mean too far: {mean}");
        assert!((var - 1.0).abs() < 0.01, "variance too far: {var}");
        assert!(max < 8.0, "tail out of support: {max}");
    }

    #[test]
    fn ziggurat_chi_square_gof() {
        let _guard = test_lock();
        // goodness-of-fit سند ۰۴ §7: ۱۱ سطل + دم‌ها (۵۰۰k نمونه، آستانهٔ 0.1%)
        reseed(999);
        let n = 500_000u64;
        let edges = [-2.0, -1.5, -1.0, -0.6, -0.3, 0.0, 0.3, 0.6, 1.0, 1.5, 2.0];
        let mut counts = vec![0u64; edges.len() + 1];
        for _ in 0..n {
            let x = normal();
            let idx = match edges.binary_search_by(|e| e.partial_cmp(&x).unwrap()) {
                Ok(i) => i + 1,
                Err(i) => i,
            };
            counts[idx] += 1;
        }
        let cdf = |z: f64| 0.5 * (1.0 + erf_approx(z / std::f64::consts::SQRT_2));
        let mut chi2 = 0.0;
        let mut lo = f64::NEG_INFINITY;
        for (i, hi) in edges.iter().chain(std::iter::once(&f64::INFINITY)).enumerate() {
            let p = cdf(*hi) - cdf(lo);
            let expected = p * n as f64;
            let diff = counts[i] as f64 - expected;
            chi2 += diff * diff / expected;
            lo = *hi;
        }
        // df = 10 → آستانهٔ 0.1% ≈ 29.6
        assert!(chi2 < 29.6, "chi2 = {chi2} (failed goodness-of-fit)");
    }

    #[test]
    fn permutation_is_bijective_and_reproducible() {
        let _guard = test_lock();
        reseed(5);
        let p1 = permutation(10, 10);
        let mut sorted = p1.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        assert_eq!(sorted, (1..=10).map(f64::from).collect::<Vec<_>>());
        reseed(5);
        assert_eq!(p1, permutation(10, 10));
        assert_eq!(permutation(10, 3).len(), 3);
    }

    #[test]
    fn global_reseed_is_deterministic() {
        let _guard = test_lock();
        reseed(42);
        let a: Vec<f64> = (0..5).map(|_| uniform()).collect();
        let an: Vec<f64> = (0..5).map(|_| normal()).collect();
        reseed(42);
        let b: Vec<f64> = (0..5).map(|_| uniform()).collect();
        let bn: Vec<f64> = (0..5).map(|_| normal()).collect();
        assert_eq!(a, b);
        assert_eq!(an, bn);
    }
}
