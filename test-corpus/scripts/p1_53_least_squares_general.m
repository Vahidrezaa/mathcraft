% MathCraft corpus P1 (doc 08 s2) — deterministic, tol 1e-9
A = [1 0; 1 1; 1 2];
b = [6; 0; 0];
x = A\b;
rn = norm(A*x - b);
