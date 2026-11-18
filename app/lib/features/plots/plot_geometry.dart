import 'dart:math' as math;

import 'package:flutter/material.dart';

import '../../src/rust/api.dart' as api;
import 'plot_format.dart';
import 'plot_palette.dart';

/// مدل میانی خنثی از قرارداد (freezed generated) — نقاش فقط با این کار
/// می‌کند تا هندسه بدون canvas و بدون union-walking تست شود (سند ۰۸).
///
/// گونه‌های ۳بعدی (Surface/Mesh/Contour/Plot3) و Quiver این‌جا `null`
/// می‌شوند → صفحه حالت صادقانهٔ «به‌روزرسانی بعدی» می‌دهد (P3-b).

/// یک سری خطی‌گونه برای رسم و crosshair.
class SeriesGeom {
  SeriesGeom({
    required this.id,
    required this.x,
    required this.y,
    required this.color,
    required this.width,
    required this.dash,
    this.marker,
  });

  final int id;
  final List<double> x;
  final List<double> y;
  final Color color;
  final double width;

  /// null = صاف؛ خالی = بدون خط (فقط مارکر).
  final List<double>? dash;
  final api.Marker? marker;
}

/// محورهای حل‌شدهٔ یک سلول — حدود «خانه» (نمای اولیه از موتور).
class AxesData {
  AxesData({
    required this.title,
    required this.xlabel,
    required this.ylabel,
    required this.grid,
    required this.xLimits,
    required this.yLimits,
    required this.equalAspect,
    required this.logX,
    required this.logY,
    required this.visible,
    required this.xTicks,
    required this.yTicks,
  });

  final String? title;
  final String? xlabel;
  final String? ylabel;
  final bool grid;
  final (double, double) xLimits;
  final (double, double) yLimits;
  final bool equalAspect;
  final bool logX;
  final bool logY;
  final bool visible;

  /// تیک‌های موتور — فقط مرجع قرارداد؛ رندر تعاملی تیک‌های خودش را
  /// برای viewport جاری می‌سازد (بازطراحی 3.4.1).
  final List<api.Tick> xTicks;
  final List<api.Tick> yTicks;
}

/// ورودی legend — entries با رنگ سری‌ها به‌ترتیب zip می‌شود.
class LegendData {
  LegendData(this.entries, this.position);
  final List<String> entries;
  final api.LegendPosition position;
}

sealed class PlotCellData {
  AxesData? get axes => null;
  LegendData? get legend => null;

  /// سری‌های خطی‌گونه (هدف crosshair/data-tip).
  List<SeriesGeom> get crosshairSeries => const [];
}

class LineCellData extends PlotCellData {
  LineCellData({
    required this.series,
    required this.axes,
    this.legend,
  });

  @override
  final AxesData axes;
  @override
  final LegendData? legend;
  final List<SeriesGeom> series;

  @override
  List<SeriesGeom> get crosshairSeries => series;
}

class BarCellData extends PlotCellData {
  BarCellData({
    required this.series,
    required this.width,
    required this.horizontal,
    required this.stacked,
    required this.axes,
    this.legend,
  });

  final List<SeriesGeom> series;
  final double width;
  final bool horizontal;
  final bool stacked;
  @override
  final AxesData axes;
  @override
  final LegendData? legend;
}

class HistCellData extends PlotCellData {
  HistCellData({
    required this.binEdges,
    required this.counts,
    required this.color,
    required this.axes,
    this.legend,
  });

  final List<double> binEdges;
  final List<double> counts;
  final Color color;
  @override
  final AxesData axes;
  @override
  final LegendData? legend;
}

class AreaCellData extends PlotCellData {
  AreaCellData({
    required this.series,
    required this.baseline,
    required this.axes,
    this.legend,
  });

  final List<SeriesGeom> series;
  final double baseline;
  @override
  final AxesData axes;
  @override
  final LegendData? legend;

  @override
  List<SeriesGeom> get crosshairSeries => series;
}

