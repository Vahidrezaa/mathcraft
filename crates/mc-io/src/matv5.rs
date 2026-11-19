//! خوانندهٔ زیرمجموعهٔ .mat **Level 5** (ROADMAP 3.7؛ اسپک رسمی «MATLAB
//! Application Program Interface Guide — MAT-File Format» نسخهٔ 1997، بخش‌های
//! 1-3 تا 1-18 + راست‌آزمایی بایتی با scipy 1.14).
//!
//! چیزی که می‌خوانیم (همه روی انواع فعلی زبان):
//! - عددی real/complex از هر کلاس ذخیره‌سازی (miINT8..miUINT64/miSINGLE/
//!   miDOUBLE) → Double یا Complex (کلاس‌های int/single به double cast می‌شوند)
//! - char (miUINT16/miUTF16/miINT8/miUTF8/miUTF32) → Char (BMP؛ دلتای مستند)
//! - struct فقط ۱×۱ (StructVal ما ۱×۱ است — سند ۰۳ §3؛ آرایهٔ ساختاری بزرگ‌تر
//!   = خطای صادقانه) با فیلدهای عددی/char/structِ تودرتو
//! - miCOMPRESSED: جریان zlib دور یک element کامل (اسپک 1-6: «the entire
//!   variable is compressed... preceded by an 8-byte tag named miCOMPRESSED»)
//!   — بازکردن با flate2 (بک‌اند rust)
//!
//! خطاهای صادقانه (E6 — بدون دروغِ «پشتیبانی می‌شود»):
//! v7.3/HDF5 · sparse · cell · object · function-handle · آرایهٔ n-بعدی (n>2)
//! · struct بزرگ‌تر از ۱×۱ · endian بزرگ (MI).
//!
//! نکتهٔ تگ کوچک (اسپک 1-6، تفسیر عملیِ scipy/matio): تگ u32 را بخوان؛ اگر
//! ۱۶ بیت بالایی غیرصفر است، قالب «small» است: type = ۱۶ بیت پایین، nbytes =
//! ۱۶ بیت بالا، داده ≤ ۴ بایت داخل خود تگ. بقیه: تگ عادی ۸ بایتی.

use std::io::Read;

use flate2::read::ZlibDecoder;
use mc_core::complex::ComplexMatrix;
use mc_core::matrix::Matrix;
use mc_core::value::{StructVal, Value};

use crate::{IoError, NamedValue};

// ---------- ثابت‌های اسپک ----------

const MI_INT8: u32 = 1;
const MI_UINT8: u32 = 2;
const MI_INT16: u32 = 3;
const MI_UINT16: u32 = 4;
const MI_INT32: u32 = 5;
const MI_UINT32: u32 = 6;
const MI_SINGLE: u32 = 7;
const MI_DOUBLE: u32 = 9;
const MI_INT64: u32 = 12;
const MI_UINT64: u32 = 13;
const MI_MATRIX: u32 = 14;
const MI_COMPRESSED: u32 = 15;
const MI_UTF8: u32 = 16;
const MI_UTF16: u32 = 17;
const MI_UTF32: u32 = 18;

const MX_CELL: u32 = 1;
const MX_STRUCT: u32 = 2;
const MX_OBJECT: u32 = 3;
const MX_CHAR: u32 = 4;
const MX_SPARSE: u32 = 5;
const MX_DOUBLE: u32 = 6;
const MX_SINGLE: u32 = 7;
const MX_INT8: u32 = 8;
const MX_UINT8: u32 = 9;
const MX_INT16: u32 = 10;
const MX_UINT16: u32 = 11;
const MX_INT32: u32 = 12;
const MX_UINT32: u32 = 13;
const MX_INT64: u32 = 14;
const MX_UINT64: u32 = 15;
/// class 17 در فایل‌های MATLAB جدیدتر (function handle به‌شکل object می‌آید؛
/// مقدار عددی mxFUNCTION_CLASS که scipy هم می‌شناسد).
const MX_FUNCTION: u32 = 17;

// بیت‌های Array Flags در بایت دوم کلمهٔ اول (راست‌آزمایی با رفتار نوشتاری
// scipy: flags = complex<<3 | global<<2 | logical<<1 سپس <<8)
const FLAG_COMPLEX: u32 = 1 << 11;
const FLAG_LOGICAL: u32 = 1 << 9;

