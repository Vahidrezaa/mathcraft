//! متن محدودشده — `readmatrix`/`writematrix` (ROADMAP 3.7؛ رفتار متلب به
//! تصریح داک: کشف خودکار جداساز از «first few non-empty data lines»،
//! «automatically detects the number of header lines»، خالی/غیرعددی → NaN،
//! شناخت توکن‌های NaN/Inf/-Inf؛ نوشتن با ~۱۵ رقم معنادار و جداساز برحسب
//! پسوند فایل).
//!
//! ساده و صریح — بدون جدول/E4: فقط ماتریس double.

use mc_core::matrix::Matrix;

use crate::IoError;

/// کاندیدهای کشف خودکار (داک readmatrix: comma, tab, semicolon, pipe,
/// whitespace).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Delimiter {
    Comma,
    Tab,
    Semicolon,
    Pipe,
    /// ران‌های فاصله/تب — چند فاصله پشت‌سرهم = یک جداساز
    Whitespace,
}

impl Delimiter {
    /// پسوند فایل → جداساز نوشتن (writematrix): .csv/.txt = کاما، .tsv = تب.
    /// پسوند دیگر = کاما (پیش‌فرض داک: «comma delimited»).
    pub fn for_extension(name: &str) -> Self {
        let lower = name.to_ascii_lowercase();
        if lower.ends_with(".tsv") {
            Delimiter::Tab
        } else {
            Delimiter::Comma
        }
    }

    fn split(self, line: &str) -> Vec<String> {
        match self {
            Delimiter::Comma => line.split(',').map(|t| t.trim().to_owned()).collect(),
            Delimiter::Tab => line.split('\t').map(|t| t.trim().to_owned()).collect(),
            Delimiter::Semicolon => line.split(';').map(|t| t.trim().to_owned()).collect(),
            Delimiter::Pipe => line.split('|').map(|t| t.trim().to_owned()).collect(),
            Delimiter::Whitespace => line.split_whitespace().map(|t| t.to_owned()).collect(),
        }
    }
}

/// توکن عددی متلب-گونه: خالی → NaN؛ NaN/nan، Inf/inf/±Inf → مقدار؛ عدد
/// اعشاری/توانی؛ هر چیز دیگر → None (صدا زننده تصمیم می‌گیرد: در ناحیهٔ
/// داده → NaN، در سرصفحه → رد کردن خط).
fn parse_token(tok: &str) -> Option<f64> {
    let t = tok.trim();
    if t.is_empty() {
        return Some(f64::NAN); // خانهٔ خالی = NaN (داک: EmptyValue → NaN)
    }
    let lower = t.to_ascii_lowercase();
    match lower.as_str() {
        "nan" => return Some(f64::NAN),
        "inf" | "+inf" => return Some(f64::INFINITY),
        "-inf" => return Some(f64::NEG_INFINITY),
        _ => {}
    }
    t.parse::<f64>().ok()
}

/// کشف جداساز: روی نخستین خطوط غیرخالیِ داده‌خوان، کاندیدی که بیشترین
/// ستون‌ها را «پایدار» می‌دهد برنده است (تساوی → ترتیب کاندیدها: کاما، تب،
/// نقطه‌ویرگول، پایپ، فاصله).
fn detect_delimiter(lines: &[&str]) -> Delimiter {
    const CANDIDATES: [Delimiter; 5] = [
        Delimiter::Comma,
        Delimiter::Tab,
        Delimiter::Semicolon,
        Delimiter::Pipe,
        Delimiter::Whitespace,
    ];
    let mut best = Delimiter::Comma;
    let mut best_cols = 1;
    for &d in &CANDIDATES {
        let cols: Vec<usize> = lines.iter().take(4).map(|l| d.split(l).len()).collect();
        let stable = cols.windows(2).all(|w| w[0] == w[1]);
        let n = cols.first().copied().unwrap_or(0);
        // پایدار و بیش از یک ستون؛ ستون برابر → کاندیدِ زودتر می‌ماند
        if stable && n > best_cols {
            best = d;
            best_cols = n;
        }
    }
    best
}

/// آیا این خط «سرصفحه» است؟ — هیچ توکن عددی‌ای ندارد (اولین توکن هم
/// ناپذیرفتنی است). خطی که دست‌کم یک عدد دارد خط داده است (حتی اگر
/// توکن‌های متنی هم داشته باشد — رفتار readmatrix: توکن غیرعددی در ناحیهٔ
/// داده NaN می‌شود، نه سرصفحه).
fn is_header_line(line: &str, d: Delimiter) -> bool {
    !line.trim().is_empty() && d.split(line).iter().all(|t| parse_token(t).is_none())
}

