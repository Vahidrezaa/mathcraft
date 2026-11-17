//! گام 2.9 ROADMAP — structs minimal (`Value::Struct`) — بندهای DoD گام.
//!
//! پوشش: سازنده/فیلدها، auto-vivification، فیلد پویا، ارزش کپی، نمایش متلب‌گونه،
//! ابرازهای خطا (E4/2.10)، خوانش‌های شکل ۱×۱، odeset ساختاری (تسک ۴) با
//! آزمون اولویت، و ساختار وضعیت rng با رفت‌وبرگشت دقیق.
//! اوراکل رفتار: Octave 10.3 (scripts/probe_structs*.m سمت دستیار)؛ متن نمایش =
//! قرارداد متلب سند ۰۳ §11 (متن اکتاو متفاوت است — COMPATIBILITY §1).

use mc_core::Value;
use mc_interp::{eval_program, Environment};
use mc_parser::parse;

fn run(src: &str) -> Environment {
    let prog = parse(src).expect("parse");
    let mut env = Environment::new();
    let out = eval_program(&prog, &mut env);
    assert!(out.error.is_none(), "engine error: {:?}", out.error);
    env
}

fn run_err(src: &str) -> mc_core::McError {
    let prog = match parse(src) {
        Ok(p) => p,
        // خطای پارس هم خطای صادقانهٔ موتور است (مثل مرز s.f(2) → 2.10)
        Err(e) => return e,
    };
    let mut env = Environment::new();
    let out = eval_program(&prog, &mut env);
    out.error.expect("expected an error")
}

/// قفل سریال‌سازی تست‌های state-دارِ RNG — هارنس Rust موازی است و همه از
/// یک وضعیت سراسری می‌خوانند (الگوی ثبت‌شدهٔ mc-num/src/rng/mod.rs).
fn rng_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    LOCK.lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn scalar(env: &Environment, name: &str) -> f64 {
    match env.get(name) {
        Some(Value::Double(m)) if m.is_scalar() => m.get(0, 0),
        other => panic!("'{name}' is not a scalar: {other:?}"),
    }
}

fn vec_of(env: &Environment, name: &str) -> Vec<f64> {
    match env.get(name) {
        Some(Value::Double(m)) => (0..m.numel()).map(|k| m.get_linear(k)).collect(),
        other => panic!("'{name}' is not a matrix: {other:?}"),
    }
}

fn char_of(env: &Environment, name: &str) -> String {
    match env.get(name) {
        Some(Value::Char(s)) => s.clone(),
        other => panic!("'{name}' is not a char: {other:?}"),
    }
}

fn output_text(src: &str) -> Vec<String> {
    let prog = parse(src).expect("parse");
    let mut env = Environment::new();
    let out = eval_program(&prog, &mut env);
    assert!(out.error.is_none(), "engine error: {:?}", out.error);
    out.outputs.into_iter().map(|o| o.text).collect()
}

// ---------- تسک ۱: مدل داده + سینتکس s.f / s.f = v / فیلد پویا ----------

#[test]
fn ctor_roundtrip_all_value_kinds() {
    let env = run(
        "s = struct('n', 3.5, 'v', [1 2 3], 't', 'hi', 'c', 1+2i, 'z', []); \
         a = s.n; w = s.v * 2; t = s.t; zz = isempty(s.z);",
    );
    assert_eq!(scalar(&env, "a"), 3.5);
    assert_eq!(vec_of(&env, "w"), vec![2.0, 4.0, 6.0]);
    assert_eq!(char_of(&env, "t"), "hi");
    assert_eq!(scalar(&env, "zz"), 1.0);
}

