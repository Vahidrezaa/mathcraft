//! دستورهای کنسولی `mc` — سمت کلاینت‌اند، به موتور ارسال نمی‌شوند (P0.1).
//!
//! سمانتیک متلب: `who/whos/clear/clc` در نسخه کامل زبان، built-in هستند
//! (سند ۰۳ بخش ۱۲)؛ اینجا به‌عنوان دستورهای کنسول روی همان توابع الحاقی
//! قرارداد (`workspace_vars` / `clear_workspace` — سند ۰۲ بخش ۳) پیاده
//! شده‌اند تا P0 بدون بازکردن دامنه زبان، تجربه کامل REPL بدهد.

use mc_ffi::api::workspace_vars;

use crate::render::Lang;

/// نتیجه تشخیص دستور کنسولی.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Meta {
    /// خط کنسولی نیست → به موتور ارسال شود.
    None,
    /// کنسول خودش انجام داد → ادامه حلقه.
    Handled,
    /// تور دمو اجرا شود.
    Demo,
    /// خروج از برنامه.
    Exit,
}

pub fn dispatch(line: &str, lang: Lang) -> Meta {
    match line {
        "quit" | "exit" => Meta::Exit,
        "help" | "?" => {
            print!("{}", help_text(lang));
            Meta::Handled
        }
        "who" => {
            print_who(lang);
            Meta::Handled
        }
        "whos" => {
            print_whos(lang);
            Meta::Handled
        }
        "clear" => {
            let n = mc_ffi::api::clear_workspace();
            match lang {
                Lang::En => println!("workspace cleared ({n} variables)."),
                Lang::Fa => println!("فضای کاری پاک شد ({n} متغیر)."),
            }
            Meta::Handled
        }
        // سمانتیک متلب: clear functions / clear all — تعریف‌ها جدا از متغیرها
        "clear functions" => {
            let n = mc_ffi::api::clear_m_functions();
            match lang {
                Lang::En => println!("{n} registered definition(s) cleared."),
                Lang::Fa => println!("{n} تعریف ثبت‌شده پاک شد."),
            }
            Meta::Handled
        }
        "clear all" => {
            let v = mc_ffi::api::clear_workspace();
            let f = mc_ffi::api::clear_m_functions();
            match lang {
                Lang::En => println!("workspace cleared ({v} variables, {f} definitions)."),
                Lang::Fa => println!("فضای کاری پاک شد ({v} متغیر، {f} تعریف)."),
            }
            Meta::Handled
        }
        "clc" => {
            // ANSI clear + home — ترمینال‌های اندروید/دسکتاپ استاندارد پشتیبانی می‌کنند.
            print!("\x1b[2J\x1b[H");
            print!("{}", banner(lang));
            Meta::Handled
        }
        "demo" => Meta::Demo,
        "functions" => {
            print_functions(lang);
            Meta::Handled
        }
        _ if line.starts_with("which ") => {
            let name = line["which ".len()..].trim();
            print_which(name, lang);
            Meta::Handled
        }
        _ if line.starts_with("addpath ") => {
            let dir = line["addpath ".len()..].trim();
            add_path(dir, lang);
            Meta::Handled
        }
        _ => Meta::None,
    }
}

pub fn banner(lang: Lang) -> String {
    match lang {
        Lang::En => format!(
            "MathCraft {} — Rust computational kernel (P0)\nEngine API contract v{} | GPL-3.0-or-later | offline by design\nType 'help' for commands, 'demo' for a quick tour, 'quit' to exit.\n",
            crate::VERSION,
            crate::API_VERSION
        ),
        Lang::Fa => format!(
            "MathCraft {} — موتور محاسباتی Rust (فاز P0)\nقرارداد موتور v{} | GPL-3.0-or-later | آفلاین به‌واسطه طراحی\nراهنما: help — تور کوتاه: demo — خروج: quit\n",
            crate::VERSION,
            crate::API_VERSION
        ),
    }
}

