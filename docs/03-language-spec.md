# سند ۰۳ — مشخصات زبان اسکریپت (Language Specification)

> **MathCraft Script v1** | سازگار با MATLAB در زیرمجموعه تعریف‌شده (تصمیم D-04)
> این سند «قرارداد سینتکسی» موتور است: هر چیزی که پارسر و مفسر باید بپذیرند یا رد کنند، اینجا مکتوب است. مرجع ثانویه برای سؤالات حل‌نشده: مستندات رسمی MATLAB R2024b و رفتار GNU Octave 9 — در تضاد، **رفتار MATLAB** ملاک است.

---

## ۱. اهداف و سیاست سازگاری

1. **هدف:** هر اسکریپت `.m` متعلق به ترم ۱-۲ مهندسی، بدون تغییر و با خروجی عددی منطبق (تحمل 1e-9 نسبی) اجرا شود.
2. **مرز سازگاری صادقانه:** ما «زیرمجموعه دقیق» را تضمین می‌کنیم، نه «سازگاری ادعایی با استثنا». هر عبارت خارج از زیرمجموعه، خطای صریح «پشتیبانی نمی‌شود: X» می‌گیرد — نه رفتار غلط بی‌صدا.
3. **تست‌محور:** سازگاری یعنی عبور از کورپوس ۲۰۰ اسکریپت + Golden Tests با Octave (سند ۰۸). هیچ فیچر سینتکسی بدون حداقل ۳ تست رفتاری merge نمی‌شود.

---

## ۲. ساختار واژگانی (Lexical Structure)

| عنصر | قاعده | مثال |
|---|---|---|
| شناسه | `[A-Za-z]\w*`، حداکثر 63 کاراکتر معنادار (مانند MATLAB)، حساس به حروف بزرگ/کوچک | `A`، `x1`، `myFun` |
| عدد صحیح/اعشاری | اعشاری با `.`، نماد علمی `e/E`، پسوند `i/j` برای موهومی، `_` جداکننده مجاز نیست | `3`، `2.5`، `1e-6`، `3+4i` |
| رشته کاراکتری | تک‌کوتیشن `'...'` با escape با `''` | `'سلام'`، `'don''t'` |
| رشته String | دابل‌کوتیشن `"..."` (رفتار R2017a+) | `"hello"` |
| کامنت | `%` تا انتهای خط؛ `%%` سکشن؛ `%{ ... %}` بلوکی | `% توضیح` |
| ادامه خط | `...` (سه نقطه) | `y = x + ...` |
| جداکننده دستور | `,` (نمایش خروجی) یا `;` (حذف خروجی) یا خط جدید | — |
| ترانهاده | `'` (ctranspose مختلط) و `.'` (ترانهاده ساده) | `A'` |

**کلمات کلیدی (رزرو):** `if elseif else end for while switch case otherwise try catch break continue return function global persistent parfor(spawn error) classdef(spawn error)` — دو مورد آخر عمداً رزرو هستند تا پیام «در نسخه‌های بعد» بدهند نه خطای سینتکس گیج‌کننده.

**حساسیت به Unicode در شناسه‌ها:** ترم ۱ نسخه ۱ فقط ASCII؛ پشتیبانی شناسه فارسی به‌عنوان extension آینده (در پارسر، tokenizer برای این توسعه‌پذیر نوشته می‌شود اما فعال نیست).

---

## ۳. انواع داده (Types)

| نوع | پیاده‌سازی داخلی | محدوده نسخه ۱ |
|---|---|---|
| `double` | ماتریس column-major از f64 (سند ۰۲ بخش ۵) | کامل — نوع اصلی زبان |
| complex | interleaved re/im | کامل (پسوند i/j، عملیات مختلط) |
| `logical` | آرایه بایت | کامل (اندیس‌گذاری منطقی، `true/false`) |
| `char` | آرایه سطر (UTF-8 با شمارش codepoint) | کامل |
| `string` | اسکالر/string array سطح ۱ | توابع اصلی (`strlength, strcat, contains`…) |
| `cell` | آرایه سلولی | پایه: سازنده `{}`، اندیس‌گذاری `{}` و `()`، `cellfun` ساده |
| `struct` | فیلدهای نام‌دار | پایه: سازنده، دسترسی `s.f`، `fieldnames` |
| function handle | closure | کامل برای anonymous؛ `@fname` |
| symbolic | mc-cas Sym | کامل (بخش CAS سند ۰۵) |
| sparse | — | **خارج از نسخه ۱** (خطای صریح) |
| int8…int64، uint | — | خواندن/تبدیل پایه (`uint8` برای تصویر — خارج از نسخه ۱) |

