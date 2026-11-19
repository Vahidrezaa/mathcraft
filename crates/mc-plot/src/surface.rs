//! هندسهٔ سه‌بعدی شبکه (سند ۰۴ §8.3): نرمال‌های رأس در **Rust** محاسبه
//! می‌شوند (مثلث‌بندی دو مثلث در هر خانه در رندرر؛ اینجا فقط نرمال و
//! دامنه‌ها). سطح‌های خودکار contour (ده سطح یکنواخت روی z) هم اینجاست.

use crate::ticks::linear_ticks;

/// نرمال رأس برای هر نقطهٔ شبکه — میانگین نرمالِ خانه‌های مجاور
/// (الگوی متلب برای shading interp). خروجی: تخت row-major با stride ۳
/// (nx, ny, nz به‌ترتیب) — طول = rows*cols*3.
///
/// z[i][j]: سطر i = y_i ثابت، ستون j = x_j ثابت (meshgrid متلب).
pub fn vertex_normals(x: &[f64], y: &[f64], z: &[Vec<f64>]) -> Vec<f32> {
    let rows = z.len();
    let cols = z.first().map_or(0, Vec::len);
    if rows == 0 || cols == 0 || x.len() != cols || y.len() != rows {
        return Vec::new();
    }
    let mut out = vec![0.0f32; rows * cols * 3];
    // جمع نرمال وجه‌های خانهٔ (i, j) (شمال‌غربی‌ترین گوشه = رأس (i,j))؛
    // نرمال وجه = قرینهٔ ضرب خارجی لبه‌ها (رو به بالا برای z مثبت).
    for i in 0..rows.saturating_sub(1) {
        for j in 0..cols.saturating_sub(1) {
            let (x0, x1) = (x[j], x[j + 1]);
            let (y0, y1) = (y[i], y[i + 1]);
            let (z00, z10, z01, z11) = (z[i][j], z[i][j + 1], z[i + 1][j], z[i + 1][j + 1]);
            // دو مثلث: (00,10,11) و (00,11,01) — نرمال = ضرب خارجی لبه‌ها
            for tri in [
                ([x0, y0, z00], [x1, y0, z10], [x1, y1, z11]),
                ([x0, y0, z00], [x1, y1, z11], [x0, y1, z01]),
            ] {
                let (a, b, c) = tri;
                let u = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
                let v = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
                let n = [
                    u[1] * v[2] - u[2] * v[1],
                    u[2] * v[0] - u[0] * v[2],
                    u[0] * v[1] - u[1] * v[0],
                ];
                for &(vi, vj) in &[(i, j), (i + 1, j), (i, j + 1), (i + 1, j + 1)] {
                    let base = (vi * cols + vj) * 3;
                    out[base] += n[0] as f32;
                    out[base + 1] += n[1] as f32;
                    out[base + 2] += n[2] as f32;
                }
            }
        }
    }
    // نرمال‌سازی
    let mut k = 0;
    while k < out.len() {
        let len = (out[k] * out[k] + out[k + 1] * out[k + 1] + out[k + 2] * out[k + 2]).sqrt();
        if len > 1e-12 {
            out[k] /= len;
            out[k + 1] /= len;
            out[k + 2] /= len;
        } else {
            out[k] = 0.0;
            out[k + 1] = 0.0;
            out[k + 2] = 1.0;
        }
        k += 3;
    }
    out
}

/// دامنهٔ [zmin, zmax] شبکه — برای caxis خودکار.
pub fn z_range(z: &[Vec<f64>]) -> Option<[f64; 2]> {
    let mut it = z.iter().flat_map(|r| r.iter()).filter(|v| v.is_finite());
    let mut lo = *it.next()?;
    let mut hi = lo;
    for v in it {
        if *v < lo {
            lo = *v;
        }
        if *v > hi {
            hi = *v;
        }
    }
    Some([lo, hi])
}

