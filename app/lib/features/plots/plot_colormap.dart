import 'dart:math' as math;
import 'dart:ui';

import '../../src/rust/api.dart' as api;

/// پورت وفادار جدول‌های colormap موتور (mc-plot/src/colormap.rs — گام 3.8).
/// الگوی تیک 1-2-5 (سند ۰۴ §8.3): هر تغییر در Rust باید اینجا حمل شود؛
/// تست دوقطبی `plot_colormap_port_matches_kernel` مقدارهای لنگر را با
/// موتور هم‌قفل می‌کند (از طریق export PDF نمی‌شود — با golden چشمی).
const _parula = <List<double>>[
  [0.2081, 0.1663, 0.5292],
  [0.2110, 0.2358, 0.6445],
  [0.2121, 0.3182, 0.7369],
  [0.2088, 0.4005, 0.7975],
  [0.1951, 0.4754, 0.8208],
  [0.1685, 0.5320, 0.8093],
  [0.1298, 0.5699, 0.7776],
  [0.0893, 0.5958, 0.7354],
  [0.0599, 0.6163, 0.6924],
  [0.0812, 0.6517, 0.6353],
  [0.1870, 0.6833, 0.5560],
  [0.3449, 0.7048, 0.4690],
  [0.5200, 0.7151, 0.3800],
  [0.6861, 0.7071, 0.2959],
  [0.8313, 0.6741, 0.2230],
  [0.9424, 0.6105, 0.1753],
];

const _viridis = <List<double>>[
  [0.2670, 0.0049, 0.3294],
  [0.2826, 0.0949, 0.4173],
  [0.2954, 0.1804, 0.4948],
  [0.3080, 0.2587, 0.5575],
  [0.3228, 0.3337, 0.6047],
  [0.3423, 0.4074, 0.6370],
  [0.3681, 0.4795, 0.6548],
  [0.3990, 0.5495, 0.6590],
  [0.4345, 0.6164, 0.6500],
  [0.4735, 0.6790, 0.6291],
  [0.5160, 0.7368, 0.5963],
  [0.5625, 0.7884, 0.5520],
  [0.6144, 0.8323, 0.4958],
  [0.6731, 0.8648, 0.4265],
  [0.7407, 0.8841, 0.3422],
  [0.8194, 0.8878, 0.2374],
];

/// نمونه‌برداری t∈[0,1] — همان قواعد Rust (جدولی/تحلیلی).
Color sampleColormap(api.Colormap cm, double t) {
  t = t.clamp(0.0, 1.0);
  final rgb = switch (cm) {
    api.Colormap.parula => _tableLerp(_parula, t),
    api.Colormap.viridis => _tableLerp(_viridis, t),
    api.Colormap.jet => _jet(t),
    api.Colormap.hot => _hot(t),
    api.Colormap.cool => [t, 1.0 - t, 1.0],
    api.Colormap.gray => [t, t, t],
  };
  return Color.fromARGB(
    255,
    (rgb[0].clamp(0.0, 1.0) * 255).round(),
    (rgb[1].clamp(0.0, 1.0) * 255).round(),
    (rgb[2].clamp(0.0, 1.0) * 255).round(),
  );
}

/// رنگ متناظر با مقدار z روی بازهٔ caxis (نرمال‌سازی همان Rust).
Color sampleColormapZ(api.Colormap cm, double z, (double, double) caxis) {
  final span = caxis.$2 - caxis.$1;
  final t = span > 0 ? (z - caxis.$1) / span : 0.5;
  return sampleColormap(cm, t);
}

List<double> _tableLerp(List<List<double>> table, double t) {
  final pos = t * (table.length - 1);
  final i = pos.floor().clamp(0, table.length - 1);
  final j = (i + 1).clamp(0, table.length - 1);
  final f = pos - i;
  return [
    table[i][0] + f * (table[j][0] - table[i][0]),
    table[i][1] + f * (table[j][1] - table[i][1]),
    table[i][2] + f * (table[j][2] - table[i][2]),
  ];
}

/// فرمول استاندارد jet (1.5−|4t−k|) — پورت عیناً از Rust.
List<double> _jet(double t) {
  final four = 4.0 * t;
  double c(double k) => (1.5 - (four - k).abs()).clamp(0.0, 1.0);
  return [c(3.0), c(2.0), c(1.0)];
}

List<double> _hot(double t) {
  return [
    (t / 0.375).clamp(0.0, 1.0),
    ((t - 0.375) / 0.375).clamp(0.0, 1.0),
    ((t - 0.75) / 0.25).clamp(0.0, 1.0),
  ];
}

/// پروجکشن دوربین سه‌بعدی — پورت وفادار mc_plot::pdf::project (گام 3.8):
/// az حول z (مثبت پادساعتگرد از −y)، el زاویه بالا؛ خروجی
/// (screenX, screenY, depth) برای نقطهٔ نرمال‌شدهٔ [0,1]³.
(double, double, double) project3d(
  double azDeg,
  double elDeg,
  double x,
  double y,
  double z,
) {
  final az = azDeg * 3.141592653589793 / 180.0;
  final el = elDeg * 3.141592653589793 / 180.0;
  final sinAz = math.sin(az), cosAz = math.cos(az);
  final sinEl = math.sin(el), cosEl = math.cos(el);
  final sx = x * cosAz + y * sinAz;
  final sy = x * (-sinEl * sinAz) + y * (sinEl * cosAz) + z * cosEl;
  final depth = x * (cosEl * sinAz) + y * (-cosEl * cosAz) + z * sinEl;
  return (sx, sy, depth);
}
