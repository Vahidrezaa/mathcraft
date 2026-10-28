import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../core/cds/cds_theme.dart';
import '../../core/cds/cds_tokens.dart';
import '../../core/icons/craft_icons.dart';
import '../../core/rust/engine_client.dart';
import '../../core/rust/engine_provider.dart';
import '../../l10n/app_localizations.dart';
import '../shell/app_shell.dart';
import 'console_widgets.dart';

/// کنسول REPL — مشخصات سند ۰۶ بخش ۷.۱:
/// - prompt `>>` با رنگ muted؛ ورودی inline ادیت‌پذیر
/// - جزیره LTR همیشگی (سند ۰۶ بخش ۴ بند ۲) حتی در UI فارسی
/// - ورود نتیجه: fade + slide 8dp، ۱۵۰ms easeOut (جدول سند ۰۶ بخش ۸)
/// - خطا: کارت با نوار قرمز ۳dp سمت چپ، پیام زبان فعال + did-you-mean
class ConsolePage extends ConsumerStatefulWidget {
  const ConsolePage({super.key});

  @override
  ConsumerState<ConsolePage> createState() => _ConsolePageState();
}

/// یک سطر کنسول — مدل زبان‌خنثی؛ رندر بر اساس نوع.
sealed class ConsoleEntry {
  final String id;
  ConsoleEntry() : id = DateTime.now().microsecondsSinceEpoch.toString();
}

class InputEntry extends ConsoleEntry {
  InputEntry(this.code);
  final String code;
}

class OutputEntry extends ConsoleEntry {
  OutputEntry(this.text);
  final String text;
}

class ErrorEntry extends ConsoleEntry {
  ErrorEntry(this.error);
  final McError error;
}

class _ConsolePageState extends ConsumerState<ConsolePage> {
  final _entries = <ConsoleEntry>[];
  final _inputController = TextEditingController();
  final _scrollController = ScrollController();
  bool _running = false;
  // وضعیت پیشرفت زنده (قرارداد v2 — ROADMAP 2.3)؛ percent = null یعنی نامعین
  String? _stage;
  double? _progress;

  @override
  void dispose() {
    _inputController.dispose();
    _scrollController.dispose();
    super.dispose();
  }

  Future<void> _run(String code) async {
    final trimmed = code.trim();
    if (trimmed.isEmpty || _running) return;
    setState(() {
      _running = true;
      _stage = null;
      _progress = null;
      _entries.add(InputEntry(trimmed));
    });
    _inputController.clear();
    HapticFeedback.lightImpact(); // درج/اجرا — جدول انیمیشن سند ۰۶

    try {
      // اجرای زندهٔ قرارداد v2: Progress به نوار پیشرفت، نتیجهٔ نهایی از Done
      // (سند ۰۲ §3 — حس «زنده بودن» کنسول؛ ROADMAP 2.3 task 3 و 4)
      final result = await collectLiveRun(
        ref.read(engineProvider).runScript(trimmed),
        onProgress: (stage, percent) {
          if (mounted) {
            setState(() {
              _stage = stage;
              _progress = percent;
            });
          }
        },
        onCancelled: () {
          // UX فوری: نوار پیشرفت بسته شود؛ پیام نهایی از Done می‌آید
          if (mounted) {
            setState(() {
              _stage = null;
              _progress = null;
            });
          }
        },
      );
      if (result == null) return; // جریان بدون Done بسته شد — finally تمیز می‌کند
      final newEntries = <ConsoleEntry>[];
      for (final event in result.consoleEvents) {
        switch (event) {
          case ConsoleEvent_Print(:final text):
            if (text.isNotEmpty) newEntries.add(OutputEntry(text));
          case ConsoleEvent_Error(:final err):
            newEntries.add(ErrorEntry(err));
          case ConsoleEvent_Warning():
            break; // هشدارهای ساختاریافته در P2 (سند ۰۲ بخش ۳)
        }
      }
      if (result.status == JobStatus.done) {
        HapticFeedback.mediumImpact(); // موفقیت اجرا — سند ۰۶ بخش ۸
      }
      if (!mounted) return;
      setState(() => _entries.addAll(newEntries));
    } catch (e) {
      // موتور بوت نشده (مثلاً بدون بیلد Rust) — پیام واضح، نه کرش (سند ۰۲ §3).
      setState(
        () => _entries.add(
          OutputEntry('engine unavailable: $e'),
        ),
      );
    } finally {
      if (mounted) {
        setState(() {
          _running = false;
          _stage = null;
          _progress = null;
        });
      }
      _scrollToBottom();
    }
  }

  /// دکمهٔ Stop — لغو job فعال (CancellationToken سند ۰۲ §4؛ ROADMAP 2.3 task 4).
  /// پس از لغو، workspace در آخرین وضعیت commit می‌ماند (مثل stop متلب).
  Future<void> _stop() async {
    await ref.read(engineProvider).cancelActive();
  }

