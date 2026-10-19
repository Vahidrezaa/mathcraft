#!/usr/bin/env bash
# بازتولید فونت‌های باندل‌شده (همه OFL — سند ۰۶ §2.3: باندل کامل)
# نسخه‌ها پین‌شده‌اند تا بیلد reproducible بماند (سند ۰۸ بخش ۷).
set -euo pipefail
cd "$(dirname "$0")/../app/assets/fonts"

for w in Regular Medium Bold; do
  echo "Vazirmatn-$w.ttf"
  curl -fL -o "Vazirmatn-$w.ttf" \
    "https://raw.githubusercontent.com/rastikerdar/vazirmatn/v33.003/fonts/ttf/Vazirmatn-$w.ttf"
done

echo "JetBrainsMono"
curl -fL -o /tmp/jbmono.zip \
  "https://github.com/JetBrains/JetBrainsMono/releases/download/v2.304/JetBrainsMono-2.304.zip"
unzip -o -j /tmp/jbmono.zip \
  "fonts/ttf/JetBrainsMono-Regular.ttf" \
  "fonts/ttf/JetBrainsMono-Bold.ttf"

echo "انجام شد — فونت‌ها در app/assets/fonts/"