/// سقف ایمن تخصیص داده (بایت) — dims از هدر فایل می‌آید و باید با بدنهٔ
/// موجود بخواند (نبض امنیتی بر ضد ابعاد دروغ‌گو).
const MAX_ALLOC: usize = 256 * 1024 * 1024;

fn unsupported(en: &str, fa: &str) -> IoError {
    IoError::new(format!(".mat: {en}"), format!(".mat: {fa}"))
}

fn corrupt(what: &str) -> IoError {
    IoError::new(
        format!("corrupt .mat file: {what}"),
        format!("فایل .mat خراب است: {what}"),
    )
}

// ---------- عنصر + تگ ----------

struct Element<'a> {
    data_type: u32,
    bytes: &'a [u8],
}

/// خواندن یک عنصر در جایگاه فعلی: تگ (عادی/کوچک) + داده + پدینگ ۸بایتی.
fn read_element<'a>(buf: &'a [u8], pos: &mut usize) -> Result<Element<'a>, IoError> {
    let first = read_u32(buf, pos)?;
    if first > 0xFFFF {
        // قالب کوچک: type=پایین، nbytes=بالا، داده در ۴ بایت باقیِ تگ
        let data_type = first & 0xFFFF;
        let n = (first >> 16) as usize;
        if n > 4 {
            return Err(corrupt("small element with >4 bytes"));
        }
        let end = *pos + 4;
        if end > buf.len() {
            return Err(corrupt("small element past end"));
        }
        let bytes = &buf[*pos..*pos + n];
        *pos = end;
        Ok(Element { data_type, bytes })
    } else {
        let data_type = first;
        let n = read_u32(buf, pos)? as usize;
        if n > buf.len().saturating_sub(*pos) {
            return Err(corrupt("element larger than file"));
        }
        let bytes = &buf[*pos..*pos + n];
        *pos += n + pad8(n);
        Ok(Element { data_type, bytes })
    }
}

fn read_u32(buf: &[u8], pos: &mut usize) -> Result<u32, IoError> {
    if buf.len() < *pos + 4 {
        return Err(corrupt("unexpected end"));
    }
    let v = u32::from_le_bytes([buf[*pos], buf[*pos + 1], buf[*pos + 2], buf[*pos + 3]]);
    *pos += 4;
    Ok(v)
}

fn pad8(n: usize) -> usize {
    (8 - (n % 8)) % 8
}

// ---------- دادهٔ عددی خام ----------

/// یک عنصر عددی را به f64 تبدیل می‌کند (کلاس ذخیره‌سازی هرچه باشد).
fn numeric_to_f64(el: &Element) -> Result<Vec<f64>, IoError> {
    let b = el.bytes;
    Ok(match el.data_type {
        MI_DOUBLE => b
            .chunks_exact(8)
            .map(|c| f64::from_le_bytes([c[0], c[1], c[2], c[3], c[4], c[5], c[6], c[7]]))
            .collect(),
        MI_SINGLE => b
            .chunks_exact(4)
            .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]) as f64)
            .collect(),
        MI_INT8 => b.iter().map(|&v| v as i8 as f64).collect(),
        MI_UINT8 => b.iter().map(|&v| v as f64).collect(),
        MI_INT16 => b
            .chunks_exact(2)
            .map(|c| i16::from_le_bytes([c[0], c[1]]) as f64)
            .collect(),
        MI_UINT16 => b
            .chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]) as f64)
            .collect(),
        MI_INT32 => b
            .chunks_exact(4)
            .map(|c| i32::from_le_bytes([c[0], c[1], c[2], c[3]]) as f64)
            .collect(),
        MI_UINT32 => b
            .chunks_exact(4)
            .map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]]) as f64)
            .collect(),
        MI_INT64 => b
            .chunks_exact(8)
            .map(|c| i64::from_le_bytes([c[0], c[1], c[2], c[3], c[4], c[5], c[6], c[7]]) as f64)
            .collect(),
        MI_UINT64 => b
            .chunks_exact(8)
            .map(|c| u64::from_le_bytes([c[0], c[1], c[2], c[3], c[4], c[5], c[6], c[7]]) as f64)
            .collect(),
        _ => {
            return Err(corrupt(&format!(
                "numeric part stored as type {}",
                el.data_type
            )));
        }
    })
}

