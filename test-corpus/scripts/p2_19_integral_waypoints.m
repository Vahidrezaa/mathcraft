% P2 corpus — integral با Waypoints (ROADMAP 2.6) — مرجع تحلیلی
% (خارج از manifest اکتاو: integral اکتاو گزینهٔ Waypoints ندارد)
q = integral(@(x) 1 + (x >= 0.5), 0, 1, 'Waypoints', [0.5]);   % = 1.5
r = integral(@(x) abs(x), -1, 1, 'Waypoints', 0);              % = 1
