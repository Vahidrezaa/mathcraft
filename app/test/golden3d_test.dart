import 'dart:math' as math;
import 'dart:ui';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mathcraft/features/plots/plot_colormap.dart';
import 'package:mathcraft/features/plots/plot_geometry.dart';
import 'package:mathcraft/features/plots/plot_painter.dart';
import 'package:mathcraft/features/plots/plot_view.dart';
import 'package:mathcraft/src/rust/api.dart' as api;

/// تست‌های سه‌بعدی گام 3.8 (P3-b — سند ۰۷ §P3 / سند ۰۸ §5):
/// golden-imageهای surf/mesh/contourf/plot3 + بودجهٔ نقاش ۵۰k مثلث
/// (سند ۰۴ §8.3: مرتب‌سازی عمق در Dart قابل‌قبول است) + پاریتی پالت
/// (پورت وفادار جدول‌های Rust) + ژست چرخش لمسی.

const double _w = 400;
const double _h = 300;

/// شبکهٔ نمونهٔ کوچک و دترمینیستیک: قلهٔ سادهٔ z = f(x,y).
(List<double>, List<double>, List<List<double>>) _grid({int n = 12}) {
  final x = List<double>.generate(n, (i) => i / (n - 1));
  final y = List<double>.generate(n, (i) => i / (n - 1));
  final z = [
    for (final yi in y)
      [
        for (final xi in x)
          math.exp(-((xi - 0.5) * (xi - 0.5) + (yi - 0.5) * (yi - 0.5)) * 8),
      ],
  ];
  return (x, y, z);
}

SurfaceCellData _surfaceCell({bool mesh = false}) {
  final (x, y, z) = _grid();
  return SurfaceCellData(
    x: x,
    y: y,
    z: z,
    isMesh: mesh,
    shading: api.Shading.faceted,
    colormap: api.Colormap.parula,
    caxis: null,
    colorbar: true,
    vertexNormals: const [], // نقاش fallback سایهٔ بدون نور دارد
    axes3d: Axes3DData(
      title: 'سطح نمونه',
      xlabel: 'x',
      ylabel: 'y',
      zlabel: 'z',
      grid: true,
      boxOn: false,
      view: (-37.5, 30),
      xLimits: (0, 1),
      yLimits: (0, 1),
      zLimits: (0, 1),
    ),
  );
}

ContourCellData _contourCell({required bool filled}) {
  final (x, y, z) = _grid(n: 16);
  return ContourCellData(
    x: x,
    y: y,
    z: z,
    levels: [0.2, 0.4, 0.6, 0.8],
    filled: filled,
    colormap: api.Colormap.parula,
    caxis: null,
    colorbar: true,
    axes: AxesData(
      title: 'هم‌ارزش',
      xlabel: 'x',
      ylabel: 'y',
      grid: false,
      xLimits: (0, 1),
      yLimits: (0, 1),
      equalAspect: false,
      logX: false,
      logY: false,
      visible: true,
      xTicks: const [
        api.Tick(position: 0, label: '0'),
        api.Tick(position: 1, label: '1'),
      ],
      yTicks: const [
        api.Tick(position: 0, label: '0'),
        api.Tick(position: 1, label: '1'),
      ],
    ),
  );
}

Plot3CellData _plot3Cell() => Plot3CellData(
      series: [
        Series3DGeom(
          x: List<double>.generate(60, (i) => (i / 59) * 4 * math.pi - 2 * math.pi),
          y: List<double>.generate(60, (i) => (i / 59) * 2),
          z: List<double>.generate(
              60, (i) => math.sin((i / 59) * 4 * math.pi - 2 * math.pi)),
          color: const Color(0xFF1f77b4),
          width: 2,
          dash: null,
        ),
      ],
      axes3d: Axes3DData(
        title: 'مارپیچ',
        xlabel: 'x',
        ylabel: 'y',
        zlabel: 'sin',
        grid: false,
        boxOn: false,
        view: (-37.5, 30),
        xLimits: (-6.5, 6.5),
        yLimits: (0, 2),
        zLimits: (-1, 1),
      ),
    );

