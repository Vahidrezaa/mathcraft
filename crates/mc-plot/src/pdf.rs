//! خروجی PDF برداری (P3-b — ROADMAP 3.8، سند ۰۷ §P3): نویسندهٔ PDF 1.4
//! مینیمال بدون وابستگی — هر شکل یک صفحهٔ افقی A4؛ همهٔ نوع‌های سلول
//! دوبعدی + سه‌بعدی (تصویرpergelation دوربین + مرتب‌سازی عمق painter) +
//! نوار رنگ. متن با فونت‌های پایهٔ Helvetica (WinAnsi): **متن غیر
//! لاتین۱ با '?' جایگزین می‌شود** — محدودیت ثبت‌شده (doc 13)؛ مسیر
//! آینده: فونت تعبیه‌شده.
//!
//! نکتهٔ هم‌ارزی با رندرر Dart: پروجکشن سه‌بعدی و marching squares
//! «پورت وفادار» همین قواعد در Dart است (الگوی تیک 1-2-5 — گام 3.4.1).

use crate::colormap;
use crate::figure::{CellSnapshot, FigureSnapshot};
use crate::spec::{Colormap, SurfKind, View3};
use crate::ticks::format_g4;

const PAGE_W: f64 = 842.0;
const PAGE_H: f64 = 595.0;
const MARGIN: f64 = 46.0;

/// عملیات رسم محتوای PDF — سریال‌سازی خطی به content stream.
#[derive(Debug, Clone)]
enum Op {
    Stroke {
        r: f32,
        g: f32,
        b: f32,
        width: f32,
        dash: Option<(f32, f32)>,
        pts: Vec<(f64, f64)>,
    },
    PolyFill {
        r: f32,
        g: f32,
        b: f32,
        pts: Vec<(f64, f64)>,
    },
    Text {
        x: f64,
        y: f64,
        size: f32,
        bold: bool,
        text: String,
        rotate: bool,
    },
}

/// رندر یک شکل به بایت‌های کامل PDF (تک‌صفحه).
pub fn render_figure(fig: &FigureSnapshot) -> Vec<u8> {
    let mut ops = Vec::new();
    content_for_cell(&fig.spec, &mut ops);
    write_pdf(vec![ops])
}

/// رندر چند شکل — هر کدام یک صفحه.
pub fn render_figures(figs: &[FigureSnapshot]) -> Vec<u8> {
    let pages: Vec<Vec<Op>> = figs
        .iter()
        .map(|f| {
            let mut ops = Vec::new();
            content_for_cell(&f.spec, &mut ops);
            ops
        })
        .collect();
    write_pdf(pages)
}

// ---------------------------------------------------------------------------
// چیدمان مشترک
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
struct Frame {
    /// ناحیهٔ رسم داده (مبدأ پایین-چپ PDF).
    x0: f64,
    y0: f64,
    w: f64,
    h: f64,
}

impl Frame {
    /// نگاشت داده (نرمال‌شده 0..1) به صفحه.
    fn px(&self, t: f64) -> f64 {
        self.x0 + t * self.w
    }
    fn py(&self, t: f64) -> f64 {
        self.y0 + t * self.h
    }
}

/// قاب پایهٔ صفحه (بدون رزرو برچسب).
fn base_frame() -> Frame {
    Frame {
        x0: MARGIN,
        y0: MARGIN,
        w: PAGE_W - 2.0 * MARGIN,
        h: PAGE_H - 2.0 * MARGIN,
    }
}

/// جمع‌کردن قاب برای فضای برچسب‌ها (عنوان بالا، xlabel پایین، ylabel چپ).
fn inset(f: &Frame, has_ylabel: bool, has_title: bool, has_xlabel: bool) -> Frame {
    let top = if has_title { 20.0 } else { 4.0 };
    let bottom = if has_xlabel { 20.0 } else { 4.0 };
    let left = if has_ylabel { 26.0 } else { 4.0 };
    Frame {
        x0: f.x0 + left,
        y0: f.y0 + bottom,
        w: f.w - left - 6.0,
        h: f.h - top - bottom,
    }
}

fn push_axes_frame(ops: &mut Vec<Op>, f: &Frame) {
    ops.push(Op::Stroke {
        r: 0.55,
        g: 0.55,
        b: 0.55,
        width: 0.8,
        dash: None,
        pts: vec![
            (f.x0, f.y0),
            (f.x0 + f.w, f.y0),
            (f.x0 + f.w, f.y0 + f.h),
            (f.x0, f.y0 + f.h),
            (f.x0, f.y0),
        ],
    });
}

fn push_title_and_labels(
    ops: &mut Vec<Op>,
    title: &Option<String>,
    xlabel: &Option<String>,
    ylabel: &Option<String>,
) {
    if let Some(t) = title {
        ops.push(Op::Text {
            x: PAGE_W / 2.0 - t.len() as f64 * 3.3,
            y: PAGE_H - MARGIN + 18.0,
            size: 13.0,
            bold: true,
            text: t.clone(),
            rotate: false,
        });
    }
    if let Some(l) = xlabel {
        ops.push(Op::Text {
            x: PAGE_W / 2.0 - l.len() as f64 * 2.8,
            y: MARGIN - 14.0,
            size: 10.0,
            bold: false,
            text: l.clone(),
            rotate: false,
        });
    }
    if let Some(l) = ylabel {
        ops.push(Op::Text {
            x: MARGIN - 30.0,
            y: PAGE_H / 2.0 - l.len() as f64 * 2.8,
            size: 10.0,
            bold: false,
            text: l.clone(),
            rotate: true,
        });
    }
}

