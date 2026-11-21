//! قالب MCWS (MathCraft Workspace) v1 — ماندگاری ورک‌اسپس (ROADMAP 3.7).
//!
//! **چرا قالب خودمان و نه .mat:** نوشتن .mat در 5.5 است (پیمان ضد-انحراف)؛
//! قالب سادهٔ نسخه‌دارِ خودمان رفت‌وبرگشت بیت‌به‌بیت را تضمین می‌کند و سند
//! فرمت مستقل از MATLAB نگه می‌دارد (مثل MC-WS جدا از .mat در سند ۰۸).
//!
//! ```text
//! بایت‌ها little-endian؛ همهٔ طول‌ها شامل خودشان نیستند.
//!
//! offset  میدان
//! 0..4    magic  b"MCWS"
//! 4       u8     version = 1
//! 5..7    u16    تعداد متغیرها
//! 7..     رکوردها (پشت‌سرهم، بدون ترازبندی):
//!
//! record = u16 name_len + name_len × utf8 + u8 kind + payload
//!   kind 0 Double : u32 rows u32 cols (r*c) × u64  بیت‌های f64 (ستونی)
//!   kind 1 Complex: u32 rows u32 cols (r*c) × u64 (re) (r*c) × u64 (im)
//!   kind 2 Char   : u32 byte_len + utf8
//!   kind 3 Struct : u32 field_count + به‌ازای هر فیلد: record کامل (نامی‌دار)
//!   kind 4 Sym    : u32 byte_len + utf8 (S-اجر کانونی — CAS-1، سند ۰۵ §۱۲)
//! ```
//!
//! بیت‌به‌بیت یعنی `to_bits/from_bits` خام — payload های NaN، `-0` و Inf
//! بدون هیچ گِرد کردن زنده می‌مانند (تست خاص دارند). ترتیب فیلدهای struct
//! حفظ می‌شود (قرارداد StructVal سند ۰۳ §3). تابع بی‌نام (Anon) ذخیره نمی‌شود —
//! خطای صادقانه (نگاه `encode_workspace`).

use mc_core::complex::ComplexMatrix;
use mc_core::matrix::Matrix;
use mc_core::value::{StructVal, Value};

use crate::{IoError, NamedValue};

pub const MAGIC: &[u8; 4] = b"MCWS";
pub const VERSION: u8 = 1;

const KIND_DOUBLE: u8 = 0;
const KIND_COMPLEX: u8 = 1;
const KIND_CHAR: u8 = 2;
const KIND_STRUCT: u8 = 3;
/// CAS-1 (گام 4.2 — سند ۰۵ §۱۲): درخت سمبولیک به‌صورت S-اجر کانونی.
const KIND_SYM: u8 = 4;

/// سقف ایمن تخصیص — رکورد خراب نباید اَبَر-ماتریس بسازد (بند بودجه سند ۰۲ §8:
/// هر بیت ورودی حداکثر ۸ بایت خروجی می‌تواند بخواهد؛ سقف کل ۲۵۶MiB داده).
const MAX_ALLOC: usize = 256 * 1024 * 1024;

fn err_unsupported(en: &str, fa: &str) -> IoError {
    IoError::new(
        format!("MCWS v{VERSION}: {en}"),
        format!("MCWS v{VERSION}: {fa}"),
    )
}

// ---------- انکود ----------

/// رمزگذاری یک Value به فرمت v1 — بدون نام (برای فیلد struct).
/// Anon عمداً پشتیبانی نمی‌شود (نام فایل/ابزار ذخیره‌سازی هندل‌ها = 5.5).
pub fn encode_value(value: &Value) -> Result<Vec<u8>, IoError> {
    let mut out = Vec::new();
    write_value(&mut out, value)?;
    Ok(out)
}

