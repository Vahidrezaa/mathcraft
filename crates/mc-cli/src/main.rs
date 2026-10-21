//! `mc` — کنسول بومی MathCraft (P0.1 — سند ۰۲ بخش ۲، سند ۰۷ بخش ۲).
//!
//! فرانت‌اند دوم موتور روی همان قرارداد عمومی mc-ffi: بدون Flutter، بدون
//! وابستگی خارجی، مناسب Termux/دسکتاپ. همه محاسبات در Rust می‌ماند؛ این
//! کرِیت فقط «ارتباط با کاربر ترمینال» است.
//!
//! استفاده:
//! - `mc` → REPL تعاملی (با TTY: بنر + پرامپت؛ با پایپ: فقط نتیجه‌ها)
//! - `mc -e "1+1"` → اجرای یک‌باره با کد خروج ۰/۱ (مناسب اسکریپت)
//! - `mc --lang fa` یا `MC_LANG=fa` → پیام‌های فارسی

#![cfg_attr(
    not(test),
    warn(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::todo,
        clippy::unimplemented
    )
)]

mod commands;
mod render;

use std::io::{self, BufRead, IsTerminal, Write};
use std::process::ExitCode;

use mc_ffi::api::{evaluate_line, JobStatus};

use crate::render::{render_result, Lang, Stream};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const API_VERSION: u32 = mc_ffi::api::API_CONTRACT_VERSION;

struct Args {
    eval: Option<String>,
    lang: Lang,
    help: bool,
    version: bool,
}

fn main() -> ExitCode {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let args = match parse_args(&argv) {
        Ok(a) => a,
        Err(msg) => {
            eprintln!("mc: {msg}");
            eprintln!("try 'mc --help'");
            return ExitCode::from(2);
        }
    };
    if args.help {
        print_help();
        return ExitCode::SUCCESS;
    }
    if args.version {
        println!("MathCraft engine {VERSION} (mc-ffi contract v{API_VERSION})");
        return ExitCode::SUCCESS;
    }
    match args.eval {
        Some(code) => run_one_shot(&code, args.lang),
        None => run_repl(args.lang),
    }
}

/// پارسر آرگومان‌ها — بدون کتابخانه خارجی؛ خطاها با پیام صادقانه.
fn parse_args(argv: &[String]) -> Result<Args, String> {
    let mut args = Args {
        eval: None,
        lang: Lang::from_env(),
        help: false,
        version: false,
    };
    let mut i = 0usize;
    while i < argv.len() {
        let a = argv[i].as_str();
        if a == "-e" || a == "--eval" {
            i += 1;
            let code = argv
                .get(i)
                .ok_or_else(|| "--eval needs a CODE argument".to_owned())?;
            args.eval = Some(code.clone());
        } else if let Some(code) = a.strip_prefix("--eval=") {
            args.eval = Some(code.to_owned());
        } else if let Some(code) = a.strip_prefix("-e=") {
            args.eval = Some(code.to_owned());
        } else if let Some(l) = a.strip_prefix("--lang=") {
            args.lang = match l {
                "fa" | "fa_IR" => Lang::Fa,
                "en" => Lang::En,
                other => return Err(format!("unknown language '{other}' (use fa or en)")),
            };
        } else if a == "--lang" {
            i += 1;
            let l = argv
                .get(i)
                .ok_or_else(|| "--lang needs 'fa' or 'en'".to_owned())?;
            args.lang = match l.as_str() {
                "fa" | "fa_IR" => Lang::Fa,
                "en" => Lang::En,
                other => return Err(format!("unknown language '{other}' (use fa or en)")),
            };
        } else if a == "-h" || a == "--help" {
            args.help = true;
        } else if a == "-V" || a == "--version" {
            args.version = true;
        } else {
            return Err(format!("unknown argument '{a}'"));
        }
        i += 1;
    }
    Ok(args)
}

/// اجرای یک‌باره برای اسکریپت‌ها: خروجی متن، خطا در stderr، کد خروج ۰/۱.
fn run_one_shot(code: &str, lang: Lang) -> ExitCode {
    let result = evaluate_line(code.to_owned());
    for (stream, text) in render_result(&result, lang) {
        match stream {
            Stream::Out => println!("{text}"),
            Stream::Err => eprintln!("{text}"),
        }
    }
    match result.status {
        JobStatus::Done => ExitCode::SUCCESS,
        JobStatus::Cancelled | JobStatus::Failed => ExitCode::from(1),
    }
}

