//! ارزیابی — سمانتیک سند ۰۳ (بخش‌های ۵، ۱۱ و ۱۴).
//!
//! - `ans`: فقط دستورهای عبارتیِ «نمایش‌دار» در `ans` می‌نشینند (رفتار متلب؛
//!   `1;` نه نمایش دارد نه `ans` را عوض می‌کند — سند ۰۳ بخش ۱۱ بند ۳).
//! - خطا یک مقدار است: ارزیابی در اولین خطا متوقف و `McError` برمی‌گردد؛
//!   خروجی‌های قبلی همان خط حفظ می‌شوند (سند ۰۲ بخش ۹).
//! - محاسبات عددی IEEE؛ `1/0 = Inf`، `0/0 = NaN` (سند ۰۳ بخش ۱۱ بند ۵).
//! - ماتریس تهی: نتیجه هر عمل با `[]` بازهم `[]` است (سند ۰۳ بخش ۴).

use mc_core::{ids, Matrix, McError, Value};
use mc_parser::{BinOp, Expr, Program, Stmt, UnOp};

use crate::env::Environment;
use crate::format::format_value;

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
}

pub fn eval_program(program: &Program, env: &mut Environment) -> EvalOutcome {
    let mut out = EvalOutcome::default();
    for stmt in &program.stmts {
        match stmt {
            Stmt::Expr { expr, suppress } => {
                let value = match eval_expr(expr, env) {
                    Ok(v) => v,
                    Err(e) => {
                        out.error = Some(e);
                        return out;
                    }
                };
                if !suppress {
                    env.set("ans", value.clone());
                    out.outputs.push(StmtOutput {
                        text: format!("ans = {}", format_value(&value)),
                    });
                }
            }
            Stmt::Assign {
                name,
                expr,
                suppress,
                ..
            } => {
                let value = match eval_expr(expr, env) {
                    Ok(v) => v,
                    Err(e) => {
                        out.error = Some(e);
                        return out;
                    }
                };
                env.set(name.clone(), value.clone());
                if !suppress {
                    out.outputs.push(StmtOutput {
                        text: format!("{name} = {}", format_value(&value)),
                    });
                }
            }
        }
    }
    out
}

fn eval_expr(expr: &Expr, env: &mut Environment) -> Result<Value, McError> {
    match expr {
        Expr::Num(n) => Ok(Value::Double(Matrix::scalar(*n))),
        Expr::Var { name, line, col } => env.get(name).cloned().ok_or_else(|| {
            let e = McError::new(
                ids::UNDEFINED_VARIABLE,
                *line,
                *col,
                format!("Undefined variable '{name}'."),
                format!("متغیر '{name}' تعریف نشده."),
            );
            match env.suggest_name(name) {
                Some(s) => e.with_suggestion(s),
                None => e,
            }
        }),
        Expr::Unary { op, operand } => {
            let v = eval_expr(operand, env)?;
            Ok(apply_scalarwise(&v, |x| match op {
                UnOp::Neg => -x,
                UnOp::Pos => x,
            }))
        }
        Expr::Binary {
            op,
            lhs,
            rhs,
            line,
            col,
        } => {
            let a = eval_expr(lhs, env)?;
            let b = eval_expr(rhs, env)?;
            binary(*op, a, b, *line, *col)
        }
    }
}

/// اعمال تابع روی همه عناصر (اسکالر یا ماتریسی) — COW پس کپی فقط در صورت اشتراک.
fn apply_scalarwise(v: &Value, f: impl Fn(f64) -> f64) -> Value {
    match v {
        Value::Double(m) => {
            let mut out = m.clone();
            for idx in 0..out.numel() {
                out.set_linear(idx, f(out.get_linear(idx)));
            }
            Value::Double(out)
        }
    }
}

/// عنصر‌به‌عنصر روی دو ماتریس **هم‌شکل**.
fn elementwise2(a: &Matrix, b: &Matrix, f: impl Fn(f64, f64) -> f64) -> Value {
    let mut out = a.clone();
    for c in 0..out.cols() {
        for r in 0..out.rows() {
            out.set(r, c, f(a.get(r, c), b.get(r, c)));
        }
    }
    Value::Double(out)
}

