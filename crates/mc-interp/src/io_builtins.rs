//! فرمان‌های فایل/دادهٔ I/O — گام 3.7 (ROADMAP §3.7).
//!
//! دو دسته:
//! - **کمکی‌های گزاره‌ای** (فراخوانی از eval.rs، الگوی `format`/`disp` با
//!   تقدم override کاربر): `who`، `whos`، `clear`/`clearvars`، `save`،
//!   `load`، `writematrix` — بدون اکو، بدون ans.
//! - **تابع رجیستری**: `readmatrix` (عبارتی، ماتریس برمی‌گرداند) و
//!   `readtable` (خطای صادقانهٔ ارجاع‌دار — نوع table = E4).
//!
//! رفتار متلب به تصریح داک‌ها (پژوهش نوبت 2026-11-19):
//! - `who`: «Your variables are:» + نام‌ها در ستون‌های جا‌شونده (مرتب
//!   الفبایی ASCII — رفتار متلب).
//! - `whos`: جدول Name/Size/Bytes/Class/Attributes + خط خالی بعد از سرصفحه
//!   (Bytes راست‌چین؛ complex در Attributes).
//! - `save` ساکت است؛ بدون نام → نام پیش‌فرض؛ بی‌پسوند → پسوند قالب می‌چسبد.
//! - `load` ساکت؛ پسوند .mat → خوانندهٔ v5؛ .mcws/بی‌پسوند → قالب خودمان.
//!
//! بایت‌های فایل از VFS می‌آیند (میزبان ثبت کرده — الگوی register_m_source)
//! و خروجی‌ها با `vfs::put_output` به میزبان برمی‌گردند.

use mc_core::error::{ids, McError};
use mc_core::value::Value;
use mc_io::delim::{self, Delimiter};
use mc_io::{matv5, mcws, vfs};

use crate::env::Environment;

/// پیش‌فرض نام فایل ذخیره (stem همان matlab متلب؛ پسوند قالب خودمان).
pub const DEFAULT_SAVE_NAME: &str = "matlab.mcws";

fn io_err(line: usize, col: usize, e: mc_io::IoError) -> McError {
    McError::new(
        ids::FILE_ERROR,
        line as u32,
        col as u32,
        e.message_en,
        e.message_fa,
    )
}

// ---------- who / whos ----------

/// نام متغیرهای موجود — مرتب ASCII (رفتار who متلب).
pub fn sorted_names(env: &Environment) -> Vec<String> {
    let mut names: Vec<String> = env.iter().map(|(n, _)| n.clone()).collect();
    names.sort();
    names
}

/// متن `who` — سرصفحهٔ متلب + نام‌ها در ستون‌هایی که در ~۷۸ کاراکتر جا
/// می‌شوند (ستون = نام + ۲ فاصله؛ اگر تک‌نام بلند بود، همان یک ستون).
pub fn who_text(env: &Environment) -> String {
    let names = sorted_names(env);
    let mut out = String::from("Your variables are:\n\n");
    if names.is_empty() {
        return out;
    }
    let width = names.iter().map(|n| n.len()).max().unwrap_or(0) + 2;
    let cols = (78 / width).max(1);
    let mut line = String::new();
    for (i, n) in names.iter().enumerate() {
        line.push_str(n);
        for _ in n.len()..width {
            line.push(' ');
        }
        if (i + 1) % cols == 0 || i + 1 == names.len() {
            out.push_str(line.trim_end());
            out.push('\n');
            line.clear();
        }
    }
    out
}

/// بایت‌های «برآوردی» متلب — payload خالص بدون سربار هدر (دلتای مستند در
/// COMPATIBILITY): double=8، complex=16 در هر عنصر، char=2 در هر کاراکتر،
/// struct=جمع بازگشتی.
pub fn value_bytes(v: &Value) -> usize {
    match v {
        Value::Double(m) => m.numel() * 8,
        Value::Complex(m) => m.numel() * 16,
        Value::Char(s) => s.chars().count() * 2,
        Value::Struct(s) => s.fields.iter().map(|(n, f)| n.len() + value_bytes(f)).sum(),
        Value::Anon(_) => 0,
        // سمبولیک: نمایش خطی به‌عنوان تخمین اندازه (whos متلب هم نمایش محور است)
        Value::Sym(s) => mc_cas::sym_display(s).len() + 8,
        // cell (لیست): جمع بازگشتی اعضا + سربار ثابت ۱۶ بایت/عضو
        Value::List(vs) => vs.iter().map(value_bytes).sum::<usize>() + vs.len() * 16,
    }
}