fn colorbar(ops: &mut Vec<Op>, cm: Colormap, caxis: [f64; 2], f: &Frame) {
    let cb_x = f.x0 + f.w + 14.0;
    let cb_w = 12.0;
    let steps = 64;
    for k in 0..steps {
        let t = k as f64 / steps as f64;
        let [r, g, b] = colormap::sample(cm, t as f32);
        let y0 = f.y0 + t * f.h;
        ops.push(Op::PolyFill {
            r,
            g,
            b,
            pts: vec![
                (cb_x, y0),
                (cb_x + cb_w, y0),
                (cb_x + cb_w, y0 + f.h / steps as f64),
                (cb_x, y0 + f.h / steps as f64),
            ],
        });
    }
    ops.push(Op::Stroke {
        r: 0.4,
        g: 0.4,
        b: 0.4,
        width: 0.7,
        dash: None,
        pts: vec![
            (cb_x, f.y0),
            (cb_x + cb_w, f.y0),
            (cb_x + cb_w, f.y0 + f.h),
            (cb_x, f.y0 + f.h),
            (cb_x, f.y0),
        ],
    });
    // سه برچسب: دو سر + میانه
    for (t, v) in [
        (0.0, caxis[0]),
        (0.5, (caxis[0] + caxis[1]) / 2.0),
        (1.0, caxis[1]),
    ] {
        ops.push(Op::Text {
            x: cb_x + cb_w + 4.0,
            y: f.y0 + t * f.h - 3.0,
            size: 8.0,
            bold: false,
            text: format_g4(v),
            rotate: false,
        });
    }
}

// ---------------------------------------------------------------------------
// دوبعدی
// ---------------------------------------------------------------------------

struct Mapper {
    f: Frame,
    lo: [f64; 2],
    hi: [f64; 2],
}

impl Mapper {
    fn new(f: Frame, lo: [f64; 2], hi: [f64; 2]) -> Self {
        let fix = |a: f64, b: f64| {
            if b - a > 1e-12 {
                (a, b)
            } else {
                (a - 1.0, b + 1.0)
            }
        };
        let (lox, hix) = fix(lo[0], hi[0]);
        let (loy, hiy) = fix(lo[1], hi[1]);
        Self {
            f,
            lo: [lox, loy],
            hi: [hix, hiy],
        }
    }
    fn map(&self, x: f64, y: f64) -> (f64, f64) {
        let nx = ((x - self.lo[0]) / (self.hi[0] - self.lo[0])).clamp(0.0, 1.0);
        let ny = ((y - self.lo[1]) / (self.hi[1] - self.lo[1])).clamp(0.0, 1.0);
        (self.f.px(nx), self.f.py(ny))
    }
}

fn bounds_of(xs: &[f64], ys: &[f64]) -> ((f64, f64), (f64, f64)) {
    let fin = |v: &[f64]| {
        let f: Vec<f64> = v.iter().copied().filter(|p| p.is_finite()).collect();
        if f.is_empty() {
            (0.0, 1.0)
        } else {
            (
                f.iter().copied().fold(f64::INFINITY, f64::min),
                f.iter().copied().fold(f64::NEG_INFINITY, f64::max),
            )
        }
    };
    let (a, b) = fin(xs);
    let (c, d) = fin(ys);
    ((a, b), (c, d))
}

fn pad(lo: f64, hi: f64) -> (f64, f64) {
    let span = (hi - lo).max(1e-9);
    (lo - span * 0.05, hi + span * 0.05)
}

fn line_series_ops(ops: &mut Vec<Op>, m: &Mapper, pts: &[(f64, f64)], color: [f32; 3], width: f32) {
    let mapped: Vec<(f64, f64)> = pts
        .iter()
        .map(|&(x, y)| m.map(x, y))
        .filter(|(x, y)| x.is_finite() && y.is_finite())
        .collect();
    if mapped.len() >= 2 {
        ops.push(Op::Stroke {
            r: color[0],
            g: color[1],
            b: color[2],
            width,
            dash: None,
            pts: mapped,
        });
    }
}

fn palette_rgb(i: usize) -> [f32; 3] {
    // آینهٔ سیکل پالت ۸رنگی سند ۰۶ §3 (مقادیر sRGB).
    const P: [[f32; 3]; 8] = [
        [0.000, 0.447, 0.741],
        [0.850, 0.325, 0.098],
        [0.929, 0.694, 0.125],
        [0.494, 0.184, 0.556],
        [0.466, 0.674, 0.188],
        [0.301, 0.745, 0.933],
        [0.635, 0.078, 0.184],
        [0.400, 0.400, 0.400],
    ];
    P[i % 8]
}

// ---------------------------------------------------------------------------
// سه‌بعدی — پروجکشن view (پورت وفادار؛ Dart همان فرمول را دارد)
// ---------------------------------------------------------------------------

/// چرخش دوربین متلب: az (درجه حول z، مثبت پادساعتگرد از -y) + el (زاویه
/// بالا). خروجی: (sx, sy, depth) برای نقطهٔ دادهٔ نرمال‌شدهٔ [0,1]³.
pub fn project(v: View3, x: f64, y: f64, z: f64) -> (f64, f64, f64) {
    let az = v.az.to_radians();
    let el = v.el.to_radians();
    let (sin_az, cos_az) = (az.sin(), az.cos());
    let (sin_el, cos_el) = (el.sin(), el.cos());
    // بردار دید متلب: دوربین در جهت (cos el sin az, -cos el cos az, sin el)
    // به مبدأ؛ محورهای صفحه:
    //   راستِ صفحه = (cos az, sin az, 0)
    //   بالای صفحه = (-sin el sin az, sin el cos az, cos el)
    //   عمق = بردار دید
    let sx = x * cos_az + y * sin_az;
    let sy = x * (-sin_el * sin_az) + y * (sin_el * cos_az) + z * cos_el;
    let depth = x * (cos_el * sin_az) + y * (-cos_el * cos_az) + z * sin_el;
    (sx, sy, depth)
}

