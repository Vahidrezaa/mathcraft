import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../core/rust/engine_provider.dart';
import '../src/rust/api.dart' as api;

/// تب فعال پوسته — نمودار با فعال‌شدن تب، فهرست شکل‌ها را pull می‌کند
/// (مدل pull سند ۰۲ §7؛ push در EvalResult.plots برای به‌روزرسانی فوری).
final activeTabProvider = StateProvider<int>((ref) => 0);

/// وضعیت شکل‌ها — pull از موتور؛ ترتیب = ترتیب ساخت.
///
/// خطای pull را نگه می‌داریم تا UI حالت واقعی نشان دهد (اصل صداقت) —
/// مثلاً وقتی libmathcraft.so بارگذاری نشده (تست‌ها/دسکتاپ).
class PlotsState {
  const PlotsState({
    required this.figures,
    required this.error,
  });

  final List<api.PlotFigure> figures;
  final Object? error;

  static const empty = PlotsState(figures: [], error: null);
}

class PlotsNotifier extends Notifier<PlotsState> {
  @override
  PlotsState build() => PlotsState.empty;

  /// pull کامل از موتور — در فعال‌شدن تب نمودار.
  Future<void> refresh() async {
    try {
      final figures = await ref.read(engineProvider).getPlots();
      state = PlotsState(figures: figures, error: null);
    } catch (e) {
      // موتور در دسترس نیست (دسکتاپ/تست) — حالت خالی با پیام، نه کرش
      state = PlotsState(figures: state.figures, error: e);
    }
  }

  /// push محلی نتیجهٔ یک اجرا — بدون رفت‌وبرگشت FFI (EvalResult.plots).
  void applyResult(List<api.PlotFigure> plots) {
    if (plots.isEmpty) return;
    final merged = {...state.figures};
    for (final p in plots) {
      merged.removeWhere((f) => f.figureId == p.figureId);
      merged.add(p);
    }
    final ordered = merged.toList()
      ..sort((a, b) => a.figureId.compareTo(b.figureId));
    state = PlotsState(figures: ordered, error: null);
  }
}

final plotsProvider =
    NotifierProvider<PlotsNotifier, PlotsState>(PlotsNotifier.new);
