import 'package:flutter/material.dart';

/// توکن‌های رنگ CDS — مقادیر مطلق سند ۰۶ بخش ۲.۱ (تیره) و ۲.۲ (روشن).
///
/// هیچ ویجتی حق رنگ هاردکد ندارد؛ همه‌چیز از همین کلاس می‌آید
/// (anti-pattern #2 سند ۰۶ بخش ۱۲).
class CdsColors {
  CdsColors._();

  // ---- تم تیره «حالت مدرن» (پیش‌فرض — سند ۰۶ اصل ۴) ----
  static const darkBgBase = Color(0xFF101418);
  static const darkBgSurface = Color(0xFF171C22);
  static const darkBgElevated = Color(0xFF1F262E);
  static const darkBorderSubtle = Color(0xFF2A323C);
  static const darkTextPrimary = Color(0xFFE8EDF2);
  static const darkTextSecondary = Color(0xFF9AA7B4);
  static const darkTextMuted = Color(0xFF5C6975);
  static const darkAccentPrimary = Color(0xFFF5A524); // کهربایی «ورکشاپ»
  static const darkAccentOnPrimary = Color(0xFF1A1206);
  static const darkAccentSecondary = Color(0xFF22D3EE); // فیروزه‌ای
  static const darkSuccess = Color(0xFF34D399);
  static const darkWarn = Color(0xFFFBBF24);
  static const darkError = Color(0xFFF87171);
  static const darkCodeComment = Color(0xFF6B7A88);

  // ---- تم روشن «حالت آشنا» (کاغذ گرم؛ سند ۰۶ بخش ۲.۲) ----
  static const lightBgBase = Color(0xFFFAFAF7);
  static const lightBgSurface = Color(0xFFFFFFFF);
  static const lightBgElevated = Color(0xFFFFFFFF);
  static const lightBorderSubtle = Color(0xFFE4E7EB);
  static const lightTextPrimary = Color(0xFF1A2027);
  static const lightTextSecondary = Color(0xFF52606D);
  static const lightTextMuted = Color(0xFF7B8794);
  static const lightAccentPrimary = Color(0xFFB45309); // کهربایی تیره برای کنتراست
  static const lightAccentOnPrimary = Color(0xFFFFFFFF);
  static const lightAccentSecondary = Color(0xFF0E7490);
  static const lightSuccess = Color(0xFF047857);
  static const lightWarn = Color(0xFF92400E);
  static const lightError = Color(0xFFB91C1C);
  static const lightCodeComment = Color(0xFF64748B);

  /// پالت نمودار ۸ رنگ — ترتیب دقیق سند ۰۶ بخش ۳ (deuteranopia-safe).
  /// PlotSpec فقط ایندکس می‌فرستد؛ resolve اینجا در Dart انجام می‌شود.
  static const plotPalette = <Color>[
    Color(0xFF22D3EE),
    Color(0xFFF5A524),
    Color(0xFFA78BFA),
    Color(0xFF34D399),
    Color(0xFFF472B6),
    Color(0xFF60A5FA),
    Color(0xFFFBBF24),
    Color(0xFFF87171),
  ];
}

/// ابعاد CDS — سند ۰۶ بخش ۲.۴: شبکه 4dp، شعاع‌ها، گاترها.
class CdsDimens {
  CdsDimens._();

  static const radiusSmall = 8.0; // chip، دکمه
  static const radiusCard = 14.0; // کارت
  static const radiusSheet = 20.0; // شیت پایین
  static const gutterPortrait = 16.0;
  static const gutterLandscape = 24.0;
  static const touchTarget = 48.0; // حداقل هدف لمسی (سند ۰۶ اصل ۳)
  static const pressScale = 0.98; // حالت فشار همه عناصر تعاملی
  static const navRailWidth = 64.0; // لنداسکیپ (سند ۰۶ بخش ۵.۲)
  static const tabletBreakpoint = 840.0; // سند ۰۶ بخش ۵.۳
}

/// گروه انیمیشن‌های اجباری سند ۰۶ بخش ۸ — مدت‌ها مطلق‌اند.
class CdsMotion {
  CdsMotion._();

  static const consoleResultIn = Duration(milliseconds: 150); // easeOut
  static const press = Duration(milliseconds: 80);
  static const tabSwitch = Duration(milliseconds: 200); // emphasized
  static const themeLanguageFade = Duration(milliseconds: 250);
  static const runMorphGreen = Duration(milliseconds: 600);
}