// ---------- رشتهٔ char ----------

fn char_to_string(el: &Element) -> Result<String, IoError> {
    let b = el.bytes;
    match el.data_type {
        MI_UTF8 | MI_INT8 | MI_UINT8 => {
            String::from_utf8(b.to_vec()).map_err(|_| corrupt("char is not valid UTF-8"))
        }
        MI_UINT16 | MI_UTF16 => {
            let units: Vec<u16> = b
                .chunks_exact(2)
                .map(|c| u16::from_le_bytes([c[0], c[1]]))
                .collect();
            String::from_utf16(&units).map_err(|_| {
                unsupported(
                    "char array contains surrogate pairs outside the BMP",
                    "آرایهٔ کاراکتری بیرون از BMP است (زوج‌های جایگزین)",
                )
            })
        }
        MI_UTF32 => {
            let mut s = String::with_capacity(b.len() / 4);
            for c in b.chunks_exact(4) {
                let cp = u32::from_le_bytes([c[0], c[1], c[2], c[3]]);
                let ch = char::from_u32(cp).ok_or_else(|| corrupt("bad UTF-32 code point"))?;
                s.push(ch);
            }
            Ok(s)
        }
        _ => Err(corrupt(&format!(
            "char part stored as type {}",
            el.data_type
        ))),
    }
}

// ---------- miMATRIX ----------

struct ArrayHeader {
    class: u32,
    complex: bool,
    logical: bool,
    dims: Vec<usize>,
    name: String,
    /// جایگاه شروع زیرعنصرهای بعد از نام (real part و …)
    rest: usize,
}

