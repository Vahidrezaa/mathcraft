import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_localizations/flutter_localizations.dart';
import 'package:intl/intl.dart' as intl;

import 'app_localizations_en.dart';
import 'app_localizations_fa.dart';

// ignore_for_file: type=lint

/// Callers can lookup localized strings with an instance of AppLocalizations
/// returned by `AppLocalizations.of(context)`.
///
/// Applications need to include `AppLocalizations.delegate()` in their app's
/// `localizationDelegates` list, and the locales they support in the app's
/// `supportedLocales` list. For example:
///
/// ```dart
/// import 'l10n/app_localizations.dart';
///
/// return MaterialApp(
///   localizationsDelegates: AppLocalizations.localizationsDelegates,
///   supportedLocales: AppLocalizations.supportedLocales,
///   home: MyApplicationHome(),
/// );
/// ```
///
/// ## Update pubspec.yaml
///
/// Please make sure to update your pubspec.yaml to include the following
/// packages:
///
/// ```yaml
/// dependencies:
///   # Internationalization support.
///   flutter_localizations:
///     sdk: flutter
///   intl: any # Use the pinned version from flutter_localizations
///
///   # Rest of dependencies
/// ```
///
/// ## iOS Applications
///
/// iOS applications define key application metadata, including supported
/// locales, in an Info.plist file that is built into the application bundle.
/// To configure the locales supported by your app, you’ll need to edit this
/// file.
///
/// First, open your project’s ios/Runner.xcworkspace Xcode workspace file.
/// Then, in the Project Navigator, open the Info.plist file under the Runner
/// project’s Runner folder.
///
/// Next, select the Information Property List item, select Add Item from the
/// Editor menu, then select Localizations from the pop-up menu.
///
/// Select and expand the newly-created Localizations item then, for each
/// locale your application supports, add a new item and select the locale
/// you wish to add from the pop-up menu in the Value field. This list should
/// be consistent with the languages listed in the AppLocalizations.supportedLocales
/// property.
abstract class AppLocalizations {
  AppLocalizations(String locale) : localeName = intl.Intl.canonicalizedLocale(locale.toString());

  final String localeName;

  static AppLocalizations? of(BuildContext context) {
    return Localizations.of<AppLocalizations>(context, AppLocalizations);
  }

  static const LocalizationsDelegate<AppLocalizations> delegate = _AppLocalizationsDelegate();

  /// A list of this localizations delegate along with the default localizations
  /// delegates.
  ///
  /// Returns a list of localizations delegates containing this delegate along with
  /// GlobalMaterialLocalizations.delegate, GlobalCupertinoLocalizations.delegate,
  /// and GlobalWidgetsLocalizations.delegate.
  ///
  /// Additional delegates can be added by appending to this list in
  /// MaterialApp. This list does not have to be used at all if a custom list
  /// of delegates is preferred or required.
  static const List<LocalizationsDelegate<dynamic>> localizationsDelegates = <LocalizationsDelegate<dynamic>>[
    delegate,
    GlobalMaterialLocalizations.delegate,
    GlobalCupertinoLocalizations.delegate,
    GlobalWidgetsLocalizations.delegate,
  ];

  /// A list of this localizations delegate's supported locales.
  static const List<Locale> supportedLocales = <Locale>[
    Locale('en'),
    Locale('fa')
  ];

  /// No description provided for @appTitle.
  ///
  /// In en, this message translates to:
  /// **'MathCraft'**
  String get appTitle;

  /// No description provided for @navConsole.
  ///
  /// In en, this message translates to:
  /// **'Console'**
  String get navConsole;

  /// No description provided for @navEditor.
  ///
  /// In en, this message translates to:
  /// **'Editor'**
  String get navEditor;

  /// No description provided for @navPlots.
  ///
  /// In en, this message translates to:
  /// **'Plots'**
  String get navPlots;

  /// No description provided for @navVars.
  ///
  /// In en, this message translates to:
  /// **'Variables'**
  String get navVars;

  /// No description provided for @navLearn.
  ///
  /// In en, this message translates to:
  /// **'Learn'**
  String get navLearn;

  /// No description provided for @themeTooltip.
  ///
  /// In en, this message translates to:
  /// **'Switch between Modern and Familiar'**
  String get themeTooltip;

  /// No description provided for @themeModern.
  ///
  /// In en, this message translates to:
  /// **'Modern'**
  String get themeModern;

