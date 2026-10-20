% MathCraft corpus P1 (doc 08 s2) — deterministic, tol 1e-9
A = [1 0; 0 1; 1 1];
b = [1; 2; 3];
x = A\b;
r = max(abs(A*x - b));
