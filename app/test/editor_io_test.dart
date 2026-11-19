import 'dart:typed_data';
import 'dart:convert' show utf8;

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mathcraft/app.dart';
import 'package:mathcraft/core/io/mathcraft_dir.dart';
import 'package:mathcraft/core/rust/engine_client.dart';
import 'package:mathcraft/core/rust/engine_provider.dart';
import 'package:mathcraft/src/rust/api.dart' as api;

/// تست‌های ذخیره/بازکردن .m ادیتور (گام 3.7 close-out — DoD).
///
/// طراحی (درس هنگِ پیشین): dart:io واقعی داخل fake-async کامل نمی‌شود و
/// runAsync هم futureهایی را که در ناحیهٔ تست ساخته شده‌اند نجات نمی‌دهد؛
/// پس درز `MathCraftDir.ioOverride` فیکِ کاملاً حافظه‌ای می‌گیرد — بدون
/// IO واقعی، همه‌چیز داخل fake-async قطعی است (سند ۰۸ §۶).

class _FakeIo implements MathCraftIo {
  final _files = <String, Uint8List>{};
  final registered = <(String, Uint8List)>[];

  void put(String name, String text) =>
      _files[name] = Uint8List.fromList(utf8.encode(text));

  @override
  Future<String> writeScript(String name, String text) async {
    var n = name.trim();
    if (n.isEmpty) n = 'script.m';
    if (!n.toLowerCase().endsWith('.m')) n = '$n.m';
    _files[n] = Uint8List.fromList(utf8.encode(text));
    return '/sandbox/MathCraft/$n';
  }

  @override
  Future<List<String>> listScripts() async {
    final names = _files.keys
        .where((n) => n.toLowerCase().endsWith('.m'))
        .toList()
      ..sort();
    return names;
  }

  @override
  Future<String> readScript(String name) async =>
      utf8.decode(_files[name]!);

  @override
  Future<String> writeOutput(String name, Uint8List bytes) async {
    _files[name] = bytes;
    return '/sandbox/MathCraft/$name';
  }

  @override
  Future<int> registerAll(
    Future<void> Function(String name, Uint8List bytes) register,
  ) async {
    for (final e in _files.entries) {
      await register(e.key, e.value);
      registered.add((e.key, e.value));
    }
    return _files.length;
  }
}

class _RecordingEngine implements EngineClient {
  final calls = <(String, Uint8List)>[];

  @override
  Future<EvalResult> evaluateLine(String code) async => _result(code);

  @override
  Stream<ScriptEvent> runScript(String code) async* {
    yield ScriptEvent_Done(result: _result(code));
  }

  EvalResult _result(String code) => EvalResult(
        jobId: BigInt.from(1),
        consoleEvents: const [],
        plots: const [],
        workspaceDelta: WorkspaceDelta(changed: const []),
        filesWritten: const [],
        status: JobStatus.done,
        durationMs: BigInt.from(1),
      );

  @override
  Future<List<api.PlotFigure>> getPlots() async => const [];

  @override
  Future<bool> cancelActive() async => false;

  @override
  Future<void> vfsRegister(String name, Uint8List bytes) async {
    calls.add((name, bytes));
  }
}

void main() {
  late _FakeIo io;
  late _RecordingEngine engine;

  setUp(() {
    io = _FakeIo();
    engine = _RecordingEngine();
    MathCraftDir.ioOverride = io;
  });

  tearDown(() {
    MathCraftDir.ioOverride = null;
  });

  Future<void> pumpApp(WidgetTester tester) async {
    await tester.pumpWidget(
      ProviderScope(
        overrides: [engineProvider.overrideWithValue(engine)],
        child: const MathCraftApp(),
      ),
    );
    await tester.pumpAndSettle();
    await tester.tap(find.text('ادیتور'));
    await tester.pumpAndSettle();
  }

  testWidgets('save writes .m to sandbox and registers with the engine',
      (tester) async {
    await pumpApp(tester);
    await tester.enterText(find.byType(TextField).first, 'q = 5;');

    await tester.tap(find.byTooltip('ذخیرهٔ .m در پوشهٔ MathCraft'));
    await tester.pumpAndSettle();
    expect(find.text('ذخیرهٔ اسکریپت'), findsOneWidget);

    await tester.tap(find.text('ذخیره'));
    await tester.pumpAndSettle();

    // فایل در «sandbox» با همان متن + ثبت در VFS موتور با همان بایت‌ها
    expect(utf8.decode(io._files['script.m']!), 'q = 5;');
    expect(engine.calls, hasLength(1));
    expect(engine.calls[0].$1, 'script.m');
    expect(engine.calls[0].$2, io._files['script.m']);
    expect(find.textContaining('ذخیره شد'), findsOneWidget);
  });

  testWidgets('save appends .m extension when missing', (tester) async {
    await pumpApp(tester);
    await tester.enterText(find.byType(TextField).first, 'x = 1;');

    await tester.tap(find.byTooltip('ذخیرهٔ .m در پوشهٔ MathCraft'));
    await tester.pumpAndSettle();
    await tester.enterText(find.byType(TextField).last, 'myscript');
    await tester.tap(find.text('ذخیره'));
    await tester.pumpAndSettle();

    expect(io._files.containsKey('myscript.m'), isTrue);
  });

  testWidgets('open loads an existing sandbox script into the editor',
      (tester) async {
    io.put('hello.m', 'z = 41 + 1;');

    await pumpApp(tester);
    await tester.tap(find.byTooltip('بازکردن اسکریپت .m'));
    await tester.pumpAndSettle();

    await tester.tap(find.text('hello.m'));
    await tester.pumpAndSettle();

    expect(find.widgetWithText(TextField, 'z = 41 + 1;'), findsOneWidget);
    expect(find.textContaining('باز شد'), findsOneWidget);
  });

  testWidgets('open with empty sandbox explains itself honestly', (tester) async {
    await pumpApp(tester);
    await tester.tap(find.byTooltip('بازکردن اسکریپت .m'));
    await tester.pumpAndSettle();
    expect(find.textContaining('هنوز فایل .m'), findsOneWidget);
  });

  testWidgets('console startup registers sandbox files into the engine VFS',
      (tester) async {
    io.put('data.mcws', 'not-really');
    io.put('runme.m', 'a = 1;');
    await pumpApp(tester);
    await tester.pumpAndSettle();
    // ثبت خودکار هنگام باز شدن کنسول (initState) — دو فایل ثبت‌شده
    expect(io.registered.length, 2);
    expect(engine.calls.length, 2);
    expect(engine.calls.map((c) => c.$1), containsAll(['data.mcws', 'runme.m']));
  });
}
