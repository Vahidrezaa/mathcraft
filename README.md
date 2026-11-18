# MathCraft — موتور محاسبات مهندسی آفلاین، موبایل‌فرست و متن‌باز

> **جایگزین MATLAB برای ترم ۱-۲ مهندسی** — موتور Rust + رابط Flutter، خروجی APK آفلاین اندروید.
> «کل توان محاسباتی یک نرم‌افزار حرفه‌ای، همیشه در جیب تو — بدون اینترنت، بدون لایسنس، بدون اشتراک.»

**TL;DR (English):** MathCraft is an offline-first, open-source MATLAB replacement for
early engineering coursework: a Rust engine (parser, interpreter, numerics on
[faer](https://github.com/sarah-ek/faer), plotting) with a Flutter UI, bilingual
FA/EN and RTL-native, shipping as an Android APK. No internet permission, ever.
As of 2026-11-18: language core + numerics + 2-D plotting are code-complete
(642 Rust + 59 Flutter tests); see [`docs/14`](docs/14-project-audit-2026-11.md)
for the current audit.

[![CI](https://github.com/Vahidrezaa/mathcraft/actions/workflows/ci.yml/badge.svg)](https://github.com/Vahidrezaa/mathcraft/actions/workflows/ci.yml)
[![License: GPL-3.0](https://img.shields.io/badge/License-GPL--3.0-blue.svg)](LICENSE)

---

## کجاییم؟ (۲۰۲۶-۱۱-۱۸ — سند ۱۴ حسابرسی)

| فاز (سند ۰۷) | وضعیت |
|---|---|
| P0 شالوده + CLI ترمینال + CI/APK | ✅ بسته |
| P1 زبان و ماتریس (۱۷/۱۷ DoD + تأیید دستگاه) | ✅ بسته |
| P2 عددی و آمار + تکمیل زبان (گام‌های ۲.۳–۲.۱۱) | ✅ بسته + تگ `p2-code-complete` |
| P3-a رسم دوبعدی (۳.۱–۳.۵.۲) | 🔶 کد-کامل — ۶۴۲ تست Rust + ۵۹ تست Flutter سبز؛ چک دستگاه در جریان |
| بعدی: 3.6 CI-golden → 3.7 فایل I/O → P3-b سه‌بعدی → P4 CAS → P5 IDE → v1.0 | ⬜ |

**الان چه کار می‌کند؟**

- **زبان متلب‌گونه:** ماتریس‌ها و عبارات کامل با سمانتیک متلب (`2^3^2=64`، `ans`، suppression)،
  انواع عددی/مختلط/`char`/`struct`، اندیس‌گذاری زنجیره‌ای `[1 2 3](2)`، توابع فایل/بی‌نام `.m`،
  سینتکس فرمانی (`hold on`، `axis equal`)، ~۱۲۷ تابع داخلی با did-you-mean، و خطاهای ساختاریافتهٔ
  دوزبانه با موقعیت سطر/ستون.
- **موتور عددی:** جبر خطی (qr/svd/eig/chol/… چندخروجی)، ode45/ode23 با نوار پیشرفت و لغو،
  fzero/integral(Waypoints)، درون‌یابی ۱/۲ بعدی، polyfit/roots/conv، آمار پایه، RNG تکرارپذیر،
  بودجهٔ حافظه (خطای پاک به‌جای کرش).
- **رسم دوبعدی:** ۲۸ تابع گزاره‌ای (`plot` چندگروهی، `semilogx/loglog`، `bar/barh`، `histogram`،
  `area/fill`، `polarplot`، `errorbar`، `stem/stairs`، `subplot`، `legend`+Location،
  `xlim/ylim/xticks/yticks`، …) + نمایشگر لمسی: پن/پینچ حول نقطهٔ لمس، data tip، crosshair،
  محور لگاریتمی با TeX، decimation هوشمند تا ۱۰۰۰+ نقطه، خروجی PNG.
- **اپ:** کنسول رویدادی + ادیتور با هایلایت + تب نمودار؛ دوزبانهٔ FA/EN، RTL بومی با جزیرهٔ LTR
  برای کد/نمودار؛ تم CDS تیره/روشن.

## معماری در یک نگاه (سند ۰۲)

```
Flutter UI (Dart)  ◄── قرارداد باریک flutter_rust_bridge v2 ──►  Rust Engine (libmathcraft.so)
نمایش، تعامل، RTL                                               mc-parser → mc-interp → mc-num
                                                                + mc-plot (مدل دادهٔ رسم)
```

**قانون بنیادین:** همهٔ محاسبات در Rust؛ در Dart هیچ الگوریتم ریاضی وجود ندارد.

| پوشه | محتوا |
|---|---|
| `crates/` | ۸ کرِیت Rust (سند ۰۲ بخش ۲): mc-core/mc-parser/mc-interp/mc-num/mc-plot/mc-ffi/mc-cas(placeholder تا P4)/mc-cli (کنسول ترمینال `mc`) |
| `app/` | پروژهٔ Flutter با تم CDS و Craft Icons (سند ۰۶) |
| `docs/` | ۱۴ سند پیش‌تولید + سه سند زنده: `ROADMAP.md` (نقشهٔ گام‌ها — BINDING) · `STATUS.md` (منبع حقیقت وضعیت) · `COMPATIBILITY.md` (ماتریس سازگاری متلب) |
| `test-corpus/` | کورپوس رگرسیون (۷۸ case) با اوراکل Octave در CI + مرجع مستقل scipy |

## راه‌اندازی در Termux (اندروید — کاملاً بومی، بدون Flutter)

```bash
pkg install git rust   # + پیش‌نیازها طبق scripts/setup-termux.sh
git clone https://github.com/Vahidrezaa/mathcraft && cd mathcraft
bash scripts/setup-termux.sh    # پیش‌نیاز + cargo test + بیلد release + نصب mc
mc                              # کنسول تعاملی — MC_LANG=fa mc برای فارسی
```

نکته: اولین بیلد روی گوشی چند دقیقه طول می‌کشد؛ دفعات بعد فوری است. تا P4 همه کرِیت‌ها
pure-Rust هستند و نیازی به clang/GMP نیست. خروجی اسکریپتی: `mc -e "2^3^2"` → `ans = 64`.

## بوت‌استرپ محلی (مسیر Flutter/APK روی رایانه)

پیش‌نیازها: Rust (پین‌شده با `rust-toolchain.toml`)، Flutter 3.24+، اندروید SDK، `cargo-ndk` و
`flutter_rust_bridge_codegen`.

```bash
git clone https://github.com/Vahidrezaa/mathcraft && cd mathcraft
./scripts/bootstrap.sh             # flutter create + codegen FRB + pub get + verify + تم لانچ CDS
./scripts/build_rust_android.sh    # cargo-ndk → libmathcraft.so (arm64-v8a, armeabi-v7a)
cd app && flutter run --release
```

**⚠️ مهم — بیلد release، نه debug:** بیلد debug حجم ~۱۵۰MB و استارتاپ کند دارد؛ این طبیعت debug
است. همیشه `--release` — برای نصب: `flutter build apk --release --split-per-abi`.

## توسعه روزمره

```bash
cargo test --workspace                                   # موتور (۶۴۲ تست)
cargo clippy --workspace --all-targets -- -D warnings    # گیت CI
cd app && flutter test                                   # UI با FakeEngineClient (۵۹ تست)
cd app && flutter analyze
flutter_rust_bridge_codegen generate                     # بعد از هر تغییر mc-ffi/src/api.rs
```

## قواعد غیرقابل‌مذاکره (سند ۰۰ بخش ۴)

1. آفلاین مطلق — حتی مجوز `INTERNET` نمی‌گیریم.
2. هرگز UI را بلاک نکن — همهٔ محاسبه روی thread pool رست.
3. صحت قبل از سرعت — Golden Tests از روز اول (کورپوس + اوراکل Octave در CI).
4. حس نرم‌افزار واقعی — ایموجی هرگز آیکون نیست؛ فقط Craft Icons (سند ۰۶ §2.5).
5. شکست صادقانه — هرگز نتیجهٔ اشتباهِ مطمئن‌نما؛ هر انحراف از متلب در COMPATIBILITY ثبت می‌شود.

## مجوز

کد: **GPL-3.0-or-later** (تصمیم D-08 — هم‌راستا با GNU Octave) — فایل [`LICENSE`](LICENSE).
مستندات `docs/`: CC BY-SA 4.0.
MathCraft محصولی مستقل است و به MathWorks/MATLAB وابستگی ندارد (سند ۰۱ بخش ۱۰).
