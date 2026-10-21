import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import 'cds_tokens.dart';

/// سازنده ThemeData از توکن‌های CDS — هر دو تم شهروند درجه‌یک
/// (سند ۰۶ اصل ۴: «تیره پیش‌فرض، روشن شهروند درجه یک»).
class CdsTheme {
  CdsTheme._();

  /// پالت نمودار — resolve ایندکس PlotSpec در Dart (سند ۰۴ §8.1).
  static Color plotColor(int paletteIndex) => CdsColors
      .plotPalette[paletteIndex.clamp(0, CdsColors.plotPalette.length - 1)];

  /// استایل کد/کنسول: JetBrains Mono با **ligature خاموش** (سند ۰۶ §2.3 —
  /// «صحت کاراکتر مهم‌تر از زیبایی ligature است»).
  static TextStyle codeStyle(BuildContext context,
      {Color? color, double? size}) {
    final scheme = Theme.of(context).colorScheme;
    return TextStyle(
      fontFamily: 'JetBrainsMono',
      fontSize: size ?? 14,
      height: 1.5,
      color: color ?? scheme.onSurface,
      fontFeatures: const [FontFeature('liga', 0), FontFeature('calt', 0)],
    );
  }

  /// نسخه مستقل از context برای theme-level استایل‌ها.
  static TextStyle monoStyle({required Color color, double size = 14}) {
    return TextStyle(
      fontFamily: 'JetBrainsMono',
      fontSize: size,
      height: 1.5,
      color: color,
      fontFeatures: const [FontFeature('liga', 0), FontFeature('calt', 0)],
    );
  }

  static ThemeData dark() => _build(
        brightness: Brightness.dark,
        bgBase: CdsColors.darkBgBase,
        bgSurface: CdsColors.darkBgSurface,
        bgElevated: CdsColors.darkBgElevated,
        border: CdsColors.darkBorderSubtle,
        textPrimary: CdsColors.darkTextPrimary,
        textSecondary: CdsColors.darkTextSecondary,
        textMuted: CdsColors.darkTextMuted,
        accent: CdsColors.darkAccentPrimary,
        onAccent: CdsColors.darkAccentOnPrimary,
        accentSecondary: CdsColors.darkAccentSecondary,
        error: CdsColors.darkError,
      );

  static ThemeData light() => _build(
        brightness: Brightness.light,
        bgBase: CdsColors.lightBgBase,
        bgSurface: CdsColors.lightBgSurface,
        bgElevated: CdsColors.lightBgElevated,
        border: CdsColors.lightBorderSubtle,
        textPrimary: CdsColors.lightTextPrimary,
        textSecondary: CdsColors.lightTextSecondary,
        textMuted: CdsColors.lightTextMuted,
        accent: CdsColors.lightAccentPrimary,
        onAccent: CdsColors.lightAccentOnPrimary,
        accentSecondary: CdsColors.lightAccentSecondary,
        error: CdsColors.lightError,
      );

  static ThemeData _build({
    required Brightness brightness,
    required Color bgBase,
    required Color bgSurface,
    required Color bgElevated,
    required Color border,
    required Color textPrimary,
    required Color textSecondary,
    required Color textMuted,
    required Color accent,
    required Color onAccent,
    required Color accentSecondary,
    required Color error,
  }) {
    final colorScheme = ColorScheme(
      brightness: brightness,
      primary: accent,
      onPrimary: onAccent,
      secondary: accentSecondary,
      onSecondary: bgBase,
      error: error,
      onError: bgBase,
      surface: bgSurface,
      onSurface: textPrimary,
      surfaceContainerHighest: bgElevated,
      onSurfaceVariant: textSecondary,
      outline: border,
      outlineVariant: border,
    );

    final base = ThemeData(
      useMaterial3: true,
      brightness: brightness,
      colorScheme: colorScheme,
      scaffoldBackgroundColor: bgBase,
    );

    // تایپوگرافی سند ۰۶ §2.3 — Vazirmatn برای UI، مونو فقط برای کد.
    final vazir = base.textTheme.apply(
      fontFamily: 'Vazirmatn',
      bodyColor: textPrimary,
      displayColor: textPrimary,
    );

    return base.copyWith(
      textTheme: vazir,
      // FB-2 (بازخورد دستگاه 2026-10-21): انتخاب متن با رنگ تم CDS —
      // هایلایت accent با شفافیت ۲۵٪ و دستگیره‌ها هم‌رنگ accent (سند ۰۶ §7.1)
      textSelectionTheme: TextSelectionThemeData(
        selectionColor: accent.withValues(alpha: 0.25),
        cursorColor: accent,
        selectionHandleColor: accent,
      ),
      appBarTheme: AppBarTheme(
        backgroundColor: bgBase,
        surfaceTintColor: Colors.transparent,
        elevation: 0,
        scrolledUnderElevation: 0,
        centerTitle: false,
        titleTextStyle: vazir.titleLarge?.copyWith(fontWeight: FontWeight.w700),
        systemOverlayStyle: brightness == Brightness.dark
            ? SystemUiOverlayStyle.light
            : SystemUiOverlayStyle.dark,
      ),
      navigationBarTheme: NavigationBarThemeData(
        backgroundColor: bgSurface,
        surfaceTintColor: Colors.transparent,
        indicatorColor: Colors.transparent,
        height: 64,
        labelBehavior: NavigationDestinationLabelBehavior.alwaysShow,
        iconTheme: WidgetStateProperty.resolveWith((states) {
          final selected = states.contains(WidgetState.selected);
          return IconThemeData(
            size: 24,
            color: selected ? accent : textMuted,
          );
        }),
        labelTextStyle: WidgetStateProperty.resolveWith((states) {
          final selected = states.contains(WidgetState.selected);
          return TextStyle(
            fontFamily: 'Vazirmatn',
            fontSize: 11,
            fontWeight: selected ? FontWeight.w700 : FontWeight.w400,
            color: selected ? accent : textMuted,
          );
        }),
      ),
      inputDecorationTheme: InputDecorationTheme(
        filled: true,
        fillColor: bgSurface,
        hintStyle: monoStyle(color: textMuted),
        border: OutlineInputBorder(
          borderRadius: BorderRadius.circular(CdsDimens.radiusSmall),
          borderSide: BorderSide(color: border),
        ),
        enabledBorder: OutlineInputBorder(
          borderRadius: BorderRadius.circular(CdsDimens.radiusSmall),
          borderSide: BorderSide(color: border),
        ),
        focusedBorder: OutlineInputBorder(
          borderRadius: BorderRadius.circular(CdsDimens.radiusSmall),
          borderSide: BorderSide(color: accent, width: 1.5),
        ),
        contentPadding: const EdgeInsets.symmetric(horizontal: 12, vertical: 12),
      ),
      dividerTheme: DividerThemeData(color: border, thickness: 1, space: 1),
      chipTheme: base.chipTheme.copyWith(
        backgroundColor: bgElevated,
        side: BorderSide(color: border),
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(CdsDimens.radiusSmall),
        ),
        labelStyle: TextStyle(fontSize: 13, color: textPrimary),
      ),
    );
  }
}
