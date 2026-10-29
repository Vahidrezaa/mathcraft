% P2 corpus — [U,S,V] = svd (ROADMAP 2.6) — ناورداها + σ تحلیلی
% A = [3 0; 0 2; 0 0] → σ = [3; 2] (دقیق)؛ علامت ستون‌های U/V پیاده‌سازی‌وابسته.
A = [3 0; 0 2; 0 0];
[U, S, V] = svd(A);
s = svd(A);
errA = max(max(abs(U*S*V' - A)));
errU = max(max(abs(U'*U - eye(3))));
errV = max(max(abs(V'*V - eye(2))));
sig1 = s(1);
sig2 = s(2);