/// ده سطح خودکار contour — فاصلهٔ یکنواخت «خوب» روی دامنهٔ z (نه
/// لگاریتمی؛ متلب هم یکنواخت می‌گذارد)، بدون قرارگیری روی zmin/zmax
/// دقیق تا خانه‌های مرزی حالت‌های تحریف marching squares نگیرند.
pub fn auto_levels(z: &[Vec<f64>], count: usize) -> Vec<f64> {
    let Some([lo, hi]) = z_range(z) else {
        return Vec::new();
    };
    if hi <= lo {
        return vec![lo];
    }
    // از ژنراتور تیک 1-2-5 برای گام‌های «خوب» استفاده می‌کنیم، بعد
    // دو سر دامنه را بیرون می‌ریزیم (متلب هم سطح روی دقیقاً zmin نمی‌گذارد).
    let mut levels: Vec<f64> = linear_ticks(lo, hi, count)
        .into_iter()
        .filter(|t| *t > lo && *t < hi)
        .collect();
    if levels.is_empty() {
        levels.push(lo + (hi - lo) / 2.0);
    }
    levels
}

#[cfg(test)]
mod tests {
    use super::*;

    fn grid() -> (Vec<f64>, Vec<f64>, Vec<Vec<f64>>) {
        // صفحهٔ z=1 با یک قله در مرکز
        let x = vec![-1.0, 0.0, 1.0];
        let y = vec![-1.0, 0.0, 1.0];
        let z = vec![
            vec![1.0, 1.0, 1.0],
            vec![1.0, 2.0, 1.0],
            vec![1.0, 1.0, 1.0],
        ];
        (x, y, z)
    }

    #[test]
    fn normals_point_up_on_flat_grid() {
        // صفحهٔ کاملاً صاف: همهٔ نرمال‌ها دقیقاً (0,0,1)
        let x = vec![-1.0, 0.0, 1.0];
        let y = vec![-1.0, 0.0, 1.0];
        let z = vec![vec![1.0; 3]; 3];
        let n = vertex_normals(&x, &y, &z);
        assert_eq!(n.len(), 9 * 3);
        for k in (0..n.len()).step_by(3) {
            assert!((n[k + 2] - 1.0).abs() < 1e-5, "رأس {k}: {:?}", &n[k..k + 3]);
            assert!(n[k].abs() < 1e-5 && n[k + 1].abs() < 1e-5);
        }
    }

    #[test]
    fn peaked_grid_normals_stay_upward() {
        // روی گرید قله‌دار، مؤلفهٔ z همهٔ رأس‌ها مثبت می‌ماند (بالا-غالب)
        let (x, y, z) = grid();
        let n = vertex_normals(&x, &y, &z);
        for k in (0..n.len()).step_by(3) {
            assert!(n[k + 2] > 0.5, "رأس {k}: {:?}", &n[k..k + 3]);
        }
    }

    #[test]
    fn normals_away_from_peak() {
        let (x, y, z) = grid();
        let n = vertex_normals(&x, &y, &z);
        // رأس مرکزی (1,1): نرمال y مثبت و x مثبت ندارد؟ مرکز قله است؛
        // رأس (0,1) (غرب قله) باید به سمت منفی x شیب داشته باشد
        let west = 3 * 3; // رأس (سطر ۰، ستون ۱) × stride ۳
        assert!(
            n[west] < -0.1,
            "غرب قله به سمت -x: {:?}",
            &n[west..west + 3]
        );
    }

    #[test]
    fn z_range_and_levels() {
        let (_, _, z) = grid();
        assert_eq!(z_range(&z), Some([1.0, 2.0]));
        let levels = auto_levels(&z, 10);
        assert!(!levels.is_empty());
        assert!(levels.iter().all(|l| (1.0..2.0).contains(l)));
    }

    #[test]
    fn degenerate_grid_is_safe() {
        assert!(vertex_normals(&[], &[], &[]).is_empty());
        let z = vec![vec![3.0]];
        assert_eq!(z_range(&z), Some([3.0, 3.0]));
        assert_eq!(auto_levels(&z, 10), vec![3.0]);
    }
}
