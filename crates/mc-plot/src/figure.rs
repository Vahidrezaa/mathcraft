//! کاهش‌دهندهٔ حالت شکل‌ها — fold مرتب روی دنبالهٔ [`PlotOp`].
//!
//! سمانتیک MATLAB-lite (مستند در COMPATIBILITY):
//! - **یک شکل فعال** + سلول‌های subplot؛ `figure(n)` ایجاد/فعال‌سازی،
//!   `figure` = شناسهٔ تازه. مدیریت کامل شکل‌ها (پنجره‌ها) = نقطهٔ توسعه.
//! - **hold per-axes** (per-cell): خاموش → گزارهٔ داده‌ای بعدی محتوای سلول
//!   را جایگزین می‌کند و سیکل رنگ از صفر؛ روشن → سری اضافه می‌شود
//!   (plot+scatter زیر hold در یک سری‌لست Line متحد می‌شوند — هر سری
//!   استایل خودش را دارد؛ «فقط مارکر» = خط نامرئی).
//! - **fresh-axes روی جایگزینی** (گام 3.5.2): گزارهٔ داده‌ای بدون hold،
//!   حاشیه‌نویسی سلول (legend/title/label/حدود/تیک سفارشی/equal/tight/
//!   log) را هم پاک می‌کند — پارتی متلب؛ grid/visible/hold می‌مانند.
//! - **یک نوع نمودار در هر سلول**: mix نوع‌های غیرخطی زیر hold جایگزین
//!   می‌شود (واگرایی مستند از متلب؛ مسیر افزودنی آینده: `Overlay`).
//! - حدود خودکار از داده + ۵٪ حاشیه؛ `axis tight` حاشیه را حذف،
//!   `axis([a b c d])` حدود دستی؛ تیک‌های 1-2-5 در لحظهٔ snapshot.
//! - دادهٔ خام هر سری در Rust می‌ماند ([`BuiltSeries::raw_x`]) و فقط
//!   نمایشِ LTTB-کاسته انتقال می‌یابد (سند ۰۲ §7).

use crate::histogram::HistNorm;
use crate::lttb;
use crate::palette;
use crate::spec::{
    DashSpec, ErrBand, LegendPos, MarkerSpec, PlotColorSpec, PlotOp, SeriesPair, StyleSpec,
};
use crate::ticks::Tick;
use std::collections::BTreeMap;

/// عرض خط پیش‌فرض (سند ۰۶ §3 — ۲dp).
pub const DEFAULT_WIDTH: f32 = 2.0;
/// حاشیهٔ حدود خودکار (کسری از دامنه، هر طرف).
const AUTO_PAD: f64 = 0.05;

// ---------------------------------------------------------------------------
// Snapshot — قرینهٔ ۱:۱ از DTO انتقالی (نگاشت مکانیکی در mc-ffi)
// ---------------------------------------------------------------------------

/// یک شکل در لحظهٔ انتقال — شبکهٔ ۱×۱ مستقیم می‌آید، بزرگ‌تر داخل Subplot.
#[derive(Debug, Clone, PartialEq)]
pub struct FigureSnapshot {
    pub figure_id: u64,
    pub spec: CellSnapshot,
}

/// محتوای یک سلول + محورهایش (نگاشت به `PlotSpec` در mc-ffi).
#[derive(Debug, Clone, PartialEq)]
pub enum CellSnapshot {
    Line {
        series: Vec<SeriesSnapshot>,
        axes: AxesSnapshot,
        legend: Option<LegendSnapshot>,
    },
    Scatter {
        series: Vec<SeriesSnapshot>,
        axes: AxesSnapshot,
        legend: Option<LegendSnapshot>,
    },
    Bar {
        series: Vec<SeriesSnapshot>,
        width: f32,
        horizontal: bool,
        stacked: bool,
        axes: AxesSnapshot,
        legend: Option<LegendSnapshot>,
    },
    Histogram {
        edges: Vec<f64>,
        counts: Vec<f64>,
        norm: HistNorm,
        style: StyleSnapshot,
        axes: AxesSnapshot,
        legend: Option<LegendSnapshot>,
    },
    Area {
        series: Vec<SeriesSnapshot>,
        baseline: f64,
        axes: AxesSnapshot,
        legend: Option<LegendSnapshot>,
    },
    Fill {
        series: Vec<SeriesSnapshot>,
        axes: AxesSnapshot,
    },
    Polar {
        series: Vec<SeriesSnapshot>,
        legend: Option<LegendSnapshot>,
    },
    ErrorBar {
        series: Vec<ErrSeriesSnapshot>,
        axes: AxesSnapshot,
        legend: Option<LegendSnapshot>,
    },
    Subplot {
        rows: u32,
        cols: u32,
        cells: Vec<SubplotCellSnapshot>,
    },
}

/// سلول شبکه — site (ردیف/ستون صفرمبنا) + محتوا.
#[derive(Debug, Clone, PartialEq)]
pub struct SubplotCellSnapshot {
    pub row: u32,
    pub col: u32,
    pub spec: CellSnapshot,
}

/// سری نمایش‌داده‌شده — LTTB-کاسته؛ `raw_len` = تعداد نقاط دادهٔ خام.
#[derive(Debug, Clone, PartialEq)]
pub struct SeriesSnapshot {
    pub id: u64,
    pub raw_len: u64,
    pub x: Vec<f64>,
    pub y: Vec<f64>,
    pub style: StyleSnapshot,
}

/// سری errorbar — نوارهای قرینه/ناقرینه.
#[derive(Debug, Clone, PartialEq)]
pub struct ErrSeriesSnapshot {
    pub id: u64,
    pub raw_len: u64,
    pub x: Vec<f64>,
    pub y: Vec<f64>,
    pub x_err: Option<ErrBand>,
    pub y_err: Option<ErrBand>,
    pub style: StyleSnapshot,
}

/// legend حل‌شدهٔ یک سلول — ورودی‌ها + موقعیت (نگاشت به `Legend` قرارداد).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LegendSnapshot {
    pub entries: Vec<String>,
    pub position: LegendPos,
}

/// استایل حل‌شده — رنگ معنایی یا دقیق؛dash/marکر معین.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StyleSnapshot {
    pub color: ColorSnapshot,
    pub width: f32,
    pub dash: DashSpec,
    pub marker: Option<MarkerSpec>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorSnapshot {
    Palette(u8),
    Rgb { r: u8, g: u8, b: u8 },
}

/// محورهای حل‌شده — حدود مؤثر + تیک‌های 1-2-5 (یا لگاریتمی).
#[derive(Debug, Clone, PartialEq)]
pub struct AxesSnapshot {
    pub title: Option<String>,
    pub xlabel: Option<String>,
    pub ylabel: Option<String>,
    pub grid: bool,
    pub x_limits: [f64; 2],
    pub y_limits: [f64; 2],
    pub equal_aspect: bool,
    pub log_x: bool,
    pub log_y: bool,
    pub visible: bool,
    pub x_ticks: Vec<Tick>,
    pub y_ticks: Vec<Tick>,
}

// ---------------------------------------------------------------------------
// حالت
// ---------------------------------------------------------------------------

/// حالت کامل نشست — fold روی PlotOpها؛ سالم برای بازپخش و تست واحد.
#[derive(Debug, Default)]
pub struct FigureState {
    figures: Vec<Figure>,
    active_id: Option<u64>,
    next_figure: u64,
    series_counter: u64,
    touched: Vec<u64>,
}

#[derive(Debug)]
struct Figure {
    id: u64,
    rows: u32,
    cols: u32,
    active_cell: u32,
    cells: BTreeMap<u32, Cell>,
}

#[derive(Debug)]
struct Cell {
    kind: CellKind,
    axes: AxesCfg,
    legend: Option<LegendCfg>,
    hold: bool,
    palette_cursor: u64,
}

/// legend سلول — ورودی‌ها + موقعیت (حالت داخلی).
#[derive(Debug, Clone, PartialEq, Eq)]
struct LegendCfg {
    entries: Vec<String>,
    position: LegendPos,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            kind: CellKind::Empty,
            axes: AxesCfg::new(),
            legend: None,
            hold: false,
            palette_cursor: 0,
        }
    }
}

