import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../core/cds/cds_theme.dart';
import '../../core/cds/cds_tokens.dart';
import '../../core/icons/craft_icons.dart';
import '../../core/rust/engine_client.dart';
import '../../core/rust/engine_provider.dart';
import '../../l10n/app_localizations.dart';
import '../console/console_widgets.dart';
import '../shell/app_shell.dart';
import 'matlab_highlight.dart';

/// ادیتور پایه P1 (آیتم DoD #8 — سند ۰۹ §2):
/// - هایلایت سینتکس متلب (کلیدواژه/عدد/رشته/کامنت) با فونت مونو JetBrains
/// - دکمه Run: اجرای کل بافر روی موتور — تعریف‌های function فوراً ثبت و
///   فراخوانی‌پذیر می‌شوند (سمانتیک متلب §9)؛ گزاره‌های اسکریپت اجرا می‌شوند
/// - پنل خروجی با همان قرارداد کنسول (خط + کارت خطا + کپی)
/// - جزیره LTR همیشگی (سند ۰۶ بخش ۴ بند ۲) حتی در UI فارسی
class EditorPage extends ConsumerStatefulWidget {
  const EditorPage({super.key});

  @override
  ConsumerState<EditorPage> createState() => _EditorPageState();
}

/// یک سطر پنل خروجی ادیتور — مدل زبان‌خنثی.
sealed class EditorEntry {
  final String id;
  EditorEntry() : id = DateTime.now().microsecondsSinceEpoch.toString();
}

class TextEntry extends EditorEntry {
  TextEntry(this.text);
  final String text;
}

class ErrEntry extends EditorEntry {
  ErrEntry(this.error);
  final McError error;
}

class _EditorPageState extends ConsumerState<EditorPage> {
  late final MatlabCodeController _code;
  final _outputScroll = ScrollController();
  final _entries = <EditorEntry>[];
  bool _running = false;

  @override
  void initState() {
    super.initState();
    _code = MatlabCodeController();
  }

  @override
  void dispose() {
    _code.dispose();
    _outputScroll.dispose();
    super.dispose();
  }

  Future<void> _run() async {
    final code = _code.text.trim();
    if (code.isEmpty || _running) return;
    setState(() {
      _running = true;
      _entries
        ..clear()
        ..add(TextEntry('>> run'));
    });
    HapticFeedback.lightImpact();

    try {
      final result = await ref.read(engineProvider).evaluateLine(code);
      final fresh = <EditorEntry>[];
      for (final event in result.consoleEvents) {
        switch (event) {
          case ConsoleEvent_Print(:final text):
            if (text.isNotEmpty) fresh.add(TextEntry(text));
          case ConsoleEvent_Error(:final err):
            fresh.add(ErrEntry(err));
          case ConsoleEvent_Warning():
            break; // هشدارهای ساختاریافته در P2 (سند ۰۲ بخش ۳)
        }
      }
      if (result.status == JobStatus.done) {
        HapticFeedback.mediumImpact();
      }
      if (!mounted) return;
      setState(() {
        _entries
          ..clear()
          ..addAll(fresh);
      });
    } catch (e) {
      // موتور بوت نشده (مثلاً بدون بیلد Rust) — پیام واضح، نه کرش (سند ۰۲ §3).
      if (!mounted) return;
      setState(() => _entries.add(TextEntry('engine unavailable: $e')));
    } finally {
      if (mounted) setState(() => _running = false);
    }
  }

  String _copyTextOf(EditorEntry e) => switch (e) {
        TextEntry(:final text) => text,
        ErrEntry(:final error) => error.messageEn,
      };