class FillCellData extends PlotCellData {
  FillCellData({
    required this.polys,
    required this.axes,
  });

  final List<SeriesGeom> polys;
  @override
  final AxesData axes;

  @override
  List<SeriesGeom> get crosshairSeries => polys;
}

class PolarCellData extends PlotCellData {
  PolarCellData({required this.series, this.legend});

  final List<SeriesGeom> series;
  @override
  final LegendData? legend;

  @override
  List<SeriesGeom> get crosshairSeries => series;
}

class ErrorCellData extends PlotCellData {
  ErrorCellData({
    required this.series,
    required this.axes,
    this.legend,
  });

  final List<SeriesWithBand> series;
  @override
  final AxesData axes;
  @override
  final LegendData? legend;

  @override
  List<SeriesGeom> get crosshairSeries =>
      series.map((s) => s.base).toList();
}

/// سری errorbar: پایه + نوارهای قرینه/ناقرینه (null = بدون خطای آن محور).
class SeriesWithBand {
  SeriesWithBand({
    required this.base,
    required this.xNeg,
    required this.xPos,
    required this.yNeg,
    required this.yPos,
  });

  final SeriesGeom base;
  final List<double>? xNeg, xPos, yNeg, yPos;
}

/// استخراج مدل میانی از قرارداد؛ null برای گونه‌های هنوز بی‌رندر (3D/Quiver).
PlotCellData? cellDataFromSpec(api.PlotSpec spec) {
  AxesData axesOf(api.Axes2D a) => AxesData(
        title: a.title,
        xlabel: a.xlabel,
        ylabel: a.ylabel,
        grid: a.grid,
        xLimits: a.xLimits == null
            ? (0, 1)
            : (a.xLimits!.min, a.xLimits!.max),
        yLimits: a.yLimits == null
            ? (0, 1)
            : (a.yLimits!.min, a.yLimits!.max),
        equalAspect: a.equalAspect,
        logX: a.logX,
        logY: a.logY,
        visible: a.visible,
        xTicks: a.xTicks,
        yTicks: a.yTicks,
      );
  LegendData? legendOf(api.Legend? l) =>
      l == null ? null : LegendData(l.entries, l.position);
  List<SeriesGeom> seriesOf(Iterable<api.Series2D> ss) => ss
      .map((s) => SeriesGeom(
            id: s.id.toInt(),
            x: s.x.toList(),
            y: s.y.toList(),
            color: resolvePlotColor(s.style.color),
            width: s.style.width,
            dash: dashPatternFor(s.style.dash),
            marker:
                s.style.marker == api.Marker.none ? null : s.style.marker,
          ))
      .toList();

  return switch (spec) {
    api.PlotSpec_Line2D() => LineCellData(
        series: seriesOf(spec.series),
        axes: axesOf(spec.axes),
        legend: legendOf(spec.legend),
      ),
    api.PlotSpec_Scatter2D() => LineCellData(
        series: seriesOf(spec.series),
        axes: axesOf(spec.axes),
        legend: legendOf(spec.legend),
      ),
    api.PlotSpec_Bar2D() => BarCellData(
        series: seriesOf(spec.series),
        width: spec.width,
        horizontal: spec.horizontal,
        stacked: spec.stacked,
        axes: axesOf(spec.axes),
        legend: legendOf(spec.legend),
      ),
    api.PlotSpec_Histogram() => HistCellData(
        binEdges: spec.binEdges.toList(),
        counts: spec.counts.toList(),
        color: resolvePlotColor(spec.style.color),
        axes: axesOf(spec.axes),
        legend: legendOf(spec.legend),
      ),
    api.PlotSpec_Area2D() => AreaCellData(
        series: seriesOf(spec.series),
        baseline: spec.baseline,
        axes: axesOf(spec.axes),
        legend: legendOf(spec.legend),
      ),
    api.PlotSpec_Fill2D() => FillCellData(
        polys: seriesOf(spec.series),
        axes: axesOf(spec.axes),
      ),
    api.PlotSpec_Polar() => PolarCellData(
        series: seriesOf(spec.series),
        legend: legendOf(spec.legend),
      ),
    api.PlotSpec_ErrorBar2D() => ErrorCellData(
        series: spec.series
            .map((s) => SeriesWithBand(
                  base: SeriesGeom(
                    id: s.id.toInt(),
                    x: s.x.toList(),
                    y: s.y.toList(),
                    color: resolvePlotColor(s.style.color),
                    width: s.style.width,
                    dash: dashPatternFor(s.style.dash),
                    marker: s.style.marker == api.Marker.none
                        ? null
                        : s.style.marker,
                  ),
                  xNeg: s.xErr?.neg.toList(),
                  xPos: s.xErr?.pos.toList(),
                  yNeg: s.yErr?.neg.toList(),
                  yPos: s.yErr?.pos.toList(),
                ))
            .toList(),
        axes: axesOf(spec.axes),
        legend: legendOf(spec.legend),
      ),
    api.PlotSpec_Quiver() ||
    api.PlotSpec_Surface() ||
    api.PlotSpec_Mesh() ||
    api.PlotSpec_Contour() ||
    api.PlotSpec_Plot3() =>
      null,
    api.PlotSpec_Subplot() => null, // سطح صفحه مدیریت می‌شود
  };
}