#[derive(Debug, Default, Clone)]
struct AxesCfg {
    title: Option<String>,
    xlabel: Option<String>,
    ylabel: Option<String>,
    grid: bool,
    limits_x: Option<[f64; 2]>,
    limits_y: Option<[f64; 2]>,
    equal: bool,
    visible: bool,
    tight: bool,
    log_x: bool,
    log_y: bool,
    /// تیک‌های سفارشی x (xticks) — None = خودکار (در رندرر تولید می‌شود).
    x_ticks_custom: Option<Vec<Tick>>,
    /// تیک‌های سفارشی y (yticks).
    y_ticks_custom: Option<Vec<Tick>>,
}

impl AxesCfg {
    fn new() -> Self {
        Self {
            visible: true,
            ..Self::default()
        }
    }
}

#[derive(Debug, Default)]
enum CellKind {
    #[default]
    Empty,
    Lines {
        series: Vec<BuiltSeries>,
        any_line: bool,
        any_scatter: bool,
    },
    Bars {
        series: Vec<BuiltSeries>,
        width: f32,
        horizontal: bool,
        stacked: bool,
    },
    Hist {
        edges: Vec<f64>,
        counts: Vec<f64>,
        norm: HistNorm,
        style: StyleSnapshot,
    },
    Area {
        series: Vec<BuiltSeries>,
        baseline: f64,
    },
    Fills {
        series: Vec<BuiltSeries>,
    },
    Polar {
        series: Vec<BuiltSeries>,
    },
    Errors {
        series: Vec<BuiltErrSeries>,
    },
}

/// سری ساخته‌شده — نمایش LTTB-کاسته + دادهٔ خام (برای re-decimation زوم).
#[derive(Debug, Clone)]
struct BuiltSeries {
    id: u64,
    raw_len: u64,
    x: Vec<f64>,
    y: Vec<f64>,
    raw_x: Vec<f64>,
    raw_y: Vec<f64>,
    style: StyleSnapshot,
}

#[derive(Debug, Clone)]
struct BuiltErrSeries {
    id: u64,
    raw_len: u64,
    x: Vec<f64>,
    y: Vec<f64>,
    raw_x: Vec<f64>,
    raw_y: Vec<f64>,
    x_err: Option<ErrBand>,
    y_err: Option<ErrBand>,
    style: StyleSnapshot,
}

/// برچسب نوع سلول برای تصمیم الحاق/جایگزینی (داخلی).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum KindTag {
    Lines,
    Bars,
    Area,
    Fills,
    Polar,
    Errors,
}

impl FigureState {
    pub fn new() -> Self {
        Self::default()
    }

    /// اعمال یک عملیات — مرتب؛ بدون خطا (اعتبارسنجی در لایهٔ builtin).
    pub fn apply(&mut self, op: PlotOp) {
        match op {
            PlotOp::ActivateFigure { id } => self.activate_figure(id),
            PlotOp::Hold { on } => self.active_cell_mut().hold = on,
            PlotOp::SubplotSelect { rows, cols, index } => {
                self.subplot_select(rows.max(1), cols.max(1), index);
            }
            PlotOp::Line { series } => {
                if !self.will_append(KindTag::Lines) {
                    self.fresh_axes();
                }
                let built = self.build_all(series, DashSpec::Solid, None);
                self.push_built(built, true);
            }
            PlotOp::Scatter { series } => {
                if !self.will_append(KindTag::Lines) {
                    self.fresh_axes();
                }
                let built = self.build_all(series, DashSpec::Invisible, Some(MarkerSpec::Circle));
                self.push_built(built, false);
            }
            PlotOp::Bar {
                series,
                width,
                horizontal,
                stacked,
            } => {
                if !self.will_append(KindTag::Bars) {
                    self.fresh_axes();
                }
                let built = self.build_all(series, DashSpec::Solid, None);
                self.push_bar(built, width, horizontal, stacked);
            }
            PlotOp::Histogram {
                edges,
                counts,
                norm,
                style,
            } => {
                // histogram همیشه جایگزین می‌کند (حتی زیر hold — پارتی متلب)
                self.fresh_axes();
                let resolved = self.default_style(&style, DashSpec::Invisible, None);
                let cell = self.active_cell_mut();
                cell.kind = CellKind::Hist {
                    edges,
                    counts,
                    norm,
                    style: resolved,
                };
            }
            PlotOp::Area { series, baseline } => {
                if !self.will_append(KindTag::Area) {
                    self.fresh_axes();
                }
                let built = self.build_all(series, DashSpec::Solid, None);
                self.push_area(built, baseline);
            }
            PlotOp::Fill { polys } => {
                if !self.will_append(KindTag::Fills) {
                    self.fresh_axes();
                }
                let mut built = Vec::with_capacity(polys.len());
                for p in polys {
                    // هر چندضلعی استایل خودش را resolve می‌کند (رنگ غایب =
                    // سیکل پالت — پارتی fill متلب)
                    let resolved = self.default_style(&p.style, DashSpec::Solid, None);
                    built.push(self.built_series(p.data, resolved));
                }
                self.push_fill(built);
            }
            PlotOp::Polar { series } => {
                if !self.will_append(KindTag::Polar) {
                    self.fresh_axes();
                }
                let built = self.build_all(series, DashSpec::Solid, None);
                self.push_polar(built);
            }
            PlotOp::ErrorBar { series } => {
                if !self.will_append(KindTag::Errors) {
                    self.fresh_axes();
                }
                let mut built = Vec::with_capacity(series.len());
                for p in series {
                    let resolved =
                        self.default_style(&p.style, DashSpec::Solid, Some(MarkerSpec::Circle));
                    built.push(self.built_err_series(p.data, p.x_err, p.y_err, resolved));
                }
                self.push_errors(built);
            }
            PlotOp::Legend { entries, position } => {
                self.active_cell_mut().legend =
                    entries.map(|entries| LegendCfg { entries, position });
            }
            PlotOp::Title { text } => self.active_cell_mut().axes.title = Some(text),
            PlotOp::XLabel { text } => self.active_cell_mut().axes.xlabel = Some(text),
            PlotOp::YLabel { text } => self.active_cell_mut().axes.ylabel = Some(text),
            PlotOp::Grid { on } => self.active_cell_mut().axes.grid = on,
            PlotOp::GridToggle => {
                let axes = &mut self.active_cell_mut().axes;
                axes.grid = !axes.grid;
            }
            PlotOp::HoldToggle => {
                let cell = self.active_cell_mut();
                cell.hold = !cell.hold;
            }
            PlotOp::SetLog { x, y } => {
                let axes = &mut self.active_cell_mut().axes;
                axes.log_x = x;
                axes.log_y = y;
            }
            PlotOp::XLim { limits } => self.active_cell_mut().axes.limits_x = limits,
            PlotOp::YLim { limits } => self.active_cell_mut().axes.limits_y = limits,
            PlotOp::XTicks { ticks } => {
                // تیک خالی = بازگشت به خودکار (پارتی xticks([]) متلب)
                let axes = &mut self.active_cell_mut().axes;
                axes.x_ticks_custom = if ticks.is_empty() { None } else { Some(ticks) };
            }
            PlotOp::YTicks { ticks } => {
                let axes = &mut self.active_cell_mut().axes;
                axes.y_ticks_custom = if ticks.is_empty() { None } else { Some(ticks) };
            }
            PlotOp::AxisLimits {
                min_x,
                max_x,
                min_y,
                max_y,
            } => {
                let axes = &mut self.active_cell_mut().axes;
                axes.limits_x = Some([min_x, max_x.max(min_x)]);
                axes.limits_y = Some([min_y, max_y.max(min_y)]);
            }
            PlotOp::AxisEqual { on } => self.active_cell_mut().axes.equal = on,
            PlotOp::AxisVisible { on } => self.active_cell_mut().axes.visible = on,
            PlotOp::AxisTight => self.active_cell_mut().axes.tight = true,
        }
    }

