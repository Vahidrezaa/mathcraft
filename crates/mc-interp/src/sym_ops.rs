//! عملگرهای سمبولیک مفسر (گام 4.1/CAS-0 — سند ۰۵ §۲).
//!
//! مسیرِ «هر عملوند Sym دارد» از `apply_binary`/منفی یکه به این‌جا می‌آید.
//! فقط **اسکالر ↔ اسکالر** پشتیبانی می‌شود (ماتریس سمبولیک = CAS-1)؛
//! هر عملِ پشتیبانی‌نشده خطای دوزبانهٔ روشن می‌دهد — هرگز جواب غلطِ
//! عددی نه (اصل صداقت، سند ۰۰ اصل ۵).

use std::sync::Arc;

use mc_cas::{sym_display, to_sexp, CasError, Sym};
use mc_core::error::ids;
use mc_core::error::McError;
use mc_core::Value;

use mc_parser::BinOp;

/// خطای CAS → پیام دوزبانهٔ مفسر.
pub fn cas_err(e: CasError, line: u32, col: u32) -> McError {
    match e {
        CasError::DivByZero => McError::new(
            ids::DIV_BY_ZERO,
            line,
            col,
            "Division by zero in exact arithmetic.",
            "تقسیم بر صفر در حساب دقیق.",
        ),
        CasError::SymbolLeft(name) => McError::new(
            ids::TYPE_ERROR,
            line,
            col,
            format!(
                "The expression still contains the symbolic variable '{name}'; substitute it (subs) before converting to double."
            ),
            format!(
                "عبارت هنوز متغیر سمبولیک «{name}» را دارد؛ پیش از تبدیل به double جای‌گذاری‌اش کنید (subs)."
            ),
        ),
        CasError::F64Overflow => McError::new(
            ids::DIV_BY_ZERO,
            line,
            col,
            "Exact value does not fit in a double.",
            "مقدار دقیق در double جا نمی‌شود.",
        ),
        CasError::Unsupported(why) => McError::new(
            ids::UNSUPPORTED,
            line,
            col,
            format!("Symbolic operation not available in this CAS phase: {why}."),
            format!("عمل سمبولیک در این فاز CAS در دسترس نیست: {why}."),
        ),
        CasError::Serde(why) => McError::new(
            ids::UNSUPPORTED,
            line,
            col,
            format!("Malformed serialized symbolic data: {why}."),
            format!("دادهٔ سمبولیک سری‌شده خراب است: {why}."),
        ),
    }
}

/// تبدیل عملوند به درخت Sym: خود Sym، یا اسکالر عددی (دقیق اگر دودویی مرتب).
fn operand_sym(v: &Value, line: u32, col: u32) -> Result<Sym, McError> {
    match v {
        Value::Sym(s) => Ok((**s).clone()),
        Value::Double(m) if m.is_scalar() => Ok(Sym::from_f64(m.get(0, 0))),
        Value::Double(m) => Err(McError::new(
            ids::UNSUPPORTED,
            line,
            col,
            format!(
                "Symbolic matrices are not supported yet — a later CAS phase (got {}x{}).",
                m.rows(),
                m.cols()
            ),
            format!(
                "ماتریس سمبولیک هنوز پشتیبانی نمی‌شود — فاز بعدی CAS (دریافت شد {}×{}).",
                m.rows(),
                m.cols()
            ),
        )),
        Value::Complex(z) if z.is_all_real() && z.rows() == 1 && z.cols() == 1 => {
            // مختلطِ تماماً-حقیقی ۱×۱ → همان عدد حقیقی
            let m = z.real_part();
            Ok(Sym::from_f64(m.get(0, 0)))
        }
        Value::Complex(_) => Err(McError::new(
            ids::UNSUPPORTED,
            line,
            col,
            "Complex symbolic values are not supported yet — a later CAS phase.",
            "مقادیر سمبولیک مختلط هنوز پشتیبانی نمی‌شود — فاز بعدی CAS.",
        )),
        Value::Char(_) => Err(McError::new(
            ids::TYPE_ERROR,
            line,
            col,
            "char has no symbolic form; use sym('x') to declare symbols via syms.",
            "char شکل سمبولیک ندارد؛ نمادها را با syms تعریف کنید.",
        )),
        Value::Anon(_) => Err(McError::new(
            ids::TYPE_ERROR,
            line,
            col,
            "Function handle in symbolic expression.",
            "function_handle در عبارت سمبولیک.",
        )),
        Value::Struct(_) => Err(McError::new(
            ids::TYPE_ERROR,
            line,
            col,
            "struct in symbolic expression.",
            "struct در عبارت سمبولیک.",
        )),
        Value::List(_) => Err(McError::new(
            ids::TYPE_ERROR,
            line,
            col,
            "Cell (list) in symbolic expression.",
            "cell (لیست) در عبارت سمبولیک.",
        )),
    }
}