// ---------------------------------------------------------------------------
// تیک‌های زیبا — پورت وفادار crates/mc-plot/src/ticks.rs (آینهٔ 1-2-5)
// ---------------------------------------------------------------------------

/// گام «زیبا» از خانوادهٔ 1-2-5: کوچک‌ترین 10^k×{1,2,5,10} که ≥ آرمان است.
double niceStep(double ideal) {
  if (ideal <= 0 || !ideal.isFinite) return 1.0;
  final mag = _pow10(_floorLog10(ideal));
  for (final mult in const [1.0, 2.0, 5.0, 10.0]) {
    if (mult * mag >= ideal - mag * 1e-9) {
      return mult * mag;
    }
  }
  return 10.0 * mag;
}

/// تیک‌های خطی 1-2-5 در بازهٔ [min, max] — مرتب و بدون تکرار مجاور.
List<double> linearTicks(double min, double max, int maxTicks) {
  if (!min.isFinite || !max.isFinite || max <= min) return [min];
  final step = niceStep((max - min) / math.max(maxTicks, 2));
  final first = (min / step).ceilToDouble();
  final last = (max / step).floorToDouble();
  final out = <double>[];
  for (var k = first; k <= last + 1e-9; k += 1.0) {
    final v = k * step;
    if (v.isFinite) out.add(v);
  }
  final dedup = <double>[];
  for (final v in out) {
    if (dedup.isEmpty || (v - dedup.last).abs() >= step * 1e-9) {
      dedup.add(v);
    }
  }
  return dedup;
}

/// تیک‌های لگاریتمی: اصلی = توان‌های ۱۰ با برچسب `10^{k}`؛ بازهٔ باریک‌تر
/// از یک دهه = فرعی‌های بی‌برچسب ۲..۹×دهه (پورت log_ticks).
List<api.Tick> logTicks(double min, double max) {
  if (!min.isFinite || !max.isFinite || max <= min || min <= 0) {
    return const [];
  }
  final k0 = _ceilLog10(min);
  final k1 = _floorLog10(max);
  final majors = <int>[for (var k = k0; k <= k1; k++) k];
  if (majors.length < 2) {
    majors.clear();
  }
  if (majors.isNotEmpty) {
    return [
      for (final k in majors)
        api.Tick(position: _pow10(k), label: '10^{$k}'),
    ];
  }
  final out = <api.Tick>[];
  var dec = _floorLog10(min);
  final decEnd = _ceilLog10(max);
  while (dec <= decEnd) {
    for (var m = 2; m < 10; m++) {
      final v = m * _pow10(dec);
      if (v >= min && v <= max) {
        out.add(api.Tick(position: v, label: null));
      }
    }
    dec += 1;
  }
  return out;
}

