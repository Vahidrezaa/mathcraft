# سند ۰۲ — معماری فنی (Technical Architecture)

> **MathCraft** | سند معماری نسخه 1.0
> این سند «چطور بسازیم» را در سطح سیستم تعریف می‌کند: مرزها، کرِیت‌ها، قراردادهای ارتباطی، مدل اجرا و بسته‌بندی.

---

## ۱. نمای کلان (Big Picture)

معماری از دو نیمه‌ی مجزا با یک قرارداد باریک تشکیل شده است:

```
┌─────────────────────────────────────────────────────────────┐
│                    Android APK (MathCraft)                   │
│                                                              │
│  ┌──────────────────────────┐      ┌──────────────────────┐ │
│  │      Flutter UI (Dart)   │      │   Rust Engine        │ │
│  │                          │      │   (native .so)       │ │
│  │  • کنسول REPL            │ FFI  │                      │ │
│  │  • ادیتور + هایلایت      │◄────►│  • mc-parser (Lexer/ │ │
│  │  • بازرس متغیرها         │ FRB  │    Parser/AST)       │ │
│  │  • موتور رسم 2D/3D       │ v2   │  • mc-interp (Evaluator)│
│  │    (CustomPainter)       │      │  • mc-num (جبرخطی/ODE)│ │
│  │  • رندر LaTeX نتایج      │      │  • mc-cas (سمبلیک)    │ │
│  │  • مستندات آفلاین        │      │  • mc-plot (PlotSpec)  │ │
│  │  • State: Riverpod       │      │  • mc-ffi (قرارداد)    │ │
│  └──────────────────────────┘      └──────────────────────┘ │
│         │                                     │              │
│   drift(SQLite): history/docs index      rug/GMP, faer      │
└─────────────────────────────────────────────────────────────┘
```

**قانون بنیادین معماری:** همه محاسبات و تصمیم‌های دامنه (منطق اجرا، عدد، سمبلیک، شکل داده خروجی رسم) در Rust است. Dart **فقط** نمایش، تعامل و ماندگاری UI را مدیریت می‌کند. هیچ الگوریتم ریاضی در Dart نوشته نمی‌شود — حتی یک interpolation ساده. این قانون باعث می‌شود موتور در آینده بدون تغییر، برای دسکتاپ/iOS/وب استفاده شود (NFR-5 سند ۰۱).

**چرا این مرز به این شکل؟** چون «صحت محاسبات» یک‌بار ساخته می‌شود و «حس مدرن UI» مرتب تغییر می‌کند. جدایی این دو نرخ تغییر (rate of change) باعث می‌شود اطمینان موتور با رشد سریع UI آلوده نشود.

---

## ۲. ساختار Workspace در Rust

مونوریپو Cargo با کرِیت‌های کوچک و مسئولیت‌دار (هر کرِیت مستقلاً تست‌پذیر، بدون وابستگی به اندروید):

```
mathcraft/
├── crates/
│   ├── mc-core/        # انواع پایه: Value، Matrix (column-major)، Complex، ذخیره COW
│   ├── mc-parser/      # Lexer، Pratt Parser، AST، خطاهای مکان‌دار
│   ├── mc-interp/      # Evaluator، Environment (workspace)، dispatch توابع، COW
│   ├── mc-num/         # جبر خطی (faer)، ODE، بهینه‌سازی ریشه، انتگرال عددی، آمار، RNG
│   ├── mc-cas/         # درخت عبارت سمبلیک، اعداد دقیق (rug)، ساده‌سازی/مشتق/انتگرال/حد/سری
│   ├── mc-plot/        # PlotSpec: مدل داده سریال‌شدنی رسم (بدون ترسیم!) + LTTB decimation
│   ├── mc-ffi/         # قرارداد flutter_rust_bridge: API عمومی، thread pool، streaming
│   └── mc-cli/         # کنسول بومی ترمینال `mc` (REPL) — فرانت‌اند دوم برای Termux/دسکتاپ (P0.1)
├── app/                # پروژه Flutter (Dart)
├── docs-pack/          # مستندات آفلاین دوزبانه (Markdown + متادیتا → باندل)
├── test-corpus/        # کورپوس رگرسیون: اسکریپت‌های .m + خروجی مرجع Octave
└── .github/workflows/  # CI (سند ۰۸)
```

