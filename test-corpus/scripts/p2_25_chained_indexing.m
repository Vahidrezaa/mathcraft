% p2_25 — Chained indexing (ROADMAP 2.10, audit item 13): postfix indexing on
% expression results. Oracle: Octave 10.3 (scripts/probe_chain210.m). The cell
% idiom x(1)(2) is deliberately absent: Octave chains freely while MathCraft
% follows MATLAB semantics (honest E4 error — unit-tested in chained_step210),
% so it cannot be corpus-compared. Manifest vars are numeric (codes) so the
% JSON differential stays numeric (p2_21 pattern).

a1 = [1 2 3](2);
a2 = [1 2 3](end);
a3 = [1 2 3](2:3);
a4 = [10; 20](2);
a5 = (1:4)(3);
b1 = ones(3,3)(2,2);
b2 = eye(2)(2,2);
f = @(x) [x x x];
c1 = f(7)(2);
c2 = f(7)([1 2]);
g = @(x) [x; 2*x];
c3 = g(3)(end);
A = [1 2; 3 4];
d1 = [1 2 3]'(1);
d2 = A(1,:)'(2);
d3 = A(2,:)'(end);
e1 = double('abc'(2));
e2 = double('hello'(end));
e3 = double('ab'([2 1]));
s = struct('v', [1 2 3]);
h1 = s.v(2);
h2 = s.v([1 3]);
h3 = s.v(end);
nm = 'v';
h4 = s.(nm)(2);
s.v(2) = 9;
h5 = s.v(2);
t = struct();
t.b(3) = 5;
h6 = t.b(3);
w = struct(); w.x.y = [1 2 3]; w.x.y(2) = 8;
h7 = w.x.y(2);
k = [[1 2](1), 5];
