import 'package:flutter/material.dart';
import 'package:flutter_svg/flutter_svg.dart';

/// Craft Icons — مجموعه آیکون اختصاصی CDS (سند ۰۶ بخش ۲.۵).
///
/// قانون مطلق: **ایموجی هرگز جایگزین آیکون نمی‌شود** و آیکون فقط از همین
/// مجموعه می‌آید. ترکیب با Material Icons پیش‌فرض ممنوع (anti-pattern ۸).
class CraftIconData {
  const CraftIconData(this.name, {this.mirrorsInRtl = false});

  /// نام فایل SVG در `assets/icons/` (`ic_<domain>_<name>.svg`).
  final String name;

  /// آیکون‌های جهت‌دار در فارسی mirror می‌شوند؛ آیکون‌های جهانی هرگز (سند ۰۶ §2.5).
  final bool mirrorsInRtl;
}

class CraftIcons {
  CraftIcons._();

  // ناوبری (۵) — سند ۰۶ بخش ۵.۱
  static const navConsole = CraftIconData('ic_nav_console');
  static const navEditor = CraftIconData('ic_nav_editor');
  static const navPlots = CraftIconData('ic_nav_plots');
  static const navVars = CraftIconData('ic_nav_vars');
  static const navLearn = CraftIconData('ic_nav_learn');

  // اکشن‌های اصلی P0
  static const actionRun = CraftIconData('ic_action_run');
  static const actionStop = CraftIconData('ic_action_stop');
  static const actionSettings = CraftIconData('ic_action_settings');
  static const actionTheme = CraftIconData('ic_action_theme');
  static const actionLang = CraftIconData('ic_action_lang');
  static const actionClear = CraftIconData('ic_action_clear');
  static const actionHistory = CraftIconData('ic_action_history');
}

/// ویجت آیکون CDS — SVG با `currentColor` (رنگ فقط از توکن‌ها می‌آید؛
/// رنگ هاردکد داخل فایل SVG ممنوع و با lint چک می‌شود — سند ۰۶ §2.5).
class CraftIcon extends StatelessWidget {
  const CraftIcon(
    this.icon, {
    super.key,
    this.size = 24,
    this.color,
  });

  final CraftIconData icon;
  final double size;
  final Color? color;

  @override
  Widget build(BuildContext context) {
    final effectiveColor = color ?? Theme.of(context).iconTheme.color;
    final rtl = Directionality.of(context) == TextDirection.rtl;

    Widget svg = SvgPicture.asset(
      'assets/icons/${icon.name}.svg',
      width: size,
      height: size,
      colorFilter: effectiveColor == null
          ? null
          : ColorFilter.mode(effectiveColor, BlendMode.srcIn),
    );

    if (icon.mirrorsInRtl && rtl) {
      svg = Transform.flip(flipX: true, child: svg);
    }
    return svg;
  }
}