fn write_value(out: &mut Vec<u8>, value: &Value) -> Result<(), IoError> {
    match value {
        Value::Double(m) => {
            out.push(KIND_DOUBLE);
            out.extend_from_slice(&(m.rows() as u32).to_le_bytes());
            out.extend_from_slice(&(m.cols() as u32).to_le_bytes());
            for v in m.as_column_slice() {
                out.extend_from_slice(&v.to_bits().to_le_bytes());
            }
        }
        Value::Complex(m) => {
            out.push(KIND_COMPLEX);
            out.extend_from_slice(&(m.rows() as u32).to_le_bytes());
            out.extend_from_slice(&(m.cols() as u32).to_le_bytes());
            for v in m.re_column_slice() {
                out.extend_from_slice(&v.to_bits().to_le_bytes());
            }
            for v in m.im_column_slice() {
                out.extend_from_slice(&v.to_bits().to_le_bytes());
            }
        }
        Value::Char(s) => {
            out.push(KIND_CHAR);
            out.extend_from_slice(&(s.len() as u32).to_le_bytes());
            out.extend_from_slice(s.as_bytes());
        }
        Value::Struct(s) => {
            out.push(KIND_STRUCT);
            out.extend_from_slice(&(s.fields.len() as u32).to_le_bytes());
            for (name, v) in &s.fields {
                write_str(out, name);
                write_value(out, v)?;
            }
        }
        Value::Anon(_) => {
            return Err(err_unsupported(
                "anonymous functions cannot be saved (planned with 5.5 interop)",
                "توابع بی‌نام قابل ذخیره نیستند (با interop گام 5.5)",
            ));
        }
        // CAS-1 (گام 4.2): درخت سمبولیک — S-اجر کانونیِ بی‌ابهام (سند ۰۵
        // §۱۲: «با CAS-1 که diff/simplify هم معنا پیدا می‌کند، KIND_SYM
        // اضافه می‌شود» — رفت‌وبرگشت بی‌اتلاف با تست پین‌شده)
        Value::Sym(s) => {
            out.push(KIND_SYM);
            let payload = mc_cas::to_sexp(s);
            out.extend_from_slice(&(payload.len() as u32).to_le_bytes());
            out.extend_from_slice(payload.as_bytes());
        }
    }
    Ok(())
}

fn write_str(out: &mut Vec<u8>, s: &str) {
    out.extend_from_slice(&(s.len() as u16).to_le_bytes());
    out.extend_from_slice(s.as_bytes());
}

/// رمزگذاری کل ورک‌اسپس — ترتیب درج نامتغیر (بازخوانی همان ترتیب را می‌دهد؛
/// `who` خودش مرتب می‌کند، پس ترتیب فایل اهمیت نمایشی ندارد).
pub fn encode_workspace(vars: &[NamedValue]) -> Result<Vec<u8>, IoError> {
    if vars.len() > u16::MAX as usize {
        return Err(err_unsupported(
            "too many variables for v1",
            "تعداد متغیرها بیش از سقف v1 است",
        ));
    }
    let mut out = Vec::new();
    out.extend_from_slice(MAGIC);
    out.push(VERSION);
    out.extend_from_slice(&(vars.len() as u16).to_le_bytes());
    for (name, v) in vars {
        if name.len() > u16::MAX as usize {
            return Err(err_unsupported(
                "variable name too long",
                "نام متغیر بیش از حد بلند است",
            ));
        }
        write_str(&mut out, name);
        write_value(&mut out, v)?;
    }
    Ok(out)
}

// ---------- دیکود ----------

struct Reader<'a> {
    buf: &'a [u8],
    pos: usize,
    /// عمق تودرتویی struct — سقف بازگشت (فایل خرابِ خودارجاع بی‌نهایت نه).
    depth: u32,
}

const MAX_DEPTH: u32 = 16;

impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], IoError> {
        let end = self
            .pos
            .checked_add(n)
            .ok_or_else(|| corrupt("length overflow"))?;
        if end > self.buf.len() {
            return Err(corrupt("unexpected end of file"));
        }
        let s = &self.buf[self.pos..end];
        self.pos = end;
        Ok(s)
    }

    fn u8(&mut self) -> Result<u8, IoError> {
        Ok(self.take(1)?[0])
    }

    fn u16(&mut self) -> Result<u16, IoError> {
        let b = self.take(2)?;
        Ok(u16::from_le_bytes([b[0], b[1]]))
    }

    fn u32(&mut self) -> Result<u32, IoError> {
        let b = self.take(4)?;
        Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }

    fn f64_bits(&mut self) -> Result<f64, IoError> {
        let b = self.take(8)?;
        Ok(f64::from_bits(u64::from_le_bytes([
            b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7],
        ])))
    }
}

fn corrupt(what: &str) -> IoError {
    IoError::new(
        format!("corrupt MCWS file: {what}"),
        format!("فایل MCWS خراب است: {what}"),
    )
}

fn read_str(r: &mut Reader) -> Result<String, IoError> {
    let n = r.u16()? as usize;
    let b = r.take(n)?;
    String::from_utf8(b.to_vec()).map_err(|_| corrupt("name is not valid UTF-8"))
}

