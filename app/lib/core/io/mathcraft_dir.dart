import 'dart:io';

import 'package:flutter/foundation.dart';
import 'package:path_provider/path_provider.dart';


/// درز IO پوشهٔ MathCraft برای تست‌های UI (سند ۰۸ §۶): وقتی ست شده باشد،
/// همهٔ عملیات از همین‌جا می‌گذرد — بدون dart:io واقعی، بدون runAsync.
abstract class MathCraftIo {
  Future<String> writeScript(String name, String text);
  Future<List<String>> listScripts();
  Future<String> readScript(String name);
  Future<String> writeOutput(String name, Uint8List bytes);
  Future<int> registerAll(
    Future<void> Function(String name, Uint8List bytes) register,
  );
}

/// پیاده‌سازی واقعی (پوشهٔ اسناد اپ).
class _RealMathCraftIo implements MathCraftIo {
  const _RealMathCraftIo();

  @override
  Future<String> writeScript(String name, String text) =>
      MathCraftDir._realWriteScript(name, text);

  @override
  Future<List<String>> listScripts() => MathCraftDir._realListScripts();

  @override
  Future<String> readScript(String name) => MathCraftDir._realReadScript(name);

  @override
  Future<String> writeOutput(String name, Uint8List bytes) =>
      MathCraftDir._realWriteOutput(name, bytes);

  @override
  Future<int> registerAll(
    Future<void> Function(String name, Uint8List bytes) register,
  ) =>
      MathCraftDir._realRegisterAll(register);
}

/// پوشهٔ MathCraft — مبادلهٔ فایل کاربر ↔ موتور (گام 3.7 — ROADMAP §3.7).
///
/// **معماری (D-39):** موتور روی اندروید مسیر SAF نمی‌بیند؛ میزبان (Dart)
/// بایت‌ها را با `vfs_register` ثبت می‌کند و خروجی‌های `save`/`writematrix`
/// را از `EvalResult.files_written` روی دیسک می‌نویسد. «پوشهٔ MathCraft» در
/// اسناد اپ همان نقطهٔ ملاقات است — بدون پرمیشن storage، مسیر در SnackBar
/// اعلام می‌شود (اصل صداقت سند ۰۰؛ الگوی savePng گام 3.4).
///
/// فایل‌های قابل ثبت: `.m` (run/ادیتور)، `.mcws` (ورک‌اسپس)، `.mat` (خوانش
/// v5)، `.csv/.txt/.tsv` (readmatrix).
class MathCraftDir {
  MathCraftDir._();

  static const _registrable = ['.m', '.mcws', '.mat', '.csv', '.txt', '.tsv'];
  static const _maxRegisterBytes = 64 * 1024 * 1024; // سقف امانت‌داری

  /// مسیر پایهٔ تست (بدون path_provider) — فقط در تست‌ها ست می‌شود.
  @visibleForTesting
  static String? testBase;

  /// درز IO تست — ست شده = همهٔ عملیات از فیک می‌گذرند (سند ۰۸ §۶).
  @visibleForTesting
  static MathCraftIo? ioOverride;

  static MathCraftIo get _io => ioOverride ?? const _RealMathCraftIo();

  static Future<Directory> _dir() async {
    final base = testBase != null
        ? Directory(testBase!)
        : await getApplicationDocumentsDirectory();
    final dir = Directory('${base.path}/MathCraft');
    if (!await dir.exists()) {
      await dir.create(recursive: true);
    }
    return dir;
  }

  /// مسیر پوشه — برای اعلام به کاربر.
  static Future<String> path() async => (await _dir()).path;

  /// ثبت همهٔ فایل‌های پوشه در VFS موتور (نام بدون مسیر). حذف‌شده‌ها هم
  /// خودکار نادیده می‌مانند (آینه از نو ساخته می‌شود، نه merge).
  static Future<int> registerAll(
    Future<void> Function(String name, Uint8List bytes) register,
  ) =>
      _io.registerAll(register);

  static Future<int> _realRegisterAll(
    Future<void> Function(String name, Uint8List bytes) register,
  ) async {
    final dir = await _dir();
    var n = 0;
    await for (final e in dir.list()) {
      if (e is! File) continue;
      final name = e.uri.pathSegments.last;
      final dot = name.lastIndexOf('.');
      final ext = dot < 0 ? '' : name.substring(dot).toLowerCase();
      if (!_registrable.contains(ext)) continue;
      final stat = await e.stat();
      if (stat.size > _maxRegisterBytes) continue; // فایل سنگین: دستی باز شود
      final bytes = await e.readAsBytes();
      await register(name, bytes);
      n++;
    }
    return n;
  }

  /// فایل‌های `.m` پوشه — برای دیالوگ «بازکردن» ادیتور.
  static Future<List<String>> listScripts() => _io.listScripts();

  static Future<List<String>> _realListScripts() async {
    final dir = await _dir();
    final names = <String>[];
    await for (final e in dir.list()) {
      if (e is! File) continue;
      final name = e.uri.pathSegments.last;
      if (name.toLowerCase().endsWith('.m')) {
        names.add(name);
      }
    }
    names.sort();
    return names;
  }

  /// خواندن متن یک اسکریپت برای ادیتور.
  static Future<String> readScript(String name) => _io.readScript(name);

  static Future<String> _realReadScript(String name) async {
    final dir = await _dir();
    final f = File('${dir.path}/$name');
    return f.readAsString();
  }

  /// نوشتن خروجی job (save/writematrix) — مسیر فایل برمی‌گردد برای SnackBar.
  static Future<String> writeOutput(String name, Uint8List bytes) =>
      _io.writeOutput(name, bytes);

  static Future<String> _realWriteOutput(String name, Uint8List bytes) async {
    final dir = await _dir();
    final f = File('${dir.path}/$name');
    await f.writeAsBytes(bytes, flush: true);
    return f.path;
  }

  /// ذخیرهٔ متن ادیتور به‌شکل `.m` — نام بدون مسیر؛ پسوند در صورت نبود
  /// چسبانده می‌شود (رفتار متلب برای save).
  static Future<String> writeScript(String name, String text) =>
      _io.writeScript(name, text);

  static Future<String> _realWriteScript(String name, String text) async {
    var n = name.trim();
    if (n.isEmpty) n = 'script.m';
    if (!n.toLowerCase().endsWith('.m')) n = '$n.m';
    // فقط basename — امنیت مسیر
    n = n.replaceAll(RegExp(r'[/\\]'), '_');
    final dir = await _dir();
    final f = File('${dir.path}/$n');
    await f.writeAsString(text, flush: true);
    return f.path;
  }
}
