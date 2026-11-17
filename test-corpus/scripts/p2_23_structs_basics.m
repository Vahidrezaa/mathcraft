% p2_23 — Structs minimal (ROADMAP 2.9, tasks 1-2): constructor, s.f access,
% field assignment with auto-vivification, dynamic fieldnames, the minimal
% builtin set, and the 1x1 shape family. Oracle: Octave 10.3 sandbox probe
% (scripts/probe_structs.m). Manifest vars are numeric so the JSON differential
% stays numeric (p2_21 pattern); fieldnames' cell-vs-char delta is NOT compared
% here (documented deviation — covered by unit tests instead).

s = struct('a', 3.5, 'b', [1 2 3]);
a1 = s.a;                % 3.5
b3 = s.b * 2;            % [2 4 6]
f1 = isfield(s, 'a');    % 1
f2 = isfield(s, 'zz');   % 0
g = getfield(s, 'b');    % [1 2 3]
s2 = setfield(s, 'c', 9);
c9 = s2.c;               % 9
o = isfield(s, 'c');     % 0 — setfield returns a new struct
s.c = 7;                 % field assignment
c7 = s.c;                % 7
nm = 'a';
d = s.(nm);              % 3.5 — dynamic fieldname
s.('e') = 5;
e5 = s.e;                % 5
sz = size(s);            % [1 1]
cnt = numel(s);          % 1
cls = isequal(class(s), 'struct');  % 1 (strcmp is E4 — string library)
w = struct();
w.x.y = 5;               % nested auto-vivification
tyc = w.x.y;             % 5
eq1 = isequal(struct('a', 1, 'b', 2), struct('b', 2, 'a', 1));  % 1
eq2 = isequal(struct('a', 1), struct('a', 2));                  % 0
dup = struct('a', 1, 'a', 2);   % duplicate field: last wins (Octave 10.3)
dv = dup.a;              % 2
n = 0; tot = 0;
for k = s, n = n + 1; tot = tot + k.a; end  % 1x1 struct iterates once