pub fn farewell(lang: Lang) -> &'static str {
    match lang {
        Lang::En => "Goodbye.",
        Lang::Fa => "خدانگهدار.",
    }
}

fn help_text(lang: Lang) -> String {
    match lang {
        Lang::En => {
            "Console commands (handled locally, not sent to the engine):\n  help            show this help\n  demo            run a short guided tour\n  who             list current variables\n  whos            variables with size / bytes / class\n  functions       list registered .m functions / scripts\n  which NAME      resolve NAME: variable / function / script / builtin\n  addpath DIR     register every .m file in DIR (file name = callable name)\n  clear           remove all variables (like MATLAB clear)\n  clear functions remove registered .m definitions (like MATLAB)\n  clear all       remove variables and definitions\n  clc             clear the screen\n  quit / exit     leave MathCraft\n\nEngine: type any MATLAB-style expression at '>>', e.g.\n  1+1        x = 7        y = x^2 - 2*x + 1        y        1/0        2^3^2\n  function files: addpath a directory, then call them like sq(5)\n\nOptions: mc [-e|--eval CODE] [--lang fa|en] [-h|--help] [-V|--version]\n"
                .to_owned()
        }
        Lang::Fa => {
            "دستورهای کنسول (محلی — به موتور ارسال نمی‌شوند):\n  help            نمایش همین راهنما\n  demo            تور کوتاه آموزشی\n  who             فهرست متغیرهای فعلی\n  whos            متغیرها با اندازه / بایت / کلاس\n  functions       فهرست توابع/اسکریپت‌های .m ثبت‌شده\n  which NAME      رزولوشن نام: متغیر / تابع / اسکریپت / builtin\n  addpath DIR     ثبت همه فایل‌های .m پوشه (نام فایل = نام فراخوانی‌پذیر)\n  clear           پاک‌کردن همه متغیرها (مثل clear متلب)\n  clear functions پاک‌کردن تعریف‌های ثبت‌شده (مثل متلب)\n  clear all       پاک‌کردن متغیرها و تعریف‌ها\n  clc             پاک‌کردن صفحه\n  quit / exit     خروج از MathCraft\n\nموتور: هر عبارت به سبک متلب جلوی >> بنویسید، مثل:\n  1+1        x = 7        y = x^2 - 2*x + 1        y        1/0        2^3^2\n  فایل‌های تابع: addpath یک پوشه، سپس فراخوانی مثل sq(5)\n\nگزینه‌ها: mc [-e|--eval CODE] [--lang fa|en] [-h|--help] [-V|--version]\n"
                .to_owned()
        }
    }
}

fn print_who(lang: Lang) {
    let vars = workspace_vars();
    if vars.is_empty() {
        match lang {
            Lang::En => println!("workspace is empty."),
            Lang::Fa => println!("فضای کاری خالی است."),
        }
        return;
    }
    match lang {
        Lang::En => println!("Your variables are:"),
        Lang::Fa => println!("متغیرهای شما:"),
    }
    for chunk in vars.chunks(4) {
        let names: Vec<&str> = chunk.iter().map(|v| v.name.as_str()).collect();
        println!("{}", names.join("    "));
    }
}

fn print_whos(lang: Lang) {
    let vars = workspace_vars();
    if vars.is_empty() {
        print_who(lang);
        return;
    }
    let name_w = vars.iter().map(|v| v.name.len()).max().unwrap_or(4).max(4);
    println!(
        "  {:<name_w$}  {:<9}  {:>6}  Class",
        "Name",
        "Size",
        "Bytes",
        name_w = name_w
    );
    for v in &vars {
        let (rows, cols) = (
            v.shape.first().copied().unwrap_or(0),
            v.shape.get(1).copied().unwrap_or(0),
        );
        println!(
            "  {:<name_w$}  {:<9}  {:>6}  {}",
            v.name,
            format!("{rows}x{cols}"),
            (rows as u64) * (cols as u64) * 8,
            v.class,
            name_w = name_w
        );
    }
}