/// متن `whos` — جدول با ستون‌های داینامیک (عرض هر ستون = max(سرصفحه،
/// محتوا))، Bytes راست‌چین، خط خالی بعد از سرصفحه (خروجی واقعی متلب).
pub fn whos_text(env: &Environment) -> String {
    let entries: Vec<(String, String, String, &'static str, &'static str)> = env
        .iter()
        .map(|(n, v)| {
            let size = format!("{}x{}", v_rows(v), v_cols(v));
            let bytes = value_bytes(v).to_string();
            let class = match v {
                Value::Struct(_) => "struct",
                Value::Char(_) => "char",
                Value::Anon(_) => "function_handle",
                Value::Sym(_) => "sym",
                Value::List(_) => "cell",
                Value::Double(_) | Value::Complex(_) => "double",
            };
            let attr: &'static str = match v {
                Value::Complex(_) => "complex",
                _ => "",
            };
            (n.clone(), size, bytes, class, attr)
        })
        .collect();
    // مرتب مثل who
    let mut sorted = entries;
    sorted.sort_by(|a, b| a.0.cmp(&b.0));

    let hdr = ["Name", "Size", "Bytes", "Class", "Attributes"];
    let w_name = sorted
        .iter()
        .map(|e| e.0.len())
        .chain([hdr[0].len()])
        .max()
        .unwrap_or(4);
    let w_size = sorted
        .iter()
        .map(|e| e.1.len())
        .chain([hdr[1].len()])
        .max()
        .unwrap_or(4);
    let w_bytes = sorted
        .iter()
        .map(|e| e.2.len())
        .chain([hdr[2].len()])
        .max()
        .unwrap_or(5);
    let w_class = sorted
        .iter()
        .map(|e| e.3.len())
        .chain([hdr[3].len()])
        .max()
        .unwrap_or(5);

    let mut out = String::new();
    out.push_str(&format!(
        "  {:<width_n$}  {:<width_s$}  {:>width_b$}  {:<width_c$}  {}\n\n",
        "Name",
        "Size",
        "Bytes",
        "Class",
        "Attributes",
        width_n = w_name,
        width_s = w_size,
        width_b = w_bytes,
        width_c = w_class,
    ));
    for (name, size, bytes, class, attr) in &sorted {
        out.push_str(&format!(
            "  {:<width_n$}  {:<width_s$}  {:>width_b$}  {:<width_c$}  {}\n",
            name,
            size,
            bytes,
            class,
            attr,
            width_n = w_name,
            width_s = w_size,
            width_b = w_bytes,
            width_c = w_class,
        ));
    }
    out
}

fn v_rows(v: &Value) -> usize {
    match v {
        Value::Double(m) => m.rows(),
        Value::Complex(m) => m.rows(),
        Value::Char(_) | Value::Struct(_) | Value::Anon(_) | Value::Sym(_) => 1,
        // cell ستونی N×1 (سمانتیک shape سند ۰۲ — خالی: 0×1)
        Value::List(vs) => vs.len(),
    }
}

fn v_cols(v: &Value) -> usize {
    match v {
        Value::Sym(s) => mc_cas::sym_display(s).len().max(1),
        Value::Double(m) => m.cols(),
        Value::Complex(m) => m.cols(),
        // کاراکتری خالی 1×0 است در متلب؛ رشتهٔ خالی → 0 (Max با ۱ نگه‌داشته شد
        // برای سازگاری جدول)
        Value::Char(s) => s.chars().count().max(1),
        Value::Struct(_) | Value::Anon(_) => 1,
        Value::List(_) => 1,
    }
}

// ---------- clear ----------

