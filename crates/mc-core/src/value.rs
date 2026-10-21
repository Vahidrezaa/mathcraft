//! Value — نوع داده زمان‌اجرای زبان (سند ۰۲ بخش ۵ و سند ۰۳ بخش ۳).
//!
//! نسخه P1: `double` (ماتریس f64) شهروند درجه‌یک + `char` برای رشته‌های
//! تک‌کوتیشن (سند ۰۳ §10) + function handle بی‌نام (سند ۰۳ §9 — capture-by-value).
//! واریانت‌های بعدی طبق سند ۰۲ بخش ۵ در P2+ اضافه می‌شوند:
//! `Logical`، `String`، `Complex`، `Cell`، `Struct`، `Sym`.

use crate::complex::ComplexMatrix;
use crate::error::McError;
use crate::matrix::Matrix;
use std::fmt;
use std::sync::Arc;

/// فراخوانیِ تابع بی‌نام — بدنهٔ AST و متغیرهای گرفته‌شده (capture-by-value)
/// داخل closure در mc-interp زندگی می‌کنند؛ mc-core فقط قرارداد فراخوانی را می‌بیند.
pub type AnonCall = Arc<dyn Fn(&[Value]) -> Result<Value, McError> + Send + Sync>;

/// تابع بی‌نام به‌عنوان مقدار (سند ۰۳ §9: `f = @(x) x.^2 + 1`).
pub struct AnonFn {
    /// نام پارامترها — برای کنترل arity در فراخوانی.
    pub params: Vec<String>,
    /// فراخوانی واقعی (snapshot متغیرها داخل closure است — رفتار متلب).
    pub call: AnonCall,
}

impl Clone for AnonFn {
    fn clone(&self) -> Self {
        Self {
            params: self.params.clone(),
            call: Arc::clone(&self.call),
        }
    }
}

impl fmt::Debug for AnonFn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AnonFn")
            .field("params", &self.params)
            .finish_non_exhaustive()
    }
}

#[derive(Debug, Clone)]
pub enum Value {
    /// ماتریس ۰×۰ هم با `Matrix::full(0, 0, 0.0)` ساخته می‌شود (سمانتیک `[]` — سند ۰۳ بخش ۴).
    Double(Matrix),
    /// ماتریس مختلط — در متلب کلاس جدا ندارد (`class(1+2i)` = double)؛
    /// نمایش و `isreal` لایهٔ بالا تصمیم می‌گیرد (سند ۰۳ بخش ۳ — P2).
    Complex(ComplexMatrix),
    /// رشته کاراکتری `'abc'` (سند ۰۳ §10 — کلاس char؛ یونیکد صحیح).
    Char(String),
    /// تابع بی‌نام (function handle).
    Anon(AnonFn),
}

impl PartialEq for Value {
    /// برابری ساختاری — برای تست‌ها. توابع بی‌نام هرگز برابر نیستند
    /// (مثل متلب که هندل‌ها را با isequal برابر نمی‌داند).
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Double(a), Value::Double(b)) => a == b,
            (Value::Complex(a), Value::Complex(b)) => a == b,
            // `isequal(1, 1+0i)` در متلب true است — از راه is_all_real برابر می‌کنیم
            (Value::Double(a), Value::Complex(b)) | (Value::Complex(b), Value::Double(a)) => {
                b.is_all_real() && a == &b.real_part()
            }
            (Value::Char(a), Value::Char(b)) => a == b,
            _ => false,
        }
    }
}

impl Value {
    /// نام کلاس به سبک متلب — برای بازرس متغیرها و `workspace_delta` (سند ۰۲ بخش ۳).
    pub fn class_name(&self) -> &'static str {
        match self {
            // مختلط هم کلاس double است (رفتار متلب)
            Value::Double(_) | Value::Complex(_) => "double",
            Value::Char(_) => "char",
            Value::Anon(_) => "function_handle",
        }
    }

    /// اگر اسکالر ۱×۱ باشد، مقدارش را می‌دهد (شرط‌ها و محاسبات اسکالری).
    pub fn as_scalar(&self) -> Option<f64> {
        match self {
            Value::Double(m) if m.is_scalar() => Some(m.get(0, 0)),
            _ => None,
        }
    }

    pub fn shape(&self) -> (usize, usize) {
        match self {
            Value::Double(m) => (m.rows(), m.cols()),
            Value::Complex(m) => (m.rows(), m.cols()),
            Value::Char(s) => (1, s.chars().count()),
            Value::Anon(_) => (1, 1),
        }
    }
}
