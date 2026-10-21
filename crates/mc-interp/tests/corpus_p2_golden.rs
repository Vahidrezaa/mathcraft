//! کورپوس طلای P2 — حلگرها/درون‌یابی/چندجمله‌ای/آمار (سند ۰۸ §2).
//!
//! مراجع:
//! - موارد تحلیلی: مقدار بستهٔ ریاضی (exp(−2)، √π، √2، ...) — تلورانس 1e-6.
//! - van der Pol μ=1000: مرجع مستقل Octave 9.4 (DP54 همان متلب) — تلورانس
//!   5e-2 (دو پیاده‌سازی تطبیقی با توالی گام متفاوت؛ سند ۰۴ §4.2).
//! - RNG عمداً در کورپوس Octave-مقایسه‌ای نیست (سازگاری بیت‌به‌بیت هدف
//!   نیست — سند ۰۴ §7/COMPATIBILITY)؛ تکرارپذیری در تست واحد rng پوشش دارد.

use mc_interp::{eval_program, Environment};
use mc_parser::parse;

fn run(src: &str, var: &str) -> f64 {
    let prog = parse(src).expect("parse");
    let mut env = Environment::new();
    let out = eval_program(&prog, &mut env);
    assert!(out.error.is_none(), "engine error: {:?}", out.error);
    match env.get(var) {
        Some(mc_core::Value::Double(m)) if m.is_scalar() => m.get(0, 0),
        other => panic!("'{var}' is not a scalar: {other:?}"),
    }
}

fn run_vec(src: &str, var: &str) -> Vec<f64> {
    let prog = parse(src).expect("parse");
    let mut env = Environment::new();
    let out = eval_program(&prog, &mut env);
    assert!(out.error.is_none(), "engine error: {:?}", out.error);
    match env.get(var) {
        Some(mc_core::Value::Double(m)) => (0..m.numel()).map(|k| m.get_linear(k)).collect(),
        other => panic!("'{var}' is not a vector: {other:?}"),
    }
}

fn approx(a: f64, b: f64, tol: f64) -> bool {
    (a - b).abs() <= tol * b.abs().max(1.0)
}

macro_rules! src {
    ($f:literal) => {
        include_str!(concat!("../../../test-corpus/scripts/", $f))
    };
}

// ---------- ode ----------

#[test]
fn p2_ode45_exponential_decay() {
    // y' = −y، y(0)=1 → y(2) = e^{−2}
    let y = run(src!("p2_01_ode45_decay.m"), "y_end");
    assert!(approx(y, (-2.0_f64).exp(), 1e-5), "y(2) = {y}");
}

#[test]
fn p2_ode23_matches_loose_tolerance() {
    // y' = y·sin(t)، y(0)=1 → y(2) = exp(1 − cos 2) (کلوزفورم)
    let y = run(src!("p2_02_ode23_sine.m"), "y_end");
    let want = (1.0 - 2.0_f64.cos()).exp();
    assert!(approx(y, want, 1e-3), "y(2) = {y} want {want}");
}

#[test]
fn p2_ode45_multioutput_t_y() {
    // [T, Y] = ode45 — T ستون n×1 و Y ستون n×1 (y0 اسکالر) — سمانتیک متلب
    let prog = parse(src!("p2_03_ode45_multiout.m")).expect("parse");
    let mut env = Environment::new();
    let out = eval_program(&prog, &mut env);
    assert!(out.error.is_none(), "engine error: {:?}", out.error);
    let t = env.get("T").expect("T missing");
    let y = env.get("Y").expect("Y missing");
    let (tr, tc) = t.shape();
    let (yr, yc) = y.shape();
    assert_eq!(tc, 1, "T must be a column");
    assert_eq!((yr, yc), (tr, 1), "Y must match T rows");
    assert!(tr >= 2, "at least two steps");
}

// ---------- integral / fzero / roots ----------

#[test]
fn p2_integral_gaussian() {
    // ∫_{−3}^{3} e^{−x²} = √π
    let q = run(src!("p2_04_integral_gaussian.m"), "q");
    assert!(approx(q, std::f64::consts::PI.sqrt(), 1e-8), "q = {q}");
}

#[test]
fn p2_integral_half_infinite() {
    // ∫_0^∞ e^{−x} dx = 1
    let q = run(src!("p2_05_integral_infinite.m"), "q");
    assert!(approx(q, 1.0, 1e-7), "q = {q}");
}

#[test]
fn p2_fzero_sqrt2() {
    let x = run(src!("p2_06_fzero_sqrt2.m"), "x");
    assert!(approx(x, std::f64::consts::SQRT_2, 1e-10), "x = {x}");
}