/// فهرست توابع/اسکریپت‌های ثبت‌شده — مکمل `who` برای تعریف‌ها.
fn print_functions(lang: Lang) {
    let fns = mc_ffi::api::m_functions();
    if fns.is_empty() {
        match lang {
            Lang::En => println!("no .m definitions registered. Use: addpath <dir>",),
            Lang::Fa => println!("هیچ تعریف .m ثبت نشده. استفاده: addpath <پوشه>"),
        }
        return;
    }
    match lang {
        Lang::En => println!("Registered .m definitions:"),
        Lang::Fa => println!("تعریف‌های .m ثبت‌شده:"),
    }
    for f in &fns {
        if f.kind == "function" {
            println!(
                "  {}  function  in: {}  out: {}",
                f.name, f.inputs, f.outputs
            );
        } else {
            println!("  {}  script", f.name);
        }
    }
}

/// `which <name>` — رزولوشن متلب §9 خلاصه‌شده (سند ۰۲ بخش ۳).
fn print_which(name: &str, lang: Lang) {
    let _ = lang;
    match mc_ffi::api::which_name(name.to_owned()) {
        Some(kind) => match kind.as_str() {
            "variable" => println!("'{name}' is a variable."),
            "function" => println!("'{name}' is a registered .m function."),
            "script" => println!("'{name}' is a registered .m script."),
            "builtin" => println!("'{name}' is a built-in function."),
            other => println!("'{name}': {other}."),
        },
        None => println!("'{name}' is undefined."),
    }
}

/// `addpath <dir>` — ثبت همه فایل‌های .m یک پوشه در مخزن نشست (سمانتیک متلب:
/// نام فایل = نام فراخوانی‌پذیر). فایل‌های معیار ثبت، خطادارها گزارش می‌شوند.
fn add_path(dir: &str, lang: Lang) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(e) => {
            match lang {
                Lang::En => println!("addpath: cannot read directory '{dir}': {e}"),
                Lang::Fa => println!("addpath: خواندن پوشه «{dir}» ممکن نیست: {e}"),
            }
            return;
        }
    };
    let mut files: Vec<std::path::PathBuf> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().map(|x| x == "m").unwrap_or(false))
        .collect();
    files.sort(); // ترتیب قطعی
    if files.is_empty() {
        match lang {
            Lang::En => println!("addpath '{dir}': no .m files found."),
            Lang::Fa => println!("addpath «{dir}»: هیچ فایل .m پیدا نشد."),
        }
        return;
    }
    let mut ok = 0usize;
    let mut fns = 0usize;
    let mut errs = 0usize;
    for p in files {
        let stem = match p.file_stem().and_then(|s| s.to_str()) {
            Some(s) => s.to_owned(),
            None => {
                errs += 1;
                continue;
            }
        };
        let source = match std::fs::read_to_string(&p) {
            Ok(s) => s,
            Err(e) => {
                errs += 1;
                println!("addpath: {stem}.m — read error: {e}");
                continue;
            }
        };
        let r = mc_ffi::api::register_m_source(stem.clone(), source);
        match r.error {
            None => {
                ok += 1;
                fns += r.functions.len();
            }
            Some(err) => {
                errs += 1;
                println!("addpath: {stem}.m — {}", err.message_en);
            }
        }
    }
    match lang {
        Lang::En => println!(
            "addpath '{dir}': {ok} file(s) registered ({fns} function(s)), {errs} error(s)."
        ),
        Lang::Fa => println!("addpath «{dir}»: {ok} فایل ثبت شد ({fns} تابع)، {errs} خطا."),
    }
}

