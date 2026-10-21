% MathCraft corpus P1 (doc 08 s2) — deterministic, tol 1e-9
A = [4 2; 2 3];
L = chol(A);
err = max(max(abs(L*L' - A)));
l11 = L(1,1);
