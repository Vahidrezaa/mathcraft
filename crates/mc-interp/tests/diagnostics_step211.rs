//! گام 2.11 ROADMAP — کارگاه دیاگنوستیک (بند ۱۴ ممیزی خارجی) — بندهای DoD.
//!
//! تاکسونومی کاتالوگ سند ۰۳ §۱۴ + سطح‌بندی سند ۰۲ §۹، به‌اجرا درآمده:
//! E3001 فقط خطای سینتکسی (lexer/parser)؛ خطای معنایی ارزیابی خانوادهٔ خودش
//! را می‌گیرد (E1001 نام متغیر · E1002 نام تابع · E2001 اندیس · E2002 نوع/
//! کاربرد · E2003 ابعاد · E4xxx منابع · E9xxx مرز صادقانه) و **هیچ خطای
//! کاربر 0:0 نمی‌ماند** (پشتیبان موقعیت عبارت/گزاره/فراخوانی).

use mc_core::McError;
use mc_interp::{eval_program, Environment};
use mc_parser::parse;

fn run_err(src: &str) -> McError {
    let prog = match parse(src) {
        Ok(p) => p,
        Err(e) => return e,
    };
    let mut env = Environment::new();
    let out = eval_program(&prog, &mut env);
    out.error.expect("expected an error")
}

fn run_ok(src: &str) {
    let prog = parse(src).expect("parse");
    let mut env = Environment::new();
    let out = eval_program(&prog, &mut env);
    assert!(out.error.is_none(), "unexpected error: {:?}", out.error);
}

// ---------- ۱) جدا بودن پارس از ارزیابی ----------

#[test]
fn parse_error_stays_e3001_with_real_position() {
    let e = run_err("1 +");
    assert_eq!(e.id, "MC:E3001_PARSE_ERROR");
    assert_eq!((e.line, e.col), (1, 4));
    // چندخطی: خطای سینتکس روی سطر درست
    let e = run_err("v = 'abc';\nw = 3 +\n4");
    assert_eq!(e.id, "MC:E3001_PARSE_ERROR");
    assert_eq!(e.line, 2, "syntax error must point at line 2: {e:?}");
}

#[test]
fn undefined_function_is_e1002_with_did_you_mean() {
    let e = run_err("y = lenght([1 2 3]);");
    assert_eq!(e.id, "MC:E1002_UNDEFINED_FUNCTION");
    assert!(e.message_en.contains("Function 'lenght' not found"));
    // did-you-mean سند ۰۲ §9 — روی توابع/builtinها
    assert_eq!(e.suggestion.as_deref(), Some("length"));
    assert_eq!((e.line, e.col), (1, 5), "position = the call site");
}

#[test]
fn undefined_variable_stays_e1001_bare_name() {
    // نام برهنه در عبارت = متغیر (E1001)؛ تمایز فراخوانی/نام برهنه = کاتالوگ
    let e = run_err("q = 2;\nr = q + zz;");
    assert_eq!(e.id, "MC:E1001_UNDEFINED_VARIABLE");
    assert_eq!(e.line, 2);
    assert!(e.col > 1, "column must point at the name, got {}", e.col);
}

// ---------- ۲) خانواده‌های معنایی ----------

#[test]
fn bounds_errors_are_e2001_all_paths() {
    // نام‌محور
    let e = run_err("x = [1 2 3];\ny = x(5);");
    assert_eq!(e.id, "MC:E2001_INDEX_OUT_OF_BOUNDS");
    assert_eq!(e.line, 2);
    // زنجیره‌ای (2.10) روی مقدار موقت — موقعیت = پرانتز اندیس
    let e = run_err("y = [1 2 3](5);");
    assert_eq!(e.id, "MC:E2001_INDEX_OUT_OF_BOUNDS", "{e:?}");
    assert_eq!((e.line, e.col), (1, 12));
    // فیلد → اندیس
    let e = run_err("s = struct('a', [1 2]);\ny = s.a(9);");
    assert_eq!(e.id, "MC:E2001_INDEX_OUT_OF_BOUNDS", "{e:?}");
    assert_eq!(e.line, 2);
    // دوبعدی
    let e = run_err("A = eye(3);\ny = A(2, 9);");
    assert_eq!(e.id, "MC:E2001_INDEX_OUT_OF_BOUNDS", "{e:?}");
}

#[test]
fn size_mismatch_is_e2003_at_operator() {
    let e = run_err("A = [1 2; 3 4];\nB = [1 2 3];\nC = A * B;");
    assert_eq!(e.id, "MC:E2003_SIZE_MISMATCH");
    // «C = A * B»: عملگر * در ستون ۷
    assert_eq!((e.line, e.col), (3, 7));
}

