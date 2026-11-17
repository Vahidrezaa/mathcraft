/// قالب اعداد نمودار — آینهٔ دقیق `mc_plot::ticks::format_g4` (پارتی `%.4g`
/// C؛ سند ۰۴ §8.1): ۴ رقم معنادار، نماد علمی فقط وقتی توان < -4 یا ≥ 4،
/// حذف صفرهای انتهایی و نقطهٔ یتیم.
///
/// هم‌ارزی با Rust با تست مشترک نگه داشته می‌شود (plots_test g4_table).
String formatG4(double v) {
  if (v == 0.0) return '0';
  if (v.isNaN) return 'NaN';
  if (v.isInfinite) return v > 0 ? 'Inf' : '-Inf';
  final negative = v < 0;
  final a = v.abs();
  // توان را از قالب علمی می‌خوانیم (مقاوم به خطای floor(log10(0.99..))) —
  // همان ترفند سمت Rust
  final sci = a.toStringAsExponential(3); // d.ddd e±x
  final eIdx = sci.indexOf('e');
  final exp = int.tryParse(sci.substring(eIdx + 1)) ?? 0;
  String body;
  if (exp < -4 || exp >= 4) {
    final mantissa = a / _pow10(exp);
    var ms = _fixedHalfEven(mantissa, 3);
    ms = _stripTrailingZeros(ms);
    final sign = exp < 0 ? '-' : '+';
    final absExp = exp.abs().toString().padLeft(2, '0');
    body = '${ms}e$sign$absExp';
  } else {
    final precision = (3 - exp).clamp(0, 20);
    var fs = _fixedHalfEven(a, precision);
    fs = _stripTrailingZeros(fs);
    body = fs;
  }
  return negative ? '-$body' : body;
}

double _pow10(int exp) {
  var result = 1.0;
  for (var i = 0; i < exp.abs(); i++) {
    result *= 10;
  }
  return exp < 0 ? 1 / result : result;
}

/// قالب اعشار با گرد نیم‌به‌زوج — هم‌ارز `{:.p}` Rust/C (رفتار متلب):
/// Dart's toStringAsFixed نیم‌به‌بالا است و در گرهٔ دقیق (مثل 1234.5) از
/// Rust منحرف می‌شود؛ این‌جا مقیاس‌کردن دقیق + قاعدهٔ زوج.
String _fixedHalfEven(double a, int p) {
  final scaled = a * _pow10(p);
  final fl = scaled.floorToDouble();
  final diff = scaled - fl;
  double r;
  if (diff > 0.5) {
    r = fl + 1;
  } else if (diff < 0.5) {
    r = fl;
  } else {
    // دقیقاً نیم — به زوج
    r = fl.toInt().isEven ? fl : fl + 1;
  }
  var s = r.toInt().toString();
  if (p == 0) return s;
  if (s.length <= p) s = s.padLeft(p + 1, '0');
  return '${s.substring(0, s.length - p)}.${s.substring(s.length - p)}';
}

String _stripTrailingZeros(String s) {
  if (!s.contains('.')) return s;
  while (s.endsWith('0')) {
    s = s.substring(0, s.length - 1);
  }
  return s.endsWith('.') ? s.substring(0, s.length - 1) : s;
}

/// برچسب تیک لگاریتمی `10^{k}` → دو تکه برای رندر superscript:
/// (پایهٔ «10»، توان «k»)؛ رشتهٔ ناهم‌شکل → بدون توان (null).
(String, String?) parseLogLabel(String label) {
  final match = RegExp(r'^10\^\{(-?\d+)\}$').firstMatch(label);
  if (match == null) return (label, null);
  return ('10', match.group(1));
}
