#!/usr/bin/env bash
# MathCraft -- full Android APK pipeline for CI and desktop Linux (doc 09 s8).
# Mirrors the developer bootstrap flow, in CI order:
#   platform skeleton (flutter create) -> minSdk/ndkVersion patches -> pub get
#   -> FRB codegen -> build_runner (.freezed.dart) -> cargo-ndk .so -> flutter build apk
# Usage:  APK_MODE=debug|release bash scripts/ci-build-apk.sh    (default: debug)
set -euo pipefail
cd "$(dirname "$0")/.."

APK_MODE="${APK_MODE:-${1:-debug}}"
case "$APK_MODE" in debug|release) ;; *) echo "APK_MODE must be debug|release"; exit 1 ;; esac
command -v flutter >/dev/null || { echo "flutter in PATH نیست"; exit 1; }

echo "== 1/6 platform skeleton (app/android) =="
cd app
if [ ! -d android ]; then
    flutter create --platforms=android --org io.mathcraft --project-name mathcraft .
    rm -f test/widget_test.dart
fi
# minSdk 24 (doc 02 s8) — patch BOTH template styles:
#   Groovy (flutter 3.24.x build.gradle):  `minSdk = flutter.minSdkVersion`
#   older Groovy:                          `minSdkVersion flutter.minSdkVersion`
#   Kotlin DSL (build.gradle.kts):         `minSdk = flutter.minSdkVersion`
for MINSDK_F in android/app/build.gradle android/app/build.gradle.kts; do
    [ -f "$MINSDK_F" ] || continue
    sed -i -e 's/minSdk = flutter.minSdkVersion/minSdk = 24/' \
           -e 's/minSdkVersion flutter.minSdkVersion/minSdkVersion 24/' "$MINSDK_F" || true
done
# guard (pipefail-proof): NO pipes, and only inspects files that exist.
# (A previous pipe version false-failed under `set -o pipefail`: grep exits 2
#  when build.gradle.kts is absent, even with matches from build.gradle.)
MINSDK_PATCHED=0
for MINSDK_F in android/app/build.gradle android/app/build.gradle.kts; do
    [ -f "$MINSDK_F" ] || continue
    if grep -qE 'minSdk(Version)?[[:space:]]*=[[:space:]]*24|minSdkVersion[[:space:]]+24' "$MINSDK_F"; then
        MINSDK_PATCHED=1
    fi
done
if [ "$MINSDK_PATCHED" -eq 0 ]; then
    echo "::error::minSdk 24 patch failed - template syntax changed? Evidence:"
    echo "::error::flutter: $(flutter --version 2>/dev/null | head -1 || echo unknown)"
    for MINSDK_F in android/app/build.gradle android/app/build.gradle.kts; do
        [ -f "$MINSDK_F" ] || continue
        echo "::error::--- $MINSDK_F ---"
        grep -n 'minSdk' "$MINSDK_F" || true
    done
    exit 1
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

echo "== 2/6 pub get =="
flutter pub get

echo "== 3/6 FRB codegen (قرارداد mc-ffi -- سند ۰۲ بخش ۳) =="
command -v flutter_rust_bridge_codegen >/dev/null || cargo install flutter_rust_bridge_codegen --version 2.12.0 --locked
flutter_rust_bridge_codegen generate

echo "== 4/6 build_runner (.freezed.dart — سند ۰۲ بخش ۳ بند ۵) =="
dart run build_runner build

echo "== 5/6 engine .so (cargo-ndk -> libmathcraft.so -- سند ۰۲ بخش ۸) =="
cd ..
bash scripts/build_rust_android.sh

echo "== 6/6 flutter build apk ($APK_MODE) =="
cd app
if [ "$APK_MODE" = release ]; then
    flutter build apk --release --split-per-abi
else
    flutter build apk --debug
fi
find build/app/outputs -name '*.apk' -print
echo "APK build done"
