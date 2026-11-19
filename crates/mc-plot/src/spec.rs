//! گرام عملیات رسم (PlotOp) — زبان میانیِ ساطع‌شده از مفسر و جمع‌شده در
//! [`crate::figure::FigureState`].
//!
//! **چرا عملیات و نه جهش مستقیم:** مفسر خالص می‌ماند (بدون حالت پنهان)؛
//! دنبالهٔ عملیات قابل بازپخش و تست واحد است و CLI می‌تواند آن را نادیده
//! بگیرد یا متنی‌سازی کند. این گرام **داخلی** است و نسخه‌بندی نمی‌خواهد —
//! نسخهٔ قرارداد فقط روی DTO انتقالی (mc-ffi) است.

pub use crate::histogram::HistNorm;
use crate::ticks::Tick;

/// یک سری داده: دامنه (x) و مقدار (y) — معنا بسته به نوع عملیات
/// (پلار: x = θ رادیان، y = شعاع؛ bar: x = مراکز میله).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SeriesData {
    pub x: Vec<f64>,
    pub y: Vec<f64>,
}

/// رنگ در گرام موتور: توکن معنایی پالت (resolve در Dart طبق تم) یا RGB دقیق.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlotColorSpec {
    /// 0..=7 — سیکل پالت سند ۰۶ §3.
    Palette(u8),
    /// رنگ صریح (خط‌اسپک متلب `'r'` یا `[r g b]`).
    Rgb { r: u8, g: u8, b: u8 },
}

/// سبک خط — `Invisible` یعنی فقط مارکر (پارتی `'b*'` متلب).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DashSpec {
    Solid,
    Dashed,
    Dotted,
    DashDot,
    Invisible,
}

/// مارکر — جدول متلب (زیرمجموعهٔ ۹تایی DTO).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarkerSpec {
    Point,
    Circle,
    Square,
    Diamond,
    Cross,
    Plus,
    Star,
    TriangleUp,
    TriangleDown,
}

/// استایل با اجزای اختیاری — None یعنی «پیش‌فرض موتور» (در کاهش‌دهنده
/// resolve می‌شود: رنگ = سیکل پالت، خط = صاف یا نامرئی بسته به نوع، …).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct StyleSpec {
    pub color: Option<PlotColorSpec>,
    pub width: Option<f32>,
    pub dash: Option<DashSpec>,
    pub marker: Option<MarkerSpec>,
    pub label: Option<String>,
    /// اندازهٔ مارکر (pt مفهومی؛ پیش‌فرض موتور در رندرر) — v3.
    pub marker_size: Option<f32>,
}

/// نوار خطا — قرینه یا ناقرینه (مقادیر ≥ 0؛ خالی = بدون خطای آن محور).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ErrBand {
    pub neg: Vec<f64>,
    pub pos: Vec<f64>,
}

/// یک سری در یک فراخوانی: داده + استایل همان سری (رنگِ غایب = سیکل پالت).
#[derive(Debug, Clone, PartialEq)]
pub struct SeriesPair {
    pub data: SeriesData,
    pub style: StyleSpec,
}

/// سری errorbar در یک فراخوانی.
#[derive(Debug, Clone, PartialEq)]
pub struct ErrorPair {
    pub data: SeriesData,
    pub x_err: Option<ErrBand>,
    pub y_err: Option<ErrBand>,
    pub style: StyleSpec,
}

/// یک چندضلعی fill با استایل خودش — پارتی متلب: `fill(x1,y1,c1,x2,y2,c2)`
/// هر چندضلعی رنگ خودش را می‌گیرد (سیکل پالت وقتی رنگ غایب است).
#[derive(Debug, Clone, PartialEq)]
pub struct FillPair {
    pub data: SeriesData,
    pub style: StyleSpec,
}

/// موقعیت legend (نگاشت مکانیکی به `LegendPosition` قرارداد در mc-ffi).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LegendPos {
    Best,
    North,
    South,
    East,
    West,
    NorthEast,
    NorthWest,
    SouthEast,
    SouthWest,
}

/// یک سری سه‌بعدی `plot3(x, y, z)` — هم‌طول، ترتیب نقاط معتبر منحنی است.
#[derive(Debug, Clone, PartialEq)]
pub struct Series3Pair {
    pub data: SeriesData,
    pub z: Vec<f64>,
    pub style: StyleSpec,
}

