#!/usr/bin/env bash
# MathCraft — بوت‌استرپ اولیه محلی (سند ۰۷ P0)
# flutter create پلتفرم‌ها + FRB codegen + pub get + verify
set -euo pipefail
cd "$(dirname "$0")/.."

command -v flutter >/dev/null || { echo "ERROR: flutter in PATH نیست"; exit 1; }
command -v cargo >/dev/null || { echo "ERROR: cargo in PATH نیست"; exit 1; }

echo "== ۱/۴ — toolchain پین‌شده Rust =="
rustup show active-toolchain >/dev/null 2>&1 || rustup toolchain install

echo "== ۲/۴ — اسکلت پلتفرم Flutter (فقط بار اول) =="
cd app
if [ ! -d android ]; then
  flutter create --platforms=android --org io.mathcraft --project-name mathcraft .
  # minSdk 24 (سند ۰۲ بخش ۸) — در build.gradle.kts جدید یا build.gradle قدیمی
  if [ -f android/app/build.gradle.kts ]; then
    sed -i 's/minSdk = flutter.minSdkVersion/minSdk = 24/' android/app/build.gradle.kts || true
  fi
  if [ -f android/app/build.gradle ]; then
    sed -i 's/minSdkVersion flutter.minSdkVersion/minSdkVersion 24/' android/app/build.gradle || true
  fi
  # تست پیش‌فرض flutter create با کد ما هم‌خوان نیست — تست واقعی: test/console_demo_test.dart
  rm -f test/widget_test.dart
  # تم لانچ CDS (بازخورد P0.2: رفع «صفحه سیاه» استارتاپ — سند ۰۶ §2.1):
  # قالب‌های tracked در scripts/android-launch-theme/ روی res تولیدشده کپی می‌شوند
  # تا اولین فریم با پس‌زمینه تیره #101418 نشان داده شود، بدون فلش سفید/خاکستری.
  cp -r ../scripts/android-launch-theme/. android/app/src/main/res/
fi
flutter pub get

echo "== ۳/۴ — کد تولیدی flutter_rust_bridge =="
command -v flutter_rust_bridge_codegen >/dev/null || {
  echo "flutter_rust_bridge_codegen نصب نیست — نصب می‌کنم (چند دقیقه)…"
  cargo install flutter_rust_bridge_codegen --locked
}
flutter_rust_bridge_codegen generate

echo "== ۴/۴ — تحلیل و تست UI (با FakeEngineClient، بدون .so) =="
flutter analyze --fatal-infos
flutter test

echo
echo "Bootstrap کامل شد. برای اجرای روی دستگاه: ./scripts/build_rust_android.sh"
