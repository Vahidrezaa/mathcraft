% MathCraft corpus P1 (doc 08 s2) — deterministic, tol 1e-9
U = [2 1; 0 4];
b = [4; 8];
x = U\b;
chk = max(abs(U*x - b));