fn read_value(r: &mut Reader) -> Result<Value, IoError> {
    r.depth += 1;
    if r.depth > MAX_DEPTH {
        return Err(corrupt("struct nesting too deep"));
    }
    let kind = r.u8()?;
    let v = match kind {
        KIND_DOUBLE | KIND_COMPLEX => {
            let rows = r.u32()? as usize;
            let cols = r.u32()? as usize;
            let n = rows
                .checked_mul(cols)
                .ok_or_else(|| corrupt("dimensions overflow"))?;
            if n.checked_mul(8)
                .ok_or_else(|| corrupt("allocation overflow"))?
                > MAX_ALLOC
            {
                return Err(corrupt("record larger than safety limit"));
            }
            let mut re = Vec::with_capacity(n);
            for _ in 0..n {
                re.push(r.f64_bits()?);
            }
            if kind == KIND_DOUBLE {
                Value::Double(Matrix::from_columns(rows, cols, re))
            } else {
                let mut im = Vec::with_capacity(n);
                for _ in 0..n {
                    im.push(r.f64_bits()?);
                }
                Value::Complex(ComplexMatrix::from_columns(rows, cols, re, im))
            }
        }
        KIND_CHAR => {
            let n = r.u32()? as usize;
            if n > MAX_ALLOC {
                return Err(corrupt("record larger than safety limit"));
            }
            let b = r.take(n)?;
            String::from_utf8(b.to_vec())
                .map(Value::Char)
                .map_err(|_| corrupt("char is not valid UTF-8"))?
        }
        KIND_STRUCT => {
            let count = r.u32()? as usize;
            let mut s = StructVal::new();
            for _ in 0..count {
                let name = read_str(r)?;
                let v = read_value(r)?;
                s.set(&name, v);
            }
            Value::Struct(s)
        }
        KIND_SYM => {
            let n = r.u32()? as usize;
            if n > MAX_ALLOC {
                return Err(corrupt("record larger than safety limit"));
            }
            let b = r.take(n)?;
            let payload = String::from_utf8(b.to_vec())
                .map_err(|_| corrupt("symbolic payload is not valid UTF-8"))?;
            let tree = mc_cas::from_sexp(&payload)
                .map_err(|_| corrupt("symbolic payload is not a valid canonical expression"))?;
            Value::Sym(std::sync::Arc::new(tree))
        }
        _ => return Err(corrupt("unknown record kind")),
    };
    r.depth -= 1;
    Ok(v)
}

