import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for Persian (`fa`).
class AppLocalizationsFa extends AppLocalizations {
  AppLocalizationsFa([String locale = 'fa']) : super(locale);

  @override
  String get appTitle => 'MathCraft';

  @override
  String get navConsole => 'کنسول';

  @override
  String get navEditor => 'ادیتور';

  @override
  String get navPlots => 'نمودار';

  @override
  String get navVars => 'متغیرها';

  @override
  String get navLearn => 'آموزش';

  @override
  String get themeTooltip => 'جابه‌جایی بین حالت مدرن و آشنا';

  @override
  String get themeModern => 'حالت مدرن';

  @override
  String get themeFamiliar => 'حالت آشنا';

  @override
  String get languageTooltip => 'تغییر زبان';

  @override
  String get consoleInputHint => 'یک عبارت بنویسید…';

  @override
  String get consoleEmptyTitle => 'اولین دستورت را امتحان کن';

  @override
  String get consoleEmptyHint => 'یک نمونه را لمس کن — واقعاً روی موتور Rust اجرا می‌شود';

  @override
  String get runTooltip => 'اجرا';

  @override
  String get stopTooltip => 'توقف';

  @override
  String get copyTooltip => 'کپی';

  @override
  String get copiedToClipboard => 'در حافظه کپی شد';

  @override
  String suggestionLabel(String name) {
    return 'منظورتان «$name» بود؟';
  }

  @override
  String errorTitle(String code) {
    return 'خطای $code';
  }

  @override
  String get emptyPlotsTitle => 'نمودار با مایل‌استون P3 می‌آید';

  @override
  String get emptyPlotsHint => 'ایده اولین نمودار: fxplot(sin, [0 6])';

  @override
  String get emptyVarsTitle => 'بازرس متغیرها با مایل‌استون P5 می‌آید';

  @override
  String get emptyVarsHint => 'تا آن موقع، موتور بعد از هر دستور تغییرات فضای کاری را گزارش می‌کند.';

  @override
  String get emptyLearnTitle => 'مستندات و درس‌ها با مایل‌استون P5 می‌آیند';

  @override
  String get emptyLearnHint => 'دوزبانه، آفلاین، با مثال‌های قابل اجرا.';

  @override
  String get editorHint => '% کد به سبک متلب — دکمهٔ اجرا کل بافر را اجرا می‌کند';

  @override
  String get editorOutputTitle => 'خروجی';

  @override
  String get editorEmptyOutput => 'خروجی اجرا اینجا می‌آید. فایل‌های تابع فوراً ثبت می‌شوند — در تب کنسول صدا بزنیدشان.';

  @override
  String get editorClearTooltip => 'پاک‌کردن ادیتور';
}
