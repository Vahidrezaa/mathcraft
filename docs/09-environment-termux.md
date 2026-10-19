# سند ۰۹ — محیط توسعه بومی Termux: نصب Flutter، Android SDK و NDK روی خود گوشی

> **وضعیت:** رسمی (راستی‌آزمایی‌شده با منابع فعال تا اکتبر ۲۰۲۶)
> **مخاطب:** توسعه‌دهنده‌ای که فقط گوشی اندرویدی و Termux دارد
> **پیش‌نیاز:** Rust و git و cargo (نصب‌شده)، سند ۰۲ بخش ۸ (بسته‌بندی)، سند ۰۷ (فاز P0)

---

## ۱. واقعیت‌های بستره — چرا «نصب Flutter روی ترموکس» رسمی نیست

گوگل و تیم Flutter به‌طور رسمی فقط میزبان‌های x86_64 (لینوکس/ویندوز/مک) را پشتیبانی می‌کنند. گوشی اندرویدی با معماری aarch64 از سه جهت با مانع روبه‌روست و شناخت دقیق این سه مانع، پیش‌شرط هر تصمیمی در این سند است:

1. **خودِ Flutter SDK:** باینری‌های رسمی میزبان برای aarch64 اندروید منتشر نمی‌شود. راه‌حل، نسخه‌های جامعهٔ کاربری است که Dart SDK و ابزار `flutter` را برای اجرای بومی در Termux بازسازی کرده‌اند.
2. **Android SDK Build-Tools:** ابزارهای بومی مثل `aapt2` و `zipalign` فقط به‌صورت x86_64 منتشر می‌شوند. روی گوشی، نسخه‌های aarch64 بازسازی‌شدهٔ جامعه جایگزین می‌شوند.
3. **Android NDK:** ساپورت رسمی میزبان لینوکس فقط x86_64 است. برای کامپایل Rust به اندروید (سند ۰۲ بخش ۸: تولید `libmathcraft.so`) به NDK بازسازی‌شدهٔ aarch64 نیاز داریم.

نکتهٔ مهم: این موانع همگی «سطح ابزار» هستند، نه «سطح معماری». موتور MathCraft (۸ کرِیت Rust) کاملاً pure-Rust است و در P0.1 بدون هیچ‌کدام از این‌ها روی گوشی تست و بیلد می‌شود (سند ۰۷ مسیر بومی Termux). یعنی **هستهٔ پروژه از امروز بدون هیچ پیش‌نیاز اضافه‌ای زنده است**؛ این سند فقط لایهٔ Flutter/APK را باز می‌کند.

---

## ۲. چهار مسیر ممکن — تصمیم‌نامه

| مسیر | توضیح | سرعت | سختی | کی استفاده کنیم |
|---|---|---|---|---|
| **A. بومی Termux** (پیشنهادی) | Flutter بستهٔ Termux + NDK/SDK بازسازی‌شدهٔ aarch64، همه‌چیز در `$HOME` | متوسط | متوسط — با اسکریپت ما خودکار | دستگاه اصلی توسعه؛ همیشه در دسترس |
| B. proot-distro اوبونتو | اوبونتوی ARM64 با proot روی هستهٔ اندروید | کند (سربار ptrace روی هر syscall) | کم | اگر مسیر A روی دستگاه شما به مشکل خورد |
| C. AVF (محیط لینوکس اندروید ۱۵+) | ماشین مجازی Debian واقعی با هستهٔ مستقل (Developer options ← Linux Development Environment) | سریع‌ترین | کم، ولی وابسته به دستگاه (اندروید ۱۵+ و ساپورت pKVM) | اگر دستگاه شما ساپورت می‌کند، برای بیلدهای سنگین عالی است |
| D. GitHub Actions CI | بیلد APK روی سرور ابری با هر push | بی‌نهایت (از دید گوشی) | کم | **همیشه فعال می‌ماند** — لنگر بیلد تکرارپذیر release و F-Droid (سند ۰۸ بخش ۷) |

