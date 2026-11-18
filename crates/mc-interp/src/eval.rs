//! ارزیابی — سمانتیک سند ۰۳ (بخش‌های ۴ تا ۱۱، نسخه P1).
//!
//! - `ans`: فقط دستورهای عبارتیِ «نمایش‌دار» در `ans` می‌نشینند (رفتار متلب).
//! - خطا یک مقدار است: ارزیابی در اولین خطا متوقف و `McError` برمی‌گرداند؛
//!   خروجی‌های قبلی همان خط حفظ می‌شوند (سند ۰۲ بخش ۹).
//! - محاسبات IEEE؛ `1/0 = Inf`، `0/0 = NaN` (سند ۰۳ بخش ۱۱ بند ۵).
//! - COW: انتساب کپی نمی‌کند؛ Matrix خودش Arc+COW است (سند ۰۳ §11 بند 1).
//! - `for` روی ماتریس = پیمایش ستون‌ها (سند ۰۳ §8).
//! - `end` فقط در زمینهٔ اندیس معتبر است؛ کولن سمانتیک دقیق سند ۰۳ §7.

use std::cell::Cell;
use std::sync::Arc;

use mc_core::{
    budget, ids, job, struct_depth, AnonFn, Matrix, McError, StructVal, Value, MAX_STRUCT_DEPTH,
};
use mc_parser::{
    AssignTarget, BinOp, CmpOp, Expr, FieldName, LogicOp, OutTarget, Program, Stmt, UnOp,
};

use crate::complex_ops as cx;
use crate::env::{Environment, StoredItem, UserFnDef};
use crate::registry::{self, broadcast2};

/// خروجی یک دستور برای کنسول — متن زبان‌خنثی (نمایش متلب؛ خطاها جداگانه FA/EN می‌آیند).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StmtOutput {
    pub text: String,
}

/// نتیجه ارزیابی برنامه: خروجی‌های دستورهای اجراشده + خطای توقف (در صورت وجود).
#[derive(Debug, Clone, Default)]
pub struct EvalOutcome {
    pub outputs: Vec<StmtOutput>,
    pub error: Option<McError>,
    /// عملیات‌های رسم این اجرا (P3-a) — ساطع‌شده به سینک mc-plot و این‌جا
    /// تخلیه می‌شود؛ حالت شکل‌ها در mc-ffi از این دنباله fold می‌شود.
    pub plot_ops: Vec<mc_plot::spec::PlotOp>,
}

/// سیگنال جریان کنترل — break/continue فقط در حلقه جاری (سند ۰۳ §8)؛
/// return از تابع/اسکریپت جاری خارج می‌شود (سمانتیک متلب).
enum Flow {
    Normal,
    Break,
    Continue,
    Return,
}

/// عمق مجاز فراخوانی توابع بی‌نام (جلوگیری از بی‌نهایت — سند ۰۳ §9:
/// recursive anonymous ممنوع است؛ خطای آموزشی).
const MAX_CALL_DEPTH: usize = 128;

/// ---------------- لغو همکاری‌گر (ROADMAP 2.3، سند ۰۲ §3–4) ----------------
/// چک لغو هر N گزاره در کد خطی (N مستند: ROADMAP 2.3 task 2)؛ حلقه‌های
/// for/while علاوه بر این، «هر تکرار» چک می‌شوند تا `while(1) end` و
/// `for i=1:1e9; end` هم‌زمان قابل لغو باشند. هزینه: یک load اتمیک هر N
/// گزاره / هر تکرار — نامحسوس (DoD ≥295 تست).
pub const CANCEL_CHECK_EVERY: u64 = 128;

thread_local! {
    static STMT_TICK: Cell<u64> = const { Cell::new(0) };
}

/// خطای لغو — نگاشت به `JobStatus::Cancelled` در mc-ffi (نه Failed).
fn cancel_error() -> McError {
    McError::new(
        ids::CANCELLED,
        0,
        0,
        "Operation terminated by user.",
        "عملیات به درخواست کاربر متوقف شد.",
    )
}

/// چک لغو گزاره‌ای: هر N گزاره یک load اتمیک (بدون کانتکست job = no-op).
fn stmt_cancel_checkpoint() -> Result<(), McError> {
    STMT_TICK.with(|t| {
        let n = t.get() + 1;
        t.set(n);
        if n % CANCEL_CHECK_EVERY == 0 && job::cancel_requested() {
            Err(cancel_error())
        } else {
            Ok(())
        }
    })
}

/// چک لغو هر تکرار حلقه (for/while) — گذرِ تکرار هم شمرده می‌شود
/// (job::loop_checkpoint — قطعیت تست لغو، رفع CI قرمز ۲۰۲۶-۱۱-۱۷).
fn loop_cancel_checkpoint() -> Result<(), McError> {
    if job::loop_checkpoint() {
        Err(cancel_error())
    } else {
        Ok(())
    }
}

pub fn eval_program(program: &Program, env: &mut Environment) -> EvalOutcome {
    eval_program_depth(program, env, 0)
}

fn eval_program_depth(program: &Program, env: &mut Environment, depth: usize) -> EvalOutcome {
    let mut out = EvalOutcome::default();
    let mut texts = Vec::new();
    let result = exec_stmts(&program.stmts, env, depth, &mut texts);
    // تخلیهٔ سینک رسم در هر حالت (موفق/خطا) — متلب نمودارهای پیش از خطا را
    // نگه می‌دارد (P3-a)
    out.plot_ops = mc_plot::session::take();
    out.outputs = texts.into_iter().map(|t| StmtOutput { text: t }).collect();
    match result {
        Ok(Flow::Break | Flow::Continue) => {
            // break/continue خارج حلقه — خطای صریح (سند ۰۳ §8)
            out.error = Some(McError::new(
                ids::TYPE_ERROR,
                0,
                0,
                "break/continue outside a loop.",
                "break/continue خارج از حلقه.",
            ));
        }
        Ok(_) => {}
        Err(e) => {
            out.error = Some(e);
        }
    }
    out
}

/// اجرای دنباله‌ای گزاره‌ها؛ خروجی‌های نمایش‌دار در `texts` جمع می‌شوند؛
/// جریان غیرعادی (break/continue/return) جریان را به بالا می‌فرستد.
fn exec_stmts(
    stmts: &[Stmt],
    env: &mut Environment,
    depth: usize,
    texts: &mut Vec<String>,
) -> Result<Flow, McError> {
    for stmt in stmts {
        stmt_cancel_checkpoint()?;
        let (flow, mut t) = exec_stmt(stmt, env, depth)?;
        texts.append(&mut t);
        if !matches!(flow, Flow::Normal) {
            return Ok(flow);
        }
    }
    Ok(Flow::Normal)
}

// ---------- گزاره‌ها ----------

/// پشتیبان موقعیت (ROADMAP 2.11 — بند ۱۴ ممیزی): خطاهای ساخته‌شده در
/// کمکی‌ها/builtinها بدون موقعیت‌اند (0,0) — هیچ خطای کاربر نباید 0:0 بماند؛
/// نزدیک‌ترین عبارت/گزاره جایگزین می‌شود (موقعیت واقعی هرگز بازنویسی نمی‌شود).
fn with_pos(mut e: McError, line: u32, col: u32) -> McError {
    if e.line == 0 && e.col == 0 {
        e.line = line;
        e.col = col;
    }
    e
}

/// موقعیت یک عبارت برای پشتیبان — گره‌های بی‌موقعیت (لفظ‌ها) (0,0) می‌دهند؛
/// خطا روی لفظ خالص رخ نمی‌دهد.
fn expr_pos(e: &Expr) -> (u32, u32) {
    match e {
        Expr::Var { line, col, .. }
        | Expr::End { line, col }
        | Expr::Binary { line, col, .. }
        | Expr::Compare { line, col, .. }
        | Expr::Logic { line, col, .. }
        | Expr::Transpose { line, col, .. }
        | Expr::Matrix { line, col, .. }
        | Expr::Call { line, col, .. }
        | Expr::AnonFn { line, col, .. }
        | Expr::FnHandle { line, col, .. }
        | Expr::Field { line, col, .. }
        | Expr::Index { line, col, .. } => (*line, *col),
        Expr::Unary { line, col, .. } | Expr::Range { line, col, .. } => (*line, *col),
        Expr::Num(_) | Expr::Imag(_) | Expr::Str(_) => (0, 0),
    }
}

/// موقعیت هدف انتساب (همهٔ اشکال هدف line/col دارند).
fn target_pos(t: &AssignTarget) -> (u32, u32) {
    match t {
        AssignTarget::Var { line, col, .. }
        | AssignTarget::Indexed { line, col, .. }
        | AssignTarget::Field { line, col, .. }
        | AssignTarget::IndexedField { line, col, .. } => (*line, *col),
    }
}

/// موقعیت گزاره برای پشتیبانِ سطح گزاره.
fn stmt_pos(s: &Stmt) -> (u32, u32) {
    match s {
        Stmt::Expr { expr, .. } => expr_pos(expr),
        Stmt::Assign { target, .. } => target_pos(target),
        Stmt::If { line, col, .. }
        | Stmt::For { line, col, .. }
        | Stmt::While { line, col, .. }
        | Stmt::Break { line, col }
        | Stmt::Continue { line, col }
        | Stmt::Return { line, col }
        | Stmt::FnDef { line, col, .. } => (*line, *col),
        Stmt::MultiAssign { targets, .. } => targets.first().map_or((0, 0), |t| (t.line, t.col)),
    }
}

fn exec_stmt(
    stmt: &Stmt,
    env: &mut Environment,
    depth: usize,
) -> Result<(Flow, Vec<String>), McError> {
    // پشتیبان موقعیت سطح گزاره (ROADMAP 2.11) — خطاهای کمکی بدون موقعیت،
    // موقعیت گزارهٔ در حال اجرا را می‌گیرند
    exec_stmt_inner(stmt, env, depth).map_err(|e| {
        let (l, c) = stmt_pos(stmt);
        with_pos(e, l, c)
    })
}

