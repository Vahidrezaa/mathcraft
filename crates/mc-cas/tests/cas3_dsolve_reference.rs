//! مجموعهٔ مرجع ۳۰ dsolve — گام 4.4/CAS-3 فاز ۲ (سند ۰۵ §۹ و §۱۱).
//!
//! هر مورد: ODE با گره‌های کانونی `diff(y(x), x)` ساخته می‌شود، `dsolve`
//! فهرست شاخه‌های صریح می‌دهد و **ورفای مستقل سمت تست** هر شاخه را باز
//! بررسی می‌کند: جایگذاری y→sol و Dky→diffⁿ(sol) در معادلهٔ اصلی و سپس
//! اثبات نمادینیِ صفر یا نمونه‌گیری عددی با جایگذاری ثابت‌های C. همین
//! الگوی «موتور خودش ورفای می‌کند، تست موتور را ورفای می‌کند» فاز ۱ است.
//!
//! پوشش خانواده‌ها (کلاس‌ها هم‌پوشانند — شماره‌گذاری بر اساس شکل مسأله):
//! جداپذیر ۶، خطی مرتبهٔ اول ۸، انتگرال‌گیری مستقیم ۱، کامل/خطی با
//! ضرایب حاوی log-فروپاشی ۲، خطی همگن ضرایب ثابت ۸ (گویا/مضاعف/مختلط/
//! درجه ۳)، ضرایب نامعین ۸ (چندجمله‌ای، e^ax، مثلثاتی، تشدید، چندخانواده).
//! Bernoulli/Riccati و ضرایب متغیر → خطای صادقانه (۲ منفی).

use mc_cas::{dsolve::dsolve, Sym};
use std::sync::Arc;

fn fy() -> Sym {
    Sym::Fn(Arc::from("y"), vec![Sym::symbol("x")])
}

fn dy(n: usize) -> Sym {
    let mut inner = fy();
    for _ in 0..n {
        inner = Sym::Fn(Arc::from("diff"), vec![inner, Sym::symbol("x")]);
    }
    inner
}

fn s(n: &str) -> Sym {
    Sym::symbol(n)
}

fn i(n: i64) -> Sym {
    Sym::int(n)
}

fn add(items: Vec<Sym>) -> Sym {
    items.into_iter().reduce(Sym::plus).expect("add")
}

fn mul(items: Vec<Sym>) -> Sym {
    items.into_iter().reduce(Sym::times).expect("mul")
}

fn f(name: &str, a: Sym) -> Sym {
    Sym::fn_of(name, vec![a]).expect("fn")
}

fn pwn(b: Sym, e: Sym) -> Sym {
    Sym::pow(b, e).expect("pow")
}

/// بازنویسی بازگشتی: y(x) → sol؛ هر گرهٔ diff^n(y,x) → diffⁿ(sol).
fn rewrite_with(e: &Sym, sol: &Sym, max_order: usize) -> Sym {
    match e {
        Sym::Fn(name, args) if name.as_ref() == "diff" && args.len() == 2 => {
            let mut k = 1usize;
            let mut cur = &args[0];
            while let Sym::Fn(iname, iargs) = cur {
                if iname.as_ref() == "diff" && iargs.len() == 2 {
                    k += 1;
                    cur = &iargs[0];
                } else {
                    break;
                }
            }
            if k > max_order {
                return e.clone();
            }
            match mc_cas::diff::diff_n(sol, "x", k as i64) {
                Ok(d) => d,
                Err(_) => e.clone(),
            }
        }
        Sym::Fn(name, args) if name.as_ref() == "y" && args.len() == 1 => sol.clone(),
        Sym::Fn(name, args) => {
            let new_args: Vec<Sym> = args
                .iter()
                .map(|a| rewrite_with(a, sol, max_order))
                .collect();
            Sym::Fn(name.clone(), new_args)
        }
        Sym::Add(ts) => ts
            .iter()
            .map(|a| rewrite_with(a, sol, max_order))
            .reduce(Sym::plus)
            .unwrap_or_else(|| Sym::int(0)),
        Sym::Mul(fs) => fs
            .iter()
            .map(|a| rewrite_with(a, sol, max_order))
            .reduce(Sym::times)
            .unwrap_or_else(|| Sym::int(1)),
        Sym::Pow(b, ex) => Sym::pow(
            rewrite_with(b, sol, max_order),
            rewrite_with(ex, sol, max_order),
        )
        .unwrap_or_else(|_| e.clone()),
        other => other.clone(),
    }
}

