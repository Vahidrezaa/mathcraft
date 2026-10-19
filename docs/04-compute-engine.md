# سند ۰۴ — طراحی موتور محاسبات عددی (Numeric Compute Engine)

> **MathCraft** | کرِیت‌های `mc-core` و `mc-num` و `mc-plot` | نسخه 1.0
> اصل حاکم: **صحت قبل از سرعت، سرعت قبل از وسواس میکرو.** هر الگوریتم این سند یک «مرجع درستی» دارد: Octave/MATLAB روی کورپوس تست.

---

## ۱. نمای ماژول‌ها

```
mc-num
├── linalg/     # facade روی faer: LU/QR/Chol/SVD/eig + dispatch بک‌اسلش
├── solvers/    # fzero(Brent)، roots، integral(GK تطبیقی)، ode45/ode23
├── interp/     # linear، pchip(Fritsch–Carlson)، cubic spline
├── poly/       # conv/deconv/polyval/roots(کامپانیون)/polyfit(QR)
├── stats/      # توصیفی، توزیع‌ها، corrcoef/cov، histogram binning
├── rng/        # PCG64 + Ziggurat نرمال + rng(seed) قابل تکرار
└── decim/      # LTTB + تیک‌های زیبا (سرویس mc-plot)
```

---

## ۲. ذخیره‌سازی ماتریس و درستی شکل‌ها (mc-core)

- Layout: **column-major** با `Vec<f64>`؛ `Matrix { rows, cols, data: Arc<ThinVec<f64>> }` با COW (`Arc::make_mut`).
- چرا column-major؟ سمانتیک MATLAB: `reshape(A,m,n)`، `A(:)`، `for k = A` و اندیس linear همگی بر این چیدمان تعریف شده‌اند. یک انحراف در چیدمان = انحراف بی‌صدا در ده‌ها تابع.
- **Property test اجباری:** هر عملیات شکل‌دار با شبیه‌سازی مرجع (روی آرایه‌های کوچک با بررسی عضوی) مقایسه می‌شود — بخش «property testing» سند ۰۸.
- امکان‌سنجی SIMD: faer خودش vectorize می‌کند؛ ما هیچ کد unsafe SIMD دستی نمی‌نویسیم (قاعده نگهداشت‌پذیری).

---

## ۳. جبر خطی (facade روی faer)

### 3.1 دکامپوزیسیون‌های مورد نیاز نسخه ۱
| تجزیه | کاربردها | یادداشت |
|---|---|---|
| LU با pivot جزئی | `det`, `\`, `inv`, `linsolve` | pivot نمایی + هشدار نزدیکی تکینگی (rcond) |
| Cholesky (LLᵀ) | ماتریس Hermitian مثبت معین در `\` | تست PSD سبک: چک diagonal dominance نیست — تلاش chol و شکست → LU |
| QR با pivoting ستونی | `\` مستطیلی (کمینه مربعات)، `rank`, `null`, `orth` | rank با tolerance روی R |
| SVD | `rank, pinv, null, orth, cond` | full SVD تا ۲۰۰۰×۲۰۰۰ بودجه‌پذیر؛ بالاتر: thin |
| Eigen (عمومی) | `eig` | مسیر دوگانه: متقارن/Hermitian → الگوریتم اختصاصی سریع؛ عمومی → Hessenberg+QR iteration با shift |

### 3.2 درخت dispatch اپراتور `\` (mldivide) — رفتار امضادار MATLAB
```
A\b:
├─ A اسکالر → b./A
├─ مثلثی (بالا/پایین، با چک) → solve مثلثی O(n²)
├─ سه‌قطری → الگوریتم Thomas O(n)
├─ مثلثیِ پرموته (permutation detected) → permute سپس مثلثی
├─ مربعی:
│   ├─ Hermitian:
│   │   ├─ مثبت معین (chol موفق) → chol solve
│   │   └─ در غیر این صورت → LDLᵀ
│   └─ عمومی → LU با pivot جزئی (+ هشدار rcond کوچک)
└─ مستطیلی:
    ├─ m > n (بلند) → QR با pivoting = کمینه مربعات + rank-deficient warning
    └─ m < n (پهن) → راه‌حل کم-نُرم (QR روی Aᵀ)
