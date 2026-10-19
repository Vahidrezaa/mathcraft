import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../core/cds/cds_tokens.dart';
import '../../core/icons/craft_icons.dart';
import '../console/console_page.dart';
import '../empty/empty_tabs.dart';
import '../../state/app_settings.dart';
import '../../l10n/app_localizations.dart';

/// پوسته ناوبری پرتره — ۵ تب (سند ۰۶ بخش ۵.۱).
/// لنداسکیپ/تبلت (ریل کناری و سه‌پنل) با breakpoint در همین فایل توسعه می‌یابد
/// (قاعده سند ۰۲ بخش ۱۱ بند ۵: جداسازی Layout برای آمادگی دسکتاپ).
class AppShell extends ConsumerStatefulWidget {
  const AppShell({super.key});

  @override
  ConsumerState<AppShell> createState() => _AppShellState();
}

class _AppShellState extends ConsumerState<AppShell> {
  int _index = 0;
  int _previous = 0;

  static const _tabIcons = [
    CraftIcons.navConsole,
    CraftIcons.navEditor,
    CraftIcons.navPlots,
    CraftIcons.navVars,
    CraftIcons.navLearn,
  ];

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    final scheme = Theme.of(context).colorScheme;

    final pages = <Widget>[
      const ConsolePage(),
      EditorTab(),
      PlotsTab(),
      VarsTab(),
      LearnTab(),
    ];

    // جهت اسلاید تب‌سوییچ — هماهنگ با جهت متن (سند ۰۶ بخش ۸: ۲۰۰ms emphasized)
    final forward = _index >= _previous;
    final dx = (Directionality.of(context) == TextDirection.rtl
            ? -1.0
            : 1.0) *
        (forward ? 0.04 : -0.04);

    return Scaffold(
      appBar: AppBar(
        title: Text(l10n.appTitle),
        actions: [
          IconButton(
            tooltip: l10n.themeTooltip,
            onPressed: () {
              final notifier = ref.read(themeModeProvider.notifier);
              notifier.state = notifier.state == ThemeMode.dark
                  ? ThemeMode.light
                  : ThemeMode.dark;
            },
            icon: const CraftIcon(CraftIcons.actionTheme),
          ),
          IconButton(
            tooltip: l10n.languageTooltip,
            onPressed: () {
              final notifier = ref.read(localeProvider.notifier);
              notifier.state = notifier.state.languageCode == 'fa'
                  ? const Locale('en')
                  : const Locale('fa');
            },
            icon: const CraftIcon(CraftIcons.actionLang),
          ),
          const SizedBox(width: 4),
        ],
      ),
      body: AnimatedSwitcher(
        duration: CdsMotion.tabSwitch,
        switchInCurve: Curves.easeOutCubic,
        switchOutCurve: Curves.easeIn,
        transitionBuilder: (child, animation) {
          // فقط صفحه جدید اسلاید می‌خورد؛ صفحه قبلی cross-fade می‌شود.
          if (child.key == ValueKey(_index)) {
            return FadeTransition(
              opacity: animation,
              child: SlideTransition(
                position: Tween<Offset>(
                  begin: Offset(dx, 0),
                  end: Offset.zero,
                ).animate(animation),
                child: child,
              ),
            );
          }
          return FadeTransition(opacity: animation, child: child);
        },
        child: KeyedSubtree(
          key: ValueKey(_index),
          child: pages[_index],
        ),
      ),
      bottomNavigationBar: NavigationBar(
        selectedIndex: _index,
        onDestinationSelected: (i) {
          setState(() {
            _previous = _index;
            _index = i;
          });
        },
        destinations: [
          for (var i = 0; i < _tabIcons.length; i++)
            NavigationDestination(
              icon: CraftIcon(_tabIcons[i]),
              label: _tabLabel(i, l10n),
              selectedIcon: CraftIcon(
                _tabIcons[i],
                color: scheme.primary,
              ),
            ),
        ],
      ),
    );
  }

  String _tabLabel(int i, AppLocalizations l10n) => switch (i) {
        0 => l10n.navConsole,
        1 => l10n.navEditor,
        2 => l10n.navPlots,
        3 => l10n.navVars,
        _ => l10n.navLearn,
      };
}

/// حالت فشار همه عناصر تعاملی CDS: scale 0.98 در ۸۰ms (سند ۰۶ §2.4 —
/// «هر عنصر تعاملی، بدون استثنا؛ این حس زنده بودن را می‌سازد»).
class CdsPressable extends StatefulWidget {
  const CdsPressable({
    super.key,
    required this.onTap,
    required this.child,
  });

  final VoidCallback? onTap;
  final Widget child;

  @override
  State<CdsPressable> createState() => _CdsPressableState();
}

class _CdsPressableState extends State<CdsPressable> {
  bool _pressed = false;

  @override
  Widget build(BuildContext context) {
    return GestureDetector(
      onTapDown: widget.onTap == null
          ? null
          : (_) => setState(() => _pressed = true),
      onTapUp: (_) => setState(() => _pressed = false),
      onTapCancel: () => setState(() => _pressed = false),
      onTap: widget.onTap,
      child: AnimatedScale(
        scale: _pressed ? CdsDimens.pressScale : 1.0,
        duration: CdsMotion.press,
        curve: Curves.easeOut,
        child: widget.child,
      ),
    );
  }
}
