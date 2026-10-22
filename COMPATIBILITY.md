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
| اندیس ۱-پایه، `end`، رشد خودکار | exact | corpus 18/19؛ **جهت رشد خطی متلب‌گونه شد (۲۰۲۶-۱۰-۲۲):** `x=[]; x(3)=5` → 1×3 سطری (مرجع Octave) — انتظار قبلی کورپوس (ستونی، numpy-دستی) اصلاح شد |
| **حذف با انتساب خالی `x(i)=[]`** | exact | ✅ ممیزی خارجی ۱: المان بردار (حفظ جهت)، حذف «سطر» با اندیس خطی روی ماتریس، `A(i,:)`/`A(:,j)` با دونقطه؛ `A(i,j)=[]` بدون دونقطه = خطای صادقانه — ۱۰ تست رگرسیون |
| **شرط if/while با آرایه/خالی** | exact | ✅ ممیزی خارجی ۷: سمانتیک متلب «غیرخالی ∧ همهٔ عناصر ناصفر» — `if []` نادرست، `if [2 3]` درست، NaN ناصفر؛ `&&`/`||` همچنان فقط اسکالر (رفتار متلب) |
| **نمایش انتساب اندیس‌دار بدون `;`** | exact | ✅ ممیزی خارجی ۴: `a(2)=5` هم مثل `a=5` نمایش داده می‌شود |
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
| اشباع صحیح (uint8) | n/a | P2-b — نوع‌های صحیح هنوز نیستند |
| اندیس‌گذاری منطقی `A(A>0)` | n/a | P2-b |
| چندخروجی builtin مثل `[V,D]=eig` | exact | ✅ P2: [V,D]=eig، [T,Y]=ode45/ode23، [q,r]=deconv — مکانیزم lookup_multi |
| مختلط (`3i`, eig نامتقارن) | exact/tol | ✅ P2: literal موهومی، عنصری/ماتریسی، `'` مزدوج، ==/~، بک‌اسلش بلوکی، roots، [V,D]=eig — real/imag/abs/angle همیشه REAL (رفتار متلب) |
| **چیدمان نمایش خروجی کنسول (format short)** | exact* | ✅ FB-1 در P2: «name =» سطر بعد بدنه، ستون‌ها راست‌چین، اکوی نام متغیر، سطر خالی بین خروجی‌ها — کالیبره روی خروجی واقعی متلب کاربر (۵×۵، 2026-10-21)؛ *باز (P2-b): قاعدهٔ رقمِ اعشاری غیرصحیح هنوز کوتاه‌ترین‌گرد است نه «۴ رقم اعشار» (`0.1` به‌جای `0.1000`) + ضریب مقیاس `1.0e+03 *` پیاده نشده — ممیزی خارجی ۱۱ |

## ۲. ثابت‌ها

| نام | وضعیت |
|---|---|
| `pi`، `eps`، `Inf/inf`، `NaN/nan` | exact (سایه‌پذیر توسط متغیر کاربر) |
| `i`، `j` | exact | ✅ P2 — سایه‌پذیر مثل pi؛ اعداد موهومی `4i`/`2.5j` |
| `realmax/realmin`، `intmax` | n/a | P2-b |

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

## ۴.۸ ممیزی خارجی P1-v0.6 (گزارش AI شخص ثالث، ۲۰۲۶-۱۰-۲۲) — دفتر ثبت

گزارش ۱۵ بندی روی P1-v0.6 تأیید شد (همه قابل بازتولید)؛ وضعیت در P2-v0.1:

