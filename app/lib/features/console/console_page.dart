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
      _entries.add(InputEntry(trimmed));
    });
    _inputController.clear();
    HapticFeedback.lightImpact(); // درج/اجرا — جدول انیمیشن سند ۰۶

    try {
      final result =
          await ref.read(engineProvider).evaluateLine(trimmed);
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
      if (mounted) setState(() => _running = false);
      _scrollToBottom();
    }
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
                : ListView.builder(
                    controller: _scrollController,
                    padding: const EdgeInsets.symmetric(
                      horizontal: CdsDimens.gutterPortrait,
                      vertical: 12,
                    ),
                    itemCount: _entries.length,
                    itemBuilder: (context, i) {
                      final entry = _entries[i];
                      // کپی هر سطر با long-press (بازخورد P0.2) — متن همیشه
                      // همان رشته LTR کنسول است، بدون عناصر تزئینی.
                      return GestureDetector(
                        onLongPress: () =>
                            _copyToClipboard(_copyTextOf(entry)),
                        child: switch (entry) {
                          InputEntry e => _InputLine(code: e.code),
                          OutputEntry e => _OutputLine(
                              text: e.text,
                              onCopy: () => _copyToClipboard(_copyTextOf(e)),
                            ),
                          ErrorEntry e => ErrorCard(
                              error: e.error,
                              onCopy: () => _copyToClipboard(_copyTextOf(e)),
                            ),
                        },
                      );
                    },
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
                child: Row(
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
                    // دکمه run — accent، بزرگ‌تر (سند ۰۶ §5.1/§6)
                    CdsPressable(
                      onTap: () => _run(_inputController.text),
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
