//! disp/fprintf — خروجی صریح کنسول (ROADMAP 2.4 — آیتم ۸ ممیزی خارجی).
//!
//! - `disp(v)`: مثل متلب بدون نام متغیر و بدون کوتیشن برای رشته چاپ می‌کند
//!   و یک سطر جدید می‌گذارد. خروجی مستقل از `;` است و `ans` را ست نمی‌کند.
//! - `fprintf(fmt, ...)`: پارسر دستی قالب — پشتیبانی‌شده: `%d %i %f %s %%`
//!   با پرچم‌های `- 0 +`، عرض و `.دقت`، و گریزهای `\n \t \r \\`. مشخص‌کنندهٔ
//!   ناشناخته → خطای صادقانه MC:E9xxx (قرارداد سند ۰۳ §14 «honest error»).
//!   هیچ panic/unwrap در مسیر محصول (سند ۰۲ §9).

use crate::format::{format_value_spec, DisplaySpec};
use mc_core::{ids, McError, Value};

/// متن `disp` — دقیقاً یک آرگومان.
pub fn disp_text(
    args: &[Value],
    line: u32,
    col: u32,
    spec: &DisplaySpec,
) -> Result<String, McError> {
    if args.len() != 1 {
        return Err(McError::new(
            ids::PARSE_ERROR,
            line,
            col,
            "disp: exactly one argument is required.",
            "disp: دقیقاً یک آرگومان لازم است.",
        ));
    }
    Ok(match &args[0] {
        Value::Char(s) => format!("{s}\n"),
        v => format_value_spec(v, spec),
    })
}

/// متن `fprintf` — رشتهٔ قالب + آرگومان‌ها؛ بدون افزودن سطر جدید (متلب).
pub fn fprintf_text(args: &[Value], line: u32, col: u32) -> Result<String, McError> {
    if args.is_empty() {
        return Err(McError::new(
            ids::PARSE_ERROR,
            line,
            col,
            "fprintf: a format string is required.",
            "fprintf: رشتهٔ قالب لازم است.",
        ));
    }
    let fmt = match &args[0] {
        Value::Char(s) => s.clone(),
        _ => {
            return Err(McError::new(
                ids::PARSE_ERROR,
                line,
                col,
                "fprintf: the format must be a char string like 'x = %d\\n'.",
                "fprintf: قالب باید رشتهٔ کاراکتری مثل 'x = %d\\n' باشد.",
            ))
        }
    };
    let chars: Vec<char> = fmt.chars().collect();
    let mut out = String::new();
    let mut argi = 1usize; // ایندکس آرگومان بعدی مصرف‌شونده
    let mut i = 0usize;
    while i < chars.len() {
        let c = chars[i];
        // گریزهای سادگی — بقیهٔ کاراکترها عیناً
        if c == '\\' && i + 1 < chars.len() {
            let esc = chars[i + 1];
            let mapped = match esc {
                'n' => Some('\n'),
                't' => Some('\t'),
                'r' => Some('\r'),
                '\\' => Some('\\'),
                _ => None,
            };
            match mapped {
                Some(ch) => {
                    out.push(ch);
                    i += 2;
                    continue;
                }
                None => {
                    out.push(c);
                    i += 1;
                    continue;
                }
            }
        }
        if c != '%' {
            out.push(c);
            i += 1;
            continue;
        }
        // %%
        if i + 1 < chars.len() && chars[i + 1] == '%' {
            out.push('%');
            i += 2;
            continue;
        }
        // %[flags][width][.precision]spec
        let mut j = i + 1;
        let mut flags = String::new();
        while j < chars.len() && matches!(chars[j], '-' | '0' | '+' | ' ') {
            flags.push(chars[j]);
            j += 1;
        }
        let mut width = String::new();
        while j < chars.len() && chars[j].is_ascii_digit() {
            width.push(chars[j]);
            j += 1;
        }
        let mut prec: Option<usize> = None;
        if j < chars.len() && chars[j] == '.' {
            j += 1;
            let mut p = String::new();
            while j < chars.len() && chars[j].is_ascii_digit() {
                p.push(chars[j]);
                j += 1;
            }
            prec = Some(p.parse().unwrap_or(0));
        }
        if j >= chars.len() {
            return Err(McError::new(
                ids::PARSE_ERROR,
                line,
                col,
                "fprintf: incomplete format specifier (missing conversion character).",
                "fprintf: مشخص‌کنندهٔ قالب ناقص است (نویسهٔ تبدیل نیامده).",
            ));
        }
        let spec_ch = chars[j];
        i = j + 1;
        let piece = match spec_ch {
            'd' | 'i' => {
                let v = next_number(args, &mut argi, spec_ch, line, col)?;
                let r = v.round();
                let s = if r == 0.0 { "0".to_owned() } else { format!("{r:.0}") };
                pad(&s, &flags, &width, true)
            }
            'f' => {
                let v = next_number(args, &mut argi, spec_ch, line, col)?;
                let p = prec.unwrap_or(6); // پیش‌فرض C/MATLAB: ۶ اعشار
                pad(&format!("{v:.p$}"), &flags, &width, true)
            }
            's' => match args.get(argi) {
                Some(Value::Char(s)) => {
                    argi += 1;
                    pad(s, &flags, &width, false)
                }
                Some(_) => {
                    return Err(McError::new(
                        ids::UNSUPPORTED,
                        line,
                        col,
                        "fprintf: %s needs a char argument — use %d/%f for numbers or wrap with num2str().",
                        "fprintf: %s به آرگومان رشته‌ای نیاز دارد — برای عدد از %d/%f استفاده کنید یا با num2str() بپوشانید.",
                    ))
                }
                None => return Err(missing_arg(line, col)),
            },
            other => {
                return Err(McError::new(
                    ids::UNSUPPORTED,
                    line,
                    col,
                    format!(
                        "fprintf: '%{other}' specifier is not supported yet (supported: %d %i %f %s %%)."
                    ),
                    format!(
                        "fprintf: مشخص‌کنندهٔ '%{other}' هنوز پشتیبانی نمی‌شود (پشتیبانی‌شده: %d %i %f %s %%)."
                    ),
                ))
            }
        };
        out.push_str(&piece);
    }
    if argi < args.len() {
        return Err(McError::new(
            ids::PARSE_ERROR,
            line,
            col,
            "fprintf: more arguments than the format string consumes.",
            "fprintf: تعداد آرگومان‌ها بیش از چیزی است که رشتهٔ قالب مصرف می‌کند.",
        ));
    }
    Ok(out)
}

