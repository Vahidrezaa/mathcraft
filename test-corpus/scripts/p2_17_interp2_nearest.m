% P2 corpus — interp2 nearest (ROADMAP 2.6) — بدون تساوی نیمه (تفاوت‌پذیر بی‌ابهام)
x = [0 1];
y = [0 1];
V = [1 2; 3 4];
q1 = interp2(x, y, V, 0.75, 0.25, 'nearest');   % گره (1,0) → 2
q2 = interp2(x, y, V, 0.25, 0.75, 'nearest');   % گره (0,1) → 3
