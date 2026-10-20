% MathCraft corpus P1 (doc 08 s2) — deterministic, tol 1e-9
c1 = cond([2 0; 0 0.5]);
ci = cond(eye(3));
n1 = norm([3 4], 1);
ni = norm([3 4], inf);
nf = norm([1 2; 3 4], 'fro');