fn exec_stmt_inner(
    stmt: &Stmt,
    env: &mut Environment,
    depth: usize,
) -> Result<(Flow, Vec<String>), McError> {
    // حالت نمایش در ورود گزاره ثابت می‌شود (Copy) — `format` وسط گزاره اثر
    // گذشته‌گر ندارد (رفتار متلب).
    let spec = env.display;
    match stmt {
        Stmt::Expr { expr, suppress } => {
            // اسکریپت/تابع فایلی ثبت‌شده — تقدم متلب: متغیر بر هر دو مقدم است.
            // - اسکریپت: اجرا در فضای کاری فراخوان، بدون مقدار و بدون ans (سمانتیک متلب §9)
            // - تابع به‌صورت گزاره: nargout = 1 اگر خروجی تعریف کرده، وگرنه 0
            //   (`show(5)` در پرامپت متلب کار می‌کند ولی `x = show(5)` خطا می‌دهد)
            if let Expr::Call { name, args, .. } = expr {
                if args.is_empty() && env.get(name).is_none() {
                    if let Some(StoredItem::Script(prog)) = env.lookup_item(name) {
                        let mut texts = Vec::new();
                        let flow = exec_stmts(&prog.stmts, env, depth, &mut texts)?;
                        match flow {
                            Flow::Break | Flow::Continue => {
                                return Err(McError::new(
                                    ids::TYPE_ERROR,
                                    0,
                                    0,
                                    "break/continue outside a loop.",
                                    "break/continue خارج از حلقه.",
                                ))
                            }
                            // return داخل اسکریپت = پایان اسکریپت، ادامه فراخوان
                            Flow::Normal | Flow::Return => {}
                        }
                        return Ok((Flow::Normal, texts));
                    }
                }
            }
            if let Expr::Call {
                name,
                args,
                line,
                col,
                ..
            } = expr
            {
                if env.get(name).is_none() {
                    if let Some(StoredItem::Fn(def)) = env.lookup_item(name) {
                        let vals: Vec<Value> = args
                            .iter()
                            .map(|a| eval_expr(a, env, depth))
                            .collect::<Result<_, _>>()?;
                        let nout = usize::from(!def.outputs.is_empty());
                        let outs =
                            call_user_fn(&def, &vals, nout, env.funcs_clone(), depth, *line, *col)?;
                        let value = match outs.into_iter().next() {
                            Some(v) => v,
                            None => return Ok((Flow::Normal, Vec::new())),
                        };
                        if !*suppress {
                            let text = crate::format::format_ans_spec(&value, &spec);
                            env.set("ans", value);
                            return Ok((Flow::Normal, vec![text]));
                        }
                        return Ok((Flow::Normal, Vec::new()));
                    }
                }
            }
            // نام برهنهٔ ثبت‌شده — سمانتیک متلب §9:
            // - اسکریپت: با نام برهنه اجرا می‌شود (در فضای کاری فراخوان)
            // - تابع بدون پارامتر: فراخوانی می‌شود
            // - تابع پارامتردار: خطای «آرگومان کم است» (رفتار متلب)
            if let Expr::Var { name, line, col } = expr {
                if env.get(name).is_none() {
                    if let Some(StoredItem::Script(prog)) = env.lookup_item(name) {
                        let mut texts = Vec::new();
                        let flow = exec_stmts(&prog.stmts, env, depth, &mut texts)?;
                        match flow {
                            Flow::Break | Flow::Continue => {
                                return Err(McError::new(
                                    ids::TYPE_ERROR,
                                    0,
                                    0,
                                    "break/continue outside a loop.",
                                    "break/continue خارج از حلقه.",
                                ))
                            }
                            Flow::Normal | Flow::Return => {}
                        }
                        return Ok((Flow::Normal, texts));
                    }
                    if let Some(StoredItem::Fn(def)) = env.lookup_item(name) {
                        if def.params.is_empty() {
                            let nout = usize::from(!def.outputs.is_empty());
                            let outs = call_user_fn(
                                &def,
                                &[],
                                nout,
                                env.funcs_clone(),
                                depth,
                                *line,
                                *col,
                            )?;
                            if let Some(v) = outs.into_iter().next() {
                                if !*suppress {
                                    let text = crate::format::format_ans_spec(&v, &spec);
                                    env.set("ans", v);
                                    return Ok((Flow::Normal, vec![text]));
                                }
                            }
                            return Ok((Flow::Normal, Vec::new()));
                        }
                        return Err(not_enough_inputs(name, def.params.len(), *line, *col));
                    }
                }
            }
            // فرمان `format` — تغییر حالت نمایش (ROADMAP 2.4 — آیتم ۱۱ ممیزی).
            // متغیر/تابع/اسکریپت هم‌نام تقدم دارند (الگوی بالای همین تابع).
            if let Expr::Call {
                name,
                args,
                line,
                col,
                ..
            } = expr
            {
                if name == "format" && env.get(name).is_none() && env.lookup_item(name).is_none() {
                    let words: Vec<String> = args
                        .iter()
                        .map(|a| match a {
                            Expr::Str(s) => Ok(s.clone()),
                            _ => Err(McError::new(
                                ids::TYPE_ERROR,
                                *line,
                                *col,
                                "format: mode must be a literal word (e.g. format short).",
                                "format: حالت باید کلمهٔ مستقیم باشد (مثل format short).",
                            )),
                        })
                        .collect::<Result<_, _>>()?;
                    if let Some(mode) = parse_format_words(&words, *line, *col)? {
                        env.display.num = mode;
                    }
                    return Ok((Flow::Normal, Vec::new()));
                }
            }
            // disp/fprintf — خروجی صریح کنسول (ROADMAP 2.4 — آیتم ۸ ممیزی).
            // خروجی مستقل از «;» چاپ می‌شود (رفتار متلب)، ans را ست نمی‌کند
            // و مقدار برنمی‌گرداند. متغیر/تابع هم‌نام تقدم دارد.
            if let Expr::Call {
                name,
                args,
                line,
                col,
                ..
            } = expr
            {
                if (name == "disp" || name == "fprintf")
                    && env.get(name).is_none()
                    && env.lookup_item(name).is_none()
                {
                    let vals: Vec<Value> = args
                        .iter()
                        .map(|a| eval_expr(a, env, depth))
                        .collect::<Result<_, _>>()?;
                    let text = if name == "disp" {
                        crate::stdio::disp_text(&vals, *line, *col, &spec)?
                    } else {
                        crate::stdio::fprintf_text(&vals, *line, *col)?
                    };
                    return Ok((Flow::Normal, vec![text]));
                }
            }
            // `rng(...)` گزاره‌ای — مثل متلب بدون اکو و بدون ans (ROADMAP 2.9):
            // شکل‌های rng / rng(seed) / rng('default') / rng(struct) فقط اثر
            // جانبی دارند؛ مقدار ساختار وضعیت فقط با خروجی‌گیری می‌آید (s = rng).
            // متغیر/تابع هم‌نام تقدم دارد (الگوی disp/fPrintf بالای همین تابع).
            if let Expr::Call {
                name,
                args,
                line,
                col,
                ..
            } = expr
            {
                if name == "rng" && env.get(name).is_none() && env.lookup_item(name).is_none() {
                    let vals: Vec<Value> = args
                        .iter()
                        .map(|a| eval_expr(a, env, depth))
                        .collect::<Result<_, _>>()?;
                    crate::registry_p2::rng_apply(&vals, *line, *col)?;
                    return Ok((Flow::Normal, Vec::new()));
                }
            }
            // توابع رسم گزاره‌ای (P3-a — ROADMAP 3.2) — الگوی disp/rng:
            // بدون اکو، بدون ans، فقط اثر جانبی (emit به سینک mc-plot)؛
            // فراخوانی داخل عبارت به رجیستری می‌خورد و خطای روشن می‌دهد.
            if let Expr::Call {
                name,
                args,
                line,
                col,
                ..
            } = expr
            {
                if crate::plot_builtins::is_plot_statement_fn(name)
                    && env.get(name).is_none()
                    && env.lookup_item(name).is_none()
                {
                    let vals: Vec<Value> = args
                        .iter()
                        .map(|a| eval_expr(a, env, depth))
                        .collect::<Result<_, _>>()?;
                    crate::plot_builtins::apply(name, &vals, *line, *col)?;
                    return Ok((Flow::Normal, Vec::new()));
                }
            }
            // `rng` برهنه — بدون خروجی اثری ندارد (وضعیت گرفته و دور ریخته می‌شود)
            if let Expr::Var { name, .. } = expr {
                if name == "rng" && env.get(name).is_none() && env.lookup_item(name).is_none() {
                    return Ok((Flow::Normal, Vec::new()));
                }
            }
            // `format` برهنه — بازگشت به پیش‌فرض short (رفتار متلب)
            if let Expr::Var { name, .. } = expr {
                if name == "format" && env.get(name).is_none() && env.lookup_item(name).is_none() {
                    env.display.num = crate::format::NumFormat::Short;
                    return Ok((Flow::Normal, Vec::new()));
                }
            }
            // توابع رسم برهنه (گام 3.5.1 — پارتی متلب): `hold` = toggle،
            // `figure` = شکل تازه، `grid` = toggle. فقط وقتی متغیر/تابع
            // کاربر هم‌نام نباشد؛ وگرنه مسیر عادی (خطای «statement-only»).
            if let Expr::Var { name, line, col } = expr {
                if crate::plot_builtins::is_plot_statement_fn(name)
                    && env.get(name).is_none()
                    && env.lookup_item(name).is_none()
                {
                    crate::plot_builtins::apply(name, &[], *line, *col)?;
                    return Ok((Flow::Normal, Vec::new()));
                }
            }
            let value = eval_expr(expr, env, depth)?;
            if !*suppress {
                // بند ۳ گزارش ممیزی خارجی — متلب: ارجاع برهنهٔ متغیر با «نام =»
                // اکو می‌شود (نه «ans =»)؛ مقدار ans هم مثل متلب همان مقدار می‌شود.
                let text = match expr {
                    Expr::Var { name, .. } => {
                        crate::format::format_assignment_spec(name, &value, &spec)
                    }
                    _ => crate::format::format_ans_spec(&value, &spec),
                };
                env.set("ans", value);
                return Ok((Flow::Normal, vec![text]));
            }
            Ok((Flow::Normal, Vec::new()))
        }
        Stmt::Assign {
            target,
            expr,
            suppress,
        } => {
            let value = eval_expr(expr, env, depth)?;
            match target {
                AssignTarget::Var { name, .. } => {
                    env.set(name.clone(), value);
                    if !*suppress {
                        let text = crate::format::format_assignment_spec(
                            name,
                            env.get(name)
                                .unwrap_or(&Value::Double(Matrix::full(0, 0, 0.0))),
                            &spec,
                        );
                        Ok((Flow::Normal, vec![text]))
                    } else {
                        Ok((Flow::Normal, Vec::new()))
                    }
                }
                AssignTarget::Indexed {
                    name,
                    subs,
                    line,
                    col,
                } => {
                    indexed_assign(env, name, subs, value, *line, *col)?;
                    // متلب نتیجهٔ انتساب اندیس‌دار بدون «;» را هم نمایش می‌دهد
                    if !*suppress {
                        let text = crate::format::format_assignment_spec(
                            name,
                            env.get(name)
                                .unwrap_or(&Value::Double(Matrix::full(0, 0, 0.0))),
                            &spec,
                        );
                        Ok((Flow::Normal, vec![text]))
                    } else {
                        Ok((Flow::Normal, Vec::new()))
                    }
                }
                // `s.f = v` / `s.(f) = v` — ROADMAP 2.9؛ نمایش مثل متلب روی
                // متغیرِ ریشه است (کل ساختار، نه فقط فیلد)
                AssignTarget::Field {
                    base,
                    field,
                    line,
                    col,
                } => {
                    let root = field_assign(env, base, field, value, *line, *col, depth)?;
                    if !*suppress {
                        let text = crate::format::format_assignment_spec(
                            &root,
                            env.get(&root)
                                .unwrap_or(&Value::Double(Matrix::full(0, 0, 0.0))),
                            &spec,
                        );
                        Ok((Flow::Normal, vec![text]))
                    } else {
                        Ok((Flow::Normal, Vec::new()))
                    }
                }
                // `s.f(i) = v` — اندیس‌گذاری داخل فیلد (ROADMAP 2.10)؛ نمایش
                // روی متغیر ریشه (رفتار متلب)
                AssignTarget::IndexedField {
                    base,
                    field,
                    subs,
                    line,
                    col,
                } => {
                    let root =
                        indexed_field_assign(env, base, field, subs, value, *line, *col, depth)?;
                    if !*suppress {
                        let text = crate::format::format_assignment_spec(
                            &root,
                            env.get(&root)
                                .unwrap_or(&Value::Double(Matrix::full(0, 0, 0.0))),
                            &spec,
                        );
                        Ok((Flow::Normal, vec![text]))
                    } else {
                        Ok((Flow::Normal, Vec::new()))
                    }
                }
            }
        }
        // انتساب چندخروجی `[a,b] = f(...)` — سمانتیک متلب §9
        Stmt::MultiAssign {
            targets,
            expr,
            suppress,
        } => {
            let texts = eval_multi_assign(targets, expr, env, depth, *suppress)?;
            Ok((Flow::Normal, texts))
        }
        // تعریف تابع فایلی — ثبت در مخزن نشست؛ هیچ اجرا/خروجی ندارد
        Stmt::FnDef {
            name,
            outputs,
            params,
            body,
            line,
            col,
        } => {
            env.register_function(UserFnDef {
                name: name.clone(),
                outputs: outputs.clone(),
                params: params.clone(),
                body: Program {
                    stmts: body.clone(),
                },
                line: *line,
                col: *col,
            });
            Ok((Flow::Normal, Vec::new()))
        }
        // `return` — خروج از تابع/اسکریپت جاری (سمانتیک متلب)
        Stmt::Return { .. } => Ok((Flow::Return, Vec::new())),
        Stmt::If {
            branches,
            else_body,
            ..
        } => {
            for (cond, body) in branches {
                let c = eval_expr(cond, env, depth)?;
                if truthy_condition(&c)? {
                    // اکوی متلب: گزاره‌های نمایش‌دار داخل شاخه هم به کنسول می‌روند
                    return exec_block(body, env, depth);
                }
            }
            if let Some(body) = else_body {
                return exec_block(body, env, depth);
            }
            Ok((Flow::Normal, Vec::new()))
        }
        Stmt::For {
            var,
            iter_expr,
            body,
            ..
        } => {
            let iter_val = eval_expr(iter_expr, env, depth)?;
            let columns: Vec<Value> = match &iter_val {
                Value::Double(m) => {
                    if m.is_scalar() {
                        vec![iter_val.clone()]
                    } else {
                        (0..m.cols())
                            .map(|c| {
                                let mut col = Matrix::full(m.rows(), 1, 0.0);
                                for r in 0..m.rows() {
                                    col.set(r, 0, m.get(r, c));
                                }
                                Value::Double(col)
                            })
                            .collect()
                    }
                }
                Value::Complex(z) => {
                    if z.is_scalar() {
                        vec![iter_val.clone()]
                    } else {
                        (0..z.cols())
                            .map(|c| {
                                let mut col = mc_core::ComplexMatrix::full(z.rows(), 1, 0.0, 0.0);
                                for r in 0..z.rows() {
                                    let (re, im) = z.get(r, c);
                                    col.set(r, 0, re, im);
                                }
                                Value::Complex(col)
                            })
                            .collect()
                    }
                }
                Value::Char(s) => s.chars().map(|ch| Value::Char(ch.to_string())).collect(),
                // ساختار ۱×۱ = یک تکرار با خود ساختار (رفتار متلب/Octave 10.3 —
                // آرایهٔ ساختاری نداریم پس پیمایش همیشه تک‌عضوی است؛ ROADMAP 2.9)
                Value::Struct(_) => vec![iter_val.clone()],
                Value::Anon(_) => {
                    return Err(McError::new(
                        ids::TYPE_ERROR,
                        0,
                        0,
                        "Cannot iterate over a function handle.",
                        "پیمایش روی function_handle ممکن نیست.",
                    ))
                }
            };
            let mut texts = Vec::new();
            for col in columns {
                loop_cancel_checkpoint()?;
                env.set(var.clone(), col);
                let (flow, mut t) = exec_block(body, env, depth)?;
                texts.append(&mut t);
                match flow {
                    Flow::Break => break,
                    // return داخل بدنهٔ حلقه باید فوراً خارج شود (سمانتیک متلب)
                    Flow::Return => return Ok((Flow::Return, texts)),
                    Flow::Normal | Flow::Continue => {}
                }
            }
            Ok((Flow::Normal, texts))
        }
        Stmt::While { cond, body, .. } => {
            let mut texts = Vec::new();
            loop {
                loop_cancel_checkpoint()?;
                let c = eval_expr(cond, env, depth)?;
                if !truthy_condition(&c)? {
                    break;
                }
                let (flow, mut t) = exec_block(body, env, depth)?;
                texts.append(&mut t);
                match flow {
                    Flow::Break => break,
                    Flow::Return => return Ok((Flow::Return, texts)),
                    Flow::Normal | Flow::Continue => {}
                }
            }
            Ok((Flow::Normal, texts))
        }
        Stmt::Break { .. } => Ok((Flow::Break, Vec::new())),
        Stmt::Continue { .. } => Ok((Flow::Continue, Vec::new())),
    }
}

/// بلوک = دنبالهٔ گزاره‌ها + متن‌های نمایش آن‌ها (اکوی متلب). در بدنهٔ توابع
/// متن‌ها توسط call_user_fn دور ریخته می‌شوند (توابع مثل متلب بی‌اکو هستند)؛
/// در کنسول/اسکریپت/شاخه‌های if و بدنهٔ حلقه‌ها اکو به بالا منتقل می‌شود.
fn exec_block(
    body: &[Stmt],
    env: &mut Environment,
    depth: usize,
) -> Result<(Flow, Vec<String>), McError> {
    let mut sink = Vec::new();
    let flow = exec_stmts(body, env, depth, &mut sink)?;
    Ok((flow, sink))
}

/// شرط if/while با سمانتیک متلب: «غیرخالی و همهٔ عناصر ناصفر» — برخلاف
/// `&&`/`||` (که اسکالر می‌خواهند)، آرایه هم مجاز است؛ NaN ناصفر است → درست؛
/// شرط خالی `[]` یعنی نادرست (بند ۷ گزارش ممیزی خارجی).
fn truthy_condition(v: &Value) -> Result<bool, McError> {
    match v {
        Value::Double(m) => {
            if m.is_empty() {
                return Ok(false);
            }
            for k in 0..m.numel() {
                if m.get_linear(k) == 0.0 {
                    return Ok(false);
                }
            }
            Ok(true)
        }
        Value::Complex(cm) => {
            if cm.is_empty() {
                return Ok(false);
            }
            for k in 0..cm.rows() * cm.cols() {
                let (re, im) = cm.get_linear(k);
                if re == 0.0 && im == 0.0 {
                    return Ok(false);
                }
            }
            Ok(true)
        }
        other => truthy_scalar(other),
    }
}

fn truthy_scalar(v: &Value) -> Result<bool, McError> {
    match v.as_scalar() {
        Some(x) if x.is_nan() => Err(McError::new(
            ids::TYPE_ERROR,
            0,
            0,
            "Condition is NaN — cannot decide.",
            "شرط NaN است — تصمیم‌گیری ممکن نیست.",
        )),
        Some(x) => Ok(x != 0.0),
        None => Err(McError::new(
            ids::TYPE_ERROR,
            0,
            0,
            "Condition must be a scalar. For matrices use all(...) or any(...).",
            "شرط باید اسکالر باشد؛ برای ماتریس از all(...) یا any(...) استفاده کنید.",
        )),
    }
}

// ---------- انتساب اندیس‌دار ----------

/// ماتریس تک‌ردیفهٔ کدها → رشتهٔ char (ROADMAP 2.8)؛ چندردیفه = char دوبعدی
/// → خطای صادقانهٔ E4.
fn char_row_from_matrix(m: &Matrix, line: u32, col: u32) -> Result<String, McError> {
    if m.rows() != 1 {
        return Err(McError::new(
            ids::UNSUPPORTED,
            line,
            col,
            "This indexed assignment would produce a 2-D char array; expansion E4.",
            "این انتساب اندیس‌دار آرایهٔ char دوبعدی می‌سازد؛ انبساط E4 است.",
        ));
    }
    let mut s = String::with_capacity(m.cols());
    for i in 0..m.cols() {
        s.push(Value::char_from_code(m.get(0, i), line, col)?);
    }
    Ok(s)
}