#[allow(clippy::too_many_arguments)]
fn surface_ops(
    ops: &mut Vec<Op>,
    x: &[f64],
    y: &[f64],
    z: &[Vec<f64>],
    kind: SurfKind,
    cm: Colormap,
    caxis: Option<[f64; 2]>,
    view: View3,
    f: Frame,
) {
    let rows = z.len();
    let cols = z.first().map_or(0, Vec::len);
    if rows == 0 || cols == 0 || x.len() != cols || y.len() != rows {
        return;
    }
    let (xlo, xhi) = (x[0], x[cols - 1]);
    let (ylo, yhi) = (y[0], y[rows - 1]);
    let [zlo, zhi] = crate::surface::z_range(z).unwrap_or([0.0, 1.0]);
    let nz = |v: f64| {
        if zhi > zlo {
            (v - zlo) / (zhi - zlo)
        } else {
            0.5
        }
    };
    let cax = caxis.unwrap_or([zlo, zhi]);
    let norm = |v: f64, lo: f64, hi: f64| {
        if hi > lo {
            ((v - lo) / (hi - lo)).clamp(0.0, 1.0)
        } else {
            0.5
        }
    };
    // همهٔ رأس‌ها را پروجکت می‌کنیم؛ سپس خانه‌ها را بر عمق مرکز مرتب.
    let mut verts: Vec<(f64, f64, f64)> = Vec::with_capacity(rows * cols);
    for (i, yi) in y.iter().enumerate() {
        for (j, xi) in x.iter().enumerate() {
            let (nx, ny) = (norm(*xi, xlo, xhi), norm(*yi, ylo, yhi));
            verts.push(project(view, nx, ny, nz(z[i][j])));
        }
    }
    let vertex = |i: usize, j: usize| verts[i * cols + j];
    let mut quads: Vec<(usize, usize, f64)> = Vec::with_capacity((rows - 1) * (cols - 1));
    for i in 0..rows - 1 {
        for j in 0..cols - 1 {
            let d = (vertex(i, j).2 + vertex(i + 1, j + 1).2) * 0.5;
            quads.push((i, j, d));
        }
    }
    quads.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
    // نور ثابت سرِ دوربین (camlight پیش‌فرض)
    let light = {
        let (lx, ly, lz) = project(view, 0.5, 0.5, 2.0);
        let len = (lx * lx + ly * ly + lz * lz).sqrt();
        (lx / len, ly / len, lz / len)
    };
    let mut fill: Vec<((usize, usize), [f32; 3])> = Vec::new();
    let mut wire: Vec<(usize, usize)> = Vec::new();
    for &(i, j, _) in &quads {
        let a = vertex(i, j);
        let c = vertex(i + 1, j + 1);
        if kind == SurfKind::Surf {
            // نرمال وجه ≈ مرکز به بیرون (تقریب مسطح — ساده و پایدار)
            let cx = (a.0 + c.0) / 2.0;
            let cy = (a.1 + c.1) / 2.0;
            let cz = (a.2 + c.2) / 2.0;
            let len = (cx * cx + cy * cy + cz * cz).sqrt().max(1e-9);
            let n = (cx / len, cy / len, cz / len);
            let diff = (n.0 * light.0 + n.1 * light.1 + n.2 * light.2).max(0.0);
            let shade = (0.35 + 0.65 * diff) as f32;
            let zc = (z[i][j] + z[i + 1][j] + z[i][j + 1] + z[i + 1][j + 1]) / 4.0;
            let [r, g, b] = colormap::sample_z(cm, zc, cax);
            fill.push(((i, j), [r * shade, g * shade, b * shade]));
        }
        wire.push((i, j));
    }
    let fit = |sx: f64, sy: f64| -> (f64, f64) {
        // مقیاس‌بندی به فریم (پروجکشن در [0,1] نرمال نبود → fit با دامنه)
        (
            f.x0 + sx * f.w * 0.92 + f.w * 0.04,
            f.y0 + sy * f.h * 0.92 + f.h * 0.04,
        )
    };
    for ((i, j), rgb) in fill {
        let a = vertex(i, j);
        let b = vertex(i + 1, j);
        let c = vertex(i + 1, j + 1);
        let d = vertex(i, j + 1);
        ops.push(Op::PolyFill {
            r: rgb[0],
            g: rgb[1],
            b: rgb[2],
            pts: vec![fit(a.0, a.1), fit(b.0, b.1), fit(c.0, c.1), fit(d.0, d.1)],
        });
    }
    for (i, j) in wire {
        let a = vertex(i, j);
        let b = vertex(i + 1, j);
        let c = vertex(i + 1, j + 1);
        let d = vertex(i, j + 1);
        ops.push(Op::Stroke {
            r: 0.1,
            g: 0.1,
            b: 0.1,
            width: 0.4,
            dash: None,
            pts: vec![
                fit(a.0, a.1),
                fit(b.0, b.1),
                fit(c.0, c.1),
                fit(d.0, d.1),
                fit(a.0, a.1),
            ],
        });
    }
}

// ---------------------------------------------------------------------------
// contour — marching squares (همان قاعده در Dart)
// ---------------------------------------------------------------------------

/// پاره‌خط داده‌ای: (نقطهٔ آغاز، نقطهٔ پایان).
pub type Seg = ((f64, f64), (f64, f64));

