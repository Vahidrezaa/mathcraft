% p2_21 — Char type & strings (ROADMAP 2.8, audit items 6/12): mixed concat
% promotes numerics to char codes; numeric promotion in arithmetic.
% Oracle: Octave 10.3 (scripts/probe_chars.m) + MATLAB char documentation.
% Manifest vars are numeric (codes) so the JSON differential stays numeric.

m = [65 'a'];            % 'Aa' — mixed concat promotes numerics to char codes
mc = double(m);          % [65 97]
s = 'abc';
plus = s + 1;            % [98 99 100] — numeric promotion
cc = 'a' + 'b';          % 195
mm = 'ab' * 2;           % [194 196] — scalar multiply stays numeric
el = 'ab' + 'cd';        % [196 198]
dv = 'ab' ./ 2;          % [48.5 49]
b66 = double(char(66));  % 'B' → 66