fn err_unsupported(line: u32, col: u32, what_en: &str, what_fa: &str) -> McError {
    McError::new(
        ids::UNSUPPORTED,
        line,
        col,
        format!("{what_en} arrives in a later milestone (linear-algebra dispatch, doc 04 §3)."),
        format!("{what_fa} در مایل‌استون‌های بعدی می‌آید (dispatch جبر خطی، سند ۰۴ بخش ۳)."),
    )
}

fn binary(op: BinOp, a: Value, b: Value, line: u32, col: u32) -> Result<Value, McError> {
    let fa = a.as_scalar();
    let fb = b.as_scalar();

    // اسکالر ↔ اسکالر: مستقیم
    if let (Some(x), Some(y)) = (fa, fb) {
        return Ok(Value::Double(Matrix::scalar(match op {
            BinOp::Add => x + y,
            BinOp::Sub => x - y,
            BinOp::Mul => x * y,
            BinOp::Div => x / y,
            BinOp::Pow => x.powf(y),
        })));
    }

    let a_empty = matches!(&a, Value::Double(m) if m.is_empty());
    let b_empty = matches!(&b, Value::Double(m) if m.is_empty());
    let (ar, ac) = a.shape();
    let (br, bc) = b.shape();

    match op {
        BinOp::Add | BinOp::Sub => {
            if a_empty || b_empty {
                return Ok(Value::Double(Matrix::full(0, 0, 0.0)));
            }
            let f = |x: f64, y: f64| match op {
                BinOp::Add => x + y,
                _ => x - y,
            };
            if a_scalar_shape(ar, ac) {
                let x = fa.unwrap_or(0.0);
                return Ok(apply_scalarwise(&b, |y| f(x, y)));
            }
            if a_scalar_shape(br, bc) {
                let y = fb.unwrap_or(0.0);
                return Ok(apply_scalarwise(&a, |x| f(x, y)));
            }
            if (ar, ac) == (br, bc) {
                // Value در P0 فقط یک واریانت دارد — الگو بی‌شرط است.
                let Value::Double(ma) = &a;
                let Value::Double(mb) = &b;
                return Ok(elementwise2(ma, mb, f));
            }
            Err(McError::new(
                ids::SIZE_MISMATCH,
                line,
                col,
                format!("Matrix dimensions must agree: ({ar}×{ac}) vs ({br}×{bc})."),
                format!("ابعاد ماتریس‌ها نمی‌خواند: ({ar}×{ac}) با ({br}×{bc})."),
            ))
        }
        BinOp::Mul => {
            if a_empty || b_empty {
                return Ok(Value::Double(Matrix::full(0, 0, 0.0)));
            }
            if a_scalar_shape(ar, ac) {
                let x = fa.unwrap_or(0.0);
                return Ok(apply_scalarwise(&b, |y| x * y));
            }
            if a_scalar_shape(br, bc) {
                let y = fb.unwrap_or(0.0);
                return Ok(apply_scalarwise(&a, |x| x * y));
            }
            // ضرب ماتریسی در P1 با dispatch سند ۰۴ بخش ۳ می‌آید — رد صادقانه.
            Err(err_unsupported(
                line,
                col,
                "Matrix product (*), shapes accepted",
                "ضرب ماتریسی (*)",
            ))
        }
        BinOp::Div => {
            if a_empty || b_empty {
                return Ok(Value::Double(Matrix::full(0, 0, 0.0)));
            }
            if a_scalar_shape(br, bc) {
                let y = fb.unwrap_or(1.0);
                return Ok(apply_scalarwise(&a, |x| x / y));
            }
            // A/s مجاز است (rdivide)؛ s/A و A/B ماتریسی = mrdivide → P1.
            Err(err_unsupported(
                line,
                col,
                "mrdivide (/) on matrices",
                "تقسیم ماتریسی (/)",
            ))
        }
        BinOp::Pow => {
            // A^2 (توان ماتریسی) و A.^2 (عضوی) هر دو P1 — رد صادقانه.
            Err(err_unsupported(
                line,
                col,
                "Power operator on arrays",
                "اپراتور توان روی آرایه‌ها",
            ))
        }
    }
}