/// `A(subs...) = v` — با رشد خودکار ماتریس مثل متلب (پرکردن با صفر).
fn indexed_assign(
    env: &mut Environment,
    name: &str,
    subs: &[Expr],
    value: Value,
    line: u32,
    col: u32,
) -> Result<(), McError> {
    // پیام نام‌دارِ مسیر قدیمی حفظ می‌شود (رفتار 2.8 — رگرسیون‌ها به همین
    // متن گره خورده‌اند)؛ خودِ نوشتن در indexed_write_value است (ROADMAP 2.10).
    if let Some(v) = env.get(name) {
        if !matches!(v, Value::Double(_) | Value::Char(_)) {
            return Err(McError::new(
                ids::TYPE_ERROR,
                line,
                col,
                format!("'{name}' is not a numeric or char array; indexed assignment needs double/char."),
                format!("«{name}» عددی نیست؛ انتساب اندیس‌دار نیاز به آرایه عددی دارد."),
            ));
        }
    }
    let out = indexed_write_value(
        env.get(name).cloned(),
        subs,
        &value,
        env,
        depth_of(line),
        line,
        col,
    )?;
    env.set(name.to_string(), out);
    Ok(())
}

fn depth_of(_line: u32) -> usize {
    0 // اندیس‌ها بدون تو در توی خاص ارزیابی می‌شوند
}

// ---------- انتساب فیلد ساختار (ROADMAP 2.9 — تسک ۱) ----------

/// نام فیلد از FieldName — نامی مستقیم؛ پویا با ارزیابی عبارت به char.
fn resolve_field_name(
    field: &FieldName,
    env: &mut Environment,
    depth: usize,
    line: u32,
    col: u32,
) -> Result<String, McError> {
    match field {
        FieldName::Name(n) => Ok(n.clone()),
        FieldName::Dynamic(e) => {
            let v = eval_expr(e, env, depth)?;
            match v {
                Value::Char(s) => Ok(s),
                _ => Err(McError::new(
                    ids::TYPE_ERROR,
                    line,
                    col,
                    "Dynamic field name s.(f) needs a char value (like f = 'name').",
                    "نام فیلد پویای s.(f) به مقدار char نیاز دارد (مثل f = 'name').",
                )),
            }
        }
    }
}

/// گارد عمق تودرتویی — سقف منابع mc-core (روح گام 2.5؛ انحراف مستند).
/// ساختنده‌های builtin (struct/setfield) هم از همین گارد می‌گذرند.
pub(crate) fn ensure_struct_depth(root: &Value, line: u32, col: u32) -> Result<(), McError> {
    if struct_depth(root) > MAX_STRUCT_DEPTH {
        return Err(McError::new(
            ids::UNSUPPORTED,
            line,
            col,
            format!(
                "Struct nesting exceeds the maximum depth of {MAX_STRUCT_DEPTH} (resource guard)."
            ),
            format!("تودرتویی ساختار از حداکثر عمق {MAX_STRUCT_DEPTH} بیشتر شد (گارد منابع)."),
        ));
    }
    Ok(())
}

/// ساختارِ متغیر نام‌دار — تعریف‌نشده = ساختار تازه (auto-vivification متلب:
/// `q.a = 1` خودِ q را می‌سازد)؛ نوع دیگر = خطای متلب‌گونه.
fn load_struct_or_new(
    env: &mut Environment,
    name: &str,
    line: u32,
    col: u32,
) -> Result<StructVal, McError> {
    match env.get(name) {
        Some(Value::Struct(s)) => Ok(s.clone()),
        None => Ok(StructVal::new()),
        Some(other) => Err(McError::new(
            ids::TYPE_ERROR,
            line,
            col,
            format!(
                "'{name}' is a {}; fields can only be assigned on structs.",
                other.class_name()
            ),
            format!(
                "«{name}» از نوع {} است؛ فیلد فقط روی ساختارها انتساب می‌پذیرد.",
                other.class_name()
            ),
        )),
    }
}

/// ساختار موجود در مسیر فیلد (بدون ساخت خودکار) — None = متغیر/فیلد غایب
/// (زنجیره با auto-viv ساخته می‌شود)؛ نوع غیرساختاری = خطا.
fn struct_at_path(
    env: &mut Environment,
    place: &Expr,
    depth: usize,
    line: u32,
    col: u32,
) -> Result<Option<StructVal>, McError> {
    match place {
        Expr::Var { name, .. } => match env.get(name) {
            Some(Value::Struct(s)) => Ok(Some(s.clone())),
            None => Ok(None),
            Some(other) => Err(McError::new(
                ids::TYPE_ERROR,
                line,
                col,
                format!(
                    "'{name}' is a {}; nested field assignment needs a struct.",
                    other.class_name()
                ),
                format!(
                    "«{name}» از نوع {} است؛ انتساب فیلد تودرتو به ساختار نیاز دارد.",
                    other.class_name()
                ),
            )),
        },
        Expr::Field { base, field, .. } => {
            let parent = struct_at_path(env, base, depth, line, col)?;
            let Some(parent) = parent else {
                // والد غایب — کل زنجیره با انتساب ساخته می‌شود (رفتار متلب)
                return Ok(None);
            };
            let fname = resolve_field_name(field, env, depth, line, col)?;
            match parent.get(&fname) {
                Some(Value::Struct(s)) => Ok(Some(s.clone())),
                None => Ok(None),
                Some(other) => Err(McError::new(
                    ids::TYPE_ERROR,
                    line,
                    col,
                    format!(
                        "Field '{fname}' is a {}; a nested struct is required.",
                        other.class_name()
                    ),
                    format!(
                        "فیلد «{fname}» از نوع {} است؛ اینجا ساختار تودرتو لازم است.",
                        other.class_name()
                    ),
                )),
            }
        }
        _ => Err(McError::new(
            ids::TYPE_ERROR,
            line,
            col,
            "Invalid assignment target: the base of a field assignment must be a variable or a field path.",
            "هدف انتساب نامعتبر: پایهٔ انتساب فیلد باید متغیر یا مسیر فیلد باشد.",
        )),
    }
}

/// `place.field = value` — بازگشتی: زیرساختارِ جایگاه گرفته/ساخته می‌شود،
/// فیلد در آن ست می‌شود و کل زنجیره به والد نوشته می‌شود (auto-vivification).
/// نام متغیر ریشه برای نمایش برگردانده می‌شود؛ عمق فقط یک‌بار در ریشه چک می‌شود.
fn field_assign(
    env: &mut Environment,
    place: &Expr,
    field: &FieldName,
    value: Value,
    line: u32,
    col: u32,
    depth: usize,
) -> Result<String, McError> {
    let fname = resolve_field_name(field, env, depth, line, col)?;
    // نام داینامیک هم باید شناسهٔ معتبر متلب باشد (مسیر نامی از پارسر تضمین
    // شده است؛ اکتاو نام‌های دلخواه می‌پذیرد — دلتای اوراکل مستند)
    if !crate::registry_p2::valid_field_name(&fname) {
        return Err(McError::new(
            ids::TYPE_ERROR,
            line,
            col,
            format!(
                "'{fname}' is not a valid field name (a letter, then letters/digits/underscore; at most 63 chars)."
            ),
            format!(
                "«{fname}» نام فیلد معتبری نیست (یک حرف، سپس حرف/رقم/زیرخط؛ حداکثر ۶۳ نویسه)."
            ),
        ));
    }
    match place {
        Expr::Var { name, .. } => {
            let mut st = load_struct_or_new(env, name, line, col)?;
            st.set(&fname, value);
            let root = Value::Struct(st);
            ensure_struct_depth(&root, line, col)?;
            env.set(name.clone(), root);
            Ok(name.clone())
        }
        Expr::Field {
            base,
            field: inner_field,
            ..
        } => {
            // مقدارِ خودِ این جایگاه (place = مسیر کامل) باید ساختار باشد یا
            // غایب (auto-viv)؛ فیلد در آن ست می‌شود و نتیجه بازگشتی در جایگاه
            // می‌نشیند. اینجا فقط همین سطح دست می‌خورد — نه ساختارِ والد.
            let mut inner = struct_at_path(env, place, depth, line, col)?.unwrap_or_default();
            inner.set(&fname, value);
            field_assign(
                env,
                base,
                inner_field,
                Value::Struct(inner),
                line,
                col,
                depth,
            )
        }
        _ => Err(McError::new(
            ids::TYPE_ERROR,
            line,
            col,
            "Invalid assignment target: field assignment needs a variable or field path on the left.",
            "هدف انتساب نامعتبر: انتساب فیلد در سمت چپ به متغیر یا مسیر فیلد نیاز دارد.",
        )),
    }
}

/// کلمات فرمان `format` → حالت نمایش (ROADMAP 2.4 — آیتم ۱۱).
/// `None` = پذیرش بی‌اثر (`compact`/`loose` — فاصله‌گذاری سطح UI است و
/// حالت عددی را تغییر نمی‌دهد، رفتار متلب). کلمهٔ ناشناخته → خطای صادقانه.
fn parse_format_words(
    words: &[String],
    line: u32,
    col: u32,
) -> Result<Option<crate::format::NumFormat>, McError> {
    use crate::format::NumFormat;
    let joined = words.join(" ");
    match joined.as_str() {
        "" => Ok(Some(NumFormat::Short)), // `format` برهنه = بازگشت به پیش‌فرض
        "short" => Ok(Some(NumFormat::Short)),
        "long" => Ok(Some(NumFormat::Long)),
        "short e" => Ok(Some(NumFormat::ShortE)),
        "long e" => Ok(Some(NumFormat::LongE)),
        "short g" => Ok(Some(NumFormat::ShortG)),
        "long g" => Ok(Some(NumFormat::LongG)),
        "compact" | "loose" => Ok(None),
        other => Err(McError::new(
            ids::UNSUPPORTED,
            line,
            col,
            format!(
                "format: unknown mode '{other}'. Supported: short, long, short e, long e, short g, long g, compact, loose."
            ),
            format!(
                "format: حالت ناشناخته «{other}». حالت‌های پشتیبانی‌شده: short، long، short e، long e، short g، long g، compact، loose."
            ),
        )),
    }
}

/// IndexKind: اسکالر 1-پایه، تمام-بعد (:)، یا برداری با شکل اصلی اندیس.
/// شکل (`rows`,`cols`) برای قاعدهٔ جهت‌گیری متلب (بند ۵ ممیزی — ROADMAP 2.4)
/// حفظ می‌شود: نتیجهٔ اندیس‌گذاری خطی از جهتِ خود اندیس پیروی می‌کند
/// (`a(end:-1:1)` روی بردار سطری سطری می‌ماند؛ `a([1;3])` ستونی می‌دهد).
#[derive(Clone)]
enum Idx {
    Scalar(usize),
    All,
    Vector {
        idx: Vec<usize>,
        rows: usize,
        cols: usize,
    },
}

/// ارزیابی زیرنویس‌ها با پشتیبانی `end` و `:` (سند ۰۳ §6).
fn resolve_subs(
    target: &Matrix,
    subs: &[Expr],
    env: &mut Environment,
    depth: usize,
) -> Result<Vec<Idx>, McError> {
    let _ = target; // dims فقط برای خوانایی امضا؛ end در eval_expr_end حل می‌شود
    subs.iter()
        .enumerate()
        .map(|(si, e)| {
            if let Expr::Var { name, .. } = e {
                if name == ":" {
                    return Ok(Idx::All);
                }
            }
            let v = eval_expr_end(e, env, depth, target, si)?;
            match &v {
                Value::Double(m) if m.is_scalar() => {
                    let idx = m.get(0, 0);
                    if idx <= 0.0 || idx.fract() != 0.0 {
                        return Err(McError::new(
                            ids::TYPE_ERROR,
                            0,
                            0,
                            "Indices must be positive integers (1-based).",
                            "اندیس‌ها باید عدد صحیح مثبت باشند (1-پایه).",
                        ));
                    }
                    Ok(Idx::Scalar(idx as usize))
                }
                Value::Double(m) => {
                    let mut list = Vec::new();
                    for i in 0..m.numel() {
                        let idx = m.get_linear(i);
                        if idx <= 0.0 || idx.fract() != 0.0 {
                            return Err(McError::new(
                                ids::TYPE_ERROR,
                                0,
                                0,
                                "Indices must be positive integers (1-based).",
                                "اندیس‌ها باید عدد صحیح مثبت باشند (1-پایه).",
                            ));
                        }
                        list.push(idx as usize);
                    }
                    Ok(Idx::Vector {
                        idx: list,
                        rows: m.rows(),
                        cols: m.cols(),
                    })
                }
                _ => Err(McError::new(
                    ids::UNSUPPORTED,
                    0,
                    0,
                    "Unsupported subscript type.",
                    "نوع زیرنویس پشتیبانی نمی‌شود.",
                )),
            }
        })
        .collect()
}

/// اندازه بعد برای `end` — بعد 1: rows، بعد 2: cols؛ خطی: numel.
fn end_size(target: &Matrix, sub_i: usize) -> usize {
    if target.rows() == 1 || target.cols() == 1 {
        target.numel()
    } else if sub_i == 0 {
        target.rows()
    } else {
        target.cols()
    }
}

// ---------- خواندن اندیس‌دار در عبارت: از Call استفاده می‌کنیم ----------

fn index_read(
    name: &str,
    args: &[Expr],
    env: &mut Environment,
    depth: usize,
    line: u32,
    col: u32,
) -> Result<Value, McError> {
    let target = match env.get(name) {
        Some(Value::Double(m)) => m.clone(),
        Some(_) => {
            return Err(McError::new(
                ids::TYPE_ERROR,
                line,
                col,
                format!("'{name}' is not numeric; indexing needs a numeric array."),
                format!("«{name}» عددی نیست؛ اندیس‌گذاری نیاز به آرایه عددی دارد."),
            ))
        }
        None => {
            return Err(undefined_var(name, line, col));
        }
    };
    let resolved = resolve_subs(&target, args, env, depth)?;
    index_double(&target, &resolved, line, col)
}