/// عملگرهای دوتایی وقتی هر سمت می‌تواند سمبولیک باشد — از `apply_binary`
/// صدا می‌شود (قبل از مسیر عددی/مختلط).
pub fn apply_binary_sym(
    op: BinOp,
    l: &Value,
    r: &Value,
    line: u32,
    col: u32,
) -> Result<Value, McError> {
    let a = operand_sym(l, line, col)?;
    let b = operand_sym(r, line, col)?;
    let tree = match op {
        BinOp::Add => Sym::plus(a, b),
        BinOp::Sub => Sym::minus(a, b),
        BinOp::Mul | BinOp::ElemMul => Sym::times(a, b), // اسکالر: .* = *
        BinOp::Div | BinOp::ElemDiv => Sym::over(a, b).map_err(|e| cas_err(e, line, col))?,
        BinOp::Pow | BinOp::ElemPow => Sym::pow(a, b).map_err(|e| cas_err(e, line, col))?,
        BinOp::Backslash => {
            return Err(McError::new(
                ids::UNSUPPORTED,
                line,
                col,
                "Backslash solves arrive in CAS-2 (symbolic linear systems).",
                "حل با \\ در CAS-2 می‌آید (دستگاه‌های خطی سمبولیک).",
            ))
        }
    };
    Ok(Value::Sym(Arc::new(tree)))
}

/// منفی یکهٔ سمبولیک.
pub fn negate_sym(v: &Value, line: u32, col: u32) -> Result<Value, McError> {
    let a = operand_sym(v, line, col)?;
    Ok(Value::Sym(Arc::new(Sym::negated(a))))
}

/// نمایش خطی برای echo کنسول/whos (`format_value`).
pub fn sym_value_to_string(v: &Value) -> String {
    match v {
        Value::Sym(s) => sym_display(s),
        _ => String::new(),
    }
}

/// هستهٔ داخلی `subs(expr, old, new)` — برای تست و رجیستری.
pub fn subs_core(
    expr: &Value,
    old: &Value,
    new: &Value,
    line: u32,
    col: u32,
) -> Result<Value, McError> {
    let e = operand_sym(expr, line, col)?;
    let name = match old {
        Value::Sym(s) => match &**s {
            Sym::Symbol(n) => n.to_string(),
            _ => {
                return Err(McError::new(
                    ids::TYPE_ERROR,
                    line,
                    col,
                    "subs: second argument must be a symbolic variable.",
                    "subs: آرگومان دوم باید متغیر سمبولیک باشد.",
                ))
            }
        },
        Value::Char(s) => s.clone(), // subs(y, 'x', 2.5) هم پذیرفتنی است
        _ => {
            return Err(McError::new(
                ids::TYPE_ERROR,
                line,
                col,
                "subs: second argument must be a symbolic variable.",
                "subs: آرگومان دوم باید متغیر سمبولیک باشد.",
            ))
        }
    };
    let with = operand_sym(new, line, col)?;
    Ok(Value::Sym(Arc::new(e.substitute(&name, &with))))
}

/// هستهٔ داخلی `double(x)` روی مقدار سمبولیک — عددی کامل → Double؛
/// نماد مانده → خطای صادقانه (هرگز حدس نمی‌زنیم).
pub fn double_sym(v: &Value, line: u32, col: u32) -> Result<Value, McError> {
    let e = operand_sym(v, line, col)?;
    match e.to_f64() {
        Ok(x) => Ok(Value::Double(mc_core::Matrix::scalar(x))),
        Err(err) => Err(cas_err(err, line, col)),
    }
}

