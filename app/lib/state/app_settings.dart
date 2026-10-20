import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

/// تنظیمات سراسری اپ — سوییچ **آنی** تم/زبان (DoD فاز P0 سند ۰۷).
///
/// پیش‌فرض‌ها: تم تیره «حالت مدرن» (سند ۰۶ اصل ۴) و زبان فارسی (بازار اصلی —
/// تصمیم D-06). ذخیره‌سازی ماندگار با shared_preferences در P5 می‌آید
/// (سند ۰۲ بخش ۶).
final themeModeProvider =
    StateProvider<ThemeMode>((ref) => ThemeMode.dark);

final localeProvider = StateProvider<Locale>((ref) => const Locale('fa'));