#[test]
fn type_errors_are_e2002_across_kinds() {
    // کلاس نادرست
    let e = run_err("s = struct('a', 1);\ny = s(1);");
    assert_eq!(e.id, "MC:E2002_TYPE_ERROR", "{e:?}");
    // تعداد آرگومان تابع بی‌نام — موقعیت فراخوانی، نه تعریف
    let e = run_err("f = @(x) x;\ny = 2;\nz = f(y, y);");
    assert_eq!(e.id, "MC:E2002_TYPE_ERROR", "{e:?}");
    assert_eq!((e.line, e.col), (3, 5), "arity error at the CALL site");
    // dot روی غیرساختار
    let e = run_err("x = 5;\ny = x.field;");
    assert_eq!(e.id, "MC:E2002_TYPE_ERROR", "{e:?}");
    // نام فیلد نامعتبر (پویا)
    let e = run_err("s = struct();\ns.('9bad') = 1;");
    assert_eq!(e.id, "MC:E2002_TYPE_ERROR", "{e:?}");
    assert_eq!(e.line, 2);
}

#[test]
fn builtin_errors_carry_the_call_position() {
    // خطای builtin (registry) قبلاً 0:0 بود — پشتیبان فراخوانی
    let e = run_err("a = eye(2);\nb = sqrt();");
    assert_eq!(e.id, "MC:E2002_TYPE_ERROR", "{e:?}");
    assert_eq!((e.line, e.col), (2, 5), "builtin error at the call: {e:?}");
}

// ---------- ۳) span روی گره‌های تازه (Unary/Range/Logic) ----------

#[test]
fn unary_logic_and_range_report_their_operator_position() {
    let e = run_err("s = struct('a', 1);\ny = -s;");
    assert_eq!(e.id, "MC:E2002_TYPE_ERROR");
    assert_eq!((e.line, e.col), (2, 5), "unary minus position: {e:?}");

    let e = run_err("a = [1 2];\nb = a && a;");
    assert_eq!(e.id, "MC:E2002_TYPE_ERROR");
    assert_eq!((e.line, e.col), (2, 7), "&& position: {e:?}");

    let e = run_err("x = [1 2];\ny = 1:x;");
    assert_eq!(e.id, "MC:E2002_TYPE_ERROR");
    assert_eq!((e.line, e.col), (2, 6), "colon position: {e:?}");
}

// ---------- ۴) بند «line 0:0 دیگر نیست» — باتری ----------

#[test]
fn no_user_error_keeps_the_zero_position() {
    // باتری نمایندهٔ همهٔ خانواده‌ها؛ هیچ‌کدام نباید (0,0) بماند
    let battery = [
        "1 +",
        "x = [1 2 3]; y = x(5);",
        "lenght(2)",
        "zz + 1",
        "s = struct(); s(1)",
        "A = [1 2]; B = [1 2 3]; A * B",
        "f = @(x) x; f(1, 2)",
        "a = [1 2]; b = a && a;",
        "x = [1 2]; y = 1:x;",
        "s = struct('a', 1); y = -s;",
        "a = eye(2); b = sqrt();",
        "x = [1 2]; y = x(1)(2);",
        "s = struct(); [s, 1];",
        "x = 1; y = x.t(2);",
        "w = 3; q = w.('dyn');",
    ];
    for src in battery {
        let e = run_err(src);
        assert!(
            e.line != 0 || e.col != 0,
            "error kept 0:0 — {src} → {} ({:?})",
            e.id,
            e.message_en
        );
    }
}

// ---------- ۵) سازگاری کاتالوگ (کد ↔ سند ۰۳ §۱۴) ----------

#[test]
fn catalog_ids_match_doc03_table() {
    // همهٔ ثابت‌های ids باید در جدول کاتالوگ سند ۰۳ نمایندliteral داشته باشند
    let doc = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../docs/03-language-spec.md"
    ))
    .expect("doc 03 readable");
    for id in [
        mc_core::ids::UNDEFINED_VARIABLE,
        mc_core::ids::UNDEFINED_FUNCTION,
        mc_core::ids::INDEX_OUT_OF_BOUNDS,
        mc_core::ids::TYPE_ERROR,
        mc_core::ids::SIZE_MISMATCH,
        mc_core::ids::SINGULAR_MATRIX,
        mc_core::ids::NOT_POSITIVE_DEFINITE,
        mc_core::ids::PARSE_ERROR,
        mc_core::ids::CANCELLED,
        mc_core::ids::OUT_OF_MEMORY,
    ] {
        let short = id.split('_').next().expect("MC:Exxxx prefix");
        assert!(
            doc.contains(short),
            "catalog row for {id} ({short}) missing in doc 03 s14"
        );
    }
    // ردیف‌های تازهٔ گام 2.11 با معنای درست
    assert!(doc.contains("MC:E1002"));
    assert!(doc.contains("MC:E2001"));
    assert!(doc.contains("MC:E2002"));
}

// ---------- ۶) رفتار مثبت دست‌نخورده ----------

#[test]
fn success_paths_are_untouched_by_the_taxonomy() {
    run_ok("x = [1 2 3]; y = x(2) + [1 2 3](end);");
    run_ok("s = struct('a', [1 2]); s.a(2) = 9; z = s.a(2);");
    run_ok("f = @(x) [x x]; y = f(3)(2);");
    run_ok("A = eye(3); b = A(2,2) + 1;");
    run_ok("i = 2; r = 1:i:5;");
    run_ok("t = true; u = t && t;");
}
