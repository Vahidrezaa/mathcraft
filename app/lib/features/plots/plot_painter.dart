import 'dart:math' as math;

import 'package:flutter/material.dart';

import '../../core/cds/cds_tokens.dart';
import '../../src/rust/api.dart' as api;
import 'plot_format.dart';
import 'plot_colormap.dart';
import 'plot_geometry.dart';

/// رنگ‌های نقاش برحسب روشنایی تم — همه از توکن‌های CDS (anti-pattern #2).
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

/// نقاش یک سلول ۲بعدی — رندر مستقیم از viewport (بازطراحی 3.4.1):
/// زوم/پن یعنی پنجرهٔ داده عوض می‌شود و تیک‌ها/برچسب‌ها/مقیاس در همین
/// فریم بازسازی می‌شوند؛ دیگر transform بوم با تیک‌های ثابت نداریم.
///
/// لایه‌های اطلاعاتی: crosshair + تگ مقدار روی هر دو محور، data-tip
/// پایدار (تپ)، و chip همیشه‌نمای «x∈[..] · y∈[..]».
class PlotPainter extends CustomPainter {
  PlotPainter({
    required this.cell,
    required this.viewport,
    required this.brightness,
    this.crosshair,
    this.dataTip,
    this.viewOverride,
  });

  final PlotCellData cell;
  final PlotViewport viewport;
  final Brightness brightness;
  final CrosshairSnap? crosshair;
  final DataTip? dataTip;

  /// چرخش تعاملی سه‌بعدی (az, el) — null = نمای قرارداد (view از DTO).
  final (double, double)? viewOverride;

  static const _mono11 = TextStyle(
    fontFamily: 'JetBrainsMono',
    fontSize: 11,
    height: 1.0,
  );

  @override
  void paint(Canvas canvas, Size size) {
    final theme = PlotTheme.of(brightness);

    // پس‌زمینهٔ بوم (immersion — کل صفحه)
    canvas.drawRect(Offset.zero & size, Paint()..color = theme.bg);

    // پلار باید قبل از CellLayout.build برگردد — سلول پلار axes ندارد و
    // cell.axes! در build می‌ترکد (باگ رندر polarplot از 3.4 که هیچ رسپی
    // دستگاهی آن را نپوشیده بود؛ golden polar_rose گام 3.6 این ترتیب را
    // پین می‌کند).
    if (cell is PolarCellData) {
      _paintPolar(canvas, size, cell as PolarCellData, theme);
      _paintLegend(canvas, size, cell, theme);
      return;
    }
    // P3-b (گام 3.8): سلول‌های سه‌بعدی مسیر چیدمان دوبعدی ندارند
    final c = cell;
    if (c is SurfaceCellData) {
      _paintSurface3D(canvas, size, c, theme);
      return;
    }
    if (c is Plot3CellData) {
      _paintPlot3(canvas, size, c, theme);
      _paintLegend(canvas, size, c, theme);
      return;
    }

    final layout = CellLayout.build(cell, size, viewport);
    final rect = layout.plotRect;

    // شبکهٔ نقطه‌چین روی همهٔ تیک‌ها (فرعی‌های لگاریتمی هم) — سند ۰۶ §3
    if (cell.axes!.grid && cell.axes!.visible) {
      final gridPaint = Paint()
        ..color = theme.grid
        ..style = PaintingStyle.stroke
        ..strokeWidth = 1;
      for (final gx in layout.xGridXs) {
        final sx = layout.dataToScreen(gx, 0).dx;
        if (sx >= rect.left && sx <= rect.right) {
          _dashedLine(canvas, Offset(sx, rect.top), Offset(sx, rect.bottom),
              gridPaint);
        }
      }
      for (final gy in layout.yGridYs) {
        final sy = layout.dataToScreen(0, gy).dy;
        if (sy >= rect.top && sy <= rect.bottom) {
          _dashedLine(canvas, Offset(rect.left, sy), Offset(rect.right, sy),
              gridPaint);
        }
      }
    }

    // فریم + برچسب‌ها (برای viewport جاری — با زوم عوض می‌شوند)
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

    // داده — clip به ناحیهٔ رسم (خارج دید = بریده می‌شود)
    canvas.save();
    canvas.clipRect(rect);
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
      case ContourCellData():
        _paintContour(canvas, layout, cell as ContourCellData);
      default:
        break;
    }
    canvas.restore();

    // نوار رنگ کانتور — بیرون از clip
    if (cell is ContourCellData) {
      final cc = cell as ContourCellData;
      if (cc.colorbar) {
        _paintColorbar(
          canvas, rect, cc.colormap, cc.caxis ?? _zRangeOf(cc), theme,
        );
      }
    }