/// readmatrix — متن → ماتریس double (M×N؛ تک‌سطر = ۱×N).
///
/// - خطوط کاملاً خالی نادیده (رفتار خانوادهٔ readmatrix)
/// - خطوط سرصفحهٔ ابتدایی رد می‌شوند (کشف خودکار)
/// - سطرهای کوتاه با NaN لایه می‌شوند (مستطیل می‌شود)
/// - توکن ناپذیرفتنی در ناحیهٔ داده = NaN (رفتار متلب در تبدیل متن)
pub fn read_matrix_text(text: &str) -> Result<Matrix, IoError> {
    let norm = text.replace("\r\n", "\n");
    let raw_lines: Vec<&str> = norm.split('\n').collect();
    // BOM ممکن است چسبیده باشد
    let nonempty: Vec<&str> = raw_lines
        .iter()
        .map(|l| l.trim_end())
        .filter(|l| !l.is_empty())
        .collect();
    if nonempty.is_empty() {
        return Ok(Matrix::full(0, 0, 0.0));
    }
    let delim = detect_delimiter(&nonempty);

    let mut rows: Vec<Vec<f64>> = Vec::new();
    let mut started = false;
    for line in &nonempty {
        if !started && is_header_line(line, delim) {
            continue; // سرصفحهٔ ابتدایی
        }
        started = true;
        rows.push(
            delim
                .split(line)
                .iter()
                .map(|t| parse_token(t).unwrap_or(f64::NAN))
                .collect(),
        );
    }
    if rows.is_empty() {
        return Ok(Matrix::full(0, 0, 0.0));
    }
    let cols = rows.iter().map(|r| r.len()).max().unwrap_or(0);
    if cols == 0 {
        return Ok(Matrix::full(0, 0, 0.0));
    }
    // کوتاه‌ها با NaN پهن می‌شوند؛ داده ستونی چیده می‌شود (قرارداد from_columns)
    let mut data = Vec::with_capacity(rows.len() * cols);
    for c in 0..cols {
        for r in &rows {
            data.push(r.get(c).copied().unwrap_or(f64::NAN));
        }
    }
    Ok(Matrix::from_columns(rows.len(), cols, data))
}

// ---------- writematrix ----------

/// قالب عدد برای نوشتن — ~۱۵ رقم معنادار (رفتار پیش‌فرض writematrix که
/// گزارش‌های کاربران «longg (15 digits of precision)» توصیفش کرده‌اند)؛
/// NaN/Inf با نام متلب می‌نویسد تا خوانش دوباره رفت‌وبرگشت بدهد.
pub fn format_number(v: f64) -> String {
    if v.is_nan() {
        "NaN".to_owned()
    } else if v == f64::INFINITY {
        "Inf".to_owned()
    } else if v == f64::NEG_INFINITY {
        "-Inf".to_owned()
    } else {
        let s = format!("{v:.15e}");
        // %.15e همیشه توان دارد؛ برای خواناییِ متلب‌گونه عدد بدون توانِ
        // کوتاه را ساده می‌کنیم (round-trip از %.15e تضمینی است؛ این فقط
        // نمایش است)
        let plain = format!("{v}");
        if plain.parse::<f64>() == Ok(v) && plain.len() <= s.len() {
            plain
        } else {
            s
        }
    }
}