/// `clear` / `clear all` / `clear a b` — تعداد پاک‌شده‌ها. توابع ثبت‌شده
/// دست نمی‌خورند (برای آن‌ها مسیر جدا؛ دلتای مستند: clear name فقط متغیر).
pub fn clear_apply(env: &mut Environment, names: &[String]) -> usize {
    if names.is_empty() || names.iter().any(|n| n == "all") {
        return env.clear();
    }
    let mut n = 0;
    for name in names {
        if env.remove(name).is_some() {
            n += 1;
        }
    }
    n
}

// ---------- save / load ----------

/// پسوند قالب خودمان را اگر لازم است می‌چسباند (رفتار متلب: بی‌پسوند →
/// پسوند قالب). نام صریحِ .mat → خطا (نوشتن .mat هنوز نیست — 5.5).
fn save_file_name(raw: &str, line: usize, col: usize) -> Result<String, McError> {
    let base = raw.rsplit(['/', '\\']).next().unwrap_or(raw);
    if base.ends_with(".mat") {
        return Err(McError::new(
            ids::UNSUPPORTED,
            line as u32,
            col as u32,
            "writing .mat files is not available yet (planned for 5.5) — save writes the MCWS format; use a .mcws name",
            "نوشتن فایل .mat هنوز موجود نیست (برای گام 5.5 برنامه‌ریزی شده) — save با قالب MCWS می‌نویسد؛ نام .mcws بدهید",
        ));
    }
    if base.contains('.') {
        Ok(base.to_owned())
    } else {
        Ok(format!("{base}.mcws"))
    }
}

/// `save` / `save fname` / `save fname a b` — MCWS v1 در VFS خروجی؛ سکوت
/// کامل در موفقیت (رفتار متلب). متغیر غایب در varlist = خطا (رفتار متلب).
pub fn save_apply(
    env: &mut Environment,
    file: Option<String>,
    names: &[String],
    line: usize,
    col: usize,
) -> Result<(), McError> {
    let file = save_file_name(file.as_deref().unwrap_or(DEFAULT_SAVE_NAME), line, col)?;
    let vars: Vec<(String, Value)> = if names.is_empty() {
        env.iter().map(|(n, v)| (n.clone(), v.clone())).collect()
    } else {
        let mut out = Vec::with_capacity(names.len());
        for n in names {
            match env.get(n) {
                Some(v) => out.push((n.clone(), v.clone())),
                None => {
                    return Err(McError::new(
                        ids::UNDEFINED_VARIABLE,
                        line as u32,
                        col as u32,
                        format!("'{n}' is not a variable in the workspace"),
                        format!("«{n}» متغیری در ورک‌اسپس نیست"),
                    ));
                }
            }
        }
        out
    };
    let bytes = mcws::encode_workspace(&vars).map_err(|e| io_err(line, col, e))?;
    vfs::put_output(&file, bytes);
    Ok(())
}

fn file_missing(name: &str, line: usize, col: usize) -> McError {
    // راهنمای عملی: چه چیزهایی هست
    let mut known = vfs::known_names();
    known.sort();
    let hint = if known.is_empty() {
        "no files are registered in this session yet".to_owned()
    } else {
        let shown: Vec<String> = known.iter().take(8).cloned().collect();
        let more = if known.len() > 8 { ", …" } else { "" };
        format!("registered files: {}{more}", shown.join(", "))
    };
    McError::new(
        ids::FILE_ERROR,
        line as u32,
        col as u32,
        format!("'{name}' was not found in this session ({hint}); on the device, files placed in the MathCraft folder are picked up automatically",
            ),
        format!("فایل «{name}» در این نشست یافت نشد ({hint})؛ روی دستگاه، فایل‌های پوشهٔ MathCraft خودکار ثبت می‌شوند"),
    )
}

