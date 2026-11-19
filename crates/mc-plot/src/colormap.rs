//! جدول‌های colormap (سند ۰۴ §8.3) — parula بازسازی‌شده با جدول ۱۶ نقطه‌ای
//! و lerp، به‌علاهٔ viridis (جدول ۱۶ نقطه‌ای) و سه پالت تحلیلی jet/hot/cool
//! و gray. **آینهٔ وفادار همین جدول‌ها در Dart** (`plot_colormap.dart`)
//! قرار می‌گیرد — الگوی تیک 1-2-5 (گام 3.4.1): هر تغییر اینجا باید آنجا
//! حمل شود؛ تست دوقطبی (`parity_tables_match_dart_port`) در همان فایل
//! تست Dart مقدارهای کلیدی را می‌خواند تا هم‌گامی بشکند (golden).

use crate::spec::Colormap;

/// جدول ۱۶ نقطه‌ای parula (بازسازی‌شده — تقریب پذیرفته‌شدهٔ سند ۰۴).
/// جدول داده‌ای است نه محاسبه — 0.7071 عمداً تقریب جدول است.
#[allow(clippy::approx_constant)]
const PARULA: [[f32; 3]; 16] = [
    [0.2081, 0.1663, 0.5292],
    [0.2110, 0.2358, 0.6445],
    [0.2121, 0.3182, 0.7369],
    [0.2088, 0.4005, 0.7975],
    [0.1951, 0.4754, 0.8208],
    [0.1685, 0.5320, 0.8093],
    [0.1298, 0.5699, 0.7776],
    [0.0893, 0.5958, 0.7354],
    [0.0599, 0.6163, 0.6924],
    [0.0812, 0.6517, 0.6353],
    [0.1870, 0.6833, 0.5560],
    [0.3449, 0.7048, 0.4690],
    [0.5200, 0.7151, 0.3800],
    [0.6861, 0.7071, 0.2959],
    [0.8313, 0.6741, 0.2230],
    [0.9424, 0.6105, 0.1753],
];

/// جدول ۱۶ نقطه‌ای viridis.
const VIRIDIS: [[f32; 3]; 16] = [
    [0.2670, 0.0049, 0.3294],
    [0.2826, 0.0949, 0.4173],
    [0.2954, 0.1804, 0.4948],
    [0.3080, 0.2587, 0.5575],
    [0.3228, 0.3337, 0.6047],
    [0.3423, 0.4074, 0.6370],
    [0.3681, 0.4795, 0.6548],
    [0.3990, 0.5495, 0.6590],
    [0.4345, 0.6164, 0.6500],
    [0.4735, 0.6790, 0.6291],
    [0.5160, 0.7368, 0.5963],
    [0.5625, 0.7884, 0.5520],
    [0.6144, 0.8323, 0.4958],
    [0.6731, 0.8648, 0.4265],
    [0.7407, 0.8841, 0.3422],
    [0.8194, 0.8878, 0.2374],
];

/// نمونه‌برداری خطی از جدول — t∈[0,1] (کلمپ).
pub fn sample(cm: Colormap, t: f32) -> [f32; 3] {
    let t = t.clamp(0.0, 1.0);
    match cm {
        Colormap::Parula => table_lerp(&PARULA, t),
        Colormap::Viridis => table_lerp(&VIRIDIS, t),
        Colormap::Jet => jet(t),
        Colormap::Hot => hot(t),
        Colormap::Cool => [t, 1.0 - t, 1.0],
        Colormap::Gray => [t, t, t],
    }
}

fn table_lerp(table: &[[f32; 3]; 16], t: f32) -> [f32; 3] {
    let pos = t * (table.len() - 1) as f32;
    let i = pos.floor() as usize;
    let j = (i + 1).min(table.len() - 1);
    let f = pos - i as f32;
    [
        table[i][0] + f * (table[j][0] - table[i][0]),
        table[i][1] + f * (table[j][1] - table[i][1]),
        table[i][2] + f * (table[j][2] - table[i][2]),
    ]
}

/// jet کلاسیک (فرمول استاندارد ۱.۵−|4t−k|، k∈{1,2,3}) — سر آبی تیره،
/// میانهٔ سبز، ته قرمز تیره (مثل متلب).
fn jet(t: f32) -> [f32; 3] {
    let four = 4.0 * t;
    let r = (1.5 - (four - 3.0).abs()).clamp(0.0, 1.0);
    let g = (1.5 - (four - 2.0).abs()).clamp(0.0, 1.0);
    let b = (1.5 - (four - 1.0).abs()).clamp(0.0, 1.0);
    [r, g, b]
}

