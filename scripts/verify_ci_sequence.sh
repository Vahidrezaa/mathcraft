#!/usr/bin/env bash
# تمرینِ ۱:۱ توالی CI (ci.yml) روی سندباکس محلی — درس CI#35 (۲۰۲۶-۱۱-۲۰):
# گام fmt با خروجیِ ماسک‌نشده باید «اولین» چیزِ تأییدشده باشد؛ خروجی هر گام
# چاپ می‌شود و اولین شکست، اسکریپت را می‌بندد (بدون >/dev/null).
# استفاده: bash scripts/verify_ci_sequence.sh [--skip-flutter]
set -euo pipefail
cd "$(dirname "$0")/.."

echo "== 1/6 fmt --check (visible, fatal) =="
cargo fmt --all -- --check
echo "   fmt CLEAN"

echo "== 2/6 clippy strict =="
cargo clippy --workspace --all-targets -- -D warnings
echo "   clippy CLEAN"

echo "== 3/6 workspace tests =="
cargo test --workspace 2>&1 | grep -E "test result" | awk -F'[ ;]' '{p+=$4; f+=$6} END {print "   TOTAL passed:", p, "failed:", f; if (f>0) exit 1}'

echo "== 4/6 panic audit =="
python3 scripts/panic_audit.py

if [[ "${1:-}" == "--skip-flutter" ]]; then
  echo "== (flutter skipped) =="
  exit 0
fi

export PATH="/home/z/my-project/scratch/flutter/bin:$PATH"
echo "== 5/6 flutter analyze =="
(cd app && flutter pub get >/dev/null && flutter analyze)
echo "   analyze CLEAN"

echo "== 6/6 flutter tests =="
(cd app && flutter test)
echo "ALL CI GATES GREEN LOCALLY"
