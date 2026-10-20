import 'package:flutter/material.dart';
import 'package:flutter_localizations/flutter_localizations.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'core/cds/cds_theme.dart';
import 'core/cds/cds_tokens.dart';
import 'features/shell/app_shell.dart';
import 'l10n/app_localizations.dart';
import 'state/app_settings.dart';

/// تم‌های کامل فقط یک‌بار ساخته و کش می‌شوند (بازخورد P0.2 — لگ سوییچ تم):
/// ساخت ThemeData در هر rebuild پرهزینه بود؛ اکنون instanceهای top-level final
/// فقط در اولین دسترسی ساخته می‌شوند و در سوییچ تم/زبان دوباره استفاده می‌شوند.
/// خود انیمیشن، cross-fade سراسری ۲۵۰ms است (جدول سند ۰۶ بخش ۸) — نه ساخت مجدد.
final ThemeData _lightTheme = CdsTheme.light();
final ThemeData _darkTheme = CdsTheme.dark();

/// ریشه MathCraft — سوییچ آنی تم/زبان (DoD سند ۰۷ P0):
/// تم پیش‌فرض تیره «حالت مدرن»، تم روشن «حالت آشنا» با همان کیفیت (سند ۰۶ §1.5).
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
      theme: _lightTheme,
      darkTheme: _darkTheme,
      themeMode: themeMode,
      themeAnimationDuration: CdsMotion.themeLanguageFade,
      debugShowCheckedModeBanner: false,
      home: const AppShell(),
    );
  }
}