/// تیک‌های کامل یک محور (خطی/لگاریتمی) با برچسب g4 — هم‌ارز axis_ticks.
List<api.Tick> axisTicks(double min, double max, bool log, int maxTicks) {
  if (log) return logTicks(min, max);
  return [
    for (final p in linearTicks(min, max, maxTicks))
      api.Tick(position: p, label: formatG4(p)),
  ];
}

/// تعداد هدف تیک از عرض پیکسلی — رفتار متلب: ~۶ تیک روی ~۳۰۰px.
int tickCountFor(double pixelSpan, [double targetSpacing = 56]) {
  if (!pixelSpan.isFinite || pixelSpan <= 0) return 3;
  return (pixelSpan / targetSpacing).floor().clamp(3, 10);
}

int _floorLog10(double v) => _log10(v).floor();
int _ceilLog10(double v) => _log10(v).ceil();

/// Dart تابع log10 بومی ندارد و ln/ln10 روی توان‌های دقیق ۱۰ خطای
/// گرد کردن دارد (مثل 1000 → 2.999…6) — به توان دقیق snap می‌کنیم تا
/// با Rust log10 (گردشده) هم‌ارز بمانیم.
double _log10(double v) {
  final r = math.log(v) / math.ln10;
  final ri = r.round();
  if ((r - ri).abs() < 1e-12 && v == _pow10(ri)) {
    return ri.toDouble();
  }
  return r;
}

double _pow10(int exp) {
  var result = 1.0;
  for (var i = 0; i < exp.abs(); i++) {
    result *= 10;
  }
  return exp < 0 ? 1 / result : result;
}

// ---------------------------------------------------------------------------
// viewport در فضای داده — تنها منبع حقیقت زوم/پن (بازطراحی 3.4.1)
// ---------------------------------------------------------------------------

/// پنجرهٔ دید در مختصات داده. زوم/پن این را تغییر می‌دهد و تیک‌ها در
/// هر فریم برای همین پنجره ساخته می‌شوند — دیگر «canvas transform با
/// تیک‌های ثابت» نداریم (عیب 3.4).
class PlotViewport {
  const PlotViewport({
    required this.xMin,
    required this.xMax,
    required this.yMin,
    required this.yMax,
  });

  /// نمای خانه — حدود fit موتور.
  factory PlotViewport.home(AxesData axes) => PlotViewport(
        xMin: axes.xLimits.$1,
        xMax: axes.xLimits.$2,
        yMin: axes.yLimits.$1,
        yMax: axes.yLimits.$2,
      );

  /// جایگزین بی‌معنا (پلار — viewport ندارد).
  static const any = PlotViewport(xMin: 0, xMax: 1, yMin: 0, yMax: 1);

  final double xMin, xMax, yMin, yMax;

  double get xSpan => xMax - xMin;
  double get ySpan => yMax - yMin;
  double get xCenter => (xMin + xMax) / 2;
  double get yCenter => (yMin + yMax) / 2;

  bool get isValid => xSpan.isFinite && ySpan.isFinite && xSpan > 0 && ySpan > 0;

  /// زوم حول یک نقطهٔ داده‌ای (focal) — همان نقطه زیر انگشت سر جایش
  /// می‌ماند: فاصله‌های focal با ۱/factor کوچک/بزرگ می‌شوند.
  PlotViewport zoomAt({
    required double focalX,
    required double focalY,
    required double factor,
  }) {
    if (!factor.isFinite || factor <= 0) return this;
    return PlotViewport(
      xMin: focalX - (focalX - xMin) / factor,
      xMax: focalX + (xMax - focalX) / factor,
      yMin: focalY - (focalY - yMin) / factor,
      yMax: focalY + (yMax - focalY) / factor,
    );
  }