/// حلقه REPL — سمانتیک کنسول متلب: پرامپت `>>`، اجرای فوری، ماندگاری workspace.
fn run_repl(lang: Lang) -> ExitCode {
    let interactive = io::stdin().is_terminal();
    if interactive {
        print!("{}", commands::banner(lang));
        let _ = io::stdout().flush();
    }
    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();
    loop {
        if interactive {
            print!(">> ");
            let _ = io::stdout().flush();
        }
        let line = match lines.next() {
            Some(Ok(l)) => l,
            Some(Err(e)) => {
                eprintln!("mc: input error: {e}");
                break;
            }
            None => break,
        };
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        // دستورهای کنسولی روی خودِ خط (بدون حساس به بزرگی حرفِ *کلیدواژه*)؛
        // آرگومان‌ها (مسیر/نام) اصل بزرگی حرف را حفظ می‌کنند — dispatch خودش
        // کلیدواژه‌ها را دقیق و lowercase می‌شناسد؛ خطوط دیگر عیناً به موتور می‌روند.
        match commands::dispatch(trimmed, lang) {
            commands::Meta::Exit => break,
            commands::Meta::Handled => continue,
            commands::Meta::Demo => {
                run_demo(lang);
                continue;
            }
            commands::Meta::None => {}
        }
        let result = evaluate_line(trimmed.to_owned());
        for (stream, text) in render_result(&result, lang) {
            match stream {
                Stream::Out => println!("{text}"),
                Stream::Err => eprintln!("{text}"),
            }
        }
    }
    if interactive {
        println!("{}", commands::farewell(lang));
    }
    ExitCode::SUCCESS
}

/// تور کوتاه — خطوط قطعیِ امن برای P0 (محاسبات اسکالری + IEEE + توان چپ‌گرا).
fn run_demo(lang: Lang) {
    match lang {
        Lang::En => println!("--- demo: a deterministic tour of the engine ---"),
        Lang::Fa => println!("--- دمو: تور قطعی موتور ---"),
    }
    for line in commands::demo_lines() {
        println!(">> {line}");
        let result = evaluate_line((*line).to_owned());
        for (stream, text) in render_result(&result, lang) {
            match stream {
                Stream::Out => println!("{text}"),
                Stream::Err => eprintln!("{text}"),
            }
        }
    }
}

fn print_help() {
    println!(
        "MathCraft `mc` console — native front-end on the mc-ffi engine contract\n\n\
Usage: mc [OPTIONS]\n\
  (no args)        interactive REPL (banner + prompt when stdin is a TTY)\n\
  -e, --eval CODE  evaluate CODE once and exit (exit code 0 = Done, 1 = Failed)\n\
      --lang LANG  console messages: en (default) or fa (or MC_LANG=fa)\n\
  -h, --help       show this help\n\
  -V, --version    show version\n\n\
Engine examples: 1+1 | x = 7 | y = x^2 - 2*x + 1 | 1/0 | 2^3^2"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(items: &[&str]) -> Vec<String> {
        items.iter().map(|x| (*x).to_owned()).collect()
    }

    #[test]
    fn parse_args_defaults() {
        let a = parse_args(&s(&[])).ok();
        let a = a.unwrap_or_else(|| panic!("empty argv must parse"));
        assert!(a.eval.is_none());
        assert!(!a.help);
        assert!(!a.version);
    }

    #[test]
    fn parse_args_eval_forms_and_lang() {
        let a = parse_args(&s(&["-e", "1+1"])).ok();
        let a = a.unwrap_or_else(|| panic!("-e CODE must parse"));
        assert_eq!(a.eval.as_deref(), Some("1+1"));

        let a = parse_args(&s(&["--eval=2^3^2"])).ok();
        let a = a.unwrap_or_else(|| panic!("--eval=CODE must parse"));
        assert_eq!(a.eval.as_deref(), Some("2^3^2"));

        let a = parse_args(&s(&["--lang", "fa"])).ok();
        let a = a.unwrap_or_else(|| panic!("--lang fa must parse"));
        assert_eq!(a.lang, Lang::Fa);

        let a = parse_args(&s(&["--lang=en"])).ok();
        let a = a.unwrap_or_else(|| panic!("--lang=en must parse"));
        assert_eq!(a.lang, Lang::En);
    }

    #[test]
    fn parse_args_rejects_unknown_and_missing_values() {
        assert!(parse_args(&s(&["--wat"])).is_err());
        assert!(parse_args(&s(&["-e"])).is_err());
        assert!(parse_args(&s(&["--lang"])).is_err());
        assert!(parse_args(&s(&["--lang", "fr"])).is_err());
    }

    #[test]
    fn help_and_version_flags() {
        let a = parse_args(&s(&["-V"])).ok();
        let a = a.unwrap_or_else(|| panic!("-V must parse"));
        assert!(a.version);
        let a = parse_args(&s(&["--help"])).ok();
        let a = a.unwrap_or_else(|| panic!("--help must parse"));
        assert!(a.help);
    }
}
