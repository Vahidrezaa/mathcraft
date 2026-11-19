# فیکسچرهای .mat Level 5 — پروونانس (اصل صداقت سند ۰۰)

تولید: `scripts/gen_mat_fixtures.py` در ریشهٔ مخزن زندهٔ کار (sandbox) با
**scipy 1.14.1 `scipy.io.savemat`** ساخته شده‌اند؛ همان اسپک Level 5
(version 0x0100، endian IM) که سند رسمی MAT-File Format و Octave می‌نویسند.

- `*_comp`/پیش‌فرض فشرده: عنصر miCOMPRESSED (جریان zlib) — مسیر flate2.
- `fake_v73`/`big_endian`/بقیهٔ ردشونده‌ها دست‌سازند و نباید بازتولید شوند.

**اوراکل CI (جاب mat-parity):** همان محتوا با Octave بازتولید می‌شود و
تست‌های `tests/matv5_fixtures.rs` روی هر دو ست اجرا می‌شوند — برابری
**ارزشی** (مقادیر ماتریس/char/struct)، نه برابری بایت نویسنده‌ها (هدر متنی
نویسنده‌ها باریک‌پایه فرق دارد؛ اسپس نه).

بازتولید: `python3 scripts/gen_mat_fixtures.py` از ریشهٔ mc-io (نیازمند
numpy/scipy) — فایل‌ها باید bitwise پایدار بمانند؛ اگر scipy در آینده
چیزی عوض کرد، بازتولید عمدی + کامیت دلیل‌دار.