| # | مورد | وضعیت در P2 |
|---|---|---|
| ۱ | پنیک `x(2)=[]` | ✅ فیکس شد (§۱ همین سند) + ۱۰ تست |
| ۲ | abort حافظه (zeros 1e10 / 1:1e9) | 🔶 P2-b: سقف حافظهٔ موتور با خطای MATLAB-گونه (abort تخصیص با catch_unwind قابل مهار نیست) |
| ۳ | اکوی متغیر بدون `ans` | ✅ در P2 (FB-1) |
| ۴ | نمایش انتساب اندیس‌دار | ✅ فیکس شد |
| ۵ | جهت نتیجهٔ اندیس‌گذاری بردار (جهت اندیس) | 🔶 P2-b |
| ۶/۱۲ | نوع char/رشته (`[65 'a']`, `'abc'+1`) | 🔶 P3 — بازطراحی مدل داده |
| ۷ | شرط خالی/آرایه‌ای | ✅ فیکس شد |
| ۸ | disp/fprintf/class | 🔶 P2-b (disp/class ارزان؛ fprintf متوسط) |
| ۹ | hold on → P3-a (نمودار طبق نقشه)؛ کامنت بلوکی `%{ %}` | 🔶 P2-b (لکسر) |
| ۱۰ | شکل آرایهٔ خالی (`1:0` → 1×0 + نمایش «1×0 empty…») | 🔶 P2-b |
| ۱۱ | قاعدهٔ رقم format short + ضریب مقیاس | 🔶 P2-b (§۱ همین سند) |
| ۱۳ | اندیس‌گذاری زنجیره‌ای روی نتیجهٔ عبارت | 🔶 P3 (پارسر) |
| ۱۴ | تاکسونومی خطا/موقعیت (E3001 برای خطای معنایی، line 0:0) | 🔶 P3 (کارگاه دیاگنوستیک) |
| ۱۵ | panic از مرز FFI | ✅ فیکس شد: catch_unwind در evaluate_line → خطای موتور؛ نکتهٔ گزارش دربارهٔ `panic="abort"` در release نادرست بود (تنظیم نشده) |

## ۴. زنجیره صحت‌سنجی

```
موتور Rust ←(182 تست workspace)→ ثابت‌های مرجع ←(gen_corpus.py)→ numpy/LAPACK
                                        ↓
              CI: octave-differential job → مقایسه Octave واقعی با expected/ (tol 1e-9)
```
هر باگ سازگاری یافته‌شده = تست رگرسیون دائمی در کورپوس (قاعده R2 سند ۰۷).
تاریخ آخرین بازبینی: 2026-10-21 · ۶۰/۶۰ کورپوس P1 + ۱۳ کورپوس P2 سبز · تأیید عملی ۵×۵ روی دستگاه کاربر (APK از CI) · P2 هستهٔ عددی کامل

## ۴.۵ — توابع عددی/آمار P2 (سند ۰۴ §4-7)

**exact / tol (کورپوس p2_* با مرجع Octave 9.4 / تحلیلی):** fzero (Brent + براکت خودکار) · roots (ماتریس همبند، مختلط) · polyfit (Householder QR — هرگز normal equations) · polyval (Horner) · conv · deconv ([q,r]) · interp1 (linear/nearest/pchip/spline not-a-knot؛ برون‌یابی پیش‌فرض برای pchip/spline مثل متلب) · median · std/var (پرچم w + omitnan) · cov/corrcoef (n−1) · integral (G7/K15 تطبیقی + بازه‌های بی‌نهایت با تبدیل متغیر) · ode45 (DP54 + FSAL + Hermite dense) · ode23 (BS32) · [T,Y] شکل ستونی · rng(seed)/rand/randn/randi/randperm · complex-aware: real/imag/conj/abs/angle/isreal/complex

**انحراف/محدودیت مستندشدهٔ P2:**
| مورد | توضیح |
|---|---|
| `rng(seed)` خروجی | متلب struct وضعیت می‌دهد؛ MathCraft مقدار seed (ساختار در P2-b با Value::Struct) |
| دنبالهٔ RNG | عمداً مستقل از Mersenne Twister متلب (سند ۰۴ §7) — PCG64+Ziggurat؛ نتایج آماری معادل؛ `rng(42)` تکرارپذیر |
| گزینه‌های ode | به‌جای `odeset` جفت نام/مقدار مستقیم: `ode45(f, tspan, y0, 'RelTol', 1e-6)` — odeset ساختاریافته = P2-b |
| dense output ode | Hermite مکعبی بین گام‌های پذیرفته‌شده (درون‌یابی درجه-۴ خود DP = P2-b) — دقت در حد تلورانس گام‌ها |
| integral گزینه‌ها | RelTol/AbsTol/Waypoints نامی = P2-b؛ فعلاً پیش‌فرض متلب (1e-10) |
| vdp1000 مرجع نقطه‌ای | ساختاری (چرخهٔ حدی/فاز آهسته) سبز؛ مرجع نقطه‌ای Octave کند است — متلب کاربر = اوراکل نهایی |
| `qr/svd` چندخروجی | هنوز تک‌خروجی (P1) — [Q,R]/[U,S,V] در P2-b با lookup_multi |

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