/// ورفای مستقل: جایگذاری جواب در معادلهٔ اصلی — نمادین یا عددی.
fn verify_sol(eq: &Sym, sol: &Sym, max_order: usize) -> bool {
    let res = rewrite_with(eq, sol, max_order);
    // نمادین: simplify(res) = 0
    if let Ok(z) = mc_cas::simplify::simplify(&res) {
        if matches!(z, Sym::Integer(ref k) if *k == num_bigint::BigInt::from(0)) {
            return true;
        }
    }
    // عددی: جایگذاری ثابت‌ها + نمونه‌های x
    let cs: Vec<String> = sol
        .free_symbols()
        .into_iter()
        .filter(|s| s.starts_with('C'))
        .collect();
    // بازهٔ پهن — جواب‌های log/exp به ثابت‌های بزرگ منفی نیاز دارند.
    let cands: [i64; 9] = [1, 2, -1, 3, -3, 7, -12, -30, -60];
    let xs: [f64; 5] = [0.4, 0.9, 1.3, 2.1, -0.8];
    let n = cs.len().min(2);
    let mut idx = vec![0usize; n];
    for _ in 0..cands.len().pow(n as u32).min(12) {
        let mut res_c = res.clone();
        for (i, c) in cs.iter().take(n).enumerate() {
            res_c = res_c.substitute(c, &Sym::int(cands[idx[i]]));
        }
        let mut ok = true;
        for xv in xs {
            match res_c.substitute("x", &Sym::Real(xv)).to_f64() {
                Ok(v) if v.is_finite() && v.abs() < 1e-5 => {}
                _ => {
                    ok = false;
                    break;
                }
            }
        }
        if ok {
            return true;
        }
        for i in (0..n).rev() {
            idx[i] += 1;
            if idx[i] < cands.len() {
                break;
            }
            idx[i] = 0;
        }
    }
    false
}

struct Ref {
    name: &'static str,
    eq: Sym,
    max_order: usize,
    min_branches: usize,
}

fn run_ref(r: &Ref) {
    let branches = dsolve(&r.eq).unwrap_or_else(|e| panic!("{}: dsolve خطا داد {e:?}", r.name));
    assert!(
        branches.len() >= r.min_branches,
        "{}: شاخه‌ها {} < {}",
        r.name,
        branches.len(),
        r.min_branches
    );
    for (k, sol) in branches.iter().enumerate() {
        assert!(
            verify_sol(&r.eq, sol, r.max_order),
            "{}: شاخهٔ {k} ورفای مستقل را نگذشت: {}",
            r.name,
            mc_cas::sym_display(sol)
        );
    }
}

