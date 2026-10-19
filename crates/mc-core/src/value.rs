//! Value — نوع داده زمان‌اجرای زبان (سند ۰۲ بخش ۵ و سند ۰۳ بخش ۳).
//!
//! در نسخه ۱، `double` (ماتریس f64) شهروند درجه‌یک است (سند ۰۳ بخش ۳:
//! «عمق به‌جای عرض»). واریانت‌های بعدی طبق سند ۰۲ بخش ۵ در P1+ اضافه می‌شوند:
//! `Logical`، `Char`، `String`، `Complex`، `Cell`، `Struct`، `Sym`، `FunctionHandle`.

use crate::matrix::Matrix;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// ماتریس ۰×۰ هم با `Matrix::full(0, 0, 0.0)` ساخته می‌شود (سمانتیک `[]` — سند ۰۳ بخش ۴).
    Double(Matrix),
}

impl Value {
    /// نام کلاس به سبک متلب — برای بازرس متغیرها و `workspace_delta` (سند ۰۲ بخش ۳).
    pub fn class_name(&self) -> &'static str {
        match self {
            Value::Double(_) => "double",
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
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn class_and_shape() {
        let v = Value::Double(Matrix::full(2, 3, 0.0));
        assert_eq!(v.class_name(), "double");
        assert_eq!(v.shape(), (2, 3));
        assert_eq!(v.as_scalar(), None);
        assert_eq!(Value::Double(Matrix::scalar(7.0)).as_scalar(), Some(7.0));
    }
}