#[test]
fn ctor_empty_struct_and_shape_family() {
    let env = run(
        "s = struct(); e = isempty(s); sc = isscalar(s); v = isvector(s); \
         sz = size(s); n = numel(s); l = length(s); r = isreal(s);",
    );
    // ۱×۱ هرگز خالی نیست — حتی بدون فیلد (اوراکل Octave 10.3: isempty(struct())=0)
    assert_eq!(scalar(&env, "e"), 0.0);
    assert_eq!(scalar(&env, "sc"), 1.0);
    assert_eq!(scalar(&env, "v"), 1.0);
    assert_eq!(vec_of(&env, "sz"), vec![1.0, 1.0]);
    assert_eq!(scalar(&env, "n"), 1.0);
    assert_eq!(scalar(&env, "l"), 1.0);
    assert_eq!(scalar(&env, "r"), 1.0);
}

#[test]
fn field_assignment_and_auto_vivification() {
    // q از صفر با فیلد ساخته می‌شود؛ w کل زنجیره را auto-viv می‌کند
    let env = run("q = struct(); q.a = 7; q.b = 'x'; \
         w.x.y = 5; \
         va = q.a; vb = q.b; wd = w.x.y;");
    assert_eq!(scalar(&env, "va"), 7.0);
    assert_eq!(char_of(&env, "vb"), "x");
    assert_eq!(scalar(&env, "wd"), 5.0);
    assert!(matches!(env.get("q"), Some(Value::Struct(_))));
    assert!(matches!(env.get("w"), Some(Value::Struct(_))));
}

#[test]
fn field_read_and_nested_chain() {
    let env = run("s = struct('a', struct('b', struct('c', 42))); \
         x = s.a.b.c; t = s.a; y = t.b.c;");
    assert_eq!(scalar(&env, "x"), 42.0);
    assert_eq!(scalar(&env, "y"), 42.0);
}

#[test]
fn nested_auto_vivification_creates_chain() {
    let env = run("w = struct(); w.a.b.c = 5; d = w.a.b.c;");
    assert_eq!(scalar(&env, "d"), 5.0);
    // زنجیرهٔ میانی ساختار است
    let wa = run("w = struct(); w.a.b = 5; t = w.a;").get("t").cloned();
    assert!(matches!(wa, Some(Value::Struct(_))));
}

#[test]
fn nested_write_lands_on_the_field_itself() {
    // رگرسیون: نوشتن در فیلدِ موجود باید همان فیلد را ویرایش کند نه ساختارِ
    // والد را درونش دوبار بپیچد؛ فیلد غیرساختاری = خطای متلب‌گونه
    let env = run("s = struct('a', struct('x', 1)); s.a.y = 2; \
         x = s.a.x; y = s.a.y; f = fieldnames(s.a);");
    assert_eq!(scalar(&env, "x"), 1.0);
    assert_eq!(scalar(&env, "y"), 2.0);
    assert_eq!(
        char_of(&env, "f"),
        "x y",
        "s.a has exactly the fields x and y"
    );
    let e = run_err("s = struct('a', 1); s.a.b = 2;");
    assert!(e.message_en.contains("nested struct"), "{}", e.message_en);
}

#[test]
fn value_semantics_copy_does_not_alias() {
    // ارزش کپی — اوراکل Octave 10.3: s_copy.a = 5 اثرش به s نمی‌رسد
    let env = run("s = struct('a', 1); s2 = s; s2.a = 5; \
         o = s.a; n = s2.a;");
    assert_eq!(scalar(&env, "o"), 1.0);
    assert_eq!(scalar(&env, "n"), 5.0);
}

#[test]
fn duplicate_field_last_wins() {
    // اوراکل Octave 10.3: struct('a',1,'a',2) → a = 2 بدون خطا
    let env = run("s = struct('a', 1, 'a', 2); v = s.a;");
    assert_eq!(scalar(&env, "v"), 2.0);
}

#[test]
fn dynamic_fieldname_read_and_assign() {
    let env = run("s = struct('a', 3.5, 'b', 9); nm = 'a'; \
         d = s.(nm); s.('c') = 11; c = s.c; e = s.(nm);");
    assert_eq!(scalar(&env, "d"), 3.5);
    assert_eq!(scalar(&env, "c"), 11.0);
    assert_eq!(scalar(&env, "e"), 3.5);
}

#[test]
fn dynamic_fieldname_from_expression() {
    let env = run("s = struct('ab', 5); d = s.(['a' 'b']);");
    assert_eq!(scalar(&env, "d"), 5.0);
}

