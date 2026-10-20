#!/usr/bin/env bash
# MathCraft — بیلد موتور Rust برای اندروید (سند ۰۲ بخش ۸)
# cargo-ndk → libmathcraft.so → jniLibs پروژه Flutter
set -euo pipefail
cd "$(dirname "$0")/.."

command -v cargo-ndk >/dev/null || {
  echo "cargo-ndk نصب نیست — نصب می‌کنم…"
  cargo install cargo-ndk --locked
}

rustup target add aarch64-linux-android armv7-linux-androideabi

OUT="app/android/app/src/main/jniLibs"
mkdir -p "$OUT/arm64-v8a" "$OUT/armeabi-v7a"

echo "== build (release) =="
cargo ndk \
  -t aarch64-linux-android \
  -t armv7-linux-androideabi \
  -p mc-ffi \
  build --release

cp target/aarch64-linux-android/release/libmathcraft.so "$OUT/arm64-v8a/"
cp target/armv7-linux-androideabi/release/libmathcraft.so "$OUT/armeabi-v7a/"

echo "libmathcraft.so برای arm64-v8a و armeabi-v7a ساخته شد → $OUT"
echo
echo "== راهنمای حجم APK (بازخورد P0.2 — APK ۱۴۸MB) =="
echo "هشدار: flutter build apk بدون فلگ = بیلد DEBUG (حجم ۳-۵ برابر؛ ~۱۵۰MB)."
echo "همیشه --release بزنید؛ APK release انتظاری: ~۲۵-۴۰MB."
echo "الف) APK تکی release برای همه دستگاه‌ها:"
echo "     cd app && flutter build apk --release"
echo "ب) APK جدا per-ABI — حجم نصب باز هم کمتر؛ توصیه‌شده:"
echo "     cd app && flutter build apk --release --split-per-abi"
echo "     خروجی: build/app/outputs/flutter-apk/app-arm64-v8a-release.apk"
echo "ج) فشرده‌سازی انتقال: flutter build appbundle برای Play (توزیع per-device)."
echo "حالا در app/: flutter run"
