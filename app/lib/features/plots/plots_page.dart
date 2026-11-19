import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../core/cds/cds_tokens.dart';
import '../../core/icons/craft_icons.dart';
import '../../l10n/app_localizations.dart';
import '../../state/plots_state.dart';
import '../../src/rust/api.dart' as api;
import 'plot_export.dart';
import 'plot_geometry.dart';
import 'plot_painter.dart';
import 'plot_view.dart';

/// تب نمودار — مشخصات سند ۰۶ §5.1 و §7.3:
/// - آخرین figure بزرگ + نوار فهرست figures (chipها)
/// - نوار شناور شفاف: فهرست، ذخیره PNG، اشتراک، اطلاعات (Craft Icons فقط)
/// - ژست‌ها در PlotView: pinch حول focal، pan، double-tap reset، crosshair
/// - جزیرهٔ LTR همیشگی برای محورها (سند ۰۶ §4) — chrome فارسی RTL می‌ماند
/// - مدل pull سند ۰۲ §7: فعال‌شدن تب = get_plots؛ push نتیجهٔ اجرا =
///   plotsProvider.applyResult (بدون رفت‌وبرگشت اضافه)
class PlotsTab extends ConsumerStatefulWidget {
  const PlotsTab({super.key});

  @override
  ConsumerState<PlotsTab> createState() => _PlotsTabState();
}