/// هستهٔ اندیس‌گذاری ماتریس — جدا شده تا مسیر char (index_read_char) نیز از
/// همان ماشین جهت‌گیری نتیجه (بند ۵ ممیزی — ROADMAP 2.4) استفاده کند.
fn index_double(m: &Matrix, resolved: &[Idx], line: u32, col: u32) -> Result<Value, McError> {
    match resolved.len() {
        1 => match &resolved[0] {
            Idx::All => {
                if m.rows() == 1 || m.cols() == 1 {
                    Ok(Value::Double(m.clone()))
                } else {
                    // A(:) = ستون‌بندی همه عناصر (سند ۰۴ §2)
                    let data: Vec<f64> = (0..m.numel()).map(|i| m.get_linear(i)).collect();
                    Ok(Value::Double(Matrix::from_columns(m.numel(), 1, data)))
                }
            }
            Idx::Scalar(i) => {
                linear_read(m, *i, line, col).map(|v| Value::Double(Matrix::scalar(v)))
            }
            // بند ۵ ممیزی (ROADMAP 2.4): نتیجهٔ اندیس‌گذاری خطی از جهتِ اندیس
            // پیروی می‌کند — خروجی همان شکل خود اندیس است (متلب).
            Idx::Vector {
                idx: list,
                rows,
                cols,
            } => {
                let mut out = Matrix::full(*rows, *cols, 0.0);
                for (k, i) in list.iter().enumerate() {
                    out.set_linear(k, linear_read(m, *i, line, col)?);
                }
                Ok(Value::Double(out))
            }
        },
        2 => match (&resolved[0], &resolved[1]) {
            (Idx::All, Idx::All) => Ok(Value::Double(m.clone())),
            (ri, ci) => {
                let rows = idx_list(ri, m.rows());
                let cols = idx_list(ci, m.cols());
                // بند ۵ ممیزی: اسکالر + زیرنویس برداری → نتیجه جهتِ زیرنویس
                // برداری را می‌گیرد (`A(1,[1;2])` ستونی 2×1؛ `A([1 2],1)` سطری 1×2).
                // در غیر این صورت شکل = طول‌ها (طبیعی ماتریسی).
                let (out_rows, out_cols) = match (ri, ci) {
                    (Idx::Scalar(_), Idx::Vector { rows, cols, .. }) => (*rows, *cols),
                    (Idx::Vector { rows, cols, .. }, Idx::Scalar(_)) => (*rows, *cols),
                    _ => (rows.len(), cols.len()),
                };
                let mut out = Matrix::full(out_rows, out_cols, 0.0);
                // پرکردن خطیِ ستونیِ گرید اندیس — با شکل خروجیِ جهت‌دار هم‌خوان است
                // (اسکالر×بردار: موقعیت k در گرید = جای ستونی واقعی نتیجه).
                for (k, r) in rows.iter().enumerate() {
                    for (l, c) in cols.iter().enumerate() {
                        if r >= &m.rows() || c >= &m.cols() {
                            return Err(index_out_of_bounds(line, col, *r, *c));
                        }
                        out.set_linear(k + l * rows.len(), m.get(*r, *c));
                    }
                }
                Ok(Value::Double(out))
            }
        },
        n => Err(McError::new(
            ids::UNSUPPORTED,
            line,
            col,
            format!("Indexing with {n} subscripts is not supported (max 2)."),
            format!("اندیس‌گذاری با {n} زیرنویس پشتیبانی نمی‌شود (حداکثر 2)."),
        )),
    }
}

/// اندیس‌گذاری خواندنی روی رشتهٔ کاراکتری (ROADMAP 2.8 — بند ۶/۱۲):
/// s(2)→'e'، s([1 3])→'hl'، s(end:-1:1)→'olleh'. ماشین اندیس روی ردیف کدها
/// اجرا می‌شود و نتیجهٔ تک‌ردیفه به char برمی‌گردد؛ نتیجهٔ ستونی = char
/// دوبعدی → خطای صادقانهٔ E4.
fn index_read_char(
    name: &str,
    args: &[Expr],
    env: &mut Environment,
    depth: usize,
    line: u32,
    col: u32,
) -> Result<Value, McError> {
    let s = match env.get(name) {
        Some(Value::Char(s)) => s.clone(),
        _ => return Err(undefined_var(name, line, col)),
    };
    let target = Value::char_codes_row(&s);
    let resolved = resolve_subs(&target, args, env, depth)?;
    let v = index_double(&target, &resolved, line, col)?;
    if let Value::Double(m) = &v {
        if m.rows() == 1 {
            let mut out = String::with_capacity(m.cols());
            for i in 0..m.cols() {
                out.push(Value::char_from_code(m.get(0, i), line, col)?);
            }
            return Ok(Value::Char(out));
        }
    }
    Err(McError::new(
        ids::UNSUPPORTED,
        line,
        col,
        "Column-shaped indexing of a char row yields a 2-D char array; expansion E4.",
        "اندیس‌گذاری ستونیِ رشتهٔ char آرایهٔ دوبعدی می‌سازد؛ انبساط E4 است.",
    ))
}

fn idx_list(kind: &Idx, size: usize) -> Vec<usize> {
    match kind {
        Idx::All => (0..size).collect(),
        Idx::Scalar(i) => vec![i.saturating_sub(1)],
        Idx::Vector { idx: v, .. } => v.iter().map(|i| i.saturating_sub(1)).collect(),
    }
}

/// اعمال زیرنویس روی یک «مقدار» — هستهٔ اندیس‌گذاری زنجیره‌ای (ROADMAP 2.10):
/// مقدار موقتِ حاصل عبارت (لفظ، فراخوانی، فیلد، پرانتز) همان ماشین نام‌محور را
/// می‌گیرد؛ `end` بر اساس شکل همان مقدال حل می‌شود.
fn index_value(
    v: &Value,
    args: &[Expr],
    env: &mut Environment,
    depth: usize,
    line: u32,
    col: u32,
) -> Result<Value, McError> {
    match v {
        Value::Double(m) => {
            let resolved = resolve_subs(m, args, env, depth)?;
            index_double(m, &resolved, line, col)
        }
        // char: همان قرارداد 2.8 — ردیف کدها + بازگشت char تک‌ردیفی
        Value::Char(s) => {
            let target = Value::char_codes_row(s);
            let resolved = resolve_subs(&target, args, env, depth)?;
            let out = index_double(&target, &resolved, line, col)?;
            if let Value::Double(m) = &out {
                if m.rows() == 1 {
                    let mut st = String::with_capacity(m.cols());
                    for i in 0..m.cols() {
                        st.push(Value::char_from_code(m.get(0, i), line, col)?);
                    }
                    return Ok(Value::Char(st));
                }
            }
            Err(McError::new(
                ids::UNSUPPORTED,
                line,
                col,
                "Column-shaped indexing of a char row yields a 2-D char array; expansion E4.",
                "اندیس‌گذاری ستونیِ رشتهٔ char آرایهٔ دوبعدی می‌سازد؛ انبساط E4 است.",
            ))
        }
        // مختلط: پشتیبانی مستقیم نیست (همان مرز نام‌محور) — اول به متغیر/فیلد
        Value::Complex(_) => Err(McError::new(
            ids::UNSUPPORTED,
            line,
            col,
            "Indexing a complex value directly is not supported yet; assign it to a variable first.",
            "اندیس‌گذاری مستقیم مقدار مختلط هنوز پشتیبانی نمی‌شود؛ اول به متغیر منتسب کنید.",
        )),
        Value::Anon(_) => Err(McError::new(
            ids::TYPE_ERROR,
            line,
            col,
            "A function handle is not indexable.",
            "function_handle اندیس‌پذیر نیست.",
        )),
        // () روی ساختار = آرایهٔ ساختاری = انبساط E4
        Value::Struct(_) => Err(McError::new(
            ids::UNSUPPORTED,
            line,
            col,
            "Paren indexing on a struct is struct-array indexing; struct arrays are expansion E4.",
            "اندیس‌گذاری پرانتزی روی ساختار = اندیس آرایهٔ ساختاری؛ آرایهٔ ساختاری انبساط E4 است.",
        )),
    }
}

/// هستهٔ مقدار-محورِ انتساب اندیس‌دار (ROADMAP 2.10): مقدار فعلی (None = از
/// خالی auto-viv، رفتار متلب `s.a(3)=5` → `[0 0 5]`) + زیرنویس‌ها + مقدار
/// تازه → مقدار نهایی. مسیر نام‌محور (`indexed_assign`) و مسیر فیلدی
/// (`indexed_field_assign`) هر دو از همین هسته استفاده می‌کنند.
fn indexed_write_value(
    current: Option<Value>,
    subs: &[Expr],
    value: &Value,
    env: &mut Environment,
    depth: usize,
    line: u32,
    col: u32,
) -> Result<Value, McError> {
    let vmat = match value {
        Value::Double(m) => m.clone(),
        Value::Char(s) => Value::char_codes_row(s),
        _ => {
            return Err(McError::new(
                ids::TYPE_ERROR,
                line,
                col,
                "Only numeric or char row values can be assigned to indexed targets.",
                "انتساب اندیس‌دار فقط با مقدار عددی یا رشتهٔ کاراکتری ممکن است.",
            ))
        }
    };
    let char_target = matches!(current, Some(Value::Char(_)));
    let cur = match current {
        Some(Value::Double(m)) => m,
        Some(Value::Char(s)) => Value::char_codes_row(&s),
        None => Matrix::full(0, 0, 0.0),
        Some(_) => {
            return Err(McError::new(
                ids::TYPE_ERROR,
                line,
                col,
                "The indexing target is not a numeric array or char row.",
                "هدف اندیس‌گذاری آرایهٔ عددی یا ردیف char نیست.",
            ))
        }
    };
    let resolved = resolve_subs(&cur, subs, env, depth)?;
    // انتساب خالی `x(i)=[]` — سمانتیک حذف متلب (میراث مسیر نام‌محور)
    let out = if vmat.is_empty() {
        delete_subscripts(&cur, &resolved, line, col)?
    } else {
        let mut out = cur.clone();
        match resolved.len() {
            1 => linear_assign(&mut out, &resolved[0], &vmat)?,
            2 => matrix_assign(&mut out, &resolved[0], &resolved[1], &vmat, line, col)?,
            n => {
                return Err(McError::new(
                    ids::UNSUPPORTED,
                    line,
                    col,
                    format!("Indexed assignment with {n} subscripts is not supported (max 2)."),
                    format!("انتساب اندیس‌دار با {n} زیرنویس پشتیبانی نمی‌شود (حداکثر 2)."),
                ))
            }
        }
        out
    };
    if char_target {
        Ok(Value::Char(char_row_from_matrix(&out, line, col)?))
    } else {
        Ok(Value::Double(out))
    }
}

/// `s.f(i) = v` — مقدار فعلی فیلد (یا None برای auto-viv متلب‌گونه:
/// `t.b(3)=5` → `[0 0 5]` — اوراکل Octave 10.3) گرفته می‌شود، هستهٔ انتساب
/// اندیس‌دار رویش اجرا می‌شود و نتیجه در همان مسیر فیلد بازنوشته می‌شود.
/// نام متغیر ریشه برای نمایش برگردانده می‌شود (ROADMAP 2.10).
// ۸ آرگومان = الگوی این امضا (محیط + مسیر + زیرنویس + مقدار + مکان/عمق) —
// همان استثنای ثبت‌شدهٔ solvers/ode.rs
#[allow(clippy::too_many_arguments)]
fn indexed_field_assign(
    env: &mut Environment,
    place: &Expr,
    field: &FieldName,
    subs: &[Expr],
    value: Value,
    line: u32,
    col: u32,
    depth: usize,
) -> Result<String, McError> {
    let fname = resolve_field_name(field, env, depth, line, col)?;
    if !crate::registry_p2::valid_field_name(&fname) {
        return Err(McError::new(
            ids::TYPE_ERROR,
            line,
            col,
            format!(
                "'{fname}' is not a valid field name (a letter, then letters/digits/underscore; at most 63 chars)."
            ),
            format!(
                "«{fname}» نام فیلد معتبری نیست (یک حرف، سپس حرف/رقم/زیرخط؛ حداکثر ۶۳ نویسه)."
            ),
        ));
    }
    // ساختارِ حامل فیلد در مسیر (None = کل مسیر غایب → auto-viv)
    let container = struct_at_path(env, place, depth, line, col)?;
    let cur = container.as_ref().and_then(|c| c.get(&fname)).cloned();
    let out = indexed_write_value(cur, subs, &value, env, depth, line, col)?;
    field_assign(env, place, field, out, line, col, depth)
}

fn linear_read(m: &Matrix, i: usize, line: u32, col: u32) -> Result<f64, McError> {
    if i == 0 || i > m.numel() {
        return Err(McError::new(
            ids::INDEX_OUT_OF_BOUNDS,
            line,
            col,
            format!("Linear index {i} is out of bounds (numel = {}).", m.numel()),
            format!("اندیس خطی {i} خارج از محدوده است (numel = {}).", m.numel()),
        ));
    }
    Ok(m.get_linear(i - 1))
}

/// حذف با انتساب خالی — `x(i)=[]` / `A(i)=[]` / `A(i,:)=[]` / `A(:,j)=[]`.
/// متلب: روی بردار المان حذف می‌شود (جهت حفظ می‌شود)؛ اندیس خطی روی ماتریس
/// «سطرِ» حاوی المان را حذف می‌کند؛ `x(:)=[]` همه‌چیز را حذف می‌کند (0×0).
fn delete_subscripts(
    cur: &Matrix,
    resolved: &[Idx],
    line: u32,
    col: u32,
) -> Result<Matrix, McError> {
    let out_of_range = |i: usize, hi: usize| {
        McError::new(
            ids::UNSUPPORTED,
            line,
            col,
            format!("Deletion index {i} out of range 1..={hi}."),
            format!("اندیس حذف {i} خارج از بازهٔ ۱ تا {hi} است."),
        )
    };
    match resolved {
        // x(:)=[] → 0×0 خالی (رفتار متلب/اکتاو)
        [Idx::All] => Ok(Matrix::full(0, 0, 0.0)),
        // حذف با اندیس خطی (تک‌زیرنویس)
        [idx] => {
            let mut list: Vec<usize> = match idx {
                Idx::Scalar(i) => vec![*i],
                Idx::Vector { idx: l, .. } => l.clone(),
                Idx::All => (1..=cur.numel()).collect(),
            };
            if list.is_empty() {
                // x([])=[] بی‌اثر است (رفتار متلب)
                return Ok(cur.clone());
            }
            let numel = cur.numel();
            for &i in &list {
                if i == 0 || i > numel {
                    return Err(out_of_range(i, numel));
                }
            }
            let mut idxs: Vec<usize> = list.drain(..).map(|i| i - 1).collect();
            idxs.sort_unstable();
            idxs.dedup();
            if cur.rows() == 1 {
                Ok(cur.remove_cols(&idxs))
            } else if cur.cols() == 1 {
                Ok(cur.remove_rows(&idxs))
            } else {
                // متلب: اندیس خطی روی ماتریس → حذف سطرِ حاوی المان
                // (column-major: سطرِ المان k ام = k % rows)
                let rows: Vec<usize> = idxs.iter().map(|&k| k % cur.rows()).collect();
                Ok(cur.remove_rows(&rows))
            }
        }
        // A(i,:)=[] — حذف سطر(ها)
        [ri, Idx::All] => {
            let mut rows: Vec<usize> = match ri {
                Idx::All => (0..cur.rows()).collect(),
                Idx::Scalar(i) => {
                    if *i == 0 || *i > cur.rows() {
                        return Err(out_of_range(*i, cur.rows()));
                    }
                    vec![*i - 1]
                }
                Idx::Vector { idx: l, .. } => {
                    for &i in l {
                        if i == 0 || i > cur.rows() {
                            return Err(out_of_range(i, cur.rows()));
                        }
                    }
                    l.iter().map(|&i| i - 1).collect()
                }
            };
            if rows.is_empty() {
                return Ok(cur.clone());
            }
            rows.sort_unstable();
            rows.dedup();
            Ok(cur.remove_rows(&rows))
        }
        // A(:,j)=[] — حذف ستون(ها)
        [Idx::All, ci] => {
            let mut cols: Vec<usize> = match ci {
                Idx::All => (0..cur.cols()).collect(),
                Idx::Scalar(i) => {
                    if *i == 0 || *i > cur.cols() {
                        return Err(out_of_range(*i, cur.cols()));
                    }
                    vec![*i - 1]
                }
                Idx::Vector { idx: l, .. } => {
                    for &i in l {
                        if i == 0 || i > cur.cols() {
                            return Err(out_of_range(i, cur.cols()));
                        }
                    }
                    l.iter().map(|&i| i - 1).collect()
                }
            };
            if cols.is_empty() {
                return Ok(cur.clone());
            }
            cols.sort_unstable();
            cols.dedup();
            Ok(cur.remove_cols(&cols))
        }
        _ => Err(McError::new(
            ids::UNSUPPORTED,
            line,
            col,
            "Empty ([]) assignment on 2-D subscripts requires a colon: A(i,:)=[] or A(:,j)=[]."
                .to_string(),
            "انتساب خالی به زیرنویس دوبعدی نیازمند دونقطه است: A(i,:)=[] یا A(:,j)=[].".to_string(),
        )),
    }
}