    // لایه‌های اطلاعاتی — بیرون از clip
    if (cell.axes!.visible) {
      if (dataTip != null) {
        _paintDataTip(canvas, layout, dataTip!, theme);
      }
      if (crosshair != null) {
        _paintCrosshair(canvas, layout, crosshair!, theme);
      }
      _paintStatusChip(canvas, layout, theme);
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
      _drawTickLabel(
        canvas,
        t.label!,
        Offset(sx, rect.bottom + 6),
        labelStyle,
        theme,
        TextAlign.center,
      );
    }
    for (final (t, sy) in layout.yTickYs) {
      canvas.drawLine(
        Offset(rect.left - 4, sy),
        Offset(rect.left, sy),
        tickPaint,
      );
      _drawTickLabel(
        canvas,
        t.label!,
        Offset(rect.left - 8, sy),
        labelStyle,
        theme,
        TextAlign.right,
      );
    }
  }

  /// برچسب تیک — لگاریتمی «10^{k}» با توان بالا (superscript).
  void _drawTickLabel(
    Canvas canvas,
    String label,
    Offset anchor,
    TextStyle style,
    PlotTheme theme,
    TextAlign align,
  ) {
    final (base, exponent) = parseLogLabel(label);
    if (exponent == null) {
      _drawText(canvas, label, anchor, style, align);
      return;
    }
    final baseTp = _textPainter('10', style);
    final expTp = _textPainter(
      exponent,
      style.copyWith(fontSize: 8),
    );
    final totalW = baseTp.width + expTp.width;
    final dx =
        align == TextAlign.right ? anchor.dx - totalW : anchor.dx - totalW / 2;
    baseTp.paint(canvas, Offset(dx, anchor.dy));
    expTp.paint(canvas, Offset(dx + baseTp.width, anchor.dy - 4));
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

  void _paintTitles(
      Canvas canvas, CellLayout layout, AxesData axes, PlotTheme theme) {
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
      // پس از rotate(-90°): محور x جدید رو به بالا
      tp.paint(canvas,
          Offset(-rect.left - 4 - tp.width, rect.center.dy - tp.width / 2));
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

  /// مسیر سری در فضای صفحه — NaN/Inf = گپ در خط (رفتار متلب).
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
    // v3: MarkerSize — مقیاس نسبت به پیش‌فرض (متلب: 6pt ≈ شعاع ۳ ما)
    final scale = s.markerSize == null || s.markerSize! <= 0
        ? 1.0
        : s.markerSize! / 6.0;
    for (var i = 0; i < s.x.length; i++) {
      final x = s.x[i];
      final y = s.y[i];
      if (!x.isFinite || !y.isFinite) continue;
      final p = layout.dataToScreen(x, y);
      _drawMarker(canvas, s.marker!, p, stroke, fill, scale);
    }
  }

  void _drawMarker(
    Canvas canvas,
    api.Marker marker,
    Offset p,
    Paint stroke,
    Paint fill, [
    double scale = 1.0,
  ]) {
    switch (marker) {
      case api.Marker.point:
        canvas.drawCircle(p, 1.6 * scale, fill);
      case api.Marker.circle:
        canvas.drawCircle(p, 3 * scale, stroke);
      case api.Marker.square:
        canvas.drawRect(
          Rect.fromCenter(
            center: p,
            width: 5.5 * scale,
            height: 5.5 * scale,
          ),
          stroke,
        );
      case api.Marker.diamond:
        canvas.drawPath(_diamond(p, scale), stroke);
      case api.Marker.cross:
        canvas.drawLine(Offset(p.dx - 3 * scale, p.dy - 3 * scale),
            Offset(p.dx + 3 * scale, p.dy + 3 * scale), stroke);
        canvas.drawLine(Offset(p.dx - 3 * scale, p.dy + 3 * scale),
            Offset(p.dx + 3 * scale, p.dy - 3 * scale), stroke);
      case api.Marker.plus:
        canvas.drawLine(Offset(p.dx - 3.5 * scale, p.dy),
            Offset(p.dx + 3.5 * scale, p.dy), stroke);
        canvas.drawLine(Offset(p.dx, p.dy - 3.5 * scale),
            Offset(p.dx, p.dy + 3.5 * scale), stroke);
      case api.Marker.star:
        for (var k = 0; k < 3; k++) {
          final a = k * math.pi / 3;
          final dx = math.cos(a) * 3.5 * scale;
          final dy = math.sin(a) * 3.5 * scale;
          canvas.drawLine(Offset(p.dx - dx, p.dy - dy),
              Offset(p.dx + dx, p.dy + dy), stroke);
        }
      case api.Marker.triangleUp:
        canvas.drawPath(_triangle(p, up: true, scale: scale), stroke);
      case api.Marker.triangleDown:
        canvas.drawPath(_triangle(p, up: false, scale: scale), stroke);
      case api.Marker.none:
        break;
    }
  }

  Path _diamond(Offset p, [double scale = 1.0]) => Path()
    ..moveTo(p.dx, p.dy - 4 * scale)
    ..lineTo(p.dx + 4 * scale, p.dy)
    ..lineTo(p.dx, p.dy + 4 * scale)
    ..lineTo(p.dx - 4 * scale, p.dy)
    ..close();

  Path _triangle(Offset p, {required bool up, double scale = 1.0}) {
    final dy = up ? -4.0 : 4.0;
    final dyBase = up ? 2.6 : -2.6;
    return Path()
      ..moveTo(p.dx, p.dy + dy * scale)
      ..lineTo(p.dx + 3.6 * scale, p.dy + dyBase * scale)
      ..lineTo(p.dx - 3.6 * scale, p.dy + dyBase * scale)
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
      canvas.drawPath(
          path, Paint()..color = poly.color.withOpacity(0.55));
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
        final xRight = b.x[i] + (i < (s.xPos?.length ?? 0) ? s.xPos![i] : 0.0);
        final left = layout.dataToScreen(b.x[i] - s.xNeg![i], b.y[i]);
        final right = layout.dataToScreen(xRight, b.y[i]);
        canvas.drawLine(left, right, paint);
        canvas.drawLine(Offset(left.dx, left.dy - cap),
            Offset(left.dx, left.dy + cap), paint);
        canvas.drawLine(Offset(right.dx, right.dy - cap),
            Offset(right.dx, right.dy + cap), paint);
      }
    }
  }

  // ---------- پلار ----------

  /// پلار — شبکهٔ حلقوی + ۸ ستون؛ x = θ (رادیان)، y = شعاع نرمال به rmax.
  void _paintPolar(
      Canvas canvas, Size size, PolarCellData cell, PlotTheme theme) {
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

  Offset _polarPoint(
      Offset center, double radius, double rMax, double theta, double r) {
    final rr = r / rMax * radius;
    return center + Offset(math.cos(theta), math.sin(theta)) * rr;
  }

  // ---------- لایه‌های اطلاعاتی (بازطراحی 3.4.1) ----------

  /// crosshair + chip عددی + تگ مقدار روی هر دو محور (خوانش مقیاس).
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
    // chip (x, y) کنار نقطه
    final text = '(${formatG4(snap.x)}, ${formatG4(snap.y)})';
    _paintChip(
      canvas,
      text: text,
      anchor: p + const Offset(10, -10),
      align: _ChipAlign.topLeft,
      rect: rect,
      style: _mono11.copyWith(color: theme.textPrimary),
      bg: theme.chipBg.withOpacity(0.95),
      theme: theme,
    );
    // تگ مقدار x زیر محور افقی (روی ردیف برچسب‌ها)
    _paintChip(
      canvas,
      text: formatG4(snap.x),
      anchor: Offset(p.dx, rect.bottom + 6),
      align: _ChipAlign.abovePoint,
      rect: rect,
      style: _mono11.copyWith(color: theme.accent),
      bg: theme.chipBg.withOpacity(0.95),
      border: theme.accent,
      theme: theme,
    );
    // تگ مقدار y کنار محور عمودی
    _paintChip(
      canvas,
      text: formatG4(snap.y),
      anchor: Offset(rect.left - 8, p.dy),
      align: _ChipAlign.leftOfPoint,
      rect: rect,
      style: _mono11.copyWith(color: theme.accent),
      bg: theme.chipBg.withOpacity(0.95),
      border: theme.accent,
      theme: theme,
    );
  }

  /// data-tip پایدار — حلقهٔ سری‌رنگ + جعبهٔ مقدار با نام سری.
  void _paintDataTip(
    Canvas canvas,
    CellLayout layout,
    DataTip tip,
    PlotTheme theme,
  ) {
    final p = layout.dataToScreen(tip.x, tip.y);
    final rect = layout.plotRect;
    canvas.drawCircle(
      p,
      6,
      Paint()
        ..color = tip.color
        ..style = PaintingStyle.stroke
        ..strokeWidth = 2,
    );
    canvas.drawCircle(p, 2.5, Paint()..color = tip.color);
    final seriesName = _seriesLabel(tip.seriesIndex);
    final valueText = '(${formatG4(tip.x)}, ${formatG4(tip.y)})';
    final valueTp = _textPainter(
        valueText, _mono11.copyWith(color: theme.textPrimary));
    final nameTp = seriesName == null
        ? null
        : _textPainter(seriesName, TextStyle(
            fontFamily: 'Vazirmatn',
            fontSize: 11,
            color: theme.textSecondary,
          ));
    const padH = 8.0;
    const padV = 5.0;
    final w = math.max(valueTp.width, nameTp?.width ?? 0) + padH * 2;
    final h =
        valueTp.height + (nameTp == null ? 0 : nameTp.height + 2) + padV * 2;
    // خط راهنما از نقطه به جعبه (بالا-راست، fallback پایین-راست)
    var topLeft = p + const Offset(14, -14) - Offset(0, h);
    if (topLeft.dy < rect.top + 2) {
      topLeft = p + const Offset(14, 14);
    }
    topLeft = Offset(
      topLeft.dx.clamp(rect.left + 2, rect.right - w - 2),
      topLeft.dy.clamp(rect.top + 2, rect.bottom - h - 2),
    );
    final leader = Paint()
      ..color = tip.color.withOpacity(0.7)
      ..strokeWidth = 1;
    canvas.drawLine(p, topLeft + Offset(w / 2, h / 2), leader);
    final rrect = RRect.fromRectAndRadius(
      Rect.fromLTWH(topLeft.dx, topLeft.dy, w, h),
      const Radius.circular(8),
    );
    canvas.drawRRect(rrect, Paint()..color = theme.chipBg.withOpacity(0.95));
    canvas.drawRRect(
      rrect,
      Paint()
        ..color = tip.color
        ..style = PaintingStyle.stroke
        ..strokeWidth = 1.2,
    );
    var cy = topLeft.dy + padV;
    nameTp?.paint(canvas, Offset(topLeft.dx + padH, cy));
    if (nameTp != null) cy += nameTp.height + 2;
    valueTp.paint(canvas, Offset(topLeft.dx + padH, cy));
  }

  String? _seriesLabel(int seriesIndex) {
    final legend = cell.legend;
    if (legend == null || seriesIndex >= legend.entries.length) return null;
    return legend.entries[seriesIndex];
  }

  /// chip همیشه‌نما: «x ∈ [a, b] · y ∈ [c, d]» — پاسخ مستقیم به
  /// «مقیاس افتضاح»؛ همیشه می‌دانی چه بازه‌ای را می‌بینی.
  void _paintStatusChip(Canvas canvas, CellLayout layout, PlotTheme theme) {
    final vp = layout.viewport;
    final rect = layout.plotRect;
    final text =
        'x: [${formatG4(vp.xMin)}, ${formatG4(vp.xMax)}]  y: [${formatG4(vp.yMin)}, ${formatG4(vp.yMax)}]';
    final tp = _textPainter(
      text,
      _mono11.copyWith(fontSize: 10, color: theme.textSecondary),
    );
    const padH = 8.0;
    const padV = 4.0;
    final w = tp.width + padH * 2;
    final h = tp.height + padV * 2;
    final topLeft =
        Offset(rect.right - w - 6, rect.bottom - h - 6);
    final rrect = RRect.fromRectAndRadius(
      Rect.fromLTWH(topLeft.dx, topLeft.dy, w, h),
      const Radius.circular(6),
    );
    canvas.drawRRect(
      rrect,
      Paint()..color = theme.chipBg.withOpacity(0.75),
    );
    tp.paint(canvas, Offset(topLeft.dx + padH, topLeft.dy + padV));
  }


  // =====================================================================
  // P3-b (گام 3.8): سه‌بعدی — پروجکشن دوربین + الگوریتم نقاش + سایه صاف
  // (پورت وفادار mc_plot::pdf::project — سند ۰۴ §8.3: نرمال‌ها از موتور)
  // =====================================================================

  /// دامنهٔ z شبکه (برای caxis خودکار رنگ‌نوار).
  (double, double) _zRangeOf(ContourCellData c) {
    var lo = double.infinity, hi = double.negativeInfinity;
    for (final row in c.z) {
      for (final v in row) {
        if (!v.isFinite) continue;
        if (v < lo) lo = v;
        if (v > hi) hi = v;
      }
    }
    if (lo > hi) return (0, 1);
    return (lo, hi);
  }

  /// ناحیهٔ رسم سه‌بعدی: حاشیه + عنوان + نوار رنگ اختیاری.
  Rect _rect3D(Size size, bool hasColorbar, bool hasTitle) {
    var left = 56.0, top = 16.0, right = 20.0, bottom = 40.0;
    if (hasTitle) top += 26;
    if (hasColorbar) right += 64;
    return Rect.fromLTWH(
      left,
      top,
      (size.width - left - right).clamp(40.0, size.width),
      (size.height - top - bottom).clamp(40.0, size.height),
    );
  }

  /// جعبهٔ مختصات سه‌بعدی (۱۲ یال مکعب نرمال‌شده) + برچسب‌ها.
  void _paintBox3D(
    Canvas canvas,
    Rect rect,
    (double, double) view,
    Axes3DData axes,
    PlotTheme theme,
  ) {
    final edge = Paint()
      ..color = theme.border
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1;
    // رأس‌های مکعب [0,1]³
    Offset p3(double x, double y, double z) {
      final (sx, sy, d) = project3d(view.$1, view.$2, x, y, z);
      // مرکز‌چین و مقیاس ۰.۸۶ برای جای‌گیری جعبه در مستطیل
      final cx = rect.left + (sx * 0.86 + 0.07) * rect.width;
      final cy = rect.bottom - (sy * 0.86 + 0.07) * rect.height;
      return Offset(cx, cy);
    }

    // یال‌های پایینی (z=0) + عمودی‌ها + بالایی (z=1)
    final corners = <List<double>>[
      [0, 0, 0], [1, 0, 0], [1, 1, 0], [0, 1, 0],
      [0, 0, 1], [1, 0, 1], [1, 1, 1], [0, 1, 1],
    ];
    const edges = [
      (0, 1), (1, 2), (2, 3), (3, 0), // کف
      (4, 5), (5, 6), (6, 7), (7, 4), // سقف
      (0, 4), (1, 5), (2, 6), (3, 7), // عمودی
    ];
    for (final (a, b) in edges) {
      final ca = corners[a], cb = corners[b];
      canvas.drawLine(
        p3(ca[0], ca[1], ca[2]),
        p3(cb[0], cb[1], cb[2]),
        edge,
      );
    }
    // شبکهٔ دیوارهای عقب (زیر پرچم grid)
    if (axes.grid) {
      final grid = Paint()
        ..color = theme.grid
        ..style = PaintingStyle.stroke
        ..strokeWidth = 0.6;
      for (var k = 1; k < 5; k++) {
        final t = k / 5;
        // دیوار z-x (y=0)
        canvas.drawLine(p3(t, 0, 0), p3(t, 0, 1), grid);
        canvas.drawLine(p3(0, 0, t), p3(1, 0, t), grid);
        // دیوار z-y (x=0)
        canvas.drawLine(p3(0, t, 0), p3(0, t, 1), grid);
        canvas.drawLine(p3(0, 0, t), p3(0, 1, t), grid);
      }
    }
    // برچسب‌ها — نزدیک وسط یال مربوطه
    final label = _mono11.copyWith(color: theme.textSecondary);
    void text(String? t, Offset pos) {
      if (t == null || t.isEmpty) return;
      final tp = TextPainter(
        text: TextSpan(text: t, style: label),
        textDirection: TextDirection.ltr,
      )..layout();
      tp.paint(canvas, pos);
    }

    text(axes.xlabel, p3(0.5, 0, -0.06) + const Offset(-10, 4));
    text(axes.ylabel, p3(-0.02, 0.5, 0) + const Offset(-34, 0));
    text(axes.zlabel, p3(0, 1.02, 0.5) + const Offset(4, -6));
  }

  void _paintSurface3D(
    Canvas canvas,
    Size size,
    SurfaceCellData cell,
    PlotTheme theme,
  ) {
    final view = viewOverride ?? cell.axes3d.view;
    final rect = _rect3D(
      size,
      cell.colorbar,
      cell.axes3d.title != null,
    );
    _paintBox3D(canvas, rect, view, cell.axes3d, theme);

    final rows = cell.z.length;
    final cols = rows > 0 ? cell.z[0].length : 0;
    if (rows < 2 || cols < 2) return;

    // نرمال‌سازی داده به [0,1]³ (از حدود محور — همان Rust)
    final (xlo, xhi) = cell.axes3d.xLimits;
    final (ylo, yhi) = cell.axes3d.yLimits;
    final (zlo, zhi) = cell.axes3d.zLimits;
    final zSpan = (zhi - zlo) > 1e-12 ? (zhi - zlo) : 1.0;
    final caxis = cell.caxis ?? (zlo, zhi);
    final hasNormals = cell.vertexNormals.length >= rows * cols * 3;

    // پروجکشن همهٔ رأس‌ها
    final pts = List<Offset>.filled(rows * cols, Offset.zero);
    final depth = List<double>.filled(rows * cols, 0);
    for (var i = 0; i < rows; i++) {
      for (var j = 0; j < cols; j++) {
        final nx = (cell.x[j] - xlo) / ((xhi - xlo) > 1e-12 ? (xhi - xlo) : 1);
        final ny = (cell.y[i] - ylo) / ((yhi - ylo) > 1e-12 ? (yhi - ylo) : 1);
        final nz = (cell.z[i][j] - zlo) / zSpan;
        final (sx, sy, d) = project3d(view.$1, view.$2, nx, ny, nz);
        pts[i * cols + j] = Offset(
          rect.left + (sx * 0.86 + 0.07) * rect.width,
          rect.bottom - (sy * 0.86 + 0.07) * rect.height,
        );
        depth[i * cols + j] = d;
      }
    }

    // خانه‌ها بر عمق مرکز — دورها اول (الگوریتم نقاش)
    final quads = <(int, int, double)>[
      for (var i = 0; i < rows - 1; i++)
        for (var j = 0; j < cols - 1; j++)
          (i, j, depth[i * cols + j] + depth[(i + 1) * cols + j + 1]),
    ]..sort((a, b) => b.$3.compareTo(a.$3));

    final light = project3d(view.$1, view.$2, 0.5, 0.5, 2.0);
    final lightLen = math.sqrt(
      light.$1 * light.$1 + light.$2 * light.$2 + light.$3 * light.$3,
    );

    for (final (i, j, _) in quads) {
      final a = pts[i * cols + j];
      final b = pts[(i + 1) * cols + j];
      final c = pts[(i + 1) * cols + j + 1];
      final d = pts[i * cols + j + 1];
      final zc = (cell.z[i][j] +
              cell.z[i + 1][j] +
              cell.z[i][j + 1] +
              cell.z[i + 1][j + 1]) /
          4;

      if (!cell.isMesh) {
        // سایهٔ صاف: میانگین نرمال رأس‌ها → فضای دید → نقطه‌چین نور سرِ
        // دوربین؛ fallback نرمال (0,0,1) وقتی موتور نفرستاده (mesh)
        var shade = 1.0;
        if (hasNormals) {
          var nx = 0.0, ny = 0.0, nz = 0.0;
          for (final (vi, vj) in [(i, j), (i + 1, j), (i + 1, j + 1), (i, j + 1)]) {
            final base = (vi * cols + vj) * 3;
            nx += cell.vertexNormals[base];
            ny += cell.vertexNormals[base + 1];
            nz += cell.vertexNormals[base + 2];
          }
          final nLen = math.sqrt(nx * nx + ny * ny + nz * nz);
          if (nLen > 1e-9) {
            final rn = project3d(view.$1, view.$2, nx / nLen, ny / nLen, nz / nLen);
            final dot =
                (rn.$1 * light.$1 + rn.$2 * light.$2 + rn.$3 * light.$3) / lightLen;
            shade = 0.35 + 0.65 * math.max(0, dot);
          }
        }
        final base = sampleColormapZ(cell.colormap, zc, caxis);
        final face = Paint()
          ..color = Color.fromARGB(
            255,
            (base.red * shade).round().clamp(0, 255),
            (base.green * shade).round().clamp(0, 255),
            (base.blue * shade).round().clamp(0, 255),
          );
        final path = Path()
          ..moveTo(a.dx, a.dy)
          ..lineTo(b.dx, b.dy)
          ..lineTo(c.dx, c.dy)
          ..lineTo(d.dx, d.dy)
          ..close();
        canvas.drawPath(path, face);
        if (cell.shading == api.Shading.faceted) {
          canvas.drawPath(
            path,
            Paint()
              ..color = Colors.black.withOpacity(0.35)
              ..style = PaintingStyle.stroke
              ..strokeWidth = 0.5,
          );
        }
      } else {
        // mesh: سیم‌باف — یال‌های رنگی از z (وجه‌ها نامرئی)
        final wire = Paint()
          ..color = sampleColormapZ(cell.colormap, zc, caxis)
          ..style = PaintingStyle.stroke
          ..strokeWidth = 1;
        final path = Path()
          ..moveTo(a.dx, a.dy)
          ..lineTo(b.dx, b.dy)
          ..lineTo(c.dx, c.dy)
          ..lineTo(d.dx, d.dy)
          ..close();
        canvas.drawPath(path, wire);
      }
    }

    if (cell.colorbar) {
      _paintColorbar(canvas, rect, cell.colormap, cell.caxis ?? (zlo, zhi), theme);
    }
    if (cell.axes3d.title case final t?) {
      _paintTitle3D(canvas, size, t, theme);
    }
  }

  void _paintPlot3(
    Canvas canvas,
    Size size,
    Plot3CellData cell,
    PlotTheme theme,
  ) {
    final view = viewOverride ?? cell.axes3d.view;
    final rect = _rect3D(size, false, cell.axes3d.title != null);
    _paintBox3D(canvas, rect, view, cell.axes3d, theme);

    final (xlo, xhi) = cell.axes3d.xLimits;
    final (ylo, yhi) = cell.axes3d.yLimits;
    final (zlo, zhi) = cell.axes3d.zLimits;
    Offset p3(double x, double y, double z) {
      final nx = (x - xlo) / ((xhi - xlo) > 1e-12 ? (xhi - xlo) : 1);
      final ny = (y - ylo) / ((yhi - ylo) > 1e-12 ? (yhi - ylo) : 1);
      final nz = (z - zlo) / ((zhi - zlo) > 1e-12 ? (zhi - zlo) : 1);
      final (sx, sy, _) = project3d(view.$1, view.$2, nx, ny, nz);
      return Offset(
        rect.left + (sx * 0.86 + 0.07) * rect.width,
        rect.bottom - (sy * 0.86 + 0.07) * rect.height,
      );
    }

    for (final s in cell.series) {
      final paint = Paint()
        ..color = s.color
        ..style = PaintingStyle.stroke
        ..strokeWidth = s.width
        ..strokeCap = StrokeCap.round;
      if (s.dash != null && s.dash!.isNotEmpty) {
        // خط‌چین پاره‌خطی — همان الگوی _dashedLine دوبعدی
        Offset? last;
        for (var k = 0; k < s.x.length; k++) {
          if (!s.x[k].isFinite || !s.y[k].isFinite || !s.z[k].isFinite) {
            last = null;
            continue;
          }
          final p = p3(s.x[k], s.y[k], s.z[k]);
          if (last != null) _dashedLine(canvas, last, p, paint);
          last = p;
        }
      } else {
        var pen = false;
        var last = Offset.zero;
        for (var k = 0; k < s.x.length; k++) {
          if (!s.x[k].isFinite || !s.y[k].isFinite || !s.z[k].isFinite) {
            pen = false;
            continue;
          }
          final p = p3(s.x[k], s.y[k], s.z[k]);
          if (pen) canvas.drawLine(last, p, paint);
          last = p;
          pen = true;
        }
      }
    }
    if (cell.axes3d.title case final t?) {
      _paintTitle3D(canvas, size, t, theme);
    }
  }

  void _paintTitle3D(Canvas canvas, Size size, String text, PlotTheme theme) {
    final tp = TextPainter(
      text: TextSpan(text: text, style: _mono11.copyWith(color: theme.textPrimary, fontSize: 13, fontWeight: FontWeight.w600)),
      textDirection: TextDirection.ltr,
    )..layout(maxWidth: size.width - 32);
    tp.paint(
      canvas,
      Offset(size.width / 2 - tp.width / 2, 6),
    );
  }

  // ---------------------------------------------------------------------
  // کانتور (v3): پُرکردن نوار-سلولی + خطوط سطح marching squares
  // (پورت وفادار mc_plot::pdf::marching_cell — الگوی تیک 1-2-5)
  // ---------------------------------------------------------------------

  void _paintContour(Canvas canvas, CellLayout layout, ContourCellData cell) {
    final rows = cell.z.length;
    final cols = rows > 0 ? cell.z[0].length : 0;
    if (rows < 2 || cols < 2) return;
    final caxis = cell.caxis ?? _zRangeOf(cell);

    if (cell.filled) {
      // پُرکردن تقریبی نوار-سلولی + روکش پاره‌خط‌های هم‌سو (مرز نرم‌نما)
      for (var i = 0; i < rows - 1; i++) {
        for (var j = 0; j < cols - 1; j++) {
          final zc = (cell.z[i][j] +
                  cell.z[i][j + 1] +
                  cell.z[i + 1][j] +
                  cell.z[i + 1][j + 1]) /
              4;
          final paint = Paint()..color = sampleColormapZ(cell.colormap, zc, caxis);
          final a = layout.dataToScreen(cell.x[j], cell.y[i]);
          final c = layout.dataToScreen(cell.x[j + 1], cell.y[i + 1]);
          canvas.drawRect(
            Rect.fromLTRB(a.dx, a.dy, c.dx, c.dy),
            paint,
          );
        }
      }
    }

    // خطوط سطح — یک رنگ بر سطح، پاره‌خط‌های یکجا
    for (final level in cell.levels) {
      final t = caxis.$2 > caxis.$1
          ? (level - caxis.$1) / (caxis.$2 - caxis.$1)
          : 0.5;
      final paint = Paint()
        ..color = sampleColormap(cell.colormap, t)
        ..style = PaintingStyle.stroke
        ..strokeWidth = cell.filled ? 0.8 : 1.4;
      final path = Path();
      for (var i = 0; i < rows - 1; i++) {
        for (var j = 0; j < cols - 1; j++) {
          _marchingCell(
            layout,
            cell.x[j], cell.x[j + 1], cell.y[i], cell.y[i + 1],
            cell.z[i][j], cell.z[i][j + 1],
            cell.z[i + 1][j + 1], cell.z[i + 1][j],
            level, path,
          );
        }
      }
      canvas.drawPath(path, paint);
    }
  }

  /// یک خانهٔ دوبی‌خطی بر سطح L — پاره‌خط به path اضافه می‌شود.
  void _marchingCell(
    CellLayout layout,
    double x0, double x1, double y0, double y1,
    double z00, double z10, double z11, double z01,
    double level,
    Path path,
  ) {
    double? lerp(double a, double b, double za, double zb) {
      if ((za < level && zb < level) || (za > level && zb > level)) return null;
      final t = (zb - za).abs() < 1e-15
          ? 0.5
          : ((level - za) / (zb - za)).clamp(0.0, 1.0);
      return a + t * (b - a);
    }

    final pts = <Offset>[
      if (lerp(x0, x1, z00, z10) case final x?)
        layout.dataToScreen(x, y0),
      if (lerp(y0, y1, z10, z11) case final y?)
        layout.dataToScreen(x1, y),
      if (lerp(x1, x0, z11, z01) case final x?)
        layout.dataToScreen(x, y1),
      if (lerp(y0, y1, z00, z01) case final y?)
        layout.dataToScreen(x0, y),
    ];
    // نقاط منطبق گوشه‌ای را یکی کن (برخورد سطح با رأس)
    final dedup = <Offset>[];
    for (final p in pts) {
      if (!dedup.any((q) => (q - p).distanceSquared < 1e-16)) dedup.add(p);
    }
    if (dedup.length == 2) {
      path
        ..moveTo(dedup[0].dx, dedup[0].dy)
        ..lineTo(dedup[1].dx, dedup[1].dy);
    }
  }

  /// نوار رنگ عمودی سمت راست + سه برچسب مقدار.
  void _paintColorbar(
    Canvas canvas,
    Rect plotRect,
    api.Colormap cm,
    (double, double) caxis,
    PlotTheme theme,
  ) {
    const steps = 64;
    final left = plotRect.right + 10;
    const width = 12.0;
    for (var k = 0; k < steps; k++) {
      final t = k / steps;
      final top = plotRect.bottom - (t + 1 / steps) * plotRect.height;
      final h = plotRect.height / steps + 0.6;
      canvas.drawRect(
        Rect.fromLTWH(left, top, width, h),
        Paint()..color = sampleColormap(cm, t),
      );
    }
    canvas.drawRect(
      Rect.fromLTWH(left, plotRect.top, width, plotRect.height),
      Paint()
        ..color = theme.border
        ..style = PaintingStyle.stroke
        ..strokeWidth = 0.8,
    );
    for (final (t, v) in [
      (0.0, caxis.$1),
      (0.5, (caxis.$1 + caxis.$2) / 2),
      (1.0, caxis.$2),
    ]) {
      final tp = TextPainter(
        text: TextSpan(text: formatG4(v), style: _mono11.copyWith(color: theme.textSecondary, fontSize: 10)),
        textDirection: TextDirection.ltr,
      )..layout();
      final y = plotRect.bottom - t * plotRect.height - tp.height / 2;
      tp.paint(canvas, Offset(left + width + 4, y.clamp(plotRect.top, plotRect.bottom - tp.height)));
    }
  }

  void _paintLegend(
      Canvas canvas, Size size, PlotCellData cell, PlotTheme theme) {
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

  void _paintChip(
    Canvas canvas, {
    required String text,
    required Offset anchor,
    required _ChipAlign align,
    required Rect rect,
    required TextStyle style,
    required Color bg,
    PlotTheme? theme,
    Color? border,
  }) {
    final tp = _textPainter(text, style);
    const padH = 8.0;
    const padV = 5.0;
    final w = tp.width + padH * 2;
    final h = tp.height + padV * 2;
    Offset topLeft;
    switch (align) {
      case _ChipAlign.topLeft:
        topLeft = anchor;
      case _ChipAlign.abovePoint:
        topLeft = Offset(anchor.dx - w / 2, anchor.dy);
      case _ChipAlign.leftOfPoint:
        topLeft = Offset(anchor.dx - w, anchor.dy - h / 2);
    }
    topLeft = Offset(
      topLeft.dx.clamp(rect.left + 2, rect.right - w - 2),
      topLeft.dy.clamp(rect.top + 2, rect.bottom - h - 2),
    );
    final rrect = RRect.fromRectAndRadius(
      Rect.fromLTWH(topLeft.dx, topLeft.dy, w, h),
      const Radius.circular(8),
    );
    canvas.drawRRect(rrect, Paint()..color = bg);
    if (border != null) {
      canvas.drawRRect(
        rrect,
        Paint()
          ..color = border
          ..style = PaintingStyle.stroke
          ..strokeWidth = 1,
      );
    }
    tp.paint(canvas, Offset(topLeft.dx + padH, topLeft.dy + padV));
  }

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
      oldDelegate.viewport != viewport ||
      oldDelegate.crosshair != crosshair ||
      oldDelegate.dataTip != dataTip ||
      oldDelegate.brightness != brightness;
}

enum _ChipAlign { topLeft, abovePoint, leftOfPoint }
