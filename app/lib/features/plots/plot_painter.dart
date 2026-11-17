import 'dart:math' as math;

import 'package:flutter/material.dart';

import '../../core/cds/cds_tokens.dart';
import '../../src/rust/api.dart' as api;
import 'plot_format.dart';
import 'plot_geometry.dart';

/// رنگ‌های نقاش برحسب روشنایی تم — همه از توکن‌های CDS (anti-pattern #2:
/// رنگ هاردکد ممنوع — سند ۰۶ §12).
class PlotTheme {
  PlotTheme._({
    required this.bg,
    required this.border,
    required this.grid,
    required this.textPrimary,
    required this.textSecondary,
    required this.accent,
    required this.chipBg,
  });

  factory PlotTheme.of(Brightness brightness) {
    final dark = brightness == Brightness.dark;
    return PlotTheme._(
      bg: dark ? CdsColors.darkBgSurface : CdsColors.lightBgSurface,
      border: dark ? CdsColors.darkBorderSubtle : CdsColors.lightBorderSubtle,
      // grid نقطه‌چین با ۳۰٪ opacity — سند ۰۶ §3
      grid: (dark ? CdsColors.darkBorderSubtle : CdsColors.lightBorderSubtle)
          .withOpacity(0.30),
      textPrimary:
          dark ? CdsColors.darkTextPrimary : CdsColors.lightTextPrimary,
      textSecondary:
          dark ? CdsColors.darkTextSecondary : CdsColors.lightTextSecondary,
      accent:
          dark ? CdsColors.darkAccentSecondary : CdsColors.lightAccentSecondary,
      chipBg: dark ? CdsColors.darkBgElevated : CdsColors.lightBgElevated,
    );
  }

  final Color bg;
  final Color border;
  final Color grid;
  final Color textPrimary;
  final Color textSecondary;
  final Color accent;
  final Color chipBg;
}

/// تبدیل دید — zoom/pan فقط روی «داده» اعمال می‌شود؛ فریم/تیک‌ها ثابت
/// می‌مانند (سند ۰۲ §7: تعامل بدون FFI در حلقهٔ فریم، فقط transform).
class PlotViewTransform {
  const PlotViewTransform({this.scale = 1.0, this.offset = Offset.zero});

  static const identity = PlotViewTransform();

  static const _maxScale = 64.0;

  final double scale;
  final Offset offset;

  bool get isIdentity => scale == 1.0 && offset == Offset.zero;

  PlotViewTransform clamped() => PlotViewTransform(
        scale: scale.clamp(1.0, _maxScale),
        // pan سقف‌دار تا نمودار از دید خارج نشود
        offset: Offset(
          offset.dx.clamp(-200.0 * (scale - 1), 200.0 * (scale - 1)),
          offset.dy.clamp(-200.0 * (scale - 1), 200.0 * (scale - 1)),
        ),
      );

  /// نقطهٔ fit-space زیر focal بعد از scale جدید سر جایش می‌ماند:
  /// S(P) = (P−c)·s + c + o ⇒ o′ = o − ((F−c−o)/s)·(s′−s)
  /// (pinch حول مرکز pinch نه مرکز صفحه — سند ۰۶ §7.3).
  PlotViewTransform pinchAround(Offset focal, double scaleFactor, Size size) {
    final c = size.center(Offset.zero);
    final newScale = (scale * scaleFactor).clamp(1.0, _maxScale);
    final deltaS = newScale - scale;
    if (deltaS.abs() < 1e-9) {
      return clamped();
    }
    final anchor = (focal - c - offset) / scale;
    final newOffset = offset - anchor * deltaS;
    return PlotViewTransform(scale: newScale, offset: newOffset).clamped();
  }

  PlotViewTransform translate(Offset delta) =>
      PlotViewTransform(scale: scale, offset: offset + delta).clamped();

