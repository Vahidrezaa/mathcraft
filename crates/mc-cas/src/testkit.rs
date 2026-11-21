//! ابزار تست مشترک ماژول‌های mc-cas (فقط cfg(test)).
#![cfg(test)]

pub use crate::{CasError, Sym};

pub fn sym(n: &str) -> Sym {
    Sym::symbol(n)
}

pub fn int(i: i64) -> Sym {
    Sym::int(i)
}

pub fn rat(n: i64, d: i64) -> Sym {
    Sym::rat(n, d).expect("test rat")
}

pub fn neg(a: &Sym) -> Sym {
    Sym::negated(a.clone())
}

pub fn add(terms: &[Sym]) -> Sym {
    let mut it = terms.iter().cloned();
    let mut acc = it.next().expect("add needs a term");
    for t in it {
        acc = Sym::plus(acc, t);
    }
    acc
}

pub fn times(factors: &[Sym]) -> Sym {
    let mut it = factors.iter().cloned();
    let mut acc = it.next().expect("times needs a factor");
    for f in it {
        acc = Sym::times(acc, f);
    }
    acc
}

pub fn over(a: &Sym, b: &Sym) -> Result<Sym, CasError> {
    Sym::over(a.clone(), b.clone())
}

pub fn pow(b: &Sym, e: &Sym) -> Sym {
    Sym::pow(b.clone(), e.clone()).expect("test pow")
}

pub fn f(name: &str, args: &[Sym]) -> Sym {
    Sym::fn_of(name, args.to_vec()).expect("test fn")
}
