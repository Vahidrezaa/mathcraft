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
import '../../core/io/mathcraft_dir.dart';
import 'dart:convert' show utf8;

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
  String? _fileName; // نام فایل ذخیره‌شدهٔ اخیر (پیش‌فرض دیالوگ ذخیره)
  final _outputScroll = ScrollController();
  final _entries = <EditorEntry>[];
  bool _running = false;
  // وضعیت پیشرفت زنده (قرارداد v2 — ROADMAP 2.3)
  String? _stage;
  double? _progress;

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

  /// ذخیرهٔ بافر در پوشهٔ MathCraft (.m) + ثبت در VFS موتور — بعداً
  /// `run('name.m')` از کنسول همان را می‌یابد (گام 3.7، D-39).
  Future<void> _save() async {
    final l10n = AppLocalizations.of(context)!;
    final controller = TextEditingController(
      text: _fileName ?? 'script.m',
    );
    final name = await showDialog<String>(
      context: context,
      builder: (ctx) => AlertDialog(
        title: Text(l10n.editorSaveTitle),
        content: TextField(
          controller: controller,
          autofocus: true,
          decoration: InputDecoration(hintText: l10n.editorSaveNameHint),
          onSubmitted: (v) => Navigator.of(ctx).pop(v),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(ctx).pop(),
            child: Text(l10n.editorCancelAction),
          ),
          FilledButton(
            onPressed: () => Navigator.of(ctx).pop(controller.text),
            child: Text(l10n.editorSaveAction),
          ),
        ],
      ),
    );
    if (name == null || name.trim().isEmpty) return;
    final path = await MathCraftDir.writeScript(name, _code.text);
    final base = path.split('/').last;
    setState(() => _fileName = base);
    // ثبت در VFS موتور — دقیقاً همان بایت‌های UTF-8 که روی دیسک رفت
    try {
      await ref.read(engineProvider).vfsRegister(
            base,
            Uint8List.fromList(utf8.encode(_code.text)),
          );
    } catch (_) {
      // موتور در دسترس نیست — فایل ذخیره شد؛ ثبت بعدی خودکار می‌آید (D-39)
    }
    if (!mounted) return;
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(content: Text(l10n.editorSavedTo(path))),
    );
  }

  /// بازکردن یک اسکریپت .m از پوشهٔ MathCraft در بافر.
  Future<void> _open() async {
    final l10n = AppLocalizations.of(context)!;
    final scripts = await MathCraftDir.listScripts();
    if (!mounted) return;
    if (scripts.isEmpty) {
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text(l10n.editorNoScripts)),
      );
      return;
    }
    final name = await showDialog<String>(
      context: context,
      builder: (ctx) => SimpleDialog(
        title: Text(l10n.editorOpenTitle),
        children: [
          for (final s in scripts)
            SimpleDialogOption(
              onPressed: () => Navigator.of(ctx).pop(s),
              child: Text(
                s,
                style: const TextStyle(fontFamily: 'JetBrainsMono'),
              ),
            ),
        ],
      ),
    );
    if (name == null) return;
    final text = await MathCraftDir.readScript(name);
    if (!mounted) return;
    _code.text = text;
    setState(() => _fileName = name);
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(content: Text(l10n.editorOpened(name))),
    );
  }

  Future<void> _run() async {
    final code = _code.text.trim();
    if (code.isEmpty || _running) return;
    setState(() {
      _running = true;
      _stage = null;
      _progress = null;
      _entries
        ..clear()
        ..add(TextEntry('>> run'));
    });
    HapticFeedback.lightImpact();

    try {
      // اجرای زندهٔ قرارداد v2 — همان مسیر کنسول (ROADMAP 2.3 task 3/4)
      final result = await collectLiveRun(
        ref.read(engineProvider).runScript(code),
        onProgress: (stage, percent) {
          if (mounted) {
            setState(() {
              _stage = stage;
              _progress = percent;
            });
          }
        },
        onCancelled: () {
          if (mounted) {
            setState(() {
              _stage = null;
              _progress = null;
            });
          }
        },
      );
      if (result == null) return; // جریان بدون Done — finally تمیز می‌کند
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
      if (mounted) {
        setState(() {
          _running = false;
          _stage = null;
          _progress = null;
        });
      }
    }
  }

  /// دکمهٔ Stop پنل خروجی — لغو job فعال (سند ۰۲ §4؛ ROADMAP 2.3 task 4).
  Future<void> _stop() async {
    await ref.read(engineProvider).cancelActive();
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
                    onTap: _save,
                    child: Tooltip(
                      message: l10n.editorSaveTooltip,
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
                          CraftIcons.actionSave,
                          color: scheme.onSurfaceVariant,
                        ),
                      ),
                    ),
                  ),
                  const SizedBox(width: 10),
                  CdsPressable(
                    onTap: _open,
                    child: Tooltip(
                      message: l10n.editorOpenTooltip,
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
                          CraftIcons.actionHistory,
                          color: scheme.onSurfaceVariant,
                        ),
                      ),
                    ),
                  ),
                  const SizedBox(width: 10),
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
                    onTap: _running ? _stop : _run,
                    child: Tooltip(
                      message: _running ? l10n.stopTooltip : l10n.runTooltip,
                      child: Container(
                        width: 44,
                        height: 44,
                        decoration: BoxDecoration(
                          color: _running ? scheme.error : scheme.primary,
                          borderRadius:
                              BorderRadius.circular(CdsDimens.radiusSmall),
                        ),
                        child: _running
                            ? CraftIcon(
                                CraftIcons.actionStop,
                                color: scheme.onError,
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
        // نوار پیشرفت زنده (قرارداد v2) — فقط حین اجرا؛ مرحلهٔ فعال هم نمایش
        if (_running) ...[
          if (_stage != null)
            Padding(
              padding: const EdgeInsets.symmetric(
                horizontal: CdsDimens.gutterPortrait,
                vertical: 4,
              ),
              child: Align(
                alignment: Alignment.centerLeft,
                child: Text(
                  _stage!,
                  style: CdsTheme.monoStyle(color: scheme.onSurfaceVariant),
                ),
              ),
            ),
          LinearProgressIndicator(
            value: _progress,
            minHeight: 2,
            backgroundColor: scheme.surfaceContainerHighest,
            valueColor: AlwaysStoppedAnimation(scheme.primary),
          ),
        ],
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