  @override
  bool operator ==(Object other) =>
      other is PlotViewTransform &&
      other.scale == scale &&
      other.offset == offset;

  @override
  int get hashCode => Object.hash(scale, offset);
}

/// نقاش یک سلول ۲بعدی — فریم/تیک‌ها ثابت، داده زیر transform و clip.
class PlotPainter extends CustomPainter {
  PlotPainter({
    required this.cell,
    required this.transform,
    required this.brightness,
    this.crosshair,
  });

  final PlotCellData cell;
  final PlotViewTransform transform;
  final Brightness brightness;
  final CrosshairSnap? crosshair;

  static const _mono11 = TextStyle(
    fontFamily: 'JetBrainsMono',
    fontSize: 11,
    height: 1.0,
  );

  @override
  void paint(Canvas canvas, Size size) {
    final theme = PlotTheme.of(brightness);
    final layout = CellLayout.build(cell, size);
    final rect = layout.plotRect;

    // پس‌زمینهٔ بوم (immersion — کل صفحه)
    canvas.drawRect(Offset.zero & size, Paint()..color = theme.bg);

    if (cell is PolarCellData) {
      _paintPolar(canvas, size, cell as PolarCellData, theme);
      _paintLegend(canvas, size, cell, theme);
      return;
    }

    // شبکهٔ نقطه‌چین روی تیک‌های برچسب‌دار — سند ۰۶ §3
    if (cell.axes!.grid && cell.axes!.visible) {
      final gridPaint = Paint()
        ..color = theme.grid
        ..style = PaintingStyle.stroke
        ..strokeWidth = 1;
      for (final (_, sx) in layout.xTickXs) {
        _dashedLine(
            canvas, Offset(sx, rect.top), Offset(sx, rect.bottom), gridPaint);
      }
      for (final (_, sy) in layout.yTickYs) {
        _dashedLine(canvas, Offset(rect.left, sy), Offset(rect.right, sy),
            gridPaint);
      }
    }

    // فریم + برچسب‌ها (ثابت)
    if (cell.axes!.visible) {
      canvas.drawRect(
        rect,
        Paint()
          ..color = theme.border
          ..style = PaintingStyle.stroke
          ..strokeWidth = 1,
      );
      _paintTicks(canvas, layout, theme);
      _paintTitles(canvas, layout, cell.axes!, theme);
    }

    // داده — transform حول مرکز بوم + clip به ناحیهٔ رسم
    canvas.save();
    canvas.clipRect(rect);
    final center = rect.center;
    canvas.translate(
      center.dx + transform.offset.dx,
      center.dy + transform.offset.dy,
    );
    canvas.scale(transform.scale);
    canvas.translate(-center.dx, -center.dy);

    switch (cell) {
      case LineCellData(:final series):
        for (final s in series) {
          _paintSeries(canvas, layout, s);
        }
      case BarCellData(
          :final series,
          :final width,
          :final horizontal,
          :final stacked,
        ):
        _paintBars(canvas, layout, series, width, horizontal, stacked);
      case HistCellData(:final binEdges, :final counts, :final color):
        _paintHistogram(canvas, layout, binEdges, counts, color);
      case AreaCellData(:final series, :final baseline):
        for (final s in series) {
          _paintArea(canvas, layout, s, baseline);
        }
      case FillCellData(:final polys):
        for (final p in polys) {
          _paintFill(canvas, layout, p);
        }
      case ErrorCellData(:final series):
        for (final s in series) {
          _paintErrorSeries(canvas, layout, s);
        }
      default:
        break;
    }
    canvas.restore();

    // crosshair — بیرون از transform (خطوط صفحه‌ای)
    if (crosshair != null && cell.axes!.visible) {
      _paintCrosshair(canvas, layout, crosshair!, theme);
    }

    _paintLegend(canvas, size, cell, theme);
  }

  // ---------- محورها ----------

