% MathCraft corpus P1 (doc 08 s2) — deterministic, tol 1e-9
A = [3 1; 1 2];
b = [9; 8];
x = A\b;
res = max(abs(A*x - b));
