import 'package:flutter/material.dart';

import '../../core/cds/cds_tokens.dart';
import '../../src/rust/api.dart' as api;

/// حل رنگ قرارداد رسم در Dart — دو لایهٔ v2 (سند ۰۴ §8.4):
/// - `PlotColor_Palette(i)` = توکن معنایی؛ resolve روی پالت ۸رنگی CDS
///   (سند ۰۶ §3) با سیکل — تغییر تم/پالت بدون بازتولید موتور بازرنگ می‌شود.
/// - `PlotColor_Rgba` = مقدار دقیق (خط‌اسپک متلب 'r' و سه‌تایی‌ها).
Color resolvePlotColor(api.PlotColor color) {
  final resolved = switch (color) {
    api.PlotColor_Palette(:final field0) =>
      CdsColors.plotPalette[field0.clamp(0, CdsColors.plotPalette.length - 1)],
    api.PlotColor_Rgba(:final r, :final g, :final b, :final a) =>
      Color.fromARGB(a, r, g, b),
  };
  return resolved;
}

/// الگوی خط‌چین برحسب پیکسل منطقی — null = خط صاف؛ لیست خالی/نامرئی =
/// بدون خط (فقط مارکر — پارتی 'b*' متلب؛ سند ۰۴ §8.4).
List<double>? dashPatternFor(api.DashStyle style) => switch (style) {
      api.DashStyle.solid => null,
      api.DashStyle.dashed => const [8, 4],
      api.DashStyle.dotted => const [2, 3],
      api.DashStyle.dashDot => const [8, 3, 2, 3],
      api.DashStyle.invisible => const [],
    };

/// آیا این استایل خط قابل‌کشیدن دارد؟ (Invisible = فقط مارکر)
bool hasLine(api.LineStyle style) => style.dash != api.DashStyle.invisible;