// ---------- گام 4.2/CAS-1: توابع روی آرگومان سمبولیک + diff/simplify/expand/collect/latex ----------

/// تابع عنصری روی آرگومان سمبولیک (sin(x)، exp(x)، …) — فقط وقتی فعال
/// می‌شود که دست‌کم یک عملوند Sym دارد؛ وگرنه مسیر عددی مفسر می‌ماند
/// (پاریت متلب: sin(0.5) عددی، sin(sym) نمادین). None = نام/شکل نامربوط.
pub fn try_symbolic_fn(
    name: &str,
    vals: &[Value],
    line: u32,
    col: u32,
) -> Option<Result<Value, McError>> {
    if !mc_cas::is_symbolic_fn(name) {
        return None;
    }
    if !vals.iter().any(|v| matches!(v, Value::Sym(_))) {
        return None;
    }
    let mut args = Vec::with_capacity(vals.len());
    for v in vals {
        match operand_sym(v, line, col) {
            Ok(s) => args.push(s),
            Err(e) => return Some(Err(e)),
        }
    }
    Some(match Sym::fn_of(name, args) {
        Ok(tree) => Ok(Value::Sym(Arc::new(tree))),
        Err(e) => Err(cas_err(e, line, col)),
    })
}

/// `diff` سمبولیک — پاریت متلب: diff(S)، diff(S,n)، diff(S,x)، diff(S,x,n).
/// دیسپچ از bi_diff با آرگومان اول Sym.
pub fn diff_sym(args: &[Value], line: u32, col: u32) -> Result<Value, McError> {
    if args.is_empty() || args.len() > 3 {
        return Err(McError::new(
            ids::TYPE_ERROR,
            line,
            col,
            "diff: symbolic form takes 1 to 3 arguments: diff(S), diff(S,n), diff(S,x) or diff(S,x,n).",
            "diff: شکل سمبولیک ۱ تا ۳ آرگومان می‌گیرد: diff(S)، diff(S,n)، diff(S,x) یا diff(S,x,n).",
        ));
    }
    let e = operand_sym(&args[0], line, col)?;
    // آرگومان دوم: متغیر (Sym نماد / char) یا مرتبهٔ عددی
    let (var, n) = match args.len() {
        1 => (None, 1),
        2 => match var_or_order(&args[1], line, col)? {
            VarOrOrder::Var(v) => (Some(v), 1),
            VarOrOrder::Order(n) => (None, n),
        },
        _ => match var_or_order(&args[1], line, col)? {
            VarOrOrder::Var(v) => {
                let n = numeric_order(&args[2], line, col)?;
                (Some(v), n)
            }
            VarOrOrder::Order(_) => {
                return Err(McError::new(
                    ids::TYPE_ERROR,
                    line,
                    col,
                    "diff: second argument must be the variable when a third argument (order) is given.",
                    "diff: وقتی آرگومان سوم (مرتبه) داده شده، آرگومان دوم باید متغیر باشد.",
                ))
            }
        },
    };
    let var = match var {
        Some(v) => v,
        None => mc_cas::diff::default_var(&e).map_err(|err| cas_err(err, line, col))?,
    };
    let d = mc_cas::diff::diff_n(&e, &var, n).map_err(|err| cas_err(err, line, col))?;
    Ok(Value::Sym(Arc::new(d)))
}

enum VarOrOrder {
    Var(String),
    Order(i64),
}

fn var_or_order(v: &Value, line: u32, col: u32) -> Result<VarOrOrder, McError> {
    match v {
        Value::Sym(s) => match &**s {
            Sym::Symbol(n) => Ok(VarOrOrder::Var(n.to_string())),
            _ => Err(McError::new(
                ids::TYPE_ERROR,
                line,
                col,
                "diff: the variable argument must be a symbolic variable.",
                "diff: آرگومان متغیر باید متغیر سمبولیک باشد.",
            )),
        },
        Value::Char(s) => Ok(VarOrOrder::Var(s.clone())),
        other => {
            if let Ok(n) = numeric_order(other, line, col) {
                return Ok(VarOrOrder::Order(n));
            }
            Err(McError::new(
                ids::TYPE_ERROR,
                line,
                col,
                "diff: expected a variable (sym/char) or a non-negative integer order.",
                "diff: متغیر (sym/char) یا مرتبهٔ صحیح نامنفی لازم است.",
            ))
        }
    }
}

