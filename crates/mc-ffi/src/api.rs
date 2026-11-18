//! API عمومی mc-ffi — تولید کد Dart با `flutter_rust_bridge_codegen generate`
//! (خروجی: `app/lib/src/rust/`).
//!
//! این فایل «قرارداد نسخه‌دار» بین UI و موتور است؛ تغییر شکانده در هر نوع =
//! افزایش `API_CONTRACT_VERSION` و پیام واضح در startup (سند ۰۲ بخش ۳ بند ۴).
//!
//! **قرارداد v2 (ROADMAP 2.3 — افزایشی):** رویدادهای job-scope
//! (`ScriptEvent::Progress/Cancelled/Done`) با `run_script` stream می‌شوند؛
//! `evaluate_line` و همهٔ انواع v1 دست‌نخورده‌اند (mc-cli همان مسیر قدیم).
//! لغو با `cancel_job(job_id)` از نخ دیگر (CancellationToken سند ۰۲ §4).

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Instant;

use mc_core::{JobSignals, McError as CoreMcError, Value};
use mc_interp::{eval_program, Environment};
use mc_parser::parse;

use crate::frb_generated::StreamSink;

/// نسخه قرارداد API — UI در startup با `api_version()` مقایسه می‌کند.
/// v2: افزودن ScriptEvent + run_script/cancel_job/active_job_id (ROADMAP 2.3).
pub const API_CONTRACT_VERSION: u32 = 2;

/// نسخه قرارداد رسم (DTO) رسم‌ها — سند ۰۴ بخش ۸، سند ۰۷ بخش ۳ بند ۲.
/// v1 = پل P0؛ **v2 = فریز P3-a** (رنگ معنایی/دقیق، تیک‌های 1-2-5،
/// Area/Fill/ErrorBar، Bar سری‌محور، نرمال‌سازی هیستوگرام، شناسهٔ شکل/سری،
/// caxis/colorbar) — افزودنی از دید UI، شکانده از دید فرم (بامپ صادقانه).
pub const PLOT_CONTRACT_VERSION: u32 = 2;

// ---------------------------------------------------------------------------
// انواع قرارداد (سند ۰۲ بخش ۳ — «قرارداد داده اصلی»)
// ---------------------------------------------------------------------------

/// نتیجه یک اجرا — ساختار کامل سند ۰۲ بخش ۳.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct EvalResult {
    pub job_id: u64,
    /// متن، خطا، warning — در P0 دسته‌ای؛ streaming کامل با `runScript` در P2.
    pub console_events: Vec<ConsoleEvent>,
    /// شکل‌های دست‌خوردهٔ این اجرا (سند ۰۴ §8 — push؛ لیست کامل با get_plots).
    pub plots: Vec<PlotFigure>,
    /// فقط متغیرهای تغییرکرده — نه کل workspace (سند ۰۲ بخش ۳).
    pub workspace_delta: WorkspaceDelta,
    pub status: JobStatus,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConsoleEvent {
    Print { text: String },
    Error { err: McError },
    Warning { code: String, text: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum JobStatus {
    #[default]
    Done,
    Cancelled,
    Failed,
}

/// خطای ساختاریافته — فیلدها دقیقاً سند ۰۲ بخش ۳.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct McError {
    /// مثل `MC:E3001_PARSE_ERROR` (کاتالوگ سند ۰۳ بخش ۱۴).
    pub id: String,
    pub line: u32,
    pub col: u32,
    pub message_en: String,
    pub message_fa: String,
    /// did-you-mean (سند ۰۲ بخش ۹: Levenshtein ≤ ۲).
    pub suggestion: Option<String>,
}

impl From<CoreMcError> for McError {
    fn from(e: CoreMcError) -> Self {
        Self {
            id: e.id,
            line: e.line,
            col: e.col,
            message_en: e.message_en,
            message_fa: e.message_fa,
            suggestion: e.suggestion,
        }
    }
}

/// خلاصه یک متغیر برای بازرس متغیرها (سند ۰۲ بخش ۳).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WorkspaceDelta {
    pub changed: Vec<VarSummary>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VarSummary {
    pub name: String,
    pub class: String,
    /// شکل [rows, cols] — سمانتیک متلب.
    pub shape: Vec<u32>,
    /// حداکثر ۲۵۶ مقدار اول (column-major) — نمای کامل lazy می‌آید (سند ۰۲ بخش ۳).
    pub preview: Vec<f64>,
}

impl VarSummary {
    fn from_value(name: &str, value: &Value) -> Self {
        // char: کلاس و شکل واقعی گزارش می‌شود؛ پیش‌نمایش عددی ندارد (ROADMAP 2.8)
        if let Value::Char(s) = value {
            return Self {
                name: name.to_owned(),
                class: value.class_name().to_owned(),
                shape: vec![1, s.chars().count() as u32],
                preview: Vec::new(),
            };
        }
        // struct: ۱×۱ بدون پیش‌نمایش عددی (ROADMAP 2.9 — الگوی char/handle)
        if let Value::Struct(_) = value {
            return Self {
                name: name.to_owned(),
                class: "struct".to_owned(),
                shape: vec![1, 1],
                preview: Vec::new(),
            };
        }
        let m = match value {
            Value::Double(m) => m,
            // هندل پیش‌نمایش عددی ندارد — فقط کلاس گزارش می‌شود (سند ۰۲ §3)
            _ => {
                return Self {
                    name: name.to_owned(),
                    class: value.class_name().to_owned(),
                    shape: vec![0],
                    preview: Vec::new(),
                }
            }
        };
        let preview_len = m.numel().min(256);
        Self {
            name: name.to_owned(),
            class: value.class_name().to_owned(),
            shape: vec![m.rows() as u32, m.cols() as u32],
            preview: m.as_column_slice()[..preview_len].to_vec(),
        }
    }
}

// ---------------------------------------------------------------------------
// قرارداد رسم — فریز v2 (P3-a، سند ۰۴ §8؛ v1 = پل P0)
// ---------------------------------------------------------------------------

/// یک شکل قابل‌انتقال — شناسه‌دار برای pull (get_plots) و به‌روزرسانی
/// هدفمند در UI؛ شناسهٔ سری‌ها هم پایهٔ درخواست re-decimation زوم است.
#[derive(Debug, Clone, PartialEq)]
pub struct PlotFigure {
    pub figure_id: u64,
    pub spec: PlotSpec,
}

/// enum بستهٔ رسم‌ها (سند ۰۴ §8.1 — فریز v2) — «داده در Rust، ترسیم در Dart».
#[derive(Debug, Clone, PartialEq)]
pub enum PlotSpec {
    Line2D {
        series: Vec<Series2D>,
        axes: Axes2D,
        legend: Option<Legend>,
    },
    Scatter2D {
        series: Vec<Series2D>,
        axes: Axes2D,
        legend: Option<Legend>,
    },
    /// میله‌ای — چند سری = گروهی/انباشته (آفست‌های گروه در رندرر).
    Bar2D {
        series: Vec<Series2D>,
        /// عرض نسبی میله (0..1؛ پیش‌فرض 0.8).
        width: f32,
        /// barh — افقی.
        horizontal: bool,
        /// انباشته (در غیر این صورت گروهی).
        stacked: bool,
        axes: Axes2D,
        legend: Option<Legend>,
    },
    /// هیستوگرام — یال‌ها/شمارش‌های از پیش محاسبه‌شده + نرمال‌سازی.
    Histogram {
        bin_edges: Vec<f64>,
        counts: Vec<f64>,
        norm: HistNorm,
        /// استایل میله‌ها (رنگ وجه از پالت).
        style: LineStyle,
        axes: Axes2D,
        legend: Option<Legend>,
    },
    /// سطح زیر خط با خط مبنا (area).
    Area2D {
        series: Vec<Series2D>,
        baseline: f64,
        axes: Axes2D,
        legend: Option<Legend>,
    },
    /// چندضلعی‌های بستهٔ توپر (fill) — وجه = رنگ استایل، لبه = خط استایل.
    Fill2D {
        series: Vec<Series2D>,
        axes: Axes2D,
    },
    /// پلار — x = θ (رادیان)، y = شعاع؛ شبکهٔ پلار در رندرر.
    Polar {
        series: Vec<Series2D>,
        legend: Option<Legend>,
    },
    /// سری با نوار خطا (errorbar).
    ErrorBar2D {
        series: Vec<ErrorSeries2D>,
        axes: Axes2D,
        legend: Option<Legend>,
    },
    /// میدان برداری (P3-b — رزرو).
    Quiver {
        x: Vec<f64>,
        y: Vec<f64>,
        u: Vec<f64>,
        v: Vec<f64>,
        axes: Axes2D,
    },
    /// سطح ۳بعدی (P3-b — رزرو؛ داده‌ها از همین حالا قراردادند).
    Surface {
        x: Vec<f64>,
        y: Vec<f64>,
        z: Vec<Vec<f64>>,
        shading: Shading,
        colormap: Colormap,
        /// بازهٔ رنگ صریح (caxis) — None = خودکار از z.
        caxis: Option<Limits>,
        colorbar: bool,
    },
    Mesh {
        x: Vec<f64>,
        y: Vec<f64>,
        z: Vec<Vec<f64>>,
        colormap: Colormap,
        caxis: Option<Limits>,
        colorbar: bool,
    },
    Contour {
        x: Vec<f64>,
        y: Vec<f64>,
        z: Vec<Vec<f64>>,
        levels: Vec<f64>,
        colormap: Colormap,
        caxis: Option<Limits>,
        colorbar: bool,
    },
    Plot3 {
        series: Vec<Series3D>,
        axes: Axes3D,
    },
    Subplot {
        rows: u32,
        cols: u32,
        cells: Vec<SubplotCell>,
    },
}

/// سری ۲بعدی — نمایش LTTB-کاسته (≤ 4000 نقطه)؛ `raw_len` = تعداد نقاط
/// دادهٔ خامِ مانده در Rust (re-decimation زوم از دادهٔ کامل).
#[derive(Debug, Clone, PartialEq)]
pub struct Series2D {
    /// شناسهٔ پایدار سری (نشست) — هدف درخواست re-decimation.
    pub id: u64,
    pub raw_len: u64,
    pub x: Vec<f64>,
    pub y: Vec<f64>,
    pub style: LineStyle,
}

/// سری errorbar — نوارهای قرینه/ناقرینهٔ هر محور.
#[derive(Debug, Clone, PartialEq)]
pub struct ErrorSeries2D {
    pub id: u64,
    pub raw_len: u64,
    pub x: Vec<f64>,
    pub y: Vec<f64>,
    pub x_err: Option<ErrorBand>,
    pub y_err: Option<ErrorBand>,
    pub style: LineStyle,
}

/// نوار خطا — جفت قرینه (neg/pos؛ طول‌ها ≥ 0، هم‌طول سری).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ErrorBand {
    pub neg: Vec<f64>,
    pub pos: Vec<f64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Series3D {
    pub x: Vec<f64>,
    pub y: Vec<f64>,
    pub z: Vec<f64>,
    pub style: LineStyle,
}

/// استایل حل‌شده — رنگ معنایی (پالت، resolve در Dart طبق تم) یا دقیق.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LineStyle {
    pub color: PlotColor,
    /// ضخامت dp (پیش‌فرض ۲ — سند ۰۶ §3).
    pub width: f32,
    pub dash: DashStyle,
    pub marker: Marker,
}

