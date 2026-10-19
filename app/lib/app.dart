import 'package:flutter/material.dart';
import 'package:flutter_localizations/flutter_localizations.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'core/cds/cds_theme.dart';
import 'core/cds/cds_tokens.dart';
import 'features/shell/app_shell.dart';
import 'l10n/app_localizations.dart';
import 'state/app_settings.dart';

/// ریشه MathCraft — سوییچ آنی تم/زبان (DoD سند ۰۷ P0):
/// تم پیش‌فرض تیره «حالت مدرن»، تم روشن «حالت آشنا» با همان کیفیت (سند ۰۶ §1.5)؛
/// تغییر تم با cross-fade سراسری ۲۵۰ms (جدول سند ۰۶ بخش ۸).
class MathCraftApp extends ConsumerWidget {
  const MathCraftApp({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final themeMode = ref.watch(themeModeProvider);
    final locale = ref.watch(localeProvider);

    return MaterialApp(
      onGenerateTitle: (ctx) => AppLocalizations.of(ctx)!.appTitle,
      localizationsDelegates: const [
        AppLocalizations.delegate,
        GlobalMaterialLocalizations.delegate,
        GlobalWidgetsLocalizations.delegate,
        GlobalCupertinoLocalizations.delegate,
      ],
      supportedLocales: const [Locale('fa'), Locale('en')],
      locale: locale,
      theme: CdsTheme.light(),
      darkTheme: CdsTheme.dark(),
      themeMode: themeMode,
      themeAnimationDuration: CdsMotion.themeLanguageFade,
      debugShowCheckedModeBanner: false,
      home: const AppShell(),
    );
  }
}
