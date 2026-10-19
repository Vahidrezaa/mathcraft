import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:mathcraft/app.dart';
import 'package:mathcraft/core/rust/engine_client.dart';
import 'package:mathcraft/core/rust/engine_provider.dart';

/// موتور ساختگی برای تست UI بدون libmathcraft.so (سند ۰۸ بخش ۶ — UI golden/test).
class FakeEngineClient implements EngineClient {
  @override
  Future<EvalResult> evaluateLine(String code) async {
    if (code.trim() == '1+1') {
      return EvalResult(
        jobId: 1,
        consoleEvents: [
          ConsoleEvent_Print(text: 'ans = 2'),
        ],
        plots: const [],
        workspaceDelta: WorkspaceDelta(changed: const []),
        status: JobStatus.done,
        durationMs: 1,
      );
    }
    return EvalResult(
      jobId: 2,
      consoleEvents: [
        ConsoleEvent_Error(
          err: McError(
            id: 'MC:E1001_UNDEFINED_VARIABLE',
            line: 1,
            col: 1,
            messageEn: "Undefined variable 'zz'.",
            messageFa: "متغیر 'zz' تعریف نشده.",
          ),
        ),
      ],
      plots: const [],
      workspaceDelta: WorkspaceDelta(changed: const []),
      status: JobStatus.failed,
      durationMs: 1,
    );
  }
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

/// دمو رسمی P0 (سند ۰۷): کنسولی که «1+1 = 2» را نشان می‌دهد.
void main() {
  testWidgets('P0 demo — typing 1+1 shows ans = 2', (tester) async {
    await _pumpApp(tester);

    await tester.enterText(find.byType(TextField), '1+1');
    await tester.testTextInput.receiveAction(TextInputAction.done);
    await tester.pumpAndSettle();

    expect(find.text('ans = 2'), findsOneWidget);
    expect(find.text('1+1'), findsOneWidget); // اکوی ورودی
  });

  testWidgets('empty state sample chip runs against the engine',
      (tester) async {
    await _pumpApp(tester);

    // اولین نمونه (1+1) روی حالت خالی
    await tester.tap(find.text('1+1'));
    await tester.pumpAndSettle();

    expect(find.text('ans = 2'), findsOneWidget);
  });

  testWidgets('errors render as structured bilingual cards', (tester) async {
    await _pumpApp(tester);

    await tester.enterText(find.byType(TextField), 'zz + 1');
    await tester.testTextInput.receiveAction(TextInputAction.done);
    await tester.pumpAndSettle();

    // پیش‌فرض فارسی است (D-06) — پیام فارسی باید دیده شود.
    expect(find.textContaining('MC:E1001'), findsOneWidget);
    expect(find.textContaining('تعریف نشده'), findsOneWidget);
  });

  testWidgets('theme and language switch instantly (DoD P0)', (tester) async {
    await _pumpApp(tester);

    // سوییچ زبان: برچسب ناوبری انگلیسی می‌شود
    await tester.tap(find.byTooltip('تغییر زبان'));
    await tester.pumpAndSettle();
    expect(find.text('Console'), findsOneWidget);
    expect(find.text('کنسول'), findsNothing);

    // سوییچ تم: تم فعلی روشن می‌شود (حالت آشنا)
    await tester.tap(find.byTooltip('Switch between Modern and Familiar'));
    await tester.pumpAndSettle();
    final context = tester.element(find.byType(TextField));
    expect(Theme.of(context).brightness, Brightness.light);
  });
}
