import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
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
        jobId: BigInt.from(1),
        consoleEvents: [
          ConsoleEvent_Print(text: 'ans = 2'),
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
}

String? _clipboardText;

/// هندلر ساختگی کانال حافظه — در تست ویجت، Clipboard بدون mock استثنای
/// MissingPluginException می‌دهد؛ اینجا setData ذخیره و getData همان متن را
/// برمی‌گرداند تا رفتار واقعی شبیه‌سازی شود (سند ۰۸ بخش ۶).
void _mockClipboard(WidgetTester tester) {
  tester.binding.defaultBinaryMessenger.setMockMethodCallHandler(
    SystemChannels.platform,
    (call) async {
      switch (call.method) {
        case 'Clipboard.setData':
          _clipboardText =
              (call.arguments as Map<Object?, Object?>?)?['text'] as String?;
        case 'Clipboard.getData':
          return <String, String?>{'text': _clipboardText};
      }
      return null;
    },
  );
}

/// بازخورد P0.2: کپی سطرهای کنسول — long-press سطر ورودی، متن بدون پرامپت؛
/// دکمه کپی روی بلوک نتیجه، SnackBar تأیید دو زبانه (پیش‌فرض فارسی — D-06).
void main() {
  setUp(() => _clipboardText = null);

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

  testWidgets('console entries live inside a themed SelectionArea (FB-2)',
      (tester) async {
    _mockClipboard(tester);
    await _pumpApp(tester);

    await tester.enterText(find.byType(TextField), '1+1');
    await tester.testTextInput.receiveAction(TextInputAction.done);
    await tester.pumpAndSettle();

    // FB-2: انتخاب تم‌دار — SelectionArea حول خروجی‌ها + رنگ‌های CDS
    expect(find.byType(SelectionArea), findsOneWidget);
    final context = tester.element(find.byType(TextField));
    final sel = Theme.of(context).textSelectionTheme;
    expect(sel.selectionHandleColor,
        Theme.of(context).colorScheme.primary);
    expect(sel.cursorColor, Theme.of(context).colorScheme.primary);
  });

  testWidgets('copy button on a result block copies the rendered text',
      (tester) async {
    _mockClipboard(tester);
    await _pumpApp(tester);

    await tester.enterText(find.byType(TextField), '1+1');
    await tester.testTextInput.receiveAction(TextInputAction.done);
    await tester.pumpAndSettle();

    // دکمه کپی محو کنار بلوک نتیجه — متن کپی = همان متن رندرشده ('ans = 2' از موتور ساختگی؛ چیدمان موتور واقعی تستِ Rust را دارد)
    await tester.tap(find.byTooltip('کپی'));
    await tester.pumpAndSettle();

    expect(find.text('در حافظه کپی شد'), findsOneWidget);
    final read = await Clipboard.getData('text/plain');
    expect(read?.text, 'ans = 2');
  });

  testWidgets('error card copy button copies the raw engine message',
      (tester) async {
    _mockClipboard(tester);
    await _pumpApp(tester);

    await tester.enterText(find.byType(TextField), 'zz + 1');
    await tester.testTextInput.receiveAction(TextInputAction.done);
    await tester.pumpAndSettle();

    // FB-2: انتخابِ بخشی از متن با long-press انجام می‌شود؛ کپی کل پیام
    // خام موتور (messageEn) از دکمهٔ کارت خطا می‌آید.
    await tester.tap(find.byTooltip('کپی').last);
    await tester.pumpAndSettle();

    expect(find.text('در حافظه کپی شد'), findsOneWidget);
    final read = await Clipboard.getData('text/plain');
    expect(read?.text, "Undefined variable 'zz'.");
  });
}
