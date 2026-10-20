# COMPATIBILITY.md — ماتریس سازگاری با MATLAB (سند مرجع: R2026b)

> **نسخه مرجع:** MATLAB R2026b (هسته زبان از ~R2008 پایدار است).
> **اوراکل‌ها:** (۱) تست‌های طلایی Rust با مقادیر مرجع اوراکل numpy/LAPACK — `test-corpus/expected/`،
> (۲) تطبیق افتراقی Octave در CI (job ‏`octave-differential`، سند ۰۸ §2)، (۳) مثال‌های مستندات MathWorks.
> **برچسب‌ها:** `exact` = مطابق بیتی/تحلیلی · `tol` = در حد tol 1e-9 (سیاست سند ۰۸ §8) ·
> `deviation` = انحراف مستندشده · `n/a` = هنوز پیاده نشده (پیش‌بینی فاز در سند ۰۳ §12).

## ۱. هسته زبان (L1 — سند ۰۳)

| ویژگی | وضعیت | یادداشت |
|---|---|---|
| اعداد، نماد علمی، اولویت عملگرها | exact | توان چپ‌گرا: `2^3^2=64` |
| **قانون ابهام علامت در براکت** `[1 -2]` / `[1 - 2]` | exact | تست: corpus 21-30 + پارسر |
| اندیس ۱-پایه، `end`، رشد خودکار | exact | corpus 18/19 |
| سمانتیک colon (`0:0.1:1` = دقیقاً ۱۱ عنصر) | exact | corpus 16/41 — الگوریتم مرجع سند ۰۳ §7 |
| for/while/if/break/continue، توابع بی‌نام با capture | exact | corpus 11-15 |
| `ans` و سرکوب با `;` | exact | corpus 17 |
| **توابع فایلی `function y = f(x)…end`** | exact* | ورک‌اسپس جدا، بازگشتی، nargin/nargout، `return` — ۴۱ تست (function_files.rs). *انحراف‌ها: §۵ |
| **انتساب چندخروجی `[a,b] = f(...)`** | exact | به ترتیب خروجی‌های تعریف؛ چندخروجی builtin هنوز n/a (P2) |
| **اسکریپت‌ها و فایل‌های .m (SAF)** | exact* | اجرا در فضای کاری فراخوان، `return` اسکریپت را قطع می‌کند؛ ثبت با `addpath` / editor Run |
| **هندل نام‌دار `@f`** | exact* | تابع کاربر و builtin؛ اسکریپت هندل‌پذیر نیست (انحراف مستند) |
| ستون‌محوری (column-major): اندیس خطی، reshape، find | exact | corpus 35/44 |
| قانون اولین بعدِ غیرتک‌عضوی (sort/sum/cumsum/diff/...) | exact | corpus 31-37 |
| NaN: آخرِ sort، نادیده در max/min، انتشار در sum | exact | corpus 38 |
| اشباع صحیح (uint8) | n/a | P2 — نوع‌های صحیح هنوز نیستند |
| اندیس‌گذاری منطقی `A(A>0)` | n/a | P2 |
| چندخروجی builtin مثل `[V,D]=eig` | n/a | P2 — multi-assign تابع کاربر exact است |
| مختلط (`3i`, eig نامتقارن) | n/a | P2 — خطای صادقانه فعلی |

## ۲. ثابت‌ها

| نام | وضعیت |
|---|---|
| `pi`، `eps`، `Inf/inf`، `NaN/nan` | exact (سایه‌پذیر توسط متغیر کاربر) |
| `i`، `j`، `realmax/realmin`، `intmax` | n/a (P2) |

## ۳. رجیستری توابع (۹۱ builtin فعلی)

**exact / tol (تست‌شده در کورپوس):** zeros ones eye diag tril triu size length numel isempty isvector isscalar repmat reshape linspace logspace · sum prod max min mean cumsum cumprod diff all any find sort('descend') · mod rem round fix floor ceil sign abs sqrt exp log log2 log10 sin cos tan asin acos atan atan2 sinh cosh tanh real imag conj angle · dot cross kron · inv det rank trace rref norm(1/2/inf/fro) cond chol qr svd eig(متقارن) · true false isnan isinf isfinite isequal isreal

**انحراف/محدودیت مستندشده:**
| مورد | توضیح |
|---|---|
| `eig` نامتقارن | خطای صادقانه «با P2 مختلط می‌آید» — مقدار غلط نمی‌دهد |
| `qr` تک‌خروجی | فقط R (مثل متلب)؛ `[Q,R]` با multi-assign |
| `svd` تک‌خروجی | فقط بردار مقادیر تکین |
| `max/min` ستونِ همه‌NaN | +Inf/−Inf به‌جای NaN (لبه؛ در اصلاح) |
| `sort` رشته کاراکتری | خطای صادقانه (P2) |

**n/a (P2+ طبق سند ۰۴):** fzero roots integral ode45 interp1 polyfit conv/deconv expm pinv null orth magic FFT آمار پیشرفته RNG

## ۴. زنجیره صحت‌سنجی

```
موتور Rust ←(182 تست workspace)→ ثابت‌های مرجع ←(gen_corpus.py)→ numpy/LAPACK
                                        ↓
              CI: octave-differential job → مقایسه Octave واقعی با expected/ (tol 1e-9)
```
هر باگ سازگاری یافته‌شده = تست رگرسیون دائمی در کورپوس (قاعده R2 سند ۰۷).
تاریخ آخرین بازبینی: 2026-10-20 · ۶۰/۶۰ اسکریپت کورپوس P1 سبز در موتور.

## ۵. توابع فایلی و اسکریپت‌ها — انحراف‌های مستند (P1)

| مورد | MATLAB | MathCraft P1 | برچسب |
|---|---|---|---|
| پایان تابع | تابع بدون `end` مجاز (فایل تک‌تابعی) | `end` الزامی — خطای آموزشی | deviation |
| توابع محلی (local functions) | خصوصی به فایل خودشان | سراسری و قابل فراخوانی (نام یکتا در نشست؛ تعریف مجدد = جایگزینی) | deviation |
| خروجی بدنه بدون `;` | در کنسول چاپ می‌شود | چاپ نمی‌شود (خروجی گزاره‌های تابع با streaming ‏`runScript` در P2) | deviation |
| `global` / `persistent` | پشتیبانی | n/a — P2+ | n/a |
| `varargin` / `varargout` | پشتیبانی | n/a — P2+ (الگوی `if nargin < 2` با پارامتر فراهم‌نشده exact است) | n/a |
| توابع تو در تو (nested) | با اشتراک ورک‌اسپس | خطای صادقانه «در سطح فایل تعریف کنید» | deviation |
| `@f` برای اسکریپت | خطای متلب | خطای آموزشی (مطابق) | exact |
| چندخروجی builtin | `[V,D] = eig(A)` | خطای صادقانه — P2 | n/a |

مکانیزم ثبت: `register_m_source` (قرارداد mc-ffi) — نام فایل = نام فراخوانی‌پذیر
(مغایرت نام تابع اصلی = خطای آموزشی مثل متلب)؛ توابع فوراً فراخوانی‌پذیر می‌شوند
و سایه بر builtin هم‌نام می‌اندازند (سمانتیک متلب §9).
