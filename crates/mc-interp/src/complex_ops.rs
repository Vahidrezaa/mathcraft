//! عملیات مختلط برای مفسر — سند ۰۳ بخش ۳ (P2).
//!
//! طراحی: Value::Complex(ComplexMatrix) شهروند کامل است؛ با Double ترکیب
//! می‌شود (مثل متلب: `1 + 2i` مختلط می‌دهد). قواعد:
//! - عنصری (+ − .* ./ .^): بروزکست اسکالر/هم‌شکل مثل نسخهٔ حقیقی.
//! - ضرب ماتریسی: فرمول بلوکی (A+Bi)(C+Di) = (AC−BD) + (AD+BC)i با matmul حقیقی.
//! - `\` و `/` مختلط: سیستم بلوکی [[Re −Im][Im Re]] روی درخت dispatch حقیقی —
//!   دقیق و بدون کد solve دوم.
//! - ترانهاده مزدوج `'` = ترانهاده + مزدوج؛ `.'` بدون مزدوج.
//! - مقایسه ترتیبی (< >) روی مختلط خطای صادقانه متلب می‌دهد؛ == و ~= کار می‌کنند.

use mc_core::{ComplexMatrix, Matrix, McError, Value};

/// خطای استاندارد مختلط.
fn cerr(line: u32, col: u32, en: impl Into<String>, fa: impl Into<String>) -> McError {
    McError::new(mc_core::error::ids::PARSE_ERROR, line, col, en, fa)
}

/// تبدیل به ماتریس مختلط (حقیقی → موهومی صفر).
pub fn to_complex(v: &Value, line: u32, col: u32) -> Result<ComplexMatrix, McError> {
    match v {
        Value::Double(m) => Ok(ComplexMatrix::from_real(m)),
        Value::Complex(z) => Ok(z.clone()),
        Value::Char(s) => {
            let mut ch = s.chars();
            match (ch.next(), ch.next()) {
                (Some(c), None) => Ok(ComplexMatrix::scalar(c as u32 as f64, 0.0)),
                _ => Err(cerr(
                    line,
                    col,
                    "Only single-character strings work in numeric expressions.",
                    "در عبارت عددی فقط رشته تک‌کاراکتری کار می‌کند.",
                )),
            }
        }
        Value::Anon(_) => Err(cerr(
            line,
            col,
            "Function handle in numeric expression.",
            "function_handle در عبارت عددی.",
        )),
    }
}

/// آیا Value حقیقی است؟ (Double یا Complex با موهومی صفر)
pub fn is_real_value(v: &Value) -> bool {
    match v {
        Value::Double(_) => true,
        Value::Complex(z) => z.is_all_real(),
        _ => false,
    }
}

/// ماتریس مختلط از دو ماتریس حقیقی هم‌شکل (بخش حقیقی/موهومی).
fn from_parts(re: &Matrix, im: &Matrix) -> ComplexMatrix {
    ComplexMatrix::from_columns(
        re.rows(),
        re.cols(),
        re.as_column_slice().to_vec(),
        im.as_column_slice().to_vec(),
    )
}

/// عنصری دوتایی مختلط با بروزکست — `f(re_x, im_x, re_y, im_y) -> (re, im)`.
pub fn complex_broadcast(
    a: &ComplexMatrix,
    b: &ComplexMatrix,
    op_name: &str,
    f: impl Fn(f64, f64, f64, f64) -> (f64, f64),
) -> Result<ComplexMatrix, McError> {
    let (ar, ac, br, bc) = (a.rows(), a.cols(), b.rows(), b.cols());
    let a_scalar = a.is_scalar();
    let b_scalar = b.is_scalar();
    if !a_scalar && !b_scalar && (ar, ac) != (br, bc) {
        return Err(McError::new(
            mc_core::error::ids::SIZE_MISMATCH,
            0,
            0,
            format!("{op_name}: sizes must agree or one scalar, got {ar}x{ac} and {br}x{bc}."),
            format!(
                "{op_name}: ابعاد باید هم‌خوان یا یکی اسکالر باشد؛ دریافت شد {ar}×{ac} و {br}×{bc}."
            ),
        ));
    }
    let (rows, cols) = if a_scalar { (br, bc) } else { (ar, ac) };
    let mut out = ComplexMatrix::full(rows, cols, 0.0, 0.0);
    for c in 0..cols {
        for r in 0..rows {
            let (x_re, x_im) = if a_scalar { a.get(0, 0) } else { a.get(r, c) };
            let (y_re, y_im) = if b_scalar { b.get(0, 0) } else { b.get(r, c) };
            let (re, im) = f(x_re, x_im, y_re, y_im);
            out.set(r, c, re, im);
        }
    }
    Ok(out)
}

