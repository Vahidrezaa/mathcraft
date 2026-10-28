import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:mathcraft/app.dart';
import 'package:mathcraft/core/rust/engine_client.dart';
import 'package:mathcraft/core/rust/engine_provider.dart';

/// موتور ساختگی قابل‌کنترل برای قرارداد v2 (ROADMAP 2.3 — سند ۰۸ بخش ۶):
/// جریان `runScript` از یک StreamController دستی تغذیه می‌شود تا تست،
/// توالی Progress → Cancelled → Done را قطعی هدایت کند.
class ControllableFakeEngine implements EngineClient {
  final controller = StreamController<ScriptEvent>();
  int cancelCalls = 0;

  void _emit(ScriptEvent event) => controller.add(event);

  static EvalResult _cancelledResult() => EvalResult(
        jobId: BigInt.one,
        consoleEvents: [
          ConsoleEvent_Error(
            err: McError(
              id: 'MC:E4002_CANCELLED',
              line: 0,
              col: 0,
              messageEn: 'Operation terminated by user.',
              messageFa: 'عملیات به درخواست کاربر متوقف شد.',
            ),
          ),
        ],
        plots: const [],
        workspaceDelta: WorkspaceDelta(changed: const []),
        status: JobStatus.cancelled,
        durationMs: BigInt.from(1),
      );

  @override
  Future<EvalResult> evaluateLine(String code) async {
    throw UnimplementedError('مسیر زندهٔ v2 باید استفاده شود');
  }

  @override
  Stream<ScriptEvent> runScript(String code) => controller.stream;

  @override
  Future<bool> cancelActive() async {
    cancelCalls++;
    // توالی قرارداد: Cancelled فوری، سپس Done با نتیجهٔ cancelled
    _emit(ScriptEvent_Cancelled(jobId: BigInt.one));
    _emit(ScriptEvent_Done(result: _cancelledResult()));
    return true;
  }
}

Future<void> _pumpApp(WidgetTester tester, ControllableFakeEngine engine) async {
  await tester.pumpWidget(
    ProviderScope(
      overrides: [engineProvider.overrideWithValue(engine)],
      child: const MathCraftApp(),
    ),
  );
  await tester.pumpAndSettle();
}

void main() {
  testWidgets('Stop وسط اجرا: نوار پیشرفت نامعین → لغو → پیام متلب‌گونه',
      (tester) async {
    final engine = ControllableFakeEngine();
    await _pumpApp(tester, engine);

    // اجرای یک حلقهٔ بی‌نهایت از کنسول (سمانتیک stop متلب — DoD گام 2.3)
    await tester.enterText(find.byType(TextField), 'while 1, x = 1; end');
    await tester.tap(find.byTooltip('اجرا'));
    await tester.pump(); // collectLiveRun مشترک می‌شود

    // حین اجرا: دکمهٔ Stop جایگزین Run شده + نوار پیشرفت نامعین نمایان
    expect(find.byTooltip('توقف'), findsOneWidget);
    expect(find.byTooltip('اجرا'), findsNothing);
    expect(find.byType(LinearProgressIndicator), findsOneWidget);

    // لغو از UI
    await tester.tap(find.byTooltip('توقف'));
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 50));

    expect(engine.cancelCalls, 1);
    // پس از لغو: نوار پیشرفت بسته، دکمهٔ Run برگشته، پیام لغو رندر شده
    expect(find.byType(LinearProgressIndicator), findsNothing);
    expect(find.byTooltip('اجرا'), findsOneWidget);
    expect(find.text('عملیات به درخواست کاربر متوقف شد.'), findsOneWidget);
  });

  testWidgets('Progress قطعی: درصد مشخص روی نوار و مرحلهٔ ode45', (tester) async {
    final engine = ControllableFakeEngine();
    await _pumpApp(tester, engine);

    await tester.enterText(find.byType(TextField), 'y = 1');
    await tester.tap(find.byTooltip('اجرا'));
    await tester.pump();

    // رویداد Progress با درصد — نوار قطعی + برچسب مرحله
    engine.controller.add(ScriptEvent_Progress(
      jobId: BigInt.one,
      stage: 'ode45',
      percent: 0.5,
    ));
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 50));

    expect(find.byType(LinearProgressIndicator), findsOneWidget);
    expect(find.text('ode45'), findsOneWidget);
  });
}
