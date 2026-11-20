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
  String get stopTooltip => 'Stop';

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
  String errorPosition(int line, int col) {
    return 'Line $line, column $col';
  }

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

  @override
  String get editorSaveTooltip => 'Save .m to the MathCraft folder';

  @override
  String get editorOpenTooltip => 'Open a .m script';

  @override
  String get editorSaveTitle => 'Save script';

  @override
  String get editorSaveNameHint => 'File name (script.m)';

  @override
  String get editorSaveAction => 'Save';

  @override
  String get editorCancelAction => 'Cancel';

  @override
  String get editorOpenTitle => 'Open script';

  @override
  String get editorNoScripts => 'No .m files in the MathCraft folder yet — save one first';

  @override
  String editorSavedTo(String path) {
    return 'Saved: $path';
  }

  @override
  String editorOpened(String name) {
    return 'Opened: $name';
  }

  @override
  String consoleFilesSaved(int count, String names) {
    return '$count file(s) saved to the MathCraft folder: $names';
  }

  @override
  String get plotsEmptyTitle => 'No plots yet';

  @override
  String get plotsEmptyHint => 'Run plot(...) in the Console — figures land here instantly.';

  @override
  String get plotsEngineError => 'Engine unavailable';

  @override
  String plotsFigureLabel(int n) {
    return 'Figure $n';
  }

  @override
  String get plotsListTooltip => 'Figure list';

  @override
  String get plotsPdfTooltip => 'Export vector PDF (Rust-rendered)';

  @override
  String get plotsSaveTooltip => 'Save PNG';

  @override
  String get plotsShareTooltip => 'Share PNG';

  @override
  String get plotsInfoTooltip => 'Figure info';

  @override
  String plotsSavedTo(String path) {
    return 'PNG saved: $path';
  }

  @override
  String plotsExportFailed(String error) {
    return 'Export failed: $error';
  }

  @override
  String get plotsInfoTitle => 'Figure info';

  @override
  String get plotsInfoFigureId => 'Figure id';

  @override
  String get plotsInfoKind => 'Type';

  @override
  String get plotsInfoSeries => 'Series';

  @override
  String get plotsInfoPoints => 'Drawn points';

  @override
  String get plotsInfoRawPoints => 'Raw points (kept in engine)';

  @override
  String get plotsCrosshairHint => 'Tap a point to pin its value · hold to inspect · drag to pan · pinch to zoom · double-tap to reset.';

  @override
  String get plotsNot3dTitle => 'Vector field (quiver) plots arrive in a future update';

  @override
  String get plotsNot3dHint => 'surf/mesh/contour/contourf/plot3 render since v3 (step 3.8) — quiver remains.';

  @override
  String get plotsSubplotNote => 'Subplot — cells are read-only in this release.';

  @override
  String get plotsKindLine => 'Line';

  @override
  String get plotsKindScatter => 'Scatter';

  @override
  String get plotsKindBar => 'Bar';

  @override
  String get plotsKindHist => 'Histogram';

  @override
  String get plotsKindArea => 'Area';

  @override
  String get plotsKindFill => 'Filled polygon';

  @override
  String get plotsKindPolar => 'Polar';

  @override
  String get plotsKindError => 'Error bar';

  @override
  String get plotsKindQuiver => 'Quiver';

  @override
  String get plotsKind3d => '3D';

  @override
  String get plotsKindSubplot => 'Subplot';
}
