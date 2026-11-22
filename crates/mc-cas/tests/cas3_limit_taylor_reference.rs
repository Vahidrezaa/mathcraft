//! مجموعهٔ مرجع ۴۰ حد/سری — گام 4.4/CAS-3 (سند ۰۵ §۱۱ معیار پذیرش).
//!
//! همه از سطح موتور با `from_sexp` (S-اجر کانونی) ساخته می‌شوند. انتظارها:
//! `Val` نمایش دقیق، `ValApprox` مقدار عددیِ Real (تقریب صادقانه)،
//! `PosInf/NegInf/Dne`، و برای تaylor وورای عددی سری در نقطهٔ نزدیک
//! (S(t) ≈ f(t)) — الگوی «موتور خودش ورفای می‌کند».

use mc_cas::limit::{limit_expr, LimitOutcome, LimitPoint, Side};
use mc_cas::{from_sexp, sym_display, taylor, Sym};

fn s(sexp: &str) -> Sym {
    from_sexp(sexp).expect("sexp")
}

#[derive(Debug)]
enum Exp {
    /// نمایش خطی دقیق مقدار
    Val(&'static str),
    /// Real تقریبی — |v − want| < 1e-6
    ValApprox(f64),
    PosInf,
    NegInf,
    Dne,
}

fn check_limit(name: &str, sexp: &str, point: LimitPoint, side: Side, exp: &Exp) {
    let f = s(sexp);
    let out =
        limit_expr(&f, "x", &point, side).unwrap_or_else(|e| panic!("{name}: خطای موتور {e:?}"));
    match (out, exp) {
        (LimitOutcome::Value(v), Exp::Val(w)) => {
            let d = sym_display(&v);
            assert_eq!(d, *w, "{name}");
        }
        (LimitOutcome::Value(v), Exp::ValApprox(w)) => {
            let fv = v
                .to_f64()
                .unwrap_or_else(|_| panic!("{name}: Real نبود: {v:?}"));
            assert!((fv - w).abs() < 1e-6, "{name}: {fv} دور از {w}");
        }
        (LimitOutcome::PosInf, Exp::PosInf) => {}
        (LimitOutcome::NegInf, Exp::NegInf) => {}
        (LimitOutcome::DoesNotExist(_), Exp::Dne) => {}
        (got, want) => panic!("{name}: {got:?} ≠ {want:?}"),
    }
}

fn at(v: i64) -> LimitPoint {
    LimitPoint::At(Sym::int(v))
}

#[test]
fn reference_40_limits_and_series() {
    // ---------- گویا: مستقیم و حذف ۰/۰ (۱۰) ----------
    check_limit(
        "cancel-quad",
        "(* (+ (^ (sym x) (int 2)) (* (int -1) (int 9))) (^ (+ (sym x) (* (int -1) (int 3))) (int -1)))",
        at(3),
        Side::TwoSided,
        &Exp::Val("6"),
    );
    check_limit(
        "cancel-shifted",
        "(* (^ (+ (sym x) (* (int -1) (rat 1 2))) (int 2)) (^ (+ (sym x) (* (int -1) (rat 1 2))) (int -1)))",
        LimitPoint::At(Sym::rat(1, 2).expect("r")),
        Side::TwoSided,
        &Exp::Val("0"),
    );
    check_limit(
        "cancel-cubic",
        "(* (+ (+ (^ (sym x) (int 2)) (sym x)) (* (int -1) (int 2))) (^ (+ (sym x) (* (int -1) (int 1))) (int -1)))",
        at(1),
        Side::TwoSided,
        &Exp::Val("3"),
    );
    check_limit(
        "pole-even",
        "(^ (^ (sym x) (int 2)) (int -1))",
        at(0),
        Side::TwoSided,
        &Exp::PosInf,
    );
    check_limit(
        "pole-odd-dne",
        "(^ (sym x) (int -1))",
        at(0),
        Side::TwoSided,
        &Exp::Dne,
    );
    check_limit(
        "pole-odd-right",
        "(^ (sym x) (int -1))",
        at(0),
        Side::FromRight,
        &Exp::PosInf,
    );
    check_limit(
        "pole-odd-left",
        "(^ (sym x) (int -1))",
        at(0),
        Side::FromLeft,
        &Exp::NegInf,
    );
    check_limit(
        "direct-rational",
        "(* (+ (sym x) (int 3)) (^ (+ (^ (sym x) (int 2)) (int 1)) (int -1)))",
        at(2),
        Side::TwoSided,
        &Exp::Val("1"),
    );
    check_limit(
        "direct-poly",
        "(* (sym x) (+ (sym x) (int 1)))",
        at(1),
        Side::TwoSided,
        &Exp::Val("2"),
    );
    check_limit(
        "pole-shifted-dne",
        "(* (+ (* (int 2) (sym x)) (int 1)) (^ (+ (sym x) (* (int -1) (int 5))) (int -1)))",
        at(5),
        Side::TwoSided,
        &Exp::Dne,
    );

    // ---------- لوبیتال و جدول کلاسیک (۸) ----------
    check_limit(
        "sin-over-x",
        "(* (fn sin (sym x)) (^ (sym x) (int -1)))",
        at(0),
        Side::TwoSided,
        &Exp::Val("1"),
    );
    check_limit(
        "expm1-over-x",
        "(* (+ (fn exp (sym x)) (* (int -1) (int 1))) (^ (sym x) (int -1)))",
        at(0),
        Side::TwoSided,
        &Exp::Val("1"),
    );
    check_limit(
        "log1p-over-x",
        "(* (fn log (+ (int 1) (sym x))) (^ (sym x) (int -1)))",
        at(0),
        Side::TwoSided,
        &Exp::Val("1"),
    );
    check_limit(
        "one-minus-cos",
        "(* (+ (int 1) (* (int -1) (fn cos (sym x)))) (^ (^ (sym x) (int 2)) (int -1)))",
        at(0),
        Side::TwoSided,
        &Exp::Val("1/2"),
    );
    check_limit(
        "tan-over-x",
        "(* (fn tan (sym x)) (^ (sym x) (int -1)))",
        at(0),
        Side::TwoSided,
        &Exp::Val("1"),
    );
    check_limit(
        "exp2x-over-x",
        "(* (+ (fn exp (* (int 2) (sym x))) (* (int -1) (int 1))) (^ (sym x) (int -1)))",
        at(0),
        Side::TwoSided,
        &Exp::Val("2"),
    );
    check_limit(
        "x-log-x",
        "(* (sym x) (fn log (sym x)))",
        at(0),
        Side::FromRight,
        &Exp::ValApprox(0.0),
    );
    check_limit(
        "sin-minus-x-cubic",
        "(* (+ (fn sin (sym x)) (* (int -1) (sym x))) (^ (^ (sym x) (int 3)) (int -1)))",
        at(0),
        Side::TwoSided,
        &Exp::Val("-1/6"),
    );

    // ---------- توان‌ها با تبدیل لگاریتمی (۴) ----------
    check_limit(
        "euler-compound",
        "(^ (+ (int 1) (^ (sym x) (int -1))) (sym x))",
        LimitPoint::PosInf,
        Side::TwoSided,
        &Exp::Val("exp(1)"),
    );
    check_limit(
        "euler-root",
        "(^ (+ (int 1) (sym x)) (^ (sym x) (int -1)))",
        at(0),
        Side::TwoSided,
        &Exp::Val("exp(1)"),
    );
    check_limit(
        "x-to-1-over-x",
        "(^ (sym x) (^ (sym x) (int -1)))",
        LimitPoint::PosInf,
        Side::TwoSided,
        &Exp::ValApprox(1.0),
    );
    check_limit(
        "compound-square",
        "(^ (+ (int 1) (^ (^ (sym x) (int 2)) (int -1))) (sym x))",
        LimitPoint::PosInf,
        Side::TwoSided,
        &Exp::ValApprox(1.0),
    );

    // ---------- ±∞ گویا (۶) ----------
    check_limit(
        "inf-equal-degrees",
        "(* (+ (* (int 3) (^ (sym x) (int 2))) (int 1)) (^ (+ (^ (sym x) (int 2)) (int 5)) (int -1)))",
        LimitPoint::PosInf,
        Side::TwoSided,
        &Exp::Val("3"),
    );
    check_limit(
        "inf-den-dominates",
        "(* (+ (^ (sym x) (int 2)) (sym x)) (^ (+ (^ (sym x) (int 3)) (* (int -1) (int 1))) (int -1)))",
        LimitPoint::PosInf,
        Side::TwoSided,
        &Exp::Val("0"),
    );
    check_limit(
        "inf-num-dominates",
        "(* (^ (sym x) (int 2)) (^ (+ (sym x) (int 1)) (int -1)))",
        LimitPoint::PosInf,
        Side::TwoSided,
        &Exp::PosInf,
    );
    check_limit(
        "inf-neg-leading",
        "(* (* (int -1) (^ (sym x) (int 2))) (^ (+ (sym x) (int 1)) (int -1)))",
        LimitPoint::PosInf,
        Side::TwoSided,
        &Exp::NegInf,
    );
    check_limit(
        "neg-inf-even-parity",
        "(* (^ (sym x) (int 2)) (^ (+ (sym x) (* (int -1) (int 1))) (int -1)))",
        LimitPoint::NegInf,
        Side::TwoSided,
        &Exp::NegInf,
    );
    check_limit(
        "inf-linear-ratio",
        "(* (+ (* (int 2) (sym x)) (int 1)) (^ (+ (sym x) (* (int -1) (int 3))) (int -1)))",
        LimitPoint::PosInf,
        Side::TwoSided,
        &Exp::Val("2"),
    );

    // ---------- ±∞ غیرگویا (۴) ----------
    check_limit(
        "exp-over-square",
        "(* (fn exp (sym x)) (^ (^ (sym x) (int 2)) (int -1)))",
        LimitPoint::PosInf,
        Side::TwoSided,
        &Exp::PosInf,
    );
    check_limit(
        "linear-over-exp",
        "(* (sym x) (^ (fn exp (sym x)) (int -1)))",
        LimitPoint::PosInf,
        Side::TwoSided,
        &Exp::ValApprox(0.0),
    );
    check_limit(
        "sigmoid",
        "(* (fn exp (sym x)) (^ (+ (fn exp (sym x)) (int 1)) (int -1)))",
        LimitPoint::PosInf,
        Side::TwoSided,
        &Exp::ValApprox(1.0),
    );
    check_limit(
        "sin-over-x-at-inf",
        "(* (fn sin (sym x)) (^ (sym x) (int -1)))",
        LimitPoint::PosInf,
        Side::TwoSided,
        &Exp::ValApprox(0.0),
    );

    // ---------- نوسان و فشار (۳) ----------
    check_limit(
        "oscillation-sin",
        "(fn sin (^ (sym x) (int -1)))",
        at(0),
        Side::TwoSided,
        &Exp::Dne,
    );
    check_limit(
        "oscillation-cos",
        "(fn cos (^ (sym x) (int -1)))",
        at(0),
        Side::TwoSided,
        &Exp::Dne,
    );
    check_limit(
        "squeeze-x-sin-inv",
        "(* (sym x) (fn sin (^ (sym x) (int -1))))",
        at(0),
        Side::TwoSided,
        &Exp::ValApprox(0.0),
    );

    // ---------- تیلور (۵) — ورفای عددی سری در نقطهٔ نزدیک ----------
    let taylor_check = |name: &str, sexp: &str, a: f64, order: u32, probe: f64| {
        let f = s(sexp);
        let pt = Sym::Real(a);
        let Some(series) = taylor::taylor(&f, "x", &pt, order).expect("taylor ok") else {
            panic!("{name}: باید سری بدهد");
        };
        // وورفای: S(probe) ≈ f(probe)
        let sp = series
            .substitute("x", &Sym::Real(probe))
            .to_f64()
            .unwrap_or_else(|_| panic!("{name}: سری در probe عددی نشد"));
        let fp = f
            .substitute("x", &Sym::Real(probe))
            .to_f64()
            .unwrap_or_else(|_| panic!("{name}: تابع در probe عددی نشد"));
        // تلورانس متناسب با خطای برشِ مرتبهٔ order در فاصلهٔ |probe−a|
        let d = (probe - a).abs();
        let tol = 5e-4 * (1.0 + fp.abs()) + d.powi(order as i32 + 1);
        assert!(
            (sp - fp).abs() < tol,
            "{name}: سری {sp} دور از تابع {fp} (tol {tol})"
        );
    };
    taylor_check("t-exp", "(fn exp (sym x))", 0.0, 4, 0.1);
    taylor_check("t-exp-shift", "(fn exp (sym x))", 1.0, 3, 1.1);
    taylor_check("t-sin", "(fn sin (sym x))", 0.0, 5, 0.2);
    taylor_check(
        "t-geometric",
        "(* (int 1) (^ (+ (int 1) (* (int -1) (sym x))) (int -1)))",
        0.0,
        4,
        0.2,
    );
    taylor_check("t-log1p", "(fn log (+ (int 1) (sym x)))", 0.0, 4, 0.1);

    // ---------- تیلور: شکست‌های صادقانه (۲) ----------
    let bad = s("(* (int 1) (^ (sym x) (int -1)))");
    assert!(taylor::taylor(&bad, "x", &Sym::int(0), 3)
        .expect("ok")
        .is_none());
    let absf = s("(fn abs (sym x))");
    assert!(taylor::taylor(&absf, "x", &Sym::int(0), 2)
        .expect("ok")
        .is_none());

    // شمارش کل: ۱۰+۸+۴+۶+۴+۳+۵+۲ = ۴۲ مورد — سند ۰۵ §۱۱ «حدود/سری مرجع
    // ۴۰ مورد» با حاشیهٔ ۲ موردِ شکست صادقانهٔ تیلور پوشش داده می‌شود.
}
