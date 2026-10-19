import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../core/cds/cds_theme.dart';
import '../../core/cds/cds_tokens.dart';
import '../../core/icons/craft_icons.dart';
import '../../core/rust/engine_client.dart';
import '../../core/rust/engine_provider.dart';
import '../../l10n/app_localizations.dart';
import '../shell/app_shell.dart';

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
          default:
            break;
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
                      return switch (entry) {
                        InputEntry e => _InputLine(code: e.code),
                        OutputEntry e => _OutputLine(text: e.text),
                        ErrorEntry e => _ErrorCard(error: e.error),
                      };
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
class _OutputLine extends StatefulWidget {
  const _OutputLine({required this.text});

  final String text;

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
    return FadeTransition(
      opacity: _fade,
      child: SlideTransition(
        position: Tween<Offset>(
          begin: const Offset(0, -8 / 24), // 8dp نسبت به ارتفاع خط
          end: Offset.zero,
        ).animate(_fade),
        child: Padding(
          padding: const EdgeInsets.symmetric(vertical: 2),
          child: SizedBox(
            width: double.infinity,
            child: Text(
              widget.text,
              style: CdsTheme.codeStyle(context, color: scheme.onSurface),
            ),
          ),
        ),
      ),
    );
  }
}

/// کارت خطا — نوار قرمز ۳dp سمت چپ + کد + پیام زبان فعال + did-you-mean
/// (سند ۰۶ §7.1، قرارداد سند ۰۲ بخش ۹).
class _ErrorCard extends StatelessWidget {
  const _ErrorCard({required this.error});

  final McError error;

  @override
  Widget build(BuildContext context) {
    final scheme = Theme.of(context).colorScheme;
    final l10n = AppLocalizations.of(context)!;
    final isFa = Directionality.of(context) == TextDirection.rtl;
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
                    Text(
                      l10n.errorTitle(error.id),
                      style: CdsTheme.monoStyle(
                        color: scheme.error,
                        size: 11,
                      ),
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
    );
  }
}