**قواعد وابستگی (اجباری، با enforce در CI از طریق `cargo-deny`):**
- `mc-core` به هیچ‌چیز وابسته نیست؛ `mc-parser` فقط به core؛ `mc-interp` به core+parser+num+cas؛ `mc-ffi` به همه؛ `mc-cli` فقط به `mc-ffi`.
- `mc-cli` بدون هیچ وابستگی خارجی (فقط std) — همان قرارداد عمومی `mc-ffi` را مصرف می‌کند تا صحت موتور روی Termux/دسکتاپ بدون Flutter هم قابل صحت‌سنجی باشد؛ هر فرانت‌اند سوم آینده باید از همین مسیر برود.
- هیچ کرِیتی جز `mc-ffi` اجازه استفاده از APIهای platform-specific ندارد.
- وابستگی‌های کلیدی: `faer` (جبر خطی پرسرعت)، `rug` + GMP (اعداد دقیق)، `egg` (e-graphs برای ساده‌سازی سمبلیک)، `rustfft` (فاز ۲)، `statrs` (توزیع‌ها)، `rand`/`pcg` (RNG قابل تکرار)، `serde` (فقط برای docs-pack، نه مسیر داغ FFI).

---

## ۳. قرارداد ارتباطی Flutter ↔ Rust (mc-ffi)

از **flutter_rust_bridge v2** استفاده می‌کنیم. اصول قرارداد:

1. **پیام‌های تایپ‌دار، نه رشته خام:** ورودی/خروجی هر فراخوانی، ساختارهای Rust هستند که FRB برایشان کد Dart تولید می‌کند (`#[frb]` annotations). JSON روی مسیر داغ ممنوع.
2. **سه الگوی فراخوانی:**
   - `request/response` ساده: `evaluateLine(code) -> EvalResult` (برای REPL).
   - **streaming:** `runScript(code, sink)` — خروجی‌های کنسول (print، خطا، پلات، پیشرفت) به‌صورت event به Dart push می‌شوند؛ UI همان لحظه آپدیت می‌شود. این چیزی است که حس «زنده بودن» کنسول را می‌سازد.
   - **cancel:** هر job یک `jobId` دارد؛ `cancel(jobId)` — کاربر نباید در برابر حل‌ بی‌پایان بی‌دفاع باشد. همه حلگرها هر N تکرار یک cooperative check انجام می‌دهند.
3. **همه APIها async هستند:** Rust روی thread pool خودش اجرا می‌شود؛ UI isolate هرگز بلاک نمی‌شود (NFR عملکرد سند ۰۱).
4. **نسخه‌بندی قرارداد:** `api_version()` در زمان startup چک می‌شود؛ ناسازگاری = پیام واضح، نه کرش.

### قرارداد داده اصلی (خلاصه)

```rust
// mc-ffi — نمای ساده‌شده
pub struct EvalResult {
    pub job_id: u64,
    pub console_events: Vec<ConsoleEvent>,   // متن، خطا، warning
    pub plots: Vec<PlotSpec>,                 // مدل داده رسم (سند ۰۴ بخش ۸)
    pub workspace_delta: WorkspaceDelta,      // فقط متغیرهای تغییرکرده (نه کل workspace!)
    pub status: JobStatus,                    // Done | Cancelled | Failed
    pub duration_ms: u64,
}

pub enum ConsoleEvent {
    Print { text: String },
    Error  { err: McError },                  // ساختاریافته: کد، پیام FA/EN، خط، ستون، پیشنهاد
    Warning { code: String, text: String },
    Progress { job_id: u64, fraction: f32 },  // برای حلگرهای طولانی
}

pub struct McError {
    pub id: String,            // مثل MC:E1001_UNDEFINED
    pub line: u32, pub col: u32,
    pub message_en: String, pub message_fa: String,
    pub suggestion: Option<String>,           // did-you-mean
}
```

**WorkspaceDelta:** بعد از هر اجرا فقط متغیرهای تغییرکرده serialize می‌شوند (نام، کلاس، اندازه، پیش‌نمایش ≤ ۲۵۶ مقدار اول، دسترسی lazy برای نمای کامل). این تصمیم، بازرس متغیرها را با ماتریس‌های بزرگ هم روان نگه می‌دارد.

**توابع الحاقی بازرسی نشست (P0.1 — افزایشی، بدون شکستن v1):** `workspace_vars() -> Vec<VarSummary>` (فهرست کامل متغیرهای فعلی برای تب «متغیرها» و `whos` کنسول) و `clear_workspace() -> usize` (سمانتیک `clear` متلب). این‌ها فقط «خواندن/پاک‌کردن نشست» هستند، روی مسیر داغ ارزیابی اثری ندارند و بخشی از همان قرارداد v1 محسوب می‌شوند.

---

## ۴. مدل اجرا و Threading