  /// پن به پیکسل — محتوا با انگشت حرکت می‌کند (درگ راست = دید به چپ).
  PlotViewport panPixels(Offset deltaPx, Size viewSize) {
    if (viewSize.width <= 0 || viewSize.height <= 0) return this;
    return PlotViewport(
      xMin: xMin - deltaPx.dx * xSpan / viewSize.width,
      xMax: xMax - deltaPx.dx * xSpan / viewSize.width,
      yMin: yMin + deltaPx.dy * ySpan / viewSize.height,
      yMax: yMax + deltaPx.dy * ySpan / viewSize.height,
    );
  }

  /// حد معقول دید: span هر محور در [خانه/۱۰۰۰۰، خانه×۴] — مرکز آزاد
  /// (مثل متلب می‌شود به فضای خالی هم پن کرد).
  PlotViewport clamped(PlotViewport home) {
    if (!isValid) return home.isValid ? home : this;
    final xS = xSpan.clamp(home.xSpan / 10000, home.xSpan * 4);
    final yS = ySpan.clamp(home.ySpan / 10000, home.ySpan * 4);
    return PlotViewport(
      xMin: xCenter - xS / 2,
      xMax: xCenter + xS / 2,
      yMin: yCenter - yS / 2,
      yMax: yCenter + yS / 2,
    );
  }

  @override
  bool operator ==(Object other) =>
      other is PlotViewport &&
      other.xMin == xMin &&
      other.xMax == xMax &&
      other.yMin == yMin &&
      other.yMax == yMax;

  @override
  int get hashCode => Object.hash(xMin, xMax, yMin, yMax);
}

// ---------------------------------------------------------------------------
// layout خالص — mapping و snap و tip (بدون canvas؛ تست‌پذیر کامل)
// ---------------------------------------------------------------------------

/// layout یک سلول ۲بعدی — تیک‌ها برای viewport جاری در Dart ساخته
/// می‌شوند (بازطراحی 3.4.1: مقیاس همیشه با دید هم‌خوان است).
class CellLayout {
  CellLayout({
    required this.plotRect,
    required this.xTickXs,
    required this.yTickYs,
    required this.xGridXs,
    required this.yGridYs,
    required this.viewport,
  });

  /// ناحیهٔ محورها (داخل حاشیه‌های برچسب‌ها).
  final Rect plotRect;

  /// تیک‌های برچسب‌دار + موقعیت صفحه.
  final List<(api.Tick, double)> xTickXs;
  final List<(api.Tick, double)> yTickYs;

  /// همهٔ تیک‌ها (فرعی‌های لگاریتمی هم) — فقط برای grid.
  final List<double> xGridXs;
  final List<double> yGridYs;

  final PlotViewport viewport;

  double _xToScreen(double x) => linearMap(
      x, viewport.xMin, viewport.xMax, plotRect.left, plotRect.right);

  double _yToScreen(double y) => linearMap(
      y, viewport.yMin, viewport.yMax, plotRect.bottom, plotRect.top);

  Offset dataToScreen(double x, double y) =>
      Offset(_xToScreen(x), _yToScreen(y));

  /// وارون مپینگ — مختصات داده برای crosshair/ژست‌ها.
  (double, double) screenToData(Offset p) => (
        linearMap(p.dx, plotRect.left, plotRect.right,
            viewport.xMin, viewport.xMax),
        linearMap(p.dy, plotRect.bottom, plotRect.top,
            viewport.yMin, viewport.yMax),
      );