```
هر شاخه log dispatch داخلی دارد که `steps(A\b)` (افزونه آموزشی سند ۰۳ بخش ۱۳) از همان درخت استفاده می‌کند — **یک درخت، دو مصرف:** اجرا و آموزش.

### 3.3 بودجه عملکرد (گوشی مرجع سند ۰۱)
| عمل | اندازه | بودجه |
|---|---|---|
| LU solve | ۱۰۰۰×۱۰۰۰ | ≤ ۱۲۰ms |
| eig متقارن | ۵۰۰×۵۰۰ | ≤ ۲۵۰ms |
| SVD thin | ۵۰۰×۵۰۰ | ≤ ۶۰۰ms |
| `\` مثلثی | ۱۰۰۰×۱۰۰۰ | ≤ ۱۵ms |
فراتر از بودجه = بهینه‌سازی (parallel blocking via Rayon در faer) — نه تغییر الگوریتم.

---

## ۴. حلگر معادلات دیفرانسیل

### 4.1 `ode45` — Dormand–Prince 5(4) تطبیقی
- Butcher tableau کامل DP54 (۷ ارزیابی مرحله‌ای، FSAL).
- کنترل خطا: `err ≤ max(RelTol*|y|, AbsTol)` با scale vector؛ قبول/رد مرحله؛ گام جدید: `h *= 0.9*(1/err)^{1/5}` با سقف رشد ۵× و کف ۰.۲×.
- پیش‌فرض‌ها مطابق MATLAB: `RelTol=1e-3`، `AbsTol=1e-6`، `MaxStep = (tEnd-tStart)/10`.
- خروجی نقطه‌ای: درون‌یابی dense-output درجه ۴ خود DP (اگر گام‌ها رقیق‌تر از خواسته tspan باشند).
- **Cooperative cancel** هر ۵۰ مرحله + `Progress` event هر ۲۵۰ms برای UI (سند ۰۲ بخش ۳).
- `ode23`: Bogacki–Shampine 3(2) — برای دقت‌های پایین سریع‌تر.
- `odeset` پشتیبانی‌شده: `RelTol, AbsTol, MaxStep, InitialStep, NonNegative(پایه)`؛ `Events` و `Mass` → P2 با پیام صریح.

### 4.2 معیارهای صحت
مسائل مرجع با حل تحلیلی: `y'=−y`، `y'=y²`، van der Pol (μ=1000 → چالش ناپایداری)، oscillator میرا، مسئله keppler lite. همه با tolerances استاندارد در Golden Tests.

---

## ۵. ریشه‌یابی و انتگرال‌گیری عددی

### 5.1 `fzero` — روش Brent (bisection + secant + inverse quadratic)
- ورودی: تابع handle + براکت `[a b]` (علامت مخالف اجباری) یا نقطه شروع → جست‌وجوی براکت خودکار با الگوی MATLAB (expansion + شکست صادقانه اگر براکت نیافت).
- معیار توقف: `|Δx| ≤ 2*eps*|x| + tol` با tol پیش‌فرض `eps`؛ حداکثر ۱۰۰۰ تکرار.
- حفاظت: محدودیت تعداد فراخوانی تابع کاربر (فلاپ‌سنج) + cancel.

### 5.2 `roots` — از طریق eigenvalues ماتریس کامپانیون
- روش MATLAB استاندارد؛ بالانس ماتریس قبل از eig برای پایداری؛ ریشه‌های مختلط با مرتب‌سازی مطابق مرجع.

### 5.3 `integral` — Gauss–Kronrod تطبیقی (G7/K15)
- تقسیم بازگشتی بازه با صف اولویت (بزرگ‌ترین خطا اول)؛ budget پرسش‌های تابع: ۱۰٬۰۰۰.
- قواعد: singularity تشخیص‌پذیر → هشدار «همگرایی کند»؛ بازه بی‌نهایت با تبدیل متغیر مطابق مرجع؛ `Waypoints` پشتیبانی می‌شود (نقاط شکست).
- خروجی همیشه همراه برآورد خطا؛ اگر خطا > tolerance×۱۰ → warning ساختاریافته (MC:E4001 style) — نتیجه هرگز silent-مشکوک نیست.

---

## ۶. درون‌یابی و برازش

| تابع | الگوریتم | یادداشت |
|---|---|---|
| `interp1(...,'linear')` | جست‌وجوی دودویی روی grid + خطی | خارج بازه: NaN (پیش‌فرض MATLAB) با `extrap` اختیاری |
| `interp1(...,'pchip')` | Fritsch–Carlson با حفظ مونوتونی | بدون overshoot — برای داده آزمایشگاه حیاتی |
| `interp1(...,'spline')` | spline مکعبی natural/نهend | حل سه‌قطری Thomas |
| `polyfit(p)` | Least squares با Householder QR | **هرگز normal equations** (AᵀA) — شرط‌بندی بد؛ `polyfit` با degree بالا → هشدار شرط‌بندی + پیشنهاد مرکز/مقیاس (مطابق هشدار MATLAB) |
| `polyval` | Horner | پایدار |

---

## ۷. آمار و تصادف

- توصیفی همه با بعد `dim` و NaN-aware versions (`nanmean` رفتارِ skip-NaN پیش‌فرض نیست — مطابق MATLAB `mean` با NaN، NaN می‌دهد؛ تابع `omitnan` flag پشتیبانی می‌شود).
- `cov/corrcoef` با نرمال‌سازی n−1.
- توزیع‌ها (pdf/cdf/inv): نرمال (erf دقیق)، t، χ²، باینومیال — از `statrs` با wrap و تست مقابل مقادیر مرجع جدولی.
- **RNG:** PCG64 برای uniform + **Ziggurat** برای نرمال (رفتار آماری نرمال‌بودن تست می‌شود: χ² goodness روی ۱e6 نمونه در CI). `rng(seed)` deterministic — تکالیف تکرارپذیر می‌شوند. سازگاری بیت‌به‌بیت با Mersenne Twister MATLAB **هدف نیست** (مستند می‌شود: `randn` در MathCraft دنباله مستقل دارد؛ نتایج آماری معادل‌اند).
- `histogram`: قاعده پیش‌فرض binهای خودکار MATLAB-مانند (Scott تعدیل‌شده با زیبایی عددی) + `BinMethod` پایه + `BinWidth/BinLimits`.

---

## ۸. پایپ‌لاین رسم (mc-plot) — قرارداد کامل

### 8.1 PlotSpec (نمونه‌های کلیدی)
```rust
pub enum PlotSpec {
    Line2D { series: Vec<Series2D>, axes: Axes2D, legend: Option<Legend> },
    Scatter2D { .. }, Bar2D { .. }, Histogram { bins: Vec<f64>, counts: Vec<u64>, .. },
    Polar { .. }, Quiver { .. },
    Surface { x: Vec<f64>, y: Vec<f64>, z: Vec<Vec<f64>>, shading: Shading, colormap: Colormap },
    Mesh { .. }, Contour { levels: Vec<f64>, .. }, Plot3 { .. },
    Subplot { rows: u32, cols: u32, cells: Vec<(u32, u32, Box<PlotSpec>)> },
}
pub struct Series2D { pub x: Vec<f64>, pub y: Vec<f64>, pub style: LineStyle /* color, dash, marker, width */ }
```
- **رنگ‌ها:** پالت ۸ رنگی دسته‌بندی پلات (سند ۰۶ بخش ۳) — `plot(x,y)` بدون رنگ، به ترتیب پالت پیش می‌رود (رفتار MATLAB با پیش‌فرض ما).
- **LineSpec پارس می‌شود:** `'r--o'` → color red, dashed, circle marker — جدول تبدیل کامل در doc تابع `plot`.
- محورها: title/labels/grid/limits/equal/log scale + **تیک‌های زیبا** (الگوریتم 1-2-5 با nice-step) در Rust تولید می‌شوند تا Dart فقط بکشد.
- fmt نمایش اعداد تیک: `%.4g` و حذف صفرهای بی‌معنا.

### 8.2 Decimation (LTTB)
- آستانه: سری با > 4000 نقطه → LTTB به ≤ 4000 نقطه برای viewport فعلی؛ zoom-in عمیق → درخواست مجدد decimation از Rust با بازه جدید (سند ۰۲ بخش ۷).
- کیفیت: تست چشمی ثابت (golden image) + تست عددی (خطای ناحیه زیر منحنی < ۱٪ پس از decimation روی مجموعه‌های مرجع).

### 8.3 مدل 3D
- `surf/mesh`: meshgrid → مثلث‌بازی grid (دو مثلث در خانه)؛ نرمال‌ها در Rust؛ **رندر در Dart** با painter's algorithm (sort بر عمق مرکز مثلث، flat shading با یک نور ثابت قابل تنظیم `camlight`) — بدون GPU dependency؛ بودجه ۶۰fps تا ۵۰k مثلث (sort O(n log n) در Dart isolate per frame قابل قبول است؛ در صورت نیاز، sort عمق در Rust cache می‌شود و چرخش کامل re-project سبک انجام می‌شود).
- `contourf`: marching squares استاندارد با ۱۰ سطح پیش‌فرض.
- colormap پیش‌فرض: **parula** بازسازی‌شده (تقریب پذیرفته‌شده چندجمله‌ای/جدول ۱۶ نقطه‌ای با lerp) + `viridis, jet, hot, cool, gray`.

---

## ۹. سیاست کیفیت عددی (Numerical Quality Policy)

1. **هر نتیجه تطبیقی، برآورد خطا دارد** و در UI به warning قابل مشاهده تبدیل می‌شود اگر مشکل‌ساز باشد.
2. **toleranceهای پیش‌فرض** همه مطابق MATLAB (RelTol=1e-3 برای ode، 1e-10 نسبی برای integral) — دانشجو تجربه یکسان ببیند.
3. **هیچ «اصلاح خفیف» ورودی کاربر مجاز نیست** (مثلاً اضافه‌کردن نقطه براکت به fzero خودسرانه) — در عوض هشدار آموزشی.
4. **Denormal/overflow:** مطابق IEEE؛ `exp(1000)=Inf` با warning یک‌بار در session.
5. **ماتریس بدشرط:** `rcond < eps*100` → هشدار «نتیجه ممکن است نادقیق باشد (شرط‌بندی بد)» + پیشنهاد `cond(A)`.

---

## ۱۰. ردیابی به تست‌ها

هر بخش این سند به مجموعه تست سند ۰۸ گره خورده است:
- جبرخطی → property tests (A·A⁻¹≈I و غیره) + golden
- ode → مسائل تحلیلی + golden van der Pol
- integral → انتگرال‌های تحلیلی مرجع
- plot → golden images + تست decimation
- stats/rng → goodness-of-fit در CI

هیچ تغییر در این موتور بدون اجرای کامل آن تست‌ها merge نمی‌شود (بند DoD سند ۰۷).
