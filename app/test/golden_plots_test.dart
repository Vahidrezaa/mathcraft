import 'dart:math' as math;
import 'dart:ui' as ui;

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mathcraft/core/cds/cds_tokens.dart';
import 'package:mathcraft/features/plots/plot_geometry.dart';
import 'package:mathcraft/features/plots/plot_painter.dart';
import 'package:mathcraft/features/plots/plot_view.dart';
import 'package:mathcraft/src/rust/api.dart' as api;

/// تست‌های golden-image گام 3.6 (ROADMAP §3.6 — سند ۰۸ §5):
/// رندر نمایشگر در پنجرهٔ ثابت و بدون موتور واقعی، تصویر مرجع می‌شود و در CI
/// (flutter test) بایت‌به‌بایت مقایسه می‌گردد — رگرسیون رندر قبل از دستگاه
/// گرفته می‌شود.
///
/// قواعد:
/// - فونت محیط تست Ahem است (دترمینیستیک روی همه ماشین‌ها) — هدف پین‌کردن
///   هندسه/رنگ/چیدمان/تعداد تیک است، نه شکل گلیف‌ها.
/// - Flutter در CI پین 3.24.5 است؛ هر ارتقای پین باید goldens را بازتولید کند.
/// - تغییر عمدی رندر = `flutter test --update-goldens` + ثبت دلیل در کامیت.
/// - بودجهٔ فریم اول سند ۰۷ §P3 سمت نقاش هم اینجا پین شده است (۳×۴۰۰۰ ≤ ۵۰ms).

const double _w = 400;
const double _h = 300;

AxesData _axes({
  (double, double) x = const (0, 10),
  (double, double) y = const (-1.5, 1.5),
  bool grid = true,
  bool logX = false,
  bool logY = false,
  String? title,
  String? xlabel,
  String? ylabel,
  List<api.Tick> xTicks = const [],
  List<api.Tick> yTicks = const [],
}) =>
    AxesData(
      title: title,
      xlabel: xlabel,
      ylabel: ylabel,
      grid: grid,
      xLimits: x,
      yLimits: y,
      equalAspect: false,
      logX: logX,
      logY: logY,
      visible: true,
      xTicks: xTicks,
      yTicks: yTicks,
    );

SeriesGeom _geom(
  int id,
  List<double> x,
  List<double> y, {
  int palette = 0,
  double width = 2,
  List<double>? dash,
  api.Marker? marker,
}) =>
    SeriesGeom(
      id: id,
      x: x,
      y: y,
      color: CdsColors.plotPalette[palette],
      width: width,
      dash: dash,
      marker: marker,
    );

(List<double>, List<double>) _sinCos(double Function(double) f) {
  final x = List<double>.generate(41, (i) => i * 0.25);
  final y = x.map(f).toList();
  return (x, y);
}

LineCellData _sinCosCell({
  bool two = true,
  String? title = 'signal',
  String? xlabel = 'x',
  String? ylabel = 'y',
  bool grid = true,
  api.LegendPosition? legendPos,
}) {
  final (sx, sy) = _sinCos((v) => math.sin(v));
  final series = [
    _geom(0, sx, sy, palette: 0),
    if (two)
      _geom(
        1,
        sx,
        sx.map((v) => math.cos(v)).toList(),
        palette: 1,
        dash: const [8, 4],
      ),
  ];
  return LineCellData(
    series: series,
    axes: _axes(
      grid: grid,
      title: title,
      xlabel: xlabel,
      ylabel: ylabel,
    ),
    legend: legendPos == null ? null : LegendData(['sin', 'cos'], legendPos),
  );
}

