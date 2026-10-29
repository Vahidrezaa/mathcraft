% P2 corpus — histcounts/histogram (ROADMAP 2.6) — مرجع تحلیلی
% (خارج از manifest اکتاو: Octave 9.4 این توابع را ندارد)
x = [0.5 0.8 1.2 1.9 2.3 3.0];
[N, edges] = histcounts(x, [0 1 2 3]);      % آخرین بین بستهٔ راست
[Nb, eb, bin] = histcounts([-1 0.5 0.7 2.0], [0 1 2]);
h = histogram([0.2 0.7 1.1], 'BinWidth', 0.5);