#[test]
fn field_reassignment_changes_type() {
    let env = run("s = struct('a', 1); s.a = 'text'; t = s.a;");
    assert_eq!(char_of(&env, "t"), "text");
}

#[test]
fn for_over_struct_iterates_once() {
    // اوراکل Octave 10.3: پیمایش ساختار ۱×۱ = یک تکرار با خود ساختار
    let env = run("s = struct('a', 7); n = 0; tot = 0; \
         for k = s, n = n + 1; tot = tot + k.a; end");
    assert_eq!(scalar(&env, "n"), 1.0);
    assert_eq!(scalar(&env, "tot"), 7.0);
}

#[test]
fn isequal_is_order_insensitive() {
    // اوراکل Octave 10.3: isequal(struct('a',1,'b',2), struct('b',2,'a',1)) = 1
    let env = run("s1 = struct('a', 1, 'b', 2); s2 = struct('b', 2, 'a', 1); \
         s3 = struct('a', 1); s4 = struct('a', 9); \
         e1 = isequal(s1, s2); e2 = isequal(s1, s3); e3 = isequal(s1, s4);");
    assert_eq!(scalar(&env, "e1"), 1.0);
    assert_eq!(scalar(&env, "e2"), 0.0);
    assert_eq!(scalar(&env, "e3"), 0.0);
}

// ---------- تسک ۲: builtins حداقلی ----------

#[test]
fn fieldnames_joined_char_row() {
    // انحراف مستند: متلب cell می‌دهد؛ cell = E4 → ردیف char با فاصله
    let env = run(
        "s = struct('alpha', 1, 'beta', 2, 'gamma', 3); f = fieldnames(s); \
         e = struct(); fe = fieldnames(e);",
    );
    assert_eq!(char_of(&env, "f"), "alpha beta gamma");
    assert_eq!(char_of(&env, "fe"), "");
}

#[test]
fn isfield_hit_miss_and_nonstruct_error() {
    let env = run(
        "s = struct('a', 1); h = isfield(s, 'a'); m = isfield(s, 'zz'); \
         q = isfield(s, 'not a valid name!');",
    );
    assert_eq!(scalar(&env, "h"), 1.0);
    assert_eq!(scalar(&env, "m"), 0.0);
    assert_eq!(scalar(&env, "q"), 0.0);
    let e = run_err("isfield(1, 'a')");
    assert!(e.message_en.contains("struct"), "{}", e.message_en);
}

#[test]
fn getfield_and_missing_error() {
    let env = run("s = struct('a', 7); g = getfield(s, 'a');");
    assert_eq!(scalar(&env, "g"), 7.0);
    let e = run_err("getfield(struct('a', 1), 'zz')");
    assert!(e.message_en.contains("no field"), "{}", e.message_en);
}

#[test]
fn setfield_returns_new_and_adds_missing() {
    // اوراکل Octave 10.3: اصل دست‌نخورده؛ فیلد غایب اضافه می‌شود
    let env = run("s = struct('a', 1); t = setfield(s, 'b', 5); \
         o = isfield(s, 'b'); v = t.b; a1 = s.a;");
    assert_eq!(scalar(&env, "o"), 0.0);
    assert_eq!(scalar(&env, "v"), 5.0);
    assert_eq!(scalar(&env, "a1"), 1.0);
}

#[test]
fn ctor_validation_errors() {
    // نام نامعتبر (قاعدهٔ شناسهٔ متلب — پرچم پین با MATLAB واقعی)
    let e1 = run_err("s = struct('1st', 1);");
    assert!(
        e1.message_en.contains("valid field name"),
        "{}",
        e1.message_en
    );
    let e2 = run_err("s = struct('a b', 1);");
    assert!(
        e2.message_en.contains("valid field name"),
        "{}",
        e2.message_en
    );
    let long_name = format!("'{}'", "f".repeat(64));
    let e3 = run_err(&format!("s = struct({long_name}, 1);"));
    assert!(
        e3.message_en.contains("valid field name"),
        "{}",
        e3.message_en
    );
    // تعداد فرد آرگومان‌ها (اوراکل: additional arguments must occur as pairs)
    let e4 = run_err("struct('a', 1, 'b')");
    assert!(e4.message_en.contains("pairs"), "{}", e4.message_en);
    // نام غیررشته‌ای
    let e5 = run_err("struct(1, 2)");
    assert!(e5.message_en.contains("string"), "{}", e5.message_en);
}

