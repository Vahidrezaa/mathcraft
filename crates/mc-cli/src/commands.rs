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
        "clc" => {
            // ANSI clear + home — ترمینال‌های اندروید/دسکتاپ استاندارد پشتیبانی می‌کنند.
            print!("\x1b[2J\x1b[H");
            print!("{}", banner(lang));
            Meta::Handled
        }
        "demo" => Meta::Demo,
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
            "Console commands (handled locally, not sent to the engine):\n  help            show this help\n  demo            run a short guided tour\n  who             list current variables\n  whos            variables with size / bytes / class\n  clear           remove all variables (like MATLAB clear)\n  clc             clear the screen\n  quit / exit     leave MathCraft\n\nEngine: type any MATLAB-style expression at '>>', e.g.\n  1+1        x = 7        y = x^2 - 2*x + 1        y        1/0        2^3^2\n\nOptions: mc [-e|--eval CODE] [--lang fa|en] [-h|--help] [-V|--version]\n"
                .to_owned()
        }
        Lang::Fa => {
            "دستورهای کنسول (محلی — به موتور ارسال نمی‌شوند):\n  help            نمایش همین راهنما\n  demo            تور کوتاه آموزشی\n  who             فهرست متغیرهای فعلی\n  whos           متغیرها با اندازه / بایت / کلاس\n  clear           پاک‌کردن همه متغیرها (مثل clear متلب)\n  clc             پاک‌کردن صفحه\n  quit / exit     خروج از MathCraft\n\nموتور: هر عبارت به سبک متلب جلوی >> بنویسید، مثل:\n  1+1        x = 7        y = x^2 - 2*x + 1        y        1/0        2^3^2\n\nگزینه‌ها: mc [-e|--eval CODE] [--lang fa|en] [-h|--help] [-V|--version]\n"
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

pub fn demo_lines() -> &'static [&'static str] {
    &["1+1", "x = 7", "y = x^2 - 2*x + 1", "y", "1/0", "2^3^2"]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn meta_dispatch_recognizes_local_commands() {
        assert_eq!(dispatch("quit", Lang::En), Meta::Exit);
        assert_eq!(dispatch("exit", Lang::Fa), Meta::Exit);
        assert_eq!(dispatch("demo", Lang::En), Meta::Demo);
        assert_eq!(dispatch("clear", Lang::En), Meta::Handled);
        assert_eq!(dispatch("clc", Lang::En), Meta::Handled);
        assert_eq!(dispatch("help", Lang::En), Meta::Handled);
        assert_eq!(dispatch("who", Lang::En), Meta::Handled);
        assert_eq!(dispatch("whos", Lang::En), Meta::Handled);
        assert_eq!(dispatch("help = 5", Lang::En), Meta::None);
        assert_eq!(dispatch("1+1", Lang::En), Meta::None);
        assert_eq!(dispatch("Clear", Lang::En), Meta::None); // فقط حروف کوچک
    }

    #[test]
    fn who_and_whos_handle_empty_workspace() {
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
        assert!(b.contains("contract v1"));
    }

    #[test]
    fn demo_lines_are_p0_safe_and_deterministic() {
        let lines = demo_lines();
        assert_eq!(lines.first().copied(), Some("1+1"));
        assert!(lines.contains(&"2^3^2"));
        assert!(lines.len() == 6);
    }
}
