import 'dart:async';

import 'package:flutter/material.dart';

import 'plot_geometry.dart';
import 'plot_painter.dart';

/// ویجت تعاملی یک سلول نمودار — ژست‌های سند ۰۶ §7.3:
/// pinch-zoom حول مرکز pinch، pan، double-tap reset، **نگه‌داشتن بی‌حرکت
/// ۵۰۰ms = crosshair** با snap به نزدیک‌ترین رأس سری.
///
/// نکتهٔ gesture-arena: ScaleGestureRecognizer با فشار بی‌حرکت هیچ‌وقت
/// arena را نمی‌برد (تنها با حرکت می‌پذیرد)؛ پس تشخیص «نگه‌داشتن» با
/// Listener خام + Timer انجام می‌شود — مستقل از رقابت شناسه‌ها.
class PlotView extends StatefulWidget {
  const PlotView({
    super.key,
    required this.cell,
    required this.brightness,
    this.onCrosshair,
  });

  final PlotCellData cell;
  final Brightness brightness;

  /// گزارش وضعیت crosshair (برای شیت اطلاعات/تست) — null = خاموش.
  final ValueChanged<CrosshairSnap?>? onCrosshair;

  @override
  State<PlotView> createState() => PlotViewState();
}

class PlotViewState extends State<PlotView> {
  PlotViewTransform _transform = PlotViewTransform.identity;
  CrosshairSnap? _crosshair;

  /// نگه‌داشتن بی‌حرکت = crosshair (سند ۰۶ §7.3).
  static const _holdDuration = Duration(milliseconds: 500);
  static const _holdSlop = 14.0;

  Timer? _holdTimer;
  Offset? _holdStart;
  bool _crosshairMode = false;
  Size _viewSize = Size.zero;

  /// برای خروجی PNG — ریست به نمای اولیه.
  void resetView() {
    _holdTimer?.cancel();
    _holdTimer = null;
    _holdStart = null;
    setState(() {
      _transform = PlotViewTransform.identity;
      _crosshair = null;
      _crosshairMode = false;
    });
    widget.onCrosshair?.call(null);
  }

  void _setCrosshair(CrosshairSnap? snap) {
    setState(() => _crosshair = snap);
    widget.onCrosshair?.call(snap);
  }

  void _handlePointer(Offset localPosition) {
    if (_viewSize == Size.zero) return;
    final layout = CellLayout.build(widget.cell, _viewSize);
    final scaled = _invert(localPosition, layout.plotRect);
    final (dx, dy) = layout.screenToData(scaled);
    final snap = snapToNearest(widget.cell.crosshairSeries, dx, dy);
    _setCrosshair(snap);
  }

  /// وارون transform داده (فریم ثابت است — فقط داده transform شده):
  /// P_fit = (S − c − o)/s + c
  Offset _invert(Offset screen, Rect rect) {
    final c = rect.center;
    return Offset(
      (screen.dx - c.dx - _transform.offset.dx) / _transform.scale + c.dx,
      (screen.dy - c.dy - _transform.offset.dy) / _transform.scale + c.dy,
    );
  }

  // ----- Listener خام: تشخیص نگه‌داشتن (مستقل از gesture arena) -----

  void _onPointerDown(Offset p) {
    _holdTimer?.cancel();
    _holdStart = p;
    // tap تازه — crosshair قبلی پاک (نگه‌داشتن بعدی دوباره می‌سازد)
    _crosshairMode = false;
    _holdTimer = Timer(_holdDuration, () {
      if (_holdStart != null) {
        _crosshairMode = true;
        _handlePointer(_holdStart!);
      }
    });
  }

  void _onPointerMove(Offset p) {
    // حرکت بزرگ پیش از آستانهٔ نگه‌داشتن = pan؛ تایمر را بی‌اثر کن
    if (!_crosshairMode && _holdStart != null) {
      if ((p - _holdStart!).distance > _holdSlop) {
        _holdTimer?.cancel();
      }
    }
    if (_crosshairMode) {
      _handlePointer(p);
    }
  }

  void _onPointerUp() {
    // chip می‌ماند (خوانا) تا pointer بعدی؛ تایمر نیمه‌کاره بی‌اثر
    _holdTimer?.cancel();
    _holdTimer = null;
    _holdStart = null;
  }

  @override
  void dispose() {
    _holdTimer?.cancel();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return LayoutBuilder(
      builder: (context, constraints) {
        _viewSize = Size(constraints.maxWidth, constraints.maxHeight);
        final size = _viewSize;
        return Listener(
          behavior: HitTestBehavior.translucent,
          onPointerDown: (e) => _onPointerDown(e.localPosition),
          onPointerMove: (e) => _onPointerMove(e.localPosition),
          onPointerUp: (_) => _onPointerUp(),
          onPointerCancel: (_) => _onPointerUp(),
          child: GestureDetector(
            behavior: HitTestBehavior.opaque,
            onScaleStart: (details) {
              if (details.pointerCount >= 2) {
                _holdTimer?.cancel();
                _holdStart = null;
              }
            },
            onScaleUpdate: (details) {
              if (details.pointerCount >= 2) {
                // pinch حول مرکز ژست (نه مرکز صفحه — سند ۰۶ §7.3)
                setState(() {
                  _transform = _transform.pinchAround(
                    details.localFocalPoint,
                    details.scale,
                    size,
                  );
                });
                return;
              }
              if (_crosshairMode) {
                // حرکت در حالت crosshair = جابجایی خط متقاطع (نه pan)
                _handlePointer(details.localFocalPoint);
                return;
              }
              if (details.pointerCount == 1) {
                setState(() {
                  _transform = _transform.translate(details.focalPointDelta);
                });
              }
            },
            onDoubleTap: resetView,
            child: CustomPaint(
              size: size,
              painter: PlotPainter(
                cell: widget.cell,
                transform: _transform,
                brightness: widget.brightness,
                crosshair: _crosshair,
              ),
            ),
          ),
        );
      },
    );
  }
}