  Future<void> _copyToClipboard(String text) async {
    await Clipboard.setData(ClipboardData(text: text));
    if (!mounted) return;
    final l10n = AppLocalizations.of(context)!;
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(content: Text(l10n.copiedToClipboard)),
    );
  }

  @override
  Widget build(BuildContext context) {
    final scheme = Theme.of(context).colorScheme;
    final l10n = AppLocalizations.of(context)!;

    return Column(
      children: [
        // نوار ابزار — Run accent بزرگ (هم‌خوان با کنسول، سند ۰۶ §6)
        SafeArea(
          bottom: false,
          child: Container(
            decoration: BoxDecoration(
              color: scheme.surface,
              border: Border(bottom: BorderSide(color: scheme.outline)),
            ),
            child: Padding(
              padding: const EdgeInsets.symmetric(
                horizontal: CdsDimens.gutterPortrait,
                vertical: 8,
              ),
              child: Row(
                children: [
                  Expanded(
                    child: Text(
                      l10n.editorOutputTitle,
                      style: Theme.of(context).textTheme.labelMedium?.copyWith(
                            color: scheme.onSurfaceVariant,
                          ),
                    ),
                  ),
                  CdsPressable(
                    onTap: () {
                      _code.clear();
                      setState(_entries.clear);
                    },
                    child: Tooltip(
                      message: l10n.editorClearTooltip,
                      child: Container(
                        width: 40,
                        height: 40,
                        decoration: BoxDecoration(
                          color: scheme.surfaceContainerHighest,
                          borderRadius:
                              BorderRadius.circular(CdsDimens.radiusSmall),
                          border: Border.all(color: scheme.outline),
                        ),
                        child: CraftIcon(
                          CraftIcons.actionClear,
                          color: scheme.onSurfaceVariant,
                        ),
                      ),
                    ),
                  ),
                  const SizedBox(width: 10),
                  CdsPressable(
                    onTap: _run,
                    child: Tooltip(
                      message: l10n.runTooltip,
                      child: Container(
                        width: 44,
                        height: 44,
                        decoration: BoxDecoration(
                          color: scheme.primary,
                          borderRadius:
                              BorderRadius.circular(CdsDimens.radiusSmall),
                        ),
                        child: _running
                            ? Padding(
                                padding: const EdgeInsets.all(12),
                                child: CircularProgressIndicator(
                                  strokeWidth: 2,
                                  color: scheme.onPrimary,
                                ),
                              )
                            : CraftIcon(
                                CraftIcons.actionRun,
                                color: scheme.onPrimary,
                              ),
                      ),
                    ),
                  ),
                ],
              ),
            ),
          ),
        ),
        // ناحیه کد — جزیره LTR همیشگی
        Expanded(
          flex: 3,
          child: Container(
            width: double.infinity,
            color: scheme.surface,
            child: Directionality(
              textDirection: TextDirection.ltr,
              child: Padding(
                padding: const EdgeInsets.symmetric(
                  horizontal: CdsDimens.gutterPortrait,
                  vertical: 10,
                ),
                child: TextField(
                  controller: _code,
                  maxLines: null,
                  expands: true,
                  textAlignVertical: TextAlignVertical.top,
                  keyboardType: TextInputType.multiline,
                  style: CdsTheme.codeStyle(context),
                  cursorColor: scheme.primary,
                  decoration: InputDecoration(
                    hintText: l10n.editorHint,
                    hintStyle: CdsTheme.codeStyle(
                      context,
                      color: scheme.onSurfaceVariant,
                    ),
                    border: InputBorder.none,
                  ),
                ),
              ),
            ),
          ),
        ),
        // پنل خروجی — همان قرارداد بصری کنسول
        Expanded(
          flex: 2,
          child: Container(
            width: double.infinity,
            color: scheme.surfaceContainerLowest,
            child: Directionality(
              textDirection: TextDirection.ltr,
              child: _entries.isEmpty
                  ? Padding(
                      padding: const EdgeInsets.all(CdsDimens.gutterPortrait),
                      child: Text(
                        l10n.editorEmptyOutput,
                        style: CdsTheme.codeStyle(
                          context,
                          color: scheme.onSurfaceVariant,
                        ),
                      ),
                    )
                  // FB-2: انتخاب تم‌دار بخشی از خروجی + کپی همان بخش؛
                  // کپی کل خط با دکمه/long-press خط باقی می‌ماند.
                  : SelectionArea(
                      child: ListView.builder(
                        controller: _outputScroll,
                        padding: const EdgeInsets.symmetric(
                          horizontal: CdsDimens.gutterPortrait,
                          vertical: 10,
                        ),
                        itemCount: _entries.length,
                        itemBuilder: (context, i) {
                          final entry = _entries[i];
                          return switch (entry) {
                            TextEntry e => Text(
                                e.text,
                                style: CdsTheme.codeStyle(
                                  context,
                                  color: scheme.onSurface,
                                ),
                              ),
                            ErrEntry e => ErrorCard(
                                error: e.error,
                                onCopy: () =>
                                    _copyToClipboard(_copyTextOf(e)),
                              ),
                          };
                        },
                      ),
                    ),
            ),
          ),
        ),
      ],
    );
  }
}