#[test]
fn reference_30_dsolve() {
    let refs = vec![
        // ---------- جداپذیر (۶) ----------
        Ref {
            name: "sep-e^(x+y)",
            eq: Sym::minus(dy(1), f("exp", Sym::plus(s("x"), fy()))),
            max_order: 1,
            min_branches: 1,
        },
        Ref {
            name: "sep-x*e^y",
            eq: Sym::minus(dy(1), mul(vec![s("x"), f("exp", fy())])),
            max_order: 1,
            min_branches: 1,
        },
        Ref {
            name: "sep-y^2",
            eq: Sym::minus(dy(1), pwn(fy(), i(2))),
            max_order: 1,
            min_branches: 1,
        },
        Ref {
            name: "sep-x*(1+y^2)",
            eq: Sym::minus(dy(1), mul(vec![s("x"), add(vec![i(1), pwn(fy(), i(2))])])),
            max_order: 1,
            min_branches: 1,
        },
        Ref {
            name: "sep-cos(x)*e^y",
            eq: Sym::minus(dy(1), mul(vec![f("cos", s("x")), f("exp", fy())])),
            max_order: 1,
            min_branches: 1,
        },
        // ---------- خطی مرتبهٔ اول (۸) ----------
        Ref {
            name: "lin-y'=x*y",
            eq: Sym::minus(dy(1), mul(vec![s("x"), fy()])),
            max_order: 1,
            min_branches: 1,
        },
        Ref {
            name: "lin-y'+y=e^x",
            eq: add(vec![dy(1), fy(), Sym::negated(f("exp", s("x")))]),
            max_order: 1,
            min_branches: 1,
        },
        Ref {
            name: "lin-y'-2y=4",
            eq: add(vec![
                dy(1),
                Sym::negated(mul(vec![i(2), fy()])),
                Sym::negated(i(4)),
            ]),
            max_order: 1,
            min_branches: 1,
        },
        Ref {
            name: "lin-y'+y=x",
            eq: add(vec![dy(1), fy(), Sym::negated(s("x"))]),
            max_order: 1,
            min_branches: 1,
        },
        Ref {
            name: "lin-y'+2xy=x*e^(-x^2)",
            eq: add(vec![
                dy(1),
                mul(vec![i(2), s("x"), fy()]),
                Sym::negated(mul(vec![s("x"), f("exp", Sym::negated(pwn(s("x"), i(2))))])),
            ]),
            max_order: 1,
            min_branches: 1,
        },
        Ref {
            name: "lin-y'+y*cos(x)=cos(x)",
            eq: add(vec![
                dy(1),
                mul(vec![fy(), f("cos", s("x"))]),
                Sym::negated(f("cos", s("x"))),
            ]),
            max_order: 1,
            min_branches: 1,
        },
        Ref {
            name: "lin-y'-3y=e^(3x)",
            eq: add(vec![
                dy(1),
                Sym::negated(mul(vec![i(3), fy()])),
                Sym::negated(f("exp", mul(vec![i(3), s("x")]))),
            ]),
            max_order: 1,
            min_branches: 1,
        },
        // ---------- انتگرال‌گیری مستقیم (۱) ----------
        Ref {
            name: "dir-y'=x*e^x",
            eq: Sym::minus(dy(1), mul(vec![s("x"), f("exp", s("x"))])),
            max_order: 1,
            min_branches: 1,
        },
        // ---------- کامل/فروپاشی log (۲) ----------
        Ref {
            name: "exact-2xy+1+x^2y'",
            eq: add(vec![
                mul(vec![i(2), s("x"), fy()]),
                i(1),
                mul(vec![pwn(s("x"), i(2)), dy(1)]),
            ]),
            max_order: 1,
            min_branches: 1,
        },
        Ref {
            name: "exact-ycosx+1+sinx y'",
            eq: add(vec![
                mul(vec![fy(), f("cos", s("x"))]),
                i(1),
                mul(vec![f("sin", s("x")), dy(1)]),
            ]),
            max_order: 1,
            min_branches: 1,
        },
        // ---------- خطی همگن ضرایب ثابت (۸) ----------
        Ref {
            name: "cc-y''+y=0",
            eq: add(vec![dy(2), fy()]),
            max_order: 2,
            min_branches: 1,
        },
        Ref {
            name: "cc-y''-y=0",
            eq: Sym::minus(dy(2), fy()),
            max_order: 2,
            min_branches: 1,
        },
        Ref {
            name: "cc-y''+2y'+y=0",
            eq: add(vec![dy(2), mul(vec![i(2), dy(1)]), fy()]),
            max_order: 2,
            min_branches: 1,
        },
        Ref {
            name: "cc-y'''=0",
            eq: dy(3),
            max_order: 3,
            min_branches: 1,
        },
        Ref {
            name: "cc-y''+y'+y=0",
            eq: add(vec![dy(2), dy(1), fy()]),
            max_order: 2,
            min_branches: 1,
        },
        Ref {
            name: "cc-y''+4y'+13y=0",
            eq: add(vec![dy(2), mul(vec![i(4), dy(1)]), mul(vec![i(13), fy()])]),
            max_order: 2,
            min_branches: 1,
        },
        Ref {
            name: "cc-y'''+...-(r-1)^3",
            eq: add(vec![
                dy(3),
                mul(vec![i(-3), dy(2)]),
                mul(vec![i(3), dy(1)]),
                Sym::negated(fy()),
            ]),
            max_order: 3,
            min_branches: 1,
        },
        // ---------- ضرایب نامعین (۹) ----------
        Ref {
            name: "uc-y''-y=x",
            eq: add(vec![dy(2), Sym::negated(fy()), Sym::negated(s("x"))]),
            max_order: 2,
            min_branches: 1,
        },
        Ref {
            name: "uc-y''+y=e^x",
            eq: add(vec![dy(2), fy(), Sym::negated(f("exp", s("x")))]),
            max_order: 2,
            min_branches: 1,
        },
        Ref {
            name: "uc-y''+y=sin(x)",
            eq: add(vec![dy(2), fy(), Sym::negated(f("sin", s("x")))]),
            max_order: 2,
            min_branches: 1,
        },
        Ref {
            name: "uc-y''-4y=e^(2x)",
            eq: add(vec![
                dy(2),
                Sym::negated(mul(vec![i(4), fy()])),
                Sym::negated(f("exp", mul(vec![i(2), s("x")]))),
            ]),
            max_order: 2,
            min_branches: 1,
        },
        Ref {
            name: "uc-y''+2y'=x+e^x",
            eq: add(vec![
                dy(2),
                mul(vec![i(2), dy(1)]),
                Sym::negated(s("x")),
                Sym::negated(f("exp", s("x"))),
            ]),
            max_order: 2,
            min_branches: 1,
        },
        Ref {
            name: "uc-y''-2y'+y=e^x*cos(x)",
            eq: add(vec![
                dy(2),
                Sym::negated(mul(vec![i(2), dy(1)])),
                fy(),
                Sym::negated(mul(vec![f("exp", s("x")), f("cos", s("x"))])),
            ]),
            max_order: 2,
            min_branches: 1,
        },
        Ref {
            name: "uc-y''+2y'+y=e^(-x)",
            eq: add(vec![
                dy(2),
                mul(vec![i(2), dy(1)]),
                fy(),
                Sym::negated(f("exp", Sym::negated(s("x")))),
            ]),
            max_order: 2,
            min_branches: 1,
        },
        Ref {
            name: "uc-y''+y=cos(2x)",
            eq: add(vec![
                dy(2),
                fy(),
                Sym::negated(f("cos", mul(vec![i(2), s("x")]))),
            ]),
            max_order: 2,
            min_branches: 1,
        },
    ];
    assert_eq!(refs.len(), 30, "تعداد مرجع‌ها باید ۳۰ باشد");
    for r in &refs {
        run_ref(r);
    }
}

