import 'package:flutter/material.dart';

import '../../core/cds/cds_theme.dart';
import '../../core/cds/cds_tokens.dart';
import '../../core/icons/craft_icons.dart';
import '../../core/rust/engine_client.dart';
import '../../l10n/app_localizations.dart';

/// ویجت‌های مشترک رندر نتیجه — بین کنسول و ادیتور (بازخوانی از console_page).
///
/// قرارداد بصری سند ۰۶ §7.1: کارت خطا با نوار قرمز ۳dp سمت چپ، پیام زبان
/// فعال + did-you-mean + کد خطای مونو؛ دکمه کپی با Craft Icon و currentColor.

/// دکمه کپی کوچک CDS — Craft Icon با currentColor، بدون رنگ هاردکد
/// (سند ۰۶ §2.5). رنگ onSurfaceVariant، هدف لمسی ۴۰dp هماهنگ با دکمه run.
class CopyButton extends StatelessWidget {
  const CopyButton({super.key, required this.onCopy, required this.tooltip});

  final VoidCallback onCopy;
  final String tooltip;

  @override
  Widget build(BuildContext context) {
    final scheme = Theme.of(context).colorScheme;
    return IconButton(
      onPressed: onCopy,
      tooltip: tooltip,
      iconSize: 18,
      padding: const EdgeInsets.all(11),
      constraints: const BoxConstraints(minWidth: 40, minHeight: 40),
      icon: CraftIcon(
        CraftIcons.actionCopy,
        size: 18,
        color: scheme.onSurfaceVariant,
      ),
    );
  }
}

/// کارت خطا — نوار قرمز ۳dp سمت چپ + کد + پیام زبان فعال + did-you-mean
/// (سند ۰۶ §7.1، قرارداد سند ۰۲ بخش ۹) + آیکون کپی در ردیف عنوان
/// برای کپی پیام خام (بازخورد P0.2).
class ErrorCard extends StatelessWidget {
  const ErrorCard({super.key, required this.error, required this.onCopy});

  final McError error;
  final VoidCallback onCopy;

  @override
  Widget build(BuildContext context) {
    final scheme = Theme.of(context).colorScheme;
    final l10n = AppLocalizations.of(context)!;
    // «پیام زبان فعال» (سند ۰۶ §7.1) — زبان UI ملاک است نه جهت؛
    // جزیره کنسول/ادیتور همیشه LTR است ولی پیام خطا فارسی می‌ماند.
    final isFa = Localizations.localeOf(context).languageCode == 'fa';
    final message = isFa ? error.messageFa : error.messageEn;

    return Container(
      margin: const EdgeInsets.symmetric(vertical: 4),
      decoration: BoxDecoration(
        color: scheme.surfaceContainerHighest,
        borderRadius: BorderRadius.circular(CdsDimens.radiusSmall),
        border: Border.all(color: scheme.outline),
      ),
      child: ClipRRect(
        borderRadius: BorderRadius.circular(CdsDimens.radiusSmall),
        // IntrinsicHeight — کارت در ListView بی‌کران (ارتفاع نامحدود) هم
        // امن است؛ نوار قرمز با stretch به ارتفاع محتوا می‌رسد.
        child: IntrinsicHeight(
          child: Row(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              Container(width: 3, color: scheme.error),
              Expanded(
                child: Padding(
                  padding: const EdgeInsets.all(12),
                  child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Row(
                      children: [
                        Expanded(
                          child: Text(
                            l10n.errorTitle(error.id),
                            style: CdsTheme.monoStyle(
                              color: scheme.error,
                              size: 11,
                            ),
                          ),
                        ),
                        CopyButton(onCopy: onCopy, tooltip: l10n.copyTooltip),
                      ],
                    ),
                    const SizedBox(height: 4),
                    Text(
                      message,
                      style: Theme.of(context).textTheme.bodyMedium?.copyWith(
                            color: scheme.onSurface,
                            height: 1.6,
                          ),
                    ),
                    if (error.suggestion != null) ...[
                      const SizedBox(height: 6),
                      Text(
                        l10n.suggestionLabel(error.suggestion!),
                        style: Theme.of(context)
                            .textTheme
                            .bodySmall
                            ?.copyWith(color: scheme.onSurfaceVariant),
                      ),
                    ],
                  ],
                ),
              ),
            ),
          ],
          ),
        ),
      ),
    );
  }
}