**قاعده نسخه ۱:** `double` شهروند درجه‌یک است؛ بقیه انواع «به اندازه نیاز ترم ۱-۲». این تصمیم عمدی است تا عمق به‌جای عرض اولویت بگیرد.

---

## ۴. ساخت ماتریس و آرایه

```matlab
a = [1 2 3];            % سطری (فاصله یا ویرگول)
b = [1; 2; 3];          % ستونی با ;
M = [1 2 3; 4 5 6];     % 2×3
c = [a 4];              % اتصال افقی
d = [b; 10];            % اتصال عمودی
E = [];                 % ته (0×0) — سمانتیک MATLAB
r = 1:5                 % [1 2 3 4 5]
s = 0:0.1:pi            % با گام اعشاری (بخش ۷)
t = linspace(0,1,11)    % ۱۱ نقطه شامل دو سر
```

- سازنده‌ها با اندازه اسکالر/برداری: `zeros(n), zeros(m,n), ones(...), eye(n), rand(m,n), randn(m,n)`.
- انتساب رشد خودکار: `x(5) = 3` روی متغیر موجود، بقیه را با 0 پر می‌کند (رفتار MATLAB؛ هشدار سبک عملکردی بعد از ۱۰۰ رشد متوالی).
- حذف: `A(2,:) = [];` — سمانتیک دقیق MATLAB.

---

## ۵. اپراتورها و اولویت

| اولویت (بالا→پایین) | اپراتورها | یادداشت |
|---|---|---|
| ۱ | `()` `[]` `{}` `.` `'` `.'` `^` `.^` | توان **چپ‌گرا** (رفتار MATLAB): `2^3^2 = (2^3)^2 = 64` |
| ۲ | unary `+ - ~` | |
| ۳ | `* / \ .* ./ .\` | `\` = حل از چپ (بخش ۳ سند ۰۴) |
| ۴ | `+ -` | |
| ۵ | `:` | `a:b`، `a:s:b` — اولویت پایین‌تر از جمع (رفتار MATLAB: `1:n-1` یعنی `1:(n-1)`) |
| ۶ | `< <= > >= == ~=` | |
| ۷ | `&` | |
| ۸ | `\|` | |
| ۹ | `&& \|\|` | short-circuit اسکالر — در غیر اسکالر خطای نوع صریح |
| ۱۰ | `=` (انتساب) | راست‌گرا؛ مقدار برنمی‌گرداند |

- **element-wise vs matrix:** `.* ./ .^` همیشه عضوی؛ `* / \ ^` ماتریسی (بلینی برای `*`، dispatch برای `\`).
- **Implicit expansion (بسط ضمنی):** مثل MATLAB R2016b+ — `(1×n) + (m×1) = m×n`؛ `A - mean(A)` و مشابه‌ها کار می‌کنند. قاعده: هر بعد یا مساوی یا ۱؛ در غیر این صورت خطای `MC:E2003_SIZE_MISMATCH` با چاپ دو شکل.

---

## ۶. اندیس‌گذاری (Indexing)

```matlab
A(3)            % linear (column-major!)
A(2,3)          % subscripts
A(end, :)       % end = اندازه همان بُعد
A(1:2, [1 3])   % بردار اندیس
A(A > 0)        % اندیس منطقی (عمودیسازی ستونی)
A([true false]) % طول کوتاه‌تر = کوتاه‌شده (رفتار MATLAB)
v(:)            % ستونی‌سازی
x(a>2) = 0      % انتساب منطقی
```

- اندیس خارج از بازه در **خواندن** = خطا؛ در **انتساب** = رشد با 0 (فقط linear/اسکالر؛ بقیه خطا).
- اندیس 0 یا اعشاری غیرصحیح = خطای صریح با مقدار گزارش‌شده.
- `end` فقط داخل `()`/`{}` معنا دارد.

---

## ۷. سمانتیک اپراتور کولن (دقیق‌ترین بخش زبان!)

رفتار MATLAB برای `a:s:b`: عناصر `a, a+s, a+2s, …` تا جایی که `≤ b` (اگر s>0). تعداد عناصر: `n = floor((b-a)/s + eps_correction) + 1`. در پیاده‌سازی، از الگوریتم مرجع (محاسبه n، سپس `a + (0:n-1)*s` با یک اصلاح پایانی روی عنصر آخر برای انطباق با `b`) استفاده می‌شود — **هرگز** جمع تکراری `x = x + s`، چون خطای انباشته float نتایج را از MATLAB منحرف می‌کند. این مورد، یک Golden Test اجباری دارد (`0:0.1:1` باید دقیقاً ۱۱ عنصر و `0.30000000000000004`-مانندِ مطابق مرجع باشد).

---

## ۸. کنترل جریان

```matlab
if x > 0, y = 1; elseif x < 0, y = -1; else y = 0; end
for k = 1:n          % پیمایش روی ستون‌های ماتریس هم می‌شود
    s = s + k;
