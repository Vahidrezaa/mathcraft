//! کورپوس طلای P1 — ۶۰ اسکریپت ماتریس/جبرخطی/سمانتیک (سند ۰۸ §2، توزیع هدفمند).
//!
//! منبع یگانه هر اسکریپت: `test-corpus/scripts/p1_*.m` — همین فایل‌ها در CI با
//! Octave هم اجرا می‌شوند (job octave-differential) و مقادیر مرجع `test-corpus/expected/`
//! با اوراکل مستقل numpy/LAPACK تولید شده‌اند (`scripts/gen_corpus.py`).
//! tolerance پیش‌فرض 1e-9 (سیاست مرکزی سند ۰۸ §8).

use mc_interp::{eval_program, Environment};
use mc_parser::parse;

/// اجرای اسکریپت و بازگرداندن مقدار اسکالر یک متغیر.
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

/// اجرای اسکریپت و بازگرداندن ماتریس به‌صورت Vec<Vec<f64>>.
fn run_mat(src: &str, var: &str) -> Vec<Vec<f64>> {
    let prog = parse(src).expect("parse");
    let mut env = Environment::new();
    let out = eval_program(&prog, &mut env);
    assert!(out.error.is_none(), "engine error: {:?}", out.error);
    match env.get(var) {
        Some(mc_core::Value::Double(m)) => {
            let mut rows = Vec::with_capacity(m.rows());
            for r in 0..m.rows() {
                rows.push((0..m.cols()).map(|c| m.get(r, c)).collect());
            }
            rows
        }
        other => panic!("'{var}' is not a matrix: {other:?}"),
    }
}

