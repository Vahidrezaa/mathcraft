import 'dart:io';
import 'dart:typed_data';
import 'dart:ui' as ui;

import 'package:flutter/rendering.dart';
import 'package:flutter/widgets.dart';
import 'package:path_provider/path_provider.dart';

import '../../core/io/mathcraft_dir.dart';
import '../../src/rust/api.dart' as api;
import 'package:share_plus/share_plus.dart';

/// خروجی PNG و اشتراک — سند ۰۶ §7.3 (نوار شناور: ذخیره PNG، اشتراک).
///
/// بدون وابستگی به موتور: capture از RepaintBoundary خود نمودار با
/// pixelRatio ۳ (کیفیت چاپ)، PNG خالص با toByteData.
Future<ui.Image> captureBoundaryToImage(
  GlobalKey boundaryKey, {
  double pixelRatio = 3.0,
}) async {
  final boundary = boundaryKey.currentContext?.findRenderObject()
      as RenderRepaintBoundary?;
  if (boundary == null) {
    throw StateError('plot boundary not mounted');
  }
  return boundary.toImage(pixelRatio: pixelRatio);
}

Future<Uint8List> imageToPng(ui.Image image) async {
  final data = await image.toByteData(format: ui.ImageByteFormat.png);
  if (data == null) {
    throw StateError('png encoding failed');
  }
  return data.buffer.asUint8List();
}

/// ذخیره در پوشهٔ اسناد اپ (بدون مجوز storage؛ مسیر در SnackBar اعلام
/// می‌شود — اصل صداقت سند ۰۰).
Future<String> savePng(Uint8List png, String fileName) async {
  final dir = await getApplicationDocumentsDirectory();
  final file = File('${dir.path}/$fileName');
  await file.writeAsBytes(png, flush: true);
  return file.path;
}

/// اشتراک — از حافظه، بدون فایل موقت (XFile.fromData).
Future<void> sharePng(Uint8List png, String fileName) async {
  await Share.shareXFiles([
    XFile.fromData(
      png,
      mimeType: 'image/png',
      name: fileName,
    ),
  ]);
}


/// خروجی PDF برداری (گام 3.8 — سند ۰۴ §8): رندر همان شکل در **موتور**
/// (mc-plot::pdf) — نه اسکرین‌شات؛ ذخیره در سندباکس MathCraft (مثل
/// exportgraphics). شکل نبود = خطای صریح.
Future<String> exportFigurePdf(BigInt figureId, String fileName) async {
  final bytes = await api.exportFigurePdf(figureId: figureId);
  if (bytes.isEmpty) {
    throw StateError('figure $figureId not found for PDF export');
  }
  return MathCraftDir.writeOutput(fileName, bytes);
}