  void _paintTicks(Canvas canvas, CellLayout layout, PlotTheme theme) {
    final rect = layout.plotRect;
    final tickPaint = Paint()
      ..color = theme.border
      ..strokeWidth = 1;
    final labelStyle = _mono11.copyWith(color: theme.textSecondary);

    for (final (t, sx) in layout.xTickXs) {
      canvas.drawLine(
        Offset(sx, rect.bottom),
        Offset(sx, rect.bottom + 4),
        tickPaint,
      );
      _drawText(
        canvas,
        t.label!,
        Offset(sx, rect.bottom + 6),
        labelStyle,
        TextAlign.center,
      );
    }
    for (final (t, sy) in layout.yTickYs) {
      canvas.drawLine(
        Offset(rect.left - 4, sy),
        Offset(rect.left, sy),
        tickPaint,
      );
      _drawText(
        canvas,
        t.label!,
        Offset(rect.left - 8, sy),
        labelStyle,
        TextAlign.right,
      );
    }
  }

  void _drawText(
    Canvas canvas,
    String text,
    Offset anchor,
    TextStyle style,
    TextAlign align,
  ) {
    final tp = _textPainter(text, style);
    final Offset origin;
    switch (align) {
      case TextAlign.center:
        origin = Offset(anchor.dx - tp.width / 2, anchor.dy);
      case TextAlign.right:
        origin = Offset(anchor.dx - tp.width, anchor.dy - tp.height / 2);
      default:
        origin = Offset(anchor.dx, anchor.dy - tp.height / 2);
    }
    tp.paint(canvas, origin);
  }

  TextPainter _textPainter(String text, TextStyle style) {
    final tp = TextPainter(
      text: TextSpan(text: text, style: style),
      textDirection: TextDirection.ltr,
    )..layout();
    return tp;
  }

  void _paintTitles(Canvas canvas, CellLayout layout, AxesData axes, PlotTheme theme) {
    final rect = layout.plotRect;
    if (axes.title != null) {
      _drawText(
        canvas,
        axes.title!,
        Offset(rect.center.dx, 4),
        TextStyle(
          fontFamily: 'Vazirmatn',
          fontSize: 14,
          fontWeight: FontWeight.w700,
          color: theme.textPrimary,
        ),
        TextAlign.center,
      );
    }
    if (axes.xlabel != null) {
      _drawText(
        canvas,
        axes.xlabel!,
        Offset(rect.center.dx, rect.bottom + 18),
        TextStyle(
          fontFamily: 'Vazirmatn',
          fontSize: 12,
          color: theme.textSecondary,
        ),
        TextAlign.center,
      );
    }
    if (axes.ylabel != null) {
      canvas.save();
      canvas.rotate(-math.pi / 2);
      final tp = _textPainter(
        axes.ylabel!,
        TextStyle(
          fontFamily: 'Vazirmatn',
          fontSize: 12,
          color: theme.textSecondary,
        ),
      );
      // پس از rotate(-90°): محور x جدید به سمت بالا؛ نقطه شروع را تنظیم
      tp.paint(canvas, Offset(-rect.left - 4 - tp.width, rect.center.dy - tp.width / 2));
      canvas.restore();
    }
  }

  // ---------- سری‌ها ----------

  void _paintSeries(Canvas canvas, CellLayout layout, SeriesGeom s) {
    final pattern = s.dash;
    if (pattern == null || pattern.isNotEmpty) {
      final path = _seriesPath(layout, s);
      final paint = Paint()
        ..color = s.color
        ..style = PaintingStyle.stroke
        ..strokeWidth = s.width
        ..strokeCap = StrokeCap.round
        ..strokeJoin = StrokeJoin.round;
      if (pattern == null) {
        canvas.drawPath(path, paint);
      } else {
        drawDashedPath(canvas, path, pattern, paint);
      }
    }
    if (s.marker != null) {
      _paintMarkers(canvas, layout, s);
    }
  }