fn read_array_header(buf: &[u8], mut pos: usize) -> Result<ArrayHeader, IoError> {
    let flags = read_element(buf, &mut pos)?;
    if flags.data_type != MI_UINT32 || flags.bytes.len() != 8 {
        return Err(corrupt("array flags subelement malformed"));
    }
    let w1 = u32::from_le_bytes([
        flags.bytes[0],
        flags.bytes[1],
        flags.bytes[2],
        flags.bytes[3],
    ]);
    let class = w1 & 0xFF;
    let complex = w1 & FLAG_COMPLEX != 0;
    let logical = w1 & FLAG_LOGICAL != 0;

    let dims_el = read_element(buf, &mut pos)?;
    if dims_el.data_type != MI_INT32 || dims_el.bytes.len() % 4 != 0 {
        return Err(corrupt("dimensions subelement malformed"));
    }
    let dims: Vec<i32> = dims_el
        .bytes
        .chunks_exact(4)
        .map(|c| i32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect();
    if dims.iter().any(|&d| d < 0) {
        return Err(corrupt("negative dimension"));
    }

    let name_el = read_element(buf, &mut pos)?;
    if name_el.data_type != MI_INT8 && name_el.data_type != MI_UINT8 {
        return Err(corrupt("array name subelement malformed"));
    }
    let name = String::from_utf8_lossy(name_el.bytes).into_owned();

    Ok(ArrayHeader {
        class,
        complex,
        logical,
        dims: dims.iter().map(|&d| d as usize).collect(),
        name,
        rest: pos,
    })
}

/// تبدیل یک miMATRIX (بدون فشرده‌سازی) به Value — نام داخل آن نادیده
/// گرفته می‌شود (نام‌گذاری در سطح فایل است؛ فیلدهای struct بی‌نام‌اند).
fn matrix_to_value(buf: &[u8], mut pos: usize) -> Result<Value, IoError> {
    let header = read_array_header(buf, pos)?;
    pos = header.rest;
    // کلاس‌های بیرون از زیرمجموعهٔ 3.7 — صادقانه رد، مستقل از ابعاد
    // (نویسنده‌هایی مثل scipy برای cell بُعد تکی دنباله می‌دهند؛ پیام
    // «cell» راهنمای‌تر از «n-بعدی» است)
    match header.class {
        MX_CELL | MX_OBJECT | MX_SPARSE | MX_FUNCTION => {
            return Err(class_unsupported(header.class));
        }
        _ => {}
    }
    let ndim_err = || {
        unsupported(
            &format!(
                "{}-dimensional arrays are not supported yet (2-D only, E-phase)",
                header.dims.len()
            ),
            &format!(
                "آرایهٔ {}-بعدی هنوز پشتیبانی نمی‌شود (فقط دوبعدی)",
                header.dims.len()
            ),
        )
    };
    if header.dims.len() != 2 {
        return Err(ndim_err());
    }
    let (rows, cols) = (header.dims[0], header.dims[1]);
    let numel = rows
        .checked_mul(cols)
        .ok_or_else(|| corrupt("dimensions overflow"))?;
    if numel
        .checked_mul(16)
        .ok_or_else(|| corrupt("allocation overflow"))?
        > MAX_ALLOC
    {
        return Err(corrupt("array larger than safety limit"));
    }

    match header.class {
        MX_DOUBLE | MX_SINGLE | MX_INT8 | MX_UINT8 | MX_INT16 | MX_UINT16 | MX_INT32
        | MX_UINT32 | MX_INT64 | MX_UINT64 => {
            let real = read_element(buf, &mut pos)?;
            let re = numeric_to_f64(&real)?;
            if re.len() != numel {
                return Err(corrupt("real part length mismatch"));
            }
            if header.complex {
                let imag = read_element(buf, &mut pos)?;
                let im = numeric_to_f64(&imag)?;
                if im.len() != numel {
                    return Err(corrupt("imaginary part length mismatch"));
                }
                Ok(Value::Complex(ComplexMatrix::from_columns(
                    rows, cols, re, im,
                )))
            } else {
                // دلتای مستند: logical ذخیره‌شده به‌شکل عددی → double 0/1
                // (نوع logical زبان هنوز نیست؛ MATLAB class آن logical است)
                let _ = header.logical;
                Ok(Value::Double(Matrix::from_columns(rows, cols, re)))
            }
        }
        MX_CHAR => {
            let el = read_element(buf, &mut pos)?;
            let s = char_to_string(&el)?;
            // chars چندبعدی فقط 2D؛ طول رشته باید با numel بخواند (مگر
            // خوانش utf8 با بایت شمرده شود)
            if el.data_type == MI_UTF8 || el.data_type == MI_INT8 || el.data_type == MI_UINT8 {
                if el.bytes.len() != numel {
                    return Err(corrupt("char length mismatch"));
                }
            } else if s.chars().count() != numel {
                return Err(corrupt("char length mismatch"));
            }
            Ok(Value::Char(s))
        }
        MX_STRUCT => {
            if rows != 1 || cols != 1 {
                return Err(unsupported(
                    "struct arrays larger than 1x1 are not supported yet (needs the cell type, E4)",
                    "آرایهٔ ساختاری بزرگ‌تر از ۱×۱ هنوز پشتیبانی نمی‌شود (نیازمند نوع cell — E4)",
                ));
            }
            let flen_el = read_element(buf, &mut pos)?;
            if flen_el.data_type != MI_INT32 || flen_el.bytes.len() != 4 {
                return Err(corrupt("field name length malformed"));
            }
            let flen = i32::from_le_bytes([
                flen_el.bytes[0],
                flen_el.bytes[1],
                flen_el.bytes[2],
                flen_el.bytes[3],
            ]) as usize;
            if flen == 0 || flen > 128 {
                return Err(corrupt("unreasonable field name length"));
            }
            let names_el = read_element(buf, &mut pos)?;
            if names_el.data_type != MI_INT8 && names_el.data_type != MI_UINT8 {
                return Err(corrupt("field names subelement malformed"));
            }
            if names_el.bytes.len() % flen != 0 {
                return Err(corrupt("field names block not a multiple of field length"));
            }
            let field_names: Vec<String> = names_el
                .bytes
                .chunks_exact(flen)
                .map(|c| {
                    let end = c.iter().position(|&b| b == 0).unwrap_or(c.len());
                    String::from_utf8_lossy(&c[..end]).into_owned()
                })
                .collect();

            // «Fields are written in place... in column order» — برای ۱×۱:
            // به‌ازای هر فیلد یک miMATRIX
            let mut st = StructVal::new();
            for name in field_names {
                let cell = read_element(buf, &mut pos)?;
                if cell.data_type != MI_MATRIX {
                    return Err(corrupt("struct field is not an miMATRIX"));
                }
                // زیرآرایه: هدر خودش را دارد؛ نامش خالی است
                let v = matrix_to_value(cell.bytes, 0)?;
                st.set(&name, v);
            }
            Ok(Value::Struct(st))
        }
        other => Err(class_unsupported(other)),
    }
}

/// پیام رد یکپارچهٔ کلاس‌های بیرون از زیرمجموعهٔ 3.7.
fn class_unsupported(class: u32) -> IoError {
    match class {
        MX_CELL => unsupported(
            "cell arrays are not supported yet (E4)",
            "آرایهٔ سلولی هنوز پشتیبانی نمی‌شود (E4)",
        ),
        MX_OBJECT => unsupported("objects are not supported", "آبجکت‌ها پشتیبانی نمی‌شوند"),
        MX_SPARSE => unsupported(
            "sparse arrays are not supported (dense only)",
            "آرایه‌های اسپارس پشتیبانی نمی‌شوند (فقط پُر)",
        ),
        MX_FUNCTION => unsupported(
            "function handles cannot be loaded",
            "هندل تابع قابل بارگذاری نیست",
        ),
        other => unsupported(
            &format!("unknown MATLAB class {other}"),
            &format!("کلاس ناشناختهٔ MATLAB: {other}"),
        ),
    }
}

// ---------- ورودی فایل ----------

/// خواندن فایل .mat v5 کامل — فهرست متغیرها به‌ترتیب فایل.
/// فایل‌های v7.3 (HDF5) با خطای راهنما رد می‌شوند.
pub fn read_mat(bytes: &[u8]) -> Result<Vec<NamedValue>, IoError> {
    if bytes.len() < 128 {
        return Err(corrupt("shorter than the 128-byte header"));
    }
    // version: u16 LE در آفست 124 — 0x0100 یعنی v5/v6/v7 (v7 = همان v5 +
    // فشرده‌سازی)؛ 0x0200 یعنی v7.3 (HDF5)
    let version = u16::from_le_bytes([bytes[124], bytes[125]]);
    let endian = &bytes[126..128];
    match (version, endian) {
        (0x0100, b"IM") => {}
        (0x0200, b"IM") | (0x0200, b"MI") => {
            return Err(unsupported(
                "MATLAB v7.3 files (HDF5) are not supported — re-save with save -v7 in MATLAB/Octave",
                "فایل‌های v7.3 متلب (HDF5) پشتیبانی نمی‌شوند — در متلب/اکتاو دوباره با save -v7 ذخیره کنید",
            ));
        }
        _ => {
            return Err(corrupt(
                "not a Level 5 MAT-file (bad version/endian indicator)",
            ));
        }
    }

    let mut vars = Vec::new();
    let mut pos = 128usize;
    while pos < bytes.len() {
        let el = read_element(bytes, &mut pos)?;
        if el.data_type == MI_COMPRESSED {
            // جریان zlib دور یک element کامل (می‌تواند خودش miMATRIX باشد)
            let mut zd = ZlibDecoder::new(el.bytes);
            let mut raw = Vec::new();
            zd.read_to_end(&mut raw)
                .map_err(|_| corrupt("zlib stream invalid"))?;
            let mut inner = 0usize;
            let m = read_element(&raw, &mut inner)?;
            if m.data_type != MI_MATRIX {
                return Err(corrupt("compressed element does not hold an miMATRIX"));
            }
            let (name, value) = named_matrix(m.bytes)?;
            vars.push((name, value));
        } else if el.data_type == MI_MATRIX {
            let (name, value) = named_matrix(el.bytes)?;
            vars.push((name, value));
        } else {
            return Err(corrupt(&format!(
                "expected miMATRIX at top level, found type {}",
                el.data_type
            )));
        }
    }
    Ok(vars)
}

/// هدر یک miMATRIX را می‌خواند و نامش را برمی‌گرداند (زیرمجموعهٔ نامدار
/// فایل؛ فیلدهای struct بی‌نام‌اند).
fn named_matrix(buf: &[u8]) -> Result<NamedValue, IoError> {
    let header = read_array_header(buf, 0)?;
    let value = matrix_to_value(buf, 0)?;
    Ok((header.name, value))
}
