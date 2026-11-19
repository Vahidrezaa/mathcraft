import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/scheduler.dart';

import 'plot_geometry.dart';
import 'plot_painter.dart';

/// ویجت تعاملی یک سلول نمودار — بازطراحی 3.4.1:
///
/// - **پن:** درگ تک‌انگشتی = جابه‌جایی viewport در واحد داده (محتوا با
///   انگشت حرکت می‌کند؛ تیک‌ها همیشه برای دید جاری ساخته می‌شوند)
/// - **زوم:** pinch حول نقطهٔ زیر انگشت‌ها (focal داده‌ای ثابت می‌ماند)؛
///   ضریب به‌صورت افزایشی روی details.scale اعمال می‌شود
/// - **بازنشانی:** double-tap = نمای خانه (حدود fit موتور)
/// - **اطلاعات:** تپ نزدیک یک رأس = data-tip پایدار با نام سری و (x, y)؛
///   نگه‌داشتن ۵۰۰ms = crosshair زنده با تگ مقدار روی هر دو محور
///
/// نکتهٔ arena: ScaleGestureRecognizer با فشار بی‌حرکت هیچ‌وقت arena را
/// نمی‌برد؛ تشخیص «نگه‌داشتن» با Listener خام + Timer است (مستقل از رقابت).
class PlotView extends StatefulWidget {
  const PlotView({
    super.key,
    required this.cell,
    required this.brightness,
    this.onCrosshair,
  });

  final PlotCellData cell;
  final Brightness brightness;

  /// گزارش وضعیت crosshair (برای تست) — null = خاموش.
  final ValueChanged<CrosshairSnap?>? onCrosshair;

  @override
  State<PlotView> createState() => PlotViewState();
}

class PlotViewState extends State<PlotView> {
  late PlotViewport _viewport;
  CrosshairSnap? _crosshair;
  DataTip? _dataTip;

  /// نگه‌داشتن بی‌حرکت = crosshair (سند ۰۶ §7.3).
  static const _holdDuration = Duration(milliseconds: 500);
  static const _holdSlop = 14.0;

  Timer? _holdTimer;
  Offset? _holdStart;
  bool _crosshairMode = false;
  Size _viewSize = Size.zero;

  /// double-tap خام در Listener (با frame timestamp مونوتیک — pump تست هم
  /// آن را جلو می‌برد)؛ اگر onDoubleTap معمولی می‌بود، هر تپِ تک ۳۰۰ms
  /// برای باختنِ رقابت double-tap تأخیر می‌خورد — برای tip زنده غیرقابل‌قبول.
  static const _doubleTapWindow = Duration(milliseconds: 300);
  static const _doubleTapSlop = 40.0;
  Duration? _lastTapTs;
  Offset? _lastTapPos;
  bool _suppressTap = false;

  /// مقیاس تجمعی ژست pinch جاری — ضریب افزایشی برای zoomAt.
  double _gestureScale = 1.0;

  bool get _isPolar => widget.cell is PolarCellData;

  /// سلول سه‌بعدی؟ (surf/mesh/plot3 — درگ = چرخش دوربین، سند ۰۶ §7.3)
  bool get _is3D => widget.cell is SurfaceCellData || widget.cell is Plot3CellData;

  Axes3DData get _axes3dOf {
    final c = widget.cell;
    if (c is SurfaceCellData) return c.axes3d;
    if (c is Plot3CellData) return c.axes3d;
    throw StateError('not a 3-D cell');
  }

  /// چرخش محلی (az, el) — null = نمای قرارداد DTO.
  (double, double)? _view3d;

  PlotViewport get _home {
    // پلار و سه‌بعدی محور دوبعدی ندارند — دید شناور بی‌معنا
    if (_isPolar || _is3D) return PlotViewport.any;
    return PlotViewport.home(widget.cell.axes!);
  }

  /// برای تست‌ها — دید و tip جاری.
  PlotViewport get viewport => _viewport;
  DataTip? get dataTip => _dataTip;

  /// چرخش سه‌بعدی محلی جاری — null = نمای قرارداد (برای تست).
  (double, double)? get viewOverride3D => _view3d;

  @override
  void initState() {
    super.initState();
    _viewport = _home;
  }