/// `load fname` / `load fname a b` — .mat v5 یا MCWS از VFS؛ تزریق به env؛
/// سکوت (رفتار متلب). بی‌پسوند → اول .mcws بعد .mat.
pub fn load_apply(
    env: &mut Environment,
    file: &str,
    names: &[String],
    line: usize,
    col: usize,
) -> Result<(), McError> {
    let base = file.rsplit(['/', '\\']).next().unwrap_or(file);
    let candidates: Vec<String> = if base.contains('.') {
        vec![base.to_owned()]
    } else {
        vec![format!("{base}.mcws"), format!("{base}.mat")]
    };
    let bytes = candidates
        .iter()
        .find_map(|c| vfs::lookup(c))
        .ok_or_else(|| file_missing(base, line, col))?;

    let vars: Vec<(String, Value)> = if base.ends_with(".mat") {
        matv5::read_mat(&bytes).map_err(|e| io_err(line, col, e))?
    } else {
        mcws::decode_workspace(&bytes).map_err(|e| io_err(line, col, e))?
    };
    for (name, v) in vars {
        if names.is_empty() || names.contains(&name) {
            env.set(name, v);
        }
    }
    Ok(())
}

// ---------- run ----------

/// عمق تودرتویی `run` — گارد بازگشت صریح (به‌جای اتمام پشته: پروفایل
/// release با panic=abort است و سرریز پشته اپ را می‌کشد؛ گارد جلوتر می‌ایستد).
fn run_depth<T>(f: impl FnOnce() -> Result<T, McError>) -> Result<T, McError> {
    thread_local!(static D: std::cell::Cell<u32> = const { std::cell::Cell::new(0) });
    let d = D.with(|c| c.get());
    if d >= 8 {
        return Err(McError::new(
            ids::FILE_ERROR,
            0,
            0,
            "run nested too deeply (max 8) — check for a script that runs itself",
            "run بیش از حد تو در تو شده است (حداکثر ۸) — اسکریپتی که خودش را اجرا می‌کند را بررسی کنید",
        ));
    }
    D.with(|c| c.set(d + 1));
    let out = f();
    D.with(|c| c.set(d));
    out
}

/// `run('script.m')` — منبع از VFS (فایل ثبت‌شده) یا اسکریپت ثبت‌شدهٔ
/// نشست (register_m_source)؛ اجرا در همان env جاری (سمانتیک متلب)؛
/// خروجی‌های گزاره‌های بدون «;» برمی‌گردند و نمودارهای اسکریپت به سینک
/// نشست دوباره ساطع می‌شوند تا eval بیرونی آن‌ها را جمع کند.
pub fn run_source(
    env: &mut Environment,
    name: &str,
    line: usize,
    col: usize,
) -> Result<Vec<String>, McError> {
    run_depth(|| {
        let stem = name.strip_suffix(".m").unwrap_or(name);
        // ۱) فایل در VFS؟ — با/بدون پسوند .m
        let bytes = vfs::lookup(name)
            .or_else(|| vfs::lookup(&format!("{name}.m")))
            .or_else(|| vfs::lookup(stem));
        let outcome = if let Some(bytes) = bytes {
            let source = String::from_utf8(bytes.as_ref().clone()).map_err(|_| {
                McError::new(
                    ids::FILE_ERROR,
                    line as u32,
                    col as u32,
                    format!("'{name}' is not valid UTF-8 text"),
                    format!("فایل «{name}» متن UTF-8 معتبر نیست"),
                )
            })?;
            let program = mc_parser::parse(&source)?;
            crate::eval_program(&program, env)
        } else {
            // ۲) اسکریپت ثبت‌شدهٔ نشست (register_m_source / توابع فایلی)
            match env.lookup_item(stem) {
                Some(crate::env::StoredItem::Script(program)) => crate::eval_program(&program, env),
                _ => {
                    return Err(file_missing(name, line, col));
                }
            }
        };
        // نمودارهای اسکریپت را به سینک نشست برگردان (eval_program سینک را
        // تخلیه کرده؛ eval بیرونی باید ببیندشان)
        for op in &outcome.plot_ops {
            mc_plot::session::emit(op.clone());
        }
        match outcome.error {
            Some(e) => Err(e),
            None => Ok(outcome.outputs.into_iter().map(|o| o.text).collect()),
        }
    })
}

// ---------- readmatrix / writematrix ----------