  void _scrollToBottom() {
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (!_scrollController.hasClients) return;
      _scrollController.animateTo(
        _scrollController.position.maxScrollExtent,
        duration: CdsMotion.consoleResultIn,
        curve: Curves.easeOut,
      );
    });
  }

  /// متن قابل کپی هر سطر — سمانتیک بازخورد P0.2:
  /// - ورودی: فقط کد، بدون پرامپت تزئینی «>>»
  /// - نتیجه: همان متن رندرشده
  /// - خطا: پیام خام موتور (متغیر messageEn، متن LTR کنسول) — نه عنوان/پیشنهاد
  String _copyTextOf(ConsoleEntry entry) => switch (entry) {
        InputEntry(:final code) => code,
        OutputEntry(:final text) => text,
        ErrorEntry(:final error) => error.messageEn,
      };

  /// کپی در حافظه + تأیید کوتاه SnackBar (بازخورد P0.2).
  /// متن کپی‌شده همیشه رشته LTR است و دست‌نخورده می‌ماند (RTL-safe)؛
  /// فقط اعلان UI با زبان فعال ساخته می‌شود.
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
    final isEmpty = _entries.isEmpty;

    return Column(
      children: [
        Expanded(
          // جزیره LTR: کنسول همیشه چپ‌چین (سند ۰۶ بخش ۴ بند ۲)
          child: Directionality(
            textDirection: TextDirection.ltr,
            child: isEmpty
                ? _EmptyState(onSample: _run)
                // FB-2: انتخاب تم‌دار بخشی از خروجی (SelectionArea) —
                // دستگیره‌ها/هایلایت از TextSelectionTheme تم CDS می‌آیند و
                // منوی سیستمی «کپی» همان بخش انتخابی را کپی می‌کند.
                // کپی کل خط با دکمهٔ همیشه‌نمایان باقی است (بازخورد P0.2).
                : SelectionArea(
                    child: ListView.builder(
                      controller: _scrollController,
                      padding: const EdgeInsets.symmetric(
                        horizontal: CdsDimens.gutterPortrait,
                        vertical: 12,
                      ),
                      itemCount: _entries.length,
                      itemBuilder: (context, i) {
                        final entry = _entries[i];
                        return switch (entry) {
                          InputEntry e => _InputLine(code: e.code),
                          OutputEntry e => _OutputLine(
                              text: e.text,
                              onCopy: () => _copyToClipboard(_copyTextOf(e)),
                            ),
                          ErrorEntry e => ErrorCard(
                              error: e.error,
                              onCopy: () => _copyToClipboard(_copyTextOf(e)),
                            ),
                        };
                      },
                    ),
                  ),
          ),
        ),
        // نوار ورودی — چسبیده به پایین (سند ۰۶ §5.1)
        SafeArea(
          top: false,
          child: Container(
            decoration: BoxDecoration(
              color: scheme.surface,
              border: Border(top: BorderSide(color: scheme.outline)),
            ),
            child: Directionality(
              textDirection: TextDirection.ltr,
              child: Padding(
                padding: const EdgeInsets.symmetric(
                  horizontal: CdsDimens.gutterPortrait,
                  vertical: 8,
                ),
                child: Column(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    // نوار پیشرفت زنده (قرارداد v2) — فقط حین اجرا؛
                    // percent = null → حالت نامعین (سند ۰۶ §5.1)
                    if (_running) ...[
                      if (_stage != null) ...[
                        Align(
                          alignment: Alignment.centerLeft,
                          child: Text(
                            _stage!,
                            style: CdsTheme.monoStyle(
                              color: scheme.onSurfaceVariant,
                            ),
                          ),
                        ),
                        const SizedBox(height: 4),
                      ],
                      LinearProgressIndicator(
                        value: _progress,
                        minHeight: 2,
                        backgroundColor: scheme.surfaceContainerHighest,
                        valueColor: AlwaysStoppedAnimation(scheme.primary),
                      ),
                      const SizedBox(height: 8),
                    ],
                    Row(
                      children: [
                        Text(
                          '>>',
                          style: CdsTheme.monoStyle(
                            color: scheme.onSurfaceVariant,
                          ),
                        ),
                        const SizedBox(width: 8),
                        Expanded(
                          child: TextField(
                            controller: _inputController,
                            style: CdsTheme.codeStyle(context),
                            textInputAction: TextInputAction.done,
                            onSubmitted: _run,
                            decoration: InputDecoration(
                              hintText: l10n.consoleInputHint,
                              isDense: true,
                            ),
                          ),
                        ),
                        const SizedBox(width: 8),
                        // دکمهٔ run/stop — حین اجرا Stop می‌شود (قرارداد v2:
                        // ROADMAP 2.3 task 4؛ آیکون‌های Craft و توکن‌های CDS)
                        CdsPressable(
                          onTap:
                              _running ? _stop : () => _run(_inputController.text),
                          child: Tooltip(
                            message:
                                _running ? l10n.stopTooltip : l10n.runTooltip,
                            child: Container(
                              width: 44,
                              height: 44,
                              decoration: BoxDecoration(
                                color:
                                    _running ? scheme.error : scheme.primary,
                                borderRadius: BorderRadius.circular(
                                  CdsDimens.radiusSmall,
                                ),
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
                  ],
                ),
              ),
            ),
          ),
        ),
      ],
    );
  }
}

/// حالت خالی کنسول — «خوش‌آمد عملی» سند ۰۶ بخش ۹: سه دستور نمونه لمس‌کردنی.
class _EmptyState extends StatelessWidget {
  const _EmptyState({required this.onSample});

  final ValueChanged<String> onSample;

  static const _samples = ['1+1', '2^3^2', '6/2*3'];

  @override
  Widget build(BuildContext context) {
    final scheme = Theme.of(context).colorScheme;
    final l10n = AppLocalizations.of(context)!;
    return Center(
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          Text(
            l10n.consoleEmptyTitle,
            style: Theme.of(context)
                .textTheme
                .titleMedium
                ?.copyWith(fontWeight: FontWeight.w700),
          ),
          const SizedBox(height: 6),
          Padding(
            padding: const EdgeInsets.symmetric(
              horizontal: CdsDimens.gutterPortrait * 2,
            ),
            child: Text(
              l10n.consoleEmptyHint,
              textAlign: TextAlign.center,
              style: Theme.of(context).textTheme.bodyMedium?.copyWith(
                    color: scheme.onSurfaceVariant,
                    height: 1.7,
                  ),
            ),
          ),
          const SizedBox(height: 20),
          Row(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              for (final s in _samples) ...[
                CdsPressable(
                  onTap: () => onSample(s),
                  child: Container(
                    padding: const EdgeInsets.symmetric(
                      horizontal: 16,
                      vertical: 10,
                    ),
                    decoration: BoxDecoration(
                      color: scheme.surfaceContainerHighest,
                      borderRadius:
                          BorderRadius.circular(CdsDimens.radiusSmall),
                      border: Border.all(color: scheme.outline),
                    ),
                    child: Text(
                      s,
                      style: CdsTheme.monoStyle(color: scheme.onSurface),
                    ),
                  ),
                ),
                const SizedBox(width: 10),
              ],
            ],
          ),
        ],
      ),
    );
  }
}

