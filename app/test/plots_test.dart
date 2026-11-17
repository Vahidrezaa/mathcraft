import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:mathcraft/core/cds/cds_tokens.dart';
import 'package:mathcraft/core/icons/craft_icons.dart';
import 'package:mathcraft/core/rust/engine_client.dart';
import 'package:mathcraft/core/rust/engine_provider.dart';
import 'package:mathcraft/features/plots/plot_format.dart';
import 'package:mathcraft/features/plots/plot_geometry.dart';
import 'package:mathcraft/features/plots/plot_palette.dart';
import 'package:mathcraft/features/plots/plot_view.dart';
import 'package:mathcraft/features/plots/plots_page.dart';
import 'package:mathcraft/l10n/app_localizations.dart';
import 'package:mathcraft/src/rust/api.dart' as api;
import 'package:mathcraft/state/plots_state.dart';

/// تست‌های نمایشگر نمودار (P3-a گام 3.4 — سند ۰۶ §7.3):
/// واحدهای خالص (فرمت g4 آینهٔ Rust، پالت، هندسه، snap) + ویجت با موتور
/// ساختگی همان الگوی console_demo_test (سند ۰۸ بخش ۶).

// ---------- فرمت %.4g — جدول هم‌ارز با Rust ticks.rs ----------

void expectG4(double v, String expected) {
  expect(formatG4(v), expected, reason: 'formatG4($v)');
}

