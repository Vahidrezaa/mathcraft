% P2 corpus — y' = -y, y(0)=1 → y(2) = exp(-2)
f = @(t, y) -y;
[T, Y] = ode45(f, [0 2], 1);
n = length(T);
y_end = Y(n);
