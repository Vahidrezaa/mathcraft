% P2 corpus — interp2 خطی (ROADMAP 2.6) — صفحهٔ خطی V = 10x + 2y: دقیق
x = [0 1 2];
y = [0 1];
V = [0 10 20; 2 12 22];
q1 = interp2(x, y, V, 1.5, 0.5);            % 10*1.5 + 2*0.5 = 16
Q = interp2(x, y, V, [0.5 1.5], [0 1]');    % گسترش meshgrid 2×2
e = max(max(abs(Q - [5 15; 7 17])));
