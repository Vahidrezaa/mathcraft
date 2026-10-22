//! ماتریس column-major با Copy-on-Write (سند ۰۲ بخش ۵، تصمیم D-11).
//!
//! - چیدمان داده: `data[c * rows + r]` — ستون‌ها پشت سر هم، دقیقاً مثل MATLAB.
//! - `set` از `Arc::make_mut` استفاده می‌کند؛ یعنی اگر کسی دیگر این داده را
//!   نگه داشته باشد، فقط همان لحظه clone می‌شود (COW).

use std::sync::Arc;

use thin_vec::ThinVec;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Matrix {
    rows: usize,
    cols: usize,
    data: Arc<ThinVec<f64>>,
}

impl Matrix {
    /// ماتریس پُر از یک مقدار.
    pub fn full(rows: usize, cols: usize, value: f64) -> Self {
        Self {
            rows,
            cols,
            data: Arc::new(vec![value; rows * cols].into()),
        }
    }

    /// ماتریس اسکالر ۱×۱.
    pub fn scalar(v: f64) -> Self {
        Self::full(1, 1, v)
    }

    /// ساخت از داده چیده‌شده **ستونی** (column-major) — قرارداد اصلی MathCraft.
    ///
    /// # Panics (فقط قرارداد برنامه‌نویس، نه مسیر محصول)
    /// اگر `data.len() != rows * cols` باشد؛ در مسیر محصول داده از توابع
    /// داخلی می‌آید که طول را تضمین می‌کنند.
    pub fn from_columns(rows: usize, cols: usize, data: Vec<f64>) -> Self {
        assert_eq!(
            data.len(),
            rows * cols,
            "from_columns: data length mismatch"
        );
        Self {
            rows,
            cols,
            data: Arc::new(ThinVec::from_iter(data)),
        }
    }

    pub fn rows(&self) -> usize {
        self.rows
    }

    pub fn cols(&self) -> usize {
        self.cols
    }

    /// تعداد کل عناصر (`numel` در متلب).
    pub fn numel(&self) -> usize {
        self.rows * self.cols
    }

    pub fn is_scalar(&self) -> bool {
        self.rows == 1 && self.cols == 1
    }

    pub fn is_empty(&self) -> bool {
        self.rows == 0 || self.cols == 0
    }

    /// حذف سطرها (اندیس‌های ۰-پایه) — `A(i,:)=[]` و حذف «سطر» با اندیس خطی
    /// (رفتار متلب). ورودی باید در محدوده باشد؛ یکتاسازی/مرتب‌سازی با تماس‌گیرنده.
    pub fn remove_rows(&self, rows: &[usize]) -> Matrix {
        let drop: std::collections::HashSet<usize> = rows.iter().copied().collect();
        let new_rows = self.rows - rows.len();
        let mut data = Vec::with_capacity(new_rows * self.cols);
        for c in 0..self.cols {
            for r in 0..self.rows {
                if !drop.contains(&r) {
                    data.push(self.data[c * self.rows + r]);
                }
            }
        }
        Matrix::from_columns(new_rows, self.cols, data)
    }

    /// حذف ستون‌ها (اندیس‌های ۰-پایه) — `A(:,j)=[]` و حذف المان بردار سطری.
    pub fn remove_cols(&self, cols: &[usize]) -> Matrix {
        let drop: std::collections::HashSet<usize> = cols.iter().copied().collect();
        let new_cols = self.cols - cols.len();
        let mut data = Vec::with_capacity(self.rows * new_cols);
        for c in 0..self.cols {
            if drop.contains(&c) {
                continue;
            }
            for r in 0..self.rows {
                data.push(self.data[c * self.rows + r]);
            }
        }
        Matrix::from_columns(self.rows, new_cols, data)
    }

    /// دسترسی با subscripts — اندیس‌ها ۰-پایه داخلی‌اند (لایه زبان ۱-پایه را می‌سازد).
    pub fn get(&self, r: usize, c: usize) -> f64 {
        debug_assert!(r < self.rows && c < self.cols, "index out of bounds");
        self.data[c * self.rows + r]
    }

    /// دسترسی خطی (column-major) — مثل `A(idx)` در متلب.
    pub fn get_linear(&self, idx: usize) -> f64 {
        debug_assert!(idx < self.numel(), "linear index out of bounds");
        self.data[idx]
    }

    /// نوشتن با COW: اگر Arc مشترک باشد، همین‌جا clone می‌شود.
    pub fn set(&mut self, r: usize, c: usize, v: f64) {
        debug_assert!(r < self.rows && c < self.cols, "index out of bounds");
        Arc::make_mut(&mut self.data)[c * self.rows + r] = v;
    }

    /// نوشتن خطی با COW.
    pub fn set_linear(&mut self, idx: usize, v: f64) {
        debug_assert!(idx < self.numel(), "linear index out of bounds");
        Arc::make_mut(&mut self.data)[idx] = v;
    }

    /// نمای خام ستونی (برای پیش‌نمایش بازرس، ذخیره‌سازی و bridge).
    pub fn as_column_slice(&self) -> &[f64] {
        &self.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn column_major_layout_is_matlab_compatible() {
        // M = [1 2; 3 4] در متلب: ستون اول (1,3) و ستون دوم (2,4).
        let m = Matrix::from_columns(2, 2, vec![1.0, 3.0, 2.0, 4.0]);
        assert_eq!(m.get(0, 0), 1.0);
        assert_eq!(m.get(1, 0), 3.0);
        assert_eq!(m.get(0, 1), 2.0);
        assert_eq!(m.get(1, 1), 4.0);
        // اندیس خطی متلب: M(3) = 2 (پایین ستون اول).
        assert_eq!(m.get_linear(2), 2.0);
    }

    #[test]
    fn copy_on_write_semantics() {
        let mut a = Matrix::from_columns(2, 1, vec![10.0, 20.0]);
        let b = a.clone();
        a.set(0, 0, 99.0);
        // b دست‌نخورده می‌ماند (value semantics متلب، سند ۰۳ بخش ۱۱ بند ۱).
        assert_eq!(b.get(0, 0), 10.0);
        assert_eq!(a.get(0, 0), 99.0);
        assert_eq!(b.numel(), 2);
    }

    #[test]
    fn scalar_helpers() {
        let s = Matrix::scalar(2.5);
        assert!(s.is_scalar());
        assert_eq!(s.numel(), 1);
        assert_eq!(s.get(0, 0), 2.5);
        assert!(!Matrix::full(2, 2, 0.0).is_scalar());
    }
}
