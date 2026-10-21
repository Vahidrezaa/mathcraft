% MathCraft corpus P1 (doc 08 s2) — deterministic, tol 1e-9
A = [1 2; 3 4];
M = A^2;
E = A.^2;
d1 = M(1,1) + M(2,2);
d2 = E(1,1) + E(2,2);
