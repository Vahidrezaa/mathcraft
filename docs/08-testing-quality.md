# سند ۰۸ — استراتژی تست و کیفیت (Testing & Quality)

> **MathCraft** | نسخه 1.0
> فلسفه: **صحت قابل اندازه‌گیری است یا وجود ندارد.** ما سازگاری با MATLAB را «احساس» نمی‌کنیم؛ با کورپوس و tolerance اندازه می‌گیریم. این سند زیرساخت اعتماد کل پروژه است و از روز اول P0 باید بنا شود (سند ۰۷ بخش ۳).

---

## ۱. هرم تست و گیت‌های کیفیت

```
        ┌──────────────┐  Benching (regression >20% = fail)
        │  Performance │
      ┌─┴──────────────┴─┐  Golden Tests (Octave، ۲۰۰ اسکریپت)
      │   Compatibility  │
    ┌─┴──────────────────┴─┐  Property-based (invariants ریاضی)
    │       Properties     │
  ┌─┴──────────────────────┴─┐  Unit/Integration (هر الگوریتم)
  │          Units           │
┌─┴──────────────────────────┴─┐  Lint/Format/Typing (fmt, clippy, analyze)
│            Static            │
└──────────────────────────────┘
```

گیت‌ها: هیچ PR بدون عبور هر ۶ لایه merge نمی‌شود. `panic!` در کد مسیر محصول با grep-guard در CI ممنوع (به‌جز assertهای لایه داخلی).

**سیاست دقیق grep-guard `todo!/unimplemented!`:** این گارد فقط روی «کد دست‌نویس ما» اعمال می‌شود — یعنی `crates/*/src` **به‌استثنای فایل‌های اتوژنریت** flutter_rust_bridge. الگوی استثنا در CI: `grep -rn -E "todo!|unimplemented!" crates/*/src --exclude='frb_generated*.rs'`. دلیل: فایل `frb_generated.rs` خروجی machine-generated کدِجن است (با هر بار اجرای codegen از نو ساخته می‌شود و هر دست‌کاری دستی در آن با اولین codegen بازنویسی می‌شود) و `unimplemented!("")`های آن در شاخه‌های codec مسیر Dart→Rust برای نوع‌هایی قرار می‌گیرد که قرارداد v1 فقط در جهت Rust→Dart منتقل می‌کند (EvalResult/ConsoleEvent/JobStatus/WorkspaceDelta) — یعنی در زمان اجرا از طرف قرارداد غیرقابل‌رسد هستند. اگر روزی قرارداد نسخه‌ای شود که این شاخه‌ها قابل‌رسد شوند (نوعی به‌عنوان آرگومان Dart→Rust منتقل شود)، FRB به‌طور خودکار decode واقعی تولید می‌کند و placeholder حذف می‌شود — هیچ اقدام دستی لازم نیست. نگهبان انسانی: هیچ‌گاه `todo!`/`unimplemented!`/`panic!` در فایل‌های دست‌نویس `crates/*/src` (به‌جز ماژول‌های test) نوشته نمی‌شود.

---

## ۲. Golden Tests — قلب سازگاری با MATLAB/Octave

### ۲.۱ ایده
پارسر/مفسر/حلگرهای ما باید خروجی‌های **عددی** منطبق بر مرجع بدهند. مرجع اجرایی: GNU Octave 9 (اوپن‌سورس و قابل اجرا در CI ابری).

### ۲.۲ مکانیک
- کورپوس: `test-corpus/cases/*.m` + `expected/*.json` (خروجی همه نمایش‌های کنسول + مقدار متغیرهای مشخص‌شده + متن خطا).
- تولید expected: در CI (job جدا، `octave --no-gui --eval`) با اسکریپت runner که هر case را اجرا و خروجی را به JSON canonical تبدیل می‌کند (sort keys، rounding به 15 رقم، escape).
- مقایسه: tolerance نسبی 1e-9 (جهت مواقع ضروری float)؛ مقادیر صحیح/گسسته دقیق؛ NaN فقط به NaN برابر؛ علامت صفر مهم نیست.
- **قاعده ضد تقلب:** expected هرگز از خروجی خود MathCraft تولید/به‌روزرسانی نمی‌شود؛ به‌روزرسانی فقط با تغییر دستی review شده.
- کورپوس نسخه‌دار است: هر باگ سازگاری گزارش‌شده → یک case جدید → هرگز حذف نمی‌شود (رگرسیون ابدی).