  /// No description provided for @themeFamiliar.
  ///
  /// In en, this message translates to:
  /// **'Familiar'**
  String get themeFamiliar;

  /// No description provided for @languageTooltip.
  ///
  /// In en, this message translates to:
  /// **'Switch language'**
  String get languageTooltip;

  /// No description provided for @consoleInputHint.
  ///
  /// In en, this message translates to:
  /// **'type an expression…'**
  String get consoleInputHint;

  /// No description provided for @consoleEmptyTitle.
  ///
  /// In en, this message translates to:
  /// **'Try your first command'**
  String get consoleEmptyTitle;

  /// No description provided for @consoleEmptyHint.
  ///
  /// In en, this message translates to:
  /// **'Tap a sample — it really runs on the Rust engine'**
  String get consoleEmptyHint;

  /// No description provided for @runTooltip.
  ///
  /// In en, this message translates to:
  /// **'Run'**
  String get runTooltip;

  /// Tooltip of the copy action shown on console result blocks and error cards
  ///
  /// In en, this message translates to:
  /// **'Copy'**
  String get copyTooltip;

  /// Brief confirmation SnackBar shown after a console entry is copied
  ///
  /// In en, this message translates to:
  /// **'Copied to clipboard'**
  String get copiedToClipboard;

  /// No description provided for @suggestionLabel.
  ///
  /// In en, this message translates to:
  /// **'Did you mean “{name}”?'**
  String suggestionLabel(String name);

  /// No description provided for @errorTitle.
  ///
  /// In en, this message translates to:
  /// **'Error {code}'**
  String errorTitle(String code);

  /// No description provided for @emptyPlotsTitle.
  ///
  /// In en, this message translates to:
  /// **'Plots ship in milestone P3'**
  String get emptyPlotsTitle;

  /// No description provided for @emptyPlotsHint.
  ///
  /// In en, this message translates to:
  /// **'First plot idea: fxplot(sin, [0 6])'**
  String get emptyPlotsHint;

  /// No description provided for @emptyVarsTitle.
  ///
  /// In en, this message translates to:
  /// **'Variable inspector ships in milestone P5'**
  String get emptyVarsTitle;

  /// No description provided for @emptyVarsHint.
  ///
  /// In en, this message translates to:
  /// **'Meanwhile the engine reports workspace changes after every command.'**
  String get emptyVarsHint;

  /// No description provided for @emptyLearnTitle.
  ///
  /// In en, this message translates to:
  /// **'Docs and lessons ship in milestone P5'**
  String get emptyLearnTitle;

  /// No description provided for @emptyLearnHint.
  ///
  /// In en, this message translates to:
  /// **'Bilingual, offline, with runnable examples.'**
  String get emptyLearnHint;

  /// No description provided for @editorHint.
  ///
  /// In en, this message translates to:
  /// **'% MATLAB-style code — Run executes the whole buffer'**
  String get editorHint;

  /// No description provided for @editorOutputTitle.
  ///
  /// In en, this message translates to:
  /// **'Output'**
  String get editorOutputTitle;

  /// No description provided for @editorEmptyOutput.
  ///
  /// In en, this message translates to:
  /// **'Run output appears here. Function files register instantly — call them in the Console.'**
  String get editorEmptyOutput;

  /// No description provided for @editorClearTooltip.
  ///
  /// In en, this message translates to:
  /// **'Clear editor'**
  String get editorClearTooltip;
}

class _AppLocalizationsDelegate extends LocalizationsDelegate<AppLocalizations> {
  const _AppLocalizationsDelegate();

  @override
  Future<AppLocalizations> load(Locale locale) {
    return SynchronousFuture<AppLocalizations>(lookupAppLocalizations(locale));
  }

  @override
  bool isSupported(Locale locale) => <String>['en', 'fa'].contains(locale.languageCode);

  @override
  bool shouldReload(_AppLocalizationsDelegate old) => false;
}

AppLocalizations lookupAppLocalizations(Locale locale) {


  // Lookup logic when only language code is specified.
  switch (locale.languageCode) {
    case 'en': return AppLocalizationsEn();
    case 'fa': return AppLocalizationsFa();
  }

  throw FlutterError(
    'AppLocalizations.delegate failed to load unsupported locale "$locale". This is likely '
    'an issue with the localizations generation tool. Please file an issue '
    'on GitHub with a reproducible sample app and the gen-l10n configuration '
    'that was used.'
  );
}