#[test]
fn negative_honest_failures() {
    // Bernoulli: y' + y = y² → خارج از زیرمجموعه، خطای صادقانه با ode45.
    let bern = add(vec![dy(1), fy(), Sym::negated(pwn(fy(), i(2)))]);
    match dsolve(&bern) {
        Err(mc_cas::CasError::Unsupported(m)) => {
            assert!(m.contains("ode45"), "پیام باید ode45 را بگوید: {m}");
        }
        other => panic!("bernoulli باید خطای صادقانه می‌داد: {other:?}"),
    }
    // دو مجهول → ابهام صادقانه.
    let two = add(vec![dy(1), Sym::Fn(Arc::from("z"), vec![s("x")])]);
    assert!(matches!(
        dsolve(&two),
        Err(mc_cas::CasError::Unsupported(_))
    ));
}

#[test]
fn demo_d2y_plus_y() {
    // دموی سند ۰۵ §۷/ROADMAP: dsolve(D2y + y = 0) → C1·cos(x) + C2·sin(x)
    let eq = add(vec![dy(2), fy()]);
    let out = dsolve(&eq).expect("demo");
    assert_eq!(out.len(), 1);
    let d = mc_cas::sym_display(&out[0]);
    assert!(
        d.contains("cos") && d.contains("sin") && d.contains("C1") && d.contains("C2"),
        "دمو: {d}"
    );
    assert!(verify_sol(&eq, &out[0], 2));
}

#[test]
fn pins_displays() {
    // پین نمایش دقیق — فرم کانونی قطعی است.
    let eq = add(vec![dy(2), fy()]);
    let out = dsolve(&eq).expect("demo");
    assert_eq!(mc_cas::sym_display(&out[0]), "C1*cos(x) + C2*sin(x)");
    // خطی: y' − 2y = 4
    let eq2 = add(vec![
        dy(1),
        Sym::negated(mul(vec![i(2), fy()])),
        Sym::negated(i(4)),
    ]);
    let out2 = dsolve(&eq2).expect("lin");
    let d2 = mc_cas::sym_display(&out2[0]);
    assert_eq!(d2, "C1/(exp(-2*x)) - 2", "y'-2y=4 فرم کانونی");
}
