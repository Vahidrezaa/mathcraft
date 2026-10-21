//! ComplexMatrix — ماتریس مختلط column-major با Copy-on-Write (P2 — سند ۰۳ بخش ۳).
//!
//! قراردادها عیناً مثل [`crate::Matrix`]:
//! - چیدمان داده: `re[c*rows + r]` و `im[c*rows + r]` — ستون‌ها پشت سر هم.
//! - COW با `Arc<ThinVec<f64>>` — کپی O(1)، نوشتن فقط هنگام اشتراک clone می‌کند.
//! - بخش موهومی همیشه ذخیره می‌شود حتی اگر همه صفر باشند (`1+0i` از `1` متمایز
//!   ذخیره می‌شود؛ لایه‌های بالا با [`ComplexMatrix::is_all_real`] تصمیم می‌گیرند —
//!   رفتار متلب: `isreal(complex(1,0))` منطقاً `false` است ولی `isequal` برابر می‌داند).
//!
//! در متلب مختلط کلاسِ جدا ندارد: `class(1+2i)` همان `double` است —
//! پس `Value::class_name` برای `Complex` هم `double` می‌دهد.

use std::sync::Arc;

use thin_vec::ThinVec;

use crate::matrix::Matrix;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ComplexMatrix {
    rows: usize,
    cols: usize,
    re: Arc<ThinVec<f64>>,
    im: Arc<ThinVec<f64>>,
}

impl ComplexMatrix {
    /// ماتریس مختلط از داده چیده‌شده **ستونی** — قرارداد اصلی MathCraft.
    ///
    /// # Panics (فقط قرارداد برنامه‌نویس، نه مسیر محصول)
    /// اگر طول‌ها با ابعاد نخواند؛ در مسیر محصول داده از توابع داخلی می‌آید.
    pub fn from_columns(rows: usize, cols: usize, re: Vec<f64>, im: Vec<f64>) -> Self {
        assert_eq!(
            re.len(),
            rows * cols,
            "from_columns: real part length mismatch"
        );
        assert_eq!(
            im.len(),
            rows * cols,
            "from_columns: imaginary part length mismatch"
        );
        Self {
            rows,
            cols,
            re: Arc::new(ThinVec::from_iter(re)),
            im: Arc::new(ThinVec::from_iter(im)),
        }
    }

    /// ماتریس مختلط از ماتریس حقیقی — بخش موهومی صفر.
    pub fn from_real(m: &Matrix) -> Self {
        let data = m.as_column_slice().to_vec();
        let n = data.len();
        Self::from_columns(m.rows(), m.cols(), data, vec![0.0; n])
    }

    pub fn full(rows: usize, cols: usize, re: f64, im: f64) -> Self {
        Self::from_columns(
            rows,
            cols,
            vec![re; rows * cols],
            vec![im; rows * cols],
        )
    }

    /// اسکالر مختلط ۱×۱.
    pub fn scalar(re: f64, im: f64) -> Self {
        Self::full(1, 1, re, im)
    }

    /// واحد موهومی `i` — اسکالر ۰+۱i.
    pub fn imaginary_unit() -> Self {
        Self::scalar(0.0, 1.0)
    }

    pub fn rows(&self) -> usize {
        self.rows
    }

    pub fn cols(&self) -> usize {
        self.cols
    }

    pub fn numel(&self) -> usize {
        self.rows * self.cols
    }

    pub fn is_scalar(&self) -> bool {
        self.rows == 1 && self.cols == 1
    }

    pub fn is_empty(&self) -> bool {
        self.rows == 0 || self.cols == 0
    }

    /// دسترسی با subscripts — اندیس‌ها ۰-پایه داخلی‌اند؛ خروجی (re، im).
    pub fn get(&self, r: usize, c: usize) -> (f64, f64) {
        debug_assert!(r < self.rows && c < self.cols, "index out of bounds");
        let idx = c * self.rows + r;
        (self.re[idx], self.im[idx])
    }

