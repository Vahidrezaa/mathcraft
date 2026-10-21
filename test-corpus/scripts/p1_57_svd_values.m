% MathCraft corpus P1 (doc 08 s2) — deterministic, tol 1e-9
s = svd([3 0; 0 2]);
A2 = [4 11 14; 8 7 -2];
sv = svd(A2);
s1 = sv(1);
s2 = sv(2);