  /// مسیر سری در فضای صفحه (fit) — NaN/Inf = گپ در خط (رفتار متلب).
  Path _seriesPath(CellLayout layout, SeriesGeom s) {
    final path = Path();
    var penDown = false;
    for (var i = 0; i < s.x.length; i++) {
      final x = s.x[i];
      final y = s.y[i];
      if (!x.isFinite || !y.isFinite) {
        penDown = false;
        continue;
      }
      final p = layout.dataToScreen(x, y);
      if (!penDown) {
        path.moveTo(p.dx, p.dy);
        penDown = true;
      } else {
        path.lineTo(p.dx, p.dy);
      }
    }
    return path;
  }

  void _paintMarkers(Canvas canvas, CellLayout layout, SeriesGeom s) {
    final stroke = Paint()
      ..color = s.color
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1.5;
    final fill = Paint()..color = s.color;
    for (var i = 0; i < s.x.length; i++) {
      final x = s.x[i];
      final y = s.y[i];
      if (!x.isFinite || !y.isFinite) continue;
      final p = layout.dataToScreen(x, y);
      _drawMarker(canvas, s.marker!, p, stroke, fill);
    }
  }

  void _drawMarker(
    Canvas canvas,
    api.Marker marker,
    Offset p,
    Paint stroke,
    Paint fill,
  ) {
    switch (marker) {
      case api.Marker.point:
        canvas.drawCircle(p, 1.6, fill);
      case api.Marker.circle:
        canvas.drawCircle(p, 3, stroke);
      case api.Marker.square:
        canvas.drawRect(
          Rect.fromCenter(center: p, width: 5.5, height: 5.5),
          stroke,
        );
      case api.Marker.diamond:
        canvas.drawPath(_diamond(p), stroke);
      case api.Marker.cross:
        canvas.drawLine(
            Offset(p.dx - 3, p.dy - 3), Offset(p.dx + 3, p.dy + 3), stroke);
        canvas.drawLine(
            Offset(p.dx - 3, p.dy + 3), Offset(p.dx + 3, p.dy - 3), stroke);
      case api.Marker.plus:
        canvas.drawLine(
            Offset(p.dx - 3.5, p.dy), Offset(p.dx + 3.5, p.dy), stroke);
        canvas.drawLine(
            Offset(p.dx, p.dy - 3.5), Offset(p.dx, p.dy + 3.5), stroke);
      case api.Marker.star:
        for (var k = 0; k < 3; k++) {
          final a = k * math.pi / 3;
          final dx = math.cos(a) * 3.5;
          final dy = math.sin(a) * 3.5;
          canvas.drawLine(
              Offset(p.dx - dx, p.dy - dy), Offset(p.dx + dx, p.dy + dy), stroke);
        }
      case api.Marker.triangleUp:
        canvas.drawPath(_triangle(p, up: true), stroke);
      case api.Marker.triangleDown:
        canvas.drawPath(_triangle(p, up: false), stroke);
      case api.Marker.none:
        break;
    }
  }

  Path _diamond(Offset p) => Path()
    ..moveTo(p.dx, p.dy - 4)
    ..lineTo(p.dx + 4, p.dy)
    ..lineTo(p.dx, p.dy + 4)
    ..lineTo(p.dx - 4, p.dy)
    ..close();

  Path _triangle(Offset p, {required bool up}) {
    final dy = up ? -4.0 : 4.0;
    final dyBase = up ? 2.6 : -2.6;
    return Path()
      ..moveTo(p.dx, p.dy + dy)
      ..lineTo(p.dx + 3.6, p.dy + dyBase)
      ..lineTo(p.dx - 3.6, p.dy + dyBase)
      ..close();
  }

  // ---------- میله‌ها و هیستوگرام ----------