void main() {
  group('formatG4 mirrors Rust %.4g', () {
    test('table', () {
      expectG4(0.0, '0');
      expectG4(0.5, '0.5');
      expectG4(0.001, '0.001');
      expectG4(0.0001, '0.0001');
      expectG4(0.00001, '1e-05');
      expectG4(1000.0, '1000');
      expectG4(1234.5, '1234');
      expectG4(12345.0, '1.234e+04');
      expectG4(100000.0, '1e+05');
      expectG4(-2.5, '-2.5');
      expectG4(-12345.0, '-1.234e+04');
      expectG4(9999.0, '9999');
      expectG4(double.nan, 'NaN');
      expectG4(double.infinity, 'Inf');
      expectG4(double.negativeInfinity, '-Inf');
    });
  });

  group('parseLogLabel', () {
    test('tex superscript and passthrough', () {
      final (b1, e1) = parseLogLabel('10^{-2}');
      expect(b1, '10');
      expect(e1, '-2');
      final (b2, e2) = parseLogLabel('0.5');
      expect(b2, '0.5');
      expect(e2, isNull);
    });
  });

  group('palette resolution (two-layer contract)', () {
    test('palette token resolves to CDS swatch', () {
      expect(
        resolvePlotColor(const api.PlotColor_Palette(3)),
        CdsColors.plotPalette[3],
      );
      // سیکل محافظت‌شده — ایندکس خارج 0..7 clamp می‌شود
      expect(
        resolvePlotColor(const api.PlotColor_Palette(9)),
        CdsColors.plotPalette[7],
      );
    });

    test('exact rgba passes through', () {
      final c = resolvePlotColor(
        const api.PlotColor_Rgba(r: 255, g: 0, b: 0, a: 255),
      );
      expect(c, const Color(0xFFFF0000));
    });

    test('dash patterns and invisible line', () {
      expect(dashPatternFor(api.DashStyle.solid), isNull);
      expect(dashPatternFor(api.DashStyle.dashed), [8, 4]);
      expect(dashPatternFor(api.DashStyle.invisible), const <double>[]);
      final invisible = api.LineStyle(
        color: const api.PlotColor_Palette(0),
        width: 2,
        dash: api.DashStyle.invisible,
        marker: api.Marker.circle,
      );
      expect(hasLine(invisible), isFalse, reason: "'b*' draws marker only");
    });
  });

  group('cellDataFromSpec', () {
    api.Series2D series({
      List<double> x = const [1, 2, 3],
      List<double> y = const [1, 2, 3],
      int palette = 0,
    }) =>
        api.Series2D(
          id: BigInt.one,
          rawLen: BigInt.from(x.length),
          x: Float64List.fromList(x),
          y: Float64List.fromList(y),
          style: api.LineStyle(
            color: api.PlotColor_Palette(palette),
            width: 2,
            dash: api.DashStyle.solid,
            marker: api.Marker.none,
          ),
        );

    api.Axes2D axes({
      List<double> xTicks = const [1, 2, 3],
    }) =>
        api.Axes2D(
          grid: true,
          equalAspect: false,
          logX: false,
          logY: false,
          visible: true,
          xLimits: const api.Limits(min: 1, max: 3),
          yLimits: const api.Limits(min: 0, max: 3),
          xTicks: [
            for (final t in xTicks)
              api.Tick(position: t, label: formatG4(t)),
          ],
          yTicks: const [
            api.Tick(position: 0, label: '0'),
            api.Tick(position: 3, label: '3'),
          ],
        );

    test('line2d extracts with resolved palette color', () {
      final cell = cellDataFromSpec(
        api.PlotSpec_Line2D(series: [series(palette: 2)], axes: axes()),
      );
      expect(cell, isA<LineCellData>());
      final line = cell! as LineCellData;
      expect(line.series.single.color, CdsColors.plotPalette[2]);
      expect(line.crosshairSeries, hasLength(1));
      expect(line.axes.xLimits, (1.0, 3.0));
    });

    test('scatter maps to line cell (engine already resolved style)', () {
      final cell = cellDataFromSpec(
        api.PlotSpec_Scatter2D(series: [series()], axes: axes()),
      );
      expect(cell, isA<LineCellData>());
    });

    test('3d and quiver are honest nulls (P3-b)', () {
      expect(
        cellDataFromSpec(api.PlotSpec_Surface(
          x: Float64List.fromList([0, 1]),
          y: Float64List.fromList([0, 1]),
          z: [
            Float64List.fromList([0, 1]),
            Float64List.fromList([1, 2]),
          ],
          shading: api.Shading.flat,
          colormap: api.Colormap.parula,
          caxis: null,
          colorbar: false,
        )),
        isNull,
      );
      expect(
        cellDataFromSpec(api.PlotSpec_Quiver(
          x: Float64List.fromList([0]),
          y: Float64List.fromList([0]),
          u: Float64List.fromList([1]),
          v: Float64List.fromList([1]),
          axes: axes(),
        )),
        isNull,
      );
    });

    test('histogram and polar extract', () {
      final hist = cellDataFromSpec(api.PlotSpec_Histogram(
        binEdges: Float64List.fromList([0, 1, 2]),
        counts: Float64List.fromList([3, 7]),
        norm: api.HistNorm.count,
        style: api.LineStyle(
          color: const api.PlotColor_Palette(1),
          width: 2,
          dash: api.DashStyle.invisible,
          marker: api.Marker.none,
        ),
        axes: axes(),
        legend: null,
      ));
      expect(hist, isA<HistCellData>());
      final polar = cellDataFromSpec(api.PlotSpec_Polar(
        series: [series()],
        legend: null,
      ));
      expect(polar, isA<PolarCellData>());
    });
  });

  group('CellLayout (pure geometry)', () {
    LineCellData line() => LineCellData(
          series: [
            SeriesGeom(
              id: 1,
              x: const [0, 5, 10],
              y: const [-1, 0, 1],
              color: CdsColors.plotPalette[0],
              width: 2,
              dash: null,
            ),
          ],
          axes: AxesData(
            title: null,
            xlabel: null,
            ylabel: null,
            grid: false,
            xLimits: (0, 10),
            yLimits: (-1, 1),
            equalAspect: false,
            visible: true,
            xTicks: const [
              api.Tick(position: 0, label: '0'),
              api.Tick(position: 10, label: '10'),
            ],
            yTicks: const [
              api.Tick(position: -1, label: '-1'),
              api.Tick(position: 1, label: '1'),
            ],
          ),
        );

    test('plotRect inside size and margins from y labels', () {
      final layout = CellLayout.build(line(), const Size(320, 240));
      expect(layout.plotRect.left, greaterThan(8));
      expect(layout.plotRect.right, lessThanOrEqualTo(320));
      expect(layout.plotRect.bottom, lessThanOrEqualTo(240));
    });

    test('mapping and inverse roundtrip', () {
      final layout = CellLayout.build(line(), const Size(320, 240));
      final p = layout.dataToScreen(5, 0);
      // وسط داده = وسط ناحیه رسم (محدوده‌های متقارن)
      expect(p.dx, closeTo(layout.plotRect.center.dx, 0.5));
      expect(p.dy, closeTo(layout.plotRect.center.dy, 0.5));
      final (bx, by) = layout.screenToData(p);
      expect(bx, closeTo(5, 1e-9));
      expect(by, closeTo(0, 1e-9));
      // تیک‌ها سر جایشان
      final (t0, sx0) = layout.xTickXs.first;
      expect(t0.position, 0);
      expect(sx0, closeTo(layout.plotRect.left, 1e-9));
    });

    test('axis off shrinks to minimal rect', () {
      final off = LineCellData(
        series: line().series,
        axes: AxesData(
          title: null,
          xlabel: null,
          ylabel: null,
          grid: false,
          xLimits: (0, 10),
          yLimits: (-1, 1),
          equalAspect: false,
          visible: false,
          xTicks: const [],
          yTicks: const [],
        ),
      );
      final layout = CellLayout.build(off, const Size(320, 240));
      expect(layout.plotRect.left, 2);
      expect(layout.xTickXs, isEmpty);
    });
  });

  group('snapToNearest', () {
    SeriesGeom geom(List<double> x, List<double> y) => SeriesGeom(
          id: 7,
          x: x,
          y: y,
          color: CdsColors.plotPalette[0],
          width: 2,
          dash: null,
        );

    test('monotone series snaps via binary path', () {
      final s = geom([0, 1, 2, 3, 4, 5], [0, 1, 4, 9, 16, 25]);
      final snap = snapToNearest([s], 3.4, 8.0)!;
      expect(snap.seriesId, 7);
      expect(snap.x, 3);
      expect(snap.y, 9);
    });

    test('unsorted series snaps by full scan', () {
      final s = geom([5, 0, 3, 1], [10, 0, 6, 2]);
      final snap = snapToNearest([s], 2.6, 5.0)!;
      expect(snap.x, 3);
      expect(snap.y, 6);
    });

    test('no series or empty → null', () {
      expect(snapToNearest(const [], 0, 0), isNull);
      final s = geom([], []);
      expect(snapToNearest([s], 0, 0), isNull);
    });

    test('nearest among multiple series picks global minimum', () {
      final a = geom([0, 2], [0, 0]);
      final b = geom([0, 2], [1.5, 1.5]);
      final snap = snapToNearest([a, b], 1.9, 0.2)!;
      expect(snap.x, 2);
      expect(snap.y, 0);
    });
  });

  group('barSlot', () {
    test('min positive delta and fallback', () {
      expect(barSlot([1, 2, 4]), 1.0);
      expect(barSlot([0, 2.5, 5]), 2.5);
      expect(barSlot([5]), 1.0);
    });
  });

  group('plots state (pull/push/merge)', () {
    api.PlotFigure fig(int id) => api.PlotFigure(
          figureId: BigInt.from(id),
          spec: api.PlotSpec_Line2D(
            series: [
              api.Series2D(
                id: BigInt.from(id),
                rawLen: BigInt.two,
                x: Float64List.fromList([0, 1]),
                y: Float64List.fromList([0, 1]),
                style: api.LineStyle(
                  color: const api.PlotColor_Palette(0),
                  width: 2,
                  dash: api.DashStyle.solid,
                  marker: api.Marker.none,
                ),
              ),
            ],
            axes: api.Axes2D(
              grid: false,
              equalAspect: false,
              logX: false,
              logY: false,
              visible: true,
              xLimits: const api.Limits(min: 0, max: 1),
              yLimits: const api.Limits(min: 0, max: 1),
              xTicks: const [],
              yTicks: const [],
            ),
            legend: null,
          ),
        );

    test('applyResult replaces same id and keeps sorted order', () {
      final container = ProviderContainer();
      addTearDown(container.dispose);
      final notifier = container.read(plotsProvider.notifier);
      notifier.applyResult([fig(2), fig(1)]);
      expect(
        container.read(plotsProvider).figures.map((f) => f.figureId),
        [BigInt.one, BigInt.two],
      );
      // شکل ۲ به‌روزرسانی (همان id) — نه تکرار
      notifier.applyResult([fig(2)]);
      final figures = container.read(plotsProvider).figures;
      expect(figures, hasLength(2));
      expect(figures.last.figureId, BigInt.two);
    });

    test('empty push keeps state untouched', () {
      final container = ProviderContainer();
      addTearDown(container.dispose);
      final notifier = container.read(plotsProvider.notifier);
      notifier.applyResult([fig(1)]);
      notifier.applyResult(const []);
      expect(container.read(plotsProvider).figures, hasLength(1));
    });
  });

  group('PlotsTab widget (fake engine)', () {
    testWidgets('empty state when engine has no figures', (tester) async {
      await tester.pumpWidget(_Harness(const []));
      await tester.pumpAndSettle();
      expect(find.text('No plots yet'), findsOneWidget);
    });

    testWidgets('line figure renders viewer with floating bar', (tester) async {
      await tester.pumpWidget(_Harness([_lineFigure()]));
      await tester.pumpAndSettle();
      expect(find.byType(PlotView), findsOneWidget);
      // نوار شناور: ۴ دکمه CraftIcon (فهرست/ذخیره/اشتراک/اطلاعات)
      expect(find.byType(CraftIcon), findsNWidgets(4));
      expect(find.text('Figure 1'), findsNothing); // chipها پنهان‌اند
    });

    testWidgets('figure list chip appears via list toggle', (tester) async {
      await tester.pumpWidget(_Harness([_lineFigure()]));
      await tester.pumpAndSettle();
      await tester.tap(find.byTooltip('Figure list'));
      await tester.pumpAndSettle();
      expect(find.text('Figure 1'), findsOneWidget);
    });

    testWidgets('crosshair callback fires on long-press and clears on end',
        (tester) async {
      CrosshairSnap? last;
      final cell = cellDataFromSpec(_lineFigure().spec)!;
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: SizedBox(
              width: 300,
              height: 220,
              child: PlotView(
                cell: cell,
                brightness: Brightness.dark,
                onCrosshair: (snap) => last = snap,
              ),
            ),
          ),
        ),
      );
      await tester.pumpAndSettle();
      final center = tester.getCenter(find.byType(PlotView));
      final gesture = await tester.startGesture(center);
      await tester.pump(const Duration(milliseconds: 700));
      await gesture.up();
      await tester.pumpAndSettle();
      // long-press کامل → snap روی نزدیک‌ترین رأس سری
      expect(last, isNotNull);
      expect(last!.x.isFinite, isTrue);
      expect(last!.y.isFinite, isTrue);
    });

    testWidgets('3d figure shows honest not-yet state', (tester) async {
      await tester.pumpWidget(_Harness([_surfaceFigure()]));
      await tester.pumpAndSettle();
      expect(find.text('3D and vector plots arrive in the next update'),
          findsOneWidget);
    });
  });
}