class _PlotsTabState extends ConsumerState<PlotsTab> {
  final _boundaryKey = GlobalKey();
  int _selected = 0;
  bool _showList = false;

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (mounted) ref.read(plotsProvider.notifier).refresh();
    });
  }

  void _onTabActivated(int tab) {
    if (tab == 2) {
      ref.read(plotsProvider.notifier).refresh();
    }
  }

  /// خروجی PDF برداری از موتور (گام 3.8) — دکمهٔ جدا از PNG.
  Future<void> _exportPdf() async {
    final l10n = AppLocalizations.of(context)!;
    final figures = ref.read(plotsProvider).figures;
    if (figures.isEmpty) return;
    final fig = figures[_selected.clamp(0, figures.length - 1)];
    try {
      final path = await exportFigurePdf(
        fig.figureId,
        'mathcraft-plot-${fig.figureId}.pdf',
      );
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text(l10n.plotsSavedTo(path))),
      );
    } catch (e) {
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text(l10n.plotsExportFailed(e.toString()))),
      );
    }
  }

  Future<void> _export({required bool share}) async {
    final l10n = AppLocalizations.of(context)!;
    final figures = ref.read(plotsProvider).figures;
    if (figures.isEmpty) return;
    final fig = figures[_selected.clamp(0, figures.length - 1)];
    final fileName = 'mathcraft-plot-${fig.figureId}.png';
    try {
      final image = await captureBoundaryToImage(_boundaryKey);
      final png = await imageToPng(image);
      if (!mounted) return;
      if (share) {
        await sharePng(png, fileName);
      } else {
        final path = await savePng(png, fileName);
        if (!mounted) return;
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text(l10n.plotsSavedTo(path))),
        );
      }
    } catch (e) {
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text(l10n.plotsExportFailed(e.toString()))),
      );
    }
  }

  void _showInfo(api.PlotFigure fig) {
    final l10n = AppLocalizations.of(context)!;
    // باگ 3.5.2: قبل از باز کردن شیت، فوکوسِ احتمالی پس‌زمینه را رها کن
    // تا با بستن آن، کیبوردِ editor بالا نیاید.
    FocusManager.instance.primaryFocus?.unfocus();
    final (kind, seriesCount, points, rawPoints) = _describe(fig.spec, l10n);
    showModalBottomSheet<void>(
      context: context,
      shape: const RoundedRectangleBorder(
        borderRadius: BorderRadius.vertical(
          top: Radius.circular(CdsDimens.radiusSheet),
        ),
      ),
      builder: (ctx) => SafeArea(
        child: Padding(
          padding: const EdgeInsets.all(CdsDimens.gutterPortrait),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              Text(
                l10n.plotsInfoTitle,
                style: Theme.of(ctx).textTheme.titleMedium?.copyWith(
                      fontWeight: FontWeight.w700,
                    ),
              ),
              const SizedBox(height: 12),
              _infoRow(ctx, l10n.plotsInfoFigureId, '${fig.figureId}'),
              _infoRow(ctx, l10n.plotsInfoKind, kind),
              _infoRow(ctx, l10n.plotsInfoSeries, '$seriesCount'),
              _infoRow(ctx, l10n.plotsInfoPoints, '$points'),
              _infoRow(ctx, l10n.plotsInfoRawPoints, '$rawPoints'),
              const SizedBox(height: 8),
              Text(
                l10n.plotsCrosshairHint,
                style: Theme.of(ctx).textTheme.bodySmall?.copyWith(
                      color: Theme.of(ctx).colorScheme.onSurfaceVariant,
                    ),
              ),
            ],
          ),
        ),
      ),
    );
  }

  Widget _infoRow(BuildContext ctx, String label, String value) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 4),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.spaceBetween,
        children: [
          Text(
            label,
            style: Theme.of(ctx).textTheme.bodyMedium?.copyWith(
                  color: Theme.of(ctx).colorScheme.onSurfaceVariant,
                ),
          ),
          Text(
            value,
            style: Theme.of(ctx).textTheme.bodyMedium?.copyWith(
                  fontWeight: FontWeight.w600,
                ),
          ),
        ],
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final state = ref.watch(plotsProvider);
    ref.listen(activeTabProvider, (_, tab) => _onTabActivated(tab));
    final l10n = AppLocalizations.of(context)!;

    if (state.error != null && state.figures.isEmpty) {
      return _EmptyPlots(
        title: l10n.plotsEngineError,
        hint: state.error.toString(),
        codeSample: 'plot(0:0.1:2*pi, sin(0:0.1:2*pi))',
      );
    }
    if (state.figures.isEmpty) {
      return _EmptyPlots(
        title: l10n.plotsEmptyTitle,
        hint: l10n.plotsEmptyHint,
        codeSample: 'plot(0:0.1:2*pi, sin(0:0.1:2*pi))',
      );
    }

    final index = _selected.clamp(0, state.figures.length - 1);
    final fig = state.figures[index];
    final brightness = Theme.of(context).brightness;

    return Stack(
      children: [
        // بدنه — جزیرهٔ LTR (محورها همیشه LTR — سند ۰۶ §4 بند ۲)
        Positioned.fill(
          child: Directionality(
            textDirection: TextDirection.ltr,
            child: _figureBody(fig, brightness),
          ),
        ),
        // فهرست chip شکل‌ها (بالا — روشن/خاموش با دکمهٔ خانه)
        if (_showList)
          Positioned(
            top: 8,
            left: 60,
            right: 12,
            child: SingleChildScrollView(
              scrollDirection: Axis.horizontal,
              child: Row(
                children: [
                  for (var i = 0; i < state.figures.length; i++)
                    Padding(
                      padding: const EdgeInsetsDirectional.only(end: 8),
                      child: _FigureChip(
                        label: l10n.plotsFigureLabel(
                          state.figures[i].figureId.toInt(),
                        ),
                        selected: i == index,
                        onTap: () => setState(() {
                          _selected = i;
                          _showList = false;
                        }),
                      ),
                    ),
                ],
              ),
            ),
          ),
        // نوار شناور شفاف — سند ۰۶ §7.3 (فهرست/ذخیره/اشتراک/اطلاعات)
        Positioned(
          top: 8,
          left: 8,
          child: _FloatingBar(
            onList: () => setState(() => _showList = !_showList),
            onSave: () => _export(share: false),
            onPdf: _exportPdf,
            onShare: () => _export(share: true),
            onInfo: () => _showInfo(fig),
          ),
        ),
      ],
    );
  }

  Widget _figureBody(api.PlotFigure fig, Brightness brightness) {
    final spec = fig.spec;
    if (spec is api.PlotSpec_Subplot) {
      return _SubplotGrid(spec: spec, brightness: brightness);
    }
    final cell = cellDataFromSpec(spec);
    if (cell == null) {
      return _NotYet3D();
    }
    return RepaintBoundary(
      key: _boundaryKey,
      child: PlotView(cell: cell, brightness: brightness),
    );
  }
}