/// رنگ قراردادی — دو لایه (حرفه‌ای‌شدن بدون بازنویسی):
/// پالت = توکن تم‌محور (رنگ خودکار سری‌ها)؛ Rgba = مقدار دقیق (LineSpec متلب).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlotColor {
    /// 0..=7 — سیکل پالت سند ۰۶ §3.
    Palette(u8),
    Rgba {
        r: u8,
        g: u8,
        b: u8,
        a: u8,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DashStyle {
    #[default]
    Solid,
    Dashed,
    Dotted,
    DashDot,
    /// بدون خط — فقط مارکر (پارتی `'b*'`).
    Invisible,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Marker {
    #[default]
    None,
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

/// محورهای ۲بعدی حل‌شده — حدود مؤثر + تیک‌های 1-2-5 با برچسب `%.4g`
/// (سند ۰۴ §8.1: تیک‌ها در Rust تولید می‌شوند تا Dart فقط بکشد).
#[derive(Debug, Clone, PartialEq)]
pub struct Axes2D {
    pub title: Option<String>,
    pub xlabel: Option<String>,
    pub ylabel: Option<String>,
    pub grid: bool,
    pub x_limits: Option<Limits>,
    pub y_limits: Option<Limits>,
    pub equal_aspect: bool,
    pub log_x: bool,
    pub log_y: bool,
    /// `axis off` — تزئینات محور مخفی، داده می‌ماند.
    pub visible: bool,
    pub x_ticks: Vec<Tick>,
    pub y_ticks: Vec<Tick>,
}

impl Default for Axes2D {
    fn default() -> Self {
        Self {
            title: None,
            xlabel: None,
            ylabel: None,
            grid: false,
            x_limits: None,
            y_limits: None,
            equal_aspect: false,
            log_x: false,
            log_y: false,
            visible: true,
            x_ticks: Vec::new(),
            y_ticks: Vec::new(),
        }
    }
}

/// یک تیک — `label: None` = فرعی (minor) بدون برچسب؛ برچسب‌های لگاریتمی
/// به شکل `10^{k}` (TeX سبک؛ رندر superscript در Dart).
#[derive(Debug, Clone, PartialEq)]
pub struct Tick {
    pub position: f64,
    pub label: Option<String>,
}

/// نرمال‌سازی هیستوگرام (پارتی 'Normalization' متلب).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HistNorm {
    #[default]
    Count,
    Pdf,
    Cdf,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Axes3D {
    pub title: Option<String>,
    pub xlabel: Option<String>,
    pub ylabel: Option<String>,
    pub zlabel: Option<String>,
    pub grid: bool,
}

/// حدود محور — جفت min/max.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Limits {
    pub min: f64,
    pub max: f64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Legend {
    pub entries: Vec<String>,
    pub position: LegendPosition,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LegendPosition {
    #[default]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Shading {
    Faceted,
    #[default]
    Flat,
    Interp,
}

/// colormap پیش‌فرض parula (بازسازی‌شده) — سند ۰۴ بخش ۸.۳.
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

#[derive(Debug, Clone, PartialEq)]
pub struct SubplotCell {
    pub row: u32,
    pub col: u32,
    pub spec: PlotSpec,
}

// ---------------------------------------------------------------------------
// موتور نشست‌محور — سمانتیک سریالی (سند ۰۲ بخش ۴)
// ---------------------------------------------------------------------------

struct Engine {
    env: Environment,
    jobs: u64,
    /// حالت شکل‌های نشست — fold روی PlotOpهای هر ارزیابی (P3-a).
    figures: mc_plot::figure::FigureState,
}

static ENGINE: OnceLock<Mutex<Engine>> = OnceLock::new();

fn engine() -> &'static Mutex<Engine> {
    ENGINE.get_or_init(|| {
        Mutex::new(Engine {
            env: Environment::new(),
            jobs: 0,
            figures: mc_plot::figure::FigureState::new(),
        })
    })
}

/// job فعال فعلی — فقط صاحب قفل موتور آن را ست/پاک می‌کند؛ `cancel_job`
/// بدون قفل موتور (فقط پرچم اتمیک) تا از نخ دیگر قابل فراخوانی باشد.
static ACTIVE_JOB: Mutex<Option<(u64, Arc<JobSignals>)>> = Mutex::new(None);

fn set_active(job_id: u64, ctx: &Arc<JobSignals>) {
    *ACTIVE_JOB
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = Some((job_id, Arc::clone(ctx)));
}

fn clear_active() {
    *ACTIVE_JOB
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
}

/// گارد کانتکست job — حتی در panic (گرفته‌شده با catch_unwind) کانتکست پاک
/// می‌ماند؛ invariant «یک job فعال» هرگز نشت نمی‌کند.
struct JobGuard {
    ctx: Arc<JobSignals>,
}

impl JobGuard {
    fn new(job_id: u64) -> Self {
        let ctx = Arc::new(JobSignals::new(job_id));
        mc_core::job::install(Arc::clone(&ctx));
        set_active(job_id, &ctx);
        Self { ctx }
    }
}

impl Drop for JobGuard {
    fn drop(&mut self) {
        mc_core::job::clear();
        clear_active();
    }
}

/// نسخه قرارداد — UI در startup چک می‌کند (سند ۰۲ بخش ۳ بند ۴).
pub fn api_version() -> u32 {
    API_CONTRACT_VERSION
}

/// نسخه قرارداد رسم‌ها.
pub fn plot_contract_version() -> u32 {
    PLOT_CONTRACT_VERSION
}

/// هستهٔ ارزیابی مشترک evaluate_line/run_script — پارس + ارزیابی + نگاشت
/// رویدادها. لغو (`MC:E4002_CANCELLED`) به `JobStatus::Cancelled` نگاشت
/// می‌شود (نه Failed) — سمانتیک stop متلب (ROADMAP 2.3).
fn eval_core(
    env: &mut Environment,
    code: &str,
    figures: &mut mc_plot::figure::FigureState,
) -> (Vec<ConsoleEvent>, JobStatus, Vec<PlotFigure>) {
    match parse(code) {
        Err(e) => (
            vec![ConsoleEvent::Error { err: e.into() }],
            JobStatus::Failed,
            Vec::new(),
        ),
        Ok(program) => {
            let outcome = eval_program(&program, env);
            // عملیات‌های رسم این اجرا → حالت شکل‌ها (متلب: نمودارهای پیش از
            // خطا می‌مانند)؛ خروجی push = فقط شکل‌های دست‌خورده.
            for op in outcome.plot_ops {
                figures.apply(op);
            }
            let touched = figures.take_touched();
            let mut events: Vec<ConsoleEvent> = outcome
                .outputs
                .into_iter()
                .map(|o| ConsoleEvent::Print { text: o.text })
                .collect();
            let status = match outcome.error {
                None => JobStatus::Done,
                Some(err) => {
                    let cancelled = err.id == mc_core::ids::CANCELLED;
                    events.push(ConsoleEvent::Error { err: err.into() });
                    if cancelled {
                        JobStatus::Cancelled
                    } else {
                        JobStatus::Failed
                    }
                }
            };
            let plots = figures
                .snapshots()
                .into_iter()
                .filter(|f| touched.contains(&f.figure_id))
                .map(map_figure)
                .collect();
            (events, status, plots)
        }
    }
}

/// نگاشت payload پنیک به خطای ساختاریافته (بند ۱۵ ممیزی خارجی).
fn panic_error(payload: Box<dyn std::any::Any + Send>) -> mc_core::McError {
    let msg = if let Some(s) = payload.downcast_ref::<&str>() {
        (*s).to_string()
    } else if let Some(s) = payload.downcast_ref::<String>() {
        s.clone()
    } else {
        "unknown panic".to_string()
    };
    mc_core::McError::new(
        mc_core::ids::UNSUPPORTED,
        0,
        0,
        format!("Internal engine panic — please report this input. ({msg})"),
        format!("پنیک داخلی موتور — لطفاً این ورودی را گزارش کنید. ({msg})"),
    )
}

/// ارزیابی یک خط کنسول — دمو رسمی P0 (سند ۰۷):
/// `evaluateLine("1+1")` از UI فارسی تا Rust و برگشت، با event های تایپ‌دار.
///
/// سمانتیک متلب: اجرای دستورها به ترتیب، توقف در اولین خطا؛ خروجی‌های
/// دستورهای قبلی حفظ می‌شوند (سند ۰۲ بخش ۹).
///
/// v2: کانتکست job هم نصب می‌شود (لغو/پیشرفت یکنواخت با run_script)؛
/// رفتار v1 کاملاً حفظ شده — mc-cli بدون تغییر سبز می‌ماند.
pub fn evaluate_line(code: String) -> EvalResult {
    let t0 = Instant::now();
    let mut engine = engine()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    engine.jobs += 1;
    let job_id = engine.jobs;
    let before: HashMap<String, Value> = engine.env.snapshot();

    // گارد مرزی: هر panic داخلی موتور هرگز نباید از مرز FFI بگذرد (کرش اپ
    // اندروید). به خطای موتور نگاشت می‌شود؛ وضعیت env بعد از panic ناسازگار
    // ممکن است بماند — نشست باید ادامه یابد (بند ۱۵ گزارش ممیزی خارجی).
    let _guard = JobGuard::new(job_id);
    let (mut events, status, plots) =
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            {
                // یک deref صریح تا دو قرضِ گسسته از Engine ممکن شود (MutexGuard)
                let eng = &mut *engine;
                eval_core(&mut eng.env, &code, &mut eng.figures)
            }
        })) {
            Ok(triple) => triple,
            Err(payload) => (
                vec![ConsoleEvent::Error {
                    err: panic_error(payload).into(),
                }],
                JobStatus::Failed,
                Vec::new(),
            ),
        };

    // هیچ رویدادی گم نمی‌شود: اگر خطا وسط خط بود، خروجی‌های قبل از خطا چاپ شده‌اند.
    if status == JobStatus::Failed && events.is_empty() {
        events.push(ConsoleEvent::Print {
            text: String::new(),
        });
    }

    let workspace_delta = build_delta(&before, &engine.env);
    EvalResult {
        job_id,
        console_events: events,
        plots,
        workspace_delta,
        status,
        duration_ms: u64::try_from(t0.elapsed().as_millis()).unwrap_or(u64::MAX),
    }
}

