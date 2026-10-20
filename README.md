# MathCraft — موتور محاسبات مهندسی آفلاین، موبایل‌فرست و متن‌باز

> **جاگزیر MATLAB برای ترم ۱-۲ مهندسی** — موتور Rust + رابط Flutter، خروجی APK آفلاین اندروید.
> «کل توان محاسباتی یک نرم‌افزار حرفه‌ای، همیشه در جیب تو — بدون اینترنت، بدون لایسنس، بدون اشتراک.»

[![CI](https://github.com/mathcraft/mathcraft/actions/workflows/ci.yml/badge.svg)](.github/workflows/ci.yml)
[![License: GPL-3.0](https://img.shields.io/badge/License-GPL--3.0-blue.svg)](LICENSE)

این ریپو در فاز **P0 (شالوده)** است — نقشه کامل در [`docs/`](docs/) (۹ سند پیش‌تولید، قرارداد رسمی پروژه).

---

## معماری در یک نگاه (سند ۰۲)

```
Flutter UI (Dart)  ◄── قرارداد باریک flutter_rust_bridge v2 ──►  Rust Engine (libmathcraft.so)
نمایش، تعامل، RTL                                                     mc-parser → mc-interp → mc-num → mc-cas
```

**قانون بنیادین:** همه محاسبات در Rust؛ در Dart هیچ الگوریتم ریاضی وجود ندارد.

| پوشه | محتوا |
|---|---|
| `crates/` | ۸ کرِیت Rust workspace (سند ۰۲ بخش ۲) — شامل `mc-cli` (کنسول بومی ترمینال) |
| `app/` | پروژه Flutter با تم CDS و Craft Icons (سند ۰۶) |
| `docs/` | ۹ سند پیش‌تولید — هر تصمیم فنی باید به آن‌ها ارجاع دهد |
| `docs-pack/` | مستندات آفلاین دوزبانه (P5) |
| `test-corpus/` | کورپوس رگرسیون Octave (P0 هارنس → P1 تکمیل) |

## وضعیت فعلی (P0 — سند ۰۷)

- [x] مونوریپو + اسکلت ۸ کرِیت + قواعد وابستگی
- [x] پارسر Pratt زیرمجموعه P0 با اولویت دقیق MATLAB (توان چپ‌گرا: `2^3^2 = 64`)
- [x] ارزیاب با سمانتیک متلب: `ans`، suppression با `;`، IEEE (`1/0=Inf`)، `format short`
- [x] قرارداد mc-ffi نسخه‌دار (`evaluate_line`، `EvalResult`، `McError` دوزبانه، `PlotSpec` v1)
- [x] ۴۲ تست سبز + clippy سخت‌گیر (`-D warnings`) + fmt
- [x] کنسول بومی `mc` (P0.1): REPL ترمینال روی همان قرارداد mc-ffi، بدون وابستگی خارجی — `mc -e "1+1"` → `ans = 2`
- [x] مسیر بومی Termux: `scripts/setup-termux.sh` (تست + بیلد release + نصب `mc` در $PREFIX/bin)
- [x] پوسته UI با تم CDS (تیره/روشن)، ناوبری ۵ تبی، ۱۲ Craft Icon اختصاصی SVG
- [x] سوییچ آنی زبان (FA/EN) و تم (مدرن/آشنا) — با تست ویجت
- [ ] اتصال end-to-end روی دستگاه واقعی (نیازمند بیلد محلی — بخش زیر)

**دموی P0:** کنسولی که `1+1` را با `ans = 2` از موتور واقعی Rust جواب می‌دهد.

## راه‌اندازی در Termux (اندروید — کاملاً بومی، بدون Flutter)

در Termux موتور Rust روی خود گوشی کامپایل می‌شود و کنسول تعاملی `mc` (فرانت‌اند دوم روی همان قرارداد mc-ffi — سند ۰۲ بخش ۲) در اختیارت است:

```bash
cd ~
tar -xzf MathCraft-P0-Foundation-v1.1.tar.gz   # بسته دانلودشده در پوشه خانه
cd mathcraft
bash scripts/setup-termux.sh    # پیش‌نیاز + cargo test + بیلد release + نصب mc
mc                              # کنسول MathCraft — تور: demo — فارسی: MC_LANG=fa mc
```

نکته‌ها:
- اولین بیلد روی گوشی چند دقیقه طول می‌کشد (کامپایل کامل workspace)؛ دفعات بعد فوری است.
- اگر rust از `pkg install rust` باشد (بدون rustup)، فایل `rust-toolchain.toml` نادیده گرفته می‌شود — مشکلی نیست. با rustup هم اسکریپت با `RUSTUP_TOOLCHAIN=stable` از دانلود دوباره جلوگیری می‌کند.
- در P0.1 همه کرِیت‌ها pure-Rust هستند (rug/GMP در P4 به mc-cas اضافه می‌شود) — نیازی به clang/GMP نیست.
- خروجی اسکریپتی: `mc -e "2^3^2"` → `ans = 64` (کد خروج ۰/۱).

## بوت‌استرپ محلی (اولین بار — مسیر Flutter/APK روی رایانه)

پیش‌نیازها: Rust (پین‌شده با `rust-toolchain.toml`)، Flutter 3.24+، اندروید SDK، `cargo-ndk` و `flutter_rust_bridge_codegen`.

```bash
git clone https://github.com/mathcraft/mathcraft && cd mathcraft
./scripts/bootstrap.sh        # flutter create + codegen FRB + pub get + verify + تم لانچ CDS
./scripts/build_rust_android.sh   # cargo-ndk → libmathcraft.so (arm64-v8a, armeabi-v7a)
cd app && flutter run --release
```

**⚠️ مهم — بیلد release، نه debug (بازخورد P0.2):**
`flutter build apk` یا `flutter run` **بدون فلگ** بیلد debug می‌سازد: حجم ~۱۵۰MB،
استارتاپ ~۵ ثانیه با صفحه سیاه، و لگ در سوییچ تم/تب. این طبیعت debug است، نه باگ برنامه.
همیشه `--release` بزنید — APK release انتظاری ~۲۵-۴۰MB و استارتاپ زیر ۱.۵ ثانیه.
برای نصب نهایی: `flutter build apk --release --split-per-abi` (خروجی per-ABI سبک‌تر).
تم لانچ تیره CDS (#101418) هم اکنون توسط bootstrap از `scripts/android-launch-theme/`
نصب می‌شود تا حتی قبل از اولین فریم، «حالت مدرن» دیده شود (سند ۰۶ §2.1).

## توسعه روزمره

```bash
cargo test --workspace                              # موتور
cargo clippy --workspace --all-targets -- -D warnings   # گیت CI
cd app && flutter test                              # UI (با FakeEngineClient — بدون .so)
cd app && flutter analyze
flutter_rust_bridge_codegen generate                # بعد از هر تغییر mc-ffi/src/api.rs
```

## قواعد غیرقابل‌مذاکره (سند ۰۰ بخش ۴)

1. آفلاین مطلق — حتی مجوز `INTERNET` نمی‌گیریم.
2. هرگز UI را بلاک نکن — همه محاسبه روی thread pool رست.
3. صحت قبل از سرعت — Golden Tests از روز اول.
4. حس نرم‌افزار واقعی — ایموجی هرگز آیکون نیست؛ فقط Craft Icons (سند ۰۶ §2.5، گیت merge).
5. شکست صادقانه — هرگز نتیجه اشتباهِ مطمئن‌نما.

## مجوز

کد: **GPL-3.0-or-later** (تصمیم D-08 — هم‌راستا با GNU Octave) — فایل [`LICENSE`](LICENSE).
مستندات `docs/`: CC BY-SA 4.0.
MathCraft محصولی مستقل است و به MathWorks/MATLAB وابستگی ندارد (سند ۰۱ بخش ۱۰).
