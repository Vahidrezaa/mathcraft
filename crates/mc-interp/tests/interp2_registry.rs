//! interp2 — درون‌یابی دوبعدی linear/nearest با سمانتیک متلب (ROADMAP 2.6، تسک ۲).
//!
//! پوشش لایهٔ رجیستری: فرم ۵-آرگومان / فرم ۳-آرگومان، گسترش meshgrid
//! (Xq سطری + Yq ستونی)، اکید-یکنوایی محورها، NaN بیرون بازه، 'extrap'.
//! مرجع‌ها تحلیلی‌اند (صفحهٔ خطی V = 10x + 2y — دوبعدیِ خطی دقیق است).

use mc_core::{Matrix, Value};
use mc_interp::{eval_program, Environment};
use mc_parser::parse;

fn eval_script(src: &str) -> Environment {
    let prog = parse(src).expect("parse");
    let mut env = Environment::new();
    let out = eval_program(&prog, &mut env);
    assert!(out.error.is_none(), "engine error: {:?}", out.error);
    env
}

fn mat(env: &Environment, name: &str) -> Matrix {
    match env.get(name) {
        Some(Value::Double(m)) => m.clone(),
        other => panic!("'{name}' is not a double matrix: {other:?}"),
    }
}

fn num(env: &Environment, name: &str) -> f64 {
    mat(env, name).get(0, 0)
}

#[test]
fn interp2_five_arg_form_matches_analytic_plane() {
    // V = 10x + 2y — دوبعدیِ خطی روی تابع خطی «دقیق» است
    let env = eval_script(
        "x = [0 1 2];\n\
         y = [0 1];\n\
         V = [0 10 20; 2 12 22];\n\
         q = interp2(x, y, V, 1.5, 0.5);\n\
         nodes = interp2(x, y, V, [1 2], [0 1]);\n",
    );
    assert!((num(&env, "q") - 16.0).abs() < 1e-12, "10*1.5 + 2*0.5 = 16");
    let nodes = mat(&env, "nodes");
    assert!((nodes.get_linear(0) - 10.0).abs() < 1e-12);
    assert!((nodes.get_linear(1) - 22.0).abs() < 1e-12);
}

#[test]
fn interp2_three_arg_form_implicit_axes() {
    // interp2(V, Xq, Yq) — X=1:n، Y=1:m
    let env = eval_script(
        "V = [1 2; 3 4];\n\
         q = interp2(V, 1.5, 1.5);\n\
         node = interp2(V, 2, 1);\n",
    );
    // V(x,y) با X=1:2، Y=1:2 → (1.5,1.5) = 2.5
    assert!((num(&env, "q") - 2.5).abs() < 1e-12);
    // node (x=2,y=1) = V(1,2) = 2 — دقت کن: y=1 سطر اول
    assert!((num(&env, "node") - 2.0).abs() < 1e-12);
}

#[test]
fn interp2_meshgrid_expansion_row_col() {
    let env = eval_script(
        "x = [0 1 2];\n\
         y = [0 1]';\n\
         V = [0 10 20; 2 12 22];\n\
         Q = interp2(x, y, V, [0.5 1.5], y);\n\
         sz = size(Q);\n",
    );
    let q = mat(&env, "Q");
    assert_eq!((q.rows(), q.cols()), (2, 2), "meshgrid expansion shape");
    // Q(1,1) = V در (0.5, 0) = 5؛ Q(1,2) = V در (1.5, 0) = 15
    assert!((q.get(0, 0) - 5.0).abs() < 1e-12);
    assert!((q.get(0, 1) - 15.0).abs() < 1e-12);
    // Q(2,1) = V در (0.5, 1) = 7؛ Q(2,2) = V در (1.5, 1) = 17
    assert!((q.get(1, 0) - 7.0).abs() < 1e-12);
    assert!((q.get(1, 1) - 17.0).abs() < 1e-12);
    let sz = mat(&env, "sz");
    assert_eq!((sz.get_linear(0), sz.get_linear(1)), (2.0, 2.0));
}

#[test]
fn interp2_outside_is_nan_unless_extrap() {
    let env = eval_script(
        "x = [0 1];\n\
         y = [0 1];\n\
         V = [0 10; 2 12];\n\
         n1 = interp2(x, y, V, 1.5, 0.5);\n\
         e1 = interp2(x, y, V, 1.5, 0.5, 'extrap');\n\
         e2 = interp2(x, y, V, 1.5, 0.5, 99);\n",
    );
    assert!(num(&env, "n1").is_nan(), "outside → NaN by default");
    // برون‌یابی خطی صفحه: 10*1.5 + 2*0.5 = 16
    assert!((num(&env, "e1") - 16.0).abs() < 1e-12);
    assert_eq!(num(&env, "e2"), 99.0, "extrapval scalar");
}

#[test]
fn interp2_nearest_method() {
    let env = eval_script(
        "x = [0 1];\n\
         y = [0 1];\n\
         V = [1 2; 3 4];\n\
         q = interp2(x, y, V, 0.75, 0.25, 'nearest');\n",
    );
    // نزدیک‌ترین گره: x→1، y→0 → V(1,2) = 2
    assert!((num(&env, "q") - 2.0).abs() < 1e-12);
}

#[test]
fn interp2_decreasing_axes_like_matlab() {
    let env = eval_script(
        "x = [2 1 0];\n\
         y = [1 0];\n\
         V = [22 12 2; 20 10 0];\n\
         q = interp2(x, y, V, 0.5, 0.25);\n",
    );
    assert!((num(&env, "q") - 5.5).abs() < 1e-12, "same as ascending");
}

#[test]
fn interp2_errors_are_honest() {
    for (src, what) in [
        // V ابعاد غلط
        (
            "x = [0 1]; y = [0 1]; V = [1 2]; q = interp2(x, y, V, 0.5, 0.5);",
            "V size",
        ),
        // محور غیریکنوا
        (
            "x = [0 2 1]; y = [0 1]; V = [1 2 3; 4 5 6]; q = interp2(x, y, V, 0.5, 0.5);",
            "monotonic X",
        ),
        // روش ناشناخته
        (
            "x = [0 1]; y = [0 1]; V = [1 2; 3 4]; q = interp2(x, y, V, 0.5, 0.5, 'cubic');",
            "cubic honest error",
        ),
        // Xq/Yq ناهم‌شکل (ستونی + سطری — نه هم‌شکل، نه قاعدهٔ سطری+ستونی)
        (
            "x = [0 1]; y = [0 1]; V = [1 2; 3 4]; q = interp2(x, y, V, [0.5; 0.75], [0.5 0.75]);",
            "query shape",
        ),
    ] {
        let prog = parse(src).expect(what);
        let mut env = Environment::new();
        let out = eval_program(&prog, &mut env);
        assert!(
            out.error.is_some(),
            "expected honest error for {what}, got {out:?}"
        );
    }
}

#[test]
fn interp2_accepts_plaid_matrices() {
    // ماتریس‌های plaid مثل خروجی meshgrid (تابع meshgrid هنوز builtin نیست —
    // docs/03 §12؛ اینجا معادل literal آن را می‌سازیم)
    let env = eval_script(
        "Xg = [0 1 2; 0 1 2; 0 1 2];\n\
         Yg = [0 0 0; 1 1 1; 2 2 2];\n\
         V = [0 10 20; 2 12 22; 4 14 24];\n\
         q = interp2(Xg, Yg, V, 0.5, 0.5);\n",
    );
    assert!(
        (num(&env, "q") - 6.0).abs() < 1e-12,
        "plaid X/Y must work like vectors"
    );
}