// ---------------------------------------------------------------------------
// قرارداد v2 — FFI رویدادی: progress + cancel (ROADMAP 2.3، سند ۰۲ §3–4)
// ---------------------------------------------------------------------------

/// رویدادهای جریان یک job — payload سمت Dartِ `runScript`.
/// در FRB 2.12 پارامتر `StreamSink` از امضای Dart حذف و تابع به
/// `Stream<ScriptEvent> runScript({required String code})` تبدیل می‌شود؛
/// مقدار بازگشتی Rust فقط برای تست‌های داخلی است (Dart آن را نمی‌بیند) —
/// نتیجهٔ نهایی از طریق [`ScriptEvent::Done`] می‌آید.
#[derive(Debug, Clone, PartialEq)]
pub enum ScriptEvent {
    /// پیشرفت حلگرهای طولانی (ode45/ode23/integral). `percent` = None یعنی
    /// نامعین (نوار نامعین UI)؛ در غیر این صورت 0.0..=1.0.
    Progress {
        job_id: u64,
        stage: String,
        percent: Option<f32>,
    },
    /// درخواست لغو پذیرفته شد — پیش از Done صادر می‌شود (UX فوری).
    Cancelled { job_id: u64 },
    /// پایان job (مسیر پایان یکتا) — نتیجهٔ کامل v1 (همان EvalResult).
    Done { result: EvalResult },
}