  void _paintBars(
    Canvas canvas,
    CellLayout layout,
    List<SeriesGeom> series,
    double width,
    bool horizontal,
    bool stacked,
  ) {
    if (series.isEmpty) return;
    final slot = barSlot(series.first.x);
    final n = series.length;
    final groupWidth = slot * width.clamp(0.05, 1.0);
    for (var i = 0; i < n; i++) {
      final s = series[i];
      final cumulative = List<double>.filled(s.x.length, 0);
      for (var k = 0; k < s.x.length; k++) {
        final x = s.x[k];
        final y = s.y[k];
        if (!x.isFinite || !y.isFinite) continue;
        final barWidth = stacked ? groupWidth : groupWidth / n;
        final bandStart = stacked
            ? x - groupWidth / 2
            : x - groupWidth / 2 + i * barWidth;
        final (vLo, vHi) = stacked
            ? (cumulative[k], cumulative[k] + y)
            : (math.min(0.0, y), math.max(0.0, y));
        if (stacked) cumulative[k] = vHi;

        final Rect rect;
        if (horizontal) {
          // x نمودار = مقدار، y نمودار = جایگاه میله
          final a = layout.dataToScreen(vLo, bandStart);
          final b = layout.dataToScreen(vHi, bandStart + barWidth);
          rect = Rect.fromPoints(
            Offset(math.min(a.dx, b.dx), math.min(a.dy, b.dy)),
            Offset(math.max(a.dx, b.dx), math.max(a.dy, b.dy)),
          );
        } else {
          final a = layout.dataToScreen(bandStart, vLo);
          final b = layout.dataToScreen(bandStart + barWidth, vHi);
          rect = Rect.fromPoints(
            Offset(math.min(a.dx, b.dx), math.min(a.dy, b.dy)),
            Offset(math.max(a.dx, b.dx), math.max(a.dy, b.dy)),
          );
        }
        canvas.drawRect(
          rect,
          Paint()..color = s.color.withOpacity(0.55),
        );
        canvas.drawRect(
          rect,
          Paint()
            ..color = s.color
            ..style = PaintingStyle.stroke
            ..strokeWidth = 1,
        );
      }
    }
  }

  void _paintHistogram(
    Canvas canvas,
    CellLayout layout,
    List<double> edges,
    List<double> counts,
    Color color,
  ) {
    for (var i = 0; i + 1 < edges.length && i < counts.length; i++) {
      final x0 = edges[i];
      final x1 = edges[i + 1];
      final c = counts[i];
      if (!x0.isFinite || !x1.isFinite || !c.isFinite) continue;
      final a = layout.dataToScreen(x0, 0);
      final b = layout.dataToScreen(x1, c);
      final rect = Rect.fromPoints(
        Offset(a.dx, math.min(a.dy, b.dy)),
        Offset(b.dx, math.max(a.dy, b.dy)),
      );
      canvas.drawRect(rect, Paint()..color = color.withOpacity(0.55));
      canvas.drawRect(
        rect,
        Paint()
          ..color = color
          ..style = PaintingStyle.stroke
          ..strokeWidth = 1,
      );
    }
  }

  void _paintArea(
    Canvas canvas,
    CellLayout layout,
    SeriesGeom s,
    double baseline,
  ) {
    final line = _seriesPath(layout, s);
    final n = s.x.length;
    if (n > 0) {
      final lastX = s.x[n - 1];
      final firstX = s.x[0];
      if (lastX.isFinite && firstX.isFinite) {
        final fillPath = Path.from(line)
          ..lineTo(
            layout.dataToScreen(lastX, baseline).dx,
            layout.dataToScreen(lastX, baseline).dy,
          )
          ..lineTo(
            layout.dataToScreen(firstX, baseline).dx,
            layout.dataToScreen(firstX, baseline).dy,
          )
          ..close();
        canvas.drawPath(
          fillPath,
          Paint()..color = s.color.withOpacity(0.35),
        );
      }
    }
    canvas.drawPath(
      line,
      Paint()
        ..color = s.color
        ..style = PaintingStyle.stroke
        ..strokeWidth = s.width
        ..strokeCap = StrokeCap.round,
    );
  }