/// نوع سطح — surf (وجه‌های رنگی) یا mesh (سیم‌باف رنگ از z، بی‌وجه).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SurfKind {
    #[default]
    Surf,
    Mesh,
}

/// وضعیت دوربین سه‌بعدی — view(az, el) بر حسب درجه (متلب: پیش‌فرض
/// az=-37.5، el=30؛ view(2) → (0, 90)).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct View3 {
    pub az: f64,
    pub el: f64,
}

impl View3 {
    /// پیش‌فرض متلب.
    pub const DEFAULT: View3 = View3 {
        az: -37.5,
        el: 30.0,
    };
    /// نمای دو‌بعدی `view(2)`.
    pub const TOP: View3 = View3 { az: 0.0, el: 90.0 };
}

/// یک عملیات رسم — کاهش‌پذیر روی [`crate::figure::FigureState`].
/// هر عملیات داده‌ای، **همهٔ سری‌های همان فراخوانی** را یکجا حمل می‌کند
/// (مثلاً `plot(X, Y_matrix)` = یک Line با چند سری) تا تصمیم جایگزینی/الحاق
/// (hold) فقط یک‌بار برای کل فراخوانی گرفته شود.
#[derive(Debug, Clone, PartialEq)]
pub enum PlotOp {
    /// `figure(n)` / `figure` — فعال‌سازی یا ایجاد شکل (None = شناسهٔ تازه).
    ActivateFigure {
        id: Option<u64>,
    },
    /// `hold on/off` — پرچم محور فعال.
    Hold {
        on: bool,
    },
    /// `subplot(m,n,p)` — شبکهٔ سلولی شکل فعال (p یک‌مبنا و خطی؛
    /// اعتبارسنجی دامنه در لایهٔ builtin، کاهش‌دهنده دفاعاً clamps می‌کند).
    SubplotSelect {
        rows: u32,
        cols: u32,
        index: u32,
    },
    /// `plot` / `fplot` — سری‌های خطی.
    Line {
        series: Vec<SeriesPair>,
    },
    /// `scatter` — سری‌های نقطه‌ای.
    Scatter {
        series: Vec<SeriesPair>,
    },
    /// `bar` — میله‌ای (چند سری = گروهی یا انباشته).
    Bar {
        series: Vec<SeriesPair>,
        width: Option<f32>,
        horizontal: bool,
        stacked: bool,
    },
    /// `histogram` — بسته‌بندی از پیش محاسبه‌شده (همیشه جایگزین).
    Histogram {
        edges: Vec<f64>,
        counts: Vec<f64>,
        norm: HistNorm,
        style: StyleSpec,
    },
    /// `area` — سطح زیر خط با خط مبنا.
    Area {
        series: Vec<SeriesPair>,
        baseline: f64,
    },
    /// `fill` — چندضلعی‌های بستهٔ یک فراخوانی، هر یک با استایل خودش
    /// (رنگ وجه؛ رنگ غایب = سیکل پالت).
    Fill {
        polys: Vec<FillPair>,
    },
    /// `polarplot` — x = θ (رادیان)، y = شعاع.
    Polar {
        series: Vec<SeriesPair>,
    },
    /// `errorbar` — سری‌های با نوار خطا.
    ErrorBar {
        series: Vec<ErrorPair>,
    },
    /// `legend('a','b',...)` / `legend off` — None = خاموش.
    Legend {
        entries: Option<Vec<String>>,
        /// `legend(..., 'Location', 'SouthWest')` — پیش‌فرض Best.
        position: LegendPos,
    },
    /// `title` / `xlabel` / `ylabel`.
    Title {
        text: String,
    },
    XLabel {
        text: String,
    },
    YLabel {
        text: String,
    },
    /// `grid on/off`.
    Grid {
        on: bool,
    },
    /// `grid` برهنه — معکوس وضعیت شبکه (پارتی متلب).
    GridToggle,
    /// `axis([a b c d])` — هر دو محور.
    AxisLimits {
        min_x: f64,
        max_x: f64,
        min_y: f64,
        max_y: f64,
    },
    /// `xlim([a b])` / `xlim('auto')` — None = بازگشت به خودکار.
    XLim {
        limits: Option<[f64; 2]>,
    },
    /// `ylim([a b])` / `ylim('auto')`.
    YLim {
        limits: Option<[f64; 2]>,
    },
    /// `semilogx/semilogy/loglog` — پرچم محور لگاریتمی.
    SetLog {
        x: bool,
        y: bool,
    },
    /// `hold` بدون آرگومان — معکوس پرچم سلول فعال (پارتی متلب).
    HoldToggle,
    /// `xticks(v)` / `xticks(v, 'a', 'b', ...)` — تیک‌های سفارشی x؛
    /// خالی = پاک‌شدن به خودکار.
    XTicks {
        ticks: Vec<Tick>,
    },
    /// `yticks(...)` — مشابه XTicks.
    YTicks {
        ticks: Vec<Tick>,
    },
    /// `axis equal`.
    AxisEqual {
        on: bool,
    },
    /// `axis on/off`.
    AxisVisible {
        on: bool,
    },
    /// `axis tight` — چسبیدن حدود به داده.
    AxisTight,
    // ---- P3-b (گام 3.8): سه‌بعدی ----
    /// `plot3(x, y, z)` — منحنی فضایی.
    Plot3 {
        series: Vec<Series3Pair>,
    },
    /// `surf(Z)` / `surf(X, Y, Z)` / `mesh(...)` — شبکهٔ منظم z ردیف×ستون
    /// (سطر i = y ثابت، ستون j = x ثابت — قرارداد meshgrid متلب).
    Surface {
        x: Vec<f64>,
        y: Vec<f64>,
        z: Vec<Vec<f64>>,
        kind: SurfKind,
        shading: Shading3,
        colormap: Colormap,
        caxis: Option<[f64; 2]>,
        colorbar: bool,
    },
    /// `contour(Z, n)` / `contourf(...)` — سطح‌های هم‌ارزش روی شبکه؛
    /// levels = None → ده سطح خودکار (marching squares در رندرر).
    Contour {
        x: Vec<f64>,
        y: Vec<f64>,
        z: Vec<Vec<f64>>,
        levels: Option<Vec<f64>>,
        filled: bool,
        colormap: Colormap,
        caxis: Option<[f64; 2]>,
        colorbar: bool,
    },
    /// `colorbar on/off` (یا برهنه = روشن).
    Colorbar {
        on: bool,
    },
    /// `caxis([cmin cmax])` — None = خودکار از داده.
    Caxis {
        limits: Option<[f64; 2]>,
    },
    /// `view(az, el)` / `view(2)` / `view(3)`.
    View {
        v: View3,
    },
    /// `zlabel(...)` (سه‌بعدی).
    ZLabel {
        text: String,
    },
    /// `colormap jet` / `colormap('jet')` — پالت سطح/کانتور سلول فعال.
    ColormapSet {
        cm: Colormap,
    },
    /// `shading flat|faceted|interp` — سایه‌زنی سطح فعال.
    ShadingSet {
        s: Shading3,
    },
    /// `exportgraphics('name.pdf')` — رندر PDF شکلِ فعال در محل مصرف
    /// (لایهٔ موتور، وسط fold؛ در خود کاهش‌دهنده no-op است).
    ExportPdf {
        name: String,
    },
}

/// سایه‌زنی سطح (نگاشت مستقیم به `Shading` قرارداد).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Shading3 {
    /// وجه‌بسته + لبهٔ مش (پیش‌فرض متلب surf).
    #[default]
    Faceted,
    /// سطح صاف (سایهٔ یک‌نواخت هر مثلث).
    Flat,
    /// درون‌یابی گورو (v3: نرمال‌های رأس از موتور می‌آیند).
    Interp,
}

/// پالت رنگ سطح (سند ۰۴ §8.3 — parula بازسازی‌شده + پنج پالت دیگر).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Colormap {
    #[default]
    Parula,
    Viridis,
    Jet,
    Hot,
    Cool,
    Gray,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn styles_default_to_engine_resolution() {
        let s = StyleSpec::default();
        assert_eq!(s.color, None);
        assert_eq!(s.dash, None);
        assert_eq!(s.marker, None);
        assert_eq!(s.label, None);
    }

    #[test]
    fn series_data_is_plain_pairs() {
        let d = SeriesData {
            x: vec![1.0, 2.0],
            y: vec![3.0, 4.0],
        };
        assert_eq!(d.x.len(), d.y.len());
    }
}