// ---------- تسک ۳: نمایش متلب‌گونه ----------

#[test]
fn display_matlab_struct_style() {
    // قرارداد متلب سند ۰۳ §11 — متن Octave متفاوت است (دلتای مستند).
    // بدون «;» تا اکو فعال بماند (با «;» سرکوب می‌شود — سند ۰۳ §11 بند ۳)
    let out = output_text("s = struct('a', 3.5, 'b', 'hi')");
    assert_eq!(
        out[0],
        "s =\n  struct with fields:\n\n    a: 3.5000\n    b: 'hi'\n"
    );
}

#[test]
fn display_previews_scalar_row_matrix_nested_empty() {
    let out = output_text(
        "s = struct('sc', 3.14159, 'row', [1 2 3], 'mat', zeros(2, 3), \
         'nested', struct('x', 1), 'big', 1:100, 'e', [])",
    );
    let text = out.join("");
    assert!(text.contains("sc: 3.1416"), "scalar preview: {text}");
    assert!(text.contains("row: [1 2 3]"), "inline row: {text}");
    assert!(text.contains("mat: [2x3 double]"), "matrix summary: {text}");
    assert!(
        text.contains("nested: [1x1 struct]"),
        "nested summary: {text}"
    );
    assert!(
        text.contains("big: [1x100 double]"),
        "long row summary: {text}"
    );
    assert!(text.contains("e: []"), "empty: {text}");
}

#[test]
fn display_empty_struct_and_ans_echo() {
    let out = output_text("struct()");
    assert_eq!(out[0], "ans =\n  struct with no fields.\n");
    // انتساب فیلد بدون «;» کل ریشه را نمایش می‌دهد (رفتار متلب)
    let out2 = output_text("q = struct(); q.a = 7");
    assert_eq!(out2[0], "q =\n  struct with fields:\n\n    a: 7\n");
}

#[test]
fn disp_struct_prints_body_without_name() {
    let out = output_text("disp(struct('a', 1))");
    assert_eq!(out[0], "  struct with fields:\n\n    a: 1\n");
}

#[test]
fn display_respects_format_long() {
    let out = output_text("format long; s = struct('a', 1/3)");
    assert!(out[0].contains("a: 0.333333333333333"), "{}", out[0]);
}

// ---------- مرزهای صادقانه ----------

#[test]
fn honest_errors_on_struct_boundaries() {
    // dot روی غیرساختار (اوراکل: scalar cannot be indexed with .)
    let e1 = run_err("x = 1; x.a");
    assert!(e1.message_en.contains("struct"), "{}", e1.message_en);
    // فیلد غایب
    let e2 = run_err("s = struct('a', 1); s.zz");
    assert!(e2.message_en.contains("no field"), "{}", e2.message_en);
    // اندیس‌گذاری پس از فیلد = 2.10
    let e3 = run_err("s = struct('a', [1 2]); s.a(2)");
    assert!(e3.message_en.contains("2.10"), "{}", e3.message_en);
    // داخل ماتریس = آرایهٔ ساختاری = E4 (اوراکل با [] آرایهٔ ساختاری می‌سازد)
    let e4 = run_err("s = struct('a', 1); [s, 1]");
    assert!(e4.message_en.contains("E4"), "{}", e4.message_en);
    // شرط if با ساختار (متلب خطا می‌دهد؛ اکتاو خیر — قرارداد متلب ملاک)
    let e5 = run_err("s = struct('a', 1); if s; end");
    assert!(e5.message_en.contains("scalar"), "{}", e5.message_en);
    // ترانهاده
    let e6 = run_err("s = struct('a', 1); s'");
    assert!(e6.message_en.contains("transpose"), "{}", e6.message_en);
    // انتساب فیلد به غیرساختار
    let e7 = run_err("x = 1; x.a = 2;");
    assert!(e7.message_en.contains("struct"), "{}", e7.message_en);
    // انتساب تودرتو روی فیلد غیرساختاری
    let e8 = run_err("s = struct('a', 1); s.a.b = 2;");
    assert!(e8.message_en.contains("nested struct"), "{}", e8.message_en);
    // نام پویای نامعتبر
    let e9 = run_err("s = struct(); s.('1st') = 1;");
    assert!(
        e9.message_en.contains("valid field name"),
        "{}",
        e9.message_en
    );
    // نام پویای غیرchar
    let e10 = run_err("s = struct('a', 1); f = 5; s.(f)");
    assert!(e10.message_en.contains("char"), "{}", e10.message_en);
    // double/num2str روی ساختار
    let e11 = run_err("double(struct('a', 1))");
    assert!(e11.message_en.contains("fields"), "{}", e11.message_en);
}

