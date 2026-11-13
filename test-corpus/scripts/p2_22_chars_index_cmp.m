% p2_22 — Char indexing + comparisons + string builtins (ROADMAP 2.8).
% Oracle: Octave 10.3 (scripts/probe_chars.m). Manifest vars numeric (codes).

h = 'hello';
second = double(h(2));            % 101
pair = double(h([1 3]));          % [104 108]
rev = double(h(end:-1:1));        % codes of 'olleh'
eqv = 'abc' == 'abd';             % [1 1 0]
ltv = 'a' < 'b';                  % 1
eqc = 'a' == 97;                  % 1
is_c = double(ischar('abc'));     % 1
is_d = double(ischar(5));         % 0
sd = str2double('1.5');           % 1.5
bl = double(blanks(3));           % [32 32 32]
ch = double(char([72 105]));      % 'Hi' → [72 105]