end
while abs(dx) > tol
    ...
end
switch op            % اسکالر/رشته؛ case چند مقدار {a,b}
    case '+', r = a+b
    otherwise, error('MC:BAD_OP', 'عملگر نامعتبر')
end
try
    risky();
catch me           % me.identifier, me.message, me.line
    warning(me.message)
end
```

- شرط `if/while`: فقط اسکالر منطقی/عددی (غیر صفر = true)؛ ماتریس = خطای صریح با پیام آموزشی («شرط باید اسکالر باشد؛ اگر قصد all/any دارید…»).
- `for` روی ماتریس: هر تکرار یک **ستون** — سمانتیک MATLAB که تکالیف به آن متکی‌اند.
- `break/continue` فقط در حلقه جاری؛ `return` از تابع.

---

## ۹. توابع و اسکریپت‌ها

### اسکریپت
فایل بدون `function` در ابتدا؛ در workspace فراخواننده اجرا می‌شود (مثل MATLAB base/local). `%%` بخش‌ها با اجرای سلولی (F-1.7).

### فایل تابع
```matlab
function [out1, out2] = myFun(in1, in2)
%MYFUN خلاصه — همین خط help تک‌خطی است
%  توضیح بیشتر برای help کامل
    out1 = in1 + in2;
    out2 = in1 * in2;
end
```
- چند تابع در یک فایل: فقط اولی از بیرون قابل فراخوانی (بقیه local).
- `nargin/nargout`، فراخوانی با خروجی کمتر مجاز است؛ خواستن خروجی بیشتر = خطا.
- `varargin/varargout` پشتیبانی می‌شود (برای توابعی مثل `plot` داخلی اجباری است).
- **Path و resolution:** workspace فعلی + پوشه کاربر؛ `which fn` محل را نشان می‌دهد. سایه‌شدن builtin توسط فایل کاربر مجاز است (رفتار MATLAB) با warning یک‌بار.
- اولویت resolution (مانند MATLAB): variable > local func در فایل > فایل کاربر > builtin > پوشه‌ها.

### توابع بی‌نام
```matlab
f = @(x) x.^2 + 1;
g = @(a) @(x) a.*x;    % closure تو در تو — a captured
```
capture-by-value در لحظه ساخت (رفتار MATLAB). recursive anonymous ممنوع (خطای آموزشی با پیشنهاد تبدیل به تابع فایل).

---

## ۱۰. رشته‌ها و فرمت

- `char` با `'`؛ اتصال با `[]`؛ `strcat, strsplit, strtrim, strcmp, strncmp, strrep, lower, upper, length` (codepoint) — همه با یونیکد صحیح (تست فارسی اجباری: `length('سلام') == 4`).
- `sprintf/fprintf` با زیرمجموعه format spec: `%d %i %f %e %g %s %c %%` + width/precision (`.2f`)؛ `%d` روی double صحیح درست چاپ می‌شود.
- `num2str, str2double, mat2str, disp, input` (کنسول تعاملی با کیبورد ریاضی).
- string دابل‌کوتیشن: اسکالر + `string()` + متدهای پایه.