/// پاره‌خط‌های تقاطع سطح L با یک خانهٔ دوبی‌خطی — خروجی به مختصات داده.
/// هستهٔ عددی با آرگومان‌های صریح (x/y/z گوشه‌ها) — نه struct واسط.
#[allow(clippy::too_many_arguments)]
pub fn marching_cell(
    x0: f64,
    x1: f64,
    y0: f64,
    y1: f64,
    z00: f64,
    z10: f64,
    z11: f64,
    z01: f64,
    level: f64,
    out: &mut Vec<Seg>,
) {
    let lerp = |a: f64, b: f64, za: f64, zb: f64| -> Option<f64> {
        if (za < level && zb < level) || (za > level && zb > level) {
            return None;
        }
        let t = if (zb - za).abs() < 1e-15 {
            0.5
        } else {
            (level - za) / (zb - za)
        };
        Some(a + t.clamp(0.0, 1.0) * (b - a))
    };
    let bottom = lerp(x0, x1, z00, z10).map(|x| (x, y0));
    let right = lerp(y0, y1, z10, z11).map(|y| (x1, y));
    let top = lerp(x1, x0, z11, z01).map(|x| (x, y1));
    let left = lerp(y0, y1, z00, z01).map(|y| (x0, y));
    let pts: Vec<(f64, f64)> = [bottom, right, top, left].into_iter().flatten().collect();
    // نقاط منطبق (برخورد سطح با گوشه) را یکی می‌کنیم
    let mut dedup: Vec<(f64, f64)> = Vec::with_capacity(4);
    for p in pts {
        if !dedup
            .iter()
            .any(|q| (q.0 - p.0).abs() < 1e-12 && (q.1 - p.1).abs() < 1e-12)
        {
            dedup.push(p);
        }
    }
    if dedup.len() == 2 {
        out.push((dedup[0], dedup[1]));
    }
}

#[allow(clippy::too_many_arguments)]
fn contour_ops(
    ops: &mut Vec<Op>,
    m: &Mapper,
    x: &[f64],
    y: &[f64],
    z: &[Vec<f64>],
    levels: &[f64],
    filled: bool,
    cm: Colormap,
    caxis: Option<[f64; 2]>,
) {
    let rows = z.len();
    let cols = z.first().map_or(0, Vec::len);
    if rows < 2 || cols < 2 {
        return;
    }
    let cax = caxis.unwrap_or_else(|| crate::surface::z_range(z).unwrap_or([0.0, 1.0]));
    if filled {
        // پُرکردن نوار-سلولی: رنگ هر خانه از مقدار مرکز + روکش
        // پاره‌خط‌های همسو (مرزها نرم دیده می‌شوند — تقریب ثبت‌شده).
        for i in 0..rows - 1 {
            for j in 0..cols - 1 {
                let zc = (z[i][j] + z[i][j + 1] + z[i + 1][j] + z[i + 1][j + 1]) / 4.0;
                let [r, g, b] = colormap::sample_z(cm, zc, cax);
                let a = m.map(x[j], y[i]);
                let c = m.map(x[j + 1], y[i + 1]);
                ops.push(Op::PolyFill {
                    r,
                    g,
                    b,
                    pts: vec![(a.0, a.1), (c.0, a.1), (c.0, c.1), (a.0, c.1)],
                });
            }
        }
    }
    // خطوط سطح — رنگ هر سطح از پالت
    for &l in levels {
        let t = ((l - cax[0]) / (cax[1] - cax[0]).max(1e-12)) as f32;
        let [r, g, b] = colormap::sample(cm, t);
        let mut segs = Vec::new();
        for i in 0..rows - 1 {
            for j in 0..cols - 1 {
                marching_cell(
                    x[j],
                    x[j + 1],
                    y[i],
                    y[i + 1],
                    z[i][j],
                    z[i][j + 1],
                    z[i + 1][j + 1],
                    z[i + 1][j],
                    l,
                    &mut segs,
                );
            }
        }
        for (p, q) in segs {
            line_series_ops(ops, m, &[p, q], [r, g, b], if filled { 0.6 } else { 1.2 });
        }
    }
}

// ---------------------------------------------------------------------------
// جمع‌بندی هر نوع سلول
// ---------------------------------------------------------------------------

fn content_for_cell(cell: &CellSnapshot, ops: &mut Vec<Op>) {
    content_in_frame(cell, base_frame(), ops);
}