/// ساخت Value ماتریس از عناصر تخت — اگر هر عنصری مختلط باشد خروجی مختلط است
/// (رفتار متلب: `[1 2i]` مختلط ۱×۲).
pub fn build_matrix_value(elements: &[Value], rows: usize, cols: usize) -> Result<Value, McError> {
    let any_complex = elements.iter().any(|v| matches!(v, Value::Complex(_)));
    let n = rows * cols;
    if elements.len() != n {
        return Err(cerr(
            0,
            0,
            format!(
                "matrix literal: expected {n} elements, got {}.",
                elements.len()
            ),
            format!("ماتریس لفظی: {n} عنصر لازم است، {} داده شد.", elements.len()),
        ));
    }
    // عناصر به ترتیب سطری می‌آیند (eval build_matrix ردیف‌ها را تخت می‌کند)
    if any_complex {
        let mut re = vec![0.0; n];
        let mut im = vec![0.0; n];
        for (k, v) in elements.iter().enumerate() {
            let z = to_complex(v, 0, 0)?;
            if z.numel() != 1 {
                return Err(cerr(
                    0,
                    0,
                    "matrix literal elements must be scalars in this milestone.",
                    "عناصر ماتریس لفظی در این مایل‌استون باید اسکالر باشند.",
                ));
            }
            let (vr, vi) = z.get(0, 0);
            let r = k / cols;
            let c = k % cols;
            re[c * rows + r] = vr;
            im[c * rows + r] = vi;
        }
        Ok(Value::Complex(ComplexMatrix::from_columns(
            rows, cols, re, im,
        )))
    } else {
        let mut data = vec![0.0; n];
        for (k, v) in elements.iter().enumerate() {
            match v {
                Value::Double(m) if m.is_scalar() => {
                    let r = k / cols;
                    let c = k % cols;
                    data[c * rows + r] = m.get(0, 0);
                }
                Value::Char(s) => {
                    let mut ch = s.chars();
                    match (ch.next(), ch.next()) {
                        (Some(c), None) => {
                            let r = k / cols;
                            let c2 = k % cols;
                            data[c2 * rows + r] = c as u32 as f64;
                        }
                        _ => {
                            return Err(cerr(
                                0,
                                0,
                                "matrix literal elements must be numeric scalars.",
                                "عناصر ماتریس لفظی باید اسکالر عددی باشند.",
                            ))
                        }
                    }
                }
                _ => {
                    return Err(cerr(
                        0,
                        0,
                        "matrix literal elements must be numeric scalars.",
                        "عناصر ماتریس لفظی باید اسکالر عددی باشند.",
                    ))
                }
            }
        }
        Ok(Value::Double(Matrix::from_columns(rows, cols, data)))
    }
}

// ---------- عملگرهای عنصری مختلط ----------

pub fn c_add(a: &ComplexMatrix, b: &ComplexMatrix) -> Result<ComplexMatrix, McError> {
    complex_broadcast(a, b, "plus", |xr, xi, yr, yi| (xr + yr, xi + yi))
}

pub fn c_sub(a: &ComplexMatrix, b: &ComplexMatrix) -> Result<ComplexMatrix, McError> {
    complex_broadcast(a, b, "minus", |xr, xi, yr, yi| (xr - yr, xi - yi))
}

pub fn c_elem_mul(a: &ComplexMatrix, b: &ComplexMatrix) -> Result<ComplexMatrix, McError> {
    complex_broadcast(a, b, "times", |xr, xi, yr, yi| {
        (xr * yr - xi * yi, xr * yi + xi * yr)
    })
}

pub fn c_elem_div(a: &ComplexMatrix, b: &ComplexMatrix) -> Result<ComplexMatrix, McError> {
    complex_broadcast(a, b, "rdivide", |xr, xi, yr, yi| {
        let den = yr * yr + yi * yi;
        ((xr * yr + xi * yi) / den, (xi * yr - xr * yi) / den)
    })
}