  void _paintFill(Canvas canvas, CellLayout layout, SeriesGeom poly) {
    final path = Path();
    var first = true;
    for (var i = 0; i < poly.x.length; i++) {
      final x = poly.x[i];
      final y = poly.y[i];
      if (!x.isFinite || !y.isFinite) continue;
      final p = layout.dataToScreen(x, y);
      if (first) {
        path.moveTo(p.dx, p.dy);
        first = false;
      } else {
        path.lineTo(p.dx, p.dy);
      }
    }
    if (!first) {
      path.close();
      canvas.drawPath(path, Paint()..color = poly.color.withOpacity(0.55));
      canvas.drawPath(
        path,
        Paint()
          ..color = poly.color
          ..style = PaintingStyle.stroke
          ..strokeWidth = 1.2,
      );
    }
  }

  void _paintErrorSeries(Canvas canvas, CellLayout layout, SeriesWithBand s) {
    _paintSeries(canvas, layout, s.base);
    const cap = 3.0;
    final b = s.base;
    for (var i = 0; i < b.x.length; i++) {
      if (!b.x[i].isFinite || !b.y[i].isFinite) continue;
      final paint = Paint()
        ..color = b.color
        ..strokeWidth = 1.2;
      if (s.yNeg != null && i < s.yNeg!.length) {
        final yTop = b.y[i] + (i < (s.yPos?.length ?? 0) ? s.yPos![i] : 0.0);
        final top = layout.dataToScreen(b.x[i], yTop);
        final bot = layout.dataToScreen(b.x[i], b.y[i] - s.yNeg![i]);
        canvas.drawLine(top, bot, paint);
        canvas.drawLine(
            Offset(top.dx - cap, top.dy), Offset(top.dx + cap, top.dy), paint);
        canvas.drawLine(
            Offset(bot.dx - cap, bot.dy), Offset(bot.dx + cap, bot.dy), paint);
      }
      if (s.xNeg != null && i < s.xNeg!.length) {
        final xRight =
            b.x[i] + (i < (s.xPos?.length ?? 0) ? s.xPos![i] : 0.0);
        final left = layout.dataToScreen(b.x[i] - s.xNeg![i], b.y[i]);
        final right = layout.dataToScreen(xRight, b.y[i]);
        canvas.drawLine(left, right, paint);
        canvas.drawLine(
            Offset(left.dx, left.dy - cap), Offset(left.dx, left.dy + cap), paint);
        canvas.drawLine(
            Offset(right.dx, right.dy - cap), Offset(right.dx, right.dy + cap), paint);
      }
    }
  }

  // ---------- پلار ----------