    /// شکل‌های دست‌خورده از آخرین تخلیه — سیگنال push برای `EvalResult.plots`.
    pub fn take_touched(&mut self) -> Vec<u64> {
        std::mem::take(&mut self.touched)
    }

    /// snapshot کامل همهٔ شکل‌ها (به‌ترتیب ساخت).
    pub fn snapshots(&self) -> Vec<FigureSnapshot> {
        self.figures
            .iter()
            .map(|f| FigureSnapshot {
                figure_id: f.id,
                spec: self.cell_grid(f),
            })
            .collect()
    }

    // ----- مسیریابی شکل/سلول -----

    fn touch(&mut self, id: u64) {
        if !self.touched.contains(&id) {
            self.touched.push(id);
        }
    }

    fn activate_figure(&mut self, id: Option<u64>) {
        let id = match id {
            Some(id) => {
                if self.next_figure <= id {
                    self.next_figure = id + 1;
                }
                id
            }
            None => {
                let id = self.next_figure.max(1);
                self.next_figure = id + 1;
                id
            }
        };
        if !self.figures.iter().any(|f| f.id == id) {
            self.figures.push(Figure {
                id,
                rows: 1,
                cols: 1,
                active_cell: 1,
                cells: BTreeMap::new(),
            });
        }
        self.active_id = Some(id);
        self.touch(id);
    }

    fn subplot_select(&mut self, rows: u32, cols: u32, index: u32) {
        let fig = self.active_figure_mut();
        fig.rows = rows;
        fig.cols = cols;
        // ایندکس خارج از شبکه دفاعاً به سلول ۱ برمی‌گردد (builtin خطا داده)
        fig.active_cell = if index >= 1 && index <= rows * cols {
            index
        } else {
            1
        };
        self.touch_active();
    }

    /// شکل فعال — در صورت نبود، شکل ۱ ضمنی ساخته/فعال می‌شود.
    fn active_figure_mut(&mut self) -> &mut Figure {
        let want = self.active_id.filter(|&id| id > 0).unwrap_or(0);
        if want == 0 || !self.figures.iter().any(|f| f.id == want) {
            let id = if want == 0 {
                self.next_figure.max(1)
            } else {
                want
            };
            if self.next_figure <= id {
                self.next_figure = id + 1;
            }
            self.figures.push(Figure {
                id,
                rows: 1,
                cols: 1,
                active_cell: 1,
                cells: BTreeMap::new(),
            });
            self.active_id = Some(id);
        }
        let id = self.active_id.unwrap_or(1);
        let idx = match self.figures.iter().position(|f| f.id == id) {
            Some(i) => i,
            // نابه (تأمین‌شده در بالا) — دفاعی: شکل تازه
            None => {
                self.figures.push(Figure {
                    id,
                    rows: 1,
                    cols: 1,
                    active_cell: 1,
                    cells: BTreeMap::new(),
                });
                self.figures.len() - 1
            }
        };
        &mut self.figures[idx]
    }

    fn active_cell_mut(&mut self) -> &mut Cell {
        self.touch_active();
        let fig = self.active_figure_mut();
        let idx = fig.active_cell;
        fig.cells.entry(idx).or_default()
    }

    fn touch_active(&mut self) {
        if self.active_id.is_none() {
            self.activate_figure(None);
        }
        if let Some(id) = self.active_id {
            self.touch(id);
        }
    }

    // ----- گزاره‌های داده‌ای (سری پیش از قرض سلول ساخته می‌شود) -----

    /// آیا گزارهٔ داده‌ای بعدی به سلول فعال «الحاق» می‌شود؟ فقط وقتی hold
    /// روشن است و نوع سلول هم‌خانوادهٔ همان برچسب باشد؛ وگرنه جایگزینی
    /// است (و سیکل پالت باید پیش از resolve رنگ ریست شود — پارتی متلب).
    fn will_append(&self, tag: KindTag) -> bool {
        let Some(id) = self.active_id else {
            return false;
        };
        let Some(f) = self.figures.iter().find(|f| f.id == id) else {
            return false;
        };
        let Some(c) = f.cells.get(&f.active_cell) else {
            return false;
        };
        if !c.hold {
            return false;
        }
        matches!(
            (&c.kind, tag),
            (CellKind::Lines { .. }, KindTag::Lines)
                | (CellKind::Bars { .. }, KindTag::Bars)
                | (CellKind::Area { .. }, KindTag::Area)
                | (CellKind::Fills { .. }, KindTag::Fills)
                | (CellKind::Polar { .. }, KindTag::Polar)
                | (CellKind::Errors { .. }, KindTag::Errors)
        )
    }

    /// پرچم hold سلول فعال (برای تست‌ها).
    pub fn active_cell_hold(&self) -> bool {
        let Some(id) = self.active_id else {
            return false;
        };
        self.figures
            .iter()
            .find(|f| f.id == id)
            .and_then(|f| f.cells.get(&f.active_cell))
            .map(|c| c.hold)
            .unwrap_or(false)
    }

    /// محورهای تازه برای گزارهٔ جایگزین‌کننده (گام 3.5.2 — باگ کاربر):
    /// وقتی محتوای سلول بدون hold (یا با تغییر نوع زیر hold) جایگزین
    /// می‌شود، حاشیه‌نویسی‌ها هم مثل متلب پاک می‌شوند — legend (فرزندِ
    /// محورهای قبلی)، title/xlabel/ylabel، حدود دستی، تیک‌های سفارشی،
    /// equal/tight و پرچم‌های لگاریتمی. grid و visible و hold می‌مانند
    /// (خودِ متلب هم اینها را بعد از plot جدید حفظ می‌کند) + سیکل پالت
    /// از صفر شروع می‌شود.
    fn fresh_axes(&mut self) {
        let cell = self.active_cell_mut();
        cell.palette_cursor = 0;
        cell.legend = None;
        let axes = &mut cell.axes;
        axes.title = None;
        axes.xlabel = None;
        axes.ylabel = None;
        axes.limits_x = None;
        axes.limits_y = None;
        axes.x_ticks_custom = None;
        axes.y_ticks_custom = None;
        axes.equal = false;
        axes.tight = false;
        axes.log_x = false;
        axes.log_y = false;
    }

    /// استایل حل‌شده: اجزای غایب از پیش‌فرض‌های موتور؛ رنگ خودکار از
    /// سیکل پالت سلول (نشانگر فقط برای رنگ خودکار جلو می‌رود).
    fn default_style(
        &mut self,
        s: &StyleSpec,
        dash: DashSpec,
        marker: Option<MarkerSpec>,
    ) -> StyleSnapshot {
        let cursor = {
            let fig = self.active_figure_mut();
            let idx = fig.active_cell;
            let cell = fig.cells.entry(idx).or_default();
            if s.color.is_none() {
                let c = cell.palette_cursor;
                cell.palette_cursor += 1;
                c
            } else {
                cell.palette_cursor
            }
        };
        StyleSnapshot {
            color: s
                .color
                .unwrap_or(PlotColorSpec::Palette(palette::cycle(cursor)))
                .into(),
            width: s.width.unwrap_or(DEFAULT_WIDTH),
            dash: s.dash.unwrap_or(dash),
            marker: s.marker.or(marker),
        }
    }

    /// ساخت همهٔ سری‌های یک فراخوانی — هر سری استایل خودش را resolve می‌کند
    /// (رنگ خودکار = سیکل پالت، پشت‌سرهم).
    fn build_all(
        &mut self,
        series: Vec<SeriesPair>,
        dash: DashSpec,
        marker: Option<MarkerSpec>,
    ) -> Vec<BuiltSeries> {
        let mut out = Vec::with_capacity(series.len());
        for sp in series {
            let resolved = self.default_style(&sp.style, dash, marker);
            out.push(self.built_series(sp.data, resolved));
        }
        out
    }