fn numeric_order(v: &Value, line: u32, col: u32) -> Result<i64, McError> {
    match v {
        Value::Double(m) if m.is_scalar() && m.get(0, 0).fract() == 0.0 && m.get(0, 0) >= 0.0 => {
            Ok(m.get(0, 0) as i64)
        }
        _ => Err(McError::new(
            ids::TYPE_ERROR,
            line,
            col,
            "diff: the derivative order must be a non-negative integer.",
            "diff: مرتبهٔ مشتق باید عدد صحیح نامنفی باشد.",
        )),
    }
}

/// `simplify(S)` — فرم نرمال گویا + قواعد کل-دامنه (سند ۰۵ §۳).
pub fn simplify_core(v: &Value, line: u32, col: u32) -> Result<Value, McError> {
    let e = operand_sym(v, line, col)?;
    match mc_cas::simplify::simplify(&e) {
        Ok(tree) => Ok(Value::Sym(Arc::new(tree))),
        Err(err) => Err(cas_err(err, line, col)),
    }
}

/// `expand(S)` — توزیع کامل (بدون حذف gcd — پاریت متلب).
pub fn expand_core(v: &Value, line: u32, col: u32) -> Result<Value, McError> {
    let e = operand_sym(v, line, col)?;
    match mc_cas::simplify::expand(&e) {
        Ok(tree) => Ok(Value::Sym(Arc::new(tree))),
        Err(err) => Err(cas_err(err, line, col)),
    }
}

/// `collect(S)` / `collect(S, x)` — گروه‌بندی بر حسب توان متغیر.
pub fn collect_core(v: &Value, var: Option<&Value>, line: u32, col: u32) -> Result<Value, McError> {
    let e = operand_sym(v, line, col)?;
    let var_name = match var {
        None => mc_cas::diff::default_var(&e).map_err(|err| cas_err(err, line, col))?,
        Some(w) => match w {
            Value::Sym(s) => match &**s {
                Sym::Symbol(n) => n.to_string(),
                _ => {
                    return Err(McError::new(
                        ids::TYPE_ERROR,
                        line,
                        col,
                        "collect: second argument must be a symbolic variable.",
                        "collect: آرگومان دوم باید متغیر سمبولیک باشد.",
                    ))
                }
            },
            Value::Char(s) => s.clone(),
            _ => {
                return Err(McError::new(
                    ids::TYPE_ERROR,
                    line,
                    col,
                    "collect: second argument must be a variable (sym/char).",
                    "collect: آرگومان دوم باید متغیر باشد (sym/char).",
                ))
            }
        },
    };
    match mc_cas::simplify::collect(&e, &var_name) {
        Ok(tree) => Ok(Value::Sym(Arc::new(tree))),
        Err(err) => Err(cas_err(err, line, col)),
    }
}

/// `latex(S)` — رشتهٔ TeX خام (سند ۰۵ §۱۰) برای گزارش‌های درس و رندر
/// سمت Dart (flutter_math_fork — سند ۰۶).
pub fn latex_core(v: &Value, line: u32, col: u32) -> Result<Value, McError> {
    let e = operand_sym(v, line, col)?;
    Ok(Value::Char(mc_cas::latex::to_latex(&e)))
}

/// سری‌سازی برای KIND_SYM (سند ۰۵ §۱۲) — برای mc-io.
pub fn sym_to_mcws_payload(s: &mc_cas::Sym) -> String {
    to_sexp(s)
}

/// بارگذاری از KIND_SYM — قالب خراب هرگز حدس نمی‌شود.
pub fn sym_from_mcws_payload(payload: &str, line: u32, col: u32) -> Result<Value, McError> {
    match mc_cas::from_sexp(payload) {
        Ok(tree) => Ok(Value::Sym(Arc::new(tree))),
        Err(err) => Err(cas_err(err, line, col)),
    }
}

