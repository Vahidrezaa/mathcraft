//! Environment — فضای کاری متغیرها (سند ۰۲ بخش ۴: سمانتیک سریالی workspace).
//!
//! did-you-mean: فاصله Levenshtein ≤ ۲ (سند ۰۲ بخش ۹) — در P0 فقط روی
//! متغیرهای workspace؛ رجیستری builtins در P1 به همین تابع وصل می‌شود
//! (سند ۰۷: «`lenght` → پیشنهاد `length`»).

use std::collections::HashMap;

use mc_core::Value;

#[derive(Debug, Clone, Default)]
pub struct Environment {
    vars: HashMap<String, Value>,
}

impl Environment {
    pub fn new() -> Self {
        Self::default()
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
    pub fn clear(&mut self) -> usize {
        let n = self.vars.len();
        self.vars.clear();
        n
    }

    pub fn contains(&self, name: &str) -> bool {
        self.vars.contains_key(name)
    }

    /// اسنپ‌شات نام↔مقدار برای محاسبه workspace_delta (سند ۰۲ بخش ۳) —
    /// کپی ارزان است چون Value از Arc/ COW استفاده می‌کند.
    pub fn snapshot(&self) -> HashMap<String, Value> {
        self.vars.clone()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &Value)> {
        self.vars.iter()
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
}