/// خط ورودی کاربر: `>> code` (سند ۰۶ §7.1)
class _InputLine extends StatelessWidget {
  const _InputLine({required this.code});

  final String code;

  @override
  Widget build(BuildContext context) {
    final scheme = Theme.of(context).colorScheme;
    return Padding(
      padding: const EdgeInsets.only(top: 10, bottom: 2),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            '>>',
            style: CdsTheme.monoStyle(color: scheme.onSurfaceVariant),
          ),
          const SizedBox(width: 8),
          Expanded(
            child: Text(
              code,
              style: CdsTheme.codeStyle(context),
            ),
          ),
        ],
      ),
    );
  }
}

/// خط خروجی — fade + slide 8dp از بالا، ۱۵۰ms easeOut (سند ۰۶ بخش ۸).
/// آیکون کپی محو ولی همیشه دیده‌شونده (بازخورد P0.2) — رنگ muted،
/// هدف لمسی ۴۰dp هماهنگ با دکمه run (۴۴dp، سند ۰۶ اصل ۳).
class _OutputLine extends StatefulWidget {
  const _OutputLine({required this.text, required this.onCopy});

  final String text;
  final VoidCallback onCopy;

  @override
  State<_OutputLine> createState() => _OutputLineState();
}

class _OutputLineState extends State<_OutputLine>
    with SingleTickerProviderStateMixin {
  late final AnimationController _controller;
  late final Animation<double> _fade;

  @override
  void initState() {
    super.initState();
    _controller = AnimationController(
      vsync: this,
      duration: CdsMotion.consoleResultIn,
    );
    _fade = CurvedAnimation(parent: _controller, curve: Curves.easeOut);
    _controller.forward();
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final scheme = Theme.of(context).colorScheme;
    final l10n = AppLocalizations.of(context)!;
    return FadeTransition(
      opacity: _fade,
      child: SlideTransition(
        position: Tween<Offset>(
          begin: const Offset(0, -8 / 24), // 8dp نسبت به ارتفاع خط
          end: Offset.zero,
        ).animate(_fade),
        child: Padding(
          padding: const EdgeInsets.symmetric(vertical: 2),
          child: Row(
            crossAxisAlignment: CrossAxisAlignment.center,
            children: [
              Expanded(
                child: SizedBox(
                  width: double.infinity,
                  child: Text(
                    widget.text,
                    style: CdsTheme.codeStyle(context, color: scheme.onSurface),
                  ),
                ),
              ),
              CopyButton(onCopy: widget.onCopy, tooltip: l10n.copyTooltip),
            ],
          ),
        ),
      ),
    );
  }
}

/// کارت خطا — ویجت مشترک ErrorCard از console_widgets.dart
/// (نوار قرمز ۳dp + پیام زبان فعال + did-you-mean + کپی پیام خام).
