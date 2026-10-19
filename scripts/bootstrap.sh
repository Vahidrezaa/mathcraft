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
fi

echo "== ۲/۴+ — تنظیمات مخصوص Termux (سند ۰۹ بخش ۷) =="
if [ -d android ] && [ -n "${PREFIX:-}" ] && [[ "$PREFIX" == *com.termux* ]]; then
  # ndkVersion قالب → NDK aarch64 نصب‌شده توسط setup-termux-flutter.sh
  if [ -f android/app/build.gradle.kts ]; then
    sed -i -E 's/ndkVersion = "[^"]*"/ndkVersion = "29.0.14206865"/' android/app/build.gradle.kts || true
  fi
  if [ -f android/app/build.gradle ]; then
    sed -i -E 's/ndkVersion "[^"]*"/ndkVersion "29.0.14206865"/' android/app/build.gradle || true
  fi
  # حافظهٔ گوشی (سند ۰۹ بخش ۵) — heap محدود به‌جای پیش‌فرض دسکتاپ
  if [ -f android/gradle.properties ]; then
    sed -i 's/^org.gradle.jvmargs=.*/org.gradle.jvmargs=-Xmx1536m -XX:MaxMetaspaceSize=512m/' android/gradle.properties || true
  fi
  # aapt2 بومی aarch64 (اگر setup-termux-flutter.sh اجرا شده باشد)
  if [ -n "${ANDROID_HOME:-}" ] && [ -x "$ANDROID_HOME/aapt2/aapt2" ] && [ -f android/gradle.properties ]; then
    grep -q "aapt2FromMavenOverride" android/gradle.properties || \
      echo "android.aapt2FromMavenOverride=$ANDROID_HOME/aapt2/aapt2" >> android/gradle.properties
  fi
  echo "Termux tuning: ndkVersion=29.0.14206865, gradle heap 1536m, arm64 aapt2 (اگر موجود)"
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