    /// plot/scatter زیر یک سقف مشترک: hold → الحاق (تا نوع Line)، وگرنه
    /// جایگزینی. mix با scatter = هر سری استایل خودش («فقط مارکر» = نامرئی).
    fn push_built(&mut self, built: Vec<BuiltSeries>, is_line: bool) {
        let cell = self.active_cell_mut();
        let kind = std::mem::take(&mut cell.kind);
        cell.kind = match kind {
            CellKind::Lines {
                mut series,
                mut any_line,
                mut any_scatter,
            } if cell.hold => {
                series.extend(built);
                if is_line {
                    any_line = true;
                } else {
                    any_scatter = true;
                }
                CellKind::Lines {
                    series,
                    any_line,
                    any_scatter,
                }
            }
            _ => CellKind::Lines {
                series: built,
                any_line: is_line,
                any_scatter: !is_line,
            },
        };
    }

    fn push_bar(
        &mut self,
        built: Vec<BuiltSeries>,
        width: Option<f32>,
        horizontal: bool,
        stacked: bool,
    ) {
        let cell = self.active_cell_mut();
        let kind = std::mem::take(&mut cell.kind);
        cell.kind = match kind {
            CellKind::Bars {
                mut series,
                width: w,
                ..
            } if cell.hold => {
                series.extend(built);
                CellKind::Bars {
                    series,
                    width: width.unwrap_or(w),
                    horizontal,
                    stacked,
                }
            }
            _ => CellKind::Bars {
                series: built,
                width: width.unwrap_or(0.8),
                horizontal,
                stacked,
            },
        };
    }

    fn push_area(&mut self, built: Vec<BuiltSeries>, baseline: f64) {
        let cell = self.active_cell_mut();
        let kind = std::mem::take(&mut cell.kind);
        cell.kind = match kind {
            CellKind::Area { mut series, .. } if cell.hold => {
                series.extend(built);
                CellKind::Area { series, baseline }
            }
            _ => CellKind::Area {
                series: built,
                baseline,
            },
        };
    }

    fn push_fill(&mut self, built: Vec<BuiltSeries>) {
        let cell = self.active_cell_mut();
        let kind = std::mem::take(&mut cell.kind);
        cell.kind = match kind {
            CellKind::Fills { mut series } if cell.hold => {
                series.extend(built);
                CellKind::Fills { series }
            }
            _ => CellKind::Fills { series: built },
        };
    }

    fn push_polar(&mut self, built: Vec<BuiltSeries>) {
        let cell = self.active_cell_mut();
        let kind = std::mem::take(&mut cell.kind);
        cell.kind = match kind {
            CellKind::Polar { mut series } if cell.hold => {
                series.extend(built);
                CellKind::Polar { series }
            }
            _ => CellKind::Polar { series: built },
        };
    }

    fn push_errors(&mut self, built: Vec<BuiltErrSeries>) {
        let cell = self.active_cell_mut();
        let kind = std::mem::take(&mut cell.kind);
        cell.kind = match kind {
            CellKind::Errors { mut series } if cell.hold => {
                series.extend(built);
                CellKind::Errors { series }
            }
            _ => CellKind::Errors { series: built },
        };
    }

    /// سری نمایشی — LTTB برای انتقال؛ دادهٔ خام برای re-decimation زوم.
    fn built_series(&mut self, data: crate::spec::SeriesData, style: StyleSnapshot) -> BuiltSeries {
        let raw_len = data.x.len() as u64;
        let (x, y) = lttb::decimate(&data.x, &data.y, lttb::THRESHOLD);
        self.series_counter += 1;
        BuiltSeries {
            id: self.series_counter,
            raw_len,
            x,
            y,
            raw_x: data.x,
            raw_y: data.y,
            style,
        }
    }

    fn built_err_series(
        &mut self,
        data: crate::spec::SeriesData,
        x_err: Option<ErrBand>,
        y_err: Option<ErrBand>,
        style: StyleSnapshot,
    ) -> BuiltErrSeries {
        let raw_len = data.x.len() as u64;
        let (x, y) = lttb::decimate(&data.x, &data.y, lttb::THRESHOLD);
        self.series_counter += 1;
        BuiltErrSeries {
            id: self.series_counter,
            raw_len,
            x,
            y,
            raw_x: data.x,
            raw_y: data.y,
            x_err,
            y_err,
            style,
        }
    }

    // ----- snapshot -----

    fn cell_grid(&self, f: &Figure) -> CellSnapshot {
        if f.rows == 1 && f.cols == 1 {
            return self.cell_snapshot(f, f.active_cell.max(1));
        }
        let mut cells = Vec::new();
        for (&idx, _) in f.cells.range(1..=f.rows * f.cols) {
            cells.push(SubplotCellSnapshot {
                row: (idx - 1) / f.cols,
                col: (idx - 1) % f.cols,
                spec: self.cell_snapshot(f, idx),
            });
        }
        CellSnapshot::Subplot {
            rows: f.rows,
            cols: f.cols,
            cells,
        }
    }

    fn cell_snapshot(&self, f: &Figure, idx: u32) -> CellSnapshot {
        let empty = Cell::default();
        let cell = f.cells.get(&idx).unwrap_or(&empty);
        let legend = cell.legend.clone().map(|l| LegendSnapshot {
            entries: l.entries,
            position: l.position,
        });
        let axes = self.axes_snapshot(cell);
        match &cell.kind {
            CellKind::Empty => CellSnapshot::Line {
                series: Vec::new(),
                axes,
                legend,
            },
            CellKind::Lines {
                series, any_line, ..
            } => {
                let s: Vec<SeriesSnapshot> = series.iter().map(|b| b.snap()).collect();
                if *any_line {
                    CellSnapshot::Line {
                        series: s,
                        axes,
                        legend,
                    }
                } else {
                    CellSnapshot::Scatter {
                        series: s,
                        axes,
                        legend,
                    }
                }
            }
            CellKind::Bars {
                series,
                width,
                horizontal,
                stacked,
            } => CellSnapshot::Bar {
                series: series.iter().map(|b| b.snap()).collect(),
                width: *width,
                horizontal: *horizontal,
                stacked: *stacked,
                axes,
                legend,
            },
            CellKind::Hist {
                edges,
                counts,
                norm,
                style,
            } => CellSnapshot::Histogram {
                edges: edges.clone(),
                counts: counts.clone(),
                norm: *norm,
                style: *style,
                axes,
                legend,
            },
            CellKind::Area { series, baseline } => CellSnapshot::Area {
                series: series.iter().map(|b| b.snap()).collect(),
                baseline: *baseline,
                axes,
                legend,
            },
            CellKind::Fills { series } => CellSnapshot::Fill {
                series: series.iter().map(|b| b.snap()).collect(),
                axes,
            },
            CellKind::Polar { series } => CellSnapshot::Polar {
                series: series.iter().map(|b| b.snap()).collect(),
                legend,
            },
            CellKind::Errors { series } => CellSnapshot::ErrorBar {
                series: series.iter().map(|b| b.snap()).collect(),
                axes,
                legend,
            },
        }
    }

    /// محورهای حل‌شده: حدود مؤثر (دستی > داده + حاشیه؛ tight بدون حاشیه)
    /// + تیک‌ها فقط وقتی سفارشی‌اند (v2.1: تولید خودکار در رندرر Dart است).
    fn axes_snapshot(&self, cell: &Cell) -> AxesSnapshot {
        let (mut mn_x, mut mx_x, mut mn_y, mut mx_y) = self.data_bounds(cell);
        if self.has_data(cell) {
            pad_bounds(&mut mn_x, &mut mx_x, cell.axes.tight);
            pad_bounds(&mut mn_y, &mut mx_y, cell.axes.tight);
        }
        if let Some([a, b]) = cell.axes.limits_x {
            mn_x = a;
            mx_x = b;
        }
        if let Some([c, d]) = cell.axes.limits_y {
            mn_y = c;
            mx_y = d;
        }
        // محور لگاریتمی به دادهٔ مثبت نیاز دارد
        if cell.axes.log_x && mn_x <= 0.0 {
            mn_x = 1e-3;
            mx_x = mx_x.max(mn_x * 10.0);
        }
        if cell.axes.log_y && mn_y <= 0.0 {
            mn_y = 1e-3;
            mx_y = mx_y.max(mn_y * 10.0);
        }
        // v2.1: تیک خودکار ساطع نمی‌شود (Dart برای viewport جاری می‌سازد)؛
        // فقط سفارشی‌های کاربر منتقل می‌شوند.
        let x_ticks = cell.axes.x_ticks_custom.clone().unwrap_or_default();
        let y_ticks = cell.axes.y_ticks_custom.clone().unwrap_or_default();
        AxesSnapshot {
            title: cell.axes.title.clone(),
            xlabel: cell.axes.xlabel.clone(),
            ylabel: cell.axes.ylabel.clone(),
            grid: cell.axes.grid,
            x_limits: [mn_x, mx_x],
            y_limits: [mn_y, mx_y],
            equal_aspect: cell.axes.equal,
            log_x: cell.axes.log_x,
            log_y: cell.axes.log_y,
            visible: cell.axes.visible,
            x_ticks,
            y_ticks,
        }
    }