fn linear_assign(m: &mut Matrix, idx: &Idx, v: &Matrix) -> Result<(), McError> {
    match idx {
        Idx::Scalar(i) => {
            if *i > m.numel() {
                grow_linear(m, *i)?;
            }
            m.set_linear(i - 1, v.get(0, 0));
            Ok(())
        }
        Idx::All => {
            for k in 0..m.numel().max(v.numel()) {
                if k < v.numel() {
                    if k >= m.numel() {
                        grow_linear(m, k + 1)?;
                    }
                    m.set_linear(k, v.get_linear(k));
                }
            }
            Ok(())
        }
        Idx::Vector { idx: list, .. } => {
            for (k, i) in list.iter().enumerate() {
                if *i > m.numel() {
                    grow_linear(m, *i)?;
                }
                let val = if v.is_scalar() {
                    v.get(0, 0)
                } else {
                    v.get_linear(k)
                };
                m.set_linear(i - 1, val);
            }
            Ok(())
        }
    }
}

fn grow_linear(m: &mut Matrix, new_len: usize) -> Result<(), McError> {
    // رشد با اندیس خطی — جهت حفظ می‌شود (رفتار متلب): بردار سطری سطری
    // می‌ماند (`x=10; x(3)=5` → 1×3)، ستونی ستونی؛ ماتریس با افزایش سطر
    // رشد می‌کند (A(5)=9 روی 2×2 → 3×2). رشد از خالی → 1×new_len.
    let (new_rows, new_cols) = if m.rows() == 1 {
        (1, new_len)
    } else if m.cols() == 1 {
        (new_len, 1)
    } else if m.cols() == 0 {
        (1, new_len)
    } else {
        (new_len.div_ceil(m.cols()), m.cols())
    };
    // بودجهٔ حافظه (ROADMAP 2.5): new_len از اندیس کاربر می‌آید (`x(1e9)=5`)
    // — چکِ قبل از تخصیص، وگرنه resize پروسه را abort می‌کند.
    budget::check_elements(new_rows, new_cols)?;
    let mut data: Vec<f64> = (0..m.numel()).map(|i| m.get_linear(i)).collect();
    data.resize(new_rows * new_cols, 0.0);
    *m = Matrix::from_columns(new_rows, new_cols, data);
    Ok(())
}

fn matrix_assign(
    m: &mut Matrix,
    ri: &Idx,
    ci: &Idx,
    v: &Matrix,
    line: u32,
    col: u32,
) -> Result<(), McError> {
    let rows = idx_list(ri, m.rows());
    let cols = idx_list(ci, m.cols());
    // رشد شکل اگر لازم باشد (فقط اندیس‌های اسکالر)
    if let (Idx::Scalar(r), Idx::Scalar(c)) = (ri, ci) {
        let new_rows = m.rows().max(*r);
        let new_cols = m.cols().max(*c);
        if (new_rows, new_cols) != (m.rows(), m.cols()) {
            // بودجهٔ حافظه (ROADMAP 2.5): ابعاد جدید از اندیس کاربر است.
            let mut nd = Matrix::try_full(new_rows, new_cols, 0.0)?;
            for cc in 0..m.cols() {
                for rr in 0..m.rows() {
                    nd.set(rr, cc, m.get(rr, cc));
                }
            }
            *m = nd;
        }
        m.set(r - 1, c - 1, v.get(0, 0));
        return Ok(());
    }
    if (rows.len(), cols.len()) != (v.rows(), v.cols()) {
        return Err(McError::new(
            ids::SIZE_MISMATCH,
            line,
            col,
            format!(
                "Assignment size mismatch: target {r}x{c}, value {vr}x{vc}.",
                r = rows.len(),
                c = cols.len(),
                vr = v.rows(),
                vc = v.cols()
            ),
            format!(
                "ناکامی ابعاد انتساب: هدف {r}×{c}، مقدار {vr}×{vc}.",
                r = rows.len(),
                c = cols.len(),
                vr = v.rows(),
                vc = v.cols()
            ),
        ));
    }
    for (k, r) in rows.iter().enumerate() {
        for (l, c) in cols.iter().enumerate() {
            if r >= &m.rows() || c >= &m.cols() {
                return Err(index_out_of_bounds(line, col, *r, *c));
            }
            m.set(*r, *c, v.get(k, l));
        }
    }
    Ok(())
}

fn index_out_of_bounds(line: u32, col: u32, r: usize, c: usize) -> McError {
    McError::new(
        ids::INDEX_OUT_OF_BOUNDS,
        line,
        col,
        format!("Index ({r},{c}) out of bounds."),
        format!("اندیس ({r},{c}) خارج از محدوده است."),
    )
}

fn undefined_var(name: &str, line: u32, col: u32) -> McError {
    McError::new(
        ids::UNDEFINED_VARIABLE,
        line,
        col,
        format!("'{name}' is undefined."),
        format!("«{name}» تعریف نشده است."),
    )
    // did-you-mean برای builtinها (سند ۰۸ DoD P1: `lenght` → `length`)
    .with_suggestion(registry::suggest_builtin(name).unwrap_or_default())
}

/// خطای undefined با did-you-mean کامل: builtin، متغیرهای workspace و
/// توابع/اسکریپت‌های ثبت‌شده (سند ۰۲ بخش ۹) — نزدیک‌ترین در کل دامنه‌ها.
fn undefined_var_in(name: &str, line: u32, col: u32, env: &Environment) -> McError {
    // هر دامنه نزدیک‌ترین خودش را می‌دهد؛ کمینهٔ کل = کمینهٔ این سه
    let candidates = [
        registry::suggest_builtin(name),
        env.suggest_name(name),
        env.suggest_function(name),
    ];
    let suggestion = candidates
        .into_iter()
        .flatten()
        .min_by_key(|c| (crate::env::levenshtein(name, c), c.clone()));
    McError::new(
        ids::UNDEFINED_VARIABLE,
        line,
        col,
        format!("'{name}' is undefined."),
        format!("«{name}» تعریف نشده است."),
    )
    .with_suggestion(suggestion.unwrap_or_default())
}

/// خطای «تابع شناخته‌نشده» در موقعیت فراخوانی (ROADMAP 2.11 — MC:E1002،
/// کاتالوگ سند ۰۳ §۱۴): نام در هیچ دامنه‌ای نبود و با پرانتز فراخوانی شده.
/// did-you-mean همان سه دامنه (سند ۰۲ §9).
fn undefined_fn_in(name: &str, line: u32, col: u32, env: &Environment) -> McError {
    let candidates = [
        registry::suggest_builtin(name),
        env.suggest_name(name),
        env.suggest_function(name),
    ];
    let suggestion = candidates
        .into_iter()
        .flatten()
        .min_by_key(|c| (crate::env::levenshtein(name, c), c.clone()));
    McError::new(
        ids::UNDEFINED_FUNCTION,
        line,
        col,
        format!("Function '{name}' not found."),
        format!("تابع «{name}» یافت نشد."),
    )
    .with_suggestion(suggestion.unwrap_or_default())
}

/// «آرگومان ورودی کم است» — رفتار متلب برای نام برهنهٔ تابع پارامتردار.
fn not_enough_inputs(name: &str, expected: usize, line: u32, col: u32) -> McError {
    McError::new(
        ids::TYPE_ERROR,
        line,
        col,
        format!("Not enough input arguments. Function '{name}' expects {expected} argument(s)."),
        format!("آرگومان ورودی کم است. تابع «{name}» {expected} آرگومان می‌خواهد."),
    )
}

// ---------- عبارت‌ها ----------

fn eval_expr(expr: &Expr, env: &mut Environment, depth: usize) -> Result<Value, McError> {
    // پشتیبان موقعیت سطح عبارت (ROADMAP 2.11 — بند ۱۴ ممیزی): خطاهای کمکی/
    // builtin بدون موقعیت (0,0) موقعیت عبارت را می‌گیرند؛ خطای کاربر 0:0 نمی‌ماند
    eval_expr_end(expr, env, depth, &Matrix::full(0, 0, 0.0), 0).map_err(|e| {
        let (l, c) = expr_pos(expr);
        with_pos(e, l, c)
    })
}

