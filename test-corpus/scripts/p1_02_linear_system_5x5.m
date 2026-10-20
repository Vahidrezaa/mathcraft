% MathCraft corpus P1 (doc 08 s2) — deterministic, tol 1e-9
A = [10 1 0 0 0; 1 10 1 0 0; 0 1 10 1 0; 0 0 1 10 1; 0 0 0 1 10];
b = [11; 12; 12; 12; 11];
x = A\b;
r = A*x - b;
maxabs = max(abs(r));