    /// آیا سلول داده‌ای برای کادربندی خودکار دارد؟
    fn has_data(&self, cell: &Cell) -> bool {
        match &cell.kind {
            CellKind::Empty => false,
            CellKind::Lines { series, .. }
            | CellKind::Bars { series, .. }
            | CellKind::Area { series, .. }
            | CellKind::Fills { series, .. }
            | CellKind::Polar { series, .. } => !series.is_empty(),
            CellKind::Errors { series } => !series.is_empty(),
            CellKind::Hist { edges, .. } => !edges.is_empty(),
        }
    }

    /// دادهٔ خام یک سری — پایهٔ re-decimation زوم (سند ۰۲ §7): دادهٔ کامل
    /// در Rust می‌ماند و هر viewport جدید از همین بازسازی می‌شود.
    pub fn series_raw(
        &self,
        figure_id: u64,
        cell_index: u32,
        series_id: u64,
    ) -> Option<(&[f64], &[f64])> {
        let fig = self.figures.iter().find(|f| f.id == figure_id)?;
        let cell = fig.cells.get(&cell_index)?;
        match &cell.kind {
            CellKind::Lines { series, .. }
            | CellKind::Bars { series, .. }
            | CellKind::Area { series, .. }
            | CellKind::Fills { series, .. }
            | CellKind::Polar { series, .. } => series
                .iter()
                .find(|s| s.id == series_id)
                .map(|s| (s.raw_x.as_slice(), s.raw_y.as_slice())),
            CellKind::Errors { series } => series
                .iter()
                .find(|s| s.id == series_id)
                .map(|s| (s.raw_x.as_slice(), s.raw_y.as_slice())),
            CellKind::Hist { .. } | CellKind::Empty => None,
        }
    }

    /// کادر دادهٔ خام سلول — برای حدود خودکار (شامل نوار خطا/خط مبنا/صفر میله).
    fn data_bounds(&self, cell: &Cell) -> (f64, f64, f64, f64) {
        let mut xs: Vec<f64> = Vec::new();
        let mut ys: Vec<f64> = Vec::new();
        match &cell.kind {
            CellKind::Empty => {}
            CellKind::Lines { series, .. }
            | CellKind::Fills { series }
            | CellKind::Polar { series } => {
                for s in series {
                    xs.extend(s.x.iter().copied());
                    ys.extend(s.y.iter().copied());
                }
            }
            CellKind::Bars { series, .. } => {
                for s in series {
                    xs.extend(s.x.iter().copied());
                    ys.extend(s.y.iter().copied());
                }
                ys.push(0.0);
            }
            CellKind::Area { series, baseline } => {
                for s in series {
                    xs.extend(s.x.iter().copied());
                    ys.extend(s.y.iter().copied());
                }
                ys.push(*baseline);
            }
            CellKind::Hist { edges, counts, .. } => {
                if let (Some(&first), Some(&last)) = (edges.first(), edges.last()) {
                    xs.push(first);
                    xs.push(last);
                }
                ys.push(0.0);
                ys.extend(counts.iter().copied());
            }
            CellKind::Errors { series } => {
                for s in series {
                    xs.extend(s.x.iter().copied());
                    for i in 0..s.y.len() {
                        ys.push(s.y[i]);
                        if let Some(e) = &s.y_err {
                            if i < e.neg.len() {
                                ys.push(s.y[i] - e.neg[i]);
                            }
                            if i < e.pos.len() {
                                ys.push(s.y[i] + e.pos[i]);
                            }
                        }
                        if let Some(e) = &s.x_err {
                            if i < e.neg.len() {
                                xs.push(s.x[i] - e.neg[i]);
                            }
                            if i < e.pos.len() {
                                xs.push(s.x[i] + e.pos[i]);
                            }
                        }
                    }
                }
            }
        }
        let fold = |v: &[f64]| -> (f64, f64) {
            let fin: Vec<f64> = v.iter().copied().filter(|p| p.is_finite()).collect();
            if fin.is_empty() {
                (0.0, 1.0)
            } else {
                (
                    fin.iter().copied().fold(f64::INFINITY, f64::min),
                    fin.iter().copied().fold(f64::NEG_INFINITY, f64::max),
                )
            }
        };
        let (a, b) = fold(&xs);
        let (c, d) = fold(&ys);
        (a, b, c, d)
    }
}

impl BuiltSeries {
    fn snap(&self) -> SeriesSnapshot {
        SeriesSnapshot {
            id: self.id,
            raw_len: self.raw_len,
            x: self.x.clone(),
            y: self.y.clone(),
            style: self.style,
        }
    }
}

impl BuiltErrSeries {
    fn snap(&self) -> ErrSeriesSnapshot {
        ErrSeriesSnapshot {
            id: self.id,
            raw_len: self.raw_len,
            x: self.x.clone(),
            y: self.y.clone(),
            x_err: self.x_err.clone(),
            y_err: self.y_err.clone(),
            style: self.style,
        }
    }
}

impl From<PlotColorSpec> for ColorSnapshot {
    fn from(c: PlotColorSpec) -> Self {
        match c {
            PlotColorSpec::Palette(i) => ColorSnapshot::Palette(i),
            PlotColorSpec::Rgb { r, g, b } => ColorSnapshot::Rgb { r, g, b },
        }
    }
}

