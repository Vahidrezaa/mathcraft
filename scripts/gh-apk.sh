#!/usr/bin/env bash
# MathCraft -- GitHub Actions APK helper, straight from the Termux terminal (doc 09 s8).
# The phone stays the editing/test station; heavy builds run on CI runners.
# Usage:
#   bash scripts/gh-apk.sh trigger     # run the android workflow now (workflow_dispatch)
#   bash scripts/gh-apk.sh watch       # watch the latest android run live
#   bash scripts/gh-apk.sh download    # download the latest successful APKs into dist/
set -euo pipefail
cd "$(dirname "$0")/.."
command -v gh >/dev/null || { echo "gh نصب نیست: pkg install gh"; exit 1; }
command -v jq >/dev/null || { echo "jq نصب نیست: pkg install jq"; exit 1; }

CMD="${1:-download}"
case "$CMD" in
    trigger)
        gh workflow run android.yml
        echo "trigger شد — پیگیری: bash scripts/gh-apk.sh watch"
        ;;
    watch)
        RUN_ID="$(gh run list -w android.yml -L 1 --json databaseId -q '.[0].databaseId')"
        [ -n "$RUN_ID" ] || { echo "ران پیدا نشد — اول: bash scripts/gh-apk.sh trigger"; exit 1; }
        gh run watch "$RUN_ID" --exit-status
        echo "تمام شد — دانلود: bash scripts/gh-apk.sh download"
        ;;
    download)
        RUN_ID="$(gh run list -w android.yml -L 1 -s success --json databaseId -q '.[0].databaseId')"
        [ -n "$RUN_ID" ] || { echo "هیچ ران موفقی نیست — اول: bash scripts/gh-apk.sh trigger"; exit 1; }
        rm -rf dist
        gh run download "$RUN_ID" -D dist
        echo "APKها آماده‌اند:"
        ls -la dist/
        ;;
    *)
        echo "usage: bash scripts/gh-apk.sh trigger|watch|download"
        exit 1
        ;;
esac
