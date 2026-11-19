% تولید فیکسچرهای .mat با نویسندهٔ واقعی Octave — اوراکل CI گام 3.7.
% اجرا: MC_OCTAVE_OUT=<dir> octave --quiet gen_octave_fixtures.m
% محتوا آینهٔ scripts/gen_mat_fixtures.py (scipy) است؛ تست
% matv5_octave_parity هر دو نویسنده را با هم می‌سنجد (برابری ارزشی).

out = getenv('MC_OCTAVE_OUT');
if isempty(out)
  error('MC_OCTAVE_OUT is not set');
end

M = [1 2 3; 4 5 6];
save('-mat-binary', fullfile(out, 'dbl_2x3.mat'), 'M');   % v6 بدون فشرده‌سازی

C = [1+2i, -1i; 3.5-4i, 0.5i];
save('-mat7-binary', fullfile(out, 'cplx_2x2.mat'), 'C');  % v7 فشرده (flate2)

s = 'hello';
save('-mat7-binary', fullfile(out, 'char_row.mat'), 's');

st = struct('a', 1.25, 'b', 'txt', 'inner', struct('x', [1; 2]));
save('-mat7-binary', fullfile(out, 'struct_1x1.mat'), 'st');

z = [NaN, Inf; -Inf, -0];
save('-mat7-binary', fullfile(out, 'nan_inf.mat'), 'z');

v = int32([7; -8; 9]);
save('-mat7-binary', fullfile(out, 'int32_col.mat'), 'v');

S = single([0.5, 1.5; -2.5, 3.5]);
save('-mat7-binary', fullfile(out, 'single_2x2.mat'), 'S');

L = logical([true, false, true]);
save('-mat7-binary', fullfile(out, 'logical_1x3.mat'), 'L');

e = zeros(0, 0);
save('-mat7-binary', fullfile(out, 'empty_0x0.mat'), 'e');

printf('octave fixtures written to %s\n', out);
