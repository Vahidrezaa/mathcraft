//! تشخیص ساختار ماتریس — فقط «آزمون دقیق صفر» (`v == 0.0`، سند ۰۴ §3.2 شاخه‌های مثلثی/سه‌قطری).
//!
//! این توابع پایه درخت dispatch بک‌اسلش هستند: هر کدام که برچسب بخورد، مسیر حلگر
//! ارزان‌تر (O(n²) یا O(n)) به‌جای LU عمومی O(n³) انتخاب می‌شود.

use mc_core::matrix::Matrix;

/// ماتریس مثلثی است؟ ساختاری و دقیق:
/// - `Some(true)` → بالا-مثلثی (زیرقطر همه دقیقاً صفر)
/// - `Some(false)` → پایین-مثلثی (بالا-مثلثیِ صرف نباشد و بالای قطر همه دقیقاً صفر)
/// - `None` → مخلوط
///
/// ماتریس قطرِ محض هم بالا-مثلثی و هم پایین-مثلثی است؛ قاعده اولویت با `Some(true)`
/// (هر دو حلگر مثلثی جواب می‌دهند، یکی کافی است — سند ۰۴ §3.2).
pub fn is_triangular(a: &Matrix) -> Option<bool> {
    if a.is_empty() {
        // شرط‌ها برای ماتریس تهی برقرارند (شرط جزئی روی هیچ عضوی) — مثلثی بالا.
        return Some(true);
    }
    let mut upper = true;
    let mut lower = true;
    for c in 0..a.cols() {
        for r in 0..a.rows() {
            let v = a.get(r, c);
            let nonzero = v != 0.0; // آزمون دقیق صفر (NaN هم «غیرصفر» شمرده می‌شود — ساختار محافظه‌کار)
            if r > c && nonzero {
                upper = false;
            }
            if r < c && nonzero {
                lower = false;
            }
        }
    }
    if upper {
        Some(true)
    } else if lower {
        Some(false)
    } else {
        None
    }
}

/// ماتریس سه‌قطری (ساختاری)؟ بیرون از سه قطرِ اصلی همه دقیقاً صفر.
/// ورودی غیرمربع یا تهی هم `true` می‌گیرد (شرط جزئی خالی؛ dispatch بعداً شکل را چک می‌کند).
pub fn is_tridiagonal(a: &Matrix) -> bool {
    if a.rows() != a.cols() {
        // سه‌قطری به‌معنای حلگر Thomas فقط برای مربع تعریف می‌شود (سند ۰۴ §3.2).
        return a.is_empty();
    }
    (0..a.rows())
        .all(|r| (0..a.cols()).all(|c| (c as isize - r as isize).abs() <= 1 || a.get(r, c) == 0.0))
}

/// ماتریس «مثلثیِ پرموته»؟ (سند ۰۴ §3.2: مثلثیِ پرموته → permute سپس مثلثی)
///
/// اگر ماتریس از قبل مثلثی باشد `None` برمی‌گردد (خودش مثلثی است، پرموته نیست).
/// وگرنه یک پرموتاسیون ردیفی `perm` (۰-پایه، طول m) برمی‌گرداند به‌طوری که
/// ردیف `i` خروجی برابر ردیف `perm[i]` ورودی باشد و خروجی مثلثی (بالا یا پایین) شود.
///
/// الگوریتم: چینش حریصانه (greedy) روی ستونِ نخستین/آخرین عضو غیرصفر هر ردیف —
/// برای مثلثیِ بالا ردیفِ جایگاه i باید تکیه‌گاهش ⊆ {i..n-1} باشد؛ حریصانه از
/// جایگاه‌های پرمحدودیت شروع می‌کنیم (بهینگی: شرط Hall برای بازه‌ها).
pub fn is_permuted_triangular(a: &Matrix) -> Option<Vec<usize>> {
    if a.is_empty() || a.rows() != a.cols() || is_triangular(a).is_some() {
        return None;
    }
    let n = a.cols();
    let m = a.rows();

    // نخستین ستون غیرصفر هر ردیف (ردیف صفر → بدون قید؛ با usize::MAX نماینده می‌کنیم تا
    // در هر جایگاهی قابل جاگذاری باشد — قید خالی)
    let mut first_nz = vec![usize::MAX; m];
    // آخرین ستون غیرصفر هر ردیف (ردیف صفر → بدون قید؛ با 0 نماینده می‌کنیم)
    let mut last_nz = vec![0usize; m];
    for r in 0..m {
        let mut first = usize::MAX;
        let mut last = 0usize;
        for c in 0..n {
            if a.get(r, c) != 0.0 {
                if first == usize::MAX {
                    first = c;
                }
                last = c;
            }
        }
        first_nz[r] = first;
        last_nz[r] = last;
    }

    // تلاش برای مثلثیِ بالا: جایگاه i (از m-1 پایین) ردیفی با first_nz >= i می‌خواهد.
    if let Some(perm) = greedy_assign(&first_nz, m, false) {
        return Some(perm);
    }
    // مثلثیِ پایین: جایگاه i (از 0 بالا) ردیفی با last_nz <= i می‌خواهد.
    greedy_assign(&last_nz, m, true)
}

/// تخصیص حریصانهٔ ردیف‌ها به جایگاه‌ها.
/// `ascending=false` (مثلثی بالا): کلید هر ردیف = نخستین غیرصفر؛ جایگاه‌ها از بالا (محدودترین)
/// پر می‌شوند و هر جایگاه i ردیفی با کلید ≥ i می‌خواهد.
/// `ascending=true` (مثلثی پایین): کلید = آخرین غیرصفر؛ جایگاه‌ها از پایین پر می‌شوند و
/// هر جایگاه i ردیفی با کلید ≤ i می‌خواهد.
fn greedy_assign(keys: &[usize], m: usize, ascending: bool) -> Option<Vec<usize>> {
    let n = keys.len();
    let mut used = vec![false; n];
    let mut perm = vec![0usize; m];

    let positions: Vec<usize> = if ascending {
        (0..m).collect()
    } else {
        (0..m).rev().collect()
    };

    for pos in positions {
        // از ردیف‌های باقیمانده، «کم‌انعطاف‌ترین» ردیفِ واجد شرط را بردار تا انعطاف حفظ شود.
        let mut pick: Option<usize> = None;
        for r in 0..n {
            if used[r] {
                continue;
            }
            let ok = if ascending {
                keys[r] <= pos
            } else {
                keys[r] >= pos
            };
            if ok
                && pick.is_none_or(|best| {
                    if ascending {
                        keys[r] >= keys[best]
                    } else {
                        keys[r] <= keys[best]
                    }
                })
            {
                pick = Some(r);
            }
        }
        let r = pick?;
        used[r] = true;
        perm[pos] = r;
    }
    Some(perm)
}