/// اجرای زندهٔ یک خط/اسکریپت با رویدادهای پیشرفت — مسیر UI گام 2.3.
///
/// - قفل موتور کل اجرا را می‌پوشاند (سمانتیک سریالی سند ۰۲ §4)؛
/// - نخ ticker هر ~100ms اسنپ‌شات کانتکست job را به صورت
///   `ScriptEvent::Progress` (فقط در تغییر) پخش می‌کند؛
/// - در لغو: `ScriptEvent::Cancelled` سپس `Done{result: status=Cancelled}`؛
/// - stream با پایان تابع بسته می‌شود (Drop کلوزر FRB) — مسیر پایان یکتا.
pub fn run_script(code: String, sink: StreamSink<ScriptEvent>) -> EvalResult {
    let t0 = Instant::now();
    let mut engine = engine()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    engine.jobs += 1;
    let job_id = engine.jobs;
    let before: HashMap<String, Value> = engine.env.snapshot();
    let guard = JobGuard::new(job_id);

    // ticker پیشرفت — park_timeout برای خروج فوری در پایان job
    let done = Arc::new(AtomicBool::new(false));
    let ticker = {
        let sink = sink.clone();
        let ctx = Arc::clone(&guard.ctx);
        let done = Arc::clone(&done);
        std::thread::Builder::new()
            .name("mc-progress".to_owned())
            .spawn(move || {
                let mut last_stage = String::new();
                let mut last_bp = mc_core::INDETERMINATE_BP;
                loop {
                    if done.load(Ordering::Relaxed) {
                        break;
                    }
                    let snap = ctx.snapshot();
                    if snap.stage != last_stage || snap.percent_bp != last_bp {
                        let _ = sink.add(ScriptEvent::Progress {
                            job_id,
                            stage: snap.stage.clone(),
                            percent: snap.fraction().map(|f| f as f32),
                        });
                        last_stage = snap.stage;
                        last_bp = snap.percent_bp;
                    }
                    std::thread::park_timeout(std::time::Duration::from_millis(100));
                }
            })
            .ok() // بدون ticker هم پیشرفت از دست نمی‌رود؛ نتیجهٔ نهایی سالم است
    };

    let (mut events, status, plots) =
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            {
                // یک deref صریح تا دو قرضِ گسسته از Engine ممکن شود (MutexGuard)
                let eng = &mut *engine;
                eval_core(&mut eng.env, &code, &mut eng.figures)
            }
        })) {
            Ok(triple) => triple,
            Err(payload) => (
                vec![ConsoleEvent::Error {
                    err: panic_error(payload).into(),
                }],
                JobStatus::Failed,
                Vec::new(),
            ),
        };

    // هیچ رویدادی گم نمی‌شود (همان قاعدهٔ v1)
    if status == JobStatus::Failed && events.is_empty() {
        events.push(ConsoleEvent::Print {
            text: String::new(),
        });
    }

    done.store(true, Ordering::Relaxed);
    if let Some(t) = ticker {
        t.thread().unpark();
        let _ = t.join();
    }
    drop(guard); // کانتکست/رکورد فعال پیش از رویداد Done پاک شود

    if status == JobStatus::Cancelled {
        let _ = sink.add(ScriptEvent::Cancelled { job_id });
    }
    let result = EvalResult {
        job_id,
        console_events: events,
        plots,
        workspace_delta: build_delta(&before, &engine.env),
        status,
        duration_ms: u64::try_from(t0.elapsed().as_millis()).unwrap_or(u64::MAX),
    };
    let _ = sink.add(ScriptEvent::Done {
        result: result.clone(),
    });
    result
}

/// درخواست لغو job فعال — بدون قفل موتور (از نخ FRB دیگر امن است).
/// true = یک job فعال با این id پیدا و پرچمش ست شد.
pub fn cancel_job(job_id: u64) -> bool {
    let active = ACTIVE_JOB
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    match active.as_ref() {
        Some((id, ctx)) if *id == job_id => {
            ctx.cancel.store(true, Ordering::Relaxed);
            true
        }
        _ => false,
    }
}

/// شناسهٔ job در حال اجرا (برای دکمهٔ Stop: پرس‌وجو + cancel) — None = بی‌کار.
pub fn active_job_id() -> Option<u64> {
    ACTIVE_JOB
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .as_ref()
        .map(|(id, _)| *id)
}

/// شمارش گذرهای چک‌پوینت حلقهٔ job فعال (هر تکرار for/while) — فقط برای
/// تست‌های داخل کرِیت: تست لغو تا ≥۱ گذرِ تثبیت‌شده صبر می‌کند تا نتیجه
/// قطعی باشد (رفع CI قرمز ۲۰۲۶-۱۱-۱۷ — رقابت «ثبت job» با «نخستین کامیت»).
/// `#[cfg(test)]` + خارج از قرارداد FFI (codegen فقط pub غیرِ‌شرطی را
/// می‌بیند؛ بدون تغییر API_CONTRACT_VERSION).
#[cfg(test)]
pub(crate) fn active_job_loop_passes() -> Option<u64> {
    ACTIVE_JOB
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .as_ref()
        .map(|(_, ctx)| ctx.loop_passes.load(Ordering::Relaxed))
}

/// لیست کامل شکل‌های نشست (مدل pull سند ۰۲ §7) — UI هر زمان می‌تواند کل
/// شکل‌ها را بگیرد؛ push برای به‌روزرسانی = `EvalResult.plots` همین اجرا.
pub fn get_plots() -> Vec<PlotFigure> {
    let engine = engine()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    engine
        .figures
        .snapshots()
        .into_iter()
        .map(map_figure)
        .collect()
}

// ----- نگاشت snapshot موتور → DTO انتقالی (mc-plot → قرارداد v2) -----

use mc_plot::figure::{
    AxesSnapshot, CellSnapshot, ColorSnapshot, ErrSeriesSnapshot, FigureSnapshot, SeriesSnapshot,
    StyleSnapshot,
};
use mc_plot::histogram::HistNorm as CoreHistNorm;
use mc_plot::spec::{DashSpec, MarkerSpec};
use mc_plot::ticks::Tick as CoreTick;

fn map_figure(f: FigureSnapshot) -> PlotFigure {
    PlotFigure {
        figure_id: f.figure_id,
        spec: map_cell(f.spec),
    }
}

fn map_cell(c: CellSnapshot) -> PlotSpec {
    match c {
        CellSnapshot::Line {
            series,
            axes,
            legend,
        } => PlotSpec::Line2D {
            series: series.into_iter().map(map_series).collect(),
            axes: map_axes(axes),
            legend: legend.map(map_legend),
        },
        CellSnapshot::Scatter {
            series,
            axes,
            legend,
        } => PlotSpec::Scatter2D {
            series: series.into_iter().map(map_series).collect(),
            axes: map_axes(axes),
            legend: legend.map(map_legend),
        },
        CellSnapshot::Bar {
            series,
            width,
            horizontal,
            stacked,
            axes,
            legend,
        } => PlotSpec::Bar2D {
            series: series.into_iter().map(map_series).collect(),
            width,
            horizontal,
            stacked,
            axes: map_axes(axes),
            legend: legend.map(map_legend),
        },
        CellSnapshot::Histogram {
            edges,
            counts,
            norm,
            style,
            axes,
            legend,
        } => PlotSpec::Histogram {
            bin_edges: edges,
            counts,
            norm: map_hist_norm(norm),
            style: map_style(style),
            axes: map_axes(axes),
            legend: legend.map(map_legend),
        },
        CellSnapshot::Area {
            series,
            baseline,
            axes,
            legend,
        } => PlotSpec::Area2D {
            series: series.into_iter().map(map_series).collect(),
            baseline,
            axes: map_axes(axes),
            legend: legend.map(map_legend),
        },
        CellSnapshot::Fill { series, axes } => PlotSpec::Fill2D {
            series: series.into_iter().map(map_series).collect(),
            axes: map_axes(axes),
        },
        CellSnapshot::Polar { series, legend } => PlotSpec::Polar {
            series: series.into_iter().map(map_series).collect(),
            legend: legend.map(map_legend),
        },
        CellSnapshot::ErrorBar {
            series,
            axes,
            legend,
        } => PlotSpec::ErrorBar2D {
            series: series.into_iter().map(map_err_series).collect(),
            axes: map_axes(axes),
            legend: legend.map(map_legend),
        },
        CellSnapshot::Subplot { rows, cols, cells } => PlotSpec::Subplot {
            rows,
            cols,
            cells: cells
                .into_iter()
                .map(|sc| SubplotCell {
                    row: sc.row,
                    col: sc.col,
                    spec: map_cell(sc.spec),
                })
                .collect(),
        },
    }
}