/// تابع رجیستری `readmatrix('file')` — متن محدودشده از VFS → Double.
/// (گزینه‌های name-value در 5.5 — دلتای مستند.)
pub fn readmatrix_fn(args: &[Value], line: usize) -> Result<Value, McError> {
    let name = str_arg(args, line, "readmatrix", "filename")?;
    let bytes = vfs::lookup(&name).ok_or_else(|| file_missing(&name, line, 0))?;
    let text = String::from_utf8(bytes.as_ref().clone()).map_err(|_| {
        McError::new(
            ids::FILE_ERROR,
            line as u32,
            0,
            "file is not valid UTF-8 text",
            "فایل متن UTF-8 معتبر نیست",
        )
    })?;
    let m = delim::read_matrix_text(&text).map_err(|e| io_err(line, 0, e))?;
    Ok(Value::Double(m))
}

/// `readtable` — ارجاع صادقانه (نوع table = E4؛ پیمان ضد-انحراف سند ۱۴ §۶).
pub fn readtable_fn(args: &[Value], line: usize) -> Result<Value, McError> {
    let _ = args;
    Err(McError::new(
        ids::UNSUPPORTED,
        line as u32,
        0,
        "readtable needs the table type, which is not in the language yet (E-phase) — use readmatrix for numeric files",
        "readtable به نوع table نیاز دارد که هنوز در زبان نیست (فاز E) — برای فایل‌های عددی از readmatrix استفاده کنید",
    ))
}

fn str_arg(args: &[Value], line: usize, func: &str, what: &str) -> Result<String, McError> {
    match args.first() {
        Some(Value::Char(s)) => Ok(s.clone()),
        _ => Err(McError::new(
            ids::TYPE_ERROR,
            line as u32,
            0,
            format!("{func}: {what} must be a char, e.g. {func}('data.csv')"),
            format!("{func}: {what} باید کاراکتری باشد، مثل {func}('data.csv')"),
        )),
    }
}

