% MathCraft corpus P1 (doc 08 s2) — deterministic, tol 1e-9
A = [1 2; 3 4];
L = tril(A);
U = triu(A);
zs = sum(sum(L + U - A));