**تصمیم (به‌روز — از وقتی که پوش به GitHub در دسترس است):** مسیر **D خطِ اصلی برای همهٔ پردازش سنگین است**: بیلد APK، تست کامل و release از روی هر push یا دستور دستی با `gh` — از ترمینال خود گوشی (بخش ۸). مسیر A محیط توسعه/تست سریع موتور روی گوشی می‌ماند (cargo test و ویرایش کد — بدون نیاز به هیچ نصب جدیدی)؛ B و C فقط fallback. این ترکیب با معماری CI سند ۰۲ بخش ۸ و استراتژی reproducible build سند ۰۸ بخش ۷ هم‌راستاست.

---

## ۳. قطعات مسیر A — منابع دقیق (نسخه‌پین‌شده)

| قطعه | منبع | نسخه / فایل | حجم | نقش |
|---|---|---|---|---|
| Flutter SDK | ریپوی `termuxvoid/repo` (APT شخص ثالث؛ مبتنی بر `mumumusuc/termux-flutter`) | Flutter 3.41.9 — `pkg install flutter` | ~۴۵۰MB | ابزار `flutter`، Dart SDK بومی aarch64 |
| Android NDK | ریپوی `lzhiyong/termux-ndk`، ریلیز `r29` (ژانویه ۲۰۲۶) | `android-ndk-r29-aarch64.tar.xz` (clang 21، فقط aarch64، اندروید ۹+) | ۳۴۴MB فشرده | کامپایل Rust → `libmathcraft.so` |
| SDK Build-Tools | همان ریپو، ریلیز `android-sdk` | `android-sdk-aarch64.tar.xz` — شامل build-tools 35.0.0، platform-tools 35.0.2، cmake 4.2.1، ninja 1.13.2 — همهٔ باینری‌های بومی aarch64 | ۲۷۲MB فشرده | `aapt2`، `zipalign`، `apksigner` و بقیهٔ ابزار بسته‌بندی |
| aapt2 مستقل (جایگزین/پشتیبان) | ریپوی `ReVanced/aapt2`، ریلیز `v1.1.0` | `aapt2-arm64-v8a` (~۴.۵MB) | ۴.۵MB | override برای Gradle: `android.aapt2FromMavenOverride` |
| Java | مخزن رسمی Termux | `pkg install openjdk-17` | ~۲۰۰MB | اجرای Gradle، sdkmanager، R8/d8 (همگی جاوا و مستقل از معماری) |
| Platform android.jar | سرور رسمی گوگل | `cmdline-tools` (جاوا) + `sdkmanager 'platforms;android-35'` | ~۱۵۰MB | `android.jar` دادهٔ خالص است و معماری ندارد |
| Rust targets | rustup | `aarch64-linux-android` + `armv7-linux-androideabi` | ~۴۰۰MB | cross-compile کرِیت mc-ffi |

نکته دربارهٔ نسخهٔ build-tools: بازسازی aarch64 فقط تا سری 35.0.x موجود است (آخرین: 35.0.2 در ریپوی `lzhiyong/android-sdk-tools`). اگر قالب پروژهٔ Flutter نسخهٔ جدیدتری از AGP خواست که build-tools 36 طلب کند، در `android/app/build.gradle.kts` مقدار `buildToolsVersion = "35.0.2"` و نسخهٔ AGP سازگار (۸.۹ یا پایین‌تر) پین می‌شود. اسکریپت ما این را چک و راهنمایی می‌کند.

---

## ۴. نصب گام‌به‌گام — منبع حقیقت: `scripts/setup-termux-flutter.sh`

کل مسیر A در اسکریپت `scripts/setup-termux-flutter.sh` خودکار شده است (idempotent — اجرای مکرر بی‌ضرر است و دانلودها با SHA256 صحه‌گذاری می‌شوند). خلاصهٔ گام‌ها برای فهم دستی:

