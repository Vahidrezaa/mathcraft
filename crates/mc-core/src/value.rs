//! Value — نوع داده زمان‌اجرای زبان (سند ۰۲ بخش ۵ و سند ۰۳ بخش ۳).
//!
//! نسخه P1: `double` (ماتریس f64) شهروند درجه‌یک + `char` برای رشته‌های
//! تک‌کوتیشن (سند ۰۳ §10) + function handle بی‌نام (سند ۰۳ §9 — capture-by-value).
//! واریانت‌های بعدی طبق سند ۰۲ بخش ۵ در P2+ اضافه می‌شوند:
//! `Logical`، `String`، `Complex`، `Cell`، `Struct`، `Sym`.
//! ROADMAP 2.9: `Struct` (ساختار ۱×۱ با نقشهٔ فیلد مرتب) اضافه شد —
//! آرایهٔ ساختاری (struct array) = انبساط E4.

use crate::complex::ComplexMatrix;
use crate::error::McError;
use crate::matrix::Matrix;
use std::fmt;
use std::sync::Arc;

/// حداکثر عمق تودرتویی ساختارها — گارد منابع در روح گام 2.5 (بودجهٔ حافظه):
/// Drop بازگشتی و محاسبهٔ عمق روی زنجیره‌های بی‌انتها هزینه می‌شوند؛ متلب
/// سقف مستندی ندارد (انحراف مستند COMPATIBILITY §1).
pub const MAX_STRUCT_DEPTH: usize = 256;

/// ساختار ۱×۱ با فیلدهای نام‌دار — نقشهٔ مرتب (ترتیب درج حفظ می‌شود،
/// سند ۰۳ §3). جست‌وجوی خطی: فیلدها کم‌شمارند. فقط ۱×۱؛ آرایهٔ ساختاری
/// نیاز به سلول دارد = انبساط E4 (خطای صادقانه).
#[derive(Debug, Clone, Default)]
pub struct StructVal {
    /// فیلدها به ترتیب درج — نام تکراری در ساخت کننده رد می‌شود.
    pub fields: Vec<(String, Value)>,
}

impl StructVal {
    pub fn new() -> Self {
        StructVal { fields: Vec::new() }
    }

    pub fn get(&self, name: &str) -> Option<&Value> {
        self.fields.iter().find(|(n, _)| n == name).map(|(_, v)| v)
    }

    /// درج/جایگزینی — نام موجود مقدارش عوض می‌شود، نام تازه انتها می‌رود.
    pub fn set(&mut self, name: &str, value: Value) {
        match self.fields.iter_mut().find(|(n, _)| n == name) {
            Some(slot) => slot.1 = value,
            None => self.fields.push((name.to_owned(), value)),
        }
    }

    pub fn field_count(&self) -> usize {
        self.fields.len()
    }
}

