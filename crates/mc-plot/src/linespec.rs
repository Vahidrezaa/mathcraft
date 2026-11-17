//! پارسر LineSpec متلب — `'r--o'` ← (رنگ، خط، مارکر) (سند ۰۴ §8.1).
//!
//! گرامر: تا سه جزء در هر ترتیب — رنگ تک‌حرفی، مارکر تک‌حرفی، سبک خط
//! (`-`, `--`, `-.`, `:`). جدول پارتی متلب:
//!
//! | نمونه | رنگ | خط | مارکر |
//! |---|---|---|---|
//! | `'r--'` | قرمز | خط‌چین | — |
//! | `'b*'` | آبی | (هیچ: فقط مارکر) | ستاره |
//! | `'k.'` | مشکی | (هیچ) | نقطه |
//! | `'-.o'` | (هیچ) | خط‌نقطه | دایره |
//! | `'go'` | سبز | (هیچ) | دایره |
//!
//! غیبت جزء = «پیش‌فرض موتور» (رنگ: سیکل پالت؛ خط: صاف برای plot، هیچ برای
//! scatter؛ مارکر: هیچ) — نه «حذف صریح». `'b*'` یعنی فقط مارکر: خط `Invisible`
//! صریح برمی‌گرداند تا رندرر خط نکشد.

use crate::spec::{DashSpec, MarkerSpec, PlotColorSpec, StyleSpec};

/// نتیجهٔ پارس — هر جزء None یعنی در رشته نبود.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LineSpecParts {
    pub color: Option<PlotColorSpec>,
    pub dash: Option<DashSpec>,
    pub marker: Option<MarkerSpec>,
}

/// کاراکترهای مارکر متلب (به‌جز جدول کامل ۱۳گانهٔ DTO).
fn marker_char(c: char) -> Option<MarkerSpec> {
    match c {
        '.' => Some(MarkerSpec::Point),
        'o' => Some(MarkerSpec::Circle),
        's' => Some(MarkerSpec::Square),
        'd' => Some(MarkerSpec::Diamond),
        'x' => Some(MarkerSpec::Cross),
        '+' => Some(MarkerSpec::Plus),
        '*' => Some(MarkerSpec::Star),
        '^' => Some(MarkerSpec::TriangleUp),
        'v' => Some(MarkerSpec::TriangleDown),
        _ => None,
    }
}

/// رنگ تک‌حرفی متلب → مقدار دقیق RGB (سند ۰۴ §8.1 — پارتی متلب).
pub fn color_char(c: char) -> Option<PlotColorSpec> {
    let (r, g, b) = match c {
        'r' => (255, 0, 0),
        'g' => (0, 255, 0),
        'b' => (0, 0, 255),
        'c' => (0, 255, 255),
        'm' => (255, 0, 255),
        'y' => (255, 255, 0),
        'k' => (0, 0, 0),
        'w' => (255, 255, 255),
        _ => return None,
    };
    Some(PlotColorSpec::Rgb { r, g, b })
}

/// پارس رشتهٔ LineSpec. رشتهٔ خالی/فاصله = خالی (همه None).
/// رشتهٔ ناشناخته = `Err` با متن دوزبانه (خطای E3xxx پارسر Linespec در
/// لایهٔ فراخوان حاوی موقعیت می‌شود — این‌جا فقط «بازشناسی» است).
pub fn parse(spec: &str) -> Result<LineSpecParts, String> {
    let s = spec.trim();
    let mut parts = LineSpecParts::default();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '-' => {
                // '-' | '--' | '-.' (توکن دوتایی اول چک می‌شود)
                let dash = match chars.peek() {
                    Some('-') => {
                        chars.next();
                        DashSpec::Dashed
                    }
                    Some('.') => {
                        chars.next();
                        DashSpec::DashDot
                    }
                    _ => DashSpec::Solid,
                };
                if parts.dash.replace(dash).is_some() {
                    return Err(format!("duplicate line style in LineSpec `{spec}`"));
                }
            }
            ':' => {
                if parts.dash.replace(DashSpec::Dotted).is_some() {
                    return Err(format!("duplicate line style in LineSpec `{spec}`"));
                }
            }
            '.' | 'o' | 's' | 'd' | 'x' | '+' | '*' | '^' | 'v' => {
                // '.' هم مارکر است هم می‌تواند ادامهٔ '-.' باشد — بالای آن
                // '-' مسیر '-.' را برداشته؛ این‌جا '.' مستقل = مارکر نقطه.
                let m = marker_char(c).unwrap_or(MarkerSpec::Point);
                if parts.marker.replace(m).is_some() {
                    return Err(format!("duplicate marker in LineSpec `{spec}`"));
                }
            }
            'r' | 'g' | 'b' | 'c' | 'm' | 'y' | 'k' | 'w' => {
                let col = color_char(c).unwrap_or(PlotColorSpec::Palette(0));
                if parts.color.replace(col).is_some() {
                    return Err(format!("duplicate color in LineSpec `{spec}`"));
                }
            }
            _ => {
                return Err(format!(
                    "unrecognized LineSpec `{spec}` (expected color/marker/line chars)"
                ));
            }
        }
    }
    Ok(parts)
}

