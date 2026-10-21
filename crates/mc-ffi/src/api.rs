//! API عمومی mc-ffi — تولید کد Dart با `flutter_rust_bridge_codegen generate`
//! (خروجی: `app/lib/src/rust/`).
//!
//! این فایل «قرارداد نسخه‌دار» بین UI و موتور است؛ تغییر شکانده در هر نوع =
//! افزایش `API_CONTRACT_VERSION` و پیام واضح در startup (سند ۰۲ بخش ۳ بند ۴).

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::Instant;

use mc_core::{McError as CoreMcError, Value};
use mc_interp::{eval_program, Environment};
use mc_parser::parse;

/// نسخه قرارداد API — UI در startup با `api_version()` مقایسه می‌کند.
pub const API_CONTRACT_VERSION: u32 = 1;

/// نسخه قرارداد رسم (DTO) رسم‌ها — سند ۰۴ بخش ۸، سند ۰۷ بخش ۳ بند ۲.
pub const PLOT_CONTRACT_VERSION: u32 = 1;

// ---------------------------------------------------------------------------
// انواع قرارداد (سند ۰۲ بخش ۳ — «قرارداد داده اصلی»)
// ---------------------------------------------------------------------------

/// نتیجه یک اجرا — ساختار کامل سند ۰۲ بخش ۳.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct EvalResult {
    pub job_id: u64,
    /// متن، خطا، warning — در P0 دسته‌ای؛ streaming کامل با `runScript` در P2.
    pub console_events: Vec<ConsoleEvent>,
    /// مدل داده رسم (سند ۰۴ بخش ۸) — در P0 همیشه خالی.
    pub plots: Vec<PlotSpec>,
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
        let m = match value {
            Value::Double(m) => m,
            // char/هندل پیش‌نمایش عددی ندارند — فقط کلاس گزارش می‌شود (سند ۰۲ §3)
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
// قرارداد رسم (نسخه پل — سند ۰۴ بخش ۸.۱؛ قطعی‌شدن نهایی قبل از P3-a)
// ---------------------------------------------------------------------------

/// enum بسته رسم‌ها (سند ۰۴ بخش ۸.۱) — «داده در Rust، ترسیم در Dart».
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
    Bar2D {
        categories: Vec<String>,
        values: Vec<f64>,
        axes: Axes2D,
    },
    Histogram {
        bin_edges: Vec<f64>,
        counts: Vec<u64>,
        axes: Axes2D,
    },
    Polar {
        series: Vec<Series2D>,
    },
    Quiver {
        x: Vec<f64>,
        y: Vec<f64>,
        u: Vec<f64>,
        v: Vec<f64>,
        axes: Axes2D,
    },
    Surface {
        x: Vec<f64>,
        y: Vec<f64>,
        z: Vec<Vec<f64>>,
        shading: Shading,
        colormap: Colormap,
    },
    Mesh {
        x: Vec<f64>,
        y: Vec<f64>,
        z: Vec<Vec<f64>>,
        colormap: Colormap,
    },
    Contour {
        x: Vec<f64>,
        y: Vec<f64>,
        z: Vec<Vec<f64>>,
        levels: Vec<f64>,
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

#[derive(Debug, Clone, PartialEq)]
pub struct Series2D {
    pub x: Vec<f64>,
    pub y: Vec<f64>,
    /// رنگ از ایندکس پالت ۸ رنگی CDS (سند ۰۶ بخش ۳) — هگز در Dart resolve می‌شود.
    pub style: LineStyle,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Series3D {
    pub x: Vec<f64>,
    pub y: Vec<f64>,
    pub z: Vec<f64>,
    pub style: LineStyle,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LineStyle {
    /// ۰..=۷ — ترتیب پالت سند ۰۶ بخش ۳ (`plot(x,y)` بدون رنگ به ترتیب پالت می‌رود).
    pub palette_index: u8,
    pub width: f32,
    pub dash: DashStyle,
    pub marker: Marker,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DashStyle {
    #[default]
    Solid,
    Dashed,
    Dotted,
    DashDot,
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

#[derive(Debug, Clone, PartialEq, Default)]
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
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Axes3D {
    pub title: Option<String>,
    pub xlabel: Option<String>,
    pub ylabel: Option<String>,
    pub zlabel: Option<String>,
    pub grid: bool,
}

/// حدود محور — جفت min/max (تیک‌های زیبای 1-2-5 در Rust تولید می‌شوند؛ سند ۰۴ §8.1).
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
}

static ENGINE: OnceLock<Mutex<Engine>> = OnceLock::new();

fn engine() -> &'static Mutex<Engine> {
    ENGINE.get_or_init(|| {
        Mutex::new(Engine {
            env: Environment::new(),
            jobs: 0,
        })
    })
}

/// نسخه قرارداد — UI در startup چک می‌کند (سند ۰۲ بخش ۳ بند ۴).
pub fn api_version() -> u32 {
    API_CONTRACT_VERSION
}

/// نسخه قرارداد رسم‌ها.
pub fn plot_contract_version() -> u32 {
    PLOT_CONTRACT_VERSION
}

/// ارزیابی یک خط کنسول — دمو رسمی P0 (سند ۰۷):
/// `evaluateLine("1+1")` از UI فارسی تا Rust و برگشت، با event های تایپ‌دار.
///
/// سمانتیک متلب: اجرای دستورها به ترتیب، توقف در اولین خطا؛ خروجی‌های
/// دستورهای قبلی حفظ می‌شوند (سند ۰۲ بخش ۹).
pub fn evaluate_line(code: String) -> EvalResult {
    let t0 = Instant::now();
    let mut engine = engine()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    engine.jobs += 1;
    let job_id = engine.jobs;
    let before: HashMap<String, Value> = engine.env.snapshot();

    let (mut events, status) = match parse(&code) {
        Err(e) => (
            vec![ConsoleEvent::Error { err: e.into() }],
            JobStatus::Failed,
        ),
        Ok(program) => {
            let outcome = eval_program(&program, &mut engine.env);
            let mut events: Vec<ConsoleEvent> = outcome
                .outputs
                .into_iter()
                .map(|o| ConsoleEvent::Print { text: o.text })
                .collect();
            let status = match outcome.error {
                None => JobStatus::Done,
                Some(err) => {
                    events.push(ConsoleEvent::Error { err: err.into() });
                    JobStatus::Failed
                }
            };
            (events, status)
        }
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
        plots: Vec::new(),
        workspace_delta,
        status,
        duration_ms: u64::try_from(t0.elapsed().as_millis()).unwrap_or(u64::MAX),
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

    /// نشست موتور جهانی است (سمانتیک سریالی سند ۰۲ بخش ۴)؛ پس آزمون‌های آن
    /// باید **ترتیبی** اجرا شوند — همه سناریوها در یک #[test] پشت سر هم.
    #[test]
    fn engine_session_end_to_end_scenarios() {
        // -- دمو رسمی P0 (سند ۰۷): 1+1 از قرارداد تا خروجی
        let r = evaluate_line("1+1".to_owned());
        assert_eq!(r.status, JobStatus::Done);
        assert_eq!(r.job_id, 1);
        assert_eq!(
            r.console_events,
            vec![ConsoleEvent::Print {
                text: "ans =\n     2
".to_owned()
            }]
        );
        assert_eq!(r.workspace_delta.changed.len(), 1);
        assert_eq!(r.workspace_delta.changed[0].name, "ans");
        assert_eq!(r.workspace_delta.changed[0].preview, vec![2.0]);

        // -- خطای پارس ساختاریافته
        let r = evaluate_line("1 +".to_owned());
        assert_eq!(r.status, JobStatus::Failed);
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

        // -- ثبات نسخه‌های قرارداد
        assert_eq!(api_version(), 1);
        assert_eq!(plot_contract_version(), 1);

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
            ConsoleEvent::Print { text } if text == "ans =\n    10\n"
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
}