1. **پیش‌شرط‌ها:** داخل Termux، معماری aarch64، حداقل ~۸GB فضای آزاد در `$HOME` (مجموع فشرده+باز‌شدهٔ همهٔ قطعات + `target/` Rust + کش Gradle/pub). مسیر نصب باید حتماً `$HOME` باشد — حافظهٔ اشتراکی `/sdcard` مجوز اجرا (noexec) ندارد.
2. **جاوا:** `pkg install openjdk-17` — از مخزن رسمی.
3. **Flutter:** افزودن ریپوی termuxvoid (اسکریپت رسمی نصب آن) و سپس `pkg install flutter`. راستی‌آزمایی با `flutter --version`. اگر این مسیر روی دستگاه شما شکست خورد: فایل `.deb` مستقیم از `mumumusuc/termux-flutter` (نسخهٔ 3.29.2) با `apt install ./flutter.deb` — و اگر آن هم خواستید از ریسک سازگاری دور بمانید: مسیر B یا C جدول بخش ۲.
4. **NDK r29** در `$ANDROID_HOME/ndk/29.0.14206865` + **SDK bundle** (build-tools/platform-tools/cmake/ninja) در `$ANDROID_HOME`.
5. **cmdline-tools رسمی** (جاوا — مستقل از معماری) + صدور مجوز لایسنس‌ها + نصب `platforms;android-35` (و ۳۶ برای پیش‌دستی روی targetSdk قالب‌های جدید).
6. **aapt2 سرریزشده:** دانلود باینری ReVanced و ثبت `android.aapt2FromMavenOverride` در `~/.gradle/gradle.properties` (سراسری؛ Gradle هر پروژه‌ای را رعایت می‌کند).
7. **تارگت‌های Rust:** `rustup target add aarch64-linux-android armv7-linux-androideabi`. توجه: اگر Rust شما از `pkg install rust` باشد (بدون rustup)، cross-compile به اندروید ممکن نیست — برای لایهٔ APK باید rustup با پروفایل minimal نصب شود (اسکریپت چک می‌کند).
8. **متغیرهای محیطی ماندگار** در `~/.profile` و `~/.bashrc`: `ANDROID_HOME`، `ANDROID_NDK_HOME`، افزودن `cmdline-tools/latest/bin` و `platform-tools` به PATH.
9. **اتصال به Flutter:** `flutter config --android-sdk $ANDROID_HOME` و سپس `flutter doctor -v` — انتظار: ✓ Flutter، ✓ Android toolchain (هشدارهای غیرحیاتی مثل android-licenses پس از گام ۵ صفر می‌شوند).
10. **ابزارهای سنگین‌تر** (اختیاری، هرکدام چند دقیقه کامپایل روی گوشی): `cargo install cargo-ndk` (بیلد `libmathcraft.so`) و `cargo install flutter_rust_bridge_codegen` (codegen قرارداد) — `scripts/bootstrap.sh` در صورت نبود، خودش نصبشان می‌کند.

---

## ۵. بودجه‌ها و انتظارات واقع‌بینانه روی سخت‌افزار گوشی

| محدودیت | مقدار | توضیح |
|---|---|---|
| فضا | ≥ ۸GB آزاد | Flutter ~۱.۵GB + NDK ~۲GB + SDK ~۱GB + جاوا/گریدل/pub cache ~۱GB + `target/` پروژه ~۱-۲GB |
| RAM | ≥ ۴GB توصیه می‌شود | کامپایل موازی Rust با `-j2` (اسکریپت‌های پروژه از `RUSTFLAGS=-Ccodegen-units=1` و LTO در dev صرف‌نظر می‌کنند)؛ Gradle با heap محدود 1.5GB (`org.gradle.jvmargs` توسط bootstrap.sh روی Termux تنظیم می‌شود) |
| زمان — cargo test کامل | ~۱-۳ دقیقه | ۸ کرِیت P0.1، بدون وابستگی سنگین |
| زمان — اولین `flutter build apk --debug` | ~۱۵-۴۰ دقیقه | دانلود Gradle+وابستگی‌ها + Dart compile + packaging؛ دفعات بعد ۵-۱۵ دقیقه |
| زمان — `cargo ndk` بیلد release | ~۵-۱۵ دقیقه | با LTO؛ در چرخهٔ توسعه از `--debug` استفاده کنید |

