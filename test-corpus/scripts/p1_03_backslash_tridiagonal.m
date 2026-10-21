% MathCraft corpus P1 (doc 08 s2) — deterministic, tol 1e-9
A = [4 1 0 0; 1 4 1 0; 0 1 4 1; 0 0 1 4];
b = [5; 6; 6; 5];
x = A\b;
r = max(abs(A*x - b));
