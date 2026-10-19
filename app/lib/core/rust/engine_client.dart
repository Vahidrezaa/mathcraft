import '../../src/rust/api.dart' as api;
import '../../src/rust/api.dart' show EvalResult;
import '../../src/rust/frb_generated.dart' show RustLib;

/// Types بازتاب‌یافته از قرارداد mc-ffi (سند ۰۲ بخش ۳) — برای راحتی UI.
/// (export کامل است تا کلاس‌های واریانت freezed مثل ConsoleEvent_Print هم
/// در دسترس مصرف‌کننده‌ها باشند؛ export باید قبل از declarations باشد — درس P0.2.3)
export '../../src/rust/api.dart';

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
    if (version != 2) {
      // قرارداد ناسازگار = پیام واضح، نه کرش (سند ۰۲ بخش ۳ بند ۴؛ v2 — P0.2.4).
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
}
