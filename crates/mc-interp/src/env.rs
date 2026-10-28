//! Environment — فضای کاری متغیرها + مخزن توابع/اسکریپت‌های ثبت‌شده
//! (سند ۰۲ بخش ۴: سمانتیک سریالی workspace؛ سند ۰۳ §9: توابع فایلی).
//!
//! did-you-mean: فاصله Levenshtein ≤ ۲ (سند ۰۲ بخش ۹) — روی متغیرهای workspace
//! و نام‌های ثبت‌شده (تابع/اسکریپت) و builtins (در eval.rs).

use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, Mutex};

use mc_core::Value;
use mc_parser::Program;

/// تعریف تابع فایلی ثبت‌شده — `function [out] = name(params) body end`.
#[derive(Debug, Clone)]
pub struct UserFnDef {
    pub name: String,
    /// خروجی‌ها به ترتیب تعریف — `[a,b] = f()` به همین ترتیب مقدار می‌گیرند.
    pub outputs: Vec<String>,
    pub params: Vec<String>,
    pub body: Program,
    pub line: u32,
    pub col: u32,
}

/// یک آیتم مخزن — تابع فایلی یا اسکریپت (فایل .m بدون تعریف function).
#[derive(Debug, Clone)]
pub enum StoredItem {
    Fn(Arc<UserFnDef>),
    Script(Arc<Program>),
}

/// مخزن توابع/اسکریپت‌های نشست — مشترک بین همهٔ Environmentهای یک نشست
/// (سمانتیک متلب: پس از تعریف/ثبت، فایل‌ها سراسری و فراخوانی‌پذیرند؛ بازگشت
/// recursive از طریق همین مخزن کار می‌کند چون ورک‌اسپیس فرزند هم آن را می‌بیند).
#[derive(Debug, Default)]
pub struct FuncStore {
    items: Mutex<BTreeMap<String, StoredItem>>,
}

impl FuncStore {
    fn lock(&self) -> std::sync::MutexGuard<'_, BTreeMap<String, StoredItem>> {
        self.items
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    pub fn register(&self, name: String, item: StoredItem) {
        // تعریف مجدد = جایگزینی (رفتار متلب: آخرین تعریف برنده است)
        self.lock().insert(name, item);
    }

    pub fn lookup(&self, name: &str) -> Option<StoredItem> {
        self.lock().get(name).cloned()
    }

    /// همه نام‌های ثبت‌شده — مرتب (برای who/whos و پیشنهاد خطا).
    pub fn names(&self) -> Vec<String> {
        self.lock().keys().cloned().collect()
    }

    pub fn count(&self) -> usize {
        self.lock().len()
    }

    /// پاک‌کردن همه تعریف‌ها (سمانتیک `clear functions` متلب) — تعداد پاک‌شده.
    pub fn clear(&self) -> usize {
        let n = self.count();
        self.lock().clear();
        n
    }

    /// نزدیک‌ترین نام ثبت‌شده با فاصله ≤ ۲ (سند ۰۲ بخش ۹).
    pub fn suggest(&self, typo: &str) -> Option<String> {
        let mut best: Option<(usize, String)> = None;
        for name in self.lock().keys() {
            if name == typo {
                continue;
            }
            let d = levenshtein(typo, name);
            if d > 2 {
                continue;
            }
            let better = match &best {
                None => true,
                Some((bd, bn)) => d < *bd || (d == *bd && name.as_str() < bn.as_str()),
            };
            if better {
                best = Some((d, name.clone()));
            }
        }
        best.map(|(_, n)| n)
    }
}

#[derive(Debug, Clone, Default)]
pub struct Environment {
    vars: HashMap<String, Value>,
    /// مخزن توابع — Arc مشترک: کپی Environment (snapshot بی‌نام و ...) آن را
    /// تکرار نمی‌کند و ورک‌اسپیس تازهٔ توابع فایلی هم تعریف‌ها را می‌بیند.
    funcs: Arc<FuncStore>,
    /// حالت نمایش کنسول — `format` command (ROADMAP 2.4 — آیتم ۱۱ ممیزی).
    /// جلسه‌محور است (در snapshot/workspace_delta نمی‌آید) مثل متلب که
    /// `format` حالت پنجرهٔ فرمان است نه متغیر.
    pub display: crate::format::DisplaySpec,
}

impl Environment {
    pub fn new() -> Self {
        Self::default()
    }

    /// ورک‌اسپیس تازه (ورک‌اسپیس تابع فایلی) — متغیرهای فراخوان را *نمی‌بیند*
    /// (سمانتیک متلب §9) ولی همان مخزن توابع نشست را می‌بیند.
    pub fn new_with_funcs(funcs: Arc<FuncStore>) -> Self {
        Self {
            vars: HashMap::new(),
            funcs,
            display: crate::format::DisplaySpec::default(),
        }
    }

    pub fn get(&self, name: &str) -> Option<&Value> {
        self.vars.get(name)
    }

    pub fn set(&mut self, name: impl Into<String>, value: Value) {
        self.vars.insert(name.into(), value);
    }

    pub fn remove(&mut self, name: &str) -> Option<Value> {
        self.vars.remove(name)
    }

    /// پاک‌کردن همه متغیرها — تعداد پاک‌شده (سمانتیک `clear` متلب، P0.1 سند ۰۲ بخش ۳).
    /// تعریف‌های تابع دست نمی‌خورد — برای آن‌ها `clear_functions` است.
    pub fn clear(&mut self) -> usize {
        let n = self.vars.len();
        self.vars.clear();
        n
    }

    pub fn contains(&self, name: &str) -> bool {
        self.vars.contains_key(name)
    }