/// ارزیابی با زمینهٔ `end` — داخل اندیس‌گذاری، `end` به اندازهٔ بعد ترجمه می‌شود.
fn eval_expr_end(
    expr: &Expr,
    env: &mut Environment,
    depth: usize,
    target: &Matrix,
    sub_i: usize,
) -> Result<Value, McError> {
    match expr {
        Expr::Num(v) => Ok(Value::Double(Matrix::scalar(*v))),
        // عدد موهومی `4i` — اسکالر مختلط 0+4i (سند ۰۳ بخش ۳ — P2)
        Expr::Imag(v) => Ok(Value::Complex(mc_core::ComplexMatrix::scalar(0.0, *v))),
        Expr::Str(s) => Ok(Value::Char(s.clone())),
        Expr::End { line, col } => {
            if target.is_empty() {
                return Err(McError::new(
                    ids::TYPE_ERROR,
                    *line,
                    *col,
                    "'end' is only valid inside indexing.",
                    "«end» فقط داخل اندیس‌گذاری معتبر است.",
                ));
            }
            Ok(Value::Double(
                Matrix::scalar(end_size(target, sub_i) as f64),
            ))
        }
        Expr::Var { name, line, col } => match env.get(name) {
            Some(v) => Ok(v.clone()),
            None => {
                // سمانتیک دستوری متلب: true/false بدون پرانتز (سند ۰۳ §12) —
                // فقط صفر-آرگومان‌های بولی؛ بقیه نام‌ها خطا با did-you-mean
                match name.as_str() {
                    "true" => Ok(Value::Double(Matrix::scalar(1.0))),
                    "false" => Ok(Value::Double(Matrix::scalar(0.0))),
                    // ثابت‌های IEEE متلب — فقط وقتی کاربر همان نام را سایه نکرده (سند ۰۳ §3):
                    // pi/eps فقط حروف کوچک؛ Inf/inf و NaN/nan هر دو نگارش (رفتار متلب)
                    "pi" => Ok(Value::Double(Matrix::scalar(std::f64::consts::PI))),
                    "eps" => Ok(Value::Double(Matrix::scalar(f64::EPSILON))),
                    "Inf" | "inf" => Ok(Value::Double(Matrix::scalar(f64::INFINITY))),
                    "NaN" | "nan" => Ok(Value::Double(Matrix::scalar(f64::NAN))),
                    // واحد موهومی — سایه‌پذیر مثل pi (سند ۰۳ §3 — P2)
                    "i" | "j" => Ok(Value::Complex(mc_core::ComplexMatrix::imaginary_unit())),
                    _ => {
                        // نام ثبت‌شده بدون فراخوانی؟ — سمانتیک متلب §9
                        match env.lookup_item(name) {
                            Some(StoredItem::Script(_)) => Err(McError::new(
                                ids::TYPE_ERROR,
                                *line,
                                *col,
                                format!(
                                    "Script '{name}' cannot be used inside an expression. Call it as a statement: {name}"
                                ),
                                format!(
                                    "اسکریپت «{name}» داخل عبارت قابل استفاده نیست؛ به‌صورت گزاره فراخوانی کنید: {name}"
                                ),
                            )),
                            Some(StoredItem::Fn(def)) => {
                                if def.params.is_empty() {
                                    let outs = call_user_fn(
                                        &def,
                                        &[],
                                        1,
                                        env.funcs_clone(),
                                        depth,
                                        *line,
                                        *col,
                                    )?;
                                    if let Some(v) = outs.into_iter().next() {
                                        return Ok(v);
                                    }
                                    Err(McError::new(
                                        ids::TYPE_ERROR,
                                        *line,
                                        *col,
                                        format!("Function '{name}' did not return a value."),
                                        format!("تابع «{name}» مقداری برنگرداند."),
                                    ))
                                } else {
                                    Err(not_enough_inputs(
                                        name,
                                        def.params.len(),
                                        *line,
                                        *col,
                                    ))
                                }
                            }
                            // builtin برهنه = فراخوانی بدون آرگومان؛
                            // پیام خطا از خود builtin می‌آید (رفتار متلب)
                            None => {
                                if let Some(f) = registry::lookup(name) {
                                    f(&[], 1)
                                } else if let Some(mf) =
                                    crate::registry_p2::lookup_multi(name)
                                {
                                    let mut outs = mf(&[], 1)?;
                                    Ok(outs.remove(0))
                                } else {
                                    Err(undefined_var_in(name, *line, *col, env))
                                }
                            }
                        }
                    }
                }
            }
        },
        Expr::Unary { op, operand, .. } => {
            let v = eval_expr_end(operand, env, depth, target, sub_i)?;
            match op {
                UnOp::Pos => Ok(v),
                UnOp::Neg => negate(&v),
                UnOp::Not => match v.as_scalar() {
                    Some(x) => Ok(Value::Double(Matrix::scalar((x == 0.0) as i32 as f64))),
                    None => Err(McError::new(
                        ids::TYPE_ERROR,
                        0,
                        0,
                        "Logical NOT needs a scalar operand.",
                        "NOT منطقی به عملوند اسکالر نیاز دارد.",
                    )),
                },
            }
        }
        Expr::Binary {
            op,
            lhs,
            rhs,
            line,
            col,
        } => {
            let l = eval_expr_end(lhs, env, depth, target, sub_i)?;
            let r = eval_expr_end(rhs, env, depth, target, sub_i)?;
            apply_binary(*op, &l, &r, *line, *col)
        }
        Expr::Compare {
            op,
            lhs,
            rhs,
            line,
            col,
        } => {
            let l = eval_expr_end(lhs, env, depth, target, sub_i)?;
            let r = eval_expr_end(rhs, env, depth, target, sub_i)?;
            apply_compare(*op, &l, &r, *line, *col)
        }
        Expr::Logic { op, lhs, rhs, .. } => {
            // short-circuit — سمانتیک متلب (سند ۰۳ §8)
            let l = eval_expr_end(lhs, env, depth, target, sub_i)?;
            let lb = truthy_scalar(&l)?;
            let result = match op {
                LogicOp::And => {
                    if !lb {
                        false
                    } else {
                        truthy_scalar(&eval_expr_end(rhs, env, depth, target, sub_i)?)?
                    }
                }
                LogicOp::Or => {
                    if lb {
                        true
                    } else {
                        truthy_scalar(&eval_expr_end(rhs, env, depth, target, sub_i)?)?
                    }
                }
            };
            Ok(Value::Double(Matrix::scalar(result as i32 as f64)))
        }
        Expr::Transpose {
            operand,
            conj,
            line,
            col,
        } => {
            let v = eval_expr_end(operand, env, depth, target, sub_i)?;
            match v {
                Value::Double(m) => {
                    let mut out = Matrix::full(m.cols(), m.rows(), 0.0);
                    for c in 0..m.cols() {
                        for r in 0..m.rows() {
                            let x = m.get(r, c);
                            out.set(c, r, x); // حقیقی: مزدوج = همانی (سند ۰۳ §5)
                        }
                    }
                    Ok(Value::Double(out))
                }
                // مختلط: `'` ترانهادهٔ مزدوج، `.'` ترانهادهٔ ساده (سند ۰۳ §5 — P2)
                Value::Complex(z) => Ok(Value::Complex(cx::c_transpose(&z, *conj))),
                Value::Char(s) => {
                    // 1×1 ترانهادهٔ خودش است؛ 1×N (طول ≥ ۲) ترانهاده‌اش N×1 یعنی
                    // آرایهٔ char دوبعدی می‌شود → انبساط E4 با خطای صادقانه
                    // (ROADMAP 2.8 تسک ۱/۳)
                    if s.chars().count() <= 1 {
                        Ok(Value::Char(s))
                    } else {
                        Err(McError::new(
                            ids::UNSUPPORTED,
                            *line,
                            *col,
                            "Transposing a 1×N char row yields an N×1 char array; 2-D char arrays are expansion E4.",
                            "ترانهادهٔ ردیف char با طول ۲ یا بیشتر، آرایهٔ char دوبعدی N×1 می‌سازد؛ char دوبعدی انبساط E4 است.",
                        ))
                    }
                }
                Value::Anon(_) => Err(McError::new(
                    ids::TYPE_ERROR,
                    *line,
                    *col,
                    "Cannot transpose a function handle.",
                    "ترانهاده function_handle ممکن نیست.",
                )),
                Value::Struct(_) => Err(McError::new(
                    ids::TYPE_ERROR,
                    *line,
                    *col,
                    "Cannot transpose a struct.",
                    "ترانهاده ساختار ممکن نیست.",
                )),
            }
        }
        Expr::Matrix { rows, line, col } => build_matrix(rows, env, depth, *line, *col),
        Expr::Range {
            start, step, end, ..
        } => {
            let a = eval_expr_end(start, env, depth, target, sub_i)?;
            let s = match step {
                Some(e) => eval_expr_end(e, env, depth, target, sub_i)?,
                None => Value::Double(Matrix::scalar(1.0)),
            };
            let b = eval_expr_end(end, env, depth, target, sub_i)?;
            range_values(&a, &s, &b)
        }
        Expr::Call {
            name,
            args,
            line,
            col,
            ..
        } => call_named(name, args, env, depth, *line, *col, target),
        Expr::AnonFn { params, body, .. } => {
            // capture-by-value — snapshot فعلی env + مخزن توابع مشترک
            // (بدنه تابع بی‌نام باید بتواند توابع فایلی ثبت‌شده را فراخوانی کند — سند ۰۳ §9)
            let captured: std::collections::HashMap<String, Value> = env.snapshot();
            let funcs_c = env.funcs_clone();
            let params_c = params.clone();
            let body_c = (**body).clone();
            let depth_c = depth;
            let call: mc_core::AnonCall = std::sync::Arc::new(move |args: &[Value]| {
                if args.len() != params_c.len() {
                    return Err(McError::new(
                        ids::TYPE_ERROR,
                        0,
                        0,
                        format!(
                            "Anonymous function expects {} argument(s), got {}.",
                            params_c.len(),
                            args.len()
                        ),
                        format!(
                            "تابع بی‌نام {} آرگومان می‌خواهد، {} دریافت شد.",
                            params_c.len(),
                            args.len()
                        ),
                    ));
                }
                let mut scoped = Environment::new_with_funcs(funcs_c.clone());
                for (k, v) in &captured {
                    scoped.set(k.clone(), v.clone());
                }
                for (p, a) in params_c.iter().zip(args.iter()) {
                    scoped.set(p.clone(), a.clone());
                }
                let body = body_c.clone();
                eval_expr_depth(&body, &mut scoped, depth_c + 1)
            });
            if depth >= MAX_CALL_DEPTH {
                return Err(McError::new(
                    ids::TYPE_ERROR,
                    0,
                    0,
                    "Too many nested anonymous function calls.",
                    "فراخوانی تو در توی توابع بی‌نام بیش از حد مجاز.",
                ));
            }
            Ok(Value::Anon(AnonFn {
                params: params.clone(),
                call,
            }))
        }
        // هندل نام‌دار `@f` — resolve در لحظهٔ ساخت (سمانتیک متلب §9):
        // تابع فایلی ثبت‌شده یا builtin؛ اسکریپت قابل هندل‌شدن نیست.
        Expr::FnHandle { name, line, col } => {
            let funcs_c = env.funcs_clone();
            if let Some(StoredItem::Fn(def)) = env.lookup_item(name) {
                let def_c = Arc::clone(&def);
                let call: mc_core::AnonCall = Arc::new(move |args: &[Value]| {
                    // هر فراخوانی با ورک‌اسپیس تازه؛ بازگشتی درون بدنه از طریق
                    // مخزن مشترک کار می‌کند
                    let outs = call_user_fn(&def_c, args, 1, funcs_c.clone(), 0, 0, 0)?;
                    if let Some(v) = outs.into_iter().next() {
                        return Ok(v);
                    }
                    Err(McError::new(
                        ids::TYPE_ERROR,
                        0,
                        0,
                        "Function handle returned no value.",
                        "function_handle مقداری برنگرداند.",
                    ))
                });
                let params = def.params.clone();
                return Ok(Value::Anon(AnonFn { params, call }));
            }
            if let Some(f) = registry::lookup(name) {
                let call: mc_core::AnonCall = Arc::new(move |args: &[Value]| f(args, 1));
                return Ok(Value::Anon(AnonFn {
                    params: Vec::new(),
                    call,
                }));
            }
            if let Some(mf) = crate::registry_p2::lookup_multi(name) {
                let call: mc_core::AnonCall = Arc::new(move |args: &[Value]| {
                    let mut outs = mf(args, 1)?;
                    match outs.len() {
                        1 => Ok(outs.remove(0)),
                        _ => Ok(Value::Double(Matrix::full(0, 0, 0.0))),
                    }
                });
                return Ok(Value::Anon(AnonFn {
                    params: Vec::new(),
                    call,
                }));
            }
            Err(undefined_var_in(name, *line, *col, env))
        }
        // دسترسی به فیلد ساختار `s.f` / `s.(expr)` (ROADMAP 2.9 — تسک ۱)
        Expr::Field {
            base,
            field,
            line,
            col,
        } => {
            let fname = resolve_field_name(field, env, depth, *line, *col)?;
            let base_val = eval_expr_end(base, env, depth, target, sub_i)?;
            match base_val {
                Value::Struct(s) => match s.get(&fname) {
                    Some(v) => Ok(v.clone()),
                    None => Err(McError::new(
                        ids::TYPE_ERROR,
                        *line,
                        *col,
                        format!("Struct has no field '{fname}'."),
                        format!("ساختار فیلدی به نام «{fname}» ندارد."),
                    )),
                },
                other => Err(McError::new(
                    ids::TYPE_ERROR,
                    *line,
                    *col,
                    format!(
                        "Dot access ('.') needs a struct on its left; got {}.",
                        other.class_name()
                    ),
                    format!(
                        "دسترسی فیلد («.») در سمت چپ به ساختار نیاز دارد؛ {} دریافت شد.",
                        other.class_name()
                    ),
                )),
            }
        }
        // اندیس‌گذاری پسوندی روی نتیجهٔ عبارت (ROADMAP 2.10 — بند ۱۳ ممیزی)
        Expr::Index {
            base,
            subs,
            line,
            col,
        } => {
            // اصطلاح سلول x(1)(2): پایهٔ فراخوانیِ نام‌دار که «متغیرِ غیرهندل»
            // است = اندیسِ متغیر → زنجیرهٔ بعدی = خواندن محتوای سلول = E4
            // (تسک ۲ گام 2.10؛ متلب هم ()() را نمی‌پذیرد — اکتاو زنجیرهٔ آزاد
            // می‌دهد = دلتای اوراکل مستند). هندلِ تابع فراخوانی است نه اندیس:
            // f(x)(1) با f=@(...) باید کار کند.
            if let Expr::Call { name, .. } = &**base {
                if let Some(v) = env.get(name) {
                    if !matches!(v, Value::Anon(_)) {
                        return Err(McError::new(
                            ids::UNSUPPORTED,
                            *line,
                            *col,
                            "Chained indexing like x(1)(2) reads cell content; cell arrays are expansion E4 (assign to an intermediate variable first).",
                            "زنجیرهٔ اندیس مثل x(1)(2) محتوای سلول می‌خواند؛ آرایهٔ سلولی انبساط E4 است (اول به متغیر میانی منتسب کنید).",
                        ));
                    }
                }
            }
            let base_val = eval_expr_end(base, env, depth, target, sub_i)?;
            index_value(&base_val, subs, env, depth, *line, *col)
        }
    }
}

fn eval_expr_depth(expr: &Expr, env: &mut Environment, depth: usize) -> Result<Value, McError> {
    eval_expr_end(expr, env, depth, &Matrix::full(0, 0, 0.0), 0)
}

/// فراخوانی نام‌دار — ترتیب resolve متلب (سند ۰۳ §9): متغیر (شامل هندل) >
/// تابع فایلی/اسکریپت ثبت‌شده (سایه بر builtin) > builtin > خطا.
fn call_named(
    name: &str,
    args: &[Expr],
    env: &mut Environment,
    depth: usize,
    line: u32,
    col: u32,
    target: &Matrix,
) -> Result<Value, McError> {
    // ۱) متغیر؟ — اگر هندل تابع باشد فراخوانی می‌شود؛ اگر ماتریس باشد اندیس است
    match env.get(name).cloned() {
        Some(Value::Anon(f)) => {
            let vals: Vec<Value> = args
                .iter()
                .map(|a| eval_expr_end(a, env, depth, target, 0))
                .collect::<Result<_, _>>()?;
            return (f.call)(&vals);
        }
        Some(Value::Double(_)) if !args.is_empty() => {
            return index_read(name, args, env, depth, line, col);
        }
        // اندیس‌گذاری خواندنی روی char (ROADMAP 2.8): s(2) → 'e'
        Some(Value::Char(_)) if !args.is_empty() => {
            return index_read_char(name, args, env, depth, line, col);
        }
        Some(_) if !args.is_empty() => {
            return Err(McError::new(
                ids::TYPE_ERROR,
                line,
                col,
                format!("'{name}' is not indexable."),
                format!("«{name}» اندیس‌پذیر نیست."),
            ));
        }
        _ => {}
    }
    // ۲) تابع فایلی ثبت‌شده — تقدم بر builtin (سمانتیک متلب §9:
    // فایل کاربر می‌تواند builtin هم‌نام را سایه کند)
    if let Some(StoredItem::Fn(def)) = env.lookup_item(name) {
        let vals: Vec<Value> = args
            .iter()
            .map(|a| eval_expr_end(a, env, depth, target, 0))
            .collect::<Result<_, _>>()?;
        let outs = call_user_fn(&def, &vals, 1, env.funcs_clone(), depth, line, col)?;
        if let Some(v) = outs.into_iter().next() {
            return Ok(v);
        }
        return Err(McError::new(
            ids::TYPE_ERROR,
            line,
            col,
            format!("Function '{name}' did not return a value."),
            format!("تابع «{name}» مقداری برنگرداند."),
        ));
    }
    if let Some(StoredItem::Script(_)) = env.lookup_item(name) {
        // اسکریپت در عبارت/با آرگومان معنا ندارد (سمانتیک متلب)
        return Err(McError::new(
            ids::TYPE_ERROR,
            line,
            col,
            format!(
                "Script '{name}' cannot be used inside an expression or take arguments. Call it as a statement: {name}",
            ),
            format!(
                "اسکریپت «{name}» داخل عبارت یا با آرگومان قابل استفاده نیست؛ به‌صورت گزاره فراخوانی کنید: {name}"
            ),
        ));
    }
    // ۳) builtin؟ — تک‌خروجی REGISTRY، سپس چندخروجی P2 با nargout=1
    if let Some(f) = registry::lookup(name) {
        let vals: Vec<Value> = args
            .iter()
            .map(|a| eval_expr_end(a, env, depth, target, 0))
            .collect::<Result<_, _>>()?;
        // پشتیبان موقعیت (ROADMAP 2.11): خطاهای builtin بدون موقعیت (0,0)
        // موقعیت فراخوانی را می‌گیرند
        return f(&vals, 1).map_err(|e| with_pos(e, line, col));
    }
    if let Some(mf) = crate::registry_p2::lookup_multi(name) {
        let vals: Vec<Value> = args
            .iter()
            .map(|a| eval_expr_end(a, env, depth, target, 0))
            .collect::<Result<_, _>>()?;
        let outs = mf(&vals, 1).map_err(|mut e| {
            // پشتیبان موقعیت — مشروط (ROADMAP 2.11): فقط خانه‌های 0,0 پر می‌شوند
            if e.line == 0 && e.col == 0 {
                e.line = line;
                e.col = col;
            }
            e
        })?;
        if let Some(v) = outs.into_iter().next() {
            return Ok(v);
        }
        return Err(McError::new(
            ids::TYPE_ERROR,
            line,
            col,
            format!("'{name}' returned no value."),
            format!("«{name}» مقداری برنگرداند."),
        ));
    }
    // ۴) هیچ‌کدام — خطا با did-you-mean (builtin، متغیر، تعریف ثبت‌شده).
    // نامِ فراخوانی‌شده = خطای «تابع» (E1002 — ROADMAP 2.11؛ نام برهنه E1001
    // می‌ماند: همان تمایز کاتالوگ سند ۰۳ §۱۴)
    Err(undefined_fn_in(name, line, col, env))
}

