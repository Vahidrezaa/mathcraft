import 'dart:async';

import '../../src/rust/api.dart';
import '../../src/rust/api.dart' as api;
import '../../src/rust/frb_generated.dart' show RustLib;

/// Types بازتاب‌یافته از قرارداد mc-ffi (سند ۰۲ بخش ۳) — برای راحتی UI.
export '../../src/rust/api.dart'
    show
        EvalResult,
        ConsoleEvent,
        ConsoleEvent_Print,
        ConsoleEvent_Error,
        ConsoleEvent_Warning,
        ScriptEvent,
        ScriptEvent_Progress,
        ScriptEvent_Cancelled,
        ScriptEvent_Done,
        JobStatus,
        McError,
        WorkspaceDelta,
        VarSummary;

/// لایه سرویس موتور — UI هرگز مستقیم به کد generated دست نمی‌زند تا
/// تست‌پذیری با Fake ممکن باشد (سند ۰۸ بخش ۶).
///
/// قرارداد v2 (ROADMAP 2.3):
/// - `runScript` جریان زندهٔ `ScriptEvent` می‌دهد (Progress/Cancelled/Done)؛
/// - `cancelActive` job فعال را لغو می‌کند (CancellationToken سند ۰۲ §4)؛
/// - `evaluateLine` مسیر دسته‌ای v1 می‌ماند (mc-cli/تست‌ها).
abstract class EngineClient {
  Future<EvalResult> evaluateLine(String code);

  Stream<ScriptEvent> runScript(String code);

  /// لغو job فعال — true اگر job فعالی لغو شد.
  Future<bool> cancelActive();
}

/// کلاینت واقعی روی موتور Rust (libmathcraft.so — سند ۰۲ بخش ۸).
class RustEngineClient implements EngineClient {
  bool _initialized = false;

  Future<void> ensureInitialized() async {
    if (_initialized) return;
    await RustLib.init();
    _initialized = true;
    final version = await api.apiVersion();
    if (version != 2) {
      // قرارداد ناسازگار = پیام واضح، نه کرش (سند ۰۲ بخش ۳ بند ۴).
      throw StateError(
        'MathCraft engine contract mismatch: expected 2, got $version',
      );
    }
  }

  @override
  Future<EvalResult> evaluateLine(String code) async {
    await ensureInitialized();
    return api.evaluateLine(code: code);
  }

  @override
  Stream<ScriptEvent> runScript(String code) {
    // RustLib.init() توسط evaluateLine اولیه می‌شود؛ برای اجرای زنده هم باید
    // آماده باشد (اگر اولین فراخوانیِ عمر اپ باشد).
    return Stream.fromFuture(ensureInitialized())
        .asyncExpand((_) => api.runScript(code: code));
  }

  @override
  Future<bool> cancelActive() async {
    await ensureInitialized();
    final id = await api.activeJobId();
    if (id == null) return false;
    return api.cancelJob(jobId: id);
  }
}

/// جمع‌بندی جریان زندهٔ قرارداد v2 — رویدادهای Progress/Cancelled به
/// callback می‌روند و نتیجهٔ نهایی از رویداد Done برمی‌گردد.
///
/// بازگشت `null` یعنی جریان بدون Done بسته شد (قطع transport) — صفحهٔ
/// فراخوان با finally حالت _running را تمیز می‌کند.
Future<EvalResult?> collectLiveRun(
  Stream<ScriptEvent> stream, {
  void Function(String stage, double? percent)? onProgress,
  void Function()? onCancelled,
}) {
  final completer = Completer<EvalResult?>();
  late final StreamSubscription<ScriptEvent> sub;
  sub = stream.listen(
    (event) => switch (event) {
      ScriptEvent_Progress(:final stage, :final percent) =>
        onProgress?.call(stage, percent),
      ScriptEvent_Cancelled() => onCancelled?.call(),
      ScriptEvent_Done(:final result) => completer.isCompleted
          ? null
          : completer.complete(result),
    },
    onError: (Object e) {
      if (!completer.isCompleted) completer.completeError(e);
    },
    onDone: () {
      if (!completer.isCompleted) completer.complete(null);
    },
    cancelOnError: true,
  );
  return completer.future.whenComplete(sub.cancel);
}
