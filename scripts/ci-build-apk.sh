#!/usr/bin/env bash
# MathCraft -- full Android APK pipeline for CI and desktop Linux (doc 09 s8).
# Mirrors the developer bootstrap flow, in CI order:
#   platform skeleton (flutter create) -> minSdk/ndkVersion patches -> pub get
#   -> FRB codegen -> cargo-ndk .so -> flutter build apk
# Usage:  APK_MODE=debug|release bash scripts/ci-build-apk.sh    (default: debug)
set -euo pipefail
cd "$(dirname "$0")/.."

APK_MODE="${APK_MODE:-${1:-debug}}"
case "$APK_MODE" in debug|release) ;; *) echo "APK_MODE must be debug|release"; exit 1 ;; esac
command -v flutter >/dev/null || { echo "flutter in PATH نیست"; exit 1; }

echo "== 1/5 platform skeleton (app/android) =="
cd app
if [ ! -d android ]; then
    flutter create --platforms=android --org io.mathcraft --project-name mathcraft .
    rm -f test/widget_test.dart
fi
# minSdk 24 (doc 02 s8)
if [ -f android/app/build.gradle.kts ]; then
    sed -i 's/minSdk = flutter.minSdkVersion/minSdk = 24/' android/app/build.gradle.kts || true
fi
if [ -f android/app/build.gradle ]; then
    sed -i 's/minSdkVersion flutter.minSdkVersion/minSdkVersion 24/' android/app/build.gradle || true
fi
# ndkVersion -> NDK نصب‌شده (CI/Termux)؛ روی دسکتاپ بدون متغیر، پیش‌فرض قالب می‌ماند
if [ -n "${ANDROID_NDK_HOME:-}" ]; then
    NDKV="$(basename "$ANDROID_NDK_HOME")"
    if [ -f android/app/build.gradle.kts ]; then
        sed -i -E "s/ndkVersion = \"[^\"]*\"/ndkVersion = \"$NDKV\"/" android/app/build.gradle.kts || true
    fi
    if [ -f android/app/build.gradle ]; then
        sed -i -E "s/ndkVersion \"[^\"]*\"/ndkVersion \"$NDKV\"/" android/app/build.gradle || true
    fi
fi

echo "== 2/5 pub get =="
flutter pub get

echo "== 3/5 FRB codegen (قرارداد mc-ffi -- سند ۰۲ بخش ۳) =="
command -v flutter_rust_bridge_codegen >/dev/null || cargo install flutter_rust_bridge_codegen --locked
flutter_rust_bridge_codegen generate

echo "== 4/5 engine .so (cargo-ndk -> libmathcraft.so -- سند ۰۲ بخش ۸) =="
cd ..
bash scripts/build_rust_android.sh

echo "== 5/5 flutter build apk ($APK_MODE) =="
cd app
if [ "$APK_MODE" = release ]; then
    flutter build apk --release --split-per-abi
else
    flutter build apk --debug
fi
find build/app/outputs -name '*.apk' -print
echo "APK build done"