- **UI Thread (Dart):** رندر و تعامل. ممنوع: هر محاسبه‌ای بیش از یک اسکن آرایه.
- **Engine Pool (Rust):** یک thread pool با `num_cpus`-1 (حداقل ۲) thread؛ jobs به‌صورت صف اجرا می‌شوند؛ **سمانتیک اجرا سریالی** است (MATLAB single-threaded workspace دارد؛ موازی‌سازی فقط درون توابع سنگین مثل `inv` و `ode45` با Rayon/توابع faer انجام می‌شود، نه بین دستورها). این تصمیم از رقابت روی workspace و رفتارهای نامشخص جلوگیری می‌کند.
- **Global Interpreter Lock منطقی:** هر job کل workspace را می‌بیند؛ اجرای همزمان دو اسکریپت پشتیبانی نمی‌شود (و نباید شود — ساده‌ترین مدل صحیح).
- **CancellationToken:** روی هر job؛ همه حلگرها داخلی چک می‌کنند.
- **First-class cancel UX:** دکمه توقف در نوار اجرا با شمارشگر زمان؛ پس از cancel، workspace در حالت consistent آخرین نقطه commit می‌ماند.

---

## ۵. مدل حافظه و داده (mc-core)

- **Value enum:** `Double (Matrix)`, `Complex (Matrix)`, `Logical`, `Char`, `String`, `Cell` (بصری محدود), `Struct` (فاز ۲), `Sym (Rc<SymNode>)`، `FunctionHandle`.
- **Matrix:** `rows, cols` + داده `Arc<ThinVec<f64>>` **column-major** (تصمیم D-11) + copy-on-write با `Arc::make_mut`. یعنی پاس‌دادن ماتریس به تابع O(1) است و فقط در نوشتن clone می‌شود — رفتار value-semantics MATLAB با هزینه reference.
- **Complex:** interleaved `[re, im, re, im, ...]` — سازگاری با Blas-like kernels و آسانی در bridge.
- **اندیس‌گذاری:** linear و subscripts هر دو پشتیبانی می‌شوند؛ `end` در پارس‌时间 به sentinel تبدیل و در ارزیابی resolve می‌شود.
- **NaN/Inf:** مطابق IEEE-754؛ مقایسه‌ها سمانتیک MATLAB (`NaN ~= NaN`)؛ جمع/N×M checks با پیام‌های دقیق.
- **بودجه حافظه:** workspace تا ~۵۰۰MB مجاز؛ در عبور، warning و پیشنهاد `clear`. هیچ دستور جامپ heap-alloc نامحدود نمی‌کند (مثلاً `zeros(1e6,1e6)` → خطای صریح «حافظه» نه کرش).

---

## ۶. ماندگاری و فایل‌ها

| دامنه | محل | فرمت |
|---|---|---|
| اسکریپت‌های کاربر | پوشه انتخابی کاربر (SAF / DocumentFile) | `.m` متنی UTF-8 |
| تاریخچه دستور | دیتابیس داخلی اپ | `drift` (SQLite) جدول `command_history` |
| مستندات | assets + کپی به دیتابیس برای FTS5 | Markdown + JSON index |
| Workspace snapshot | داخلی اپ + خروجی/ورودی کاربر | باینری MCWS1 (magic + version + typed arrays، little-endian، crc32) |
| تنظیمات | `shared_preferences` | JSON |
| `.mat` ورودی | فقط خواندن v7 (ZLib compressed، بدون v7.3/HDF5) | ماتریس‌های عددی/کاراکتر |

**پایداری پس از کرش:** ادیتور هر ۵ ثانیه (و در pause) snapshot در app-private می‌نویسد؛ در startup بعدی، بازیابی خودکار با toast «اسکریپت بازیابی شد».

**چرا MCWS به‌جای `.mat` کامل؟** چون `.mat` استاندارد باز ساده‌ای برای همه انواع ما نیست ( structs/cells پیچیده می‌شوند). می‌خوانیم که سازگاری بدهیم، می‌نویسیم فرمت خودمان را که مستند و نسخه‌دار است.

---

## ۷. پایپ‌لاین رسم (تصمیم D-10 به تفصیل)

مسئولیت‌ها به‌شدت تفکیک شده‌اند:

```
Rust (mc-num + mc-plot)                    Dart (Flutter)
─────────────────────────                  ─────────────────
محاسبه داده خام (مثلاً ۱۰۰k نقطه)   →    دریافت PlotSpec (typed)
downsample هوشمند LTTB تا ≤ ۴k نقطه/سری    نگهداری در Riverpod state
هندسه محورها، tickهای «زیبا» (1-2-5)        ترسیم با CustomPainter
برای 3D: مثلث‌بندی سطح + نرمال‌ها     →    رندر مثلث‌ها (painter's algorithm،
+ sort عمق اولیه                            flat shading با نور، canvas)
                                            تعامل: زوم/پن/چرخش
                                        →  درخواست مجدد داده در ناحیه جدید (zoom-in عمیق)
```