/// گزارهٔ `writematrix(A)` / `writematrix(A, 'file')` — نام فایل از نام
/// متغیر (رفتار متلب؛ اگر نشود «matrix.txt»)، جداساز برحسب پسوند، ~۱۵ رقم
/// معنادار؛ خروجی در VFS؛ سکوت.
pub fn writematrix_apply(
    value: &Value,
    file: Option<&str>,
    fallback_name: &str,
    line: usize,
    col: usize,
) -> Result<(), McError> {
    let Value::Double(m) = value else {
        return Err(McError::new(
            ids::TYPE_ERROR,
            line as u32,
            col as u32,
            "writematrix: only numeric matrices can be written (char/struct files come with 5.5)",
            "writematrix: فقط ماتریس عددی نوشته می‌شود (فایل char/struct با گام 5.5)",
        ));
    };
    let base = match file {
        Some(f) => f.rsplit(['/', '\\']).next().unwrap_or(f).to_owned(),
        None => {
            let stem: String = fallback_name
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                .collect();
            if stem.is_empty() {
                "matrix.txt".to_owned()
            } else {
                format!("{stem}.txt")
            }
        }
    };
    let text = delim::write_matrix_text(m, Delimiter::for_extension(&base));
    vfs::put_output(&base, text.into_bytes());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use mc_core::matrix::Matrix;

    fn env_with() -> Environment {
        let mut env = Environment::new();
        env.set("A", Value::Double(Matrix::from_columns(2, 3, vec![1.0; 6])));
        env.set("m2", Value::Double(Matrix::scalar(2.0)));
        env.set("z", Value::Char("hello".to_owned()));
        env.set(
            "C",
            Value::Complex(mc_core::ComplexMatrix::from_columns(
                1,
                1,
                vec![1.0],
                vec![2.0],
            )),
        );
        env
    }

    #[test]
    fn who_lists_sorted_and_headered() {
        let text = who_text(&env_with());
        assert!(text.starts_with("Your variables are:\n\n"), "{text}");
        let names_line = text.trim_end().lines().last().unwrap_or_default();
        // A < C < m2 < z (ASCII: بزرگ‌ها اول)
        assert!(names_line.contains("A") && names_line.contains("z"));
    }

    #[test]
    fn whos_table_shape_and_alignment() {
        let text = whos_text(&env_with());
        assert!(text.contains("Name"), "header");
        assert!(text.contains("Bytes"));
        assert!(text.contains("Class"));
        assert!(text.contains("Attributes"));
        // خط خالی بعد از سرصفحه (خروجی واقعی متلب)
        let mut lines = text.lines();
        let hdr = lines.next().unwrap_or_default();
        assert!(hdr.contains("Name"));
        assert!(lines.next().unwrap_or_default().is_empty(), "blank line");
        // complex در Attributes؛ struct نداریم اینجا
        assert!(text.contains("complex"));
        assert!(text.contains("char"));
        assert!(text.contains("double"));
        assert!(!text.contains("struct"));
    }

    #[test]
    fn clear_selective_and_all() {
        let mut env = env_with();
        assert_eq!(clear_apply(&mut env, &["A".to_owned()]), 1);
        assert!(env.get("A").is_none() && env.get("z").is_some());
        assert_eq!(clear_apply(&mut env, &[]), 3);
        assert!(env.get("z").is_none());
    }

    #[test]
    fn save_load_roundtrip_via_vfs() {
        vfs::clear_all();
        let mut env = env_with();
        save_apply(&mut env, Some("ws1".to_owned()), &[], 1, 1).expect("save");
        let outs = vfs::take_outputs();
        assert_eq!(outs.len(), 1);
        assert_eq!(outs[0].0, "ws1.mcws", "پسوند قالب می‌چسبد");
        // میزبان می‌نویسد و دوباره ثبت می‌کند
        vfs::register("ws1.mcws", outs[0].1.clone());
        let mut fresh = Environment::new();
        load_apply(&mut fresh, "ws1", &[], 1, 1).expect("load");
        assert_eq!(
            fresh.get("A"),
            Some(&Value::Double(Matrix::from_columns(2, 3, vec![1.0; 6])))
        );
        vfs::clear_all();
    }

    #[test]
    fn save_mat_name_rejected_and_default_name() {
        vfs::clear_all();
        let mut env = env_with();
        let e = save_apply(&mut env, Some("x.mat".to_owned()), &[], 1, 1).unwrap_err();
        assert!(e.message_en.contains("5.5"));
        save_apply(&mut env, None, &[], 1, 1).expect("default save");
        let outs = vfs::take_outputs();
        assert_eq!(outs[0].0, "matlab.mcws");
        vfs::clear_all();
    }

    #[test]
    fn load_missing_file_mentions_registered() {
        vfs::clear_all();
        vfs::register("only.txt", b"1,2".to_vec());
        let mut env = Environment::new();
        let e = load_apply(&mut env, "nope", &[], 1, 1).unwrap_err();
        assert!(e.message_en.contains("only.txt"));
        assert!(e.message_fa.contains("MathCraft"));
        vfs::clear_all();
    }

    #[test]
    fn writematrix_names_and_outputs() {
        vfs::clear_all();
        let m = Value::Double(Matrix::from_columns(2, 2, vec![1.0, 2.0, 3.0, 4.0]));
        writematrix_apply(&m, None, "results", 1, 1).expect("w");
        writematrix_apply(&m, Some("out/t.tsv"), "ignored", 1, 1).expect("w2");
        let outs = vfs::take_outputs();
        assert_eq!(outs[0].0, "results.txt");
        assert_eq!(outs[0].1, b"1,3\n2,4\n");
        assert_eq!(outs[1].0, "t.tsv"); // basename
        assert_eq!(outs[1].1, b"1\t3\n2\t4\n");
        vfs::clear_all();
    }

    #[test]
    fn readmatrix_reads_registered_text() {
        vfs::clear_all();
        vfs::register("d.csv", b"a,b\n1,2\n3,4\n".to_vec());
        let v = readmatrix_fn(&[Value::Char("d.csv".to_owned())], 1).expect("rm");
        let Value::Double(m) = v else { panic!("kind") };
        assert_eq!((m.rows(), m.cols()), (2, 2));
        assert_eq!(m.as_column_slice(), &[1.0, 3.0, 2.0, 4.0]);
        vfs::clear_all();
    }

    #[test]
    fn readtable_points_to_readmatrix() {
        let e = readtable_fn(&[Value::Char("x.csv".to_owned())], 1).unwrap_err();
        assert!(e.message_en.contains("readmatrix"));
    }
}
