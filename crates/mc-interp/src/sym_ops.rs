//! عملگرهای سمبولیک مفسر (گام 4.1/CAS-0 — سند ۰۵ §۲).
//!
//! مسیرِ «هر عملوند Sym دارد» از `apply_binary`/منفی یکه به این‌جا می‌آید.
//! فقط **اسکالر ↔ اسکالر** پشتیبانی می‌شود (ماتریس سمبولیک = CAS-1)؛
//! هر عملِ پشتیبانی‌نشده خطای دوزبانهٔ روشن می‌دهد — هرگز جواب غلطِ
//! عددی نه (اصل صداقت، سند ۰۰ اصل ۵).

use std::sync::Arc;

use mc_cas::{sym_display, CasError, Sym};
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
            format!("Symbolic operation not available in CAS-0: {why}."),
            format!("عمل سمبولیک در CAS-0 در دسترس نیست: {why}."),
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
                "Symbolic arithmetic with matrices arrives in CAS-1 (got {}x{}).",
                m.rows(),
                m.cols()
            ),
            format!(
                "حساب سمبولیک با ماتریس در CAS-1 می‌آید (دریافت شد {}×{}).",
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
            "Complex symbolic values arrive in CAS-1.",
            "مقادیر سمبولیک مختلط در CAS-1 می‌آیند.",
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