/// hot کلاسیک: سیاه → قرمز → زرد → سفید.
fn hot(t: f32) -> [f32; 3] {
    let r = (t / 0.375).clamp(0.0, 1.0);
    let g = ((t - 0.375) / 0.375).clamp(0.0, 1.0);
    let b = ((t - 0.75) / 0.25).clamp(0.0, 1.0);
    [r, g, b]
}

/// دامنهٔ مؤثر رنگ سطح: caxis صریح یا [zmin, zmax] داده (زمان ساخت در
/// figure.rs resolve می‌شود؛ این تابع کمکی نمونه‌برداری نرمال‌شده است).
pub fn sample_z(cm: Colormap, z: f64, caxis: [f64; 2]) -> [f32; 3] {
    let span = caxis[1] - caxis[0];
    let t = if span > 0.0 {
        ((z - caxis[0]) / span) as f32
    } else {
        0.5
    };
    sample(cm, t)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parula_endpoints_are_blue_orange() {
        assert_eq!(sample(Colormap::Parula, 0.0), PARULA[0]);
        assert_eq!(sample(Colormap::Parula, 1.0), PARULA[15]);
        // سرِ آبی شفاف و تهِ نارنجی — ویژهٔ parula
        assert!(PARULA[0][2] > 0.5 && PARULA[15][0] > 0.9);
    }

    #[test]
    fn monotone_gray_and_cool_extremes() {
        assert_eq!(sample(Colormap::Gray, 0.0), [0.0, 0.0, 0.0]);
        assert_eq!(sample(Colormap::Gray, 1.0), [1.0, 1.0, 1.0]);
        assert_eq!(sample(Colormap::Cool, 0.0), [0.0, 1.0, 1.0]);
        assert_eq!(sample(Colormap::Cool, 1.0), [1.0, 0.0, 1.0]);
    }

    #[test]
    fn jet_shape_is_blue_green_red() {
        // میانه: قلهٔ سبز (r=b=0.5، g=1 — فرمول استاندارد)
        let [r, g, b] = sample(Colormap::Jet, 0.5);
        assert!(g > 0.99 && (r - 0.5).abs() < 1e-5 && (b - 0.5).abs() < 1e-5);
        // یک‌هشتم: آبی خالص
        let [r, g, b] = sample(Colormap::Jet, 0.125);
        assert!(r < 1e-5 && g < 1e-5 && b > 0.99);
        // هفت‌هشتم: قرمز خالص
        let [r, g, b] = sample(Colormap::Jet, 0.875);
        assert!(r > 0.99 && g < 1e-5 && b < 1e-5);
        let [r, _, _] = sample(Colormap::Jet, 0.0);
        assert!(r < 1e-5, "jet سرش آبی تیره است نه قرمز");
        let [_, _, b] = sample(Colormap::Jet, 1.0);
        assert!(b < 1e-5, "jet تهش قرمز تیره است نه آبی");
    }

    #[test]
    fn hot_is_black_to_white() {
        assert_eq!(sample(Colormap::Hot, 0.0), [0.0, 0.0, 0.0]);
        let [r, g, b] = sample(Colormap::Hot, 1.0);
        assert!(r == 1.0 && g == 1.0 && b == 1.0);
    }

    #[test]
    fn sample_z_normalizes_by_caxis() {
        let c = sample_z(Colormap::Gray, 5.0, [0.0, 10.0]);
        assert_eq!(c, [0.5, 0.5, 0.5]);
        // caxis تخت → وسط پالت
        let c = sample_z(Colormap::Gray, 3.0, [3.0, 3.0]);
        assert_eq!(c, [0.5, 0.5, 0.5]);
    }

    #[test]
    fn viridis_ends_are_purple_yellow() {
        let lo = sample(Colormap::Viridis, 0.0);
        let hi = sample(Colormap::Viridis, 1.0);
        assert!(lo[2] > 0.3 && lo[0] < 0.3, "سر بنفش");
        assert!(hi[1] > 0.85 && hi[2] < 0.3, "ته زرد");
    }
}
