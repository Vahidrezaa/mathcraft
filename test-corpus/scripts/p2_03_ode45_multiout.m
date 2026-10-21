% P2 corpus — [T, Y] = ode45 شکل چندخروجی
f = @(t, y) -y;
[T, Y] = ode45(f, [0 1], 2);
n = length(T);
last_t = T(n);