pub fn c_elem_pow(a: &ComplexMatrix, b: &ComplexMatrix) -> Result<ComplexMatrix, McError> {
    // w^z = exp(z·ln w) — شاخهٔ اصلی لگاریتم
    complex_broadcast(a, b, "power", |xr, xi, yr, yi| {
        if xr == 0.0 && xi == 0.0 {
            // 0^z متلب: 0^0=1، 0^مثبت=0، 0^منفی=Inf
            if yr == 0.0 {
                return (1.0, 0.0);
            }
            if yr > 0.0 {
                return (0.0, 0.0);
            }
            return (f64::INFINITY, 0.0);
        }
        let modulus = (xr * xr + xi * xi).sqrt();
        let lnr = modulus.ln();
        let theta = xi.atan2(xr);
        // z·ln w
        let pr = yr * lnr - yi * theta;
        let pi = yr * theta + yi * lnr;
        let er = pr.exp();
        (er * pi.cos(), er * pi.sin())
    })
}

/// ضرب ماتریسی حقیقی کوچک (بلوک‌ها).
fn matmul_pair(a: &Matrix, b: &Matrix) -> Matrix {
    let mut out = Matrix::full(a.rows(), b.cols(), 0.0);
    for c in 0..b.cols() {
        for r in 0..a.rows() {
            let mut acc = 0.0;
            for k in 0..a.cols() {
                acc += a.get(r, k) * b.get(k, c);
            }
            out.set(r, c, acc);
        }
    }
    out
}

/// جمع/تفریق عنصری ماتریس حقیقی (Matrix عمداً Add/Sub ندارد).
fn m_add(a: &Matrix, b: &Matrix) -> Matrix {
    let mut out = Matrix::full(a.rows(), a.cols(), 0.0);
    for i in 0..a.numel() {
        out.set_linear(i, a.get_linear(i) + b.get_linear(i));
    }
    out
}

fn m_sub(a: &Matrix, b: &Matrix) -> Matrix {
    let mut out = Matrix::full(a.rows(), a.cols(), 0.0);
    for i in 0..a.numel() {
        out.set_linear(i, a.get_linear(i) - b.get_linear(i));
    }
    out
}

/// ضرب ماتریسی مختلط — فرمول بلوکی (A+Bi)(C+Di) = (AC−BD) + (AD+BC)i.
/// اسکالر×ماتریس هم همین مسیر را درست طی می‌کند (بلوک‌ها اسکالر/بروزکست).
pub fn c_matmul(a: &ComplexMatrix, b: &ComplexMatrix) -> Result<ComplexMatrix, McError> {
    if a.cols() != b.rows() {
        return Err(McError::new(
            mc_core::error::ids::SIZE_MISMATCH,
            0,
            0,
            format!(
                "mtimes: inner dimensions must agree, got {}x{} and {}x{}.",
                a.rows(),
                a.cols(),
                b.rows(),
                b.cols()
            ),
            format!(
                "ضرب ماتریسی: ابعاد داخلی باید هم‌خوان باشند؛ دریافت شد {}×{} و {}×{}.",
                a.rows(),
                a.cols(),
                b.rows(),
                b.cols()
            ),
        ));
    }
    let ar = a.real_part();
    let ai = a.imag_part();
    let br = b.real_part();
    let bi = b.imag_part();
    if a.is_scalar() || b.is_scalar() {
        // بروزکست اسکالر: ضرب عنصری مختلط همان معنا را دارد
        return c_elem_mul(a, b);
    }
    let re = m_sub(&matmul_pair(&ar, &br), &matmul_pair(&ai, &bi));
    let im = m_add(&matmul_pair(&ar, &bi), &matmul_pair(&ai, &br));
    Ok(from_parts(&re, &im))
}

/// ترانهاده (با یا بدون مزدوج) مختلط.
pub fn c_transpose(z: &ComplexMatrix, conj: bool) -> ComplexMatrix {
    let mut out = ComplexMatrix::full(z.cols(), z.rows(), 0.0, 0.0);
    for c in 0..z.cols() {
        for r in 0..z.rows() {
            let (re, im) = z.get(r, c);
            if conj {
                out.set(c, r, re, -im);
            } else {
                out.set(c, r, re, im);
            }
        }
    }
    out
}

