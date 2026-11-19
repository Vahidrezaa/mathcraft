//! رگرسیون باگ 3.5.2 (گزارش کاربر): بستن شیتِ info نمودار، کیبوردِ
//! editor را بالا می‌آورد — چون IndexedStack تب‌ها را با فوکوسِ زنده نگه
//! می‌داشت و route-pop فوکوس را به TextField ادیتور برمی‌گرداند.
//!
//! دو فیکس: unfocus روی تعویض تب (app_shell) + unfocus پیش از شیت
//! (plots_page). این تست هر دو را از بیرون می‌سنجد: نشانهٔ کاربر =
//! testTextInput.isVisible بعد از هر مرحله.

import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:mathcraft/app.dart';
import 'package:mathcraft/core/rust/engine_client.dart';
import 'package:mathcraft/core/rust/engine_provider.dart';
import 'package:mathcraft/src/rust/api.dart' as api;

/// موتور ساختگی — همان الگوی editor_test؛ یک شکل خطی برای دکمهٔ info.
class FakeEngineClient implements EngineClient {
  @override
  Future<void> vfsRegister(String name, Uint8List bytes) async {}

  api.PlotFigure _fig() => api.PlotFigure(
        figureId: BigInt.one,
        spec: api.PlotSpec_Line2D(
          series: [
            api.Series2D(
              id: BigInt.one,
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

  @override
  Future<EvalResult> evaluateLine(String code) async => EvalResult(
        jobId: BigInt.one,
        consoleEvents: const [],
        plots: const [],
        workspaceDelta: WorkspaceDelta(changed: const []),
        status: JobStatus.done,
        durationMs: BigInt.one,
        filesWritten: const [],
      );

  @override
  Stream<ScriptEvent> runScript(String code) => const Stream.empty();

  @override
  Future<List<api.PlotFigure>> getPlots() async => [_fig()];

  @override
  Future<bool> cancelActive() async => false;
}

Future<void> _pumpApp(WidgetTester tester) async {
  await tester.pumpWidget(
    ProviderScope(
      overrides: [engineProvider.overrideWithValue(FakeEngineClient())],
      child: const MathCraftApp(),
    ),
  );
  await tester.pumpAndSettle();
}

void main() {
  testWidgets(
      'تعویض تب فوکوس ادیتور را رها می‌کند و بستن info کیبورد نمی‌آورد',
      (tester) async {
    await _pumpApp(tester);

    // ۱) ادیتور: فوکوس + کیبورد
    await tester.tap(find.text('ادیتور'));
    await tester.pumpAndSettle();
    await tester.tap(find.byType(TextField));
    await tester.pump();
    expect(tester.testTextInput.isVisible, isTrue,
        reason: 'ادیتور باید فوکوس و کیبورد داشته باشد');

    // ۲) رفتن به تب نمودار: کیبورد پایین (فیکس app_shell — باگ 3.5.2)
    await tester.tap(find.text('نمودار'));
    await tester.pumpAndSettle();
    expect(tester.testTextInput.isVisible, isFalse,
        reason: 'تعویض تب باید فوکوس/کیبورد ادیتور را رها کند');

    // ۳) شیت info: باز → بسته → کیبورد نباید برگردد (فیکس plots_page —
    //    عین سناریوی گزارش کاربر)
    await tester.tap(find.byTooltip('اطلاعات شکل'));
    await tester.pumpAndSettle();
    expect(find.byType(BottomSheet), findsOneWidget);
    // بستن با تپ روی مانع بیرونیِ شیت (pageBack دکمهٔ back می‌خواهد)
    await tester.tap(find.byType(ModalBarrier).last);
    await tester.pumpAndSettle();
    expect(tester.testTextInput.isVisible, isFalse,
        reason: 'بستن info نباید کیبورد editor را بالا بیاورد (باگ 3.5.2)');
    // نکته: زنده‌ماندن EditorPage در IndexedStack در editor_test پوشش
    // داده شده؛ فایندرهای پیش‌فرض فرزندان غیرفعال IndexedStack را رد
    // می‌کنند، پس اینجا ادعایی درباره‌شان نمی‌کنیم.
  });
}