---

## ۱۱. قواعد سمانتیکی حیاتی (Cheatsheet پیاده‌ساز)

1. **Value semantics + COW:** انتساب `b = a` کپی نمی‌کند؛ اولین نوشتن روی یکی، کپی می‌کند (سند ۰۲ بخش ۵).
2. **1-based indexing، column-major** — همه‌جا، بدون استثنا.
3. **حذف خروجی با `;`** — فقط پیام‌های صریح (`disp/print/fprintf`) چاپ می‌شوند.
4. **نمایش عدد پیش‌فرض `format short`** (۵ رقم معنادار، `format long` = ۱۵)؛ `format` command پشتیبانی می‌شود: `short long shortG longG compact loose`.
5. **NaN، Inf، -0:** رفتار IEEE؛ `isnan/isinf/isfinite`؛ مرتب‌سازی NaN در انتها (رفتار MATLAB).
6. **مقایسه float برای حل دستگاه/مساوی:** سخت‌گیرانه (`==` بی‌رحم است) — این رفتار حفظ می‌شود چون آموزشی است؛ اما پیام خطای رایج «مقایسه اعشاری با ==» به‌صورت hint در did-you-mean آینده است (P2).
7. **متغیر global/persistent:** پشتیبانی کامل (برای الگوهای رایج کلاس).
8. **حساسیت وقت اجرا:** `clear x`، `who/whos`، `exists`، `isvarname`.

---

## ۱۲. رجیستری توابع Built-in (مرجع پیاده‌سازی)

> این فهرست، scope توابع نسخه ۱.۰ است. امضاها مطابق MATLAB. علامت † = پیاده‌سازی در CAS (سند ۰۵).

### ریشه و سازه‌های ماتریسی
`zeros ones eye rand randn randi linspace logspace diag tril triu meshgrid repmat cat horzcat vertcat ndgrid fliplr flipud rot90 reshape size length numel isempty isvector ismatrix isscalar`

### عملیات ماتریسی و جبر خطی
`+ - * / \ ^ .* ./ .\ .^ ' .'` | `inv det rank trace rref norm(norm1,2,inf,fro) cond null orth pinv` | `lu qr chol svd eig` | `cross dot kron` | `expm` (P2) | `isSymmetric isHermitian` (extension تشخیصی برای dispatch)

### توابع عنصری (elementwise)
`abs sqrt exp log log2 log10 sin cos tan asin acos atan atan2 sinh cosh tanh asinh acosh atanh` | `round floor ceil fix mod rem sign` | `real imag conj angle complex` | `min max sum prod cumsum cumprod diff gradient` | `sort issorted unique intersect union setdiff ismember`

### چندجمله‌ای
`polyval roots poly conv deconv polyder† polyint` `polyfit polyvalm(P2) residue(P2)`

### روش‌های عددی
`fzero integral quadl(legacy alias) ode45 ode23 interp1 spline pchip interp2(P2) gradient del2 tic toc`

### آمار و تصادف
`mean median mode var std cov corrcoef prctile histcounts histogram rng randperm` | `normpdf normcdf norminv tpdf tcdf chi2pdf chi2cdf binopdf binocdf`

### رسم
`figure subplot plot scatter bar barh histogram area fill polarplot errorbar fplot fplot3 plot3 surf mesh contour contourf quiver colorbar title xlabel ylabel zlabel legend grid axis hold clf close gca(محدود) set(محدود به properties رسم) shading view colormap`

### I/O و محیط
`disp fprintf sprintf input num2str str2double mat2str save load csvread csvwrite readmatrix writematrix who whos clear clc close doc help which exists what dir cd fopen/fscanf/fprintf(فایل — P2)`

