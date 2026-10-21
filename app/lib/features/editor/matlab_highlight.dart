import 'package:flutter/material.dart';

/// کنترلر متن ادیتور با هایلایت سینتکس متلب (سند ۰۶ §5.1 — تب ادیتور).
///
/// پویش کاراکتری تک‌گذر (بدون regex) — برای فایل‌های موبایلی کافی و سریع:
/// - کامنت `%…` تا انتهای خط
/// - رشته `'…'` و `"…"` تا جفت‌بست یا انتهای خط
/// - اعداد (شامل نماد علمی 1.5e-3)
/// - کلیدواژه‌های رزرو (همان KEYWORDS سند ۰۳ §2)
/// رنگ‌ها از ColorScheme تم فعال (تیره/روشن) — بدون رنگ هاردکد (اصل CDS).
class MatlabCodeController extends TextEditingController {
  MatlabCodeController({super.text});

  static const _keywords = <String>{
    'if', 'elseif', 'else', 'end', 'for', 'while', 'break', 'continue',
    'return', 'function',
  };

  @override
  TextSpan buildTextSpan({
    required BuildContext context,
    TextStyle? style,
    required bool withComposing,
  }) {
    final scheme = Theme.of(context).colorScheme;
    final commentStyle = style?.copyWith(
      color: scheme.onSurfaceVariant,
      fontStyle: FontStyle.italic,
    );
    final stringStyle = style?.copyWith(color: scheme.tertiary);
    final numberStyle = style?.copyWith(color: scheme.secondary);
    final keywordStyle = style?.copyWith(
      color: scheme.primary,
      fontWeight: FontWeight.w700,
    );

    final text = this.text;
    final spans = <TextSpan>[];
    final buffer = StringBuffer();
    var i = 0;

    void flush() {
      if (buffer.isEmpty) return;
      spans.add(TextSpan(text: buffer.toString(), style: style));
      buffer.clear();
    }

    void flushStyled(TextStyle? s) {
      if (buffer.isEmpty) return;
      spans.add(TextSpan(text: buffer.toString(), style: s));
      buffer.clear();
    }

    while (i < text.length) {
      final ch = text[i];

      // کامنت — تا انتهای خط
      if (ch == '%') {
        flush();
        final end = text.indexOf('\n', i);
        final stop = end == -1 ? text.length : end;
        buffer.write(text.substring(i, stop));
        flushStyled(commentStyle);
        i = stop;
        continue;
      }

      // رشته — جفت‌بست یا انتهای خط (سادگی: بدون escape چندخطی)
      if (ch == '\'' || ch == '"') {
        flush();
        buffer.write(ch);
        i++;
        while (i < text.length) {
          final c = text[i];
          buffer.write(c);
          i++;
          if (c == ch) break;
          if (c == '\n') break; // رشتهٔ بسته‌نشده تا انتهای خط
        }
        flushStyled(stringStyle);
        continue;
      }

      // عدد — شروع با رقم
      if (ch.codeUnitAt(0) >= 0x30 && ch.codeUnitAt(0) <= 0x39) {
        flush();
        var j = i;
        while (j < text.length) {
          final c = text[j];
          final isDigit = c.codeUnitAt(0) >= 0x30 && c.codeUnitAt(0) <= 0x39;
          if (isDigit ||
              c == '.' ||
              c == 'e' ||
              c == 'E' ||
              ((c == '+' || c == '-') &&
                  j > i &&
                  (text[j - 1] == 'e' || text[j - 1] == 'E'))) {
            j++;
          } else {
            break;
          }
        }
        buffer.write(text.substring(i, j));
        flushStyled(numberStyle);
        i = j;
        continue;
      }

      // شناسه — کلیدواژه؟
      final cu = ch.codeUnitAt(0);
      final isWordStart =
          (cu >= 0x41 && cu <= 0x5A) || (cu >= 0x61 && cu <= 0x7A) || ch == '_';
      if (isWordStart) {
        flush();
        var j = i;
        while (j < text.length) {
          final c = text[j];
          final cu2 = c.codeUnitAt(0);
          final isWord = (cu2 >= 0x41 && cu2 <= 0x5A) ||
              (cu2 >= 0x61 && cu2 <= 0x7A) ||
              (cu2 >= 0x30 && cu2 <= 0x39) ||
              c == '_';
          if (!isWord) break;
          j++;
        }
        final word = text.substring(i, j);
        spans.add(TextSpan(
          text: word,
          style: _keywords.contains(word) ? keywordStyle : style,
        ));
        i = j;
        continue;
      }

      buffer.write(ch);
      i++;
    }
    flush();
    return TextSpan(children: spans, style: style);
  }
}