  /// layout کامل از مدل سلول + viewport جاری.
  factory CellLayout.build(
      PlotCellData cell, Size size, PlotViewport viewport) {
    final axes = cell.axes!;
    if (!axes.visible) {
      // axis off — فقط داده، بدون تزئینات (حاشیهٔ حداقلی)
      return CellLayout(
        plotRect: Rect.fromLTWH(2, 2, size.width - 4, size.height - 4),
        xTickXs: const [],
        yTickYs: const [],
        xGridXs: const [],
        yGridYs: const [],
        viewport: viewport,
      );
    }
    // گذر ۱: تیک y از viewport (مستقل از rect) → حاشیهٔ چپ از برچسب‌ها
    final yTicks = axisTicks(viewport.yMin, viewport.yMax, axes.logY,
        tickCountFor(size.height - 44));
    var left = 8.0;
    for (final t in yTicks) {
      final label = t.label;
      if (label != null) {
        left = math.max(left, _measureMono(label) + 14);
      }
    }
    const top = 10.0;
    const bottom = 30.0;
    const right = 12.0;
    final rect = Rect.fromLTWH(
      left,
      top,
      math.max(10, size.width - left - right),
      math.max(10, size.height - top - bottom),
    );
    // گذر ۲: تیک x با عرض واقعی rect
    final xTicks = axisTicks(
        viewport.xMin, viewport.xMax, axes.logX, tickCountFor(rect.width));

    final probe = CellLayout(
      plotRect: rect,
      xTickXs: const [],
      yTickYs: const [],
      xGridXs: const [],
      yGridYs: const [],
      viewport: viewport,
    );
    final xTickXs = <(api.Tick, double)>[];
    for (final t in xTicks) {
      if (t.label != null) {
        xTickXs.add((t, probe._xToScreen(t.position)));
      }
    }
    final yTickYs = <(api.Tick, double)>[];
    for (final t in yTicks) {
      if (t.label != null) {
        yTickYs.add((t, probe._yToScreen(t.position)));
      }
    }
    return CellLayout(
      plotRect: rect,
      xTickXs: List.unmodifiable(xTickXs),
      yTickYs: List.unmodifiable(yTickYs),
      xGridXs: List.unmodifiable([for (final t in xTicks) t.position]),
      yGridYs: List.unmodifiable([for (final t in yTicks) t.position]),
      viewport: viewport,
    );
  }
}

/// مپینگ خطی داده→صفحه — مقدار مپ‌شدهٔ مستقیم.
double linearMap(double d, double dMin, double dMax, double sMin, double sMax) {
  final span = (dMax - dMin).abs() < 1e-12 ? 1.0 : (dMax - dMin);
  return sMin + (d - dMin) / span * (sMax - sMin);
}

/// اندازهٔ متن mono 11sp — خالص با TextPainter (سند ۰۶ §2.3).
double _measureMono(String text) {
  final tp = TextPainter(
    text: TextSpan(
      text: text,
      style: const TextStyle(
        fontFamily: 'JetBrainsMono',
        fontSize: 11,
        height: 1.0,
      ),
    ),
    textDirection: TextDirection.ltr,
  )..layout();
  return tp.width;
}

// ---------------------------------------------------------------------------
// snap و tip
// ---------------------------------------------------------------------------

/// snap crosshair به نزدیک‌ترین رأس سری — binary search روی سری‌های
/// یکنوای x (سند ۰۶ §7.3)، scan کامل برای نامنظم‌ها.
CrosshairSnap? snapToNearest(
  List<SeriesGeom> series,
  double dataX,
  double dataY,
) {
  CrosshairSnap? best;
  var bestDist = double.infinity;
  for (var si = 0; si < series.length; si++) {
    final s = series[si];
    if (s.x.isEmpty) continue;
    final monotone = _isMonotone(s.x);
    final candidateIdx = monotone
        ? _nearestByBinarySearch(s.x, dataX)
        : _nearestByScan(s.x, s.y, dataX, dataY);
    final idxs = monotone ? _neighbors(s.x, candidateIdx) : [candidateIdx];
    for (final i in idxs) {
      if (i < 0 || i >= s.x.length) continue;
      final dx = s.x[i] - dataX;
      final dy = s.y[i] - dataY;
      final d = dx * dx + dy * dy;
      if (d < bestDist) {
        bestDist = d;
        best = CrosshairSnap(
          seriesId: s.id,
          seriesIndex: si,
          color: s.color,
          x: s.x[i],
          y: s.y[i],
        );
      }
    }
  }
  return best;
}

