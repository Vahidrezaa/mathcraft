#!/usr/bin/env bash
# MathCraft -- Termux Flutter/Android toolchain installer (doc 09, path A).
# Installs the on-device toolchain that doc 02 s8 needs to produce an APK
# from inside Termux: openjdk-17, Flutter (termuxvoid pkg), NDK r29 aarch64,
# SDK build-tools aarch64, official cmdline-tools + android platforms, and
# the ARM64 aapt2 override for Gradle.
#
# Idempotent: safe to re-run. Large downloads are checksum-verified.
# Docs: docs/09-environment-termux.md  (golden rule: change doc first, then this)
#
# Usage (repo root inside Termux):
#   bash scripts/setup-termux-flutter.sh
set -euo pipefail

# ---------------------------------------------------------------- pins (doc 09 s3)
NDK_TAG="r29"
NDK_DIRVER="29.0.14206865"
NDK_URL="https://github.com/lzhiyong/termux-ndk/releases/download/${NDK_TAG}/android-ndk-r29-aarch64.tar.xz"
NDK_SHA256="02e10e4ddfe8deaeb0bd0cf29d04c981ed5bc8a5d6b560ebb9e7661f472d684b"
SDKB_URL="https://github.com/lzhiyong/termux-ndk/releases/download/android-sdk/android-sdk-aarch64.tar.xz"
SDKB_SHA256="8a23d2a10897ad74e34e10d7d2647ed450fad194d622b8b46e1ebd44557171ad"
CLT_URL="https://dl.google.com/android/repository/commandlinetools-linux-11076708_latest.zip"
AAPT2_URL="https://github.com/ReVanced/aapt2/releases/download/v1.1.0/aapt2-arm64-v8a"
TV_REPO_INSTALL="https://github.com/termuxvoid/repo/raw/main/install.sh"
PLATFORMS=("platforms;android-35" "platforms;android-36")

say()  { printf '\n\033[1;36m== %s\033[0m\n' "$*"; }
warn() { printf '\033[1;33m[warn]\033[0m %s\n' "$*"; }
die()  { printf '\033[1;31m[error]\033[0m %s\n' "$*" >&2; exit 1; }
fetch(){ # fetch <url> <dest>
    curl -fL --retry 3 --retry-delay 2 -o "$2" "$1" || die "download failed: $1"
}
sha_ok(){ # sha_ok <file> <expected>
    echo "$2  $1" | sha256sum -c - >/dev/null 2>&1 || die "SHA256 mismatch: $1"
}

# ---------------------------------------------------------------- 1/9 preflight
say "1/9 preflight"
case "${PREFIX:-}" in
    *com.termux*) : ;;
    *) die "this script must run inside Termux (\$PREFIX=$PREFIX)" ;;
esac
[ "$(uname -m)" = "aarch64" ] || die "aarch64 device required (found: $(uname -m))"
AVAIL_KB="$(df -Pk "$HOME" | awk 'NR==2{print $4}')"
if [ "$AVAIL_KB" -lt 7000000 ]; then
    warn "free space in \$HOME is ~$((AVAIL_KB/1000000))GB; doc 09 s5 budgets ~8GB. Continuing."
fi
command -v pkg >/dev/null 2>&1 || die "pkg not found"
mkdir -p "$HOME/android/tmp"

# ---------------------------------------------------------------- 2/9 base packages
say "2/9 base packages (openjdk-17, unzip, curl)"
pkg install -y openjdk-17 unzip which curl >/dev/null 2>&1 || pkg install -y openjdk-17 unzip which curl
java -version 2>&1 | head -1 || die "openjdk-17 did not install"

# ---------------------------------------------------------------- 3/9 flutter
say "3/9 flutter (termuxvoid repo -- doc 09 s3)"
if ! command -v flutter >/dev/null 2>&1; then
    fetch "$TV_REPO_INSTALL" "$HOME/android/tmp/tv-install.sh"
    bash "$HOME/android/tmp/tv-install.sh" || die \
        "termuxvoid repo bootstrap failed; fallback options in doc 09 s2 (proot/AVF) and s7 (mumumusuc deb)"
    pkg install -y flutter || die "pkg install flutter failed (see doc 09 s7 fallbacks)"
fi
flutter --version || die "flutter is installed but not runnable (doc 09 s7)"

# ---------------------------------------------------------------- 4/9 ANDROID_HOME
say "4/9 android sdk root"
export ANDROID_HOME="$HOME/android/sdk"
export ANDROID_NDK_HOME="$ANDROID_HOME/ndk/$NDK_DIRVER"
mkdir -p "$ANDROID_HOME/ndk" "$ANDROID_HOME/cmdline-tools" "$HOME/.gradle"

