import 'dart:math' as math;
import 'dart:typed_data';

import 'package:flutter/material.dart';

import '../../src/rust/api.dart' as api;
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

/// محورهای حل‌شدهٔ یک سلول — حدود همیشه از موتور می‌آید (تیک‌ها هم).
class AxesData {
  AxesData({
    required this.title,
    required this.xlabel,
    required this.ylabel,
    required this.grid,
    required this.xLimits,
    required this.yLimits,
    required this.equalAspect,
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
  final bool visible;
  final List<api.Tick> xTicks;
  final List<api.Tick> yTicks;
}

/// ورودی legend — entries با رنگ سری‌ها به‌ترتیب zip می‌شود (کمتر از تعداد
/// سری = فقط همان تعداد؛ رفتار متلب-lite مستند).
class LegendData {
  LegendData(this.entries, this.position);
  final List<String> entries;
  final api.LegendPosition position;
}

sealed class PlotCellData {
  AxesData? get axes => null;
  LegendData? get legend => null;

  /// سری‌های خطی‌گونه (هدف crosshair) — پیش‌فرض خالی.
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
// هندسهٔ خالص — layout و مپینگ و snap (بدون canvas؛ تست‌پذیر کامل)
// ---------------------------------------------------------------------------

/// layout یک سلول ۲بعدی روی مستطیل داده‌شده (کل صفحه یا سلول subplot).
class CellLayout {
  CellLayout({
    required this.plotRect,
    required this.xTickXs,
    required this.yTickYs,
    required this.xLimits,
    required this.yLimits,
  });

  /// ناحیهٔ محورها (داخل حاشیه‌های برچسب‌ها).
  final Rect plotRect;

  /// موقعیت صفحهٔ هر تیک (فقط تیک‌های برچسب‌دار برای متن؛ grid همه را می‌کشد).
  final List<(api.Tick, double)> xTickXs;
  final List<(api.Tick, double)> yTickYs;
  final (double, double) xLimits;
  final (double, double) yLimits;

  double _xToScreen(double x) =>
      linearMap(x, xLimits.$1, xLimits.$2, plotRect.left, plotRect.right);

  double _yToScreen(double y) =>
      linearMap(y, yLimits.$1, yLimits.$2, plotRect.bottom, plotRect.top);

  Offset dataToScreen(double x, double y) =>
      Offset(_xToScreen(x), _yToScreen(y));

  /// وارون مپینگ — مختصات داده برای crosshair.
  (double, double) screenToData(Offset p) => (
        linearMap(p.dx, plotRect.left, plotRect.right, xLimits.$1, xLimits.$2),
        linearMap(p.dy, plotRect.bottom, plotRect.top, yLimits.$1, yLimits.$2),
      );

  /// layout کامل از مدل سلول — تیک‌ها فقط برچسب‌دار موقعیت می‌گیرند.
  factory CellLayout.build(PlotCellData cell, Size size) {
    final axes = cell.axes!;
    final rect = plotAreaRect(
      axes: axes,
      size: size,
      measureLabelWidth: (label) => _measureMono(label),
    );
    final layout = CellLayout(
      plotRect: rect,
      xTickXs: [],
      yTickYs: [],
      xLimits: axes.xLimits,
      yLimits: axes.yLimits,
    );
    for (final t in axes.xTicks) {
      if (t.label != null) {
        layout.xTickXs.add((t, layout._xToScreen(t.position)));
      }
    }
    for (final t in axes.yTicks) {
      if (t.label != null) {
        layout.yTickYs.add((t, layout._yToScreen(t.position)));
      }
    }
    return layout;
  }
}

/// اندازهٔ متن mono 11sp — خالص با TextPainter (سند ۰۶ §2.3: اعداد محور).
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

/// ناحیهٔ رسم را با حاشیهٔ برچسب‌ها می‌سازد — خالص و تست‌پذیر.
/// مقیاس متن = mono 11sp سند ۰۶ §2.3 (ارقام tabular).
Rect plotAreaRect({
  required AxesData axes,
  required Size size,
  required double Function(String) measureLabelWidth,
}) {
  if (!axes.visible) {
    // axis off — فقط داده، بدون تزئینات (حاشیهٔ حداقلی)
    return Rect.fromLTWH(2, 2, size.width - 4, size.height - 4);
  }
  var left = 8.0;
  for (final t in axes.yTicks) {
    final label = t.label;
    if (label != null) {
      left = math.max(left, measureLabelWidth(label) + 14);
    }
  }
  const bottom = 30.0; // ردیف برچسب x + xlabel
  const top = 10.0;
  const right = 12.0;
  return Rect.fromLTWH(
    left,
    top,
    math.max(10, size.width - left - right),
    math.max(10, size.height - top - bottom),
  );
}

/// مپینگ خطی داده→صفحه برای یک محور — مقدار مپ‌شدهٔ مستقیم
/// (به‌جای برگرداندن closure — lint سند ۰۸).
double linearMap(double d, double dMin, double dMax, double sMin, double sMax) {
  final span = (dMax - dMin).abs() < 1e-12 ? 1.0 : (dMax - dMin);
  return sMin + (d - dMin) / span * (sMax - sMin);
}

/// snap crosshair به نزدیک‌ترین رأس سری — binary search روی سری‌های
/// یکنوای x (سند ۰۶ §7.3)، scan کامل برای نامنظم‌ها.
CrosshairSnap? snapToNearest(
  List<SeriesGeom> series,
  double dataX,
  double dataY,
) {
  CrosshairSnap? best;
  var bestDist = double.infinity;
  for (final s in series) {
    if (s.x.isEmpty) continue;
    final monotone = _isMonotone(s.x);
    final candidateIdx = monotone
        ? _nearestByBinarySearch(s.x, dataX)
        : _nearestByScan(s.x, s.y, dataX, dataY);
    final idxs = monotone ? _neighbors(s.x, candidateIdx, dataX) : [candidateIdx];
    for (final i in idxs) {
      if (i < 0 || i >= s.x.length) continue;
      final dx = s.x[i] - dataX;
      final dy = s.y[i] - dataY;
      final d = dx * dx + dy * dy;
      if (d < bestDist) {
        bestDist = d;
        best = CrosshairSnap(
          seriesId: s.id,
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
    required this.color,
    required this.x,
    required this.y,
  });

  final int seriesId;
  final Color color;
  final double x;
  final double y;
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

/// ایندکس‌های همسایهٔ نتیجهٔ binary search — برای یکنوای x، نزدیک‌ترین در
/// محور x کافی است اما رأس واقعی ممکن است در یکی از دو همسایه باشد.
List<int> _neighbors(List<double> x, int i, double target) {
  final out = <int>[i];
  if (i - 1 >= 0) out.add(i - 1);
  if (i + 1 < x.length) out.add(i + 1);
  return out;
}

/// متن‌های برچسب y برای اندازه‌گیری حاشیه — خالص.
double maxLabelWidth(List<api.Tick> ticks, double Function(String) measure) {
  var w = 0.0;
  for (final t in ticks) {
    final label = t.label;
    if (label != null) w = math.max(w, measure(label));
  }
  return w;
}

/// جمع نقاط خام یک سلول — برای شیت اطلاعات.
int rawPointCount(List<SeriesGeom> series) =>
    series.fold(0, (acc, s) => acc + s.x.length);

/// فاصلهٔ min مثبت بین xهای یکنواخت — عرض slot میله‌ها (fallback: 1).
double barSlot(List<double> x) {
  final sorted = [...x]..sort();
  var slot = double.infinity;
  for (var i = 1; i < sorted.length; i++) {
    final d = sorted[i] - sorted[i - 1];
    if (d > 1e-12) slot = math.min(slot, d);
  }
  return slot.isFinite ? slot : 1.0;
}

/// فیلتر NaN/Inf از Float64List — نقاش مسیر با مقدار غیرمتناهی نمی‌کشد.
List<double> finiteOf(List<double> v) =>
    v.where((e) => e.isFinite).toList(growable: false);

/// تبدیل Float64List به List<double> (بدون کپی اضافی در مصرف داخلی).
extension Float64Ext on Float64List {
  List<double> toListFinite() =>
      where((e) => e.isFinite).toList(growable: false);
}