توصیهٔ عملیاتی: بیلدهای release و golden را به CI (مسیر D) بسپارید؛ روی گوشی فقط debug + test. این دقیقاً همان تفکیک نقش‌هایی است که سند ۰۸ برای CI تعریف کرده است.

---

## ۶. راستی‌آزمایی — چک‌لیست سبز

ترتیب اجرا بعد از `scripts/setup-termux-flutter.sh`:

```bash
java -version                      # 17.x
flutter --version                  # 3.41.x
flutter doctor -v                  # [✓] Flutter + [✓] Android toolchain
ls $ANDROID_HOME/ndk/29.0.14206865/toolchains/llvm/prebuilt/*/bin/clang  # موجود
aapt2 version                      # via ANDROID_HOME/aapt2/aapt2
rustup target list --installed     # شامل aarch64-linux-android
cargo test --workspace             # ۵۵+ تست سبز (موتور، سند ۰۷)
bash scripts/bootstrap.sh          # flutter create + FRB codegen + flutter test
bash scripts/build_rust_android.sh # libmathcraft.so → jniLibs
cd app && flutter build apk --debug
```

اگر به‌جای بیلد محلی از Actions استفاده می‌کنی (بخش ۸)، چک‌لیست به این ساده می‌شود:

```bash
cargo test --workspace             # موتور — روی گوشی
bash scripts/gh-apk.sh trigger && bash scripts/gh-apk.sh watch
bash scripts/gh-apk.sh download    # APK در dist/
```

خروجی نهایی: `app/build/app/outputs/flutter-apk/app-debug.apk` — نصب با `pkg install apkeep` یا کپی به حافظه و نصب دستی (Termux بدون root نمی‌تواند مستقیم adb install کند؛ APK را با file manager باز کنید).

---

## ۷. ریسک‌های شناخته‌شده و مسیر بازگشت

1. **سازگاری بستهٔ Flutter با نسخهٔ اندروید:** یکی از بیلدهای قدیمی جامعه (`mumumusuc` نسخه‌های قدیم) روی اندروید ۱۴ مشکل گزارش شده است؛ بستهٔ termuxvoid (3.41.9، مه ۲۰۲۶) برای نسخه‌های جدیدتر است. اگر روی دستگاه شما flutter بومی بالا نیامد: مسیر B (proot) یا C (AVF). اسکریپت ما خطای شکست را صریح گزارش می‌کند و پیش از هر دانلود سنگین متوقف می‌شود.
2. **نسخهٔ build-tools:** سقف aarch64 فعلی 35.0.x است (بخش ۳). اگر قالب پروژه build-tools 36 خواست: پین کردن `buildToolsVersion` + نسخهٔ AGP (راهنمای خطای Gradle دقیقاً همین را می‌گوید).
3. **ndkVersion قالب Flutter:** قالب‌های جدید نسخهٔ NDK را پین می‌کنند (مثلاً 27.x)؛ `bootstrap.sh` روی Termux این مقدار را به `29.0.14206865` تغییر می‌دهد تا با NDK نصب‌شده بخواند.
4. **کمبود حافظه در بیلدهای بزرگ:** اگر Gradle یا rustc کشته شد (OOM): کاهش `-j`، بستن اپ‌های پس‌زمینه، و برای Gradle کاهش بیشتر `org.gradle.jvmargs`. بیلد release کامل را به CI منتقل کنید.
5. **تغییر لینک‌های دانلود جامعه:** ریلیزهای GitHub ممکن است جابه‌جا شوند؛ اسکریپت نسخه‌ها را پین کرده و SHA256 صحه‌گذاری می‌کند — اگر شکست خورد، به فهرست ریلیز همان ریپو سر بزنید و متغیرهای بالای اسکریپت را به‌روز کنید (تغییر باید اول در همین سند ثبت شود — قاعدهٔ طلایی).
6. **F-Droid در آینده:** بیلد نهایی انتشار همیشه از CI تکرارپذیر می‌آید (سند ۰۲ بخش ۸ و ۰۸ بخش ۷)؛ محیط گوشی صرفاً محیط توسعه است و خروجی آن هرگز مستقیماً منتشر نمی‌شود.