// ---------- بدنه‌ها ----------

class _FigureChip extends StatelessWidget {
  const _FigureChip({
    required this.label,
    required this.selected,
    required this.onTap,
  });

  final String label;
  final bool selected;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    final scheme = Theme.of(context).colorScheme;
    return Material(
      color: selected ? scheme.primaryContainer : scheme.surfaceContainerHighest,
      borderRadius: BorderRadius.circular(CdsDimens.radiusSmall),
      child: InkWell(
        onTap: onTap,
        borderRadius: BorderRadius.circular(CdsDimens.radiusSmall),
        child: Padding(
          padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
          child: Text(
            label,
            style: TextStyle(
              color: selected
                  ? scheme.onPrimaryContainer
                  : scheme.onSurface,
              fontWeight: selected ? FontWeight.w700 : FontWeight.w500,
            ),
          ),
        ),
      ),
    );
  }
}

class _FloatingBar extends StatelessWidget {
  const _FloatingBar({
    required this.onList,
    required this.onSave,
    required this.onPdf,
    required this.onShare,
    required this.onInfo,
  });

  final VoidCallback onList;
  final VoidCallback onSave;
  final VoidCallback onPdf;
  final VoidCallback onShare;
  final VoidCallback onInfo;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    final scheme = Theme.of(context).colorScheme;
    Widget button(CraftIconData icon, String tooltip, VoidCallback onTap) {
      return Material(
        color: scheme.surface.withOpacity(0.85),
        borderRadius: BorderRadius.circular(CdsDimens.radiusSmall),
        child: InkWell(
          onTap: onTap,
          borderRadius: BorderRadius.circular(CdsDimens.radiusSmall),
          child: Tooltip(
            message: tooltip,
            child: SizedBox(
              width: CdsDimens.touchTarget - 8,
              height: CdsDimens.touchTarget - 8,
              child: Center(child: CraftIcon(icon, size: 20)),
            ),
          ),
        ),
      );
    }

    return Container(
      padding: const EdgeInsets.all(4),
      decoration: BoxDecoration(
        color: scheme.surface.withOpacity(0.55),
        borderRadius: BorderRadius.circular(CdsDimens.radiusSmall + 4),
      ),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          button(CraftIcons.plotList, l10n.plotsListTooltip, onList),
          const SizedBox(height: 4),
          button(CraftIcons.actionSave, l10n.plotsSaveTooltip, onSave),
          const SizedBox(height: 4),
          // دکمهٔ PDF — برچسب متنی (آیکون اختصاصی هنوز در مجموعه نیست)
          Material(
            color: scheme.surface.withOpacity(0.85),
            borderRadius: BorderRadius.circular(CdsDimens.radiusSmall),
            child: InkWell(
              onTap: onPdf,
              borderRadius: BorderRadius.circular(CdsDimens.radiusSmall),
              child: Tooltip(
                message: l10n.plotsPdfTooltip,
                child: SizedBox(
                  width: CdsDimens.touchTarget - 8,
                  height: CdsDimens.touchTarget - 8,
                  child: Center(
                    child: Text(
                      'PDF',
                      style: TextStyle(
                        fontFamily: 'JetBrainsMono',
                        fontSize: 10,
                        fontWeight: FontWeight.w700,
                        color: scheme.primary,
                      ),
                    ),
                  ),
                ),
              ),
            ),
          ),
          const SizedBox(height: 4),
          button(CraftIcons.actionShare, l10n.plotsShareTooltip, onShare),
          const SizedBox(height: 4),
          button(CraftIcons.actionInfo, l10n.plotsInfoTooltip, onInfo),
        ],
      ),
    );
  }
}