/// writematrix — ماتریس → متن محدودشده (ردیف-به-ردیف، پُرینگ \n).
pub fn write_matrix_text(m: &Matrix, d: Delimiter) -> String {
    let sep = match d {
        Delimiter::Comma => ",",
        Delimiter::Tab => "\t",
        Delimiter::Semicolon => ";",
        Delimiter::Pipe => "|",
        Delimiter::Whitespace => " ",
    };
    let mut out = String::new();
    for r in 0..m.rows() {
        for c in 0..m.cols() {
            if c > 0 {
                out.push_str(sep);
            }
            out.push_str(&format_number(m.get(r, c)));
        }
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn col(m: &Matrix) -> Vec<f64> {
        m.as_column_slice().to_vec()
    }

    #[test]
    fn csv_with_header_and_nan_tokens() {
        let text = "a,b,c\n1,2,3\n4,,6\nnan,Inf,-Inf\n";
        let m = read_matrix_text(text).expect("parse");
        assert_eq!((m.rows(), m.cols()), (3, 3));
        // ستونی: [1,4,nan | 2,nan,Inf | 3,6,-Inf] — مقایسهٔ بیتی (NaN در == نمی‌آید)
        let want = [
            1.0,
            4.0,
            f64::NAN,
            2.0,
            f64::NAN,
            f64::INFINITY,
            3.0,
            6.0,
            f64::NEG_INFINITY,
        ];
        for (a, b) in col(&m).iter().zip(want) {
            assert_eq!(a.to_bits(), b.to_bits());
        }
    }

    #[test]
    fn semicolon_tab_pipe_detected() {
        let m = read_matrix_text("1;2;3\n4;5;6\n").expect("p");
        assert_eq!((m.rows(), m.cols()), (2, 3));
        let m = read_matrix_text("1\t2\t3\n4\t5\t6\n").expect("p");
        assert_eq!((m.rows(), m.cols()), (2, 3));
        let m = read_matrix_text("1|2|3\n4|5|6\n").expect("p");
        assert_eq!((m.rows(), m.cols()), (2, 3));
    }

    #[test]
    fn whitespace_runs_single_delim() {
        let m = read_matrix_text("1  2   3\n   4  5 6\n").expect("p");
        assert_eq!((m.rows(), m.cols()), (2, 3));
        assert_eq!(col(&m), vec![1.0, 4.0, 2.0, 5.0, 3.0, 6.0]);
    }

    #[test]
    fn ragged_rows_padded_with_nan() {
        let m = read_matrix_text("1,2,3\n4,5\n").expect("p");
        assert_eq!((m.rows(), m.cols()), (2, 3));
        assert!(m.get(1, 2).is_nan());
    }

    #[test]
    fn empty_and_header_only() {
        assert_eq!(read_matrix_text("").expect("e").numel(), 0);
        assert_eq!(
            read_matrix_text("\n\nname,value\n\n").expect("e").numel(),
            0
        );
    }

    #[test]
    fn nonnumeric_token_in_data_is_nan() {
        let m = read_matrix_text("1,x,3\n4,5,y\n").expect("p");
        assert!(m.get(0, 1).is_nan());
        assert!(m.get(1, 2).is_nan());
        assert_eq!(m.get(0, 0), 1.0);
    }

    #[test]
    fn single_row_is_1x_n() {
        let m = read_matrix_text("10,20,30,40\n").expect("p");
        assert_eq!((m.rows(), m.cols()), (1, 4));
    }

    #[test]
    fn write_defaults_and_roundtrip() {
        let m = Matrix::from_columns(2, 3, vec![1.0, 2.0, 0.5, -0.25, 1e-7, f64::NAN]);
        let text = write_matrix_text(&m, Delimiter::Comma);
        assert_eq!(text, "1,0.5,0.0000001\n2,-0.25,NaN\n");
        // رفت‌وبرگشت: NaN → NaN و بقیه دقیق
        let back = read_matrix_text(&text).expect("rt");
        assert_eq!((back.rows(), back.cols()), (2, 3));
        let want = [1.0, 2.0, 0.5, -0.25, 1e-7, f64::NAN];
        for (a, b) in col(&back).iter().zip(want) {
            assert_eq!(a.to_bits(), b.to_bits());
        }
    }

    #[test]
    fn write_precision_fifteen_sig_digits() {
        let v = 0.1 + 0.2; // 0.30000000000000004
        let m = Matrix::from_columns(1, 1, vec![v]);
        let text = write_matrix_text(&m, Delimiter::Comma);
        let back = read_matrix_text(&text).expect("rt");
        assert_eq!(
            back.get_linear(0).to_bits(),
            v.to_bits(),
            "round-trip must be bit-exact at 15 sig digits for this value"
        );
    }

    #[test]
    fn extension_mapping() {
        assert_eq!(Delimiter::for_extension("a.csv"), Delimiter::Comma);
        assert_eq!(Delimiter::for_extension("a.txt"), Delimiter::Comma);
        assert_eq!(Delimiter::for_extension("A.TSV"), Delimiter::Tab);
        assert_eq!(Delimiter::for_extension("out"), Delimiter::Comma);
    }

    #[test]
    fn format_specials_match_matlab_names() {
        assert_eq!(format_number(f64::NAN), "NaN");
        assert_eq!(format_number(f64::INFINITY), "Inf");
        assert_eq!(format_number(f64::NEG_INFINITY), "-Inf");
    }
}