  /// پلار — شبکهٔ حلقوی + ۸ ستون؛ x = θ (رادیان)، y = شعاع نرمال به rmax.
  /// (تیک‌های پلار در قرارداد نیستند — پیش‌فرض رندرر، مستند.)
  void _paintPolar(Canvas canvas, Size size, PolarCellData cell, PlotTheme theme) {
    final center = Offset(size.width / 2, size.height / 2);
    final radius = math.min(size.width, size.height) / 2 - 34;
    final gridPaint = Paint()
      ..color = theme.grid
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1;
    for (var ring = 1; ring <= 4; ring++) {
      canvas.drawCircle(center, radius * ring / 4, gridPaint);
    }
    for (var spoke = 0; spoke < 8; spoke++) {
      final a = spoke * math.pi / 4;
      canvas.drawLine(
        center,
        center + Offset(math.cos(a), math.sin(a)) * radius,
        gridPaint,
      );
    }
    var rMax = 0.0;
    for (final s in cell.series) {
      for (final y in s.y) {
        if (y.isFinite) rMax = math.max(rMax, y.abs());
      }
    }
    if (rMax <= 0) rMax = 1;
    for (final s in cell.series) {
      final paint = Paint()
        ..color = s.color
        ..style = PaintingStyle.stroke
        ..strokeWidth = s.width
        ..strokeCap = StrokeCap.round;
      final path = Path();
      var penDown = false;
      for (var i = 0; i < s.x.length; i++) {
        final theta = s.x[i];
        final r = s.y[i];
        if (!theta.isFinite || !r.isFinite) {
          penDown = false;
          continue;
        }
        final p = _polarPoint(center, radius, rMax, theta, r);
        if (!penDown) {
          path.moveTo(p.dx, p.dy);
          penDown = true;
        } else {
          path.lineTo(p.dx, p.dy);
        }
      }
      final pattern = s.dash;
      if (pattern == null) {
        canvas.drawPath(path, paint);
      } else if (pattern.isNotEmpty) {
        drawDashedPath(canvas, path, pattern, paint);
      }
      if (s.marker != null) {
        final stroke = Paint()
          ..color = s.color
          ..style = PaintingStyle.stroke
          ..strokeWidth = 1.5;
        final fill = Paint()..color = s.color;
        for (var i = 0; i < s.x.length; i++) {
          if (!s.x[i].isFinite || !s.y[i].isFinite) continue;
          final p = _polarPoint(center, radius, rMax, s.x[i], s.y[i]);
          _drawMarker(canvas, s.marker!, p, stroke, fill);
        }
      }
    }
    _drawText(
      canvas,
      formatG4(rMax),
      Offset(center.dx + radius + 6, center.dy),
      _mono11.copyWith(color: theme.textSecondary),
      TextAlign.left,
    );
  }

  Offset _polarPoint(Offset center, double radius, double rMax, double theta, double r) {
    final rr = r / rMax * radius;
    return center + Offset(math.cos(theta), math.sin(theta)) * rr;
  }

  // ---------- crosshair و legend ----------

  void _paintCrosshair(
    Canvas canvas,
    CellLayout layout,
    CrosshairSnap snap,
    PlotTheme theme,
  ) {
    final p = layout.dataToScreen(snap.x, snap.y);
    final rect = layout.plotRect;
    final line = Paint()
      ..color = theme.accent.withOpacity(0.6)
      ..strokeWidth = 1;
    canvas.drawLine(Offset(p.dx, rect.top), Offset(p.dx, rect.bottom), line);
    canvas.drawLine(Offset(rect.left, p.dy), Offset(rect.right, p.dy), line);
    canvas.drawCircle(
      p,
      5,
      Paint()
        ..color = snap.color
        ..style = PaintingStyle.stroke
        ..strokeWidth = 2,
    );
    // chip عددی (x, y) — mono روی bg/elevated (سند ۰۶ §7.3)
    final text = '(${formatG4(snap.x)}, ${formatG4(snap.y)})';
    final tp = _textPainter(text, _mono11.copyWith(color: theme.textPrimary));
    const padH = 8.0;
    const padV = 5.0;
    final chipW = tp.width + padH * 2;
    final chipH = tp.height + padV * 2;
    var topLeft = p + Offset(10, -10 - chipH);
    topLeft = Offset(
      topLeft.dx.clamp(rect.left + 2, rect.right - chipW - 2),
      topLeft.dy.clamp(rect.top + 2, rect.bottom - chipH - 2),
    );
    final rrect = RRect.fromRectAndRadius(
      Rect.fromLTWH(topLeft.dx, topLeft.dy, chipW, chipH),
      const Radius.circular(8),
    );
    canvas.drawRRect(
      rrect,
      Paint()..color = theme.chipBg.withOpacity(0.95),
    );
    tp.paint(canvas, Offset(topLeft.dx + padH, topLeft.dy + padV));
  }