Future<void> _golden(
  WidgetTester tester,
  String name,
  PlotCellData cell, {
  Brightness brightness = Brightness.dark,
}) async {
  await tester.pumpWidget(
    MaterialApp(
      debugShowCheckedModeBanner: false,
      theme: ThemeData(brightness: brightness, fontFamily: 'Roboto'),
      home: Scaffold(
        body: Center(
          child: SizedBox(
            width: _w,
            height: _h,
            child: KeyedSubtree(
              key: Key('golden-$name'),
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

  group('golden images (step 3.8 — 3D)', () {
    testWidgets('surf default view + colorbar', (tester) async {
      await _golden(tester, 'surf_parula_default', _surfaceCell());
    });

    testWidgets('mesh wireframe', (tester) async {
      await _golden(tester, 'mesh_wireframe', _surfaceCell(mesh: true));
    });

    testWidgets('contourf filled + colorbar', (tester) async {
      await _golden(tester, 'contourf_filled', _contourCell(filled: true));
    });

    testWidgets('contour lines only', (tester) async {
      await _golden(tester, 'contour_lines', _contourCell(filled: false));
    });

    testWidgets('plot3 helix + box', (tester) async {
      await _golden(tester, 'plot3_helix', _plot3Cell());
    });
  });

  group('rotation gesture (step 3.8)', () {
    testWidgets('drag rotates az/el; double-tap resets to contract view',
        (tester) async {
      final cell = _surfaceCell();
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: SizedBox(
              width: _w,
              height: _h,
              child: PlotView(cell: cell, brightness: Brightness.dark),
            ),
          ),
        ),
      );
      await tester.pumpAndSettle();

      // درگ افقی-عمودی → az و el جابه‌جا
      final center = Offset(_w / 2, _h / 2);
      await tester.dragFrom(center, const Offset(40, 20));
      await tester.pumpAndSettle();

      final state = tester.state<PlotViewState>(find.byType(PlotView));
      final view = state.viewOverride3D;
      expect(view, isNotNull, reason: 'چرخش محلی ثبت شد');
      // مقدار دقیق به سنتز درگِ تست وابسته است — جهت و دامنه را می‌سنجم
      expect(view!.$1, greaterThan(-37.4), reason: 'az با درگ راست جلو رفت');
      expect(view.$1, lessThan(-37.5 + 40), reason: 'az بیش از درگ جابه‌جا نشد');
      expect(view.$2, greaterThan(30.0), reason: 'el با درگ پایین جلو رفت');
      expect(view.$2, lessThan(30.0 + 20), reason: 'el بیش از درگ جابه‌جا نشد');

      // double-tap → بازگشت به نمای قرارداد
      await tester.tapAt(const Offset(100, 100));
      await tester.pump(const Duration(milliseconds: 50));
      await tester.tapAt(const Offset(100, 100));
      await tester.pumpAndSettle();
      expect(state.viewOverride3D, isNull, reason: 'بازنشانی به view قرارداد');
    });
  });

  group('paint budget (DoD سند ۰۷ §P3)', () {
    testWidgets('50k triangles paint under 120ms budget (headless CI)', (tester) async {
      // شبکهٔ ۱۵۸×۱۵۸ ≈ ۲۴,۹۶۴ خانه ≈ ۴۹,۹۲۸ مثلث — آستانهٔ DoD
      const n = 158;
      final x = List<double>.generate(n, (i) => i / (n - 1));
      final y = List<double>.generate(n, (i) => i / (n - 1));
      final z = [
        for (var i = 0; i < n; i++)
          List<double>.generate(
              n, (j) => math.sin(x[j] * 9) * math.cos(y[i] * 9) * 0.5 + 0.5),
      ];
      final cell = SurfaceCellData(
        x: x,
        y: y,
        z: z,
        isMesh: false,
        shading: api.Shading.flat,
        colormap: api.Colormap.parula,
        caxis: null,
        colorbar: false,
        vertexNormals: const [],
        axes3d: Axes3DData(
          title: null,
          xlabel: null,
          ylabel: null,
          zlabel: null,
          grid: false,
          boxOn: false,
          view: (-37.5, 30),
          xLimits: (0, 1),
          yLimits: (0, 1),
          zLimits: (0, 1),
        ),
      );

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: SizedBox(
              width: _w,
              height: _h,
              child: CustomPaint(
                size: const Size(_w, _h),
                painter: PlotPainter(
                  cell: cell,
                  viewport: PlotViewport.any,
                  brightness: Brightness.dark,
                ),
              ),
            ),
          ),
        ),
      );

      // زمان paint مستقیم — بدون IO/فونت؛ سقف CI بدون شتاب گرافیکی
      // (دستگاه مرجع با Skia/GPU خیلی سریع‌تر است؛ این سقف headless است)
      final recorder = PictureRecorder();
      final canvas = Canvas(recorder);
      final painter = PlotPainter(
        cell: cell,
        viewport: PlotViewport.any,
        brightness: Brightness.dark,
      );
      final sw = Stopwatch()..start();
      painter.paint(canvas, const Size(_w, _h));
      sw.stop();
      // چرخش کامل = بازپروجکشن + مرتب‌سازی + ~25k وجه — بودجهٔ سخاوانه
      // headless (JIT و بدون GPU): 120ms؛ روی دستگاه با AOT بسیار کمتر.
      expect(sw.elapsedMilliseconds, lessThan(120),
          reason: 'paint 50k-tri: ${sw.elapsedMilliseconds}ms');
      // و رندر واقعی هم بی‌خطا تمام می‌شود
      await tester.pumpAndSettle();
    });
  });

  group('colormap port parity (الگوی تیک 1-2-5)', () {
    test('anchor values match the Rust tables', () {
      // مقدارهای لنگر از تست‌های mc-plot/src/colormap.rs
      final gray0 = sampleColormap(api.Colormap.gray, 0);
      expect(gray0, const Color(0xFF000000));
      final gray1 = sampleColormap(api.Colormap.gray, 1);
      expect(gray1, const Color(0xFFFFFFFF));

      // jet: یک‌هشتم آبی خالص، میانه سبز قله، هفت‌هشتم قرمز خالص
      final jet18 = sampleColormap(api.Colormap.jet, 0.125);
      expect(jet18.blue, 255);
      expect(jet18.red, 0);
      final jetMid = sampleColormap(api.Colormap.jet, 0.5);
      expect(jetMid.green, 255);
      final jet78 = sampleColormap(api.Colormap.jet, 0.875);
      expect(jet78.red, 255);
      expect(jet78.blue, 0);

      // parula: سرِ آبی، تهِ نارنجی (روی جدول ۱۶نقطه‌ای)
      final parLo = sampleColormap(api.Colormap.parula, 0);
      expect(parLo.blue, greaterThan(120), reason: 'سر آبی');
      final parHi = sampleColormap(api.Colormap.parula, 1);
      expect(parHi.red, greaterThan(230), reason: 'ته نارنجی');

      // cool: سر فیروزه‌ای/ته صورتی
      expect(sampleColormap(api.Colormap.cool, 0).blue, 255);
      expect(sampleColormap(api.Colormap.cool, 1).red, 255);
    });

    test('sampleColormapZ normalizes like the kernel', () {
      final c = sampleColormapZ(api.Colormap.gray, 5, (0, 10));
      expect(c, const Color(0xFF808080)); // 0.5 → 128
      final flat = sampleColormapZ(api.Colormap.gray, 3, (3, 3));
      expect(flat, const Color(0xFF808080)); // دامنهٔ تخت → وسط پالت
    });

    test('project3d view(2) is top-down', () {
      final (sx, sy, _) = project3d(0, 90, 0.3, 0.7, 0.9);
      expect((sx - 0.3).abs(), lessThan(1e-9));
      expect((sy - 0.7).abs(), lessThan(1e-9));
    });
  });
}