fn map_series(s: SeriesSnapshot) -> Series2D {
    Series2D {
        id: s.id,
        raw_len: s.raw_len,
        x: s.x,
        y: s.y,
        style: map_style(s.style),
    }
}

fn map_err_series(s: ErrSeriesSnapshot) -> ErrorSeries2D {
    ErrorSeries2D {
        id: s.id,
        raw_len: s.raw_len,
        x: s.x,
        y: s.y,
        x_err: s.x_err.map(map_err_band),
        y_err: s.y_err.map(map_err_band),
        style: map_style(s.style),
    }
}

fn map_err_band(b: mc_plot::spec::ErrBand) -> ErrorBand {
    ErrorBand {
        neg: b.neg,
        pos: b.pos,
    }
}

fn map_style(st: StyleSnapshot) -> LineStyle {
    LineStyle {
        color: map_color(st.color),
        width: st.width,
        dash: match st.dash {
            DashSpec::Solid => DashStyle::Solid,
            DashSpec::Dashed => DashStyle::Dashed,
            DashSpec::Dotted => DashStyle::Dotted,
            DashSpec::DashDot => DashStyle::DashDot,
            DashSpec::Invisible => DashStyle::Invisible,
        },
        marker: match st.marker {
            None => Marker::None,
            Some(MarkerSpec::Point) => Marker::Point,
            Some(MarkerSpec::Circle) => Marker::Circle,
            Some(MarkerSpec::Square) => Marker::Square,
            Some(MarkerSpec::Diamond) => Marker::Diamond,
            Some(MarkerSpec::Cross) => Marker::Cross,
            Some(MarkerSpec::Plus) => Marker::Plus,
            Some(MarkerSpec::Star) => Marker::Star,
            Some(MarkerSpec::TriangleUp) => Marker::TriangleUp,
            Some(MarkerSpec::TriangleDown) => Marker::TriangleDown,
        },
    }
}

fn map_color(c: ColorSnapshot) -> PlotColor {
    match c {
        ColorSnapshot::Palette(i) => PlotColor::Palette(i),
        ColorSnapshot::Rgb { r, g, b } => PlotColor::Rgba { r, g, b, a: 255 },
    }
}

fn map_axes(a: AxesSnapshot) -> Axes2D {
    Axes2D {
        title: a.title,
        xlabel: a.xlabel,
        ylabel: a.ylabel,
        grid: a.grid,
        x_limits: Some(Limits {
            min: a.x_limits[0],
            max: a.x_limits[1],
        }),
        y_limits: Some(Limits {
            min: a.y_limits[0],
            max: a.y_limits[1],
        }),
        equal_aspect: a.equal_aspect,
        log_x: a.log_x,
        log_y: a.log_y,
        visible: a.visible,
        x_ticks: a.x_ticks.into_iter().map(map_tick).collect(),
        y_ticks: a.y_ticks.into_iter().map(map_tick).collect(),
    }
}

fn map_tick(t: CoreTick) -> Tick {
    Tick {
        position: t.position,
        label: t.label,
    }
}

fn map_hist_norm(n: CoreHistNorm) -> HistNorm {
    match n {
        CoreHistNorm::Count => HistNorm::Count,
        CoreHistNorm::Pdf => HistNorm::Pdf,
        CoreHistNorm::Cdf => HistNorm::Cdf,
    }
}

fn map_legend(l: mc_plot::figure::LegendSnapshot) -> Legend {
    Legend {
        entries: l.entries,
        position: match l.position {
            mc_plot::spec::LegendPos::Best => LegendPosition::Best,
            mc_plot::spec::LegendPos::North => LegendPosition::North,
            mc_plot::spec::LegendPos::South => LegendPosition::South,
            mc_plot::spec::LegendPos::East => LegendPosition::East,
            mc_plot::spec::LegendPos::West => LegendPosition::West,
            mc_plot::spec::LegendPos::NorthEast => LegendPosition::NorthEast,
            mc_plot::spec::LegendPos::NorthWest => LegendPosition::NorthWest,
            mc_plot::spec::LegendPos::SouthEast => LegendPosition::SouthEast,
            mc_plot::spec::LegendPos::SouthWest => LegendPosition::SouthWest,
        },
    }
}

/// فهرست کامل متغیرهای فعلی نشست (مرتب بر اساس نام) — تب «متغیرها»ی UI و
/// `whos` کنسول ترمینال (توابع الحاقی P0.1 — سند ۰۲ بخش ۳).
pub fn workspace_vars() -> Vec<VarSummary> {
    let engine = engine()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let mut vars: Vec<VarSummary> = engine
        .env
        .iter()
        .map(|(name, value)| VarSummary::from_value(name, value))
        .collect();
    vars.sort_by(|a, b| a.name.cmp(&b.name));
    vars
}

/// پاک‌کردن همه متغیرهای نشست (سمانتیک `clear` متلب) — تعداد متغیرهای پاک‌شده.
pub fn clear_workspace() -> usize {
    let mut engine = engine()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    engine.env.clear()
}

// ---------------------------------------------------------------------------
// فایل‌های .m — توابع و اسکریپت‌ها (سند ۰۳ §9؛ آیتم‌های DoD P1: #2 و #9)
// الحاقی غیر‌شکننده: قرارداد v1 دست‌نخورده می‌ماند (فقط افزودن)
// ---------------------------------------------------------------------------

/// گزارش ثبت یک منبع .m.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct RegisterResult {
    /// نام توابع تعریف‌شده به ترتیب تعریف (اولی = تابع اصلی).
    pub functions: Vec<String>,
    /// بخش اسکریپتی فایل ثبت شد (با نام فایل فراخوانی‌پذیر است).
    pub script: bool,
    /// در صورت خطا: ساختاریافته (کد/موقعیت/دوزبانه).
    pub error: Option<McError>,
}

/// ثبت منبع یک فایل .m (بدون پسوند) در مخزن نشست — توابع فوراً
/// فراخوانی‌پذیر می‌شوند؛ اسکریپت با نامش اجرا می‌شود (سمانتیک متلب §9).
pub fn register_m_source(name: String, source: String) -> RegisterResult {
    let mut engine = engine()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    match mc_interp::register_source(&mut engine.env, &name, &source) {
        Ok(r) => RegisterResult {
            functions: r.functions,
            script: r.script_registered,
            error: None,
        },
        Err(e) => RegisterResult {
            functions: Vec::new(),
            script: false,
            error: Some(e.into()),
        },
    }
}

/// خلاصه یک تابع/اسکریپت ثبت‌شده — برای فهرست تعریف‌ها در UI و `whos`.
#[derive(Debug, Clone, PartialEq)]
pub struct MFunctionSummary {
    pub name: String,
    /// "function" یا "script" — رشته ساده برای قرارداد پایدار.
    pub kind: String,
    /// تعداد خروجی‌های تعریف‌شده (اسکریپت = ۰).
    pub outputs: u32,
    /// حداکثر آرگومان‌های ورودی (اسکریپت = ۰).
    pub inputs: u32,
}

/// فهرست همه توابع/اسکریپت‌های ثبت‌شده — مرتب بر اساس نام.
pub fn m_functions() -> Vec<MFunctionSummary> {
    let engine = engine()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let mut out: Vec<MFunctionSummary> = engine
        .env
        .function_names()
        .into_iter()
        .filter_map(|name| match engine.env.lookup_item(&name) {
            Some(mc_interp::env::StoredItem::Fn(def)) => Some(MFunctionSummary {
                name,
                kind: "function".to_owned(),
                outputs: def.outputs.len() as u32,
                inputs: def.params.len() as u32,
            }),
            Some(mc_interp::env::StoredItem::Script(_)) => Some(MFunctionSummary {
                name,
                kind: "script".to_owned(),
                outputs: 0,
                inputs: 0,
            }),
            None => None,
        })
        .collect();
    out.sort_by(|a, b| a.name.cmp(&b.name));
    out
}