---

## ۸. خط تولید GitHub Actions (مسیر D — اصلی برای APK و پردازش سنگین)

وقتی پروژه روی GitHub است، پردازش سنگین به رانرهای x86_64 ابری منتقل می‌شود و گوشی فقط ایستگاه ویرایش/تست موتور می‌ماند. دو وورک‌فلو:

| وورک‌فلو | فایل | تریگر | خروجی |
|---|---|---|---|
| CI | `.github/workflows/ci.yml` | هر push به main / PR | fmt + clippy + test + deny + FRB codegen + flutter analyze/test (گیت‌های سند ۰۸) |
| Android | `.github/workflows/android.yml` | push به main (مسیرهای مرتبط)، `workflow_dispatch`، تگ `v*` | APK debug (artifact) یا APK release (artifact + GitHub Release روی تگ) |

نکته‌های طراحی:
- وورک‌فلو Android کل خط لولهٔ سند ۰۲ بخش ۸ را از طریق `scripts/ci-build-apk.sh` اجرا می‌کند — همان ترتیب بوت‌استرپ توسعه‌دهنده (flutter create ← پچ minSdk 24 و ndkVersion ← pub get ← codegen FRB ← cargo-ndk .so ← build apk). یعنی CI آینهٔ دقیق جریان توسعه است، نه یک مسیر موازی متفاوت.
- `app/android/` و `app/lib/src/rust/` در ریپو ذخیره نمی‌شوند (gitignore) — CI هر بار از flutter create و codegen تولیدشان می‌کند؛ هیچ تناقضی با نسخهٔ محلی پیش نمی‌آید.
- Toolchain Rust پین 1.99.1 است (rust-toolchain.toml، سند ۰۸ §7)؛ Flutter موقتاً channel stable — قبل از اولین release عمومی، نسخهٔ دقیق پین می‌شود تا بیلد تکرارپذیر F-Droid کامل شود.
- امضا: تا وقتی keystore انتشار در CI secrets قرار نگیرد (سند ۰۲ بخش ۸)، APKهای release با امضای debug بسته می‌شوند — برای تست نصب کافی‌اند، برای انتشار نه.
- پوشهٔ `dist/` (خروجی دانلود `gh-apk.sh`) در `.gitignore` است — APK دانلودشده هرگز کامیت نمی‌شود؛ قبل از اولین کامیت، خط `/dist/` باید در `.gitignore` باشد.
- وابستگی‌های codegen FRB (درس اولین ران CI، اکتبر ۲۰۲۶): FRB کد Dart را با `freezed` تولید می‌کند — نبود `freezed_annotation`/`freezed`/`build_runner` در pubspec = خطای `MissingDep` و شکست هر دو وورک‌فلو؛ بعد از codegen هم گام `dart run build_runner build` اجباری است (سند ۰۲ بخش ۳ بند ۵).
- دو درس ران دوم (P0.2.3): (۱) `build_runner` همهٔ فایل‌های Dart پکیج را parse می‌کند — هر خطای گرامری حتی در فایل‌های دست‌نویس (مثل export بعد از declarations در engine_client.dart) کل بیلد را می‌شکند؛ فایل‌های Dart در پوش قبل از push باید حداقل یک‌بار analyze شده باشند. (۲) فلگ `--delete-conflicting-outputs` در build_runnerهای جدید حذف شده (فقط warning) — استفاده نمی‌شود.
- درس ران سوم (P0.2.4 — کل خط تولید این‌بار در sandbox واقعاً اجرا و سبز شد: cargo test ۵۵ سبز، codegen، build_runner، analyze صفر، flutter test ۴ سبز، بیلد host مسیر .so): (۱) `-p` در cargo-ndk یعنی platform/API-level نه package — انتخاب پکیج با `--manifest-path crates/mc-ffi/Cargo.toml`؛ (۲) تزریق `mod frb_generated;` پیش‌فرض FRB بالای کامنت‌های داخلی lib.rs می‌رود و E0753 می‌دهد — خط به انتهای فایل منتقل شد و `frb_generated.rs` کامیت می‌شود (با allow سطح ماژول برای lint های سخت‌گیرانه — allow در lib.rs می‌ماند و با regenerate نمی‌پرد)؛ (۳) کدک فهرست u64 در FRB 2.13.0 با تحلیل‌گر Dart 3.13 خطای type می‌دهد — قرارداد به u32 رفت (سند ۰۲ §3 بند ۵، سند ۰۴ §8.1؛ نسخهٔ قرارداد v2)؛ (۴) جفت پکیج Dart `flutter_rust_bridge` و codegen دقیقاً هم‌نسخه (۲.۱۳.۰) پین شد؛ (۵) دو باگ UI واقعی که تست ویجت گرفت: Row با crossAxisAlignment stretch در ارتفاع بی‌کران ListView می‌شکند (نوار خطا شد border چپ) و انتخاب زبان پیام خطا با Directionality به‌جای locale در جزیرهٔ LTR همیشه انگلیسی می‌داد. تنها گام راستی‌آزمایی‌نشدهٔ محلی: خود gradle/NDK اندروید (نیازمند ~۵GB — در CI رانرهای حرفه‌ای انجام می‌شود).