/// حالت خالی «خوش‌آمد عملی» — سند ۰۶ §9؛ نمونهٔ کد جزیرهٔ LTR.
class _EmptyPlots extends StatelessWidget {
  const _EmptyPlots({
    required this.title,
    required this.hint,
    required this.codeSample,
  });

  final String title;
  final String hint;
  final String codeSample;

  @override
  Widget build(BuildContext context) {
    final scheme = Theme.of(context).colorScheme;
    return Center(
      child: Padding(
        padding: const EdgeInsets.all(CdsDimens.gutterPortrait * 2),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Container(
              padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 10),
              decoration: BoxDecoration(
                color: scheme.surfaceContainerHighest,
                borderRadius: BorderRadius.circular(CdsDimens.radiusSmall),
                border: Border.all(color: scheme.outline),
              ),
              child: Directionality(
                textDirection: TextDirection.ltr,
                child: Text(
                  codeSample,
                  style: Theme.of(context)
                      .textTheme
                      .bodyMedium
                      ?.copyWith(fontFamily: 'JetBrainsMono'),
                ),
              ),
            ),
            const SizedBox(height: 20),
            Text(
              title,
              textAlign: TextAlign.center,
              style: Theme.of(context)
                  .textTheme
                  .titleMedium
                  ?.copyWith(fontWeight: FontWeight.w700),
            ),
            const SizedBox(height: 8),
            Text(
              hint,
              textAlign: TextAlign.center,
              style: Theme.of(context).textTheme.bodyMedium?.copyWith(
                    color: scheme.onSurfaceVariant,
                    height: 1.7,
                  ),
            ),
          ],
        ),
      ),
    );
  }
}

/// گونه‌های قرارداد که رندرشان با P3-b می‌آید — حالت صادقانه (اصل ۵ سند ۰۰).
class _NotYet3D extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    final scheme = Theme.of(context).colorScheme;
    return Center(
      child: Padding(
        padding: const EdgeInsets.all(CdsDimens.gutterPortrait * 2),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            CraftIcon(CraftIcons.navPlots, size: 32, color: scheme.primary),
            const SizedBox(height: 16),
            Text(
              l10n.plotsNot3dTitle,
              textAlign: TextAlign.center,
              style: Theme.of(context)
                  .textTheme
                  .titleMedium
                  ?.copyWith(fontWeight: FontWeight.w700),
            ),
            const SizedBox(height: 8),
            Text(
              l10n.plotsNot3dHint,
              textAlign: TextAlign.center,
              style: Theme.of(context).textTheme.bodyMedium?.copyWith(
                    color: scheme.onSurfaceVariant,
                    height: 1.7,
                  ),
            ),
          ],
        ),
      ),
    );
  }
}

/// subplot — شبکهٔ سلول‌ها؛ در این نسخه فقط‌نمایش (بدون ژست per-cell).
class _SubplotGrid extends StatelessWidget {
  const _SubplotGrid({required this.spec, required this.brightness});

  final api.PlotSpec_Subplot spec;
  final Brightness brightness;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    return Column(
      children: [
        Padding(
          padding: const EdgeInsets.all(8),
          child: Text(
            l10n.plotsSubplotNote,
            style: Theme.of(context).textTheme.bodySmall?.copyWith(
                  color: Theme.of(context).colorScheme.onSurfaceVariant,
                ),
          ),
        ),
        Expanded(
          child: Padding(
            padding: const EdgeInsets.all(8),
            child: LayoutBuilder(
              builder: (context, constraints) {
                final cellW =
                    constraints.maxWidth / spec.cols.clamp(1, 16);
                final cellH = constraints.maxHeight / spec.rows.clamp(1, 16);
                return Stack(
                  children: [
                    for (final c in spec.cells)
                      Positioned(
                        left: c.col * cellW,
                        top: c.row * cellH,
                        width: cellW,
                        height: cellH,
                        child: _SubplotCellView(
                          spec: c.spec,
                          brightness: brightness,
                        ),
                      ),
                  ],
                );
              },
            ),
          ),
        ),
      ],
    );
  }
}