/// ادغام اجزای پارس‌شده در یک StyleSpec پایه: اجزای غایب از پایه می‌مانند
/// (پیش‌فرض موتور)، اجزای حاضر بازنویسی می‌شوند. «فقط مارکر» یعنی خط
/// نامرئی صریح.
pub fn merge_into(base: &StyleSpec, parts: LineSpecParts) -> StyleSpec {
    let mut s = base.clone();
    if let Some(c) = parts.color {
        s.color = Some(c);
    }
    if let Some(d) = parts.dash {
        s.dash = Some(d);
    }
    if parts.marker.is_some() {
        s.marker = parts.marker;
        // سبک خط غایب + مارکر حاضر = فقط مارکر (رفتار متلب 'b*')
        if parts.dash.is_none() {
            s.dash = Some(DashSpec::Invisible);
        }
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> StyleSpec {
        StyleSpec {
            color: None,
            width: None,
            dash: None,
            marker: None,
            label: None,
        }
    }

    #[test]
    fn parity_table_matlab_doc04() {
        // جدول پارتی سند ۰۴ §8.1
        let r = parse("r--").expect("valid");
        assert_eq!(r.color, Some(PlotColorSpec::Rgb { r: 255, g: 0, b: 0 }));
        assert_eq!(r.dash, Some(DashSpec::Dashed));
        assert_eq!(r.marker, None);

        let b = parse("b*").expect("valid");
        assert_eq!(b.color, Some(PlotColorSpec::Rgb { r: 0, g: 0, b: 255 }));
        assert_eq!(b.dash, None);
        assert_eq!(b.marker, Some(MarkerSpec::Star));

        let k = parse("k.").expect("valid");
        assert_eq!(k.color, Some(PlotColorSpec::Rgb { r: 0, g: 0, b: 0 }));
        assert_eq!(k.marker, Some(MarkerSpec::Point));
    }

    #[test]
    fn token_order_handles_line_before_marker() {
        let p = parse("-.o").expect("valid");
        assert_eq!(p.dash, Some(DashSpec::DashDot));
        assert_eq!(p.marker, Some(MarkerSpec::Circle));
        assert_eq!(p.color, None);

        let p = parse(":").expect("valid");
        assert_eq!(p.dash, Some(DashSpec::Dotted));
    }

    #[test]
    fn order_independent_and_full_triple() {
        let p = parse("or-").expect("valid");
        assert_eq!(p.color, Some(PlotColorSpec::Rgb { r: 255, g: 0, b: 0 }));
        assert_eq!(p.marker, Some(MarkerSpec::Circle));
        assert_eq!(p.dash, Some(DashSpec::Solid));
    }

    #[test]
    fn empty_and_whitespace_are_empty() {
        assert_eq!(parse("").expect("empty"), LineSpecParts::default());
        assert_eq!(parse("  ").expect("blank"), LineSpecParts::default());
    }

    #[test]
    fn unknown_chars_are_rejected() {
        assert!(parse("q").is_err());
        assert!(parse("rxg").is_err()); // رنگ تکراری
        assert!(parse("--:").is_err()); // سبک خط تکراری
        assert!(parse("**").is_err()); // مارکر تکراری
    }

    #[test]
    fn marker_only_spec_explicitly_hides_the_line() {
        let s = merge_into(&base(), parse("b*").expect("valid"));
        assert_eq!(s.dash, Some(DashSpec::Invisible));
        assert_eq!(s.marker, Some(MarkerSpec::Star));

        // مارکر + خط صریح: خط می‌ماند
        let s = merge_into(&base(), parse("b-*").expect("valid"));
        assert_eq!(s.dash, Some(DashSpec::Solid));
    }
}
