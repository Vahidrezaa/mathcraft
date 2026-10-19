#!/usr/bin/env bash
# MathCraft — Termux one-shot setup (P0.1, doc 07 §2 P0 / doc 02 §2 mc-cli)
# Installs prerequisites (if missing), runs the test suite, builds the native
# engine + `mc` console in release mode, installs it into $PREFIX/bin and
# smoke-tests it. Everything below runs fully offline AFTER package install.
#
# Usage (from the repo root inside Termux):
#   bash scripts/setup-termux.sh
set -euo pipefail

say() { printf '\n\033[1;33m== %s\033[0m\n' "$*"; }

cd "$(dirname "$0")/.."

say "1/5 packages (Termux)"
if command -v pkg >/dev/null 2>&1; then
    # Termux: `rust` brings rustc + cargo. No C toolchain is needed in P0.1
    # because every workspace crate is pure Rust (mc-cas gains rug/GMP in P4).
    command -v cargo >/dev/null 2>&1 || pkg install -y rust
else
    command -v cargo >/dev/null 2>&1 || {
        echo "cargo not found — install Rust first (https://rustup.rs)"; exit 1;
    }
fi

say "2/5 toolchain"
if command -v rustup >/dev/null 2>&1 && [ -f rust-toolchain.toml ]; then
    # rust-toolchain.toml pins the CI/F-Droid reproducible toolchain (doc 08 §7).
    # On a phone we deliberately reuse the installed toolchain instead of
    # downloading a second one — RUSTUP_TOOLCHAIN overrides the pin.
    export RUSTUP_TOOLCHAIN=stable
fi
rustc --version
cargo --version

say "3/5 tests (cargo test --workspace)"
cargo test --workspace

say "4/5 release build (LTO, stripped)"
cargo build --release -p mc-cli

say "5/5 install + smoke test"
BIN="$PWD/target/release/mc"
if [ -n "${PREFIX:-}" ] && [ -d "$PREFIX/bin" ]; then
    DEST="$PREFIX/bin"        # Termux
else
    DEST="$HOME/.local/bin"   # desktop fallback
    mkdir -p "$DEST"
    case ":$PATH:" in
        *":$DEST:"*) ;;
        *) echo "note: add $DEST to your PATH" ;;
    esac
fi
cp "$BIN" "$DEST/mc"
"$DEST/mc" -e "1+1"

cat <<'EOF'

MathCraft console is ready. Start it with:  mc
Quick tour inside the console:  demo
Persian messages:               MC_LANG=fa mc
One-shot scripting:             mc -e "x = 7; x^2"
EOF