#[test]
fn depth_guard_is_a_clean_error() {
    // سقف منابع ۲۵۶ — اکتاو سقفی ندارد (انحراف مستند)؛ خطای پاک نه پنیک
    let e = run_err("d = struct('x', 1); for i = 1:300, d = struct('n', d); end;");
    assert!(e.message_en.contains("depth"), "{}", e.message_en);
}

// ---------- تسک ۴: odeset ساختاری + اولویت ----------

#[test]
fn odeset_defaults_pairs_and_merge_precedence() {
    let env = run("o0 = odeset(); r0 = o0.RelTol; a0 = o0.AbsTol; \
         o1 = odeset('RelTol', 1e-6); r1 = o1.RelTol; a1 = o1.AbsTol; \
         o3 = odeset(o1, 'RelTol', 1e-4, 'AbsTol', 1e-9); r3 = o3.RelTol; a3 = o3.AbsTol;");
    assert_eq!(scalar(&env, "r0"), 1e-3);
    assert_eq!(scalar(&env, "a0"), 1e-6);
    assert_eq!(scalar(&env, "r1"), 1e-6);
    assert_eq!(scalar(&env, "a1"), 1e-6, "merge keeps unspecified defaults");
    // اولویت: جفت‌های تازه برنده‌اند (سمانتیک odeset متلب)
    assert_eq!(scalar(&env, "r3"), 1e-4);
    assert_eq!(scalar(&env, "a3"), 1e-9);
}

#[test]
fn odeset_unknown_option_is_honest_error() {
    // اکتاو فقط هشدار می‌دهد — قرارداد موتور: خطا (دلتای مستند)
    let e1 = run_err("odeset('Bogus', 1)");
    assert!(
        e1.message_en.contains("unsupported option"),
        "{}",
        e1.message_en
    );
    let e2 = run_err("odeset(struct('Bogus', 1))");
    assert!(
        e2.message_en.contains("unsupported option"),
        "{}",
        e2.message_en
    );
    let e3 = run_err("odeset('RelTol')");
    assert!(e3.message_en.contains("pairs"), "{}", e3.message_en);
}

#[test]
fn ode45_struct_form_is_bit_identical_to_named_pairs() {
    // آزمون ناوردا (DoD گام): شکل ساختاری همان شکل جفتی است — بیت‌به‌بیت
    let env = run("f = @(t, y) -y; \
         o = odeset('RelTol', 1e-8, 'AbsTol', 1e-11); \
         [T1, Y1] = ode45(f, [0 2], 1, o); \
         [T2, Y2] = ode45(f, [0 2], 1, 'RelTol', 1e-8, 'AbsTol', 1e-11); \
         d = max(abs(Y1 - Y2)); e = max(abs(T1 - T2));");
    assert_eq!(scalar(&env, "d"), 0.0, "struct form must equal pair form");
    assert_eq!(scalar(&env, "e"), 0.0);
}