fn next_number(
    args: &[Value],
    argi: &mut usize,
    spec_ch: char,
    line: u32,
    col: u32,
) -> Result<f64, McError> {
    match args.get(*argi) {
        Some(Value::Double(m)) if m.is_scalar() => {
            *argi += 1;
            Ok(m.get(0, 0))
        }
        Some(_) => Err(McError::new(
            ids::PARSE_ERROR,
            line,
            col,
            format!("fprintf: '%{spec_ch}' needs a scalar numeric argument."),
            format!("fprintf: '%{spec_ch}' به آرگومان عددی اسکالر نیاز دارد."),
        )),
        None => Err(missing_arg(line, col)),
    }
}

fn missing_arg(line: u32, col: u32) -> McError {
    McError::new(
        ids::PARSE_ERROR,
        line,
        col,
        "fprintf: not enough arguments for the format string.",
        "fprintf: آرگومان‌ها برای رشتهٔ قالب کافی نیستند.",
    )
}

/// پرکردن عرض — `-` چپ‌چین؛ `0` پرکردن با صفر (فقط عددی).
fn pad(s: &str, flags: &str, width: &str, numeric: bool) -> String {
    let w: usize = width.parse().unwrap_or(0);
    let len = s.chars().count();
    if w <= len {
        return s.to_owned();
    }
    let fill = w - len;
    if flags.contains('-') {
        format!("{s}{}", " ".repeat(fill))
    } else if flags.contains('0') && numeric && !s.starts_with('-') {
        format!("{}{}", "0".repeat(fill), s)
    } else if flags.contains('0') && numeric && s.starts_with('-') {
        format!("-{}{}", "0".repeat(fill), &s[1..])
    } else {
        format!("{}{s}", " ".repeat(fill))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mc_core::Matrix;

    fn d(x: f64) -> Value {
        Value::Double(Matrix::scalar(x))
    }
    fn c(s: &str) -> Value {
        Value::Char(s.to_owned())
    }

    #[test]
    fn disp_char_raw_and_matrix_via_value() {
        assert_eq!(
            disp_text(&[c("salam")], 0, 0, &DisplaySpec::default()).unwrap(),
            "salam\n"
        );
        // ماتریس از مسیر format_value — بدون نام و بدون کوتیشن
        let t = disp_text(&[d(2.0)], 0, 0, &DisplaySpec::default()).unwrap();
        assert_eq!(t, "     2\n");
    }

    #[test]
    fn disp_arg_count_is_honest_error() {
        assert!(disp_text(&[], 0, 0, &DisplaySpec::default()).is_err());
        assert!(disp_text(&[c("a"), c("b")], 0, 0, &DisplaySpec::default()).is_err());
    }

    #[test]
    fn fprintf_basic_table() {
        // جدول تست قالب‌ها (ROADMAP 2.4 — آیتم ۸)
        let f = |fmt: &str, rest: &[Value]| {
            fprintf_text(
                &[c(fmt)]
                    .iter()
                    .chain(rest.iter())
                    .cloned()
                    .collect::<Vec<_>>(),
                0,
                0,
            )
            .unwrap()
        };
        assert_eq!(f("%d\n", &[d(42.0)]), "42\n");
        assert_eq!(f("x = %d\n", &[d(-7.0)]), "x = -7\n");
        assert_eq!(f("%f\n", &[d(1.5)]), "1.500000\n");
        assert_eq!(f("%.2f\n", &[d(1.005)]), "1.00\n");
        assert_eq!(f("%8.3f|\n", &[d(12.3456)]), "  12.346|\n");
        assert_eq!(f("%-6d|\n", &[d(42.0)]), "42    |\n");
        assert_eq!(f("%05d\n", &[d(42.0)]), "00042\n");
        assert_eq!(f("%s!\n", &[c("salam")]), "salam!\n");
        assert_eq!(f("100%%\n", &[]), "100%\n");
        assert_eq!(f("a\tb\n", &[]), "a\tb\n");
        // %d روی اعشار گرد می‌کند (رفتار متلب)
        assert_eq!(f("%d\n", &[d(2.7)]), "3\n");
    }

    #[test]
    fn fprintf_unsupported_is_honest_error() {
        assert!(fprintf_text(&[c("%e\n"), d(1.0)], 0, 0).is_err());
        assert!(fprintf_text(&[c("%g\n"), d(1.0)], 0, 0).is_err());
        assert!(fprintf_text(&[c("%s\n"), d(65.0)], 0, 0).is_err());
        assert!(fprintf_text(&[d(1.0)], 0, 0).is_err());
        assert!(fprintf_text(&[], 0, 0).is_err());
        // آرگومان کمتر از نیاز قالب
        assert!(fprintf_text(&[c("%d %d\n"), d(1.0)], 0, 0).is_err());
        // آرگومان اضافی
        assert!(fprintf_text(&[c("%d\n"), d(1.0), d(2.0)], 0, 0).is_err());
    }
}