class CrosshairSnap {
  CrosshairSnap({
    required this.seriesId,
    required this.seriesIndex,
    required this.color,
    required this.x,
    required this.y,
  });

  final int seriesId;
  final int seriesIndex;
  final Color color;
  final double x;
  final double y;
}

/// data-tip پایدار — با TAP نزدیک یک رأس کاشته می‌شود (کشف‌پذیر، برخلاف
/// نگه‌داشتنِ پنهان 3.4).
class DataTip {
  DataTip({
    required this.seriesId,
    required this.seriesIndex,
    required this.color,
    required this.x,
    required this.y,
  });

  final int seriesId;
  final int seriesIndex;
  final Color color;
  final double x;
  final double y;
}

/// انتخاب tip با تپ: نزدیک‌ترین رأس در شعاع پیکسلی — برخورد در فضای
/// صفحه (شعاع لمس، نه داده — روی محورهای با مقیاس متفاوت درست است).
DataTip? pickTip(
  List<SeriesGeom> series,
  CellLayout layout, {
  required Offset positionPx,
  double radiusPx = 48,
}) {
  DataTip? best;
  var bestDist = double.infinity;
  final r2 = radiusPx * radiusPx;
  for (var si = 0; si < series.length; si++) {
    final s = series[si];
    for (var i = 0; i < s.x.length; i++) {
      final x = s.x[i];
      final y = s.y[i];
      if (!x.isFinite || !y.isFinite) continue;
      final p = layout.dataToScreen(x, y);
      final d2 = (p - positionPx).distanceSquared;
      if (d2 <= r2 && d2 < bestDist) {
        bestDist = d2;
        best = DataTip(
          seriesId: s.id,
          seriesIndex: si,
          color: s.color,
          x: x,
          y: y,
        );
      }
    }
  }
  return best;
}

bool _isMonotone(List<double> x) {
  if (x.length < 2) return true;
  var up = true;
  var down = true;
  for (var i = 1; i < x.length; i++) {
    if (x[i] < x[i - 1]) up = false;
    if (x[i] > x[i - 1]) down = false;
    if (!up && !down) return false;
  }
  return up || down;
}

int _nearestByBinarySearch(List<double> x, double target) {
  var lo = 0;
  var hi = x.length - 1;
  while (hi - lo > 1) {
    final mid = (lo + hi) >> 1;
    if (x[mid] < target) {
      lo = mid;
    } else {
      hi = mid;
    }
  }
  return (x[lo] - target).abs() <= (x[hi] - target).abs() ? lo : hi;
}

int _nearestByScan(List<double> x, List<double> y, double tx, double ty) {
  var best = 0;
  var bestD = double.infinity;
  for (var i = 0; i < x.length; i++) {
    final dx = x[i] - tx;
    final dy = y[i] - ty;
    final d = dx * dx + dy * dy;
    if (d < bestD) {
      bestD = d;
      best = i;
    }
  }
  return best;
}

List<int> _neighbors(List<double> x, int i) {
  final out = <int>[i];
  if (i - 1 >= 0) out.add(i - 1);
  if (i + 1 < x.length) out.add(i + 1);
  return out;
}

/// جمع نقاط یک سلول — برای شیت اطلاعات.
int rawPointCount(List<SeriesGeom> series) =>
    series.fold(0, (acc, s) => acc + s.x.length);

/// فاصلهٔ min مثبت بین xهای مرتب — عرض slot میله‌ها (fallback: 1).
double barSlot(List<double> x) {
  final sorted = [...x]..sort();
  var slot = double.infinity;
  for (var i = 1; i < sorted.length; i++) {
    final d = sorted[i] - sorted[i - 1];
    if (d > 1e-12) slot = math.min(slot, d);
  }
  return slot.isFinite ? slot : 1.0;
}

/// فیلتر NaN/Inf — نقاش مسیر با مقدار غیرمتناهی نمی‌کشد.
List<double> finiteOf(List<double> v) =>
    v.where((e) => e.isFinite).toList(growable: false);