/// مساوی عنصری مختلط — خروجی حقیقی 0/1 (سمانتیک == متلب).
pub fn c_eq(a: &ComplexMatrix, b: &ComplexMatrix) -> Result<Matrix, McError> {
    let z = complex_broadcast(a, b, "eq", |xr, xi, yr, yi| {
        let eq = (xr == yr) && (xi == yi);
        (if eq { 1.0 } else { 0.0 }, 0.0)
    })?;
    Ok(z.real_part())
}

/// `\` مختلط با سیستم بلوکی — [[Ar −Ai][Ai Ar]]·[xr; xi] = [br; bi].
/// معادل حل مختلط با ماشین حقیقی؛ درخت dispatch حقیقی حفظ می‌شود.
pub fn c_backslash(a: &ComplexMatrix, b: &ComplexMatrix) -> Result<ComplexMatrix, McError> {
    let (n, ma) = (a.rows(), a.cols());
    if a.rows() != a.cols() {
        return Err(McError::new(
            mc_core::error::ids::SIZE_MISMATCH,
            0,
            0,
            "backslash: complex path needs a square matrix in this milestone.",
            "بک‌اسلش: مسیر مختلط در این مایل‌استون ماتریس مربع می‌خواهد.",
        ));
    }
    if b.rows() != n {
        return Err(McError::new(
            mc_core::error::ids::SIZE_MISMATCH,
            0,
            0,
            "backslash: row counts must agree.",
            "بک‌اسلش: تعداد ردیف‌ها باید هم‌خوان باشد.",
        ));
    }
    let ar = a.real_part();
    let ai = a.imag_part();
    let br = b.real_part();
    let bi = b.imag_part();
    let big_rows = 2 * n;
    let big_cols = 2 * ma;
    let mut big_a = Matrix::full(big_rows, big_cols, 0.0);
    let mut big_b = Matrix::full(big_rows, br.cols(), 0.0);
    for c in 0..ma {
        for r in 0..n {
            big_a.set(r, c, ar.get(r, c));
            big_a.set(r, ma + c, -ai.get(r, c));
            big_a.set(n + r, c, ai.get(r, c));
            big_a.set(n + r, ma + c, ar.get(r, c));
        }
    }
    for c in 0..br.cols() {
        for r in 0..n {
            big_b.set(r, c, br.get(r, c));
            big_b.set(n + r, c, bi.get(r, c));
        }
    }
    let (sol, _) = mc_num::mldivide(&big_a, &big_b)?;
    let mut re = Matrix::full(n, br.cols(), 0.0);
    let mut im = Matrix::full(n, br.cols(), 0.0);
    for c in 0..br.cols() {
        for r in 0..n {
            re.set(r, c, sol.get(r, c));
            im.set(r, c, sol.get(n + r, c));
        }
    }
    Ok(from_parts(&re, &im))
}