fn a_scalar_shape(r: usize, c: usize) -> bool {
    r == 1 && c == 1
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::env::Environment;
    use mc_parser::parse;

    fn run(src: &str, env: &mut Environment) -> EvalOutcome {
        let program = parse(src).unwrap_or_else(|e| panic!("parse {src:?}: {e}"));
        eval_program(&program, env)
    }

    fn last_text(out: &EvalOutcome) -> &str {
        out.outputs
            .last()
            .map(|o| o.text.as_str())
            .unwrap_or("<no output>")
    }

    #[test]
    fn demo_one_plus_one() {
        // دمو رسمی P0 (سند ۰۷): کنسول خالی که 1+1 = 2 را نشان می‌دهد.
        let mut env = Environment::new();
        let out = run("1+1", &mut env);
        assert!(out.error.is_none());
        assert_eq!(last_text(&out), "ans = 2");
    }

    #[test]
    fn assignment_and_ans_chain() {
        let mut env = Environment::new();
        let out = run("x = 5", &mut env);
        assert_eq!(last_text(&out), "x = 5");
        let out = run("2^3^2", &mut env); // چپ‌گرا: 64
        assert_eq!(last_text(&out), "ans = 64");
        let out = run("x + ans", &mut env);
        assert_eq!(last_text(&out), "ans = 69");
    }

    #[test]
    fn suppression_semantics() {
        let mut env = Environment::new();
        let out = run("1; x = 2;", &mut env);
        assert!(out.outputs.is_empty());
        assert!(env.get("x").is_some());
        // ans نباید عوض شده باشد — فقط دستور نمایش‌دار ans را می‌سازد.
        assert!(env.get("ans").is_none());
    }

    #[test]
    fn matlab_precedence_examples() {
        let mut env = Environment::new();
        assert_eq!(last_text(&run("-2^2", &mut env)), "ans = -4");
        assert_eq!(last_text(&run("2^-2", &mut env)), "ans = 0.25");
        assert_eq!(last_text(&run("6/2*3", &mut env)), "ans = 9");
        assert_eq!(last_text(&run("1-2-3", &mut env)), "ans = -4");
        assert_eq!(last_text(&run("(1+2)*3", &mut env)), "ans = 9");
    }

    #[test]
    fn ieee_semantics() {
        let mut env = Environment::new();
        assert_eq!(last_text(&run("1/0", &mut env)), "ans = Inf");
        assert_eq!(last_text(&run("0/0", &mut env)), "ans = NaN");
        assert_eq!(last_text(&run("-1/0", &mut env)), "ans = -Inf");
    }

    #[test]
    fn undefined_variable_with_did_you_mean() {
        let mut env = Environment::new();
        run("x1 = 7", &mut env);
        let out = run("x2 + 1", &mut env);
        let err = out.error.expect("error expected");
        assert_eq!(err.id, ids::UNDEFINED_VARIABLE);
        assert_eq!(err.suggestion.as_deref(), Some("x1"));
        assert_eq!(err.line, 1);
        assert_eq!(err.col, 1);
    }

    #[test]
    fn size_mismatch_reports_both_shapes() {
        let mut env = Environment::new();
        env.set("A", Value::Double(Matrix::full(2, 3, 1.0)));
        env.set("B", Value::Double(Matrix::full(4, 3, 1.0)));
        let out = run("A + B", &mut env);
        let err = out.error.expect("mismatch expected");
        assert_eq!(err.id, ids::SIZE_MISMATCH);
        assert!(err.message_en.contains("(2×3)"));
        assert!(err.message_en.contains("(4×3)"));
    }

    #[test]
    fn scalar_broadcast_and_same_shape_add() {
        let mut env = Environment::new();
        env.set(
            "A",
            Value::Double(Matrix::from_columns(2, 1, vec![1.0, 2.0])),
        );
        let out = run("A + 1", &mut env);
        assert_eq!(last_text(&out), "ans = [2; 3]");
        let out = run("A + A", &mut env);
        assert_eq!(last_text(&out), "ans = [2; 4]");
    }

    #[test]
    fn empty_matrix_semantics() {
        let mut env = Environment::new();
        env.set("E", Value::Double(Matrix::full(0, 0, 0.0)));
        assert_eq!(last_text(&run("E + 1", &mut env)), "ans = []");
    }

    #[test]
    fn matrix_product_is_honestly_rejected_in_p0() {
        let mut env = Environment::new();
        env.set("A", Value::Double(Matrix::full(2, 2, 1.0)));
        let out = run("A * A", &mut env);
        let err = out.error.expect("unsupported expected");
        assert_eq!(err.id, ids::UNSUPPORTED);
    }
}