/// پاک‌کردن همه تعریف‌های ثبت‌شده (سمانتیک `clear functions` متلب) — تعداد پاک‌شده.
pub fn clear_m_functions() -> usize {
    let engine = engine()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    engine.env.clear_functions()
}

/// رزولوشن یک نام به سبک متلب §9 (خلاصه‌شده) — برای `which` کنسول:
/// "variable" > "function" > "script" > "builtin"؛ None = تعریف‌نشده.
pub fn which_name(name: String) -> Option<String> {
    let engine = engine()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if engine.env.get(&name).is_some() {
        return Some("variable".to_owned());
    }
    match engine.env.lookup_item(&name) {
        Some(mc_interp::env::StoredItem::Fn(_)) => return Some("function".to_owned()),
        Some(mc_interp::env::StoredItem::Script(_)) => return Some("script".to_owned()),
        None => {}
    }
    if mc_interp::registry::lookup(&name).is_some() {
        return Some("builtin".to_owned());
    }
    None
}

fn build_delta(before: &HashMap<String, Value>, env: &Environment) -> WorkspaceDelta {
    let mut changed: Vec<VarSummary> = env
        .iter()
        .filter(|(name, value)| match before.get(*name) {
            None => true,
            Some(prev) => prev != *value,
        })
        .map(|(name, value)| VarSummary::from_value(name, value))
        .collect();
    changed.sort_by(|a, b| a.name.cmp(&b.name));
    WorkspaceDelta { changed }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};
    use std::time::Duration;

    /// قفل سریال‌سازی آزمون‌های موتور — کانتکست job و رکورد فعال سراسری‌اند
    /// (invariant سند ۰۲ §4)؛ بدون این قفل، لغو cross-thread ممکن است job
    /// آزمون موازی را بگیرد.
    static SERIAL: OnceLock<Mutex<()>> = OnceLock::new();

    fn serial_guard() -> std::sync::MutexGuard<'static, ()> {
        SERIAL
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// نشست موتور جهانی است (سمانتیک سریالی سند ۰۲ بخش ۴)؛ پس آزمون‌های آن
    /// باید **ترتیبی** اجرا شوند — همه سناریوها در یک #[test] پشت سر هم.
    #[test]
    fn engine_session_end_to_end_scenarios() {
        let _guard = serial_guard();

        // مقاوم‌سازی ترتیبی (درس flake نسخهٔ CI): این سناریو رفتار end-to-end
        // را می‌سنجد، نه وراثت state از آزمون‌های دیگر پروسهٔ مشترک (مثلاً ans
        // یا متغیرهای جاگذاری‌شده). خالی‌کردن ابتدایی هیچ job مصرف نمی‌کند و
        // شمارنده job_id را هم لمس نمی‌کند — فقط env و فهرست توابع .m را.
        let _ = clear_workspace();
        let _ = clear_m_functions();

        // -- دمو رسمی P0 (سند ۰۷): 1+1 از قرارداد تا خروجی
        let r = evaluate_line("1+1".to_owned());
        assert_eq!(r.status, JobStatus::Done);
        // قرارداد سند ۰۲ §4: job_id شمارنده‌ای سراسری و اکیداً صعودی است و در
        // طول عمر پروسه reset نمی‌شود (clear فقط env را خالی می‌کند). ترتیب
        // اجرای آزمون‌ها در harness قطعی نیست و سه آزمون موتوری فقط با
        // SERIAL سریال می‌شوند — شواهد CI: اگر cancel/oom اول قفل را بگیرند،
        // «1+1» به‌جای ۱، id=۳ می‌گیرد و assert مطلق flaky می‌شود. پس
        // قراردادِ قابل‌آزمون در پروسهٔ مشترک: id تخصیص‌یافته (نامتعادل با صفر)
        // و صعودی برای هر اجرای بعدی (تحت گارد، قطعی).
        assert!(r.job_id != 0, "job id must be assigned");
        let first_job_id = r.job_id;
        assert_eq!(
            r.console_events,
            vec![ConsoleEvent::Print {
                text: "ans =\n     2
"
                .to_owned()
            }]
        );
        assert_eq!(r.workspace_delta.changed.len(), 1);
        assert_eq!(r.workspace_delta.changed[0].name, "ans");
        assert_eq!(r.workspace_delta.changed[0].preview, vec![2.0]);

        // -- خطای پارس ساختاریافته
        let r = evaluate_line("1 +".to_owned());
        assert_eq!(r.status, JobStatus::Failed);
        // شمارنده برای job ناموفق هم جلو می‌رود — هر evaluate_line دقیقاً یک
        // job؛ تحت گارد SERIAL هیچ اجرای دیگری بین این دو نمی‌آید.
        assert!(r.job_id > first_job_id);
        let [ConsoleEvent::Error { err }] = r.console_events.as_slice() else {
            panic!("error event expected");
        };
        assert_eq!(err.id, "MC:E3001_PARSE_ERROR");
        assert!(err.message_fa.contains("غیرمنتظره"));

        // -- خطای زمان اجرا: خروجی‌های قبل از خطا حفظ می‌شوند
        let r = evaluate_line("x = 3, y = z + 1".to_owned());
        assert_eq!(r.status, JobStatus::Failed);
        assert_eq!(r.console_events.len(), 2);
        assert!(matches!(
            &r.console_events[0],
            ConsoleEvent::Print { text } if text == "x =\n     3\n"
        ));
        assert!(matches!(
            &r.console_events[1],
            ConsoleEvent::Error { err } if err.id == "MC:E1001_UNDEFINED_VARIABLE"
        ));

        // -- ماندگاری workspace + delta حداقلی
        let _ = evaluate_line("k = 42".to_owned());
        let r = evaluate_line("k * 2".to_owned());
        assert_eq!(r.status, JobStatus::Done);
        assert!(r.workspace_delta.changed.iter().all(|v| v.name == "ans"));
        assert!(matches!(
            &r.console_events[0],
            ConsoleEvent::Print { text } if text == "ans =\n    84\n"
        ));

        // -- سقف پیش‌نمایش (سند ۰۲ بخش ۳: ≤ ۲۵۶ مقدار اول)
        let r = evaluate_line("bignumber = 1e9".to_owned());
        assert_eq!(r.status, JobStatus::Done);
        assert_eq!(r.workspace_delta.changed[0].preview.len(), 1);
        let r = evaluate_line("matrixish = ans * 1".to_owned());
        assert!(r.workspace_delta.changed[0].preview.len() <= 256);

        // -- ثبات نسخه‌های قرارداد (رسم: v2 از فریز P3-a)
        assert_eq!(api_version(), 2);
        assert_eq!(plot_contract_version(), 2);

        // -- بازرسی نشست (P0.1): workspace_vars مرتب و clear_workspace سمانتیک clear
        let _ = evaluate_line("zv = 9".to_owned());
        let vars = workspace_vars();
        assert!(!vars.is_empty());
        assert!(vars.windows(2).all(|w| w[0].name <= w[1].name));
        assert!(vars.iter().any(|v| v.name == "zv" && v.shape == vec![1, 1]));
        let cleared = clear_workspace();
        assert_eq!(cleared, vars.len());
        assert!(workspace_vars().is_empty());
        let r = evaluate_line("zv + 1".to_owned());
        assert_eq!(r.status, JobStatus::Failed);

        // -- فایل‌های .m (آیتم‌های DoD P1: #2 و #9) — تابع، اسکریپت، خطا، فهرست
        let r = register_m_source(
            "sq".to_owned(),
            "function y = sq(x)\ny = x^2\nend".to_owned(),
        );
        assert_eq!(r.error, None);
        assert_eq!(r.functions, vec!["sq".to_owned()]);
        assert!(!r.script);
        let r = evaluate_line("sq(9)".to_owned());
        assert_eq!(r.status, JobStatus::Done);
        assert!(matches!(
            &r.console_events[0],
            ConsoleEvent::Print { text } if text == "ans =\n    81\n"
        ));

        // اسکریپت: ثبت بدون اجرا، اجرا با نام در فضای کاری
        let r = register_m_source("setup".to_owned(), "k = 5\nj = k * 2".to_owned());
        assert_eq!(r.error, None);
        assert!(r.script);
        assert!(r.functions.is_empty());
        assert!(workspace_vars().iter().all(|v| v.name != "k"));
        let r = evaluate_line("setup".to_owned());
        assert_eq!(r.status, JobStatus::Done);
        let r = evaluate_line("j".to_owned());
        assert!(matches!(
            &r.console_events[0],
            // بند ۳ ممیزی خارجی: اکوی متغیر برهنه با «نام =» است (رفتار متلب)
            ConsoleEvent::Print { text } if text == "j =\n    10\n"
        ));

        // مغایرت نام فایل/تابع اصلی = خطای آموزشی (سمانتیک متلب)
        let r = register_m_source(
            "nope".to_owned(),
            "function y = f(x); y = x; end".to_owned(),
        );
        assert!(r.functions.is_empty() && r.error.is_some());

        // چندخروجی از طریق قرارداد evaluate_line
        let r = register_m_source(
            "stats".to_owned(),
            "function [s,p] = stats(v); s = sum(v); p = prod(v); end".to_owned(),
        );
        assert_eq!(r.error, None);
        let r = evaluate_line("[a,b] = stats([1 2 4])".to_owned());
        assert_eq!(r.status, JobStatus::Done);
        assert!(r
            .workspace_delta
            .changed
            .iter()
            .any(|v| v.name == "a" && v.preview == vec![7.0]));
        assert!(r
            .workspace_delta
            .changed
            .iter()
            .any(|v| v.name == "b" && v.preview == vec![8.0]));

        // فهرست مرتب + پاک‌کردن تعریف‌ها
        let fns = m_functions();
        let names: Vec<&str> = fns.iter().map(|f| f.name.as_str()).collect();
        assert_eq!(names, vec!["setup", "sq", "stats"]);
        let sq = fns.iter().find(|f| f.name == "sq").expect("sq");
        assert_eq!(
            (sq.kind.as_str(), sq.inputs, sq.outputs),
            ("function", 1, 1)
        );
        let cleared = clear_m_functions();
        assert_eq!(cleared, 3);
        assert!(m_functions().is_empty());
        let r = evaluate_line("sq(1)".to_owned());
        assert_eq!(r.status, JobStatus::Failed);
    }

    /// قرارداد v2 (ROADMAP 2.3): لغو cross-thread روی مسیر evaluate_line
    /// (که حالا کانتکست job نصب می‌کند) + سقوط تمیز به Cancelled + workspace
    /// consistent + رفتار cancel_job بیرون از بازهٔ فعال.
    #[test]
    fn engine_v2_cancel_cross_thread() {
        let _guard = serial_guard();

        // job بی‌نهایت در نخ جدا — قفل موتور را می‌گیرد و حلقه می‌چرخد
        let worker = std::thread::spawn(|| evaluate_line("while 1, x = 1; end".to_owned()));

        // منتظر فعال‌شدن job (حداکثر ۵ ثانیه — جلوگیری از هنگ CI)
        let mut job_id = None;
        for _ in 0..500 {
            if let Some(id) = active_job_id() {
                job_id = Some(id);
                break;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        let job_id = job_id.expect("running job must be observable via active_job_id");

        // قطعیت (درس CI قرمز ۲۰۲۶-۱۱-۱۷): «فعال‌بودن job» فقط یعنی ثبت شده؛
        // لغوِ پیش از نخستین چک‌پوینتِ حلقه = خروج با ورک‌اسپیس خالی — که
        // رفتار درست موتور است (متلب هم اگر پیش از هر گزاره stop کنید هیچ
        // نمی‌ماند). پس تا ≥۱ گذرِ تثبیت‌شدهٔ حلقه صبر می‌کنیم: آنگاه تکرار
        // اول آغاز شده و `x = 1` قطعاً کامیت می‌شود (لغو فقط مرز تکرار بعدی
        // دیده می‌شود، وسط گزاره نه) — ادعای «مثل stop متلب: x باقی می‌ماند»
        // این‌جا از قطعیت برمی‌خیزد، نه از شانس زمان‌بندی runner.
        let mut ready = false;
        for _ in 0..5_000 {
            if matches!(active_job_loop_passes(), Some(n) if n >= 1) {
                ready = true;
                break;
            }
            std::thread::sleep(Duration::from_millis(1));
        }
        assert!(
            ready,
            "engine must pass >=1 loop checkpoint before we cancel (got {:?})",
            active_job_loop_passes()
        );

        // لغو پذیرفته می‌شود؛ id ناموجود رد می‌شود
        assert!(cancel_job(job_id));
        assert!(!cancel_job(job_id + 1));

        let result = worker.join().expect("worker must not panic");
        assert_eq!(result.job_id, job_id);
        assert_eq!(result.status, JobStatus::Cancelled);
        // پیام لغو به کنسول رفت (سطح ۴ سند ۰۲ §9 — هرگز silent)
        assert!(matches!(
            result.console_events.last(),
            Some(ConsoleEvent::Error { err }) if err.id == mc_core::ids::CANCELLED
        ));
        // workspace consistent (مثل stop متلب): x با مقدار کامل باقی است
        let vars = workspace_vars();
        assert!(vars.iter().any(|v| v.name == "x" && v.shape == vec![1, 1]));

        // پس از پایان job: رکورد فعال خالی است و cancel رد می‌شود
        assert!(active_job_id().is_none());
        assert!(!cancel_job(job_id));

        // نشست پس از لغو سالم ادامه می‌یابد
        let r = evaluate_line("2 * 21".to_owned());
        assert_eq!(r.status, JobStatus::Done);
        assert!(matches!(
            &r.console_events[0],
            ConsoleEvent::Print { text } if text == "ans =\n    42\n"
        ));
    }

    /// اسنپ‌شات کانتکست/رویدادها: ScriptEvent باید بسازدنی و مقایسه‌پذیر باشد
    /// (قرارداد round-trip v2 — سمت wire با codegen + CI Dart طرف می‌شود).
    #[test]
    fn contract_v2_script_event_shapes() {
        let progress = ScriptEvent::Progress {
            job_id: 7,
            stage: "ode45".to_owned(),
            percent: Some(0.5),
        };
        assert_eq!(progress.clone(), progress);
        let cancelled = ScriptEvent::Cancelled { job_id: 7 };
        assert_ne!(progress, cancelled);
        let done = ScriptEvent::Done {
            result: EvalResult {
                job_id: 7,
                status: JobStatus::Cancelled,
                ..Default::default()
            },
        };
        assert!(
            matches!(done, ScriptEvent::Done { result } if result.status == JobStatus::Cancelled)
        );
    }

    /// ROADMAP 2.5 — بستن بند ۲ دفتر ممیزی (COMPATIBILITY §4.8): تخصیص بزرگ
    /// باید خطای MATLAB-گونه (`MC:E4003_OUT_OF_MEMORY`) بدهد، نه abort پروسه
    /// (SIGABRT خروج ۱۳۴ — تخصیصِ ناموفق با catch_unwind مهارشدنی نیست).
    /// «پروسهٔ زنده» یعنی: تست بعد از هر خطا ادامه می‌یابد و موتور همچنان
    /// به درخواست‌های بعدی جواب می‌دهد (رفتار متلب: session ادامه دارد).
    #[test]
    fn memory_budget_oom_is_clean_error_process_alive() {
        let _guard = serial_guard();
        // -- DoD گام 2.5: zeros(100000,100000); → خطای پاک (۱۰¹⁰ عنصر = ~76GB)
        let r = evaluate_line("zeros(100000,100000);".to_owned());
        assert_eq!(r.status, JobStatus::Failed);
        let [ConsoleEvent::Error { err }] = r.console_events.as_slice() else {
            panic!("error event expected");
        };
        assert_eq!(err.id, "MC:E4003_OUT_OF_MEMORY");
        assert!(err.message_en.contains("Out of memory"));
        assert!(err.message_en.contains("100000x100000"));
        assert!(err.message_en.contains("Reduce the problem size"));
        assert!(err.message_fa.contains("خارج از حافظه"));

        // -- DoD گام 2.5: x=1:1e9 → همان خطا (materialize شدن بازه)
        let r = evaluate_line("x = 1:1e9;".to_owned());
        assert_eq!(r.status, JobStatus::Failed);
        assert!(matches!(
            &r.console_events[0],
            ConsoleEvent::Error { err } if err.id == "MC:E4003_OUT_OF_MEMORY"
        ));

        // -- رشد اندیس‌دار (مسیر grow_linear): اندیس از کاربر می‌آید
        let r = evaluate_line("v = 7; v(1000000000) = 5;".to_owned());
        assert_eq!(r.status, JobStatus::Failed);
        assert!(matches!(
            &r.console_events[0],
            ConsoleEvent::Error { err } if err.id == "MC:E4003_OUT_OF_MEMORY"
        ));

        // -- ضرب ابعاد کاربر: repmat و diag(k بزرگ)
        let r = evaluate_line("repmat(eye(2), 100000, 100000);".to_owned());
        assert_eq!(r.status, JobStatus::Failed);
        assert!(matches!(
            &r.console_events[0],
            ConsoleEvent::Error { err } if err.id == "MC:E4003_OUT_OF_MEMORY"
        ));
        let r = evaluate_line("diag(ones(3,1), 1000000000);".to_owned());
        assert_eq!(r.status, JobStatus::Failed);
        assert!(matches!(
            &r.console_events[0],
            ConsoleEvent::Error { err } if err.id == "MC:E4003_OUT_OF_MEMORY"
        ));

        // -- تلفیق افقی/عمودی (مسیر hcat/vcat): مجموع می‌تواند از بودجه بگذرد
        let r = evaluate_line("[zeros(1,100000000), zeros(1,2)];".to_owned());
        assert_eq!(r.status, JobStatus::Failed);
        assert!(matches!(
            &r.console_events[0],
            ConsoleEvent::Error { err } if err.id == "MC:E4003_OUT_OF_MEMORY"
        ));
        let r = evaluate_line("[zeros(100000000,1); zeros(2,1)];".to_owned());
        assert_eq!(r.status, JobStatus::Failed);
        assert!(matches!(
            &r.console_events[0],
            ConsoleEvent::Error { err } if err.id == "MC:E4003_OUT_OF_MEMORY"
        ));

        // -- پروسه زنده است: موتور پس از همهٔ این‌ها سرِ کارش + تهی همیشه مجاز
        let r = evaluate_line("zeros(2,3)".to_owned());
        assert_eq!(r.status, JobStatus::Done);
        let r = evaluate_line("w = 1:0;".to_owned());
        assert_eq!(r.status, JobStatus::Done);
        let r = evaluate_line("v(3) = 9; v".to_owned());
        assert_eq!(r.status, JobStatus::Done); // رشد کوچک همان مسیر، مجاز
    }

    // ----- قرارداد رسم v2 (P3-a — ROADMAP 3.1/3.3) -----

    /// نسخهٔ قرارداد پس از فریز P3-a.
    #[test]
    fn plot_contract_version_is_two_after_p3a_freeze() {
        assert_eq!(plot_contract_version(), 2);
    }

    /// رفت‌وبرگشت کامل: گزارهٔ plot → PlotOp → FigureState → snapshot → DTO.
    #[test]
    fn plot_roundtrip_pushes_line_figure_with_ticks() {
        let _guard = serial_guard();
        // نشست تازه: شکل‌های قبلی را با شمارهٔ شکل بزرگ دور می‌زنیم
        let r = evaluate_line("figure(9); x = 0:0.1:2*pi; plot(x, sin(x));".to_owned());
        assert_eq!(r.status, JobStatus::Done);
        assert_eq!(r.plots.len(), 1);
        let f = &r.plots[0];
        assert_eq!(f.figure_id, 9);
        match &f.spec {
            PlotSpec::Line2D { series, axes, .. } => {
                assert_eq!(series.len(), 1);
                assert_eq!(series[0].raw_len, 63); // 0:0.1:2π
                assert_eq!(series[0].x.len(), 63); // زیر آستانهٔ LTTB — دست‌نخورده
                assert_eq!(series[0].style.color, PlotColor::Palette(0));
                assert_eq!(series[0].style.dash, DashStyle::Solid);
                // v2.1: تیک خودکار ساطع نمی‌شود (تولید در رندرر Dart)؛
                // فقط سفارشیِ کاربر منتقل می‌شود
                assert!(axes.x_ticks.is_empty());
                assert!(axes.y_ticks.is_empty());
                assert!(axes.visible); // پیش‌فرض
            }
            other => panic!("expected Line2D, got {other:?}"),
        }
        // بدون اکو/ans: کنسول تمیز
        assert!(r.console_events.is_empty());
    }

    /// مدل pull: get_plots کل شکل‌ها؛ push فقط دست‌خورده‌های همان اجرا.
    #[test]
    fn get_plots_pull_lists_all_figures() {
        let _guard = serial_guard();
        let r =
            evaluate_line("figure(11); plot(1:3); figure(12); bar([1 2 3], [4 5 6]); 1".to_owned());
        assert_eq!(r.status, JobStatus::Done);
        assert_eq!(r.plots.len(), 2); // هر دو شکل در همین اجرا ساخته شدند
        let all = get_plots();
        assert!(all.len() >= 2);
        let ids: Vec<u64> = all.iter().map(|f| f.figure_id).collect();
        assert!(ids.contains(&11) && ids.contains(&12));
        // اجرای بدون رسم: push خالی، pull همان‌قدر
        let r = evaluate_line("2 * 21".to_owned());
        assert_eq!(r.status, JobStatus::Done);
        assert!(r.plots.is_empty());
        assert_eq!(get_plots().len(), all.len());
    }

    /// hold + چند فراخوانی: سری‌ها در یک سلول جمع می‌شوند (سطح FFI).
    #[test]
    fn hold_accumulates_series_across_statements() {
        let _guard = serial_guard();
        let r = evaluate_line(
            "figure(13); hold('on'); plot(1:3); plot(1:3, 'r--'); title('T'); 7".to_owned(),
        );
        assert_eq!(r.status, JobStatus::Done);
        match &r.plots[0].spec {
            PlotSpec::Line2D {
                series,
                axes,
                legend,
            } => {
                assert_eq!(series.len(), 2);
                assert_eq!(
                    series[1].style.color,
                    PlotColor::Rgba {
                        r: 255,
                        g: 0,
                        b: 0,
                        a: 255
                    }
                );
                assert_eq!(series[1].style.dash, DashStyle::Dashed);
                assert_eq!(axes.title.as_deref(), Some("T"));
                assert!(legend.is_none());
            }
            other => panic!("expected Line2D, got {other:?}"),
        }
    }

    /// خطای وسط اسکریپت: نمودارهای پیش از خطا می‌مانند (سمانتیک متلب).
    #[test]
    fn plots_before_error_survive_in_push() {
        let _guard = serial_guard();
        let r = evaluate_line("figure(14); plot(1:3); undefined_fn_xyz()".to_owned());
        assert_eq!(r.status, JobStatus::Failed);
        assert_eq!(r.plots.len(), 1, "figure drawn before the error is kept");
    }
}
