//! رندر رویدادهای قرارداد mc-ffi برای ترمینال — خالص و تست‌پذیر (بدون I/O).
//!
//! زبان پیام‌ها با `MC_LANG` یا `--lang fa|en` انتخاب می‌شود (پیش‌فرض: en —
//! ترمینال‌ها bidi ناپایداری دارند؛ فارسی انتخاب آگاهانه کاربر است).

use mc_ffi::api::{ConsoleEvent, EvalResult, McError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    En,
    Fa,
}

impl Lang {
    /// از `MC_LANG` — مقادیر شناخته‌شده: `fa`, `fa_IR` (بقیه → En).
    pub fn from_env() -> Self {
        match std::env::var("MC_LANG") {
            Ok(v) if v == "fa" || v == "fa_IR" => Lang::Fa,
            _ => Lang::En,
        }
    }
}

/// جریان مقصد هر سطر خروجی — خطاها به stderr می‌روند (scripting درست).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stream {
    Out,
    Err,
}

/// نتیجه ارزیابی را به سطرهای ترمینال تبدیل می‌کند — تابع خالص برای تست.
pub fn render_result(result: &EvalResult, lang: Lang) -> Vec<(Stream, String)> {
    result
        .console_events
        .iter()
        .flat_map(|event| render_event(event, lang))
        .collect()
}

fn render_event(event: &ConsoleEvent, lang: Lang) -> Vec<(Stream, String)> {
    match event {
        ConsoleEvent::Print { text } => vec![(Stream::Out, text.clone())],
        ConsoleEvent::Error { err } => vec![(Stream::Err, render_error(err, lang))],
        ConsoleEvent::Warning { text, .. } => {
            vec![(Stream::Err, format!("Warning: {text}"))]
        }
    }
}

/// خطای ساختاریافته → متن ترمینال: کد، مکان، پیام زبان فعال + did-you-mean.
pub fn render_error(err: &McError, lang: Lang) -> String {
    let head = match lang {
        Lang::En => format!(
            "error {} (line {}, col {}): {}",
            err.id, err.line, err.col, err.message_en
        ),
        Lang::Fa => format!(
            "خطا {} (سطر {}، ستون {}): {}",
            err.id, err.line, err.col, err.message_fa
        ),
    };
    match &err.suggestion {
        None => head,
        Some(s) => match lang {
            Lang::En => format!("{head}\n  Did you mean '{s}' ?"),
            Lang::Fa => format!("{head}\n  آیا منظورتان «{s}» بود؟"),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mc_ffi::api::{JobStatus, WorkspaceDelta};

    fn sample_result(events: Vec<ConsoleEvent>, status: JobStatus) -> EvalResult {
        EvalResult {
            job_id: 1,
            console_events: events,
            plots: Vec::new(),
            workspace_delta: WorkspaceDelta::default(),
            status,
            duration_ms: 0,
        }
    }

    #[test]
    fn print_goes_to_stdout_verbatim() {
        let r = sample_result(
            vec![ConsoleEvent::Print {
                text: "ans = 2".to_owned(),
            }],
            JobStatus::Done,
        );
        assert_eq!(
            render_result(&r, Lang::En),
            vec![(Stream::Out, "ans = 2".to_owned())]
        );
    }

    #[test]
    fn error_both_languages_with_suggestion() {
        let err = McError {
            id: "MC:E1001_UNDEFINED_VARIABLE".to_owned(),
            line: 1,
            col: 5,
            message_en: "undefined variable 'zv'".to_owned(),
            message_fa: "متغیر تعریف‌نشده «zv»".to_owned(),
            suggestion: Some("x".to_owned()),
        };
        let r = sample_result(vec![ConsoleEvent::Error { err }], JobStatus::Failed);
        let en = render_result(&r, Lang::En);
        assert_eq!(en[0].0, Stream::Err);
        assert!(en[0]
            .1
            .starts_with("error MC:E1001_UNDEFINED_VARIABLE (line 1, col 5):"));
        assert!(en[0].1.contains("Did you mean 'x' ?"));
        let fa = render_result(&r, Lang::Fa);
        assert!(fa[0].1.contains("خطا MC:E1001"));
        assert!(fa[0].1.contains("آیا منظورتان «x» بود؟"));
    }

    #[test]
    fn error_without_suggestion_single_line() {
        let err = McError {
            id: "MC:E3001_PARSE_ERROR".to_owned(),
            line: 1,
            col: 3,
            message_en: "unexpected token".to_owned(),
            message_fa: "توکن غیرمنتظره".to_owned(),
            suggestion: None,
        };
        let r = sample_result(vec![ConsoleEvent::Error { err }], JobStatus::Failed);
        let out = render_result(&r, Lang::En);
        assert_eq!(out.len(), 1);
        assert!(!out[0].1.contains("Did you mean"));
    }

    #[test]
    fn lang_from_env_matches_environment() {
        // منطق از محیط پیروی می‌کند؛ اگر MC_LANG ست نشده باشد پیش‌فرض En است.
        let expected = match std::env::var("MC_LANG") {
            Ok(v) if v == "fa" || v == "fa_IR" => Lang::Fa,
            _ => Lang::En,
        };
        assert_eq!(Lang::from_env(), expected);
    }
}