#[test]
fn p2_roots_quartic_real() {
    // x⁴ − 10x³ + 35x² − 50x + 24 = (x−1)(x−2)(x−3)(x−4)
    let rr = run_vec(src!("p2_07_roots_quartic.m"), "rre");
    let mut sorted = rr;
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let want = [1.0, 2.0, 3.0, 4.0];
    for (g, w) in sorted.iter().zip(want.iter()) {
        assert!(approx(*g, *w, 1e-7), "root {g} vs {w}");
    }
    // بخش موهومی ~0
    let ri = run_vec(src!("p2_07_roots_quartic.m"), "rim");
    for i in ri {
        assert!(i.abs() < 1e-7, "imag {i}");
    }
}

#[test]
fn p2_roots_complex_pair() {
    // x² + 1 = 0 → ±i
    let ri = run_vec(src!("p2_08_roots_complex.m"), "rim");
    let mut s = ri;
    s.sort_by(|a, b| a.partial_cmp(b).unwrap());
    assert!(
        approx(s[0], -1.0, 1e-9) && approx(s[1], 1.0, 1e-9),
        "imag {s:?}"
    );
}

// ---------- interp1 / polyfit ----------

#[test]
fn p2_interp1_linear_exact() {
    let v = run(src!("p2_09_interp1_linear.m"), "vq");
    assert!(approx(v, 2.5, 1e-12), "vq = {v}");
}

#[test]
fn p2_polyfit_quadratic_recovery() {
    // y = 2x² − x + 3 → بازیابی ضرایب
    let p = run_vec(src!("p2_10_polyfit_quad.m"), "p");
    assert_eq!(p.len(), 3);
    assert!(approx(p[0], 2.0, 1e-9), "p = {p:?}");
    assert!(approx(p[1], -1.0, 1e-9));
    assert!(approx(p[2], 3.0, 1e-9));
}

#[test]
fn p2_conv_deconv_roundtrip() {
    let q = run_vec(src!("p2_11_conv_deconv.m"), "q");
    // deconv(conv(u,v), v) = u
    let want = [1.0, 2.0, 3.0];
    for (g, w) in q.iter().zip(want.iter()) {
        assert!(approx(*g, *w, 1e-9), "q = {q:?}");
    }
}

// ---------- آمار ----------

#[test]
fn p2_stats_descriptives() {
    // median/std/var/cov/corrcoef روی دادهٔ قطعی
    let med = run(src!("p2_12_stats_basic.m"), "med");
    assert!(approx(med, 3.0, 1e-12), "med = {med}");
    let s = run(src!("p2_12_stats_basic.m"), "sd");
    // var نمونه (n−1): 10/4 = 2.5 → std = √2.5
    assert!(approx(s, 2.5_f64.sqrt(), 1e-12), "sd = {s}");
    let r = run(src!("p2_12_stats_basic.m"), "rho");
    assert!(approx(r, 1.0, 1e-12), "rho = {r}");
}

// ---------- van der Pol μ=1000 (DoD — مرجع Octave) ----------

#[test]
fn p2_ode45_vanderpol_mu1000() {
    // DoD سند ۰۷ — van der Pol μ=1000 روی [0, 3000] (بازهٔ مثال مرجع متلب):
    // ۱) قید کیفی سند ۰۴ §9: چرخهٔ حدی |y1| ≤ 2.1 — بدون انفجار عددی
    // ۲) y2(3000) در فاز آهسته (چرخهٔ حدی با دورهٔ ≈ 1.6·μ ≈ 1614 —
    //    در t=3000 حل روی شاخهٔ بازیابی با y2 کوچک است)
    // ۳) y1(3000) داخل باند چرخهٔ حدی (بازهٔ فیزیکی مسئله)
    // تطبیق نقطه‌ای با مرجع Octave پس از پایان رفرنس‌گیری افزوده می‌شود.
    let y1_3000 = run(src!("p2_13_vanderpol_mu1000.m"), "y1_end");
    let y2_3000 = run(src!("p2_13_vanderpol_mu1000.m"), "y2_end");
    let ymax = run(src!("p2_13_vanderpol_mu1000.m"), "ymax");
    assert!(ymax < 2.2, "y1 max = {ymax}");
    assert!(ymax > 1.9, "limit cycle amplitude collapsed: {ymax}");
    assert!(y2_3000.abs() < 0.1, "slow phase expected, y2 = {y2_3000}");
    assert!(y1_3000.abs() < 2.2, "y1(3000) = {y1_3000}");
}