/// `/` مختلط = (b'\a')' سمانتیک متلب.
pub fn c_mrdivide(a: &ComplexMatrix, b: &ComplexMatrix) -> Result<ComplexMatrix, McError> {
    let at = c_transpose(a, true);
    let bt = c_transpose(b, true);
    let x = c_backslash(&bt, &at)?;
    Ok(c_transpose(&x, true))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn z(re: f64, im: f64) -> ComplexMatrix {
        ComplexMatrix::scalar(re, im)
    }

    #[test]
    fn complex_arithmetic_basics() {
        // (1+2i) + (3−i) = 4+i
        let s = c_add(&z(1.0, 2.0), &z(3.0, -1.0)).unwrap_or_else(|e| panic!("{e:?}"));
        assert_eq!(s.get(0, 0), (4.0, 1.0));
        // (1+2i)(3−i) = 3 − i + 6i + 2 = 5 + 5i
        let p = c_elem_mul(&z(1.0, 2.0), &z(3.0, -1.0)).unwrap();
        assert_eq!(p.get(0, 0), (5.0, 5.0));
        // (1+i)/(1−i) = i
        let d = c_elem_div(&z(1.0, 1.0), &z(1.0, -1.0)).unwrap();
        assert_eq!(d.get(0, 0), (0.0, 1.0));
        // i² = −1 (با تلورانس — π در ممیز شناور دقیق نیست)
        let p2 = c_elem_pow(&z(0.0, 1.0), &z(2.0, 0.0)).unwrap();
        let (pr, pi) = p2.get(0, 0);
        assert!(
            (pr - -1.0).abs() < 1e-12 && pi.abs() < 1e-12,
            "i² = ({pr}, {pi})"
        );
        // 0^0 = 1
        let p3 = c_elem_pow(&z(0.0, 0.0), &z(0.0, 0.0)).unwrap();
        assert_eq!(p3.get(0, 0), (1.0, 0.0));
    }

    #[test]
    fn complex_matmul_block_formula() {
        // [[1 i]][[i]][1; i] — A(1×2)·B(2×1)
        let a = ComplexMatrix::from_columns(1, 2, vec![1.0, 0.0], vec![0.0, 1.0]);
        let b = ComplexMatrix::from_columns(2, 1, vec![0.0, 1.0], vec![1.0, 0.0]);
        let c = c_matmul(&a, &b).unwrap_or_else(|e| panic!("{e:?}"));
        // [1 i]·[i; 1] = 1·i + i·1 = 2i
        assert_eq!(c.get(0, 0), (0.0, 2.0));
    }

    #[test]
    fn complex_backslash_block_solve() {
        // (1+i)x = 2 → x = 1−i
        let a = z(1.0, 1.0);
        let b = z(2.0, 0.0);
        let x = c_backslash(&a, &b).unwrap_or_else(|e| panic!("{e:?}"));
        assert!((x.get(0, 0).0 - 1.0).abs() < 1e-12);
        assert!((x.get(0, 0).1 + 1.0).abs() < 1e-12);
        // mrdivide: 2/(1+i) = 1−i
        let d = c_mrdivide(&z(2.0, 0.0), &z(1.0, 1.0)).unwrap();
        assert!((d.get(0, 0).0 - 1.0).abs() < 1e-12);
        assert!((d.get(0, 0).1 + 1.0).abs() < 1e-12);
    }

    #[test]
    fn transpose_with_and_without_conjugate() {
        // z = [1+i; 2] — ' → [1−i 2] ، .' → [1+i 2]
        let z = ComplexMatrix::from_columns(2, 1, vec![1.0, 2.0], vec![1.0, 0.0]);
        let ct = c_transpose(&z, true);
        assert_eq!(ct.get(0, 0), (1.0, -1.0));
        assert_eq!(ct.get(0, 1), (2.0, 0.0));
        let pt = c_transpose(&z, false);
        assert_eq!(pt.get(0, 0), (1.0, 1.0));
    }

    #[test]
    fn matrix_literal_mixed_complex() {
        // [1 2i; 3 4+i] → مختلط ۲×۲
        let vals = vec![
            Value::Double(Matrix::scalar(1.0)),
            Value::Complex(ComplexMatrix::scalar(0.0, 2.0)),
            Value::Double(Matrix::scalar(3.0)),
            Value::Complex(ComplexMatrix::scalar(4.0, 1.0)),
        ];
        let v = build_matrix_value(&vals, 2, 2).unwrap_or_else(|e| panic!("{e:?}"));
        let Value::Complex(z) = &v else {
            panic!("expected complex, got {v:?}")
        };
        assert_eq!(z.get(0, 0), (1.0, 0.0));
        assert_eq!(z.get(0, 1), (0.0, 2.0));
        assert_eq!(z.get(1, 0), (3.0, 0.0));
        assert_eq!(z.get(1, 1), (4.0, 1.0));
        // همهٔ حقیقی → Double
        let real_vals = vec![
            Value::Double(Matrix::scalar(1.0)),
            Value::Double(Matrix::scalar(2.0)),
        ];
        let v2 = build_matrix_value(&real_vals, 1, 2).unwrap();
        assert!(matches!(v2, Value::Double(_)));
    }

    #[test]
    fn eq_elementwise_complex() {
        let a = ComplexMatrix::from_columns(1, 2, vec![1.0, 0.0], vec![0.0, 1.0]);
        let b = ComplexMatrix::from_columns(1, 2, vec![1.0, 0.0], vec![0.0, 2.0]);
        let r = c_eq(&a, &b).unwrap_or_else(|e| panic!("{e:?}"));
        assert_eq!(r.get(0, 0), 1.0);
        assert_eq!(r.get(0, 1), 0.0);
    }
}