pub fn demo_lines() -> &'static [&'static str] {
    &["1+1", "x = 7", "y = x^2 - 2*x + 1", "y", "1/0", "2^3^2"]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// نشست موتور سراسری است — تست‌های state-دار باید سریالی اجرا شوند
    /// (همان الگوی سند ۰۲ بخش ۴ که در mc-ffi هم به‌کار رفته).
    fn session_lock() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: std::sync::OnceLock<std::sync::Mutex<()>> = std::sync::OnceLock::new();
        LOCK.get_or_init(|| std::sync::Mutex::new(()))
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    #[test]
    fn meta_dispatch_recognizes_local_commands() {
        let _session = session_lock();
        assert_eq!(dispatch("quit", Lang::En), Meta::Exit);
        assert_eq!(dispatch("exit", Lang::Fa), Meta::Exit);
        assert_eq!(dispatch("demo", Lang::En), Meta::Demo);
        assert_eq!(dispatch("clear", Lang::En), Meta::Handled);
        assert_eq!(dispatch("clear functions", Lang::En), Meta::Handled);
        assert_eq!(dispatch("clear all", Lang::En), Meta::Handled);
        assert_eq!(dispatch("clc", Lang::En), Meta::Handled);
        assert_eq!(dispatch("help", Lang::En), Meta::Handled);
        assert_eq!(dispatch("who", Lang::En), Meta::Handled);
        assert_eq!(dispatch("whos", Lang::En), Meta::Handled);
        assert_eq!(dispatch("functions", Lang::En), Meta::Handled);
        assert_eq!(dispatch("which sq", Lang::En), Meta::Handled);
        assert_eq!(dispatch("which", Lang::En), Meta::None); // بدون آرگومان = به موتور
        assert_eq!(
            dispatch("addpath /nonexistent-dir-mc", Lang::En),
            Meta::Handled
        );
        assert_eq!(dispatch("help = 5", Lang::En), Meta::None);
        assert_eq!(dispatch("1+1", Lang::En), Meta::None);
        assert_eq!(dispatch("Clear", Lang::En), Meta::None); // فقط حروف کوچک
    }

    #[test]
    fn which_reports_resolution_order() {
        let _session = session_lock();
        // نشست سراسری است — ترتیبی و با پاک‌سازی اول و آخر
        let _ = mc_ffi::api::clear_workspace();
        let _ = mc_ffi::api::clear_m_functions();
        let _ = mc_ffi::api::evaluate_line("xv = 1".to_owned());
        let r = mc_ffi::api::register_m_source(
            "sq".to_owned(),
            "function y = sq(x); y = x^2; end".to_owned(),
        );
        assert!(r.error.is_none());
        let _ = mc_ffi::api::register_m_source("dosetup".to_owned(), "k = 5".to_owned());
        assert_eq!(
            mc_ffi::api::which_name("xv".to_owned()).as_deref(),
            Some("variable")
        );
        assert_eq!(
            mc_ffi::api::which_name("sq".to_owned()).as_deref(),
            Some("function")
        );
        assert_eq!(
            mc_ffi::api::which_name("dosetup".to_owned()).as_deref(),
            Some("script")
        );
        assert_eq!(
            mc_ffi::api::which_name("sin".to_owned()).as_deref(),
            Some("builtin")
        );
        assert_eq!(mc_ffi::api::which_name("nope".to_owned()), None);
        let _ = mc_ffi::api::clear_workspace();
        let _ = mc_ffi::api::clear_m_functions();
    }

    #[test]
    fn who_and_whos_handle_empty_workspace() {
        let _session = session_lock();
        let _ = mc_ffi::api::clear_workspace();
        let vars = workspace_vars();
        assert!(vars.is_empty());
        // نباید panic کنند — خروجی فقط متن است.
        print_who(Lang::En);
        print_whos(Lang::Fa);
    }

    #[test]
    fn banner_contains_version_and_contract() {
        let b = banner(Lang::En);
        assert!(b.contains(crate::VERSION));
        assert!(b.contains("contract v2"));
    }

    #[test]
    fn demo_lines_are_p0_safe_and_deterministic() {
        let lines = demo_lines();
        assert_eq!(lines.first().copied(), Some("1+1"));
        assert!(lines.contains(&"2^3^2"));
        assert!(lines.len() == 6);
    }
}
