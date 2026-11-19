//! فایل‌سازمان مجازی نشست (VFS) — پل میزبان↔موتور برای بایت‌های فایل.
//!
//! **چرا:** روی اندروید موتور Rust به مسیرهای SAF دسترسی ندارد و پرمیشن
//! storage نمی‌خواهیم. الگوی موجود پروژه (`register_m_source`) همین کار را
//! برای متن .m می‌کند؛ اینجا برای بایت‌های هر فایل تکرار می‌شود:
//! - میزبان (Dart) فایل‌های پوشهٔ MathCraft و فایل‌های گشوده‌شده را پیش از
//!   ارزیابی با [`register`] ثبت می‌کند (نام = نام فایل بدون مسیر).
//! - دستورهای `load`/`run`/`readmatrix` با [`lookup`] می‌خوانند.
//! - خروجی‌های `save`/`writematrix` با [`put_output`] در صف می‌روند و میزبان
//!   با [`take_outputs`] یک‌جا می‌گیرد و روی دیسک می‌نویسد (آینهٔ workspace_delta).
//!
//! thread-local است تا همان قرارداد «ارزیابی سریالی سند ۰۲ §4» را حفظ کند و
//! تست‌ها بدون هم‌زمانی مصرف کنند؛ `clear_all` بین تست‌ها فرا‌خوانده می‌شود.

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

/// نام فایل را نرمال می‌کند: اسلش‌های آغاشین حذف و نام فایل (basename) جدا
/// می‌شود — کاربر می‌تواند `load('~/MathCraft/data.mat')` هم بزند و همان
/// `data.mat` ثبت‌شده پیدا شود. اندازهٔ حافظهٔ ثبت‌شده را برمی‌گرداند.
fn normalize(name: &str) -> &str {
    let trimmed = name.trim();
    match trimmed.rsplit(['/', '\\']).next() {
        Some(base) if !base.is_empty() => base,
        _ => "",
    }
}

#[derive(Default)]
struct VfsStore {
    files: HashMap<String, std::sync::Arc<Vec<u8>>>,
    outputs: Vec<(String, Vec<u8>)>,
}

thread_local! {
    static STORE: RefCell<VfsStore> = RefCell::new(VfsStore::default());
}

/// ثبت/جایگزینی محتوای یک فایل (میزبان → موتور). اگر از قبل هست، جایگزین
/// می‌شود (فایل تازه‌تر). نام خالی نادیده گرفته می‌شود.
pub fn register(name: &str, data: Vec<u8>) {
    register_shared(name, Arc::new(data));
}

/// ثبت با مالکیت مشترک — لایهٔ FFI آینهٔ سراسری را بدون کپی بایت‌ها
/// به نخِ هر job سینک می‌کند (فقط clone شمارشی Arc).
pub fn register_shared(name: &str, data: Arc<Vec<u8>>) {
    let key = normalize(name).to_owned();
    if key.is_empty() {
        return;
    }
    STORE.with(|s| {
        s.borrow_mut().files.insert(key, data);
    });
}

/// خواندن محتوای فایل ثبت‌شده (کپی نمی‌شود — Rc).
pub fn lookup(name: &str) -> Option<Arc<Vec<u8>>> {
    let key = normalize(name);
    STORE.with(|s| s.borrow().files.get(key).cloned())
}

/// آیا فایلی با این نام (یا هر پسوند/بی‌پسوندِ هم‌ستیر) ثبت شده؟
/// برای پیام خطای راهنما: «فایل X.mat نیست؛ X.mcws هست».
pub fn known_names() -> Vec<String> {
    STORE.with(|s| s.borrow().files.keys().cloned().collect())
}

/// صف‌کردن یک خروجی برای نوشتن توسط میزبان (موتور → میزبان).
/// نام خالی نادیده گرفته می‌شود؛ خروجی‌های هم‌نام به‌ترتیب صف می‌مانند
/// (میزبان آخری را می‌نویسد — رفتار بازنویسی save پیدرپی).
pub fn put_output(name: &str, data: Vec<u8>) {
    let key = normalize(name).to_owned();
    if key.is_empty() {
        return;
    }
    STORE.with(|s| s.borrow_mut().outputs.push((key, data)));
}

/// برداشت یکجای صف خروجی (ترتیب درج) — میزبان بعد از هر ارزیابی صدا می‌زند.
pub fn take_outputs() -> Vec<(String, Vec<u8>)> {
    STORE.with(|s| std::mem::take(&mut s.borrow_mut().outputs))
}

/// پاک‌کردن کامل — فقط برای تست‌ها (جدا نگه‌داشتن حالت نشست).
pub fn clear_all() {
    STORE.with(|s| {
        *s.borrow_mut() = VfsStore::default();
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_lookup_roundtrip() {
        clear_all();
        register("data.csv", b"a,b".to_vec());
        let got = lookup("data.csv").expect("must be registered");
        assert_eq!(&*got, b"a,b");
        assert!(lookup("missing.csv").is_none());
        clear_all();
    }

    #[test]
    fn basename_normalization() {
        clear_all();
        register("data.mat", vec![1, 2, 3]);
        // مسیر کامل به همان کلید می‌خورد
        assert!(lookup("~/MathCraft/data.mat").is_some());
        assert!(lookup("./data.mat").is_some());
        assert!(lookup("C:\\x\\data.mat").is_some());
        clear_all();
    }

    #[test]
    fn outputs_fifo_order_and_drain() {
        clear_all();
        put_output("a.mcws", vec![1]);
        put_output("b.csv", vec![2]);
        put_output("a.mcws", vec![3]);
        let outs = take_outputs();
        assert_eq!(outs.len(), 3);
        assert_eq!(outs[0], ("a.mcws".to_owned(), vec![1]));
        assert_eq!(outs[2], ("a.mcws".to_owned(), vec![3]));
        // درین شد — فراخوانی دوم خالی است
        assert!(take_outputs().is_empty());
        clear_all();
    }

    #[test]
    fn replace_on_reregister_and_empty_names_ignored() {
        clear_all();
        register("f.txt", vec![1]);
        register("f.txt", vec![9]);
        assert_eq!(&*lookup("f.txt").expect("registered"), &[9]);
        register("", vec![1]);
        register("/", vec![1]);
        assert!(known_names().len() <= 1);
        clear_all();
    }
}
