//! ثبت منبع فایل‌های `.m` — سمانتیک متلب §9 برای «Scripts And Functions».
//!
//! - فایل با تعریف `function` = فایل تابع: نام تابع اصلی باید با نام فایل یکی
//!   باشد (رفتار متلب — متلب نام *فایل* را فراخوانی‌پذیر می‌کند).
//! - فایل بدون تعریف = اسکریپت: با نام خودش اجرا می‌شود (در فضای کاری فراخوان).
//! - اسکریپت با توابع محلی (R2016b+): هم اسکریپت ثبت می‌شود و هم توابع.
//! - توابع محلی بعد از تابع اصلی ثبت می‌شوند.

use mc_core::{ids, McError};
use mc_parser::Stmt;

use crate::env::Environment;
use crate::eval::eval_program;

/// گزارش ثبت یک منبع `.m`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SourceReport {
    /// نام توابع ثبت‌شده به ترتیب تعریف (اولی = تابع اصلی).
    pub functions: Vec<String>,
    /// بخش اسکریپتیِ فایل ثبت شد (با نام فایل فراخوانی‌پذیر است).
    pub script_registered: bool,
}

/// ثبت منبع `source` با نام فایل `name` (بدون پسوند) در مخزن نشست.
pub fn register_source(
    env: &mut Environment,
    name: &str,
    source: &str,
) -> Result<SourceReport, McError> {
    validate_file_name(name)?;
    let program = mc_parser::parse(source)?;
    let mut functions = Vec::new();
    let mut has_script_part = false;
    for stmt in &program.stmts {
        match stmt {
            Stmt::FnDef { name, .. } => {
                if !functions.contains(name) {
                    functions.push(name.clone());
                }
            }
            _ => has_script_part = true,
        }
    }
    // سمانتیک متلب: نام تابع اصلی = نام فایل؛ مغایرت = خطای آموزشی
    if let Some(primary) = functions.first() {
        if primary != name && !has_script_part {
            return Err(McError::new(
                ids::PARSE_ERROR,
                0,
                0,
                format!(
                    "Function name '{primary}' must match the file name '{name}'. Rename the file or the function."
                ),
                format!(
                    "نام تابع اصلی «{primary}» باید با نام فایل «{name}» یکی باشد؛ نام فایل یا تابع را اصلاح کنید."
                ),
            ));
        }
    }
    // ثبت همه تعریف‌ها با اجرای گزاره‌های FnDef — ولی بخش اسکریپتی باید جدا
    // اجرا شود (ثبت نباید گزاره‌های اجرایی را اجرا کند — سمانتیک متلب).
    let mut def_program = mc_parser::Program::default();
    for stmt in &program.stmts {
        if matches!(stmt, Stmt::FnDef { .. }) {
            def_program.stmts.push(stmt.clone());
        }
    }
    if !def_program.stmts.is_empty() {
        let outcome = eval_program(&def_program, env);
        if let Some(e) = outcome.error {
            return Err(e);
        }
    }
    let script_registered = if has_script_part {
        // اسکریپت (یا اسکریپت با توابع محلی) با نام فایل ثبت می‌شود
        env.register_script(name.to_owned(), program);
        true
    } else {
        false
    };
    Ok(SourceReport {
        functions,
        script_registered,
    })
}

/// نام فایل: حرف/زیرخط در آغاز، سپس حرف/رقم/زیرخط (قاعده شناسهٔ سند ۰۳ §2).
fn validate_file_name(name: &str) -> Result<(), McError> {
    let mut chars = name.chars();
    let ok = match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {
            chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
        }
        _ => false,
    };
    if ok {
        Ok(())
    } else {
        Err(McError::new(
            ids::PARSE_ERROR,
            0,
            0,
            format!(
                "'{name}' is not a valid .m file name. Use letters, digits and underscore, starting with a letter."
            ),
            format!(
                "«{name}» نام معتبر فایل .m نیست: حروف، رقم و زیرخط، با آغازِ حرفی."
            ),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mc_core::Value;

    #[test]
    fn register_function_file_and_call() {
        let mut env = Environment::new();
        let r =
            register_source(&mut env, "sq", "function y = sq(x)\ny = x^2\nend").expect("register");
        assert_eq!(r.functions, vec!["sq".to_owned()]);
        assert!(!r.script_registered);
        let out = eval_program(&parse_ok("sq(9)"), &mut env);
        assert!(out.error.is_none());
        assert_eq!(
            env.get("ans"),
            Some(&Value::Double(mc_core::Matrix::scalar(81.0)))
        );
    }

    #[test]
    fn register_script_file_and_run() {
        let mut env = Environment::new();
        let r = register_source(&mut env, "setup", "a = 5\nb = a * 2").expect("register");
        assert!(r.functions.is_empty());
        assert!(r.script_registered);
        let out = eval_program(&parse_ok("setup"), &mut env);
        assert!(out.error.is_none());
        assert_eq!(
            env.get("b"),
            Some(&Value::Double(mc_core::Matrix::scalar(10.0)))
        );
    }

    #[test]
    fn primary_name_mismatch_is_rejected() {
        let mut env = Environment::new();
        let e = register_source(&mut env, "file1", "function y = other(x)\ny = x\nend")
            .expect_err("mismatch must error");
        assert!(e.message_en.contains("must match"), "{e:?}");
    }

    #[test]
    fn script_part_stays_dormant_on_registration() {
        let mut env = Environment::new();
        // فایل اسکریپت‌دار: ثبت نباید گزارهٔ اجرایی را اجرا کند
        let r = register_source(
            &mut env,
            "myscript",
            "x = 123\nfunction y = f(); y = 1; end",
        )
        .expect("register");
        assert!(r.script_registered);
        assert_eq!(r.functions, vec!["f".to_owned()]);
        assert!(env.get("x").is_none(), "ثبت نباید اسکریپت را اجرا کند");
    }

    #[test]
    fn invalid_file_name_rejected() {
        let mut env = Environment::new();
        assert!(register_source(&mut env, "9bad", "a = 1").is_err());
        assert!(register_source(&mut env, "has space", "a = 1").is_err());
        assert!(register_source(&mut env, "ok_name_2", "a = 1").is_ok());
    }

    fn parse_ok(src: &str) -> mc_parser::Program {
        mc_parser::parse(src).expect("parse")
    }
}
