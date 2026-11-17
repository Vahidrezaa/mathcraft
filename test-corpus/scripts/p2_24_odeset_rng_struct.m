% p2_24 — odeset struct form + rng status struct (ROADMAP 2.9, task 4).
% Engine-internal case: solver-path numerics and the RNG sequence are
% engine-specific (documented deviations — COMPATIBILITY §4.5), so this case
% stays OUTSIDE the Octave manifest (same precedent as vdp1000 / RNG / Waypoints
% / histcounts). Assertions (Rust test corpus_p2_golden.rs):
%   - struct form is bit-identical to the named-pair form (option invariance)
%   - y(2) matches exp(-2) analytically within the solver tolerance
%   - odeset(old, ...) merge: fresh pairs win (MATLAB precedence)
%   - rng capture/restore replays the exact sequence

f = @(t, y) -y;
opts = odeset('RelTol', 1e-8, 'AbsTol', 1e-11);
[T1, Y1] = ode45(f, [0 2], 1, opts);
[T2, Y2] = ode45(f, [0 2], 1, 'RelTol', 1e-8, 'AbsTol', 1e-11);
inv = max(abs(Y1(:) - Y2(:)));   % 0 — struct form ≡ named pairs
yend = Y1(end);
err = abs(yend - exp(-2));       % < 1e-6 (analytic)
o3 = odeset(opts, 'RelTol', 1e-4);  % merge: pairs win
rt = o3.RelTol;                  % 1e-4
at = o3.AbsTol;                  % 1e-11 (kept)
st = rng(42);                    % status struct after reseeding
a = rand;
b = rand;
rng(st);                         % restore
a2 = rand;
b2 = rand;
rep = [a == a2, b == b2];        % [1 1]
sd = st.Seed;                    % 42