  @override
  void didUpdateWidget(covariant PlotView oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.cell != widget.cell) {
      // شکل عوض شد — نمای خانه و پاک‌سازی لایه‌های اطلاعاتی
      _holdTimer?.cancel();
      _holdStart = null;
      _crosshairMode = false;
      _viewport = _home;
      _crosshair = null;
      _dataTip = null;
      _view3d = null;
    }
  }

  /// بازنشانی کامل به نمای خانه (خروجی PNG هم از این استفاده می‌کند).
  void resetView() {
    _holdTimer?.cancel();
    _holdTimer = null;
    _holdStart = null;
    setState(() {
      _viewport = _home;
      _crosshair = null;
      _dataTip = null;
      _crosshairMode = false;
      _view3d = null; // سه‌بعدی: بازگشت به view قرارداد
    });
    widget.onCrosshair?.call(null);
  }

  void _setCrosshair(CrosshairSnap? snap) {
    setState(() => _crosshair = snap);
    widget.onCrosshair?.call(snap);
  }

  CellLayout _layout() =>
      CellLayout.build(widget.cell, _viewSize, _viewport);

  // ----- crosshair (نگه‌داشتن) -----

  void _crosshairAt(Offset localPosition) {
    if (_isPolar || _is3D || _viewSize == Size.zero) return;
    final layout = _layout();
    final (dx, dy) = layout.screenToData(localPosition);
    _setCrosshair(snapToNearest(widget.cell.crosshairSeries, dx, dy));
  }

  // ----- Listener خام: تشخیص نگه‌داشتن (مستقل از gesture arena) -----

  void _onPointerDown(Offset p) {
    _holdTimer?.cancel();
    _holdStart = p;
    _crosshairMode = false;
    _holdTimer = Timer(_holdDuration, () {
      if (_holdStart != null) {
        _crosshairMode = true;
        _crosshairAt(_holdStart!);
      }
    });
  }

  void _onPointerMove(Offset p) {
    if (_crosshairMode) {
      _crosshairAt(p);
      return;
    }
    // حرکت بزرگ پیش از آستانه = pan؛ تایمر نگه‌داشتن بی‌اثر
    if (_holdStart != null && (p - _holdStart!).distance > _holdSlop) {
      _holdTimer?.cancel();
    }
  }

  void _onPointerUp(Offset position) {
    // chip می‌ماند (خوانا) تا pointer بعدی؛ تایمر نیمه‌کاره بی‌اثر
    _holdTimer?.cancel();
    _holdTimer = null;
    _holdStart = null;
    // double-tap خام: دو بالا-آمدن نزدیک در پنجرهٔ ۳۰۰ms
    final now = SchedulerBinding.instance.currentSystemFrameTimeStamp;
    final lastTs = _lastTapTs;
    final lastPos = _lastTapPos;
    _lastTapTs = now;
    _lastTapPos = position;
    if (lastTs != null &&
        now - lastTs <= _doubleTapWindow &&
        (position - (lastPos ?? position)).distance <= _doubleTapSlop) {
      _lastTapTs = null;
      _suppressTap = true; // تپ دومی tip نکارد — این بازنشانی است
      resetView();
    }
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
          onPointerUp: (e) => _onPointerUp(e.localPosition),
          onPointerCancel: (_) => _onPointerUp(Offset.infinite),
          child: GestureDetector(
            behavior: HitTestBehavior.opaque,
            onScaleStart: (details) {
              _gestureScale = 1.0;
              if (details.pointerCount >= 2) {
                _holdTimer?.cancel();
                _holdStart = null;
              }
            },
            onScaleUpdate: (details) {
              if (_isPolar) return;
              if (_is3D) {
                // چرخش لمسی (گام 3.8 — DoD سند ۰۷ §P3): افقی = az،
                // عمودی = el (کلمپ ±۸۹ تا گیمبال نچرخد)
                if (details.pointerCount == 1) {
                  final base = _view3d ?? _axes3dOf.view;
                  final d = details.focalPointDelta;
                  setState(() {
                    _view3d = (
                      base.$1 + d.dx * 0.5,
                      (base.$2 + d.dy * 0.5).clamp(-89.0, 89.0),
                    );
                  });
                }
                return;
              }
              if (details.pointerCount >= 2) {
                // pinch حول focal جاری — نقطهٔ داده زیر انگشت‌ها ثابت
                final factor = details.scale / _gestureScale;
                _gestureScale = details.scale;
                if (factor.isFinite && factor > 0 && factor != 1.0) {
                  final layout = _layout();
                  final (fx, fy) =
                      layout.screenToData(details.localFocalPoint);
                  setState(() {
                    _viewport = _viewport
                        .zoomAt(focalX: fx, focalY: fy, factor: factor)
                        .clamped(_home);
                  });
                }
                return;
              }
              if (_crosshairMode) {
                // حرکت در حالت crosshair = جابه‌جایی خط متقاطع (نه pan)
                _crosshairAt(details.localFocalPoint);
                return;
              }
              if (details.pointerCount == 1) {
                // پن طبیعی: محتوا با انگشت
                setState(() {
                  _viewport = _viewport
                      .panPixels(details.focalPointDelta, size)
                      .clamped(_home);
                });
              }
            },
            onTapUp: (details) {
              if (_suppressTap) {
                _suppressTap = false;
                return;
              }
              if (_isPolar || _is3D || _viewSize == Size.zero) return;
              // تپ نزدیک رأس = tip پایدار؛ تپ دور = پاک
              final tip = pickTip(
                widget.cell.crosshairSeries,
                _layout(),
                positionPx: details.localPosition,
              );
              setState(() => _dataTip = tip);
            },
            child: CustomPaint(
              size: size,
              painter: PlotPainter(
                cell: widget.cell,
                viewport: _viewport,
                brightness: widget.brightness,
                crosshair: _crosshair,
                dataTip: _dataTip,
                viewOverride: _view3d,
              ),
            ),
          ),
        );
      },
    );
  }
}
