% MathCraft corpus P1 (doc 08 s2) — deterministic, tol 1e-9
L = chol([4 2; 2 3]);
err = max(max(abs(L*L' - [4 2; 2 3])));
n = norm([3 4]);