// ---------- گام 4.3/CAS-2: int / solve (سند ۰۵ §۵ و §۸) ----------

/// آرگومان متغیر (نماد سمبولیک یا char) — برای int/solve. عدد و عبارت
/// دیگر خطای صادقانه است (برخلاف diff که مرتبهٔ عددی هم می‌پذیرد).
fn var_name_arg(v: &Value, fn_name: &str, line: u32, col: u32) -> Result<String, McError> {
    match v {
        Value::Sym(s) => match &**s {
            Sym::Symbol(n) => Ok(n.to_string()),
            _ => Err(McError::new(
                ids::TYPE_ERROR,
                line,
                col,
                format!("{fn_name}: the variable argument must be a symbolic variable."),
                format!("{fn_name}: آرگومان متغیر باید متغیر سمبولیک باشد."),
            )),
        },
        Value::Char(s) => Ok(s.clone()),
        _ => Err(McError::new(
            ids::TYPE_ERROR,
            line,
            col,
            format!("{fn_name}: expected a variable (sym/char)."),
            format!("{fn_name}: متغیر لازم است (sym/char)."),
        )),
    }
}

/// خطای int معین — پیام صادقانهٔ موتور با اشارهٔ مسیر عددی integral()
/// (سند ۰۵ §۵: «معین = دقیق + cross-check عددی؛ شکست = خطا نه حدس»).
fn int_definite_err(e: CasError, line: u32, col: u32) -> McError {
    let why = match e {
        CasError::Unsupported(why) => why.to_string(),
        CasError::DivByZero => "division by zero while simplifying the antiderivative.".to_string(),
        _ => "the antiderivative could not be simplified exactly.".to_string(),
    };
    McError::new(
        ids::UNSUPPORTED,
        line,
        col,
        format!("int: {why}"),
        format!("int: {why} — برای پاس عددی از integral() استفاده کنید."),
    )
}

/// `int` سمبولیک — سند ۰۵ §۵. پاریت متلب: int(S)، int(S, x) و
/// int(S, x, a, b) معین. شکست نامعین → گرهٔ unevaluated `int(f, x)`
/// (echo همان را می‌بیند؛ diff آن با قضیهٔ بنیادی حسابان کار می‌کند —
/// CAS-2 هسته)؛ شکست معین → خطای صادقانه با اشارهٔ integral().
/// هرگز جواب غلطِ مطمئن‌نما نه (اصل ۵ سند ۰۰).
pub fn int_sym(args: &[Value], line: u32, col: u32) -> Result<Value, McError> {
    if !matches!(args.len(), 1 | 2 | 4) {
        return Err(McError::new(
            ids::TYPE_ERROR,
            line,
            col,
            "int: symbolic form takes 1, 2 or 4 arguments: int(S), int(S, x) or int(S, x, a, b).",
            "int: شکل سمبولیک ۱، ۲ یا ۴ آرگومان می‌گیرد: int(S)، int(S, x) یا int(S, x, a, b).",
        ));
    }
    let f = operand_sym(&args[0], line, col)?;
    let x = match args.len() {
        1 => mc_cas::diff::default_var(&f).map_err(|err| cas_err(err, line, col))?,
        _ => var_name_arg(&args[1], "int", line, col)?,
    };
    if args.len() == 4 {
        let a = operand_sym(&args[2], line, col)?;
        let b = operand_sym(&args[3], line, col)?;
        return match mc_cas::int::integrate_definite(&f, &x, &a, &b) {
            Ok(tree) => Ok(Value::Sym(Arc::new(tree))),
            Err(e) => Err(int_definite_err(e, line, col)),
        };
    }
    match mc_cas::int::integrate(&f, &x) {
        Ok(Some(tree)) => Ok(Value::Sym(Arc::new(tree))),
        // شکست صادقانه → گرهٔ unevaluated؛ نمایش int(f, x) و diff آن FTC است
        Ok(None) => Ok(Value::Sym(Arc::new(Sym::Fn(
            Arc::from("int"),
            vec![f, Sym::Symbol(Arc::from(x.as_str()))],
        )))),
        Err(e) => Err(cas_err(e, line, col)),
    }
}