#[test]
fn ode45_ode23_struct_form_analytic() {
    // y'=-y → y(2)=e^-2 (تحلیلی — تلورانس گام)
    let env = run("f = @(t, y) -y; \
         Y1 = ode45(f, [0 2], 1, odeset('RelTol', 1e-9)); y1 = Y1(end, 2); \
         Y2 = ode23(f, [0 2], 1, odeset('RelTol', 1e-9)); y2 = Y2(end, 2); \
         e1 = abs(y1 - exp(-2)); e2 = abs(y2 - exp(-2));");
    assert!(scalar(&env, "e1") < 1e-6, "e1 = {}", scalar(&env, "e1"));
    assert!(scalar(&env, "e2") < 1e-4, "e2 = {}", scalar(&env, "e2"));
}

#[test]
fn ode_struct_form_validations() {
    let e1 = run_err("ode45(@(t,y)-y, [0 1], 1, odeset('RelTol', 1e-8), 'AbsTol', 1e-12)");
    assert!(e1.message_en.contains("not both"), "{}", e1.message_en);
    let e2 = run_err("ode45(@(t,y)-y, [0 1], 1, struct('Bogus', 1))");
    assert!(
        e2.message_en.contains("unsupported option"),
        "{}",
        e2.message_en
    );
    // [] = بازگشت به پیش‌فرض (قرارداد متلب — همهٔ گزینه‌ها)؛ در ساختار []
    // ذخیره می‌شود و حلگر هنگام اعمال به پیش‌فرض می‌رود
    let env = run(
        "o = odeset('MaxStep', 0.5); o2 = odeset(o, 'MaxStep', []); \
         m1 = o.MaxStep; e = isempty(o2.MaxStep); \
         o3 = odeset('RelTol', 1e-8); o4 = odeset(o3, 'RelTol', []); \
         e4 = isempty(o4.RelTol); \
         f = @(t,y)-y; y = ode45(f, [0 1], 1, o4); ok = ~isempty(y);",
    );
    assert_eq!(scalar(&env, "m1"), 0.5);
    assert_eq!(scalar(&env, "e"), 1.0);
    assert_eq!(
        scalar(&env, "e4"),
        1.0,
        "[] is stored; the solver applies the default"
    );
    assert_eq!(
        scalar(&env, "ok"),
        1.0,
        "solve works with the reset default"
    );
}

// ---------- تسک ۴: rng ساختار وضعیت ----------

#[test]
fn rng_returns_status_struct_and_reseed_is_deterministic() {
    let _guard = rng_lock();
    let env = run(
        "st = rng(42); ty = st.Type; sd = st.Seed; n = numel(st.State); \
         a = rand; rng(42); b = rand; d = a - b;",
    );
    assert_eq!(char_of(&env, "ty"), "PCG64");
    assert_eq!(scalar(&env, "sd"), 42.0);
    assert_eq!(scalar(&env, "n"), 16.0);
    assert_eq!(scalar(&env, "d"), 0.0);
}

#[test]
fn rng_capture_restore_exact_replay() {
    let _guard = rng_lock();
    let env = run("st = rng; a1 = rand; a2 = randn; a3 = rand; \
         rng(st); b1 = rand; b2 = randn; b3 = rand; \
         r = [a1 - b1, a2 - b2, a3 - b3];");
    assert_eq!(vec_of(&env, "r"), vec![0.0, 0.0, 0.0]);
}

#[test]
fn rng_statement_form_has_no_output_and_no_ans() {
    let _guard = rng_lock();
    // متلب: rng(42) گزاره‌ای بدون اکو/ans است
    let prog = parse("rng(42)").unwrap();
    let mut env = Environment::new();
    let out = eval_program(&prog, &mut env);
    assert!(out.error.is_none());
    assert!(out.outputs.is_empty(), "no echo expected");
    assert!(env.get("ans").is_none(), "ans must stay unset");
    // rng برهنه هم همین‌طور
    let prog2 = parse("rng").unwrap();
    let mut env2 = Environment::new();
    let out2 = eval_program(&prog2, &mut env2);
    assert!(out2.error.is_none());
    assert!(out2.outputs.is_empty());
}