# ---------------------------------------------------------------- 5/9 NDK r29 (aarch64)
say "5/9 android NDK r29 aarch64 (for Rust -> libmathcraft.so)"
if [ ! -x "$ANDROID_NDK_HOME/toolchains/llvm/prebuilt"/*/bin/clang ]; then
    fetch "$NDK_URL" "$HOME/android/tmp/ndk.tar.xz"
    sha_ok "$HOME/android/tmp/ndk.tar.xz" "$NDK_SHA256"
    tar -xJf "$HOME/android/tmp/ndk.tar.xz" -C "$HOME/android/tmp"
    NDK_SRC="$(ls -d "$HOME/android/tmp"/android-ndk-* | head -1)"
    [ -n "$NDK_SRC" ] || die "unexpected NDK tarball layout"
    rm -rf "$ANDROID_NDK_HOME"
    mv "$NDK_SRC" "$ANDROID_NDK_HOME"
fi
NDK_PREBUILT="$(ls -d "$ANDROID_NDK_HOME/toolchains/llvm/prebuilt"/* | head -1)"
[ -x "$NDK_PREBUILT/bin/clang" ] || die "NDK clang missing at $NDK_PREBUILT"
echo "NDK ok: $ANDROID_NDK_HOME (host tag: $(basename "$NDK_PREBUILT"))"

# ---------------------------------------------------------------- 6/9 SDK bundle (aarch64)
say "6/9 android SDK bundle (build-tools 35, platform-tools, cmake, ninja -- aarch64)"
if [ ! -d "$ANDROID_HOME/build-tools/35.0.0" ]; then
    fetch "$SDKB_URL" "$HOME/android/tmp/sdk.tar.xz"
    sha_ok "$HOME/android/tmp/sdk.tar.xz" "$SDKB_SHA256"
    tar -xJf "$HOME/android/tmp/sdk.tar.xz" -C "$HOME/android/tmp"
    SDK_SRC="$(ls -d "$HOME/android/tmp"/*/ 2>/dev/null | while read -r d; do
        [ -d "$d/build-tools" ] && echo "$d"; done | head -1)"
    # some layouts extract the SDK contents at the tarball root
    if [ -z "$SDK_SRC" ] && [ -d "$HOME/android/tmp/build-tools" ]; then
        SDK_SRC="$HOME/android/tmp"
    fi
    [ -n "$SDK_SRC" ] || die "build-tools not found inside sdk tarball"
    cp -a "$SDK_SRC"/. "$ANDROID_HOME/"
fi
ls -d "$ANDROID_HOME/build-tools"/*/ >/dev/null 2>&1 || die "SDK bundle extraction failed"
AAPT2_BUNDLED="$(ls "$ANDROID_HOME"/build-tools/*/aapt2 2>/dev/null | head -1 || true)"
[ -n "$AAPT2_BUNDLED" ] && echo "bundled aapt2: $AAPT2_BUNDLED"

# ---------------------------------------------------------------- 7/9 cmdline-tools + platforms
say "7/9 cmdline-tools (official, java) + platforms android-35/36"
if [ ! -x "$ANDROID_HOME/cmdline-tools/latest/bin/sdkmanager" ]; then
    fetch "$CLT_URL" "$HOME/android/tmp/clt.zip"
    unzip -qo "$HOME/android/tmp/clt.zip" -d "$HOME/android/tmp/clt"
    rm -rf "$ANDROID_HOME/cmdline-tools/latest"
    mv "$HOME/android/tmp/clt/cmdline-tools" "$ANDROID_HOME/cmdline-tools/latest"
fi
export PATH="$ANDROID_HOME/cmdline-tools/latest/bin:$ANDROID_HOME/platform-tools:$PATH"
yes | sdkmanager --licenses >/dev/null 2>&1 || warn "license step reported issues (continuing)"
sdkmanager "${PLATFORMS[@]}" >/dev/null || sdkmanager "${PLATFORMS[0]}" || die "sdkmanager platforms failed"

# ---------------------------------------------------------------- 8/9 aapt2 override + gradle props
say "8/9 arm64 aapt2 override (ReVanced) + user gradle.properties"
mkdir -p "$ANDROID_HOME/aapt2"
fetch "$AAPT2_URL" "$ANDROID_HOME/aapt2/aapt2"
chmod +x "$ANDROID_HOME/aapt2/aapt2"
USER_GP="$HOME/.gradle/gradle.properties"
touch "$USER_GP"
grep -q "aapt2FromMavenOverride" "$USER_GP" || \
    echo "android.aapt2FromMavenOverride=$ANDROID_HOME/aapt2/aapt2" >> "$USER_GP"
grep -q "org.gradle.jvmargs" "$USER_GP" || \
    echo "org.gradle.jvmargs=-Xmx1536m -XX:MaxMetaspaceSize=512m" >> "$USER_GP"
echo "user gradle.properties: $USER_GP"

# ---------------------------------------------------------------- 9/9 rust targets + env + doctor
say "9/9 rust android targets, persistent env, flutter doctor"
if command -v rustup >/dev/null 2>&1; then
    rustup target add aarch64-linux-android armv7-linux-androideabi
else
    warn "rustup not found: 'pkg rust' cannot cross-compile to Android."
    warn "for the APK layer install rustup minimal: curl https://sh.rustup.rs -sSf | sh -s -- -y --profile minimal"
fi

ENV_BLOCK="# --- MathCraft toolchain (doc 09) ---
export ANDROID_HOME=\"$ANDROID_HOME\"
export ANDROID_NDK_HOME=\"$ANDROID_NDK_HOME\"
export PATH=\"\$ANDROID_HOME/cmdline-tools/latest/bin:\$ANDROID_HOME/platform-tools:\$ANDROID_HOME/aapt2:\$PATH\""
for RC in "$HOME/.profile" "$HOME/.bashrc"; do
    touch "$RC"
    grep -q "MathCraft toolchain" "$RC" || printf '\n%s\n' "$ENV_BLOCK" >> "$RC"
done

flutter config --android-sdk "$ANDROID_HOME" >/dev/null
flutter doctor -v || warn "flutter doctor reported issues above -- review before bootstrap"

cat <<EOF

Toolchain ready (doc 09, path A). Next steps:
  bash scripts/bootstrap.sh           # flutter create + FRB codegen + flutter test
  bash scripts/build_rust_android.sh  # libmathcraft.so -> app jniLibs
  cd app && flutter build apk --debug
Notes:
  - first APK build takes ~15-40 min on a phone (doc 09 s5)
  - release builds belong to CI (doc 08 s7), not the phone
EOF
