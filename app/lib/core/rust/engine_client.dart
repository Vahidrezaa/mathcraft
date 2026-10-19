import '../../src/rust/api.dart' as api;
import '../../src/rust/frb_generated.dart' show RustLib;

/// لایه سرویس موتور — UI هرگز مستقیم به کد generated دست نمی‌زند تا
/// تست‌پذیری با Fake ممکن باشد (سند ۰۸ بخش ۶).
abstract class EngineClient {
  Future<EvalResult> evaluateLine(String code);
}

/// کلاینت واقعی روی موتور Rust (libmathcraft.so — سند ۰۲ بخش ۸).
class RustEngineClient implements EngineClient {
  bool _initialized = false;

  Future<void> ensureInitialized() async {
    if (_initialized) return;
    await RustLib.init();
    _initialized = true;
    final version = await api.apiVersion();
    if (version != 1) {
      // قرارداد ناسازگار = پیام واضح، نه کرش (سند ۰۲ بخش ۳ بند ۴).
      throw StateError(
        'MathCraft engine contract mismatch: expected 1, got $version',
      );
    }
  }

  @override
  Future<EvalResult> evaluateLine(String code) async {
    await ensureInitialized();
    return api.evaluateLine(code: code);
  }
}

/// Types بازتاب‌یافته از قرارداد mc-ffi (سند ۰۲ بخش ۳) — برای راحتی UI.
export '../../src/rust/api.dart'
    show
        EvalResult,
        ConsoleEvent,
        JobStatus,
        McError,
        WorkspaceDelta,
        VarSummary;
