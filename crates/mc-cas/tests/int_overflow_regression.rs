//! رگرسیون CAS-2 — درس مجموعهٔ مرجع ۱۲۰ (گام 4.3).
//!
//! باقیماندهٔ ورفایِ پادمشتق‌های log|·| پس از back-substitution جایگذاری
//! می‌تواند abs/sign تودرتو بسازد (مثل log(|abs(x−1)|))؛ مسیر گویای
//! simplify آن را به gcd ژنراتورهای اُپک می‌کشد و بدون گارد عمقِ بازگشت
//! متقابل pdiv_exact ↔ cancel ↔ poly_gcd سرریز پشته می‌داد. دو اصلاح
//! ریشه‌ای: گارد عمق ۶۴ در poly_gcd (خطای صادقانه در فراتر از بودجه) و
//! قواعد کل-دامنهٔ | |u| | = |u| و sign(|u|) = sign(u) در simplify.

use mc_cas::Sym;

fn sym(n: &str) -> Sym {
    Sym::symbol(n)
}

#[test]
fn division_with_remainder_verifies_cleanly() {
    // (x²+1)/(x−1) = x + 1 + 2/(x−1) → (x+1) + 2·log|x−1| — هرگز crash نه
    let x = sym("x");
    let g = Sym::over(
        Sym::plus(Sym::pow(x.clone(), Sym::int(2)).expect("pow"), Sym::int(1)),
        Sym::minus(x.clone(), Sym::int(1)),
    )
    .expect("over");
    let r = mc_cas::int::integrate(&g, "x");
    // integrate فقط نتیجهٔ ورفای‌شده را Some می‌دهد (DoD هسته) — پس
    // Ok(Some) یعنی هم بدون crash و هم با پادمشتق درست
    let f = r.expect("integrate ok").expect("verified antiderivative");
    // شکل log|x−1| در نتیجه (لوگ‌اسپلیت باقیماندهٔ 2/(x−1))
    let disp = mc_cas::sym_display(&f);
    assert!(
        disp.contains("abs(x - 1)"),
        "پادمشتق باید log|x−1| داشته باشد: {disp}"
    );
}

#[test]
fn double_abs_rules_collapse_residues() {
    // | |u| | = |u| و sign(|u|) = sign(u) — کل-دامنه، شامل u=0
    let x = sym("x");
    let inner = Sym::fn_of("abs", vec![Sym::minus(x.clone(), Sym::int(1))]).expect("abs");
    let double_abs = Sym::fn_of("abs", vec![inner.clone()]).expect("abs");
    let z = mc_cas::simplify::simplify(&double_abs).expect("simplify");
    assert_eq!(mc_cas::sym_display(&z), "abs(x - 1)");
    let s_of_abs = Sym::fn_of("sign", vec![inner]).expect("sign");
    let z2 = mc_cas::simplify::simplify(&s_of_abs).expect("simplify");
    assert_eq!(mc_cas::sym_display(&z2), "sign(x - 1)");
}

#[test]
fn double_root_denominator_honest_none() {
    // 1/(4x²+4x+1) با ریشهٔ مضاعف گویا: RRT باید بگیرد یا None صادقانه —
    // در هر دو حالت Ok بدون crash (مورد مرجع ۳۰)
    let x = sym("x");
    let den = Sym::plus(
        Sym::plus(
            Sym::times(Sym::int(4), Sym::pow(x.clone(), Sym::int(2)).expect("pow")),
            Sym::times(Sym::int(4), x.clone()),
        ),
        Sym::int(1),
    );
    let f = Sym::over(Sym::int(1), den).expect("over");
    let r = mc_cas::int::integrate(&f, "x");
    assert!(r.is_ok(), "integrate باید Ok باشد: {:?}", r.err());
}

#[test]
fn regression_five_fixed_forms() {
    // پنج فرم که در تمرین مجموعهٔ مرجع شکست/سرریز داشتند — پس از فیکس‌های
    // گام 4.3 (انتگرال خارج‌قسمت، گارد gcd، مونیک PF، بازنویسی توان فرد،
    // توزیع ضرب-روی-جمع، جدول هذلولی) همه باید پادمشتق ورفای‌شده بدهند.
    let x = sym("x");
    let cases: Vec<(&str, Sym)> = vec![
        (
            "x^(1/3)",
            Sym::pow(x.clone(), Sym::rat(1, 3).expect("r")).expect("pow"),
        ),
        (
            "sin(x)^3",
            Sym::pow(Sym::fn_of("sin", vec![x.clone()]).expect("s"), Sym::int(3)).expect("pow"),
        ),
        (
            "sinh(x)^2",
            Sym::pow(Sym::fn_of("sinh", vec![x.clone()]).expect("s"), Sym::int(2)).expect("pow"),
        ),
        (
            "exp(x)*(sin(x)+cos(x))",
            Sym::times(
                Sym::fn_of("exp", vec![x.clone()]).expect("e"),
                Sym::plus(
                    Sym::fn_of("sin", vec![x.clone()]).expect("s"),
                    Sym::fn_of("cos", vec![x.clone()]).expect("c"),
                ),
            ),
        ),
        (
            "1/(4x^2+4x+1)",
            Sym::over(
                Sym::int(1),
                Sym::plus(
                    Sym::plus(
                        Sym::times(Sym::int(4), Sym::pow(x.clone(), Sym::int(2)).expect("p")),
                        Sym::times(Sym::int(4), x.clone()),
                    ),
                    Sym::int(1),
                ),
            )
            .expect("o"),
        ),
    ];
    for (name, f) in &cases {
        let r = mc_cas::int::integrate(f, "x");
        let f_anti = r
            .expect("integrate ok")
            .unwrap_or_else(|| panic!("{name}: باید پادمشتق ورفای‌شده بدهد"));
        let disp = mc_cas::sym_display(&f_anti);
        assert!(!disp.is_empty(), "{name}: پادمشتق خالی");
    }
}