### سمبلیک †
`syms sym symfun` | `simplify expand factor collect numden pretty` | `diff int limit taylor series` | `solve vpasolve dsolve` | `subs vpa double digits` | `assume` (زیرمجموعه: real/positive) | `latex`

### کنترل جریان و خطا
`error warning try/catch assert input` + error IDs با قالب `MC:*`

### توابع منطقی/مقایسه‌ای
`all any find isnan isinf isfinite isreal ischar isstring iscell isstruct islogical isequal isempty` | `and or not xor true false`

**قاعده شمارش:** مجموع ≈ ۲۳۰ تابع. هر تابع: امضا + صفحات doc دوزبانه + حداقل ۲ تست رفتاری (سند ۰۸). هر تابعِ خارج از این فهرست که کد کورپوس بخواهد → به رجیستری اضافه + doc + تست، **قبل از** ادامه.

---

## ۱۳. افزونه‌های MathCraft (Extensions — غیرشکستن‌دهنده)

افزونه‌ها فقط «اضافه»اند؛ هیچ رفتار MATLAB را بازتعریف نمی‌کنند:

1. `doc fn` → باز کردن مستندات درون‌اپ؛ `examples fn` → لیست مثال‌های اجراشدنی.
2. `fxplot(f, [a b])` → رسم سریع تابع handle با نمونه‌برداری تطبیقی (درهای ورود به رسم را کوتاه می‌کند).
3. `steps(A\b)` → نمایش گام‌به‌گام حل دستگاه (حالت آموزشی — برجسته‌ترین تفاوت ما با MATLAB؛ P1).
4. پیام‌های خطا با ID ساختاریافته + پیشنهاد + دکمه «مستند مرتبط» (سند ۰۲ بخش ۹).
5. `format fa` → نمایش اعداد با ارقام فارسی در کنسول (فقط نمایش؛ P2).

---

## ۱۴. کاتالوگ خطاها (نمونه‌های مرجع)

| ID | وضعیت | پیام EN (نمونه) | پیام FA (نمونه) |
|---|---|---|---|
| MC:E1001 | undefined variable | Undefined variable 'x'. Did you mean 'y'? | متغیر 'x' تعریف نشده. منظورتان 'y' بود؟ |
| MC:E1002 | undefined function | Function 'lenght' not found. Did you mean 'length'? | تابع 'lenght' یافت نشد. منظورتان 'length' بود؟ |
| MC:E2001 | index out of bounds | Index exceeds array bounds: A is 3×3, tried A(4,2). | اندیس خارج از محدوده: A سه‌درسه است ولی A(4,2) خواسته شد. |
| MC:E2003 | size mismatch | Inner matrix dimensions must agree: (2×3)*(4×3). | ابعاد داخلی ماتریس‌ها نمی‌خواند: (2×3)*(4×3). |
| MC:E3001 | parse error | Unexpected ')': expected expression. | ')' غیرمنتظره: عبارت انتظار می‌رفت. |
| MC:E4001 | ode integration | ODE solver failed to meet tolerance (RelTol=1e-3) at t=…; result flagged. | حلگر به تحمل خطا نرسید؛ نتیجه با برچسب «مشکوک» برگردانده شد. |

قاعده: هر خطا حتماً یکی از: (الف) راه اصلاح، (ب) لینک مستند، (ج) توضیح مفهومی کوتاه داشته باشد. خطای خالی ممنوع.

---

## ۱۵. آنچه عمداً پشتیبانی نمی‌شود (با پیام صریح)

`classdef` و OOP، `parfor` (اجرای موازی)، `sparse`، handle classes، `try` تو در توی عمیق با cleanup سمانتیک‌های خاص، `eval` و `evalin` (**ممنوع دائمی** — امنیت و پارسر پاک)، `inline` (منسوخ)، MEX، `.mlx`. هر کدام خطای `MC:E9xxx_UNSUPPORTED` با توضیح جایگزین می‌دهند. `eval` با این پیام: «eval در MathCraft وجود ندارد؛ از توابع و متغیرها استفاده کنید» — این تصمیم کیفیت پیام خطا و امنیت را تا ابد تضمین می‌کند.
