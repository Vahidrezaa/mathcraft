% P2 corpus — [Q,R] = qr (ROADMAP 2.6) — کمیت‌های ناوردا
% علامت ستون‌های Q بین LAPACK (متلب/Octave) و faer (MathCraft) مشروعاً
% می‌تواند متفاوت باشد؛ اوراکل = ناورداها: A=Q·R، ارتونالیتی، مثلثی‌بودن R.
A = [1 2; 3 4; 5 6];
[Q, R] = qr(A);
errA = max(max(abs(Q*R - A)));
errQ = max(max(abs(Q'*Q - eye(3))));
errR = max(max(abs(tril(R, -1))));