class _SubplotCellView extends StatelessWidget {
  const _SubplotCellView({required this.spec, required this.brightness});

  final api.PlotSpec spec;
  final Brightness brightness;

  @override
  Widget build(BuildContext context) {
    final cell = cellDataFromSpec(spec);
    if (cell == null) {
      return DecoratedBox(
        decoration: BoxDecoration(
          border: Border.all(color: Theme.of(context).colorScheme.outline),
        ),
        child: Center(
          child: Text(
            'P3-b',
            style: TextStyle(
              fontFamily: 'JetBrainsMono',
              fontSize: 11,
              color: Theme.of(context).colorScheme.onSurfaceVariant,
            ),
          ),
        ),
      );
    }
    return CustomPaint(
      painter: PlotPainter(
        cell: cell,
        viewport: cell is PolarCellData
            ? PlotViewport.any
            : PlotViewport.home(cell.axes!),
        brightness: brightness,
      ),
      child: const SizedBox.expand(),
    );
  }
}

// ---------- توصیف شکل برای شیت اطلاعات ----------

(String, int, int, int) _describe(
  api.PlotSpec spec,
  AppLocalizations l10n,
) {
  return switch (spec) {
    api.PlotSpec_Line2D(:final series) => (
        l10n.plotsKindLine,
        series.length,
        _sumPoints(series.map((s) => s.x.length)),
        _sumRaw(series.map((s) => s.rawLen.toInt())),
      ),
    api.PlotSpec_Scatter2D(:final series) => (
        l10n.plotsKindScatter,
        series.length,
        _sumPoints(series.map((s) => s.x.length)),
        _sumRaw(series.map((s) => s.rawLen.toInt())),
      ),
    api.PlotSpec_Bar2D(:final series) => (
        l10n.plotsKindBar,
        series.length,
        _sumPoints(series.map((s) => s.x.length)),
        _sumRaw(series.map((s) => s.rawLen.toInt())),
      ),
    api.PlotSpec_Histogram(:final counts) => (
        l10n.plotsKindHist,
        1,
        counts.length,
        counts.length,
      ),
    api.PlotSpec_Area2D(:final series) => (
        l10n.plotsKindArea,
        series.length,
        _sumPoints(series.map((s) => s.x.length)),
        _sumRaw(series.map((s) => s.rawLen.toInt())),
      ),
    api.PlotSpec_Fill2D(:final series) => (
        l10n.plotsKindFill,
        series.length,
        _sumPoints(series.map((s) => s.x.length)),
        _sumRaw(series.map((s) => s.rawLen.toInt())),
      ),
    api.PlotSpec_Polar(:final series) => (
        l10n.plotsKindPolar,
        series.length,
        _sumPoints(series.map((s) => s.x.length)),
        _sumRaw(series.map((s) => s.rawLen.toInt())),
      ),
    api.PlotSpec_ErrorBar2D(:final series) => (
        l10n.plotsKindError,
        series.length,
        _sumPoints(series.map((s) => s.x.length)),
        _sumRaw(series.map((s) => s.rawLen.toInt())),
      ),
    api.PlotSpec_Quiver() => (l10n.plotsKindQuiver, 0, 0, 0),
    api.PlotSpec_Surface() ||
    api.PlotSpec_Mesh() ||
    api.PlotSpec_Contour() ||
    api.PlotSpec_Plot3() =>
      (l10n.plotsKind3d, 0, 0, 0),
    api.PlotSpec_Subplot(:final cells) => (
        l10n.plotsKindSubplot,
        cells.length,
        0,
        0,
      ),
  };
}

int _sumPoints(Iterable<int> lens) => lens.fold(0, (a, b) => a + b);
int _sumRaw(Iterable<int> lens) => lens.fold(0, (a, b) => a + b);