  void _paintLegend(Canvas canvas, Size size, PlotCellData cell, PlotTheme theme) {
    final legend = cell.legend;
    if (legend == null || legend.entries.isEmpty) return;
    final colors = switch (cell) {
      LineCellData(:final series) => series.map((s) => s.color).toList(),
      BarCellData(:final series) => series.map((s) => s.color).toList(),
      AreaCellData(:final series) => series.map((s) => s.color).toList(),
      PolarCellData(:final series) => series.map((s) => s.color).toList(),
      ErrorCellData(:final series) => series.map((s) => s.base.color).toList(),
      FillCellData(:final polys) => polys.map((s) => s.color).toList(),
      _ => <Color>[],
    };
    final topSide = switch (legend.position) {
      api.LegendPosition.south ||
      api.LegendPosition.southEast ||
      api.LegendPosition.southWest =>
        false,
      _ => true,
    };
    final leftSide = switch (legend.position) {
      api.LegendPosition.west ||
      api.LegendPosition.northWest ||
      api.LegendPosition.southWest =>
        true,
      _ => false,
    };
    final tpEntries = <TextPainter, Color>{};
    var maxW = 0.0;
    for (var i = 0; i < legend.entries.length; i++) {
      final tp = _textPainter(
        legend.entries[i],
        TextStyle(
          fontFamily: 'Vazirmatn',
          fontSize: 12,
          color: theme.textPrimary,
        ),
      );
      maxW = math.max(maxW, tp.width);
      tpEntries[tp] = colors.isEmpty
          ? theme.textSecondary
          : colors[i % colors.length];
    }
    const rowH = 20.0;
    const swatch = 16.0;
    const padH = 10.0;
    const padV = 7.0;
    final boxW = maxW + swatch + padH * 2 + 6;
    final boxH = tpEntries.length * rowH + padV * 2 - 6;
    final x = leftSide ? 10.0 : size.width - boxW - 10;
    final y = topSide ? 8.0 : size.height - boxH - 8;
    canvas.drawRRect(
      RRect.fromRectAndRadius(
        Rect.fromLTWH(x, y, boxW, boxH),
        const Radius.circular(8),
      ),
      Paint()..color = theme.chipBg.withOpacity(0.92),
    );
    var cy = y + padV;
    for (final MapEntry(key: tp, value: color) in tpEntries.entries) {
      final linePaint = Paint()
        ..color = color
        ..strokeWidth = 2.5
        ..strokeCap = StrokeCap.round;
      canvas.drawLine(
        Offset(x + padH, cy + rowH / 2 - 3),
        Offset(x + padH + swatch, cy + rowH / 2 - 3),
        linePaint,
      );
      tp.paint(canvas, Offset(x + padH + swatch + 6, cy - 2));
      cy += rowH;
    }
  }

  // ---------- ابزار ----------

  /// خط‌چین دستی با PathMetrics — Canvas dash بومی ندارد.
  static void drawDashedPath(
    Canvas canvas,
    Path path,
    List<double> pattern,
    Paint paint,
  ) {
    for (final m in path.computeMetrics()) {
      var dist = 0.0;
      var on = true;
      var i = 0;
      while (dist < m.length) {
        final seg = pattern[i % pattern.length].clamp(0.5, 1000.0);
        final end = math.min(dist + seg, m.length);
        if (on) {
          canvas.drawPath(m.extractPath(dist, end), paint);
        }
        dist = end;
        on = !on;
        i++;
      }
    }
  }

  static void _dashedLine(Canvas canvas, Offset a, Offset b, Paint paint) {
    drawDashedPath(
      canvas,
      Path()
        ..moveTo(a.dx, a.dy)
        ..lineTo(b.dx, b.dy),
      const [2, 3],
      paint,
    );
  }

  @override
  bool shouldRepaint(covariant PlotPainter oldDelegate) =>
      oldDelegate.cell != cell ||
      oldDelegate.transform != transform ||
      oldDelegate.crosshair != crosshair ||
      oldDelegate.brightness != brightness;
}
