% MathCraft corpus P1 (doc 08 s2) — deterministic, tol 1e-9
A = [1 2; 3 4];
B = A';
err = max(max(abs(A' - A.')));
