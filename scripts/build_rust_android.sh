#!/usr/bin/env bash
# MathCraft — بیلد موتور Rust برای اندروید (سند ۰۲ بخش ۸)
# cargo-ndk → libmathcraft.so → jniLibs پروژه Flutter
set -euo pipefail
cd "$(dirname "$0")/.."

command -v cargo-ndk >/dev/null || {
  echo "cargo-ndk نصب نیست — نصب می‌کنم (پین 4.1.2 — سمانتیک v4 در پایین)…"
  cargo install cargo-ndk --version 4.1.2 --locked
}

rustup target add aarch64-linux-android armv7-linux-androideabi

OUT="app/android/app/src/main/jniLibs"

echo "== build (release) =="
# cargo-ndk >= 4 semantics (was the CI breakage: `-p` no longer exists for
# cargo-ndk and -t takes Android ABI names, not Rust triples):
#   -t arm64-v8a / armeabi-v7a   Android ABIs
#   -P 24                        Android API level (minSdk 24 — doc 02 §8)
#   -o OUT                       copies lib*.so into OUT/<abi>/ itself
#   everything after -- goes to cargo: -p mc-ffi is CARGO's package selector
# Version pinned at install time; see .github/workflows/android.yml.
cargo ndk \
  -t arm64-v8a \
  -t armeabi-v7a \
  -P 24 \
  -o "$OUT" \
  -- build --release -p mc-ffi --lib

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