/// `solve` سمبولیک — سند ۰۵ §۸: «solve همیشه list برمی‌گرداند».
/// بارگذاری MATLAB-گونه: هر آرگومان Symbol/char = متغیر، بقیه Sym =
/// معادله در فرم eq = 0 (لیترال `[…]` ماتریس است و عضو Sym نمی‌پذیرد —
/// CAS-0؛ پس دستگاه با آرگومان‌های چندگانه داده می‌شود):
/// solve(S) / solve(S, x) — تک‌معادله → لیست ریشه‌های حقیقی؛
/// solve(S1, S2) / solve(S1, S2, x, y) — دستگاه (خطی یکتا/پارامتریک با
/// _t یا جفت غیرخطی ۲×۲) → لیست تخت به‌ازای هر متغیر در ترتیب vars.
/// تک‌معادله با چند متغیر بدون نام → خطای ابهام صادقانه. فراتر از
/// جدول/مختلط → خطای صادقانه، هرگز ریشهٔ کاذب از ژنراتورهای اُپک نه.
pub fn solve_sym(args: &[Value], line: u32, col: u32) -> Result<Value, McError> {
    if args.is_empty() {
        return Err(McError::new(
            ids::TYPE_ERROR,
            line,
            col,
            "solve: needs at least one equation: solve(S), solve(S, x) or solve(S1, S2, x, y).",
            "solve: دست‌کم یک معادله لازم است: solve(S)، solve(S, x) یا solve(S1, S2, x, y).",
        ));
    }
    // جداسازی آرگومان‌ها: نماد/char = متغیر؛ بقیه = معادله؛ لیست = معادله‌ها
    let mut eqs: Vec<Sym> = Vec::new();
    let mut vars: Vec<String> = Vec::new();
    for a in args {
        match a {
            Value::List(vs) => {
                if vs.is_empty() {
                    return Err(McError::new(
                        ids::TYPE_ERROR,
                        line,
                        col,
                        "solve: the equation list must not be empty.",
                        "solve: فهرست معادلات نمی‌تواند خالی باشد.",
                    ));
                }
                for v in vs {
                    eqs.push(operand_sym(v, line, col)?);
                }
            }
            Value::Char(_) => vars.push(var_name_arg(a, "solve", line, col)?),
            Value::Sym(s) if matches!(&**s, Sym::Symbol(_)) => {
                vars.push(var_name_arg(a, "solve", line, col)?)
            }
            v => eqs.push(operand_sym(v, line, col)?),
        }
    }
    if eqs.is_empty() {
        return Err(McError::new(
            ids::TYPE_ERROR,
            line,
            col,
            "solve: no equation found — symbolic expressions are equations in eq = 0 form.",
            "solve: معادله‌ای پیدا نشد — عبارت‌های سمبولیک در فرم eq = 0 معادله‌اند.",
        ));
    }
    // متغیر بدون نام → انتخاب خودکار = اجتماع نمادهای آزاد (symvar متلب‌گونه)
    if vars.is_empty() {
        for e in &eqs {
            for s in e.free_symbols() {
                if !vars.contains(&s) {
                    vars.push(s);
                }
            }
        }
        vars.sort();
    }
    if eqs.len() == 1 && vars.len() == 1 {
        let roots =
            mc_cas::solve::solve_expr(&eqs[0], &vars[0]).map_err(|e| cas_err(e, line, col))?;
        return Ok(Value::List(
            roots.into_iter().map(|r| Value::Sym(Arc::new(r))).collect(),
        ));
    }
    if vars.is_empty() {
        return Err(McError::new(
            ids::TYPE_ERROR,
            line,
            col,
            "solve: the equations contain no symbolic variable.",
            "solve: معادله‌ها هیچ متغیر سمبولیک ندارند.",
        ));
    }
    // دستگاه (خطی یکتا/پارامتریک یا غیرخطی ۲×۲) — لیست تخت به‌ازای هر متغیر
    let sols = mc_cas::solve::solve_system(&eqs, &vars).map_err(|e| cas_err(e, line, col))?;
    Ok(Value::List(
        sols.into_iter().map(|r| Value::Sym(Arc::new(r))).collect(),
    ))
}