// ---------- سازنده‌های تست ----------

api.PlotFigure _lineFigure() => api.PlotFigure(
      figureId: BigInt.one,
      spec: api.PlotSpec_Line2D(
        series: [
          api.Series2D(
            id: BigInt.one,
            rawLen: BigInt.from(3),
            x: Float64List.fromList([0, 1, 2]),
            y: Float64List.fromList([0, 1, 0]),
            style: api.LineStyle(
              color: const api.PlotColor_Palette(0),
              width: 2,
              dash: api.DashStyle.solid,
              marker: api.Marker.none,
            ),
          ),
        ],
        axes: api.Axes2D(
          title: 'T',
          xlabel: 'x',
          ylabel: 'y',
          grid: true,
          equalAspect: false,
          logX: false,
          logY: false,
          visible: true,
          xLimits: const api.Limits(min: 0, max: 2),
          yLimits: const api.Limits(min: 0, max: 1),
          xTicks: const [
            api.Tick(position: 0, label: '0'),
            api.Tick(position: 1, label: '1'),
            api.Tick(position: 2, label: '2'),
          ],
          yTicks: const [
            api.Tick(position: 0, label: '0'),
            api.Tick(position: 1, label: '1'),
          ],
        ),
        legend: null,
      ),
    );

api.PlotFigure _surfaceFigure() => api.PlotFigure(
      figureId: BigInt.two,
      spec: api.PlotSpec_Surface(
        x: Float64List.fromList([0, 1]),
        y: Float64List.fromList([0, 1]),
        z: [
          Float64List.fromList([0, 1]),
          Float64List.fromList([1, 2]),
        ],
        shading: api.Shading.flat,
        colormap: api.Colormap.parula,
        caxis: null,
        colorbar: false,
      ),
    );

class _FakeEngine implements EngineClient {
  _FakeEngine(this.figures);

  final List<api.PlotFigure> figures;

  @override
  Future<api.EvalResult> evaluateLine(String code) async =>
      api.EvalResult(
        jobId: BigInt.one,
        consoleEvents: const [],
        plots: const [],
        workspaceDelta: const api.WorkspaceDelta(changed: []),
        status: api.JobStatus.done,
        durationMs: BigInt.one,
      );

  @override
  Future<List<api.PlotFigure>> getPlots() async => figures;

  @override
  Stream<api.ScriptEvent> runScript(String code) => const Stream.empty();

  @override
  Future<bool> cancelActive() async => false;
}

class _Harness extends StatelessWidget {
  const _Harness(this.figures);

  final List<api.PlotFigure> figures;

  @override
  Widget build(BuildContext context) {
    return ProviderScope(
      overrides: [
        engineProvider.overrideWithValue(_FakeEngine(figures)),
      ],
      child: MaterialApp(
        localizationsDelegates: AppLocalizations.localizationsDelegates,
        supportedLocales: AppLocalizations.supportedLocales,
        locale: const Locale('en'),
        home: const Scaffold(body: PlotsTab()),
      ),
    );
  }
}