/// فراخوانی تابع فایلی ثبت‌شده — سمانتیک متلب §9:
/// - ورک‌اسپیس تازه (متغیرهای فراخوان دیده نمی‌شوند؛ متغیرهای تابع نشت نمی‌کنند)
/// - پارامترها با مقدار بسته می‌شوند؛ `nargin`/`nargout` داخل بدنه تعریف می‌شوند
/// - پارامتر فراهم‌نشده مجاز است (الگوی `if nargin < 2` متلب) — خواندنش خطای undefined می‌دهد
/// - خروجی‌ها به ترتیب تعریف از ورک‌اسپیس تابع برداشته می‌شوند؛ نخوانده = خطای آموزشی
/// - بازگشت: `return` یا پایان بدنه؛ break/continue خارج حلقه = خطا
fn call_user_fn(
    def: &Arc<UserFnDef>,
    args: &[Value],
    nargout: usize,
    funcs: Arc<crate::env::FuncStore>,
    depth: usize,
    line: u32,
    col: u32,
) -> Result<Vec<Value>, McError> {
    if depth >= MAX_CALL_DEPTH {
        return Err(McError::new(
            ids::TYPE_ERROR,
            line,
            col,
            format!(
                "Maximum recursion depth ({MAX_CALL_DEPTH}) reached while calling '{}'.",
                def.name
            ),
            format!(
                "عمق بازگشتی حداکثر ({MAX_CALL_DEPTH}) هنگام فراخوانی «{}» پُر شد.",
                def.name
            ),
        ));
    }
    if args.len() > def.params.len() {
        return Err(McError::new(
            ids::TYPE_ERROR,
            line,
            col,
            format!(
                "Too many input arguments: '{}' expects at most {} argument(s), got {}.",
                def.name,
                def.params.len(),
                args.len()
            ),
            format!(
                "آرگومان ورودی بیش از حد: «{}» حداکثر {} آرگومان می‌خواهد، {} دریافت شد.",
                def.name,
                def.params.len(),
                args.len()
            ),
        ));
    }
    let mut child = Environment::new_with_funcs(funcs);
    for (p, a) in def.params.iter().zip(args.iter()) {
        child.set(p.clone(), a.clone());
    }
    // nargin/nargout — سمانتیک متلب (سایه‌پذیر مثل هر متغیر دیگر)
    child.set("nargin", Value::Double(Matrix::scalar(args.len() as f64)));
    child.set("nargout", Value::Double(Matrix::scalar(nargout as f64)));
    // متن‌های نمایش بدنهٔ تابع عمداً دور ریخته می‌شوند — متلب در فراخوانی تابع
    // اکو نمی‌کند (فقط disp/fprintf چاپ می‌کنند).
    let (flow, _body_texts) = exec_block(&def.body.stmts, &mut child, depth + 1)?;
    match flow {
        Flow::Break | Flow::Continue => {
            return Err(McError::new(
                ids::TYPE_ERROR,
                line,
                col,
                format!("break/continue outside a loop inside '{}'.", def.name),
                format!("break/continue خارج از حلقه، داخل «{}».", def.name),
            ))
        }
        Flow::Normal | Flow::Return => {}
    }
    if nargout == 0 {
        return Ok(Vec::new());
    }
    if nargout > def.outputs.len() {
        return Err(McError::new(
            ids::TYPE_ERROR,
            line,
            col,
            format!(
                "Too many output arguments: '{}' has {} output(s), {} requested.",
                def.name,
                def.outputs.len(),
                nargout
            ),
            format!(
                "خروجی بیش از حد: «{}» {} خروجی دارد، {} خواسته شد.",
                def.name,
                def.outputs.len(),
                nargout
            ),
        ));
    }
    let mut outs = Vec::with_capacity(nargout);
    for out in &def.outputs[..nargout] {
        match child.get(out) {
            Some(v) => outs.push(v.clone()),
            None => {
                return Err(McError::new(
                    ids::TYPE_ERROR,
                    line,
                    col,
                    format!(
                    "Output argument '{}' (and maybe others) was not assigned during call to '{}'.",
                    out, def.name
                ),
                    format!(
                        "آرگومان خروجی «{}» (و شاید دیگران) هنگام فراخوانی «{}» مقدار نگرفت.",
                        out, def.name
                    ),
                ))
            }
        }
    }
    Ok(outs)
}

/// انتساب چندخروجی `[a,b] = f(...)` — سمانتیک متلب §9:
/// فقط فراخوانی تابع می‌تواند چند خروجی بدهد؛ هر هدف یکی از خروجی‌ها به ترتیب
/// تعریف می‌گیرد؛ builtin هم فقط با یک هدف مجاز است (چندخروجی builtin در P2).
fn eval_multi_assign(
    targets: &[OutTarget],
    expr: &Expr,
    env: &mut Environment,
    depth: usize,
    suppress: bool,
) -> Result<Vec<String>, McError> {
    let spec = env.display;
    let (name, args, line, col) = match expr {
        Expr::Call {
            name,
            args,
            line,
            col,
            ..
        } => (name, args, *line, *col),
        _ => {
            return Err(McError::new(
                ids::TYPE_ERROR,
                0,
                0,
                "Only function calls can provide multiple outputs: [a,b] = f(...).",
                "فقط فراخوانی تابع می‌تواند چند خروجی بدهد: [a,b] = f(...).",
            ))
        }
    };
    match env.lookup_item(name) {
        Some(StoredItem::Fn(def)) => {
            let vals: Vec<Value> = args
                .iter()
                .map(|a| eval_expr(a, env, depth))
                .collect::<Result<_, _>>()?;
            let outs = call_user_fn(
                &def,
                &vals,
                targets.len(),
                env.funcs_clone(),
                depth,
                line,
                col,
            )?;
            let mut texts = Vec::new();
            for (t, v) in targets.iter().zip(outs) {
                env.set(t.name.clone(), v.clone());
                if !suppress {
                    texts.push(crate::format::format_assignment_spec(&t.name, &v, &spec));
                }
            }
            Ok(texts)
        }
        Some(StoredItem::Script(_)) => Err(McError::new(
            ids::TYPE_ERROR,
            line,
            col,
            format!("Script '{name}' cannot be used inside an expression or take arguments."),
            format!("اسکریپت «{name}» داخل عبارت یا با آرگومان قابل استفاده نیست."),
        )),
        None => {
            // چندخروجی builtin P2: [T,Y] = ode45، [q,r] = deconv، [V,D] = eig (سند ۰۳ §9)
            if targets.len() > 1 {
                if let Some(mf) = crate::registry_p2::lookup_multi(name) {
                    let vals: Vec<Value> = args
                        .iter()
                        .map(|a| eval_expr(a, env, depth))
                        .collect::<Result<_, _>>()?;
                    let outs = mf(&vals, targets.len()).map_err(|mut e| {
                        e.line = line;
                        e.col = col;
                        e
                    })?;
                    if outs.len() != targets.len() {
                        return Err(McError::new(
                            ids::TYPE_ERROR,
                            line,
                            col,
                            format!(
                                "'{name}' produced {} outputs, {} requested.",
                                outs.len(),
                                targets.len()
                            ),
                            format!(
                                "«{name}» {} خروجی داد، {} خواسته شده بود.",
                                outs.len(),
                                targets.len()
                            ),
                        ));
                    }
                    let mut texts = Vec::new();
                    for (t, v) in targets.iter().zip(outs) {
                        env.set(t.name.clone(), v.clone());
                        if !suppress {
                            texts.push(crate::format::format_assignment(&t.name, &v));
                        }
                    }
                    return Ok(texts);
                }
            }
            if let Some(f) = registry::lookup(name) {
                if targets.len() > 1 {
                    return Err(McError::new(
                        ids::UNSUPPORTED,
                        line,
                        col,
                        format!(
                            "Multi-output builtins like [V,D] = eig are not available yet; '{name}' gets one output here."
                        ),
                        format!(
                            "چندخروجی‌های builtin مثل [V,D] = eig هنوز در راه‌اندازه؛ «{name}» اینجا یک خروجی دارد."
                        ),
                    ));
                }
                let vals: Vec<Value> = args
                    .iter()
                    .map(|a| eval_expr(a, env, depth))
                    .collect::<Result<_, _>>()?;
                let v = f(&vals, targets.len())?;
                let t = &targets[0];
                env.set(t.name.clone(), v.clone());
                if !suppress {
                    return Ok(vec![crate::format::format_assignment(&t.name, &v)]);
                }
                return Ok(Vec::new());
            }
            Err(undefined_var_in(name, line, col, env))
        }
    }
}

/// ساخت ماتریس از ردیف‌های AST (سند ۰۳ §4) — سمانتیک کامل تلفیق متلب:
/// سلول‌ها می‌توانند خودشان ماتریس باشند؛ هم‌ردیف‌ها افقی می‌چسبند (تعداد ردیف برابر)
/// و ردیف‌ها عمودی (تعداد ستون برابر). رشته = سطر کدهای کاراکتر (سند ۰۳ §10).
fn build_matrix(
    rows: &[Vec<Expr>],
    env: &mut Environment,
    depth: usize,
    line: u32,
    col: u32,
) -> Result<Value, McError> {
    // سلول ماتریس لفظی — حقیقی یا مختلط (سند ۰۳ بخش ۳ — P2)
    enum Cell {
        Real(Matrix),
        Complex(mc_core::ComplexMatrix),
        Char(Vec<char>),
    }
    fn as_cell(v: Value, line: u32, col: u32) -> Result<Cell, McError> {
        match v {
            Value::Double(m) => Ok(Cell::Real(m)),
            Value::Complex(z) => Ok(Cell::Complex(z)),
            // رشته = سلول char؛ تلفیق مخلوط با عددی → char مثل متلب
            // ([65 'a'] → 'Aa' — ROADMAP 2.8 بند ۶/۱۲ ممیزی)
            Value::Char(s) => Ok(Cell::Char(s.chars().collect())),
            Value::Anon(_) => Err(McError::new(
                ids::TYPE_ERROR,
                line,
                col,
                "Cannot put a function handle inside a matrix.",
                "function_handle داخل ماتریس قرار نمی‌گیرد.",
            )),
            // ساختار در ماتریس لفظی = آرایهٔ ساختاری → انبساط E4 (ROADMAP 2.9)
            Value::Struct(_) => Err(McError::new(
                ids::UNSUPPORTED,
                line,
                col,
                "Cannot put a struct inside a matrix; struct arrays are expansion E4.",
                "ساختار داخل ماتریس قرار نمی‌گیرد؛ آرایهٔ ساختاری انبساط E4 است.",
            )),
        }
    }
    // تلفیق افقی/عمودی روی (re، im) — برای سلول حقیقی im صفر است
    fn hcat(parts: &[Cell], line: u32, col: u32) -> Result<Cell, McError> {
        let dims = |c: &Cell| match c {
            Cell::Real(m) => (m.rows(), m.cols()),
            Cell::Complex(z) => (z.rows(), z.cols()),
            Cell::Char(cs) => (1, cs.len()),
        };
        let r = parts.first().map(dims).unwrap_or((0, 0)).0;
        if parts.iter().map(dims).any(|(rr, _)| rr != r) {
            return Err(McError::new(
                ids::SIZE_MISMATCH,
                line,
                col,
                "Horizontal concatenation needs equal row counts.",
                "تلفیق افقی نیاز به تعداد ردیف برابر دارد.",
            ));
        }
        let any_complex = parts
            .iter()
            .any(|c| matches!(c, Cell::Complex(z) if !z.is_all_real()));
        // بودجهٔ حافظه (ROADMAP 2.5): تلفیق می‌تواند از مجموع ورودی‌ها بزرگ‌تر شود؛
        // جمع امن + چک قبل از دو تخصیص re/im.
        let c: usize = parts
            .iter()
            .map(|p| dims(p).1)
            .try_fold(0usize, |acc, w| acc.checked_add(w))
            .ok_or_else(|| budget::out_of_memory_error(r, usize::MAX))?;
        budget::check_elements(r, c)?;
        // تلفیق مخلوط char/عددی → char (ROADMAP 2.8 — بند ۶/۱۲): اعداد با
        // قرارداد char() متلب به کد تبدیل می‌شوند؛ مختلطِ غیرواقعی خطای صادقانه
        // (اوراکل Octave 10.3: wrong type argument 'complex scalar').
        if parts.iter().any(|p| matches!(p, Cell::Char(_))) {
            if any_complex {
                return Err(McError::new(
                    ids::TYPE_ERROR,
                    line,
                    col,
                    "Complex values cannot be converted to character codes.",
                    "مقادیر مختلط به کد کاراکتری تبدیل نمی‌شوند.",
                ));
            }
            let mut chars: Vec<char> = Vec::with_capacity(c);
            for p in parts {
                match p {
                    Cell::Char(cs) => chars.extend_from_slice(cs),
                    Cell::Real(m) => {
                        for i in 0..m.numel() {
                            chars.push(Value::char_from_code(m.get_linear(i), line, col)?);
                        }
                    }
                    Cell::Complex(z) => {
                        // همه‌واقعی — مانند double به کد تبدیل می‌شود
                        let re = z.real_part();
                        for i in 0..re.numel() {
                            chars.push(Value::char_from_code(re.get_linear(i), line, col)?);
                        }
                    }
                }
            }
            return Ok(Cell::Char(chars));
        }
        let mut re = Matrix::full(r, c, 0.0);
        let mut im = Matrix::full(r, c, 0.0);
        let mut off = 0usize;
        for p in parts {
            let w = dims(p).1;
            for cc in 0..w {
                for rr in 0..r {
                    match p {
                        Cell::Real(m) => {
                            re.set(rr, off + cc, m.get(rr, cc));
                        }
                        Cell::Complex(z) => {
                            let (vr, vi) = z.get(rr, cc);
                            re.set(rr, off + cc, vr);
                            im.set(rr, off + cc, vi);
                        }
                        // غیرممکن — مسیر char بالاتر return می‌کند
                        Cell::Char(_) => {}
                    }
                }
            }
            off += w;
        }
        if any_complex {
            Ok(Cell::Complex(mc_core::ComplexMatrix::from_columns(
                r,
                c,
                re.as_column_slice().to_vec(),
                im.as_column_slice().to_vec(),
            )))
        } else {
            Ok(Cell::Real(re))
        }
    }

    let mut row_cells: Vec<Cell> = Vec::with_capacity(rows.len());
    for row in rows {
        let mut parts: Vec<Cell> = Vec::with_capacity(row.len());
        for e in row {
            let v = eval_expr_end(e, env, depth, &Matrix::full(0, 0, 0.0), 0)?;
            parts.push(as_cell(v, line, col)?);
        }
        if parts.is_empty() {
            continue;
        }
        row_cells.push(hcat(&parts, line, col)?);
    }
    if row_cells.is_empty() {
        return Ok(Value::Double(Matrix::full(0, 0, 0.0)));
    }
    // char فقط در قالب تک‌ردیفهٔ 1×N (v1 — ROADMAP 2.8 تسک ۱)؛ چندردیفی =
    // آرایهٔ char دوبعدی → انبساط E4 با خطای صادقانه.
    if row_cells.iter().any(|c| matches!(c, Cell::Char(_))) {
        if row_cells.len() != 1 {
            return Err(McError::new(
                ids::UNSUPPORTED,
                line,
                col,
                "2-D char arrays (multi-row) are expansion E4; only 1×N char rows are supported.",
                "آرایهٔ char دوبعدی (چندردیفی) انبساط E4 است؛ فقط ردیف char با ابعاد 1×N پشتیبانی می‌شود.",
            ));
        }
        if let Cell::Char(cs) = &row_cells[0] {
            return Ok(Value::Char(cs.iter().collect()));
        }
    }
    // تلفیق عمودی ردیف‌ها
    let dims = |c: &Cell| match c {
        Cell::Real(m) => (m.rows(), m.cols()),
        Cell::Complex(z) => (z.rows(), z.cols()),
        Cell::Char(cs) => (1, cs.len()),
    };
    let c = dims(&row_cells[0]).1;
    if row_cells.iter().map(dims).any(|(_, cc)| cc != c) {
        return Err(McError::new(
            ids::SIZE_MISMATCH,
            line,
            col,
            "Vertical concatenation needs equal column counts.",
            "تلفیق عمودی نیاز به تعداد ستون برابر دارد.",
        ));
    }
    // بودجهٔ حافظه (ROADMAP 2.5): جمع سطرها امن + چک قبل از دو تخصیص re/im.
    let r: usize = row_cells
        .iter()
        .map(|m| dims(m).0)
        .try_fold(0usize, |acc, h| acc.checked_add(h))
        .ok_or_else(|| budget::out_of_memory_error(usize::MAX, c))?;
    budget::check_elements(r, c)?;
    let any_complex = row_cells
        .iter()
        .any(|c| matches!(c, Cell::Complex(z) if !z.is_all_real()));
    let mut re = Matrix::full(r, c, 0.0);
    let mut im = Matrix::full(r, c, 0.0);
    let mut off = 0usize;
    for m in &row_cells {
        let h = dims(m).0;
        for cc in 0..c {
            for rr in 0..h {
                match m {
                    Cell::Real(mm) => re.set(off + rr, cc, mm.get(rr, cc)),
                    Cell::Complex(z) => {
                        let (vr, vi) = z.get(rr, cc);
                        re.set(off + rr, cc, vr);
                        im.set(off + rr, cc, vi);
                    }
                    // غیرممکن — گارد char بالاتر return/error می‌کند
                    Cell::Char(_) => {}
                }
            }
        }
        off += h;
    }
    if any_complex {
        Ok(Value::Complex(mc_core::ComplexMatrix::from_columns(
            r,
            c,
            re.as_column_slice().to_vec(),
            im.as_column_slice().to_vec(),
        )))
    } else {
        Ok(Value::Double(re))
    }
}

