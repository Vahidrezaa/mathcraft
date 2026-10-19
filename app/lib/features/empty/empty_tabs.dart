import 'package:flutter/material.dart';
import '../../core/cds/cds_theme.dart';
import '../../core/cds/cds_tokens.dart';
import '../../l10n/app_localizations.dart';

/// تب‌های هنوز فعال‌نشده — طبق سند ۰۶ بخش ۹، حالت خالی «خوش‌آمد عملی» است
/// نه لوگوی خالی؛ و طبق اصل صداقت (سند ۰۰ اصل ۵)، مایل‌استون واقعی رسیدن
/// هر قابلیت اعلام می‌شود (سند ۰۷).
class EditorTab extends StatelessWidget {
  const EditorTab({super.key});

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    return _EmptyTab(
      title: l10n.emptyEditorTitle,
      hint: l10n.emptyEditorHint,
      codeSample: 'x = 1:5',
    );
  }
}

class PlotsTab extends StatelessWidget {
  const PlotsTab({super.key});

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    return _EmptyTab(
      title: l10n.emptyPlotsTitle,
      hint: l10n.emptyPlotsHint,
      codeSample: 'fxplot(sin, [0 6])',
    );
  }
}

class VarsTab extends StatelessWidget {
  const VarsTab({super.key});

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    return _EmptyTab(
      title: l10n.emptyVarsTitle,
      hint: l10n.emptyVarsHint,
      codeSample: 'A = 2',
    );
  }
}

class LearnTab extends StatelessWidget {
  const LearnTab({super.key});

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    return _EmptyTab(
      title: l10n.emptyLearnTitle,
      hint: l10n.emptyLearnHint,
      codeSample: 'doc inv',
    );
  }
}

class _EmptyTab extends StatelessWidget {
  const _EmptyTab({
    required this.title,
    required this.hint,
    required this.codeSample,
  });

  final String title;
  final String hint;

  /// یک نمونه کد واقعی از آینده — جزیره LTR همیشگی (سند ۰۶ بخش ۴ بند ۲).
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
              padding: const EdgeInsets.symmetric(
                horizontal: 14,
                vertical: 10,
              ),
              decoration: BoxDecoration(
                color: scheme.surfaceContainerHighest,
                borderRadius:
                    BorderRadius.circular(CdsDimens.radiusSmall),
                border: Border.all(color: scheme.outline),
              ),
              child: Text(
                codeSample,
                style: CdsTheme.monoStyle(
                  color: scheme.onSurface,
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
