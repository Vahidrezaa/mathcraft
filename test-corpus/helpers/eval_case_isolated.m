function rec = eval_case_isolated(scripts_dir, script_name, varnames)
% eval_case_isolated — run one corpus script in THIS function's own workspace.
%
% Why a function: `run` inside a function executes the script in the function's
% LOCAL workspace, so the caller's loop variables (k, c, n, man, ...) can never
% be clobbered by the corpus script itself. Corpus scripts DO define such names
% (p1_09 defines n, p1_11 defines k, p1_12 defines c) — running them in the
% caller's scope corrupted the regen loop ("regen 9/5" counter + "matrix
% cannot be indexed with ." on c.vars).
%
% Layout note: this file lives in test-corpus/helpers/ (NOT next to
% octave_regen.m). Octave's function resolution for m-files sitting in the
% same directory as a run()'d script is unreliable across builds (observed
% "function called with too many inputs" for a correct 3-arg call); resolution
% via an explicitly addpath()'d helper directory behaves correctly everywhere.
%
% varnames: cell array of variable names the script assigns.
% Requires Octave >= 7 (jsonencode/jsonencode built-in) — doc 08 §2.

  cwd = pwd;
  unwind_protect
    cd(scripts_dir);
    run(script_name);
    rec = struct();
    for j = 1:numel(varnames)
      name = varnames{j};
      val = eval(name);   % sees the script's variables in this local scope
      if iscomplex(val)
        error('eval_case_isolated: complex value for "%s" - extend the JSON contract first', name);
      end
      rec.(name) = val;   % jsonencode: scalar/vector/matrix/[] ; NaN and +-Inf -> null
    end
  unwind_protect_cleanup
    cd(cwd);
  end_unwind_protect
end