/// بازخوانی کل ورک‌اسپس — میزیره در انتهای فایل = خرابی (سختی صادقانه، نه
/// بی‌صدا قیچی‌کردن؛ فرمت خودمان را با طول‌های درست خودمان می‌نویسیم).
pub fn decode_workspace(bytes: &[u8]) -> Result<Vec<NamedValue>, IoError> {
    let mut r = Reader {
        buf: bytes,
        pos: 0,
        depth: 0,
    };
    if r.take(4)? != MAGIC {
        return Err(IoError::new(
            "not an MCWS file (bad magic)",
            "فایل MCWS نیست (امضای نامعتبر)",
        ));
    }
    let version = r.u8()?;
    if version != VERSION {
        return Err(IoError::new(
            format!("MCWS version {version} is not supported (this build reads v{VERSION})"),
            format!("نسخهٔ MCWS {version} پشتیبانی نمی‌شود (این ساخت v{VERSION} می‌خواند)"),
        ));
    }
    let count = r.u16()? as usize;
    let mut vars = Vec::with_capacity(count.min(1024));
    for _ in 0..count {
        let name = read_str(&mut r)?;
        let v = read_value(&mut r)?;
        vars.push((name, v));
    }
    if r.pos != bytes.len() {
        return Err(corrupt("trailing bytes after last record"));
    }
    Ok(vars)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dbl(data: &[f64], rows: usize, cols: usize) -> Value {
        Value::Double(Matrix::from_columns(rows, cols, data.to_vec()))
    }

    fn roundtrip(vars: Vec<NamedValue>) -> Vec<NamedValue> {
        let bytes = encode_workspace(&vars).expect("encode");
        decode_workspace(&bytes).expect("decode")
    }

    #[test]
    fn roundtrip_double_char_struct_empty() {
        let s = {
            let mut st = StructVal::new();
            st.set("x", dbl(&[1.5], 1, 1));
            st.set("name", Value::Char("héllo".to_owned()));
            Value::Struct(st)
        };
        let vars = vec![
            ("A".to_owned(), dbl(&[1.0, 2.0, 3.0, 4.0], 2, 2)),
            ("empty".to_owned(), Value::Double(Matrix::full(0, 0, 0.0))),
            ("s".to_owned(), Value::Char("سلام こんにちは".to_owned())),
            ("st".to_owned(), s),
        ];
        let out = roundtrip(vars.clone());
        assert_eq!(out, vars);
    }

    #[test]
    fn roundtrip_bit_exact_nan_negzero_inf() {
        // بیت‌های خام زنده می‌مانند — نه == (که NaN را نادرست می‌گیرد)
        let vals = [f64::NAN, -0.0, f64::INFINITY, f64::NEG_INFINITY];
        let m = Matrix::from_columns(1, 4, vals.to_vec());
        let bytes = encode_workspace(&[("z".to_owned(), Value::Double(m))]).expect("enc");
        let out = decode_workspace(&bytes).expect("dec");
        let Value::Double(got) = &out[0].1 else {
            panic!("kind");
        };
        for (i, v) in vals.iter().enumerate() {
            assert_eq!(
                got.get_linear(i).to_bits(),
                v.to_bits(),
                "bit pattern of element {i} must survive"
            );
        }
        assert!(got.get_linear(0).is_nan());
        // -0 علامتش می‌ماند (== آن را می‌پوشاند)
        assert!(got.get_linear(1).is_sign_negative());
    }

    #[test]
    fn roundtrip_complex() {
        let c = ComplexMatrix::from_columns(
            2,
            2,
            vec![1.0, 2.0, 3.0, 4.0],
            vec![10.0, f64::NAN, -0.0, 4.5],
        );
        let vars = vec![("C".to_owned(), Value::Complex(c))];
        // == با NaN نادرست است — مقایسهٔ بیتی (رفت‌وبرگشت تضمینی است)
        let out = roundtrip(vars.clone());
        let Value::Complex(orig) = &vars[0].1 else {
            panic!()
        };
        let Value::Complex(got) = &out[0].1 else {
            panic!()
        };
        for (a, b) in orig.re_column_slice().iter().zip(got.re_column_slice()) {
            assert_eq!(a.to_bits(), b.to_bits());
        }
        for (a, b) in orig.im_column_slice().iter().zip(got.im_column_slice()) {
            assert_eq!(a.to_bits(), b.to_bits());
        }
        let Value::Complex(g) = &out[0].1 else {
            panic!("kind")
        };
        assert!(g.im_column_slice()[1].is_nan());
        assert!(g.im_column_slice()[2].is_sign_negative());
    }

    #[test]
    fn anon_is_honest_error() {
        let anon = mc_core::value::AnonFn {
            params: vec!["x".to_owned()],
            call: std::sync::Arc::new(|_: &[Value]| Ok(Value::Double(Matrix::scalar(1.0)))),
        };
        let e = encode_value(&Value::Anon(anon)).unwrap_err();
        assert!(e.message_en.contains("anonymous"));
    }

    #[test]
    fn bad_magic_version_trailing_rejected() {
        let good = encode_workspace(&[("a".to_owned(), dbl(&[1.0], 1, 1))]).expect("enc");
        assert!(decode_workspace(b"NOPE rest").is_err());
        let mut bad_ver = good.clone();
        bad_ver[4] = 9;
        assert!(decode_workspace(&bad_ver).is_err());
        let mut trailing = good.clone();
        trailing.extend_from_slice(&[0, 0, 0]);
        assert!(decode_workspace(&trailing).is_err());
        assert!(decode_workspace(&good).is_ok());
    }

    #[test]
    fn truncated_records_rejected() {
        let good = encode_workspace(&[(
            "abc".to_owned(),
            dbl(&(0..100).map(|i| i as f64).collect::<Vec<_>>(), 10, 10),
        )])
        .expect("enc");
        // بریدن هر جا = خطا، نه سکوت
        for cut in [7usize, 12, 20, good.len() / 2, good.len() - 1] {
            assert!(decode_workspace(&good[..cut]).is_err(), "cut at {cut}");
        }
    }

    #[test]
    fn huge_dims_are_guarded_not_allocated() {
        // ابعاد دروغ‌گوی 1e9×1e9 با بدنهٔ کوتاه → خطای سقف، نه OOM
        let mut evil = Vec::new();
        evil.extend_from_slice(MAGIC);
        evil.push(VERSION);
        evil.extend_from_slice(&1u16.to_le_bytes());
        write_str(&mut evil, "boom");
        evil.push(KIND_DOUBLE);
        evil.extend_from_slice(&u32::MAX.to_le_bytes());
        evil.extend_from_slice(&u32::MAX.to_le_bytes());
        let e = decode_workspace(&evil).unwrap_err();
        assert!(e.message_en.contains("corrupt") || e.message_en.contains("overflow"));
    }
}
