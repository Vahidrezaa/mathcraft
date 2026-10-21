# test-corpus — کورپوس رگرسیون سازگاری (سند ۰۸ بخش ۲)

کورپوس ۲۰۰ اسکریپتی: اسکریپت‌های `.m` واقعی ترم ۱-۲ + خروجی مرجع Octave.

- توزیع هدفمند: سینتکس/پارسر ۵۰، جبر خطی ۴۵، روش‌های عددی ۳۵، آمار ۲۵،
  چندجمله‌ای/مختلط ۲۰، رسم (دامنه داده PlotSpec) ۱۵، مرزی ۱۰ — سند ۰۸ §2.3
- تولید expected در CI با `octave --no-gui --eval` به JSON canonical
- قاعده ضد تقلب: expected هرگز از خود MathCraft تولید نمی‌شود
- هارنس از P0 (این ریپو) شروع می‌شود؛ ۶۰ اسکریپت اول در P1 (سند ۰۷)

ساختار (فعال از جلسه 2026-10-20 — ۶۰ اسکریپت P1):

```
scripts/           p1_01..p1_60.m — منبع یگانه اسکریپت‌ها (قطعی، tol 1e-9)
expected/          خروجی مرجع JSON (اوراکل numpy/LAPACK — scripts/gen_corpus.py)
manifest.json      متادیتا + متغیرهای capture + tolerance per case
octave_regen.m     اجرای کل کورپوس با Octave ≥ 7 → generated/
generated/         خروجی موقت Octave (گیت‌شده) — با expected مقایسه می‌شود
```

اجرای محلی: `python3 scripts/gen_corpus.py` · تست Rust: `cargo test -p mc-interp --test corpus_p1_golden`
CI: job ‏`octave-differential` = بازتولید با Octave واقعی + `scripts/compare_expected.py` (tol 1e-9).
تا ۶۰/۲۰۰ — باقی در P2+ (سند ۰۷).