رابط ترمینالی روی گوشی (پیش‌نیاز: `pkg install gh jq`، یک‌بار `gh auth login`):

```bash
# یک‌بار — کامیت اولیه سپس ساخت ریپو و پوش (gh --push با ریپوی بدون کامیت شکست می‌خورد):
cd ~/mathcraft
git add -A && git commit -m "MathCraft P0.2 — initial import"
git branch -m main                 # وورک‌فلوها روی push به main تریگر می‌شوند
gh repo create mathcraft --public --source=. --remote=origin --push
# عمومی = Actions رایگان نامحدود (خصوصی: سهمیهٔ ~۲۰۰۰ دقیقه/ماه)؛ پروژه GPL-3.0 است و انتشار عمومی طراحی شده — تغییر بعدی: Settings > Danger Zone

# چرخهٔ روزمره:
bash scripts/gh-apk.sh trigger     # اجرای دستی وورک‌فلو android (یا فقط push کن)
bash scripts/gh-apk.sh watch       # پیگیری زندهٔ ران در ترمینال گوشی
bash scripts/gh-apk.sh download    # آخرین APK موفق → dist/
git tag v0.1.0-p0 && git push origin v0.1.0-p0   # release خودکار با تگ
```

زمان انتظار واقع‌بینانه: ران کامل Android با کش گرم ~۱۰-۲۰ دقیقه، بار اول (بدون کش) ~۲۵-۴۰ دقیقه.

---

## ۹. پیوند با سایر اسناد

- سند ۰۲ بخش ۸ — خط لولهٔ بسته‌بندی رسمی (cargo-ndk، ABIها، بودجهٔ ۴۵MB، امضا): این سند فقط «محیط اجرای آن خط لوله روی گوشی» را مشخص می‌کند، نه خود خط لوله را.
- سند ۰۷ — فاز P0: مسیر بومی Termux (P0.1 بدون Flutter) و بند end-to-end روی دستگاه واقعی (P0.2 با این سند).
- سند ۰۸ بخش ۷ — CI و بیلد تکرارپذیر: نقش لنگر (مسیر D جدول بخش ۲).
- `scripts/setup-termux.sh` — راه‌اندازی موتور Rust و کنسول `mc` (بدون Flutter)؛ `scripts/setup-termux-flutter.sh` — لایهٔ Flutter/NDK این سند؛ `scripts/bootstrap.sh` — flutter create + codegen + پچ‌های مخصوص Termux.
