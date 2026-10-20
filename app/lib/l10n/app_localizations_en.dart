import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for English (`en`).
class AppLocalizationsEn extends AppLocalizations {
  AppLocalizationsEn([String locale = 'en']) : super(locale);

  @override
  String get appTitle => 'MathCraft';

  @override
  String get navConsole => 'Console';

  @override
  String get navEditor => 'Editor';

  @override
  String get navPlots => 'Plots';

  @override
  String get navVars => 'Variables';

  @override
  String get navLearn => 'Learn';

  @override
  String get themeTooltip => 'Switch between Modern and Familiar';

  @override
  String get themeModern => 'Modern';

  @override
  String get themeFamiliar => 'Familiar';

  @override
  String get languageTooltip => 'Switch language';

  @override
  String get consoleInputHint => 'type an expression…';

  @override
  String get consoleEmptyTitle => 'Try your first command';

  @override
  String get consoleEmptyHint => 'Tap a sample — it really runs on the Rust engine';

  @override
  String get runTooltip => 'Run';

  @override
  String get copyTooltip => 'Copy';

  @override
  String get copiedToClipboard => 'Copied to clipboard';

  @override
  String suggestionLabel(String name) {
    return 'Did you mean “$name”?';
  }

  @override
  String errorTitle(String code) {
    return 'Error $code';
  }

  @override
  String get emptyPlotsTitle => 'Plots ship in milestone P3';

  @override
  String get emptyPlotsHint => 'First plot idea: fxplot(sin, [0 6])';

  @override
  String get emptyVarsTitle => 'Variable inspector ships in milestone P5';

  @override
  String get emptyVarsHint => 'Meanwhile the engine reports workspace changes after every command.';

  @override
  String get emptyLearnTitle => 'Docs and lessons ship in milestone P5';

  @override
  String get emptyLearnHint => 'Bilingual, offline, with runnable examples.';

  @override
  String get editorHint => '% MATLAB-style code — Run executes the whole buffer';

  @override
  String get editorOutputTitle => 'Output';

  @override
  String get editorEmptyOutput => 'Run output appears here. Function files register instantly — call them in the Console.';

  @override
  String get editorClearTooltip => 'Clear editor';
}
