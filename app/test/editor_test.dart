import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:mathcraft/app.dart';
import 'package:mathcraft/core/rust/engine_client.dart';
import 'package:mathcraft/core/rust/engine_provider.dart';
import 'package:mathcraft/features/editor/editor_page.dart';
import 'package:mathcraft/features/editor/matlab_highlight.dart';

/// موتور ساختگی — همان الگوی console_demo_test (سند ۰۸ بخش ۶):
/// بدون libmathcraft.so؛ رفتار قطعی برای ادیتور.
class FakeEngineClient implements EngineClient {
  @override
  Future<EvalResult> evaluateLine(String code) async {
    if (code.trim() == 'x = 41 + 1') {
      return EvalResult(
        jobId: BigInt.from(1),
        consoleEvents: [
          ConsoleEvent_Print(text: 'x = 42'),
        ],
        plots: const [],
        workspaceDelta: WorkspaceDelta(changed: const []),
        status: JobStatus.done,
        durationMs: BigInt.from(1),
      );
    }
    return EvalResult(
      jobId: BigInt.from(2),
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
      durationMs: BigInt.from(1),
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
  // رفتن به تب ادیتور (برچسب فارسی پیش‌فرض — D-06)
  await tester.tap(find.text('ادیتور'));
  await tester.pumpAndSettle();
}

void main() {
  testWidgets('editor tab renders empty state', (tester) async {
    await _pumpApp(tester);
    // ادیتور در پوسته زنده است (IndexedStack) + پنل خروجی خالی
    expect(find.byType(EditorPage), findsOneWidget);
    // پیش‌فرض زبان فارسی (D-06)
    expect(find.textContaining('خروجی اجرا'), findsOneWidget);
  });

  testWidgets('run executes buffer and shows output', (tester) async {
    await _pumpApp(tester);

    await tester.enterText(find.byType(TextField), 'x = 41 + 1');
    await tester.tap(find.byTooltip('اجرا'));
    await tester.pumpAndSettle();

    expect(find.text('x = 42'), findsOneWidget);
  });

  testWidgets('errors render as structured cards in output pane',
      (tester) async {
    await _pumpApp(tester);

    await tester.enterText(find.byType(TextField), 'zz + 1');
    await tester.tap(find.byTooltip('اجرا'));
    await tester.pumpAndSettle();

    expect(find.textContaining('MC:E1001'), findsOneWidget);
    expect(find.textContaining('تعریف نشده'), findsOneWidget);
  });

  testWidgets('syntax highlighter styles keywords, comments and numbers',
      (tester) async {
    const code = 'function y = sq(x) % comment\ny = 2.5\n';
    final controller = MatlabCodeController(text: code);

    late BuildContext captured;
    await tester.pumpWidget(
      MaterialApp(
        home: Builder(
          builder: (context) {
            captured = context;
            return const Scaffold(body: SizedBox.shrink());
          },
        ),
      ),
    );

    final scheme = Theme.of(captured).colorScheme;
    final span = controller.buildTextSpan(
      context: captured,
      style: const TextStyle(fontSize: 14),
      withComposing: false,
    );

    String textOf(TextSpan span) => span.text ?? '';

    final spans = span.children!.cast<TextSpan>().toList();
    // کلیدواژه function با رنگ primary و وزن bold
    final fn = spans.firstWhere((s) => textOf(s) == 'function');
    expect(fn.style!.color, scheme.primary);
    expect(fn.style!.fontWeight, FontWeight.w700);
    // کامنت با رنگ muted و ایتالیک
    final comment =
        spans.firstWhere((s) => textOf(s).startsWith('% comment'));
    expect(comment.style!.color, scheme.onSurfaceVariant);
    expect(comment.style!.fontStyle, FontStyle.italic);
    // عدد با رنگ secondary
    final num = spans.firstWhere((s) => textOf(s) == '2.5');
    expect(num.style!.color, scheme.secondary);
    // شناسه معمولی (sq) با استایل پایه — بدون رنگ
    final ident = spans.firstWhere((s) => textOf(s) == 'sq');
    expect(ident.style!.color, null);
  });
}