### ۲.۳ ساختار کورپوس ۲۰۰ اسکریپتی (توزیع هدفمند)
| حوزه | تعداد | نمونه |
|---|---|---|
| سینتکس/پارسر/سمانتیک (colon، اندیس، COW، strings فارسی) | ۵۰ | `0:0.1:1`، `A(end:-1:1)`، انتساب منطقی |
| جبر خطی و ماتریس | ۴۵ | eig/svd/lu، `\` روی همه dispatchها، بدشرط‌ها |
| روش‌های عددی | ۳۵ | ode45 دو مسئله، integral با singularity، interp/spline |
| آمار/RNG (الگوهای آماری، نه دنباله) | ۲۰ | توزیع نمونه‌ها، corrcoef |
| چندجمله‌ای و مختلط | ۲۰ | roots، conv، atan2 مختلط |
| رسم (تست دامنه داده PlotSpec، نه پیکسل) | ۱۵ | شکل داده سری‌ها، حدود محور، تیک‌ها |
| خطاها و edge | ۱۵ | پیام‌های ساختاریافته، حذف خروجی، empty |
| CAS مرجع عددی (خروجی sym با eval عددی چک می‌شود) | ۱۰ | diff/int/solve معتبر با чисدقیق |

---

## ۳. Property-Based Testing (proptest)

بیان‌های ثابت که هرجا نقض شوند، shrinking می‌شوند:

| دامنه | Property |
|---|---|
| ماتریس | `A·A⁻¹ ≈ I`، `‖A−Q·R‖` کوچک، `eig_sym → مقادیر حقیقی`، `det(LU) == det(A)` |
| colon | طول و عنصر اول/آخر مطابق فرمول مرجع برای ۱۰k سه‌تایی تصادفی |
| COW | بعد از انتساب، تغییر b روی a اثر ندارد (هرگز) |
| مشتق CAS | `diff(f)` در نقاط تصادفی = finite-difference(f) در tolerance تطبیقی |
| انتگرال CAS | `d/dx F(x) == f(x)` — خودآزمایی اجباری هر نتیجه (سند ۰۵) |
| spline | گذر از نقاط داده + پیوستگی C² |
| RNG | نرمال‌بودن Ziggurat (χ²)، بازه uniform، تکرارپذیری seed |
| ode | حفظ انرژی روی مسئله هامیلتونی با tol مناسب |

---

## ۴. Fuzzing

- هدف: **mc-parser** (ورودی نامطمئن‌ترین لایه) — cargo-fuzz با corpus شروع از کورپوس؛ nightly، ۱۵ دقیقه.
- شناسه‌های panic/انفجار زمانی (parser هرگز نباید هنگ کند) → باگ = gater.
- ثانویه: fuzz قرارداد FFI با داده‌های خراب MCWS (پارس snapshot خراب هرگز نباید panic کند).

---

## ۵. بنچمارک و رگرسیون عملکرد

- ابزار: `criterion` در Rust (linalg، ode، integral، plot pipeline) + integration_test بنچمارک Flutter (باز شدن تب‌ها، اسکرول بازرس).
- آستانه: regression > ۲۰٪ نسبت به baseline = گیر CI؛ baseline به‌صورت weekly refresh با annotation در repo.
- هر بودجه سند ۰۱ (NFR-1) یک bench named دارد؛ جدول وضعیت بودجه‌ها در README تولیدی CI منتشر می‌شود.

---

## ۶. تست UI (Flutter)

- Widget tests: رفتار key (تب‌سوییچ، crosshair snap، ویرایش سلول بازرس).
- Golden image tests: اجزای CDS (کارت خروجی، خطا، کیبورد ریاضی، نمودار با داده ثابت) در ۴ حالت (دو تم × دو زبان) — رنگ/فاصله درست = قفل شده.
- تست inset: کیبورد باز در هر دو جهت، بدون overlap (هشدار مستمر در CI شبیه‌ساز).
- دسترس‌پذیری: talkback semantics smoke + dynamic font 130% golden.

---

## ۷. CI/CD (GitHub Actions)

```
PR checks:
  rust: fmt --check | clippy -D warnings | cargo-deny (license/advisory) | test --all-features
  dart: analyze | format | test
  build: cargo-ndk (arm64, armv7, x86_64) + flutter build apk (debug) — اطمینان از link شدن
Nightly:
  golden: octave runner → مقایسه کامل کورپوس
  fuzz: parser 15min | property expanded (۱۰k iteration)
  bench: criterion + مقایسه baseline
Release (tag):
  reproducible build → signed APK (arm64, armv7) → artifact + release notes خودکار از milestone changelog
```

- Lockfiles پین‌شده (Cargo.lock + pubspec.lock)؛ toolchain نسخه‌دار (rust-toolchain.toml) — مسیر F-Droid.
- گزارش‌ها: کاورج (llvm-cov، هدف موتور ≥ ۸۵٪؛ لایه‌های bridge/UI بدون سقف سخت).

---

## ۸. سیاست tolerance و مقایسه (مرجع واحد)

| مقایسه | tolerance |
|---|---|
| عدد صحیح/گسسته (شمارش‌ها، اندازه‌ها، bins) | دقیق |
| float عمومی | نسبی 1e-9 (یا 1e-12 برای عملیات پس‌محض) |
| حلگرهای تطبیقی (ode/integral) | 1e-6 نسبی + چک order of error estimate |
| RNG | آماری (χ² در CI)، نه بیت‌به‌بیت |
| متن خروجی | دقیق (format مشترک) — چون display خودمان هم تست می‌شود |
| خطاها | ID + نوع دقیق؛ متن آزاد |

هر انحراف tolerance جدید = تغییر این سند + review؛ tolerance مخفی در کد ممنوع (const مرکزی `TolerancePolicy`).

---

## ۹. QA انسانی (آنچه اتوماسیون نمی‌بیند)

- چک‌لیست دستی هر release (سند ۰۷ بخش ۵): ۴ پروفایل دستگاه × ۲ جهت × ۲ زبان × ۲ تم.
- تست کاربردپذیری دوره‌ای با ۵ دانشجوی واقعی (P5 به بعد) — شاخص: «اولین پلات موفق ≤ ۵ دقیقه» (سند ۰۱ بخش ۹).
- بازبینی طراحی هر دمو مایل‌استون با چک‌لیست anti-pattern سند ۰۶ بخش ۱۲.

---

## ۱۰. مدیریت باگ و رگرسیون

- هر باگ سازگاری/صحتی → case کورپوس + تست واحد → ابدی.
- برچسب‌ها: `compat` (سازگاری مرجع)، `correctness`، `perf`، `ux`.
- سیاست اولویت: سازگاری silently-wrong > کرش > کندی > زیبایی. (چون جواب اشتباه بی‌صدا اعتماد محصول را می‌کشد.)