Future<void> _golden(
  WidgetTester tester,
  String name,
  PlotCellData cell, {
  Brightness brightness = Brightness.dark,
}) async {
  await tester.pumpWidget(
    MaterialApp(
      home: Scaffold(
        body: Center(
          child: RepaintBoundary(
            key: Key('golden-$name'),
            child: SizedBox(
              width: _w,
              height: _h,
              child: PlotView(cell: cell, brightness: brightness),
            ),
          ),
        ),
      ),
    ),
  );
  await tester.pumpAndSettle();
  await expectLater(
    find.byKey(Key('golden-$name')),
    matchesGoldenFile('goldens/$name.png'),
  );
}

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  group('golden images (step 3.6)', () {
    testWidgets('line sin/cos + legend SW + labels (dark)', (tester) async {
      await _golden(
        tester,
        'line_sincos_legend_dark',
        _sinCosCell(legendPos: api.LegendPosition.southWest),
      );
    });

    testWidgets('line single series (light theme)', (tester) async {
      await _golden(
        tester,
        'line_single_light',
        _sinCosCell(two: false, title: null, ylabel: null),
        brightness: Brightness.light,
      );
    });

    testWidgets('scatter markers + marker-only series', (tester) async {
      final xs = List<double>.generate(9, (i) => i.toDouble());
      final cell = LineCellData(
        series: [
          _geom(0, xs, xs.map((v) => 0.5 * v + 0.2).toList(),
              palette: 2, marker: api.Marker.circle, width: 1.5),
          _geom(
            1,
            [1, 3, 5, 7, 9],
            [1.2, -0.8, 0.9, -1.1, 0.4],
            palette: 3,
            // 'b*' پارتی متلب — خط نامرئی، فقط مارکر
            dash: const [],
            marker: api.Marker.triangleUp,
          ),
        ],
        axes: _axes(x: (0, 9), y: (-1.5, 3)),
      );
      await _golden(tester, 'scatter_markers', cell);
    });

    testWidgets('bar grouped', (tester) async {
      final cell = BarCellData(
        series: [
          _geom(0, [1, 3, 5], [3, 6, 4], palette: 0),
          _geom(1, [1, 3, 5], [2, 4, 7], palette: 1),
        ],
        width: 0.8,
        horizontal: false,
        stacked: false,
        axes: _axes(x: (0, 6), y: (0, 8), title: 'grouped'),
      );
      await _golden(tester, 'bar_grouped', cell);
    });

    testWidgets('barh stacked', (tester) async {
      final cell = BarCellData(
        series: [
          _geom(0, [3, 5, 2], [1, 2, 3], palette: 2),
          _geom(1, [4, 1, 3], [1, 2, 3], palette: 3),
        ],
        width: 0.7,
        horizontal: true,
        stacked: true,
        axes: _axes(x: (0, 9), y: (0.5, 3.5), title: 'barh stacked'),
      );
      await _golden(tester, 'barh_stacked', cell);
    });

    testWidgets('histogram', (tester) async {
      final cell = HistCellData(
        binEdges: [0, 1, 2, 3, 4, 5, 6],
        counts: [3, 7, 9, 5, 2, 1],
        color: CdsColors.plotPalette[2],
        axes: _axes(x: (0, 6), y: (0, 10), title: 'hist'),
      );
      await _golden(tester, 'histogram', cell);
    });

    testWidgets('area with baseline', (tester) async {
      final (x, y) = _sinCos((v) => math.sin(v) * 0.8 + 0.9);
      final cell = AreaCellData(
        series: [_geom(0, x, y, palette: 4)],
        baseline: 0,
        axes: _axes(x: (0, 10), y: (0, 2), title: 'area'),
      );
      await _golden(tester, 'area_baseline', cell);
    });

    testWidgets('fill two polygons per-call colors', (tester) async {
      final cell = FillCellData(
        polys: [
          _geom(0, [0, 4, 2], [0, 0, 3], palette: 0),
          _geom(1, [5, 9, 9, 5], [-1, -1, 1.2, 1.2], palette: 1),
        ],
        axes: _axes(x: (0, 10), y: (-1.5, 3.5), grid: false),
      );
      await _golden(tester, 'fill_two_polys', cell);
    });

    testWidgets('polarplot rose', (tester) async {
      final th = List<double>.generate(73, (i) => i * (2 * 3.141592653589793) / 72);
      final r = th.map((t) => 1 + 0.3 * math.cos(3 * t)).toList();
      final cell = PolarCellData(
        series: [_geom(0, th, r, palette: 5)],
        legend: LegendData(['rose'], api.LegendPosition.northEast),
      );
      await _golden(tester, 'polar_rose', cell);
    });

    testWidgets('errorbar symmetric bands', (tester) async {
      final cell = ErrorCellData(
        series: [
          SeriesWithBand(
            base: _geom(0, [1, 2, 3, 4, 5], [1, 2, 1.5, 3, 2.5], palette: 6),
            xNeg: null,
            xPos: null,
            yNeg: [0.2, 0.3, 0.1, 0.4, 0.2],
            yPos: [0.3, 0.2, 0.25, 0.1, 0.35],
          ),
        ],
        axes: _axes(x: (0, 6), y: (0, 4), title: 'errorbar'),
      );
      await _golden(tester, 'errorbar_bands', cell);
    });

    testWidgets('semilogx decades', (tester) async {
      // دیتا روی دهه‌ها؛ برچسب تیک‌های موتور 10^{k} — دترمینیستیک
      final xs = [1.0, 10.0, 100.0, 1000.0];
      final cell = LineCellData(
        series: [
          _geom(0, xs, [1, 2, 2.5, 3], palette: 0, marker: api.Marker.plus),
        ],
        axes: _axes(
          x: (1, 1000),
          y: (0, 4),
          logX: true,
          title: 'semilogx',
          xlabel: 'f (Hz)',
        ),
      );
      await _golden(tester, 'semilogx_decades', cell);
    });

    testWidgets('custom ticks honored as-is (v2.1)', (tester) async {
      final cell = LineCellData(
        series: [_geom(0, [0, 10], [0, 10], palette: 1)],
        axes: _axes(
          x: (0, 10),
          y: (0, 10),
          grid: false,
          title: null,
          xlabel: null,
          ylabel: null,
          xTicks: const [
            api.Tick(position: 0, label: 'a'),
            api.Tick(position: 2.5, label: 'b'),
            api.Tick(position: 5, label: 'c'),
            api.Tick(position: 7.5, label: 'd'),
            api.Tick(position: 10, label: 'e'),
          ],
        ),
      );
      await _golden(tester, 'custom_ticks', cell);
    });

    testWidgets('crosshair + data tip layers (direct painter)', (tester) async {
      final cell = _sinCosCell(legendPos: api.LegendPosition.northEast);
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: Center(
              child: RepaintBoundary(
                key: const Key('golden-line_crosshair_tip_dark'),
                child: CustomPaint(
                  size: const Size(_w, _h),
                  painter: PlotPainter(
                    cell: cell,
                    viewport: PlotViewport.home(cell.axes),
                    brightness: Brightness.dark,
                    crosshair: CrosshairSnap(
                      seriesId: 0,
                      seriesIndex: 0,
                      color: CdsColors.plotPalette[0],
                      x: 2.4,
                      y: 0.675463,
                    ),
                    dataTip: DataTip(
                      seriesId: 1,
                      seriesIndex: 1,
                      color: CdsColors.plotPalette[1],
                      x: 7.2,
                      y: 0.608351,
                    ),
                  ),
                ),
              ),
            ),
          ),
        ),
      );
      await tester.pump();
      await expectLater(
        find.byKey(const Key('golden-line_crosshair_tip_dark')),
        matchesGoldenFile('goldens/line_crosshair_tip_dark.png'),
      );
    });
  });

  group('first-frame paint budget (docs/07 §P3, painter side)', () {
    test('3 series × 4000 pts paint ≤ 50 ms', () {
      List<double> pts(double Function(double) f) =>
          List<double>.generate(4000, (i) => f(i * 0.0025));
      final cell = LineCellData(
        series: [
          _geom(0, pts((v) => v), pts(math.sin), palette: 0),
          _geom(1, pts((v) => v), pts((v) => math.sin(v * 1.1)), palette: 1),
          _geom(2, pts((v) => v), pts((v) => math.cos(v * 0.9)), palette: 2),
        ],
        axes: _axes(x: (0, 10), y: (-1.5, 1.5), grid: true),
      );
      final painter = PlotPainter(
        cell: cell,
        viewport: PlotViewport.home(cell.axes),
        brightness: Brightness.dark,
      );
      final recorder = ui.PictureRecorder();
      final canvas = Canvas(recorder);
      final sw = Stopwatch()..start();
      painter.paint(canvas, const Size(_w, _h));
      sw.stop();
      expect(
        sw.elapsedMilliseconds,
        lessThan(50),
        reason: 'نقاش فریم اول ۱۲ هزار نقطه از بودجهٔ ۵۰ms گذشت',
      );
    });
  });
}
