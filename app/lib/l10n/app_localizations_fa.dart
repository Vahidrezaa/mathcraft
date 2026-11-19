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
  String errorPosition(int line, int col) {
    return 'سطر $line، ستون $col';
  }

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

  @override
  String get editorSaveTooltip => 'ذخیرهٔ .m در پوشهٔ MathCraft';

  @override
  String get editorOpenTooltip => 'بازکردن اسکریپت .m';

  @override
  String get editorSaveTitle => 'ذخیرهٔ اسکریپت';

  @override
  String get editorSaveNameHint => 'نام فایل (script.m)';

  @override
  String get editorSaveAction => 'ذخیره';

  @override
  String get editorCancelAction => 'انصراف';

  @override
  String get editorOpenTitle => 'بازکردن اسکریپت';

  @override
  String get editorNoScripts => 'هنوز فایل .m‌ای در پوشهٔ MathCraft نیست — اول یکی ذخیره کنید';

  @override
  String editorSavedTo(String path) {
    return 'ذخیره شد: $path';
  }

  @override
  String editorOpened(String name) {
    return 'باز شد: $name';
  }

  @override
  String consoleFilesSaved(int count, String names) {
    return '$count فایل در پوشهٔ MathCraft ذخیره شد: $names';
  }

  @override
  String get plotsEmptyTitle => 'هنوز نموداری نیست';

  @override
  String get plotsEmptyHint => 'در کنسول plot(...) اجرا کنید — شکل‌ها همین‌جا می‌آیند.';

  @override
  String get plotsEngineError => 'موتور در دسترس نیست';

  @override
  String plotsFigureLabel(int n) {
    return 'شکل $n';
  }

  @override
  String get plotsListTooltip => 'فهرست شکل‌ها';

  @override
  String get plotsSaveTooltip => 'ذخیرهٔ PNG';

  @override
  String get plotsShareTooltip => 'اشتراک PNG';

  @override
  String get plotsInfoTooltip => 'اطلاعات شکل';

  @override
  String plotsSavedTo(String path) {
    return 'PNG ذخیره شد: $path';
  }

  @override
  String plotsExportFailed(String error) {
    return 'خروجی ناموفق: $error';
  }

  @override
  String get plotsInfoTitle => 'اطلاعات شکل';

  @override
  String get plotsInfoFigureId => 'شناسهٔ شکل';

  @override
  String get plotsInfoKind => 'نوع';

  @override
  String get plotsInfoSeries => 'سری‌ها';

  @override
  String get plotsInfoPoints => 'نقاط رسم‌شده';

  @override
  String get plotsInfoRawPoints => 'نقاط خام (نزد موتور)';

  @override
  String get plotsCrosshairHint => 'تپ روی نقطه = پایش مقدار · نگه‌داشتن = crosshair · کشیدن = جابه‌جایی · دو انگشت = زوم · دو تپ = بازنشانی.';

  @override
  String get plotsNot3dTitle => 'نمودارهای سه‌بعدی و برداری با به‌روزرسانی بعدی می‌آیند';

  @override
  String get plotsNot3dHint => 'surf/mesh/contour/plot3/quiver از حالا در قرارداد آماده‌اند (P3-b).';

  @override
  String get plotsSubplotNote => 'Subplot — سلول‌ها در این نسخه فقط‌نمایش‌اند.';

  @override
  String get plotsKindLine => 'خط';

  @override
  String get plotsKindScatter => 'پراکنده';

  @override
  String get plotsKindBar => 'میله‌ای';

  @override
  String get plotsKindHist => 'هیستوگرام';

  @override
  String get plotsKindArea => 'سطح';

  @override
  String get plotsKindFill => 'چندضلعی';

  @override
  String get plotsKindPolar => 'قطبی';

  @override
  String get plotsKindError => 'نوار خطا';

  @override
  String get plotsKindQuiver => 'برداری';

  @override
  String get plotsKind3d => 'سه‌بعدی';

  @override
  String get plotsKindSubplot => 'چندقابی';
}