/// عمق تودرتویی مقدار — مقدار غیرساختاری ۰ است؛ ساختارِ بدون فیلد ساختاری ۱.
pub fn struct_depth(v: &Value) -> usize {
    match v {
        Value::Struct(s) => {
            1 + s
                .fields
                .iter()
                .map(|(_, v)| struct_depth(v))
                .max()
                .unwrap_or(0)
        }
        _ => 0,
    }
}

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
    /// ساختار ۱×۱ با فیلدهای نام‌دار (ROADMAP 2.9 — سند ۰۳ §3/§10).
    Struct(StructVal),
    /// عبارت سمبولیک (گام 4.1/CAS-0 — سند ۰۵ §۲): درخت دقیق؛ `class` متلب‌گونه
    /// آن "sym" است. حساب روی آن درخت را بزرگ می‌کند؛ عمل ناموجود = خطای
    /// صادقانه، هرگز جواب غلطِ عددی نه.
    Sym(Arc<mc_cas::Sym>),
    /// لیست مقادیر (گام 4.3/CAS-2 — قرارداد سند ۰۵ §۸: «solve همیشه list
    /// برمی‌گرداند»؛ ستون ریشه‌های سمبولیک). کلاس متلب‌گونه: `cell`
    /// (انحراف مستند COMPATIBILITY — متلب آرایهٔ sym برمی‌گرداند؛ ماتریس
    /// سمبولیک هنوز نداریم). سازندهٔ فعلی فقط solve است؛ حساب و
    /// اندیس‌گذاری روی cell هنوز خطای صادقانه می‌دهد (سند ۰۳ — Cell کامل بعداً).
    List(Vec<Value>),
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
            // سمبولیک: برابری ساختاری درخت (تست‌ها)؛ Rc ارزان clone می‌شود
            (Value::Sym(a), Value::Sym(b)) => a == b,
            // لیست: برابری عنصری (بازگشتی از طریق Value::eq)
            (Value::List(a), Value::List(b)) => a == b,
            // ساختارها: مجموعهٔ فیلد یکسان (ترتیب بی‌اثر — اوراکل Octave 10.3:
            // isequal(struct('a',1,'b',2), struct('b',2,'a',1)) = 1؛ رفتار متلب)
            (Value::Struct(a), Value::Struct(b)) => {
                a.fields.len() == b.fields.len()
                    && a.fields
                        .iter()
                        .all(|(n, v)| b.fields.iter().any(|(n2, v2)| n == n2 && v == v2))
            }
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
            Value::Struct(_) => "struct",
            Value::Sym(_) => "sym",
            Value::List(_) => "cell",
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
            // فقط ۱×۱ — آرایهٔ ساختاری = انبساط E4 (سند ۰۳ §3)
            Value::Struct(_) => (1, 1),
            // عبارت سمبولیک همیشه اسکالر است (CAS-0؛ ماتریس سمبولیک CAS-1+)
            Value::Sym(_) => (1, 1),
            // لیست = آرایهٔ ستونی N×1 (پاریت solve متلب: ستون ریشه‌ها)؛
            // خالی: 0×1 سمانتیک []
            Value::List(v) => (v.len(), 1),
        }
    }

    /// ردیف کدهای یک رشتهٔ char (1×N) — `'abc'` → `[97 98 99]`
    /// (سمانتیک «char در زمینهٔ عددی» متلب — ROADMAP 2.8، بند ۶/۱۲ ممیزی).
    pub fn char_codes_row(s: &str) -> Matrix {
        let codes: Vec<f64> = s.chars().map(|c| c as u32 as f64).collect();
        Matrix::from_columns(1, codes.len(), codes)
    }

    /// تبدیل کد عددی به کاراکتر — قرارداد `char` متلب: گرد کردن به سمت صفر
    /// (سند رسمی متلب). دامنهٔ موتور = codepoint کامل یونیکد `0..=0x10FFFF`
    /// بدون surrogate (مدل یونیکد سند ۰۳ §10)؛ بیرون از دامنه خطای صادقانه —
    /// متلب به 0/65535 اشباع می‌کند (انحراف مستند COMPATIBILITY).
    pub fn char_from_code(x: f64, line: u32, col: u32) -> Result<char, McError> {
        if x.is_nan() {
            return Err(McError::new(
                crate::error::ids::PARSE_ERROR,
                line,
                col,
                "NaN has no character code.",
                "NaN کد کاراکتری ندارد.",
            ));
        }
        let t = x.trunc();
        if !(0.0..=0x10FFFF as f64).contains(&t) {
            return Err(McError::new(
                crate::error::ids::UNSUPPORTED,
                line,
                col,
                format!("Character code {x} is outside the valid Unicode range 0..=1114111."),
                format!("کد کاراکتر {x} بیرون از دامنهٔ معتبر یونیکد 0..=1114111 است."),
            ));
        }
        char::from_u32(t as u32).ok_or_else(|| {
            McError::new(
                crate::error::ids::UNSUPPORTED,
                line,
                col,
                format!("Character code {x} is a UTF-16 surrogate, not a valid Unicode scalar."),
                format!("کد کاراکتر {x} جایگزینِ UTF-16 است و مقدار یونیکد معتبر نیست."),
            )
        })
    }
}
