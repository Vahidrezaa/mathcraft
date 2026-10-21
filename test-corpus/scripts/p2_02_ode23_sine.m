% P2 corpus — y' = y*sin(t), y(0)=1 → y(2) = exp(1-cos(2))
f = @(t, y) y * sin(t);
[T, Y] = ode23(f, [0 2], 1);
n = length(T);
y_end = Y(n);