- **قرارداد PlotSpec:** enum بسته (Line، Scatter، Bar، Histogram، Surface، Mesh، Contour، Quiver، Polar، Subplot) + استایل‌ها + محورها + legend. کاملاً سریال‌شدنی به Dart structs.
- **decimation در Rust** چون: (الف) انتقال ۱۰۰k نقطه روی FFI گران است، (ب) LTTB شکل منحنی را حفظ می‌کند، (ج) در هر تغییر zoom می‌توان دوباره از داده کامل، نمایش بهینه ساخت — داده کامل در Rust می‌ماند و هرگز به UI منتقل نمی‌شود.
- **تعامل بدون FFI در حلقه فریم:** برای pan/zoom 2D، Dart فقط transform اعمال می‌کند (affine on painted coords) — بدون فراخوانی Rust؛ FFI فقط وقتی لازم است داده جدید/decimation جدید بخواهد. برای 3D، پروجکشن و sort مثلث‌ها در Dart با `vector_math` انجام می‌شود (داده mesh یک‌بار از Rust می‌آید) تا ۶۰fps بدون overhead bridge تضمین شود.

---

## ۸. بسته‌بندی و Build اندروید

- **ابزار:** `cargo-ndk` برای کامپایل کرِیت‌ها به `aarch64-linux-android`، `armv7-linux-androideabi`، `x86_64-linux-android` (فقط debug/شبیه‌ساز) → تولید `libmathcraft.so` → link در پروژه Flutter.
- **minSdk 24 / targetSdk 35.** ABIs انتشار: `arm64-v8a` (اصلی) و `armeabi-v7a` (سازگاری)؛ APK split per-ABI برای حجم.
- **بودجه حجم:** Flutter engine ~8MB + Rust `.so` ~4-7MB (strip + LTO + `opt-level=3`, `panic=abort` در release) + فونت‌ها ~2MB + docs-pack ~3-6MB → **کل ≤ ۴۵MB** (NFR-6).
- **امنیت build:** `cargo-deny` برای لایسنس‌ها/advisories؛ CI هر release را از روی tag می‌سازد (reproducible path برای F-Droid: toolchain pin + lockfile).
- **امضا:** keystore در secretهای CI؛ APKهای release همیشه امضای ثابت (برای به‌روزرسانی در بازارها).

---

## ۹. مدیریت خطا در سراسر مرز (Cross-Boundary Errors)

فلسفه: **خطا یک مقدار است، نه یک استثنای مهلک.** در Rust، همه‌چیز `Result<_, McError>` (بدون panic در مسیر محصول؛ `panic=abort` فقط آخرین خط دفاعی است). در Dart، `EvalResult.status` و `McError` ساختاریافته به لایه نمایش می‌روند و UI آن را به کارت خطای آموزش‌دهنده (F-1.5 سند ۰۱) تبدیل می‌کند.

سطح‌های خطا:
1. **سینتکسی** (پارسر): موقعیت دقیق + پیشنهاد — در gutter ادیتور هم زنده نمایش داده می‌شود.
2. **اجرایی** (ارزیابی): نوع، message دو زبانه، did-you-mean روی نام متغیر/تابع (فاصله Levenshtein ≤ ۲ روی workspace + builtins).
3. **عددی** (حلگرها): هشدار ساختاریافته — «انتگرال به همگرایی نرسید؛ نتیجه با tolerance X» — هرگز silent.
4. **منابع**: حافظه/timeout/cancel با پیام مشخص.

---

## ۱۰. امنیت

- بدون مجوز شبکه → سطح حمله شبکه‌ای صفر.
- اجرای اسکریپت فقط روی داده‌های خود کاربر؛ **هیچ executor از راه دور نداریم.**
- فایل‌های دریافتی (مثلاً `.m` از پیام‌رسان) در sandbox اپ باز می‌شوند؛ بدون اجرای خودکار.
- rng پیش‌فرض برای کاربرد آموزشی کافی است؛ هیچ ادعای رمزنگاشتی در مستندات نمی‌آید.

---

## ۱۱. آمادگی برای پلتفرم‌های آینده (تصمیم D-03)

نسخه ۱ فقط اندروید است، اما این پنج قاعده از الان رعایت می‌شود تا دسکتاپ/iOS بعداً «پورت» نباشد بلکه «بیلد» باشد:

1. تمام منطق در کرِیت‌های خالص Rust بدون وابستگی platform.
2. قرارداد mc-ffi مستقل از اندروید (تست‌های موتور روی میزکار با همان API اجرا می‌شوند).
3. UI Flutter بدون فرض اندروید (بدون channel بومی اختصاصی؛ SAF با abstraction `FileRepo` پوشانده شده).
4. مستندات به‌صورت asset مستقل از پلتفرم.
5. تصمیم‌های موبایل‌محور در UI (مثل bottom nav) در لایه Layout سند ۰۶ با breakpoint جداشده‌اند تا دسکتاپ layout جایگزین بگیرد.