    /// اسنپ‌شات نام↔مقدار برای محاسبه workspace_delta (سند ۰۲ بخش ۳) و
    /// capture-by-value توابع بی‌نام (سند ۰۳ §9) — کپی ارزان است چون
    /// Value از Arc/COW استفاده می‌کند.
    pub fn snapshot(&self) -> HashMap<String, Value> {
        self.vars.clone()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &Value)> {
        self.vars.iter()
    }

    // ---------- مخزن توابع (سند ۰۳ §9 — فایل‌های .m) ----------

    pub fn funcs_clone(&self) -> Arc<FuncStore> {
        Arc::clone(&self.funcs)
    }

    pub fn register_function(&self, def: UserFnDef) {
        self.funcs
            .register(def.name.clone(), StoredItem::Fn(Arc::new(def)));
    }

    pub fn register_script(&self, name: impl Into<String>, program: Program) {
        self.funcs
            .register(name.into(), StoredItem::Script(Arc::new(program)));
    }

    pub fn lookup_item(&self, name: &str) -> Option<StoredItem> {
        self.funcs.lookup(name)
    }

    /// نام‌های تابع/اسکریپت ثبت‌شده — مرتب (برای who/whos و UI).
    pub fn function_names(&self) -> Vec<String> {
        self.funcs.names()
    }

    pub fn clear_functions(&self) -> usize {
        self.funcs.clear()
    }

    /// نزدیک‌ترین تعریف ثبت‌شده با فاصله ≤ ۲ (did-you-mean برای توابع کاربر).
    pub fn suggest_function(&self, typo: &str) -> Option<String> {
        self.funcs.suggest(typo)
    }

    /// نزدیک‌ترین نام موجود با فاصله ≤ ۲ (سند ۰۲ بخش ۹).
    pub fn suggest_name(&self, typo: &str) -> Option<String> {
        let mut best: Option<(usize, String)> = None;
        for name in self.vars.keys() {
            if name == typo {
                continue;
            }
            let d = levenshtein(typo, name);
            if d > 2 {
                continue;
            }
            let better = match &best {
                None => true,
                Some((bd, bn)) => d < *bd || (d == *bd && name.as_str() < bn.as_str()),
            };
            if better {
                best = Some((d, name.clone()));
            }
        }
        best.map(|(_, n)| n)
    }
}

/// فاصله ویرایش کلاسیک (حافظه O(min)) — روی codepoint‌ها (سازگار با سند ۰۳ بخش ۱۰).
pub fn levenshtein(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    if a.is_empty() {
        return b.len();
    }
    if b.is_empty() {
        return a.len();
    }
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut cur = vec![0usize; b.len() + 1];
    for i in 1..=a.len() {
        cur[0] = i;
        for j in 1..=b.len() {
            let cost = usize::from(a[i - 1] != b[j - 1]);
            cur[j] = (prev[j] + 1).min(cur[j - 1] + 1).min(prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut cur);
    }
    prev[b.len()]
}

#[cfg(test)]
mod tests {
    use super::*;
    use mc_core::Matrix;

    #[test]
    fn levenshtein_basics() {
        assert_eq!(levenshtein("lenght", "length"), 2); // جابجایی دو همسایه = ۲ ویرایش
        assert_eq!(levenshtein("x1", "x2"), 1);
        assert_eq!(levenshtein("abc", "abc"), 0);
    }

    #[test]
    fn suggestion_within_two_edits() {
        let mut env = Environment::new();
        env.set("length", Value::Double(Matrix::scalar(5.0)));
        env.set("x1", Value::Double(Matrix::scalar(1.0)));
        assert_eq!(env.suggest_name("lenght").as_deref(), Some("length"));
        assert_eq!(env.suggest_name("x2").as_deref(), Some("x1"));
        assert_eq!(env.suggest_name("zzzzzzzz"), None);
    }

    #[test]
    fn clear_counts_and_empties() {
        let mut env = Environment::new();
        assert_eq!(env.clear(), 0);
        env.set("a", Value::Double(Matrix::scalar(1.0)));
        env.set("b", Value::Double(Matrix::scalar(2.0)));
        assert_eq!(env.clear(), 2);
        assert!(!env.contains("a"));
        assert!(!env.contains("b"));
    }

    #[test]
    fn func_store_register_lookup_clear() {
        let store = FuncStore::default();
        assert_eq!(store.count(), 0);
        let def = UserFnDef {
            name: "sq".into(),
            outputs: vec!["y".into()],
            params: vec!["x".into()],
            body: Program { stmts: Vec::new() },
            line: 1,
            col: 1,
        };
        store.register("sq".into(), StoredItem::Fn(Arc::new(def)));
        assert!(store.lookup("sq").is_some());
        assert_eq!(store.names(), vec!["sq".to_owned()]);
        assert_eq!(store.suggest("sq1").as_deref(), Some("sq"));
        assert_eq!(store.clear(), 1);
        assert!(store.lookup("sq").is_none());
    }

    #[test]
    fn environment_shares_func_store_across_clones_and_children() {
        let mut env = Environment::new();
        let child = Environment::new_with_funcs(env.funcs_clone());
        // تعریف از طریق یکی، از دیگری هم دیده می‌شود (مخزن مشترک نشست)
        child.register_function(UserFnDef {
            name: "f".into(),
            outputs: Vec::new(),
            params: Vec::new(),
            body: Program { stmts: Vec::new() },
            line: 1,
            col: 1,
        });
        assert!(env.lookup_item("f").is_some());
        // متغیرها مشترک نیستند
        env.set("x", Value::Double(Matrix::scalar(1.0)));
        assert!(child.get("x").is_none());
        // clear متغیرها مخزن را دست نمی‌زند
        assert_eq!(env.clear(), 1);
        assert!(env.lookup_item("f").is_some());
    }
}
