//! Property tests جبرخطی (سند ۰۸ §3 — DoD سند ۰۷ P1: «Property tests جبرخطی سبز»).
//!
//! invariantهای ریاضی روی ماتریس‌های تصادفی با proptest. ماتریس‌های
//! قطری-مسلط ساخته می‌شوند تا خوش‌شرط باشند (کنترل cond برای آستانه‌های 1e-8).

use mc_core::matrix::Matrix;
use proptest::prelude::*;

/// ماتریس قطری-مسلط n×n تصادفی — قطر از مولفه‌ها بزرگ‌تر تا cond کنترل شود.
fn dominant_matrix(n: usize, seed: u64) -> Matrix {
    // LCG ساده — تکرارپذیر و بدون وابستگی خارجی.
    let mut s = seed | 1;
    let mut next = move || {
        s ^= s << 13;
        s ^= s >> 7;
        s ^= s << 17;
        (s % 2000) as f64 / 1000.0 - 1.0
    };
    let mut m = Matrix::full(n, n, 0.0);
    for i in 0..n {
        let mut row_sum = 0.0;
        for j in 0..n {
            if i != j {
                let v = next();
                m.set(i, j, v);
                row_sum += v.abs();
            }
        }
        m.set(i, i, row_sum + 1.0 + next().abs());
    }
    m
}

fn col_vec(n: usize, seed: u64) -> Matrix {
    let mut s = seed | 3;
    let mut v = Matrix::full(n, 1, 0.0);
    for i in 0..n {
        s ^= s << 13;
        s ^= s >> 7;
        s ^= s << 17;
        v.set(i, 0, (s % 2000) as f64 / 1000.0 - 1.0);
    }
    v
}

fn max_diff(a: &Matrix, b: &Matrix) -> f64 {
    debug_assert_eq!((a.rows(), a.cols()), (b.rows(), b.cols()));
    (0..a.numel())
        .map(|i| (a.get_linear(i) - b.get_linear(i)).abs())
        .fold(0.0f64, f64::max)
}

fn matmul(a: &Matrix, b: &Matrix) -> Matrix {
    let mut out = Matrix::full(a.rows(), b.cols(), 0.0);
    for c in 0..b.cols() {
        for r in 0..a.rows() {
            let mut acc = 0.0;
            for k in 0..a.cols() {
                acc += a.get(r, k) * b.get(k, c);
            }
            out.set(r, c, acc);
        }
    }
    out
}

