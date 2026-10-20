% octave_regen.m — regenerate the golden-corpus reference output with Octave (doc 08 §2).
% Run: octave --no-gui --eval "run('test-corpus/octave_regen.m')"
% Output: test-corpus/generated/*.json (temporary — compared against expected/ by
% scripts/compare_expected.py; the checked-in expected/ comes from the numpy/LAPACK
% oracle, never from MathCraft, and is never overwritten here).
% Requires Octave >= 7 (built-in jsondecode/jsonencode).
%
% NOTE (NaN/Inf): Octave's jsonencode maps NaN AND +-Inf to null — unlike MATLAB,
% which emits "NaN"/"Infinity" strings. compare_expected.py normalizes null->NaN.
% The P1 corpus contains NaN but no Inf (see manifest.json / p1_38).
%
% NOTE (isolated workspace + helpers/): each corpus script runs inside the
% eval_case_isolated function's own workspace, so it can never clobber this
% script's loop variables (corpus scripts define n, k, c, ...). The helper
% lives in helpers/ and is resolved via addpath — see its header comment.

root = fileparts(mfilename('fullpath'));
cd(root);
addpath(fullfile(root, 'helpers'));
man = jsondecode(fileread(fullfile(root, 'manifest.json')));

cases = man.cases;
if ~iscell(cases)
  % jsondecode yields: struct array (N cases) or scalar struct (1 case, collapsed).
  % num2cell -> Nx1 cell of structs. NEVER wrap with {cases}: that nests the whole
  % array in a 1x1 cell, cases{k} returns the entire struct array, c.vars becomes
  % a comma-separated list and the helper call expands to N+2 arguments.
  cases = num2cell(cases);
end

if ~exist(fullfile(root, 'generated'), 'dir')
  mkdir(fullfile(root, 'generated'));
end

n = numel(cases);
for k = 1:n
  c = cases{k};
  vars = c.vars;
  if ischar(vars)          % jsondecode collapse guard (single-var case)
    vars = {vars};
  end
  rec = eval_case_isolated(fullfile(root, 'scripts'), c.script, vars);
  json = jsonencode(rec);
  fname = fullfile(root, 'generated', regexprep(c.script, '\.m$', '.json'));
  fid = fopen(fname, 'w');
  fwrite(fid, json, 'char');
  fclose(fid);
  fprintf('regen %d/%d: %s\n', k, n, c.script);
end
fprintf('OK: %d files in test-corpus/generated/\n', n);