#[test]
fn rng_default_and_generator_names() {
    let _guard = rng_lock();
    // 'default' = seed صفر = وضعیت آغازین (تست واحد mc-num همین را می‌سنجد)
    let env = run("rng('default'); a = rand; rng(0); b = rand; d = a - b;");
    assert_eq!(scalar(&env, "d"), 0.0);
    // نام مولد/‘shuffle’ — صادقانه: موتور PCG64 واحد دارد (انحراف مستند)
    let e1 = run_err("rng('twister')");
    assert!(e1.message_en.contains("PCG64"), "{}", e1.message_en);
    let e2 = run_err("rng('shuffle')");
    assert!(e2.message_en.contains("PCG64"), "{}", e2.message_en);
    let e3 = run_err("rng(-1)");
    assert!(e3.message_en.contains("non-negative"), "{}", e3.message_en);
    let e4 = run_err("rng(1.5)");
    assert!(e4.message_en.contains("non-negative"), "{}", e4.message_en);
}

#[test]
fn rng_restore_validates_struct() {
    let _guard = rng_lock();
    let e1 = run_err("rng(struct('Type', 'twister'))");
    assert!(e1.message_en.contains("PCG64"), "{}", e1.message_en);
    // خراب‌کردن State با انتساب کل‌فیلد (اندیس‌گذاری فیلد = 2.10)
    let e2 = run_err("rng(42); st = rng; st.State = st.State + 0.5; rng(st)");
    assert!(
        e2.message_en.contains("exact integers") || e2.message_en.contains("State"),
        "{}",
        e2.message_en
    );
    let e3 = run_err("rng(42); st = rng; st.State = [1 2 3]; rng(st)");
    assert!(e3.message_en.contains("16x1"), "{}", e3.message_en);
}

// ---------- لکسر/پارسر: توکن نقطه و هدف انتساب ----------

#[test]
fn lexer_dot_token_and_field_parse() {
    use mc_parser::lexer::{tokenize, Tok};
    let toks = tokenize("s.a").unwrap();
    assert_eq!(
        toks[1].tok,
        Tok::Dot,
        "bare '.' between identifiers is the field-access dot"
    );
    // عملگرهای عنصری همچنان جدا
    let toks2 = tokenize("a .* b ./ c .^ d .'").unwrap();
    assert!(toks2.iter().any(|t| matches!(t.tok, Tok::DotStar)));
    assert!(!toks2.iter().any(|t| matches!(t.tok, Tok::Dot)));
    // 1.5 و .5 عدد می‌مانند
    let toks3 = tokenize("x = 1.5 + .5").unwrap();
    assert!(toks3.iter().any(|t| matches!(t.tok, Tok::Num(1.5))));
    assert!(!toks3.iter().any(|t| matches!(t.tok, Tok::Dot)));
}

#[test]
fn field_access_parses_after_call_and_parens() {
    // f(x).a و (s).a — پسوند روی هر عملوند؛ s.a(2) → خطای 2.10
    let env = run("f = @(x) struct('v', x); r = f(3).v; \
         s = struct('a', 5); t = (s).a;");
    assert_eq!(scalar(&env, "r"), 3.0);
    assert_eq!(scalar(&env, "t"), 5.0);
    let e = run_err("s = struct('a', [1 2]); s.a(2)");
    assert!(e.message_en.contains("2.10"), "{}", e.message_en);
}

#[test]
fn struct_fields_in_matrix_literal_read_like_matlab() {
    // [s.f 2] — فیلد داخل ماتریس لفظی (متلب مجاز می‌داند)
    let env = run("s = struct('a', 10); v = [s.a 2];");
    assert_eq!(vec_of(&env, "v"), vec![10.0, 2.0]);
}

#[test]
fn class_and_varsummary_shape_for_structs() {
    let env = run("s = struct('a', 1); c = class(s);");
    assert_eq!(char_of(&env, "c"), "struct");
}