fn approx(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

fn assert_mat(got: &[Vec<f64>], want: &[&[f64]]) {
    assert_eq!(got.len(), want.len(), "row count");
    for (r, (g, w)) in got.iter().zip(want.iter()).enumerate() {
        assert_eq!(g.len(), w.len(), "col count in row {r}");
        for (c, (gv, wv)) in g.iter().zip(w.iter()).enumerate() {
            assert!(
                approx(*gv, *wv) || (gv.is_nan() && wv.is_nan()),
                "({r},{c}): got {gv}, want {wv}"
            );
        }
    }
}

macro_rules! src {
    ($f:literal) => {
        include_str!(concat!("../../../test-corpus/scripts/", $f))
    };
}

// ---------- ۰۱-۲۰: دستگاه معادلات، دکامپوزیسیون، کنترل جریان، سمانتیک حیاتی ----------

#[test]
fn corpus_01_linear_system_2x2() {
    assert_mat(
        &run_mat(src!("p1_01_linear_system_2x2.m"), "x"),
        &[&[-0.125], &[0.75]],
    );
}

#[test]
fn corpus_02_linear_system_5x5() {
    let r = run(src!("p1_02_linear_system_5x5.m"), "maxabs");
    assert!(r < 1e-9, "residual {r}");
}

#[test]
fn corpus_03_backslash_tridiagonal() {
    let r = run(src!("p1_03_backslash_tridiagonal.m"), "r");
    assert!(r < 1e-9, "residual {r}");
}

#[test]
fn corpus_04_overdetermined_exact() {
    let src = src!("p1_04_overdetermined_exact.m");
    assert_mat(&run_mat(src, "x"), &[&[1.0], &[2.0]]);
    assert!(run(src, "r") < 1e-9);
}

#[test]
fn corpus_05_underdetermined_min_norm() {
    assert_mat(
        &run_mat(src!("p1_05_underdetermined_min_norm.m"), "x"),
        &[&[1.0], &[2.0], &[0.0]],
    );
}

#[test]
fn corpus_06_rref_free_variable() {
    assert_mat(
        &run_mat(src!("p1_06_rref_free_variable.m"), "r"),
        &[&[1.0, 2.0, 3.0], &[0.0, 0.0, 0.0]],
    );
}

#[test]
fn corpus_07_det_and_inv() {
    let src = src!("p1_07_det_and_inv.m");
    // det([4 2;2 3]) = 12-4 = 8
    assert!(approx(run(src, "d"), 8.0));
    // inv([4 2;2 3]) = 1/8*[3 -2; -2 4]
    assert_mat(&run_mat(src, "I"), &[&[0.375, -0.25], &[-0.25, 0.5]]);
}

#[test]
fn corpus_08_eig_symmetric() {
    assert_mat(
        &run_mat(src!("p1_08_eig_symmetric.m"), "v"),
        &[&[1.0], &[3.0]],
    );
}

#[test]
fn corpus_09_chol_and_norm() {
    let src = src!("p1_09_chol_and_norm.m");
    // MATLAB chol is UPPER (R'*R = A), so R*R' - A has max-abs exactly 1.0 —
    // orientation proof: a lower-triangular chol would give err = 0 here.
    // Matches Octave 9.4/10.3 (differential job) and expected/p1_09 (err = 1.0).
    assert!(approx(run(src, "err"), 1.0));
    assert!(approx(run(src, "n"), 5.0));
}

#[test]
fn corpus_10_transpose_identity() {
    assert!(run(src!("p1_10_transpose_identity.m"), "err") < 1e-12);
}

#[test]
fn corpus_11_for_sum_squares() {
    assert!(approx(run(src!("p1_11_for_sum_squares.m"), "s"), 55.0));
}

#[test]
fn corpus_12_for_matrix_columns() {
    assert!(approx(run(src!("p1_12_for_matrix_columns.m"), "t"), 6.0));
}

#[test]
fn corpus_13_while_break() {
    let src = src!("p1_13_while_break.m");
    assert!(approx(run(src, "s"), 15.0));
    assert!(approx(run(src, "k"), 5.0));
}

#[test]
fn corpus_14_anon_capture() {
    assert!(approx(run(src!("p1_14_anon_capture.m"), "y"), 15.0));
}

#[test]
fn corpus_15_nested_anon_closure() {
    assert!(approx(run(src!("p1_15_nested_anon_closure.m"), "y"), 12.0));
}

#[test]
fn corpus_16_colon_semantics() {
    let src = src!("p1_16_colon_semantics.m");
    assert_eq!(run(src, "n") as usize, 11);
    assert!(approx(run(src, "last"), 1.0));
}

#[test]
fn corpus_17_ans_suppression() {
    // `1;` نه نمایش دارد نه ans را عوض می‌کند (سند ۰۳ §11 بند 3)
    let src = src!("p1_17_ans_suppression.m");
    let prog = parse(src).unwrap();
    let mut env = Environment::new();
    let _ = eval_program(&prog, &mut env);
    match env.get("ans") {
        Some(mc_core::Value::Double(m)) => assert!(approx(m.get(0, 0), 7.0)),
        _ => panic!("ans must stay 7"),
    }
}

#[test]
fn corpus_18_indexed_growth() {
    assert_mat(
        &run_mat(src!("p1_18_indexed_growth.m"), "x"),
        &[&[0.0], &[0.0], &[5.0]],
    );
}

#[test]
fn corpus_19_end_indexing() {
    let src = src!("p1_19_end_indexing.m");
    assert!(approx(run(src, "a"), 30.0));
    assert!(approx(run(src, "s"), 30.0));
}

#[test]
fn corpus_20_matrix_vs_elementwise_power() {
    let src = src!("p1_20_matrix_vs_elementwise_power.m");
    assert!(approx(run(src, "d1"), 29.0));
    assert!(approx(run(src, "d2"), 17.0));
}

// ---------- ۲۱-۳۰: توابع عنصری ----------

#[test]
fn corpus_21_elementwise_identities() {
    let s = src!("p1_21_elementwise_identities.m");
    assert!(approx(run(s, "y1"), 0.5));
    assert!(approx(run(s, "y2"), 0.5));
    assert!(approx(run(s, "y3"), 7.0));
    assert!(approx(run(s, "y4"), 4.0));
}

#[test]
fn corpus_22_trig_identity() {
    let s = src!("p1_22_trig_identity.m");
    assert!(approx(run(s, "s"), 1.0));
    assert!(approx(run(s, "t"), 1.0));
}

#[test]
fn corpus_23_inverse_trig() {
    let s = src!("p1_23_inverse_trig.m");
    assert!(approx(run(s, "a"), std::f64::consts::FRAC_PI_6));
    assert!(approx(run(s, "b"), std::f64::consts::FRAC_PI_3));
    assert!(approx(run(s, "c"), std::f64::consts::FRAC_PI_4));
}

#[test]
fn corpus_24_hyperbolic_identity() {
    let s = src!("p1_24_hyperbolic_identity.m");
    assert!(approx(run(s, "a"), 1.175201193643801));
    assert!(approx(run(s, "b"), 1.543080634815244));
    assert!(approx(run(s, "idd"), 1.0));
}

#[test]
fn corpus_25_rounding_family() {
    let s = src!("p1_25_rounding_family.m");
    // متلب: round نیم‌فاصله-دور-از-صفر؛ fix به سمت صفر
    assert_mat(&run_mat(s, "r"), &[&[3.0, -3.0, 3.0, -3.0]]);
    assert_mat(&run_mat(s, "f"), &[&[2.0, -2.0, 2.0, -2.0]]);
    assert_mat(&run_mat(s, "fl"), &[&[2.0, -3.0, 2.0, -3.0]]);
    assert_mat(&run_mat(s, "ce"), &[&[3.0, -2.0, 3.0, -2.0]]);
}

#[test]
fn corpus_26_sign_abs() {
    let s = src!("p1_26_sign_abs.m");
    assert_mat(&run_mat(s, "s"), &[&[-1.0, 0.0, 1.0]]);
    assert_mat(&run_mat(s, "a"), &[&[3.0, 4.0]]);
}

#[test]
fn corpus_27_mod_rem_signs() {
    // سمانتیک متلب: mod علامت مقسوم‌علیه، rem علامت مقسوم
    let s = src!("p1_27_mod_rem_signs.m");
    assert!(approx(run(s, "a"), 2.0));
    assert!(approx(run(s, "b"), -1.0));
    assert!(approx(run(s, "c"), -2.0));
    assert!(approx(run(s, "d"), 1.0));
}

#[test]
fn corpus_28_log_family() {
    let s = src!("p1_28_log_family.m");
    assert!(approx(run(s, "l1"), 3.0));
    assert!(approx(run(s, "l2"), 3.0));
    assert!(approx(run(s, "l3"), 2.0));
}

#[test]
fn corpus_29_sqrt_exp_elementwise() {
    let s = src!("p1_29_sqrt_exp_elementwise.m");
    assert_mat(&run_mat(s, "S"), &[&[1.0, 2.0], &[3.0, 4.0]]);
    let e = run_mat(s, "e");
    assert!(approx(e[0][0], 1.0) && approx(e[0][1], std::f64::consts::E));
}

#[test]
fn corpus_30_atan2_quadrants() {
    let s = src!("p1_30_atan2_quadrants.m");
    let p = std::f64::consts::PI;
    assert!(approx(run(s, "q1"), p / 4.0));
    assert!(approx(run(s, "q2"), 3.0 * p / 4.0));
    assert!(approx(run(s, "q3"), -3.0 * p / 4.0));
    assert!(approx(run(s, "q4"), -p / 4.0));
}

// ---------- ۳۱-۳۸: کاهشی/آمار پایه و NaN ----------

#[test]
fn corpus_31_reductions_vector() {
    let s = src!("p1_31_reductions_vector.m");
    assert!(approx(run(s, "s"), 8.0));
    assert!(approx(run(s, "p"), 12.0));
    assert!(approx(run(s, "m"), 4.0));
    assert!(approx(run(s, "mn"), 1.0));
    assert_mat(&run_mat(s, "cs"), &[&[4.0, 5.0, 8.0]]);
}

#[test]
fn corpus_32_reductions_matrix_dims() {
    let s = src!("p1_32_reductions_matrix_dims.m");
    assert_mat(&run_mat(s, "c"), &[&[4.0, 6.0]]);
    assert_mat(&run_mat(s, "r"), &[&[3.0], &[7.0]]);
    assert!(approx(run(s, "t"), 10.0));
}

#[test]
fn corpus_33_cumprod_diff() {
    let s = src!("p1_33_cumprod_diff.m");
    assert_mat(&run_mat(s, "cp"), &[&[1.0, 2.0, 6.0, 24.0]]);
    assert_mat(&run_mat(s, "d"), &[&[1.0, 1.0, 1.0]]);
}

#[test]
fn corpus_34_mean_all_any() {
    let s = src!("p1_34_mean_all_any.m");
    assert_mat(&run_mat(s, "m"), &[&[2.0, 3.0]]);
    assert_mat(&run_mat(s, "a1"), &[&[1.0, 0.0]]);
    assert_mat(&run_mat(s, "q"), &[&[0.0], &[1.0]]);
}

#[test]
fn corpus_35_find_column_major() {
    let s = src!("p1_35_find_column_major.m");
    assert_mat(&run_mat(s, "f"), &[&[2.0], &[3.0], &[6.0]]);
    assert!(approx(run(s, "s"), 11.0));
}

#[test]
fn corpus_36_sort_vector_flags() {
    let s = src!("p1_36_sort_vector_flags.m");
    assert_mat(&run_mat(s, "a"), &[&[1.0, 2.0, 3.0]]);
    assert_mat(&run_mat(s, "d"), &[&[3.0, 2.0, 1.0]]);
}

#[test]
fn corpus_37_sort_matrix_columns() {
    let s = src!("p1_37_sort_matrix_columns.m");
    assert_mat(&run_mat(s, "a"), &[&[1.0, 4.0], &[2.0, 5.0]]);
    assert_mat(&run_mat(s, "d"), &[&[2.0, 5.0], &[1.0, 4.0]]);
}

#[test]
fn corpus_38_nan_semantics() {
    // متلب: NaN در مرتب‌سازی آخر، در max/min نادیده، در sum منتشر می‌شود
    let s = src!("p1_38_nan_semantics.m");
    assert_mat(&run_mat(s, "s"), &[&[1.0, 2.0, 3.0, f64::NAN]]);
    assert!(approx(run(s, "mx"), 3.0));
    assert!(approx(run(s, "mn"), 1.0));
    assert!(run(s, "sm").is_nan());
}

// ---------- ۳۹-۴۶: سازه‌ها و شکل ----------

#[test]
fn corpus_39_constructors() {
    let s = src!("p1_39_constructors.m");
    assert!(approx(run(s, "zsum"), 0.0));
    assert!(approx(run(s, "osum"), 3.0));
    assert!(approx(run(s, "esum"), 2.0));
}

#[test]
fn corpus_40_linspace_logspace() {
    let s = src!("p1_40_linspace_logspace.m");
    assert_mat(&run_mat(s, "l"), &[&[0.0, 0.25, 0.5, 0.75, 1.0]]);
    assert!(approx(run(s, "n"), 5.0));
    assert_mat(&run_mat(s, "lg"), &[&[1.0, 10.0, 100.0, 1000.0]]);
}

#[test]
fn corpus_41_colon_steps() {
    let s = src!("p1_41_colon_steps.m");
    assert_mat(&run_mat(s, "a"), &[&[5.0, 4.0, 3.0, 2.0, 1.0]]);
    assert_mat(&run_mat(s, "b"), &[&[0.0, 0.5, 1.0, 1.5, 2.0]]);
    assert!(approx(run(s, "n"), 11.0));
}

#[test]
fn corpus_42_diag_modes() {
    let s = src!("p1_42_diag_modes.m");
    assert_mat(&run_mat(s, "m"), &[&[2.0, 0.0], &[0.0, 3.0]]);
    assert_mat(&run_mat(s, "dd"), &[&[2.0], &[3.0]]);
}

#[test]
fn corpus_43_tril_triu() {
    let s = src!("p1_43_tril_triu.m");
    assert_mat(&run_mat(s, "L"), &[&[1.0, 0.0], &[3.0, 4.0]]);
    assert_mat(&run_mat(s, "U"), &[&[1.0, 2.0], &[0.0, 4.0]]);
    // tril+triu قطر را دوبار می‌شمارد → L+U-A = diag(diag(A))؛ جمع = 1+4 = 5
    assert!(approx(run(s, "zs"), 5.0));
}

#[test]
fn corpus_44_reshape_repmat() {
    let s = src!("p1_44_reshape_repmat.m");
    // reshape ستون‌محور است (سند ۰۳ §4)
    assert_mat(&run_mat(s, "A"), &[&[1.0, 3.0, 5.0], &[2.0, 4.0, 6.0]]);
    assert!(approx(run(s, "t"), 15.0));
    assert_mat(
        &run_mat(s, "R"),
        &[&[1.0, 2.0, 1.0, 2.0], &[1.0, 2.0, 1.0, 2.0]],
    );
}

#[test]
fn corpus_45_size_length_numel() {
    let s = src!("p1_45_size_length_numel.m");
    assert_mat(&run_mat(s, "s"), &[&[2.0, 3.0]]);
    assert!(approx(run(s, "l"), 3.0));
    assert!(approx(run(s, "n"), 6.0));
    assert!(approx(run(s, "q"), 0.0));
}

#[test]
fn corpus_46_transpose_forms() {
    let s = src!("p1_46_transpose_forms.m");
    assert_mat(&run_mat(s, "B"), &[&[1.0, 3.0], &[2.0, 4.0]]);
    // روی حقیقی ' و .' برابرند
    assert!(run(s, "err") < 1e-12);
}

// ---------- ۴۷-۵۸: جبر خطی ----------

#[test]
fn corpus_47_det_inv_diagonal() {
    let s = src!("p1_47_det_inv_diagonal.m");
    assert!(approx(run(s, "d"), 24.0));
    assert_mat(
        &run_mat(s, "I"),
        &[&[0.5, 0.0, 0.0], &[0.0, 1.0 / 3.0, 0.0], &[0.0, 0.0, 0.25]],
    );
    assert!(approx(run(s, "t"), 9.0));
}

#[test]
fn corpus_48_det_upper_triangular() {
    let s = src!("p1_48_det_upper_triangular.m");
    assert!(approx(run(s, "d"), 24.0));
    assert!(approx(run(s, "d2"), 24.0));
}

#[test]
fn corpus_49_rank_trace() {
    let s = src!("p1_49_rank_trace.m");
    assert!(approx(run(s, "rk"), 1.0));
    assert!(approx(run(s, "rk2"), 4.0));
    assert!(approx(run(s, "t"), 5.0));
}

#[test]
fn corpus_50_rref_two_cases() {
    let s = src!("p1_50_rref_two_cases.m");
    assert_mat(&run_mat(s, "R"), &[&[1.0, 2.0, 3.0], &[0.0, 0.0, 0.0]]);
    assert_mat(&run_mat(s, "R2"), &[&[1.0, 0.0], &[0.0, 1.0]]);
}

#[test]
fn corpus_51_solve_exact_2x2() {
    let s = src!("p1_51_solve_exact_2x2.m");
    assert_mat(&run_mat(s, "x"), &[&[2.0], &[3.0]]);
    assert!(run(s, "res") < 1e-9);
}

#[test]
fn corpus_52_solve_overdetermined() {
    let s = src!("p1_52_solve_overdetermined.m");
    assert_mat(&run_mat(s, "x"), &[&[1.0], &[2.0]]);
    assert!(run(s, "r") < 1e-9);
}

#[test]
fn corpus_53_least_squares_general() {
    // کمینه-مربعات مرجع: A'A=[[3,3],[3,5]]، A'b=[6,0] → x=[5;-3]، ‖r‖=sqrt(6)
    let s = src!("p1_53_least_squares_general.m");
    assert_mat(&run_mat(s, "x"), &[&[5.0], &[-3.0]]);
    assert!(approx(run(s, "rn"), 2.449489742783178));
}

#[test]
fn corpus_54_triangular_solve() {
    let s = src!("p1_54_triangular_solve.m");
    assert_mat(&run_mat(s, "x"), &[&[1.0], &[2.0]]);
    assert!(run(s, "chk") < 1e-9);
}

#[test]
fn corpus_55_chol_reconstruction() {
    let s = src!("p1_55_chol_reconstruction.m");
    // err = 1.0 proves the UPPER-triangular MATLAB convention (see corpus_09);
    // l11 = R(1,1) = 2 (same for both orientations, err is the discriminator).
    assert!(approx(run(s, "err"), 1.0));
    assert!(approx(run(s, "l11"), 2.0));
}

#[test]
fn corpus_56_eig_properties() {
    let s = src!("p1_56_eig_properties.m");
    assert_mat(&run_mat(s, "v"), &[&[1.0], &[3.0]]);
    assert!(approx(run(s, "sumv"), 4.0));
}

#[test]
fn corpus_57_svd_values() {
    let s = src!("p1_57_svd_values.m");
    assert_mat(&run_mat(s, "s"), &[&[3.0], &[2.0]]);
    // singular values مرجع [4 11 14; 8 7 -2] (اوراکل LAPACK)
    assert!(approx(run(s, "s1"), 18.973665961010127));
    assert!(approx(run(s, "s2"), 9.486832980505138));
}

#[test]
fn corpus_58_qr_upper_factor() {
    let s = src!("p1_58_qr_upper_factor.m");
    // R برای [1 2; 3 4]: r11=-sqrt(10)، r21=0، r22=-2/sqrt(10)
    assert!(approx(run(s, "r11"), -3.162277660168379));
    assert!(approx(run(s, "r21"), 0.0));
    assert!(approx(run(s, "r22"), -0.6324555320336759));
}

// ---------- ۵۹-۶۰: هندسی/نُرم‌ها ----------

#[test]
fn corpus_59_dot_cross_kron() {
    let s = src!("p1_59_dot_cross_kron.m");
    assert!(approx(run(s, "d"), 32.0));
    assert_mat(&run_mat(s, "c"), &[&[0.0], &[0.0], &[1.0]]);
    assert_mat(&run_mat(s, "k"), &[&[10.0, 100.0, 20.0, 200.0]]);
}

#[test]
fn corpus_60_cond_and_norms() {
    let s = src!("p1_60_cond_and_norms.m");
    assert!(approx(run(s, "c1"), 4.0));
    assert!(approx(run(s, "ci"), 1.0));
    assert!(approx(run(s, "n1"), 7.0));
    assert!(approx(run(s, "ni"), 4.0));
    assert!(approx(run(s, "nf"), 5.477225575051661));
}
