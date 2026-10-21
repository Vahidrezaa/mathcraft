% P2 corpus — van der Pol mu=1000 (DoD سند ۰۷) — مرجع Octave 9.4
f = @(t, y) [y(2); 1000*(1-y(1)^2)*y(2) - y(1)];
[T, Y] = ode45(f, [0 3000], [2; 0]);
n = length(T);
y1_end = Y(n, 1);
y2_end = Y(n, 2);
ymax = max(Y(:, 1));