/// سمانتیک دقیق کولن (سند ۰۳ §7): عناصر a, a+s, ... و تعداد
/// n = floor((b-a)/s + eps_correction) + 1 — هرگز جمع تکراری.
fn range_values(a: &Value, s: &Value, b: &Value) -> Result<Value, McError> {
    let (av, sv, bv) = match (a.as_scalar(), s.as_scalar(), b.as_scalar()) {
        (Some(a), Some(s), Some(b)) => (a, s, b),
        _ => {
            return Err(McError::new(
                ids::TYPE_ERROR,
                0,
                0,
                "Colon operands must be scalars.",
                "عملوندهای کولن باید اسکالر باشند.",
            ))
        }
    };
    if sv == 0.0 {
        return Err(McError::new(
            ids::TYPE_ERROR,
            0,
            0,
            "Colon step cannot be zero.",
            "گام کولن نمی‌تواند صفر باشد.",
        ));
    }
    // الگوریتم مرجع متلب: محاسبه n، سپس a + (0:n-1)*s با اصلاح پایانی
    let n_float = (bv - av) / sv;
    let tol = f64::EPSILON * (av.abs().max(bv.abs()).max(1.0));
    let n = if sv > 0.0 {
        if bv < av - tol {
            0
        } else {
            (n_float + tol).floor() as usize + 1
        }
    } else if bv > av + tol {
        0
    } else {
        ((av - bv) / -sv + tol).floor() as usize + 1
    };
    if n == 0 {
        return Ok(Value::Double(Matrix::full(1, 0, 0.0))); // ماتریس تهی 1×0 (سند ۰۳ §4)
    }
    // بودجهٔ حافظه (ROADMAP 2.5، بند ۲ دفتر): materialize شدن بازه از ورودی کاربر است —
    // «x=1:1e9» باید خطای MATLAB-گونه بدهد، نه abort پروسه.
    let mut out = Matrix::try_full(1, n, 0.0)?;
    for i in 0..n {
        out.set(0, i, av + (i as f64) * sv);
    }
    // اصلاح عنصر آخر برای انطباق دقیق با b (رفتار مرجع — سند ۰۳ §7)
    out.set(0, n - 1, bv);
    Ok(Value::Double(out))
}

// ---------- عملگرها ----------

fn negate(v: &Value) -> Result<Value, McError> {
    match v {
        Value::Double(m) => {
            let mut out = m.clone();
            for idx in 0..out.numel() {
                out.set_linear(idx, -out.get_linear(idx));
            }
            Ok(Value::Double(out))
        }
        Value::Complex(z) => {
            let mut out = z.clone();
            for idx in 0..out.numel() {
                let (re, im) = out.get_linear(idx);
                out.set_linear(idx, -re, -im);
            }
            Ok(Value::Complex(out))
        }
        _ => Err(McError::new(
            ids::TYPE_ERROR,
            0,
            0,
            "Unary minus needs a numeric operand.",
            "منفی یگانی به عملوند عددی نیاز دارد.",
        )),
    }
}

fn apply_compare(op: CmpOp, l: &Value, r: &Value, line: u32, col: u32) -> Result<Value, McError> {
    // مختلط: == و ~= عنصری؛ ترتیبی (< > <= >=) خطای صادقانه متلب
    let complex_involved = matches!(l, Value::Complex(_)) || matches!(r, Value::Complex(_));
    if complex_involved {
        if !matches!(op, CmpOp::Eq | CmpOp::Ne) {
            return Err(McError::new(
                ids::TYPE_ERROR,
                line,
                col,
                "Ordered comparison (<, <=, >, >=) is not defined for complex values.",
                "مقایسهٔ ترتیبی (<، <=، >، >=) برای مقدارهای مختلط تعریف ندارد.",
            ));
        }
        let a = cx::to_complex(l, line, col)?;
        let b = cx::to_complex(r, line, col)?;
        let m = cx::c_eq(&a, &b)?;
        return Ok(Value::Double(if matches!(op, CmpOp::Eq) {
            m
        } else {
            let mut out = m.clone();
            for idx in 0..out.numel() {
                let v = out.get_linear(idx);
                out.set_linear(idx, if v == 0.0 { 1.0 } else { 0.0 });
            }
            out
        }));
    }
    let (a, b) = (as_num(l, line, col)?, as_num(r, line, col)?);
    let m = broadcast2(&a, &b, "comparison", |x, y| match op {
        CmpOp::Eq => (x == y) as i32 as f64,
        CmpOp::Ne => (x != y) as i32 as f64,
        CmpOp::Lt => (x < y) as i32 as f64,
        CmpOp::Le => (x <= y) as i32 as f64,
        CmpOp::Gt => (x > y) as i32 as f64,
        CmpOp::Ge => (x >= y) as i32 as f64,
    })?;
    Ok(Value::Double(m))
}

fn as_num(v: &Value, line: u32, col: u32) -> Result<Matrix, McError> {
    match v {
        Value::Double(m) => Ok(m.clone()),
        Value::Complex(z) if z.is_all_real() => Ok(z.real_part()),
        Value::Complex(_) => Err(McError::new(
            ids::TYPE_ERROR,
            line,
            col,
            "Complex value in a real-only context (e.g. indexing or loop bounds).",
            "مقدار مختلط در زمینهٔ فقط-حقیقی (مثل اندیس‌گذاری یا مرز حلقه).",
        )),
        Value::Char(s) => {
            // متلب: char در زمینهٔ عددی = ردیف کدها ('abc'+1 → [98 99 100]؛
            // 'abc'=='abd' → [1 1 0]) — ROADMAP 2.8 بند ۶/۱۲
            Ok(Value::char_codes_row(s))
        }
        Value::Anon(_) => Err(McError::new(
            ids::TYPE_ERROR,
            line,
            col,
            "Function handle in numeric comparison.",
            "function_handle در مقایسه عددی.",
        )),
        Value::Struct(_) => Err(McError::new(
            ids::TYPE_ERROR,
            line,
            col,
            "Struct in a numeric context; extract fields with s.f first.",
            "ساختار در زمینهٔ عددی؛ ابتدا فیلد را با s.f بیرون بکشید.",
        )),
    }
}

fn apply_binary(op: BinOp, l: &Value, r: &Value, line: u32, col: u32) -> Result<Value, McError> {
    // مختلط: هر عملوندی که مختلط باشد کل مسیر مختلط می‌رود (رفتار متلب)
    let complex_involved = matches!(l, Value::Complex(_)) || matches!(r, Value::Complex(_));
    if complex_involved {
        return apply_binary_complex(op, l, r, line, col);
    }
    let (a, b) = (as_num(l, line, col)?, as_num(r, line, col)?);
    let m = match op {
        BinOp::Add | BinOp::Sub => {
            let (ar, ac, br, bc) = (a.rows(), a.cols(), b.rows(), b.cols());
            let a_scalar = a.is_scalar();
            let b_scalar = b.is_scalar();
            // + و − متلب: ابعاد برابر، یا یکی اسکالر (بروزکست)
            if !a_scalar && !b_scalar && (ar, ac) != (br, bc) {
                return Err(McError::new(
                    ids::SIZE_MISMATCH, line, col,
                    format!("Plus/minus: sizes must agree or one scalar, got {ar}x{ac} and {br}x{bc}."),
                    format!("جمع/تفریق: ابعاد باید هم‌خوان یا یکی اسکالر باشد؛ دریافت شد {ar}×{ac} و {br}×{bc}."),
                ));
            }
            broadcast2(&a, &b, "plus", |x, y| {
                if matches!(op, BinOp::Add) {
                    x + y
                } else {
                    x - y
                }
            })?
        }
        BinOp::Mul => {
            // ضرب ماتریسی؛ اسکالرها بروزکست (سند ۰۲ §5)
            if a.is_scalar() || b.is_scalar() {
                broadcast2(&a, &b, "mtimes", |x, y| x * y)?
            } else {
                if a.cols() != b.rows() {
                    return Err(McError::new(
                        ids::SIZE_MISMATCH, line, col,
                        format!("mtimes: inner dimensions must agree, got {ar}x{ac} and {br}x{bc}.", ar = a.rows(), ac = a.cols(), br = b.rows(), bc = b.cols()),
                        format!("ضرب ماتریسی: ابعاد داخلی باید هم‌خوان باشند؛ دریافت شد {ar}×{ac} و {br}×{bc}.", ar = a.rows(), ac = a.cols(), br = b.rows(), bc = b.cols()),
                    ));
                }
                matmul(&a, &b)
            }
        }
        BinOp::ElemMul | BinOp::ElemDiv | BinOp::ElemPow => {
            let f = match op {
                BinOp::ElemMul => |x: f64, y: f64| x * y,
                BinOp::ElemDiv => |x: f64, y: f64| x / y, // IEEE: 0/0 = NaN (سند ۰۳ §11)
                _ => |x: f64, y: f64| x.powf(y),
            };
            broadcast2(&a, &b, "elementwise", f)?
        }
        BinOp::Div => {
            // سمانتیک mrdivide متلب: a/b = a*inv(b) = (b'\a')' — اسکالر↔اسکالر = تقسیم معمولی
            // (باگ قبلی: عملوندها برعکس بودند — 8/2 می‌داد 0.25)
            if a.is_scalar() && b.is_scalar() {
                Matrix::scalar(a.get(0, 0) / b.get(0, 0))
            } else {
                let at = transpose(&a);
                let bt = transpose(&b);
                let (x, _) = mc_num::mldivide(&bt, &at)?;
                transpose(&x)
            }
        }
        BinOp::Backslash => {
            let (x, _) = mc_num::mldivide(&a, &b)?;
            x
        }
        BinOp::Pow => {
            if a.is_scalar() && b.is_scalar() {
                Matrix::scalar(a.get(0, 0).powf(b.get(0, 0)))
            } else if b.is_scalar() && a.rows() == a.cols() {
                // توان ماتریسی توان صحیح نامنفی: A^0=I، A^n = تکرار ضرب (سمانتیک متلب §5)
                let p = b.get(0, 0);
                if p < 0.0 || p.fract() != 0.0 {
                    return Err(McError::new(
                        ids::UNSUPPORTED,
                        line,
                        col,
                        "Negative/fractional matrix power needs inv/expm — arrives in P2.",
                        "توان منفی/اعشاری ماتریسی به inv/expm نیاز دارد — در P2 می‌آید.",
                    ));
                }
                let n = p as usize;
                let mut acc = Matrix::full(a.rows(), a.cols(), 0.0);
                for i in 0..a.rows() {
                    acc.set(i, i, 1.0);
                }
                for _ in 0..n {
                    acc = matmul(&acc, &a);
                }
                acc
            } else {
                return Err(McError::new(
                    ids::UNSUPPORTED, line, col,
                    "Matrix power needs a square matrix and scalar exponent; use .^ for elementwise.",
                    "توان ماتریسی به ماتریس مربع و توان اسکالر نیاز دارد؛ برای عنصری از .^ استفاده کنید.",
                ));
            }
        }
    };
    Ok(Value::Double(m))
}

/// مسیر مختلط عملگرهای دوتایی (سند ۰۳ بخش ۳ — P2).
fn apply_binary_complex(
    op: BinOp,
    l: &Value,
    r: &Value,
    line: u32,
    col: u32,
) -> Result<Value, McError> {
    let a = cx::to_complex(l, line, col)?;
    let b = cx::to_complex(r, line, col)?;
    let z = match op {
        BinOp::Add => cx::c_add(&a, &b)?,
        BinOp::Sub => cx::c_sub(&a, &b)?,
        BinOp::Mul => {
            if a.is_scalar() || b.is_scalar() {
                cx::c_elem_mul(&a, &b)? // اسکالر ← ضرب عنصری = بروزکست
            } else {
                cx::c_matmul(&a, &b)?
            }
        }
        BinOp::ElemMul => cx::c_elem_mul(&a, &b)?,
        BinOp::ElemDiv => cx::c_elem_div(&a, &b)?,
        BinOp::ElemPow => cx::c_elem_pow(&a, &b)?,
        BinOp::Div => {
            if a.is_scalar() && b.is_scalar() {
                cx::c_elem_div(&a, &b)?
            } else {
                cx::c_mrdivide(&a, &b)?
            }
        }
        BinOp::Backslash => cx::c_backslash(&a, &b)?,
        BinOp::Pow => {
            // A^z فقط اسکالر↔اسکالر (توان ماتریسی مختلط در P2-b)
            if a.is_scalar() && b.is_scalar() {
                cx::c_elem_pow(&a, &b)?
            } else {
                return Err(McError::new(
                    ids::UNSUPPORTED,
                    line,
                    col,
                    "Complex matrix power is not supported yet; use .^ for elementwise.",
                    "توان ماتریسی مختلط هنوز پشتیبانی نمی‌شود؛ برای عنصری از .^ استفاده کنید.",
                ));
            }
        }
    };
    Ok(Value::Complex(z))
}

fn matmul(a: &Matrix, b: &Matrix) -> Matrix {
    let mut out = Matrix::full(a.rows(), b.cols(), 0.0);
    for c in 0..b.cols() {
        for r in 0..a.rows() {
            let mut acc = 0.0;
            for k in 0..a.cols() {
                acc += a.get(r, k) * b.get(k, c);
            }
            out.set(r, c, acc);
        }
    }
    out
}

fn transpose(m: &Matrix) -> Matrix {
    let mut out = Matrix::full(m.cols(), m.rows(), 0.0);
    for c in 0..m.cols() {
        for r in 0..m.rows() {
            out.set(c, r, m.get(r, c));
        }
    }
    out
}