fn content_in_frame(cell: &CellSnapshot, base: Frame, ops: &mut Vec<Op>) {
    match cell {
        CellSnapshot::Line { series, axes, .. } | CellSnapshot::Scatter { series, axes, .. } => {
            let fr = inset(
                &base,
                axes.ylabel.is_some(),
                axes.title.is_some(),
                axes.xlabel.is_some(),
            );
            let (xl, xh) = bounds_of(
                &series
                    .iter()
                    .flat_map(|s| s.x.iter().copied())
                    .collect::<Vec<_>>(),
                &series
                    .iter()
                    .flat_map(|s| s.y.iter().copied())
                    .collect::<Vec<_>>(),
            );
            let (xlo, xhi) = pad(xl.0, xl.1);
            let (ylo, yhi) = pad(xh.0, xh.1);
            let m = Mapper::new(fr, [xlo, ylo], [xhi, yhi]);
            push_axes_frame(ops, &fr);
            for (i, s) in series.iter().enumerate() {
                let pts: Vec<(f64, f64)> =
                    s.x.iter().zip(s.y.iter()).map(|(&a, &b)| (a, b)).collect();
                line_series_ops(ops, &m, &pts, palette_rgb(i), 1.4);
            }
            push_title_and_labels(ops, &axes.title, &axes.xlabel, &axes.ylabel);
        }
        CellSnapshot::Bar { series, axes, .. } => {
            let fr = inset(&base, axes.ylabel.is_some(), false, axes.xlabel.is_some());
            let xs: Vec<f64> = series.iter().flat_map(|s| s.x.iter().copied()).collect();
            let mut ys: Vec<f64> = series.iter().flat_map(|s| s.y.iter().copied()).collect();
            ys.push(0.0);
            let (xl, yl) = bounds_of(&xs, &ys);
            let (xlo, xhi) = pad(xl.0, xl.1);
            let (ylo, yhi) = pad(yl.0, yl.1);
            let m = Mapper::new(fr, [xlo, ylo], [xhi, yhi]);
            push_axes_frame(ops, &fr);
            let bw = (m.hi[0] - m.lo[0]) * 0.4 / series.len().max(1) as f64;
            for (i, s) in series.iter().enumerate() {
                let [r, g, b] = palette_rgb(i);
                for k in 0..s.x.len() {
                    let a = m.map(s.x[k] - bw, 0.0);
                    let c = m.map(s.x[k] + bw, s.y[k]);
                    if c.1 < a.1 {
                        continue;
                    }
                    ops.push(Op::PolyFill {
                        r,
                        g,
                        b,
                        pts: vec![(a.0, a.1), (c.0, a.1), (c.0, c.1), (a.0, c.1)],
                    });
                }
            }
            push_title_and_labels(ops, &axes.title, &axes.xlabel, &axes.ylabel);
        }
        CellSnapshot::Histogram {
            edges,
            counts,
            axes,
            ..
        } => {
            let fr = inset(&base, axes.ylabel.is_some(), false, axes.xlabel.is_some());
            let (xl, yl) = bounds_of(edges, counts);
            let m = Mapper::new(fr, [xl.0, 0.0], [xl.1, yl.1.max(1.0)]);
            push_axes_frame(ops, &fr);
            for k in 0..edges.len().saturating_sub(1) {
                let a = m.map(edges[k], 0.0);
                let c = m.map(edges[k + 1], counts[k]);
                if c.1 < a.1 {
                    continue;
                }
                ops.push(Op::PolyFill {
                    r: 0.30,
                    g: 0.55,
                    b: 0.85,
                    pts: vec![(a.0, a.1), (c.0, a.1), (c.0, c.1), (a.0, c.1)],
                });
            }
            push_title_and_labels(ops, &axes.title, &axes.xlabel, &axes.ylabel);
        }
        CellSnapshot::Area { series, axes, .. } | CellSnapshot::Fill { series, axes } => {
            let is_area = matches!(cell, CellSnapshot::Area { .. });
            let fr = inset(&base, axes.ylabel.is_some(), false, axes.xlabel.is_some());
            let xs: Vec<f64> = series.iter().flat_map(|s| s.x.iter().copied()).collect();
            let mut ys: Vec<f64> = series.iter().flat_map(|s| s.y.iter().copied()).collect();
            if is_area {
                ys.push(0.0);
            }
            let (xl, yl) = bounds_of(&xs, &ys);
            let (xlo, xhi) = pad(xl.0, xl.1);
            let (ylo, yhi) = pad(yl.0, yl.1);
            let m = Mapper::new(fr, [xlo, ylo], [xhi, yhi]);
            push_axes_frame(ops, &fr);
            for (i, s) in series.iter().enumerate() {
                let [r, g, b] = palette_rgb(i);
                let mut pts: Vec<(f64, f64)> =
                    s.x.iter().zip(s.y.iter()).map(|(&a, &b)| (a, b)).collect();
                if is_area && pts.len() >= 2 {
                    let (first, last) = (pts[0], pts[pts.len() - 1]);
                    pts.push((last.0, 0.0));
                    pts.push((first.0, 0.0));
                    let mapped: Vec<(f64, f64)> = pts.iter().map(|&(a, b)| m.map(a, b)).collect();
                    ops.push(Op::PolyFill {
                        r,
                        g,
                        b,
                        pts: mapped,
                    });
                } else if pts.len() >= 2 {
                    let mut p = pts.clone();
                    let first = p[0];
                    p.push(first);
                    let mapped: Vec<(f64, f64)> = p.iter().map(|&(a, b)| m.map(a, b)).collect();
                    ops.push(Op::PolyFill {
                        r,
                        g,
                        b,
                        pts: mapped,
                    });
                }
            }
            push_title_and_labels(ops, &axes.title, &axes.xlabel, &axes.ylabel);
        }
        CellSnapshot::Polar { series, .. } => {
            // شبکهٔ قطبی ساده + سری‌ها؛ فریم مربعی وسط صفحه
            let side = f64::min(PAGE_H - 2.0 * MARGIN, PAGE_W - 2.0 * MARGIN) * 0.8;
            let fr = Frame {
                x0: (PAGE_W - side) / 2.0,
                y0: (PAGE_H - side) / 2.0,
                w: side,
                h: side,
            };
            let rmax = series
                .iter()
                .flat_map(|s| s.y.iter().copied())
                .filter(|v| v.is_finite())
                .fold(0.0_f64, f64::max)
                .max(1e-9);
            let cx = fr.x0 + fr.w / 2.0;
            let cy = fr.y0 + fr.h / 2.0;
            for k in 1..=3 {
                let rr = fr.w / 2.0 * k as f64 / 3.0;
                let circle: Vec<(f64, f64)> = (0..=64)
                    .map(|t| {
                        let a = t as f64 / 64.0 * std::f64::consts::TAU;
                        (cx + rr * a.cos(), cy + rr * a.sin())
                    })
                    .collect();
                ops.push(Op::Stroke {
                    r: 0.8,
                    g: 0.8,
                    b: 0.8,
                    width: 0.5,
                    dash: Some((2.0, 2.0)),
                    pts: circle,
                });
            }
            for (i, s) in series.iter().enumerate() {
                let pts: Vec<(f64, f64)> =
                    s.x.iter()
                        .zip(s.y.iter())
                        .map(|(&th, &r)| {
                            (
                                cx + (r / rmax) * fr.w / 2.0 * th.cos(),
                                cy + (r / rmax) * fr.h / 2.0 * th.sin(),
                            )
                        })
                        .collect();
                if pts.len() >= 2 {
                    ops.push(Op::Stroke {
                        r: palette_rgb(i)[0],
                        g: palette_rgb(i)[1],
                        b: palette_rgb(i)[2],
                        width: 1.4,
                        dash: None,
                        pts,
                    });
                }
            }
        }
        CellSnapshot::ErrorBar { series, axes, .. } => {
            let fr = inset(&base, axes.ylabel.is_some(), false, axes.xlabel.is_some());
            let mut xs: Vec<f64> = Vec::new();
            let mut ys: Vec<f64> = Vec::new();
            for s in series {
                xs.extend(s.x.iter().copied());
                for k in 0..s.x.len() {
                    ys.push(s.y[k]);
                    if let Some(e) = &s.y_err {
                        if k < e.neg.len() {
                            ys.push(s.y[k] - e.neg[k]);
                        }
                        if k < e.pos.len() {
                            ys.push(s.y[k] + e.pos[k]);
                        }
                    }
                }
            }
            let (xl, yl) = bounds_of(&xs, &ys);
            let (xlo, xhi) = pad(xl.0, xl.1);
            let (ylo, yhi) = pad(yl.0, yl.1);
            let m = Mapper::new(fr, [xlo, ylo], [xhi, yhi]);
            push_axes_frame(ops, &fr);
            for (i, s) in series.iter().enumerate() {
                let c = palette_rgb(i);
                let pts: Vec<(f64, f64)> =
                    s.x.iter().zip(s.y.iter()).map(|(&a, &b)| (a, b)).collect();
                line_series_ops(ops, &m, &pts, c, 1.2);
                for k in 0..s.x.len() {
                    if let Some(e) = &s.y_err {
                        let lo = e.neg.get(k).copied().unwrap_or(0.0);
                        let hi = e.pos.get(k).copied().unwrap_or(0.0);
                        if lo > 0.0 || hi > 0.0 {
                            let a = m.map(s.x[k], s.y[k] - lo);
                            let b = m.map(s.x[k], s.y[k] + hi);
                            ops.push(Op::Stroke {
                                r: c[0],
                                g: c[1],
                                b: c[2],
                                width: 0.8,
                                dash: None,
                                pts: vec![a, b],
                            });
                        }
                    }
                }
            }
            push_title_and_labels(ops, &axes.title, &axes.xlabel, &axes.ylabel);
        }
        CellSnapshot::Plot3 { series, axes, .. } => {
            let fr = inset(&base, false, axes.title.is_some(), false);
            let mut xs: Vec<f64> = series.iter().flat_map(|s| s.x.iter().copied()).collect();
            let mut ys: Vec<f64> = series.iter().flat_map(|s| s.y.iter().copied()).collect();
            let zs: Vec<f64> = series.iter().flat_map(|s| s.z.iter().copied()).collect();
            xs.extend(zs.iter().copied());
            ys.extend(zs.iter().copied());
            let (xl, _) = bounds_of(&xs, &ys);
            let span = (xl.1 - xl.0).max(1e-9);
            let norm = |v: f64| ((v - xl.0) / span).clamp(0.0, 1.0);
            for (i, s) in series.iter().enumerate() {
                let c = palette_rgb(i);
                let mapped: Vec<(f64, f64)> =
                    s.x.iter()
                        .zip(s.y.iter().zip(s.z.iter()))
                        .map(|(&a, (&b, &cz))| {
                            let (sx, sy, _) = project(axes.view, norm(a), norm(b), norm(cz));
                            (fr.x0 + sx * fr.w, fr.y0 + sy * fr.h)
                        })
                        .collect();
                if mapped.len() >= 2 {
                    ops.push(Op::Stroke {
                        r: c[0],
                        g: c[1],
                        b: c[2],
                        width: 1.4,
                        dash: None,
                        pts: mapped,
                    });
                }
            }
            push_title_and_labels(ops, &axes.title, &axes.xlabel, &axes.zlabel);
        }
        CellSnapshot::Surface {
            x,
            y,
            z,
            kind,
            colormap,
            caxis,
            colorbar: cb,
            axes,
            ..
        } => {
            let fr = inset(&base, false, axes.title.is_some(), false);
            surface_ops(ops, x, y, z, *kind, *colormap, *caxis, axes.view, fr);
            if *cb {
                let cax = caxis.unwrap_or([0.0, 1.0]);
                colorbar(ops, *colormap, cax, &fr);
            }
            push_title_and_labels(ops, &axes.title, &axes.xlabel, &axes.zlabel);
        }
        CellSnapshot::Contour {
            x,
            y,
            z,
            levels,
            filled,
            colormap,
            caxis,
            colorbar: cb,
            axes,
            ..
        } => {
            let fr = inset(&base, axes.ylabel.is_some(), false, axes.xlabel.is_some());
            let (xl, yl) = bounds_of(x, y);
            let m = Mapper::new(fr, [xl.0, yl.0], [xl.1, yl.1]);
            push_axes_frame(ops, &fr);
            contour_ops(ops, &m, x, y, z, levels, *filled, *colormap, *caxis);
            if *cb {
                let cax = caxis.unwrap_or([0.0, 1.0]);
                colorbar(ops, *colormap, cax, &fr);
            }
            push_title_and_labels(ops, &axes.title, &axes.xlabel, &axes.ylabel);
        }
        CellSnapshot::Subplot { rows, cols, cells } => {
            // شبکهٔ زیرنمودارها — هر سلول بازگشتی در زیرقاب خودش
            let (rows, cols) = ((*rows as f64).max(1.0), (*cols as f64).max(1.0));
            let gap = 10.0;
            let cw = (base.w - gap * (cols - 1.0)) / cols;
            let ch = (base.h - gap * (rows - 1.0)) / rows;
            for c in cells {
                let row = c.row as f64;
                let col = c.col as f64;
                let sub = Frame {
                    x0: base.x0 + col * (cw + gap),
                    y0: base.y0 + (rows - 1.0 - row) * (ch + gap),
                    w: cw,
                    h: ch,
                };
                content_in_frame(&c.spec, sub, ops);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// سریال‌سازی PDF
// ---------------------------------------------------------------------------

fn sanitize_text(s: &str) -> String {
    // WinAnsi: لاتین۱ چاپی می‌ماند؛ بقیهٔ یونیکد با «?» صادقانه جایگزین
    // می‌شود (محدودیت فونت پایه — سند ۱۳).
    s.chars()
        .map(|c| {
            if (c as u32) < 256 && !c.is_control() {
                c
            } else {
                '?'
            }
        })
        .collect()
}

fn esc(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('(', "\\(")
        .replace(')', "\\)")
}

fn num(v: f64) -> String {
    let v = if v.is_finite() { v } else { 0.0 };
    if (v - v.round()).abs() < 1e-6 {
        format!("{}", v.round() as i64)
    } else {
        format!("{v:.3}")
    }
}

fn stream_for(ops: &[Op]) -> String {
    let mut s = String::new();
    for op in ops {
        match op {
            Op::Stroke {
                r,
                g,
                b,
                width,
                dash,
                pts,
            } => {
                s.push_str(&format!(
                    "{} {} {} RG {} w ",
                    num(*r as f64),
                    num(*g as f64),
                    num(*b as f64),
                    num(*width as f64)
                ));
                if let Some((a, b)) = dash {
                    s.push_str(&format!("[{} {}] 0 d ", num(*a as f64), num(*b as f64)));
                }
                if let (Some(first), rest) = (pts.first(), &pts[1..]) {
                    s.push_str(&format!("{} {} m ", num(first.0), num(first.1)));
                    for p in rest {
                        s.push_str(&format!("{} {} l ", num(p.0), num(p.1)));
                    }
                    s.push_str("S\n");
                }
            }
            Op::PolyFill { r, g, b, pts } => {
                if let Some(first) = pts.first() {
                    s.push_str(&format!(
                        "{} {} {} rg {} {} m ",
                        num(*r as f64),
                        num(*g as f64),
                        num(*b as f64),
                        num(first.0),
                        num(first.1)
                    ));
                    for p in &pts[1..] {
                        s.push_str(&format!("{} {} l ", num(p.0), num(p.1)));
                    }
                    s.push_str("f\n");
                }
            }
            Op::Text {
                x,
                y,
                size,
                bold,
                text,
                rotate,
            } => {
                let f = if *bold { "/F2" } else { "/F1" };
                let t = esc(&sanitize_text(text));
                if *rotate {
                    s.push_str(&format!(
                        "BT {} {} Tf 0 1 -1 0 {} {} Tm ({}) Tj ET\n",
                        f,
                        num(*size as f64),
                        num(*x),
                        num(*y),
                        t
                    ));
                } else {
                    s.push_str(&format!(
                        "BT {} {} Tf {} {} Td ({}) Tj ET\n",
                        f,
                        num(*size as f64),
                        num(*x),
                        num(*y),
                        t
                    ));
                }
            }
        }
    }
    s
}

fn write_pdf(pages: Vec<Vec<Op>>) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::new();
    let mut offsets: Vec<usize> = Vec::new();
    out.extend_from_slice(b"%PDF-1.4\n%\xE2\xE3\xCF\xD3\n");
    let n = pages.len();
    // هر صفحه: محتوا + آبجکت صفحه
    let mut page_ids = Vec::new();
    for (k, ops) in pages.iter().enumerate() {
        let content_id = 3 + 2 * k;
        let page_id = 4 + 2 * k;
        page_ids.push(page_id as u32);
        offsets.push(out.len());
        let stream = stream_for(ops);
        out.extend_from_slice(
            format!(
                "{} 0 obj\n<< /Length {} >>\nstream\n{}\nendstream\nendobj\n",
                content_id,
                stream.len(),
                stream
            )
            .as_bytes(),
        );
        offsets.push(out.len());
        out.extend_from_slice(
            format!(
                "{} 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {} {}] \
                 /Resources << /Font << /F1 {} 0 R /F2 {} 0 R >> >> /Contents {} 0 R >>\nendobj\n",
                page_id,
                num(PAGE_W),
                num(PAGE_H),
                3 + 2 * n,
                4 + 2 * n,
                content_id
            )
            .as_bytes(),
        );
    }
    let kids: Vec<String> = page_ids.iter().map(|id| format!("{id} 0 R")).collect();
    offsets.push(out.len());
    out.extend_from_slice("1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n".as_bytes());
    offsets.push(out.len());
    out.extend_from_slice(
        format!(
            "2 0 obj\n<< /Type /Pages /Kids [{}] /Count {} >>\nendobj\n",
            kids.join(" "),
            n
        )
        .as_bytes(),
    );
    // فونت‌ها بعد از جفت‌های صفحه: شناسه‌ها 3+2n و 4+2n
    let f1 = 3 + 2 * n;
    let f2 = 4 + 2 * n;
    offsets.push(out.len());
    out.extend_from_slice(
        format!("{f1} 0 obj\n<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>\nendobj\n").as_bytes(),
    );
    offsets.push(out.len());
    out.extend_from_slice(
        format!("{f2} 0 obj\n<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica-Bold /Encoding /WinAnsiEncoding >>\nendobj\n").as_bytes(),
    );
    let xref = out.len();
    let count = 5 + 2 * n;
    out.extend_from_slice(format!("xref\n0 {count}\n").as_bytes());
    out.extend_from_slice(b"0000000000 65535 f \n");
    for off in &offsets {
        out.extend_from_slice(format!("{off:010} 00000 n \n").as_bytes());
    }
    out.extend_from_slice(
        format!("trailer\n<< /Size {count} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n").as_bytes(),
    );
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::figure::FigureSnapshot;
    use crate::spec::{Colormap, PlotOp, SeriesData, StyleSpec, SurfKind, View3};

    fn simple_line_figure() -> FigureSnapshot {
        let mut st = crate::figure::FigureState::new();
        st.apply(PlotOp::Line {
            series: vec![crate::spec::SeriesPair {
                data: SeriesData {
                    x: vec![0.0, 1.0, 2.0, 3.0],
                    y: vec![0.0, 1.0, 0.0, 1.0],
                },
                style: StyleSpec::default(),
            }],
        });
        st.apply(PlotOp::Title {
            text: "demo".into(),
        });
        st.snapshots().remove(0)
    }

    fn surf_figure() -> FigureSnapshot {
        let mut st = crate::figure::FigureState::new();
        st.apply(PlotOp::Surface {
            x: vec![0.0, 1.0, 2.0],
            y: vec![0.0, 1.0, 2.0],
            z: vec![
                vec![0.0, 1.0, 2.0],
                vec![1.0, 2.0, 3.0],
                vec![2.0, 3.0, 4.0],
            ],
            kind: SurfKind::Surf,
            shading: crate::spec::Shading3::Flat,
            colormap: Colormap::Parula,
            caxis: None,
            colorbar: true,
        });
        st.snapshots().remove(0)
    }

    #[test]
    fn pdf_has_valid_skeleton() {
        let pdf = render_figure(&simple_line_figure());
        assert!(pdf.starts_with(b"%PDF-1.4"), "هدر");
        let tail = String::from_utf8_lossy(&pdf[pdf.len() - 32..]);
        assert!(tail.contains("%%EOF"), "پایان: {tail}");
        assert!(pdf.windows(4).any(|w| w == b"xref"));
        assert!(pdf.windows(11).any(|w| w == b"/Type /Page"));
        // متن عنوان با فونت bold رفته
        assert!(pdf.windows(9).any(|w| w == b"/F2 13 Tf"));
        assert!(String::from_utf8_lossy(&pdf).contains("(demo)"));
    }

    #[test]
    fn line_plot_has_stroke_ops() {
        let pdf = render_figure(&simple_line_figure());
        let s = String::from_utf8_lossy(&pdf);
        assert!(s.contains(" RG "), "رنگ خط");
        assert!(s.contains(" m\n") || s.contains(" m "), "شروع مسیر");
        assert!(s.contains("S\n"), "کشیدن");
    }

    #[test]
    fn surf_page_has_fills_and_colorbar() {
        let pdf = render_figure(&surf_figure());
        let s = String::from_utf8_lossy(&pdf);
        assert!(s.contains(" rg "), "پُر کردن وجه‌ها");
        // نوار رنگ: ۶۴ نوار
        assert!(s.matches(" rg ").count() >= 64, "نوار رنگ + وجه‌ها");
    }

    #[test]
    fn multi_figure_pdf_has_two_pages() {
        let f1 = simple_line_figure();
        let f2 = surf_figure();
        let pdf = render_figures(&[f1, f2]);
        let s = String::from_utf8_lossy(&pdf);
        assert!(s.contains("/Count 2"), "دو صفحه");
    }

    #[test]
    fn non_latin_text_is_replaced_honestly() {
        assert_eq!(sanitize_text("سلام"), "????");
        assert_eq!(sanitize_text("ok 1!"), "ok 1!");
    }

    #[test]
    fn numbers_never_contain_nan_or_exp() {
        assert_eq!(num(f64::NAN), "0");
        assert_eq!(num(12.345678), "12.346");
        assert_eq!(num(5.0), "5");
        assert!(!num(1e20).contains('e'));
    }

    #[test]
    fn marching_cell_finds_midline() {
        let mut out = Vec::new();
        // صفحهٔ شیب‌دار z = x + y؛ سطح 1 از وسط خانه رد می‌شود
        marching_cell(0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 2.0, 1.0, 1.0, &mut out);
        assert_eq!(out.len(), 1, "یک پاره‌خط");
        let ((ax, ay), (bx, by)) = out[0];
        assert!(
            (ax - 1.0).abs() < 1e-9 && ay == 0.0,
            "نقطهٔ روی لبهٔ پایین: {ax},{ay}"
        );
        assert!(
            (bx - 0.0).abs() < 1e-9 && (by - 1.0).abs() < 1e-9,
            "نقطهٔ روی لبهٔ چپ: {bx},{by}"
        );
    }

    #[test]
    fn projection_view2_is_top_down() {
        // view(2) → نگاه از بالا: x داده → راست صفحه، y داده → بالای صفحه
        let (sx, sy, _) = project(View3::TOP, 0.3, 0.7, 0.9);
        assert!(
            (sx - 0.3).abs() < 1e-9 && (sy - 0.7).abs() < 1e-9,
            "({sx},{sy})"
        );
        // نمای پیش‌فرض: y داده باید به «بالای» صفحه بیفتد (زاویهٔ el)
        let (_, sy, _) = project(View3 { az: 0.0, el: 30.0 }, 0.0, 1.0, 0.0);
        assert!(sy > 0.4, "sin(30°)=0.5: {sy}");
    }
}