    /// دسترسی خطی column-major — مثل `z(idx)` در متلب.
    pub fn get_linear(&self, idx: usize) -> (f64, f64) {
        debug_assert!(idx < self.numel(), "linear index out of bounds");
        (self.re[idx], self.im[idx])
    }

    /// نوشتن با COW.
    pub fn set(&mut self, r: usize, c: usize, re: f64, im: f64) {
        debug_assert!(r < self.rows && c < self.cols, "index out of bounds");
        let idx = c * self.rows + r;
        Arc::make_mut(&mut self.re)[idx] = re;
        Arc::make_mut(&mut self.im)[idx] = im;
    }

    /// نوشتن خطی با COW.
    pub fn set_linear(&mut self, idx: usize, re: f64, im: f64) {
        debug_assert!(idx < self.numel(), "linear index out of bounds");
        Arc::make_mut(&mut self.re)[idx] = re;
        Arc::make_mut(&mut self.im)[idx] = im;
    }

    /// نمای خام ستونی بخش حقیقی.
    pub fn re_column_slice(&self) -> &[f64] {
        &self.re
    }

    /// نمای خام ستونی بخش موهومی.
    pub fn im_column_slice(&self) -> &[f64] {
        &self.im
    }

    /// آیا هیچ عنصری بخش موهومی غیرصفر ندارد؟
    /// (`1+0i` → false مثل سمانتیک متلب در `isreal`)
    pub fn is_all_real(&self) -> bool {
        self.im.iter().all(|v| *v == 0.0)
    }

    /// بخش حقیقی — مثل `real(z)` متلب.
    pub fn real_part(&self) -> Matrix {
        Matrix::from_columns(self.rows, self.cols, self.re.as_ref().to_vec())
    }

    /// بخش موهومی — مثل `imag(z)` متلب.
    pub fn imag_part(&self) -> Matrix {
        Matrix::from_columns(self.rows, self.cols, self.im.as_ref().to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn column_major_layout_matches_matrix() {
        // z = [1+10i 2+20i; 3+30i 4+40i]
        let z = ComplexMatrix::from_columns(
            2,
            2,
            vec![1.0, 3.0, 2.0, 4.0],
            vec![10.0, 30.0, 20.0, 40.0],
        );
        assert_eq!(z.get(0, 0), (1.0, 10.0));
        assert_eq!(z.get(1, 0), (3.0, 30.0));
        assert_eq!(z.get(0, 1), (2.0, 20.0));
        assert_eq!(z.get_linear(2), (2.0, 20.0));
    }

    #[test]
    fn copy_on_write_semantics() {
        let mut a = ComplexMatrix::from_columns(2, 1, vec![1.0, 2.0], vec![0.0, 0.0]);
        let b = a.clone();
        a.set(0, 0, 99.0, 7.0);
        // b دست‌نخورده می‌ماند (value semantics متلب)
        assert_eq!(b.get(0, 0), (1.0, 0.0));
        assert_eq!(a.get(0, 0), (99.0, 7.0));
    }

    #[test]
    fn is_all_real_matches_matlab_isreal() {
        let real_like = ComplexMatrix::scalar(5.0, 0.0);
        assert!(real_like.is_all_real());
        let truly = ComplexMatrix::scalar(5.0, 0.0);
        assert!(truly.is_all_real());
        let imag = ComplexMatrix::scalar(5.0, 1.0);
        assert!(!imag.is_all_real());
        assert_eq!(ComplexMatrix::imaginary_unit().get(0, 0), (0.0, 1.0));
    }

    #[test]
    fn parts_round_trip() {
        let m = Matrix::from_columns(2, 1, vec![3.0, -1.5]);
        let z = ComplexMatrix::from_real(&m);
        assert!(z.is_all_real());
        assert_eq!(z.real_part(), m);
        assert_eq!(z.imag_part(), Matrix::full(2, 1, 0.0));
    }
}