fn transpose(a: &Matrix) -> Matrix {
    let mut out = Matrix::full(a.cols(), a.rows(), 0.0);
    for c in 0..a.cols() {
        for r in 0..a.rows() {
            out.set(c, r, a.get(r, c));
        }
    }
    out
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn mldivide_solves_exactly(seed in 0u64..10_000) {
        let n = 2 + (seed % 7) as usize; // 2..8
        let a = dominant_matrix(n, seed);
        let b = col_vec(n, seed.wrapping_mul(31));
        let (x, _) = mc_num::mldivide(&a, &b).unwrap();
        let ax = matmul(&a, &x);
        prop_assert!(max_diff(&ax, &b) < 1e-8, "n={n} residual={}", max_diff(&ax, &b));
    }

    #[test]
    fn det_multiplicative(seed in 0u64..10_000) {
        let n = 2 + (seed % 4) as usize;
        let a = dominant_matrix(n, seed);
        let b = dominant_matrix(n, seed.wrapping_mul(17) + 7);
        let det_ab = mc_num::det(&matmul(&a, &b)).unwrap();
        let det_a = mc_num::det(&a).unwrap();
        let det_b = mc_num::det(&b).unwrap();
        prop_assert!((det_ab - det_a * det_b).abs() <= 1e-8 * (1.0 + det_a.abs() * det_b.abs()));
    }

    #[test]
    fn inv_times_a_is_identity(seed in 0u64..10_000) {
        let n = 2 + (seed % 6) as usize;
        let a = dominant_matrix(n, seed);
        let inv = mc_num::inv(&a).unwrap();
        let eye = matmul(&inv, &a);
        let mut want = Matrix::full(n, n, 0.0);
        for i in 0..n { want.set(i, i, 1.0); }
        prop_assert!(max_diff(&eye, &want) < 1e-8);
    }

    #[test]
    fn qr_q_is_orthogonal(seed in 0u64..10_000) {
        let (m, n) = (3 + (seed % 4) as usize, 2 + (seed % 3) as usize);
        let mut a = Matrix::full(m, n, 0.0);
        let mut s = seed | 5;
        for c in 0..n { for r in 0..m {
            s ^= s << 13; s ^= s >> 7; s ^= s << 17;
            a.set(r, c, (s % 2000) as f64 / 1000.0 - 1.0);
        }}
        let (q, _) = mc_num::qr(&a).unwrap();
        let qqr = matmul(&transpose(&q), &q);
        let mut want = Matrix::full(q.cols(), q.cols(), 0.0);
        for i in 0..q.cols() { want.set(i, i, 1.0); }
        prop_assert!(max_diff(&qqr, &want) < 1e-9);
    }

    #[test]
    fn chol_reconstructs_spd(seed in 0u64..10_000) {
        // A = D·Dᵀ با D مثلثی پایین تصادفی → قطعاً SPD.
        let n = 2 + (seed % 5) as usize;
        let mut d = Matrix::full(n, n, 0.0);
        let mut s = seed | 9;
        for c in 0..n { for r in c..n {
            s ^= s << 13; s ^= s >> 7; s ^= s << 17;
            d.set(r, c, (s % 1000) as f64 / 1000.0);
        }}
        for i in 0..n { d.set(i, i, d.get(i, i) + 0.5); } // قطر ناصفر
        let a = matmul(&d, &transpose(&d));
        let l = mc_num::chol(&a).unwrap();
        let back = matmul(&l, &transpose(&l));
        prop_assert!(max_diff(&back, &a) < 1e-9);
    }

    #[test]
    fn svd_singular_values_descending_nonneg(seed in 0u64..10_000) {
        let (m, n) = (3 + (seed % 3) as usize, 2 + (seed % 3) as usize);
        let mut a = Matrix::full(m, n, 0.0);
        let mut s = seed | 11;
        for c in 0..n { for r in 0..m {
            s ^= s << 13; s ^= s >> 7; s ^= s << 17;
            a.set(r, c, (s % 2000) as f64 / 1000.0 - 1.0);
        }}
        let (_, sv, _) = mc_num::svd(&a).unwrap();
        prop_assert!(sv.iter().all(|&v| v >= 0.0));
        for w in sv.windows(2) {
            prop_assert!(w[0] >= w[1] - 1e-9, "not descending: {sv:?}");
        }
    }

    #[test]
    fn rref_of_invertible_is_identity(seed in 0u64..10_000) {
        let n = 2 + (seed % 6) as usize;
        let a = dominant_matrix(n, seed);
        let (r, pivots) = mc_num::rref(&a);
        let mut want = Matrix::full(n, n, 0.0);
        for i in 0..n { want.set(i, i, 1.0); }
        prop_assert!(max_diff(&r, &want) < 1e-9);
        prop_assert_eq!(pivots, (0..n).collect::<Vec<_>>());
    }

    #[test]
    fn transpose_involution(seed in 0u64..10_000) {
        let (m, n) = (3 + (seed % 3) as usize, 2 + (seed % 4) as usize);
        let mut a = Matrix::full(m, n, 0.0);
        let mut s = seed | 13;
        for c in 0..n { for r in 0..m {
            s ^= s << 13; s ^= s >> 7; s ^= s << 17;
            a.set(r, c, (s % 2000) as f64 / 1000.0 - 1.0);
        }}
        let tt = transpose(&transpose(&a));
        prop_assert!(max_diff(&tt, &a) < 1e-15);
    }

    #[test]
    fn rank_bounded_by_min_dim(seed in 0u64..10_000) {
        let (m, n) = (3 + (seed % 4) as usize, 2 + (seed % 4) as usize);
        let mut a = Matrix::full(m, n, 0.0);
        let mut s = seed | 15;
        for c in 0..n { for r in 0..m {
            s ^= s << 13; s ^= s >> 7; s ^= s << 17;
            a.set(r, c, (s % 2000) as f64 / 1000.0 - 1.0);
        }}
        prop_assert!(mc_num::rank(&a, None) <= m.min(n));
        prop_assert!(mc_num::rank(&a, None) >= 1, "random matrix almost surely full-rank-ish");
    }
}
