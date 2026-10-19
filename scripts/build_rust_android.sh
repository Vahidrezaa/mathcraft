#!/usr/bin/env bash
# MathCraft — بیلد موتور Rust برای اندروید (سند ۰۲ بخش ۸)
# cargo-ndk → libmathcraft.so → jniLibs پروژه Flutter
set -euo pipefail
cd "$(dirname "$0")/.."

command -v cargo-ndk >/dev/null || {
  echo "cargo-ndk نصب نیست — نصب می‌کنم (روی گوشی چند دقیقه)…"
  cargo install cargo-ndk --locked
}

# NDK بومی aarch64 (سند ۰۹) — مسیر پیش‌فرض نصب setup-termux-flutter.sh
export ANDROID_NDK_HOME="${ANDROID_NDK_HOME:-$HOME/android/sdk/ndk/29.0.14206865}"

# روی Termux فقط arm64 بیلد می‌شود (سرعت/حافظه — سند ۰۹ بخش ۵)؛ armv7 اختیاری:
#   bash scripts/build_rust_android.sh --armv7   (روی رایانه هر دو پیش‌فرض است)
TARGETS=(aarch64-linux-android)
if [ "${1:-}" = "--armv7" ] || [[ "${PREFIX:-}" != *com.termux* ]]; then
  TARGETS+=(armv7-linux-androideabi)
fi

rustup target add "${TARGETS[@]}"

NDK_ARGS=()
for t in "${TARGETS[@]}"; do NDK_ARGS+=(-t "$t"); done

OUT="app/android/app/src/main/jniLibs"
mkdir -p "$OUT/arm64-v8a" "$OUT/armeabi-v7a"

echo "== build (release) =="
cargo ndk "${NDK_ARGS[@]}" -p mc-ffi build --release

for t in "${TARGETS[@]}"; do
  case "$t" in
    aarch64-linux-android)     cp "target/$t/release/libmathcraft.so" "$OUT/arm64-v8a/" ;;
    armv7-linux-androideabi)   cp "target/$t/release/libmathcraft.so" "$OUT/armeabi-v7a/" ;;
  esac
done

echo "libmathcraft.so ساخته شد برای: ${TARGETS[*]} → $OUT"
echo "حالا در app/: flutter run"
