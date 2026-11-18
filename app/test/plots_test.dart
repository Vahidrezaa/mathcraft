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

  group('CellLayout (viewport-based, re-ticking)', () {
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
            logX: false,
            logY: false,
            visible: true,
            xTicks: const [],
            yTicks: const [],
          ),
        );

    test('generates nice 1-2-5 ticks for the home viewport', () {
      final cell = line();
      final layout =
          CellLayout.build(cell, const Size(320, 240), PlotViewport.home(cell.axes));
      // y: بازهٔ [-1,1] و ۳ تیک هدف → گام ۱ → [-1, 0, 1]
      expect(layout.yTickYs.map((t) => t.$1.label).toList(), ['-1', '0', '1']);
      // x: بازهٔ [0,10] و ۴ تیک هدف → گام ۵ → [0, 5, 10]
      expect(layout.xTickXs.map((t) => t.$1.label).toList(), ['0', '5', '10']);
      expect(layout.plotRect.left, greaterThan(8));
    });

    test('zoomed viewport re-ticks (regression: fixed ticks of 3.4)', () {
      final cell = line();
      final zoomed = const PlotViewport(
          xMin: 0, xMax: 1, yMin: -1, yMax: 1);
      final layout = CellLayout.build(cell, const Size(320, 240), zoomed);
      // دید به [0,1] → گام ۰.۵ → برچسب‌های جدید، نه تیک‌های ثابت خانه
      expect(
        layout.xTickXs.map((t) => t.$1.label).toList(),
        ['0', '0.5', '1'],
      );
    });

    test('mapping and inverse roundtrip on current viewport', () {
      final cell = line();
      final vp = PlotViewport.home(cell.axes);
      final layout = CellLayout.build(cell, const Size(320, 240), vp);
      final p = layout.dataToScreen(5, 0);
      expect(p.dx, closeTo(layout.plotRect.center.dx, 0.5));
      expect(p.dy, closeTo(layout.plotRect.center.dy, 0.5));
      final (bx, by) = layout.screenToData(p);
      expect(bx, closeTo(5, 1e-9));
      expect(by, closeTo(0, 1e-9));
      // تیک‌ها روی موقعیت صحیح
      final (t0, sx0) = layout.xTickXs.first;
      expect(t0.position, 0);
      expect(sx0, closeTo(layout.plotRect.left, 1e-6));
    });

    test('axis off shrinks to minimal rect without ticks', () {
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
          logX: false,
          logY: false,
          visible: false,
          xTicks: const [],
          yTicks: const [],
        ),
      );
      final layout = CellLayout.build(off, const Size(320, 240),
          PlotViewport.home(off.axes));
      expect(layout.plotRect.left, 2);
      expect(layout.xTickXs, isEmpty);
      expect(layout.yTickYs, isEmpty);
    });
  });

  group('custom ticks (v2.1 — engine sends only user ticks)', () {
    LineCellData cellWith(List<api.Tick> xt, List<api.Tick> yt) => LineCellData(
          series: [
            SeriesGeom(
              id: 1,
              x: const [0, 2, 4],
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
            logX: false,
            logY: false,
            visible: true,
            xTicks: xt,
            yTicks: yt,
          ),
        );

    test('user ticks replace auto-generation, labels as given', () {
      final cell = cellWith(
        const [
          api.Tick(position: 0, label: 'a'),
          api.Tick(position: 2, label: 'b'),
          api.Tick(position: 4, label: 'c'),
        ],
        const [],
      );
      final layout = CellLayout.build(
          cell, const Size(320, 240), PlotViewport.home(cell.axes));
      expect(
        layout.xTickXs.map((t) => t.$1.label).toList(),
        ['a', 'b', 'c'],
      );
      // y خودکار مانده (سفارشی خالی)
      expect(layout.yTickYs, isNotEmpty);
      expect(layout.yTickYs.first.$1.label, isNot('a'));
    });

    test('custom ticks filter to the viewport on zoom (no regeneration)', () {
      final cell = cellWith(
        const [
          api.Tick(position: 0, label: 'a'),
          api.Tick(position: 2, label: 'b'),
          api.Tick(position: 4, label: 'c'),
        ],
        const [],
      );
      final zoomed = const PlotViewport(xMin: 0, xMax: 1, yMin: -1, yMax: 1);
      final layout = CellLayout.build(cell, const Size(320, 240), zoomed);
      // فقط تیک داخل دید — بازتولید نمی‌شود (پارتی متلب)
      expect(layout.xTickXs.map((t) => t.$1.label).toList(), ['a']);
    });

    test('custom y ticks win and feed the left margin', () {
      final cell = cellWith(
        const [],
        const [
          api.Tick(position: 0, label: 'baseline'),
          api.Tick(position: 1, label: 'top'),
        ],
      );
      final layout = CellLayout.build(
          cell, const Size(320, 240), PlotViewport.home(cell.axes));
      expect(layout.yTickYs.map((t) => t.$1.label).toList(),
          ['baseline', 'top']);
      // حاشیهٔ چپ از «baseline» پهن‌تر از حالت خودکار است
      expect(layout.plotRect.left, greaterThan(30));
    });
  });

  group('nice ticks (mirror of crates/mc-plot/src/ticks.rs)', () {
    test('nice_step follows 1-2-5 ladder', () {
      expect(niceStep(0.3), 0.5);
      expect(niceStep(0.7), 1.0);
      expect(niceStep(1.1), 2.0);
      expect(niceStep(3.0), 5.0);
      expect(niceStep(6.0), 10.0);
      expect(niceStep(17.0), 20.0);
      // ورودی غیرمثبت → ۱ (دفاعی)
      expect(niceStep(0.0), 1.0);
      expect(niceStep(-3.0), 1.0);
    });

    test('linear_ticks cover unit interval', () {
      final t = linearTicks(0.0, 1.0, 8);
      expect(t.first, 0.0);
      expect(t.last, 1.0);
      expect(t.length, inInclusiveRange(5, 9), reason: 'got $t');
      for (var i = 1; i < t.length; i++) {
        expect(t[i], greaterThan(t[i - 1]));
      }
    });

    test('linear_ticks degenerate ranges return single', () {
      expect(linearTicks(2.0, 2.0, 8), [2.0]);
      expect(linearTicks(5.0, 1.0, 8), [5.0]);
    });

    test('log_ticks major decades with tex labels', () {
      final t = logTicks(0.01, 100.0);
      expect(
        t.map((x) => x.label).toList(),
        ['10^{-2}', '10^{-1}', '10^{0}', '10^{1}', '10^{2}'],
      );
      expect(t[0].position, 0.01);
      expect(t[4].position, 100.0);
    });

    test('log_ticks narrow range falls back to minors', () {
      final t = logTicks(2.0, 8.0);
      expect(t.every((x) => x.label == null), isTrue, reason: 'all minor: $t');
      expect(t.any((x) => x.position == 3.0), isTrue);
      expect(t.any((x) => x.position == 5.0), isTrue);
      expect(logTicks(-1.0, 10.0), isEmpty);
    });

    test('axis_ticks dispatches by log flag', () {
      final lin = axisTicks(0.0, 10.0, false, 8);
      expect(lin.every((t) => t.label != null), isTrue);
      expect(lin[0].label, '0');
      final lg = axisTicks(1.0, 1000.0, true, 8);
      expect(lg, hasLength(4)); // 10^0..10^3
    });

    test('tick_count_for adapts to pixel span', () {
      expect(tickCountFor(60), 1.clamp(3, 10)); // کمینهٔ ۳
      expect(tickCountFor(60), 3);
      expect(tickCountFor(280), 5);
      expect(tickCountFor(1200), 10); // سقف
    });
  });

  group('PlotViewport math', () {
    const home = PlotViewport(xMin: 0, xMax: 10, yMin: -1, yMax: 1);

    test('zoomAt keeps the focal data point under the finger', () {
      final z = home.zoomAt(focalX: 3.0, focalY: 0.0, factor: 4.0);
      // فاصله‌های focal یک‌چهارم
      expect(z.xMin, closeTo(2.25, 1e-12));
      expect(z.xMax, closeTo(4.75, 1e-12));
      expect(z.yMin, closeTo(-0.25, 1e-12));
      expect(z.yMax, closeTo(0.25, 1e-12));
      // و وارون: دادهٔ زیر «انگشت» (پیکسل ثابت) در layout جدید همان
      // focal است. با زوم y هم برچسب‌های y و حاشیهٔ چپ عوض می‌شود و
      // نقطهٔ پیکسلی می‌لغزد (رفتار متلب هم همین است) — پس ناوردایی را
      // با دیدی که فقط x آن زوم شده (حاشیهٔ پایدار) می‌سنجیم
      final cell = _lineCell();
      final xZoom = const PlotViewport(
          xMin: 2.25, xMax: 4.75, yMin: -1, yMax: 1);
      final before = CellLayout.build(cell, const Size(320, 240), home);
      final after = CellLayout.build(cell, const Size(320, 240), xZoom);
      final fingerPx = before.dataToScreen(3.0, 0.0);
      final (fx, fy) = after.screenToData(fingerPx);
      expect(fx, closeTo(3.0, 1e-6));
      expect(fy, closeTo(0.0, 1e-6));
    });

    test('zoomAt guards invalid factor', () {
      expect(identical(home.zoomAt(focalX: 1, focalY: 0, factor: 0), home),
          isTrue);
      expect(
          identical(home.zoomAt(focalX: 1, focalY: 0, factor: double.nan),
              home),
          isTrue);
    });

    test('panPixels moves content with the finger', () {
      final v = home.panPixels(const Offset(60, 0), const Size(300, 200));
      // درگ راست → دید به چپ: xMin کم می‌شود؛ span ثابت
      expect(v.xMin, closeTo(-2.0, 1e-12));
      expect(v.xSpan, home.xSpan);
      // درگ پایین → دید به بالا: yMax بیشتر می‌شود
      final v2 = home.panPixels(const Offset(0, 40), const Size(300, 200));
      expect(v2.yMax, closeTo(1.4, 1e-12));
      expect(v2.ySpan, home.ySpan);
    });

    test('clamped bounds zoom span (home/10000 .. home*4), free center', () {
      // زوم عمیق فراتر از حد → span روی کف
      var deep = home;
      for (var i = 0; i < 6; i++) {
        deep = deep.zoomAt(focalX: 5, focalY: 0, factor: 10.0);
      }
      final c = deep.clamped(home);
      expect(c.xSpan, closeTo(home.xSpan / 10000, 1e-9));
      // زوم-اوت فراتر از حد → span روی سقف
      var wide = home;
      for (var i = 0; i < 8; i++) {
        wide = wide.zoomAt(focalX: 5, focalY: 0, factor: 0.1);
      }
      final w = wide.clamped(home);
      expect(w.xSpan, closeTo(home.xSpan * 4, 1e-9));
      // مرکز آزاد: پن به فضای خالی (دید به [5,15])، زوم حول focal=10
      // مرکز ۱۰ باقی می‌ماند — خارج از خانهٔ [0,10] و بدون نشت‌گیری
      final panned = home
          .panPixels(const Offset(-150, 0), const Size(300, 200))
          .zoomAt(focalX: 10, focalY: 0, factor: 2)
          .clamped(home);
      expect(panned.xCenter, greaterThan(9.9));
      expect(panned.xSpan, closeTo(5.0, 1e-9));
    });
  });

  group('pickTip (tap on series)', () {
    test('nearest vertex within pixel radius wins', () {
      final cell = _lineCell();
      final layout = CellLayout.build(
          cell, const Size(300, 220), PlotViewport.home(cell.axes));
      final target = layout.dataToScreen(1, 1);
      final tip = pickTip(cell.series, layout,
          positionPx: target + const Offset(10, 8));
      expect(tip, isNotNull);
      expect(tip!.x, 1);
      expect(tip.y, 1);
      expect(tip.seriesIndex, 0);
    });

    test('tap far from any vertex clears', () {
      final cell = _lineCell();
      final layout = CellLayout.build(
          cell, const Size(300, 220), PlotViewport.home(cell.axes));
      // گوشهٔ بالا-راستِ دور از هر سه رأس
      final tip = pickTip(cell.series, layout,
          positionPx: const Offset(285, 25));
      expect(tip, isNull);
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

    testWidgets('drag pans the viewport (content follows finger)',
        (tester) async {
      final cell = cellDataFromSpec(_lineFigure().spec)!;
      await tester.pumpWidget(_plotViewHarness(cell));
      await tester.pumpAndSettle();
      final state = tester.state<PlotViewState>(find.byType(PlotView));
      final homeCenter = state.viewport.xCenter;
      final homeXMin = state.viewport.xMin;
      // درگ به راست → دید به چپ
      await tester.drag(find.byType(PlotView), const Offset(60, 0));
      await tester.pumpAndSettle();
      expect(state.viewport.xCenter, lessThan(homeCenter));
      expect(state.viewport.xMin, lessThan(homeXMin));
      // span دست‌نخورده — پن است نه زوم
      expect(state.viewport.xSpan, closeTo(2.0, 1e-6));
    });

    testWidgets('pinch zooms the viewport around the focal point',
        (tester) async {
      final cell = cellDataFromSpec(_lineFigure().spec)!;
      await tester.pumpWidget(_plotViewHarness(cell));
      await tester.pumpAndSettle();
      final state = tester.state<PlotViewState>(find.byType(PlotView));
      final homeSpan = state.viewport.xSpan;
      final center = tester.getCenter(find.byType(PlotView));
      final g1 = await tester.startGesture(center - const Offset(30, 0));
      final g2 = await tester.startGesture(center + const Offset(30, 0));
      await g1.moveBy(const Offset(-60, 0));
      await g2.moveBy(const Offset(60, 0));
      await tester.pumpAndSettle();
      await g1.up();
      await g2.up();
      await tester.pumpAndSettle();
      // فاصلهٔ انگشت‌ها بیشتر → span کوچک‌تر (سیم‌کشی pinch→zoom)
      expect(state.viewport.xSpan, lessThan(homeSpan * 0.95));
      expect(state.viewport.xSpan, greaterThan(homeSpan * 0.1));
    });

    testWidgets('tap near a vertex plants a persistent data tip',
        (tester) async {
      final cell = cellDataFromSpec(_lineFigure().spec)!;
      await tester.pumpWidget(_plotViewHarness(cell));
      await tester.pumpAndSettle();
      final state = tester.state<PlotViewState>(find.byType(PlotView));
      // جای دقیق رأس (1,1) را از همان ریاضیات خالص می‌گیریم
      final layout = CellLayout.build(
          cell, const Size(300, 220), PlotViewport.home(cell.axes!));
      final topLeft = tester.getTopLeft(find.byType(PlotView));
      final vertexPx = layout.dataToScreen(1, 1);
      // tapAt مختصات جهانی می‌گیرد — آفست ویجت را اضافه می‌کنیم
      await tester.tapAt(topLeft + vertexPx);
      await tester.pumpAndSettle();
      expect(state.dataTip, isNotNull);
      expect(state.dataTip!.x, 1);
      expect(state.dataTip!.y, 1);
      // تپ دور → پاک می‌شود
      await tester.tapAt(topLeft + const Offset(285, 25));
      await tester.pumpAndSettle();
      expect(state.dataTip, isNull);
    });

    testWidgets('double-tap resets pan and zoom to home', (tester) async {
      final cell = cellDataFromSpec(_lineFigure().spec)!;
      await tester.pumpWidget(_plotViewHarness(cell));
      await tester.pumpAndSettle();
      final state = tester.state<PlotViewState>(find.byType(PlotView));
      await tester.drag(find.byType(PlotView), const Offset(80, 0));
      await tester.pumpAndSettle();
      expect(state.viewport.xCenter, isNot(closeTo(1.0, 0.05)));
      await tester.tap(find.byType(PlotView));
      await tester.pump(const Duration(milliseconds: 60));
      await tester.tap(find.byType(PlotView));
      await tester.pumpAndSettle();
      expect(state.viewport.xMin, 0.0);
      expect(state.viewport.xMax, 2.0);
      expect(state.viewport.yMin, 0.0);
      expect(state.viewport.yMax, 1.0);
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

LineCellData _lineCell() => LineCellData(
      series: [
        SeriesGeom(
          id: 1,
          x: const [0, 1, 2],
          y: const [0, 1, 0],
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
        xLimits: (0, 2),
        yLimits: (0, 1),
        equalAspect: false,
        logX: false,
        logY: false,
        visible: true,
        xTicks: const [],
        yTicks: const [],
      ),
    );

Widget _plotViewHarness(PlotCellData cell) => MaterialApp(
      home: Scaffold(
        body: Center(
          child: SizedBox(
            width: 300,
            height: 220,
            child: PlotView(
              cell: cell,
              brightness: Brightness.dark,
            ),
          ),
        ),
      ),
    );

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