fn pad_bounds(mn: &mut f64, mx: &mut f64, tight: bool) {
    if *mx <= *mn {
        // بازهٔ صفرعرض: ±۱ (یا ±۰.۵ حول صفر)
        let mid = (*mn + *mx) / 2.0;
        let half = if mid == 0.0 { 0.5 } else { mid.abs() * 0.5 };
        *mn = mid - half;
        *mx = mid + half;
        return;
    }
    if tight {
        return;
    }
    let pad = (*mx - *mn) * AUTO_PAD;
    *mn -= pad;
    *mx += pad;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec::{ErrorPair, FillPair, LegendPos, PlotOp, SeriesData, SeriesPair, StyleSpec};

    /// سری پیش‌فرض برای تست‌ها.
    fn sop(d: SeriesData) -> SeriesPair {
        SeriesPair {
            data: d,
            style: StyleSpec::default(),
        }
    }

    fn data(n: usize) -> SeriesData {
        SeriesData {
            x: (0..n).map(|i| i as f64).collect(),
            y: (0..n).map(|i| (i as f64 * 0.1).sin()).collect(),
        }
    }

    fn apply_all(ops: Vec<PlotOp>) -> FigureState {
        let mut st = FigureState::new();
        for op in ops {
            st.apply(op);
        }
        st
    }

    #[test]
    fn bare_plot_creates_figure_one_line_series() {
        let st = apply_all(vec![PlotOp::Line {
            series: vec![sop(data(10))],
        }]);
        let snaps = st.snapshots();
        assert_eq!(snaps.len(), 1);
        assert_eq!(snaps[0].figure_id, 1);
        let CellSnapshot::Line { series, axes, .. } = &snaps[0].spec else {
            panic!("expected Line");
        };
        assert_eq!(series.len(), 1);
        assert_eq!(series[0].raw_len, 10);
        assert_eq!(series[0].style.color, ColorSnapshot::Palette(0));
        assert_eq!(series[0].style.dash, DashSpec::Solid);
        // v2.1: تیک خودکار ساطع نمی‌شود — تولید در رندرر Dart است
        assert!(axes.x_ticks.is_empty());
        assert!(axes.y_ticks.is_empty());
    }

    /// گام 3.5.2 — باگ کاربر: semilogx بعد از نشست قبلی با legend،
    /// زیر نمودار تازه «sin/cos» مانده بود. جایگزینی بدون hold باید
    /// حاشیه‌نویسی‌ها را پاک کند ولی grid را نگه دارد (پارتی متلب).
    #[test]
    fn replace_clears_stale_annotations_keeps_grid() {
        let st = apply_all(vec![
            PlotOp::Line {
                series: vec![sop(data(3))],
            },
            PlotOp::Legend {
                entries: Some(vec!["sin".into(), "cos".into()]),
                position: LegendPos::SouthWest,
            },
            PlotOp::Title { text: "old".into() },
            PlotOp::XLabel { text: "t".into() },
            PlotOp::SetLog { x: true, y: false },
            PlotOp::Grid { on: true },
            PlotOp::XTicks {
                ticks: vec![Tick {
                    position: 1.0,
                    label: Some("یک".into()),
                }],
            },
            // جایگزینی بدون hold → fresh axes
            PlotOp::Line {
                series: vec![sop(data(5))],
            },
        ]);
        let CellSnapshot::Line {
            series,
            axes,
            legend,
        } = &st.snapshots()[0].spec
        else {
            panic!("expected Line");
        };
        assert_eq!(series.len(), 1, "old series replaced");
        assert_eq!(series[0].raw_len, 5);
        assert!(legend.is_none(), "stale legend must be cleared");
        assert!(axes.title.is_none());
        assert!(axes.xlabel.is_none());
        assert!(!axes.log_x, "stale log flag must be cleared");
        assert!(axes.x_ticks.is_empty(), "custom ticks back to auto");
        assert!(axes.grid, "grid persists across replace (MATLAB parity)");
    }

    /// زیر hold هیچ ریستی نیست: legend و سری‌ها می‌مانند.
    #[test]
    fn hold_appends_keep_legend() {
        let st = apply_all(vec![
            PlotOp::Line {
                series: vec![sop(data(3))],
            },
            PlotOp::Legend {
                entries: Some(vec!["a".into()]),
                position: LegendPos::Best,
            },
            PlotOp::Hold { on: true },
            PlotOp::Line {
                series: vec![sop(data(3))],
            },
        ]);
        let CellSnapshot::Line { series, legend, .. } = &st.snapshots()[0].spec else {
            panic!("expected Line");
        };
        assert_eq!(series.len(), 2);
        assert!(legend.is_some(), "legend under hold must stay");
    }

    #[test]
    fn palette_cycles_and_hold_appends() {
        let st = apply_all(vec![
            PlotOp::Line {
                series: vec![sop(data(3))],
            },
            PlotOp::Hold { on: true },
            PlotOp::Line {
                series: vec![sop(data(3))],
            },
            PlotOp::Line {
                series: vec![sop(data(3))],
            },
        ]);
        let CellSnapshot::Line { series, .. } = &st.snapshots()[0].spec else {
            panic!()
        };
        assert_eq!(series.len(), 3);
        assert_eq!(series[0].style.color, ColorSnapshot::Palette(0));
        assert_eq!(series[1].style.color, ColorSnapshot::Palette(1));
        assert_eq!(series[2].style.color, ColorSnapshot::Palette(2));
    }

    #[test]
    fn no_hold_replaces_and_resets_palette() {
        let st = apply_all(vec![
            PlotOp::Line {
                series: vec![sop(data(3))],
            },
            PlotOp::Line {
                series: vec![sop(data(3))],
            },
        ]);
        let CellSnapshot::Line { series, .. } = &st.snapshots()[0].spec else {
            panic!()
        };
        assert_eq!(series.len(), 1);
        assert_eq!(series[0].style.color, ColorSnapshot::Palette(0));
    }

    #[test]
    fn scatter_defaults_to_marker_only_and_scatter_variant() {
        let st = apply_all(vec![PlotOp::Scatter {
            series: vec![sop(data(4))],
        }]);
        let CellSnapshot::Scatter { series, .. } = &st.snapshots()[0].spec else {
            panic!("expected Scatter");
        };
        assert_eq!(series[0].style.dash, DashSpec::Invisible);
        assert_eq!(series[0].style.marker, Some(MarkerSpec::Circle));
        // خط‌اسپک صریح مارکر را بازنویسی می‌کند
        let st = apply_all(vec![PlotOp::Scatter {
            series: vec![SeriesPair {
                data: data(4),
                style: StyleSpec {
                    marker: Some(MarkerSpec::Star),
                    ..StyleSpec::default()
                },
            }],
        }]);
        let CellSnapshot::Scatter { series, .. } = &st.snapshots()[0].spec else {
            panic!()
        };
        assert_eq!(series[0].style.marker, Some(MarkerSpec::Star));
    }

    #[test]
    fn mixed_line_scatter_under_hold_unifies_as_line() {
        let st = apply_all(vec![
            PlotOp::Line {
                series: vec![sop(data(3))],
            },
            PlotOp::Hold { on: true },
            PlotOp::Scatter {
                series: vec![sop(data(3))],
            },
        ]);
        let CellSnapshot::Line { series, .. } = &st.snapshots()[0].spec else {
            panic!("mixed → Line");
        };
        assert_eq!(series.len(), 2);
        assert_eq!(series[1].style.dash, DashSpec::Invisible);
    }

    #[test]
    fn different_kind_under_hold_replaces_documented() {
        let st = apply_all(vec![
            PlotOp::Line {
                series: vec![sop(data(3))],
            },
            PlotOp::Hold { on: true },
            PlotOp::Bar {
                series: vec![sop(data(3))],
                width: None,
                horizontal: false,
                stacked: false,
            },
        ]);
        assert!(matches!(&st.snapshots()[0].spec, CellSnapshot::Bar { .. }));
    }

    #[test]
    fn bar_defaults_and_last_call_wins() {
        let st = apply_all(vec![PlotOp::Bar {
            series: vec![sop(data(3))],
            width: Some(0.5),
            horizontal: true,
            stacked: false,
        }]);
        let CellSnapshot::Bar {
            width,
            horizontal,
            stacked,
            ..
        } = &st.snapshots()[0].spec
        else {
            panic!()
        };
        assert!((width - 0.5).abs() < 1e-6);
        assert!(*horizontal);
        assert!(!*stacked);
        // محور y خودکار صفر را دربر می‌گیرد
        let CellSnapshot::Bar { axes, .. } = &st.snapshots()[0].spec else {
            panic!()
        };
        assert!(axes.y_limits[0] <= 0.0 && axes.y_limits[1] >= 0.0);
    }

    #[test]
    fn axis_grid_labels_and_limits_flow_into_axes() {
        // پس از گام 3.5.2: گزارهٔ داده‌ای جایگزین‌کننده حاشیه‌نویسیِ
        // «قبل از خودش» را پاک می‌کند (fresh axes) — مثل متلب. پس ترتیب
        // واقعی کاربر: اول Line، بعد حاشیه‌نویسی. grid می‌ماند.
        let st = apply_all(vec![
            PlotOp::Line {
                series: vec![sop(data(3))],
            },
            PlotOp::Grid { on: true },
            PlotOp::Title { text: "T".into() },
            PlotOp::XLabel { text: "t".into() },
            PlotOp::YLabel { text: "s".into() },
            PlotOp::AxisLimits {
                min_x: -1.0,
                max_x: 1.0,
                min_y: 0.0,
                max_y: 2.0,
            },
            PlotOp::AxisEqual { on: true },
        ]);
        let CellSnapshot::Line { axes, .. } = &st.snapshots()[0].spec else {
            panic!()
        };
        assert!(axes.grid);
        assert_eq!(axes.title.as_deref(), Some("T"));
        assert_eq!(axes.xlabel.as_deref(), Some("t"));
        assert_eq!(axes.ylabel.as_deref(), Some("s"));
        assert_eq!(axes.x_limits, [-1.0, 1.0]);
        assert_eq!(axes.y_limits, [0.0, 2.0]);
        assert!(axes.equal_aspect);
    }

    #[test]
    fn legend_op_sets_entries_and_position_on_active_cell() {
        let st = apply_all(vec![
            PlotOp::Line {
                series: vec![sop(data(3))],
            },
            PlotOp::Legend {
                entries: Some(vec!["sin".into(), "cos".into()]),
                position: LegendPos::SouthWest,
            },
        ]);
        let CellSnapshot::Line { legend, .. } = &st.snapshots()[0].spec else {
            panic!()
        };
        let l = legend.as_ref().expect("legend set");
        assert_eq!(l.entries, vec!["sin".to_owned(), "cos".to_owned()]);
        assert_eq!(l.position, LegendPos::SouthWest);
        // پیش‌فرض موقعیت = Best
        let st = apply_all(vec![PlotOp::Legend {
            entries: Some(vec!["a".into()]),
            position: LegendPos::Best,
        }]);
        let CellSnapshot::Line { legend, .. } = &st.snapshots()[0].spec else {
            panic!()
        };
        assert_eq!(legend.as_ref().unwrap().position, LegendPos::Best);
    }

    #[test]
    fn figures_are_independent_and_fig_selects() {
        let st = apply_all(vec![
            PlotOp::Line {
                series: vec![sop(data(3))],
            },
            PlotOp::ActivateFigure { id: Some(3) },
            PlotOp::Line {
                series: vec![sop(data(4))],
            },
            PlotOp::ActivateFigure { id: None }, // بعدی = 4
        ]);
        // {1, 3, 4} — شکل ۴ با figure جدید ایجاد شد (فعال)
        let snaps = st.snapshots();
        assert_eq!(snaps.len(), 3);
        assert_eq!(snaps[0].figure_id, 1);
        assert_eq!(snaps[1].figure_id, 3);
        assert_eq!(snaps[2].figure_id, 4);
        // سری بعدی به شکل فعال (۴) می‌رود؛ بقیه دست‌نخورده
        let mut st2 = st;
        st2.apply(PlotOp::Line {
            series: vec![sop(data(2))],
        });
        assert_eq!(st2.snapshots().len(), 3);
        let CellSnapshot::Line { series, .. } = &st2.snapshots()[2].spec else {
            panic!()
        };
        assert_eq!(series.len(), 1);
    }

    #[test]
    fn subplot_grid_builds_nested_snapshot() {
        let st = apply_all(vec![
            PlotOp::SubplotSelect {
                rows: 2,
                cols: 2,
                index: 1,
            },
            PlotOp::Line {
                series: vec![sop(data(3))],
            },
            PlotOp::SubplotSelect {
                rows: 2,
                cols: 2,
                index: 4,
            },
            PlotOp::Bar {
                series: vec![sop(data(3))],
                width: None,
                horizontal: false,
                stacked: false,
            },
        ]);
        let CellSnapshot::Subplot { rows, cols, cells } = &st.snapshots()[0].spec else {
            panic!("expected Subplot");
        };
        assert_eq!((*rows, *cols), (2, 2));
        assert_eq!(cells.len(), 2);
        assert_eq!((cells[0].row, cells[0].col), (0, 0));
        assert!(matches!(cells[0].spec, CellSnapshot::Line { .. }));
        assert_eq!((cells[1].row, cells[1].col), (1, 1));
        assert!(matches!(cells[1].spec, CellSnapshot::Bar { .. }));
    }

    #[test]
    fn subplot_invalid_index_defensively_clamps_to_one() {
        // ایندکس خارج ۱..=rows*cols دفاعاً به سلول ۱ می‌رود (خطای واقعی در builtin)
        let mut st = FigureState::new();
        st.apply(PlotOp::SubplotSelect {
            rows: 2,
            cols: 2,
            index: 5,
        });
        st.apply(PlotOp::Line {
            series: vec![sop(data(2))],
        });
        let CellSnapshot::Subplot { cells, .. } = &st.snapshots()[0].spec else {
            panic!()
        };
        assert_eq!((cells[0].row, cells[0].col), (0, 0));
    }

    #[test]
    fn axis_off_hides_axes_decoration_in_snapshot() {
        let st = apply_all(vec![
            PlotOp::AxisVisible { on: false },
            PlotOp::Line {
                series: vec![sop(data(3))],
            },
        ]);
        let CellSnapshot::Line { axes, .. } = &st.snapshots()[0].spec else {
            panic!()
        };
        assert!(!axes.visible);
        // پیش‌فرض: نمایان
        let st = apply_all(vec![PlotOp::Line {
            series: vec![sop(data(3))],
        }]);
        let CellSnapshot::Line { axes, .. } = &st.snapshots()[0].spec else {
            panic!()
        };
        assert!(axes.visible);
    }

    #[test]
    fn histogram_replaces_even_under_hold() {
        let st = apply_all(vec![
            PlotOp::Line {
                series: vec![sop(data(3))],
            },
            PlotOp::Hold { on: true },
            PlotOp::Histogram {
                edges: vec![0.0, 1.0, 2.0],
                counts: vec![3.0, 7.0],
                norm: HistNorm::Count,
                style: StyleSpec::default(),
            },
        ]);
        let CellSnapshot::Histogram {
            edges,
            counts,
            axes,
            ..
        } = &st.snapshots()[0].spec
        else {
            panic!("expected Histogram");
        };
        assert_eq!(edges, &[0.0, 1.0, 2.0]);
        assert_eq!(counts, &[3.0, 7.0]);
        assert!(axes.y_limits[0] <= 0.0 && axes.y_limits[1] >= 7.0);
    }

    #[test]
    fn fill_and_polar_and_area_shapes() {
        let st = apply_all(vec![PlotOp::Fill {
            polys: vec![
                FillPair {
                    data: data(4),
                    style: StyleSpec::default(),
                },
                FillPair {
                    data: data(4),
                    style: StyleSpec::default(),
                },
            ],
        }]);
        // هر چندضلعی رنگ خودش را از سیکل می‌گیرد (پارتی fill متلب)
        let CellSnapshot::Fill { series, .. } = &st.snapshots()[0].spec else {
            panic!()
        };
        assert_eq!(series.len(), 2);
        assert_eq!(series[0].style.color, ColorSnapshot::Palette(0));
        assert_eq!(series[1].style.color, ColorSnapshot::Palette(1));
        let st = apply_all(vec![PlotOp::Polar {
            series: vec![sop(data(4))],
        }]);
        assert!(matches!(st.snapshots()[0].spec, CellSnapshot::Polar { .. }));
        let st = apply_all(vec![PlotOp::Area {
            series: vec![sop(data(4))],
            baseline: 0.0,
        }]);
        let CellSnapshot::Area { baseline, axes, .. } = &st.snapshots()[0].spec else {
            panic!()
        };
        assert!((baseline - 0.0).abs() < 1e-12);
        assert!(axes.y_limits[0] <= 0.0); // خط مبنا در کادر
    }

    #[test]
    fn errorbar_series_keeps_bands() {
        let st = apply_all(vec![PlotOp::ErrorBar {
            series: vec![ErrorPair {
                data: SeriesData {
                    x: vec![1.0, 2.0, 3.0],
                    y: vec![1.0, 2.0, 3.0],
                },
                x_err: None,
                y_err: Some(ErrBand {
                    neg: vec![0.1; 3],
                    pos: vec![0.2; 3],
                }),
                style: StyleSpec::default(),
            }],
        }]);
        let CellSnapshot::ErrorBar { series, axes, .. } = &st.snapshots()[0].spec else {
            panic!()
        };
        assert_eq!(series.len(), 1);
        assert!(series[0].y_err.is_some());
        assert!(series[0].x_err.is_none());
        // کادر خودکار نوارها را دربر می‌گیرد
        assert!(axes.y_limits[0] < 1.0 && axes.y_limits[1] > 3.0);
    }

    #[test]
    fn lttb_threshold_applies_to_display_only() {
        let n = 10_000;
        let st = apply_all(vec![PlotOp::Line {
            series: vec![sop(data(n))],
        }]);
        let CellSnapshot::Line { series, .. } = &st.snapshots()[0].spec else {
            panic!()
        };
        assert_eq!(series[0].raw_len, n as u64);
        assert!(series[0].x.len() <= lttb::THRESHOLD);
        assert!(series[0].x.len() >= lttb::THRESHOLD - 2);
    }

    #[test]
    fn multi_series_call_lands_together_without_hold() {
        // plot(X, [Y1 Y2 Y3]) = سه سری در «یک» فراخوانی — همه با هم
        let st = apply_all(vec![PlotOp::Line {
            series: vec![sop(data(3)), sop(data(3)), sop(data(3))],
        }]);
        let CellSnapshot::Line { series, .. } = &st.snapshots()[0].spec else {
            panic!()
        };
        assert_eq!(series.len(), 3);
        assert_eq!(series[0].style.color, ColorSnapshot::Palette(0));
        assert_eq!(series[1].style.color, ColorSnapshot::Palette(1));
        assert_eq!(series[2].style.color, ColorSnapshot::Palette(2));
    }

    #[test]
    fn series_raw_exposes_full_data_for_redecimation() {
        let n = 10_000;
        let mut st = FigureState::new();
        st.apply(PlotOp::Line {
            series: vec![sop(data(n))],
        });
        let CellSnapshot::Line { series, .. } = &st.snapshots()[0].spec else {
            panic!()
        };
        let sid = series[0].id;
        let (rx, ry) = st.series_raw(1, 1, sid).expect("raw series");
        assert_eq!(rx.len(), n);
        assert_eq!(ry.len(), n);
        // نمایش کاسته، خام نه
        assert!(series[0].x.len() < rx.len());
        assert!(st.series_raw(99, 1, sid).is_none());
        assert!(st.series_raw(1, 1, 999).is_none());
    }

    #[test]
    fn touched_drains_once_per_figure() {
        let mut st = FigureState::new();
        st.apply(PlotOp::Line {
            series: vec![sop(data(2))],
        });
        st.apply(PlotOp::Grid { on: true });
        assert_eq!(st.take_touched(), vec![1]);
        assert!(st.take_touched().is_empty());
        st.apply(PlotOp::ActivateFigure { id: Some(2) });
        assert_eq!(st.take_touched(), vec![2]);
    }

    #[test]
    fn empty_cell_transports_as_empty_line_axes() {
        let st = apply_all(vec![PlotOp::Title {
            text: "فقط عنوان".into(),
        }]);
        let CellSnapshot::Line { series, axes, .. } = &st.snapshots()[0].spec else {
            panic!()
        };
        assert!(series.is_empty());
        assert_eq!(axes.title.as_deref(), Some("فقط عنوان"));
        assert_eq!(axes.x_limits, [0.0, 1.0]); // کادر پیش‌فرض
    }

    #[test]
    fn log_axes_get_log_ticks_and_positive_limits() {
        let st = apply_all(vec![
            PlotOp::Line {
                series: vec![sop(SeriesData {
                    x: vec![1.0, 10.0, 100.0],
                    y: vec![1.0, 10.0, 100.0],
                })],
            },
            PlotOp::AxisTight,
        ]);
        let CellSnapshot::Line { axes, .. } = &st.snapshots()[0].spec else {
            panic!()
        };
        assert!(!axes.log_x);
        // tight: بدون حاشیه
        assert!((axes.x_limits[0] - 1.0).abs() < 1e-9);
        assert!((axes.x_limits[1] - 100.0).abs() < 1e-9);
    }

    #[test]
    fn set_log_flags_flow_and_clamp_positive() {
        // semilogx: y خطی می‌ماند؛ x لگاریتمی و حدود نامثبت کف می‌خورد
        let st = apply_all(vec![
            PlotOp::Line {
                series: vec![sop(SeriesData {
                    x: vec![-5.0, 1.0, 50.0],
                    y: vec![1.0, 2.0, 3.0],
                })],
            },
            // پس از 3.5.2: SetLog بعد از Line می‌آید (fresh-axes در Line)
            PlotOp::SetLog { x: true, y: false },
        ]);
        let CellSnapshot::Line { axes, .. } = &st.snapshots()[0].spec else {
            panic!()
        };
        assert!(axes.log_x);
        assert!(!axes.log_y);
        assert!(axes.x_limits[0] > 0.0, "log x must be positive: {:?}", axes);
        // loglog: هر دو
        let st = apply_all(vec![
            PlotOp::Line {
                series: vec![sop(SeriesData {
                    x: vec![1.0, 10.0],
                    y: vec![1.0, 10.0],
                })],
            },
            PlotOp::SetLog { x: true, y: true },
        ]);
        let CellSnapshot::Line { axes, .. } = &st.snapshots()[0].spec else {
            panic!()
        };
        assert!(axes.log_x && axes.log_y);
    }

    #[test]
    fn xlim_sets_only_x_and_auto_clears() {
        // xlim دستی: y خودکار می‌ماند (داده + حاشیه)
        let st = apply_all(vec![
            PlotOp::Line {
                series: vec![sop(data(10))],
            },
            PlotOp::XLim {
                limits: Some([0.0, 1.0]),
            },
        ]);
        let CellSnapshot::Line { axes, .. } = &st.snapshots()[0].spec else {
            panic!()
        };
        assert_eq!(axes.x_limits, [0.0, 1.0]);
        // y خودکار: دادهٔ sin (بیشینه ≈0.783) + حاشیه ۵٪
        assert!(axes.y_limits[1] > 0.79, "y خودکار: {:?}", axes.y_limits);
        // xlim('auto'): بازگشت به خودکار
        let st = apply_all(vec![
            PlotOp::Line {
                series: vec![sop(data(10))],
            },
            PlotOp::XLim {
                limits: Some([0.0, 1.0]),
            },
            PlotOp::XLim { limits: None },
        ]);
        let CellSnapshot::Line { axes, .. } = &st.snapshots()[0].spec else {
            panic!()
        };
        assert!(axes.x_limits[0] < 0.0, "auto باید حاشیه بدهد: {:?}", axes);
    }

    #[test]
    fn hold_toggle_flips_the_flag() {
        let mut st = FigureState::new();
        st.apply(PlotOp::HoldToggle);
        assert!(st.active_cell_hold());
        st.apply(PlotOp::HoldToggle);
        assert!(!st.active_cell_hold());
        // toggle بعد از جایگزینی: سری دوم باید الحاق شود
        let st = apply_all(vec![
            PlotOp::Line {
                series: vec![sop(data(3))],
            },
            PlotOp::HoldToggle,
            PlotOp::Line {
                series: vec![sop(data(3))],
            },
        ]);
        let CellSnapshot::Line { series, .. } = &st.snapshots()[0].spec else {
            panic!()
        };
        assert_eq!(series.len(), 2);
    }

    #[test]
    fn custom_ticks_flow_and_empty_clears_to_auto() {
        let ticks = vec![
            Tick {
                position: 0.0,
                label: Some("a".into()),
            },
            Tick {
                position: 2.0,
                label: Some("b".into()),
            },
        ];
        let st = apply_all(vec![
            PlotOp::Line {
                series: vec![sop(data(4))],
            },
            PlotOp::XTicks {
                ticks: ticks.clone(),
            },
        ]);
        let CellSnapshot::Line { axes, .. } = &st.snapshots()[0].spec else {
            panic!()
        };
        assert_eq!(axes.x_ticks, ticks);
        assert!(axes.y_ticks.is_empty());
        // xticks([]) = پاک‌کردن به خودکار
        let st = apply_all(vec![
            PlotOp::XTicks { ticks },
            PlotOp::XTicks { ticks: Vec::new() },
        ]);
        let CellSnapshot::Line { axes, .. } = &st.snapshots()[0].spec else {
            panic!()
        };
        assert!(axes.x_ticks.is_empty());
    }
}
