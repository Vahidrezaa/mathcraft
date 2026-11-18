# ROADMAP.md — Master Execution Plan to v1.0

> **MathCraft** | Document version 1.0 | 2026-10-25 | Status: **BINDING**
>
> This is the operative, step-level execution plan implementing the milestone model of
> `docs/07-roadmap.md` (scope / DoD / demo per milestone remain governed there).
> It answers one question for every working session: **"what exactly do I focus on now?"**
>
> **Governance (user decision 2026-10-25):** this document is the single plan of record.
> Steps are executed in the order written. Skipping a step, reordering, re-scoping, or
> closing a phase requires the user's **explicit approval in chat first** — the change is
> then recorded in the Change Log at the bottom of this file before any code is touched
> (golden rule of docs/00: documents before code).
>
> **Progress metric:** test counts + DoD checkboxes only. No calendar dates (D-07,
> milestone model). A phase without a running demo is not closed — "almost done" means
> not done.
>
> **Session protocol (mandatory, per docs/STATUS §4 protocol v2):**
> Start of session: read this file (Current Focus) → `docs/STATUS.md` latest section →
> `worklog.md` tail → `git log`. End of session: update STATUS + worklog, commit, mirror
> package to `upload/`, hand package/patch to user.

---

## 1. Where we are (baseline, verified 2026-10-25)

Repository state at document creation: **HEAD `1ce39a2`**, clean tree, **281/281 Rust tests
green** (fmt + clippy strict + cargo-deny green; Flutter analyze + 11 widget tests green in CI;
Octave differential job 60/60). Package of record:
`MathCraft-P2-v0.3-audit-batch15-WITH-GIT.tar.gz` (sha256 `b4debe72…`).

| Milestone (docs/07) | State | Evidence |
|---|---|---|
| P0 Foundation (+ P0.1 Termux CLI, P0.2 contract v2 / Android CI) | ✅ COMPLETE | commits `cb2cbba…7748a62`, `9d8403f` |
| P1 Language & Matrix (17/17 DoD, device-confirmed) | ✅ COMPLETE | `669170d`, corpus 60/60, 182 tests |
| P2 Numeric & Statistics — core engine | ✅ core done | `e608219…c11601f`, 256 tests, corpus p2 (13 scripts) |
| P2 — external audit hardening, batch 1 + 1.5 | ✅ done | `1dcbfe4`, `25038fc`, 281 tests; audit items 1/3/4/7 closed, 15 half-closed |
| P2 — remaining (event FFI, display parity, robustness, language gaps) | 🔶 in progress | scheduled below as steps 2.3–2.11 |
| P3 Plotting / P4 CAS / P5 IDE / P6 Beta / v1.0 | ⬜ | planned below |

Known open device confirmations carried forward: final touch verification of FB-1/FB-2
(display layout, themed selection) — folded into step 2.7.

## 2. Phase map to v1.0

```
P2 Engine & Language Hardening (steps 2.3–2.7 + sub-phase P2-c: 2.8–2.11)
        │
        ▼
P3 Plotting 2D → 3D (steps 3.1–3.6)
        │
        ▼
P4 Symbolic CAS (CAS-1 → CAS-2 → CAS-3, steps 4.1–4.4)
        │
        ▼
P5 IDE Experience (steps 5.1–5.6)
        │
        ▼
P6 Beta & Stabilization (steps 6.1–6.3)
        │
        ▼
v1.0 Public Release  ──►  E1..E8 expansion phases (named only, §8)
```

Numbering note: COMPATIBILITY §4.8 rows marked "P3" for audit items 6/12/13/14 predate
this document; in this plan they are executed as **P2-c (steps 2.8–2.11)**, *before*
plotting begins. docs/07 milestone **P3 remains Plotting**. The ledger note was updated
accordingly. Rationale: plotting, IDE and CAS all build on a complete language core;
shipping a plotter on top of a half-language would force rework.

---

## 3. Phase 2 (continued) — Engine & Language Hardening

> Phase exit gate: docs/07 §P2 DoD + P2-c completion (§2.7).

### Step 2.3 — Event-based FFI: progress + cancel (+ release panic policy)  **[NEXT UP]**
- **Goal:** long solver runs (ode45 / ode23 / integral / fzero) report progress to the UI and
  can be cancelled mid-run from the device; runaway user loops (`while(1) end`) are
  cancellable too. The half-open audit item 15 (panic policy in release) is decided and
  closed here.
- **Tasks:**
  1. Extend the FFI contract (docs/02 §3–4) to contract **v2 (additive)**: job-scoped
     events — `Progress{job_id, stage, percent}`, `Cancelled{job_id}`, streamed via FRB;
     keep `evaluate_line` working unchanged (mc-cli must stay green in the same commit).
  2. Plumb solver `AtomicBool` cancel flags (already built into ode45/ode23/integral)
     through the eval loop; add cooperative cancel checks in `for`/`while` interpretation
     (bound: check every N statements, N documented).
  3. FRB stream wiring + codegen freshness gate; update Flutter glue.
  4. Flutter: Stop button on console/editor output panel + progress indicator using CDS
     tokens and Craft Icons only (no emoji, no Material icons).
  5. Decide release panic policy with the user: **Option A (default recommendation)** keep
     `panic="abort"` + CI panic-audit gate (grep-guard `panic!/unwrap/expect` in non-test
     engine paths, already partially enforced) ; **Option B** release unwinds + `catch_unwind`
     shield (binary size cost). Record decision in COMPATIBILITY §4.8 row 15.
- **DoD:**
  - [ ] Pendulum `ode45` demo on device shows a progress bar; Stop mid-run cancels cleanly;
        workspace afterwards is consistent (no partial garbage assigned), like MATLAB `stop`.
  - [ ] `while(1) end` is cancellable from the UI.
  - [ ] Contract v2 round-trip tests; mc-cli consumer updated and green.
  - [ ] ≥ 295 tests green (281 + new); fmt / clippy strict / deny green; glue-freshness gate green.
- **Tests:** FFI event unit tests with a fake slow solver; cancel-consistency regression
  (workspace snapshot before/after cancel); widget test for Stop-button states; contract
  v2 round-trip.
- **Risks:** FRB stream overhead or instability on Android/Termux → fallback: polling
  `JobStatus` API (still satisfies cancel/progress DoD; stream upgrade becomes backlog);
  global RNG state + events → sessions remain serialized (docs/02 §4), documented.
- **Deliverable:** commit series + package `…v0.4…` (WITH-GIT + src) with sha256 recorded
  in worklog.
- **Stop / rollback:** if event streaming is not stable at packaging time, ship the polling
  fallback and register the stream path as backlog item; rollback = `git revert` of the FFI
  commits, contract v1 restored.

### Step 2.4 — Display & output parity (audit items 5, 8, 10, 11 + `%{ %}` comments)
- **Goal:** console output matches MATLAB in the remaining cheap cases; closes ledger
  items 5/8/10/11 and the block-comment lexer gap (part of item 9).
- **Tasks:**
  1. Item 5 — index orientation: indexing result follows the index's orientation
     (`a(end:-1:1)` stays a row); add matrix-row/col regression cases.
  2. Item 8 — builtins: `disp`, `class`, `fprintf` (`%d %f %s %.Nf`, `\n`; honest error on
     unsupported specifiers), `num2str`; constants `intmax/realmax/realmin` (ledger §2).
  3. Item 10 — empty-array shape: `1:0` → `1×0 empty double row vector`, plus MATLAB-style
     empty display text.
  4. Item 11 — format engine: `format short` 4-decimal rule (`0.1000` not `0.1`), `1.0e+03 *`
     scale factor, `format long` / `short e` / `long e` skeleton; keep FB-1 layout untouched.
  5. Lexer: block comments `%{ ... %}`.
- **DoD:**
  - [ ] Each item has a MATLAB-parity test; oracle = user's real MATLAB output when
        available, else Octave 9.4 (oracle recorded per case in COMPATIBILITY §4.8).
  - [ ] FB-1 display golden tests all still green (regression guard).
  - [ ] ≥ 310 tests green.
- **Tests:** display goldens per item; fprintf format-string table tests; corpus re-run.
- **Risks:** item 11 rewrites the core formatter (touches everything) → full corpus +
  display goldens gate the commit; per-item commits so any single item is revertable.
- **Deliverable:** commits + package.
- **Stop / rollback:** per-item commit series; revert any item independently.

### Step 2.5 — Robustness: memory budget + panic audit (audit items 2 and 15 closure)
- **Goal:** the engine never kills the process on user input; oversized allocations produce
  a clean MATLAB-like error instead of abort (SIGABRT exit 134).
- **Tasks:**
  1. Allocation budget in mc-core matrix constructors (configurable constant, device-aware
     default documented in one place); error message in MATLAB's spirit ("Out of memory").
  2. Guard range materialization (`x=1:1e9`) at range→matrix conversion — same budget.
  3. Panic audit: extend CI grep-guard to all non-test engine paths; fix every reachable
     `panic!/unwrap/expect`; close the item-15 decision if deferred from 2.3.
- **DoD:**
  - [ ] `zeros(100000,100000);` → clean error, `mc -e` exit code 1, process alive.
  - [ ] `x=1:1e9` → clean error, process alive.
  - [ ] ≥ 315 tests green; deny / clippy green.
- **Tests:** budget-boundary unit tests; repro tests for both audit items.
- **Risks:** budget too low blocks legitimate large matrices → configurable + documented;
  per-allocation check cost is one branch (negligible).
- **Deliverable:** commits + package.
- **Stop / rollback:** the budget is a single constant — trivial revert.

### Step 2.6 — Numeric extras & structured options (STATUS §3.1 leftovers)
- **Goal:** close the documented P2 deviations so COMPATIBILITY §4.5 shrinks before phase close.
- **Tasks:**
  1. Multi-output `[Q,R]=qr` / `[U,S,V]=svd` via existing `lookup_multi`.
  2. `interp2` linear + nearest (spline 2-D explicitly out of scope for v1 — expansion E7).
  3. `histogram` with MATLAB auto-binning rule.
  4. `integral` named option `Waypoints`.
  5. Pin the van der Pol μ=1000 **point reference** into the corpus. ✅ Executed differently
     than planned — no real MATLAB exists on the user side (chat: «من متلب واقعی ندارم خودت
     کارا رو درست کن»): pinned from scipy 1.14.1 stiff-solver triangulation (Radau
     rtol=1e-10/atol=1e-13 + BDF/LSODA rtol=1e-8, spread ≤ 2.3e-7;
     `test-corpus/references/p2_13_vanderpol_mu1000.json`); Octave ode45 measured impractical
     (probe [0,300] > 5 min). Re-verification pattern = re-run the reference script on scipy
     upgrades.
  6. `odeset` structured form: blocked on `Value::Struct` (step 2.9) — named pairs remain,
     deviation stays documented until then.
- **DoD:**
  - [x] All items in corpus `p2_*` golden with recorded references; ≥ 325 tests green.
  - [x] COMPATIBILITY §4.5 deviation table updated (shrunk); vdp1000 point reference pinned.
- **Tests:** golden corpus entries with Octave/analytic/user-MATLAB references.
- **Risks:** user-MATLAB reference requires user cooperation — schedule explicitly, do not
  fake it; interp2 grid validation edge cases.
- **Deliverable:** commits + package.
- **Stop / rollback:** independent feature commits; corpus manifest additive.

### Step 2.7 — Phase 2 close (numeric milestone gate)
- **Goal:** formal docs/07 §P2 milestone close — nothing "almost done".
- **DoD (all must hold):**
  - [x] van der Pol μ=1000 matches the pinned point reference; `rng(42)` reproducible.
  - [x] Cancel mid-ode45 leaves a consistent workspace (step 2.3, device-verified).
  - [x] Pendulum demo on device: runs in < 1 s with a progress bar (docs/07 §P2 demo) —
        user-confirmed 2026-11-05 (STATUS §3.9).
  - [x] FB-1/FB-2 final touch verification by user (carried from P2 start) — user-confirmed
        2026-11-05 (STATUS §3.9).
  - [x] STATUS + COMPATIBILITY updated; tag + package: tag `p2-code-complete` pushed by the
        user at `e189195` (CI #18 + Android #18 green, 2026-11-01); package of record =
        `MathCraft-P2-v0.9-phase2-closed-WITH-GIT.tar.gz`; phase declared **CLOSED** in §11.
- **Stop / rollback:** if device confirmation is unavailable, the phase stays
  "code-complete, pending device" (same as P1 did) — never silently closed.

---

## 4. Sub-phase P2-c — Language completeness (steps 2.8–2.11)

> Runs after 2.7, still inside milestone P2's umbrella. Closes the audit ledger's
> data-model items so plotting/CAS/IDE start from a complete language core.

### Step 2.8 — Char type & strings (audit items 6 and 12)
- **Goal:** 1×N character arrays as first-class values; `[65 'a']` → `'Aa'`, `'abc'+1` →
  `[98 99 100]`, correct display, `char`/`double`/`ischar`/`num2str` interplay.
- **Tasks:**
  1. Data model: `Value::Char` (1×N row char arrays for v1; 2-D char arrays → expansion E4,
     honest error otherwise).
  2. Lexer/parser: single-quote char literal vs transpose disambiguation using MATLAB's
     context rule (quote after operand = transpose; in operand position = char literal) —
     adversarial tests mandatory.
  3. Semantics: concat with numerics promotes to char codes; `'abc' + 1` numeric promotion;
     transpose of char; display of char values in console (MATLAB style).
  4. Builtins: `char`, `double`, `ischar`, `blanks`, `str2double` (minimal set; full string
     function library = expansion E4).
- **DoD:**
  - [x] Audit repro tests for items 6 and 12 green (tests/chars_step28.rs, 2026-11-13).
  - [x] ≥ 20 new tests; corpus extended (char cases with Octave reference) — 44 new tests
        (40 in chars_step28 + lexer adversarial suite + display golden + corpus p2_21/p2_22);
        manifest 74 → 76 cases, expected/ regenerated with the Octave 10.3 sandbox oracle;
        460/460 workspace green (was 416).
  - [x] COMPATIBILITY §1 rows updated (char semantics rows + documented deviations:
        out-of-range char() errors vs MATLAB clamping — pin flag; "…" behaves as char,
        string scalars = E4; sort-on-char = E4).
- **Tests:** lexer disambiguation adversarial suite; arithmetic/display goldens.
- **Risks:** quote/transpose ambiguity is the classic MATLAB lexer trap → rule + tests first,
  then code; `Value` enum widening touches every match site (compiler-guided sweep).
- **Deliverable:** commits + package.
- **Stop / rollback:** one `Value` variant + formatter branch; per-commit revert.

### Step 2.9 — Structs minimal (`Value::Struct`)
- **Goal:** named-field containers unlock structured options (`odeset`), named results, and
  later persistence (P5); closes the groundwork for the `rng(seed)` struct deviation.
- **Tasks:**
  1. `Value::Struct` (ordered field map) + syntax `s.a`, `s.a = v`, dynamic fieldnames.
  2. Builtins: `struct`, `fieldnames`, `isfield`, `getfield`, `setfield` (minimal set).
  3. Console display in MATLAB struct style (field list with sizes).
  4. Switch `odeset/ode45/ode23` options to accept struct form; named pairs stay accepted.
     `rng(seed)` returns a status struct like MATLAB (closes §4.5 deviation).
- **DoD:**
  - [x] `opts = odeset('RelTol',1e-6); [T,Y]=ode45(f,tspan,y0,opts);` works; named-pair form
        still works (regression) — executed 2026-11-17: struct form is bit-identical to the
        named-pair form (invariance test, max|ΔY| = 0); odeset() defaults / pairs /
        odeset(old, ...) merge with fresh-pairs-win precedence; `[]` = reset-to-default.
  - [x] ≥ 15 new tests — 41 delivered: 39 in tests/structs_step29.rs + corpus p2_23
        (Octave-compared, manifest 76 → 77) + p2_24 (engine-internal, outside the Octave
        manifest — same precedent as vdp1000/RNG); 501/501 workspace green (was 460).
- **Tests:** struct round-trips; option-precedence tests (struct vs named pairs).
- **Risks:** scope creep toward full OO — explicit non-goal: **no classes/objects in v1**
  (honest "not supported" error); display edge cases.
- **Deliverable:** commits + package.
- **Stop / rollback:** additive Value variant; revert per commit.

### Step 2.10 — Chained indexing (audit item 13)
- **Goal:** `[1 2 3](2)`, `ones(3,3)(2,2)`, `f(x)(1)` evaluate like MATLAB.
- **Tasks:**
  1. Parser: postfix indexing on call/expression results (index node over `Expr::Call`).
  2. Eval: generalize index application to temporary values; `x(1)(2)` (cell content) →
     honest "cell arrays not supported yet" error (cells = expansion E4).
- **DoD:**
  - [x] Audit item 13 repro tests green — `[1 2 3](2)`, `ones(3,3)(2,2)`, `f(x)(1)`
        plus `end`/ranges/char literals/field chains all green against the Octave 10.3
        oracle; ≥ 8 new tests — 14 adversarial tests in tests/chained_step210.rs
        (written before code per the step rule) + corpus p2_25 (manifest 77 → 78);
        516 workspace green (was 501).
- **Tests:** precedence interactions with transpose/imag suffixes (adversarial).
- **Risks:** parser precedence traps → tests before code, per project rule.
- **Deliverable:** commit + package.
- **Stop / rollback:** single parser/eval area — isolated revert.

### Step 2.11 — Diagnostics workshop (audit item 14)
- **Goal:** structured error taxonomy actually enforced: distinct parse vs eval error IDs,
  real `line:col` positions, no more `E3001` for semantic errors, no `line 0:0`.
- **Tasks:**
  1. Carry spans through the AST (parser already tracks positions — extend systematically).
  2. Map error kinds to the MC:E* catalog of docs/03; FFI error DTO carries id + span;
     UI error card shows position.
  3. Lexer recovery (continue after first error): default **deferred** (stop-at-first stays
     for v1, documented) — revisit only if cheap during the span work.
- **DoD:**
  - [x] Audit item 14 repro cases produce distinct IDs with correct `line:col` —
        E3001 now syntax-only (lexer/parser); semantic eval errors carry their
        catalog family (E1002 undefined fn · E2001 bounds · E2002 type/usage —
        new family, level-2 of doc 02 §9 · E2003 dims · E9xxx boundaries);
        position backstops (expression/statement/call-site) mean no user error
        keeps `line 0:0` (sole documented exception: contained FFI panics).
        11 tests in tests/diagnostics_step211.rs; 527 workspace green (was 516).
  - [x] Error-catalog table updated: doc 03 §14 rows E1002/E2001/E2002 added
        (+E2004/E2005 pre-existing gaps surfaced by the new catalog test);
        COMPATIBILITY audit row 14 closed. ≥ 10 new tests ✓ (11).
- **Tests:** span correctness across multi-line inputs; catalog completeness check.
- **Risks:** span plumbing is wide but mechanical — schedule immediately after 2.10 while
  parser context is warm.
- **Deliverable:** commit + package; **P2 (numeric + language) fully closed**.

---

## 5. Phase 3 — Plotting 2D, then 3D (docs/07 §P3)

### Step 3.1 — PlotSpec contract freeze
- **Goal:** the plot data contract becomes versioned reality before any UI work (docs/07
  critical dependency #2).
- **Tasks:**
  1. Finalize mc-plot: `PlotSpec` / axes / series / markers / line specs (`'r--'`, `'b*'`,
     `'k.'`, …) per docs/04 §8; `PLOT_CONTRACT_VERSION = 1`.
  2. LTTB decimation module with quality test (docs/07: area error < 1%).
  3. Architecture decision (docs/02 §7, already made): Rust computes draw-lists; Flutter
     renders via CustomPainter — **no FFI calls in the frame loop**.
- **DoD:** [x] contract v2 committed with tests (52 mc-plot unit tests; v1 = P0 bridge, superseded by the freeze); [x] LTTB quality test green (area-error < 1% on chirp/gaussian/sinc @ 200k→4k).
- **Deliverable:** commits. **Risks:** none serious — pure data model. **Stop/rollback:** additive crate work.

### Step 3.2 — Figure state + 2-D builtins
- **Goal:** engine-side figure/axes/hold state and the 2-D plotting registry.
- **Tasks:** `figure`, `hold on/off` (**closes audit item 9's hold-on part**), `plot`,
  `scatter`, `bar`, `histogram`, `area`, `fill`, `polarplot`, `errorbar`, `fplot`,
  `subplot`, `legend`, `axis`, `grid`, `xlabel/ylabel/title`; MATLAB line-spec parser.
- **DoD:** [x] 33 builtin-level tests (plot_builtins.rs); [x] line-spec parity table vs MATLAB (linespec.rs parity_table_matlab_doc04).
- **Risks:** hold/figure state model — keep one active figure + axes list (MATLAB-lite,
  documented); full figure management = expansion.

### Step 3.3 — FFI transport
- **Goal:** versioned retrieval of the figure list (pull model per docs/02 §8).
- **Tasks:** `get_plots()` DTO extension; contract bump; glue freshness gate.
- **DoD:** [x] round-trip tests (mc-ffi api: push/pull/hold/error-survival, 5 tests); [x] mc-cli unaffected (13/13 green — statement fns are echo-free like disp/rng).
- **Risks:** DTO size on big series → decimation applied engine-side before transport.

### Step 3.4 — Flutter plot viewer
- **Goal:** interactive plotting on device, Craft-look, RTL-aware.
- **Tasks:** plot page; zoom/pan/crosshair; export PNG + share; CDS palette tokens; LTR
  island inside RTL layout (docs/06); Craft Icons only.
- **DoD:** [x] widget tests (30 new: g4-format mirror table, palette/two-layer
  resolve, pure layout+inverse, snap binary/scan, plots state merge, page
  empty/3D-honest/list-toggle, crosshair long-hold callback — 37 total green,
  analyze clean on the pinned 3.24.5); [ ] device verification by user (demo:
  crosshair on `plot(sin(x))`) — pending this push + APK build.
- **Risks:** rendering perf on low-end → decimation + draw lists (already designed).

### Step 3.5 — CI for plots
- **Goal:** plotting regressions caught automatically.
- **Tasks:** golden-image tests with a fixed software renderer (correctness only);
  performance budget test: 3 series × 100k points → first frame ≤ 50 ms (docs/07 §P3).
- **DoD:** [ ] golden images in CI; [ ] LTTB perf test green.

### Step 3.6 — P3-b: 3D (plot3/surf/mesh/contour/contourf/colorbar/view)
- **Goal:** 3D surfaces with smooth touch rotation; PDF export.
- **Tasks:** 3D series/mesh draw-lists; touch rotate (60 fps budget on reference device);
  PDF export; raster fallback if GPU budget fails on low-end devices (decision point,
  documented).
- **DoD (docs/07 §P3):** [ ] 50k triangles @ 60 fps on reference device; [ ] no jank in a
  10-minute test; [ ] demo `surf(peaks)` with finger rotation.
- **Deliverable:** package `…v0.6/v0.7…`; **Phase 3 close** (device-verified demo).

---

## 6. Phase 4 — Symbolic CAS (docs/07 §P4; docs/05 §11)

> Largest scope risk of the whole project (risk R1). Sub-milestones are hard boundaries:
> each one is independently shippable, and "full Risch" is an explicit non-goal.

### Step 4.1 — CAS-0: foundations
- **Goal:** symbolic values exist in the same language; no second parser (docs/07 critical
  dependency #4 — CAS reads the shared AST).
- **Tasks:**
  1. `Sym` expression tree + exact arithmetic on `rug`/GMP (mc-cas crate skeleton exists
     since P0).
  2. `sym` / `syms` declarations; `double(sym)` and `sym(double)` bridges; symbolic values
     as a new `Value` variant.
  3. Honest numeric fallback everywhere: any symbolic op that cannot complete returns a
     clear error, never a wrong answer.
- **DoD:** [ ] `syms x; y = x^2 + 3*x; double(subs(y, x, 2.5))` round-trips exactly;
  [ ] ≥ 15 tests.
- **Risks:** rug/GMP build weight on Android (measure; static-link decision documented);
  Value-variant sweep (same as 2.8).
- **Deliverable:** commits + package. **Stop/rollback:** mc-cas is additive; revert per commit.

### Step 4.2 — CAS-1 (docs/05 §11): exact arithmetic, `simplify/expand/collect`, full `diff`, `subs`, LaTeX rendering
- **DoD:** docs/05 §11 CAS-1 criteria table green + diff-verify self-check active;
  demo: `diff(exp(-x^2)*x^3)` with beautiful LaTeX output.
- **Risks:** simplification scope creep → auto-simplify = level-1 normal forms only (docs/05
  §3); egg e-graphs optional behind a feature flag — if it slips, ship normal-form only.

### Step 4.3 — CAS-2: `int` pipeline (Risch-lite 8 stages, docs/05 §5–6), `solve` (polynomial + transcendental subset), symbolic linear systems
- **DoD:** docs/05 §11 CAS-2 table green; **release-worthy standalone gate** ("usable
  product even without CAS-3"); demo: `int(exp(-x^2)*x^3)` rendered in LaTeX.
- **Risks:** R1 again — the int pipeline stops at the documented stage list; every
  non-integrable form → honest "cannot integrate" (never wrong); timeout with clear
  message (R6).

### Step 4.4 — CAS-3: `limit`, `taylor`, `dsolve` subset, `vpa/assume`, `factor`
- **DoD:** docs/05 §11 CAS-3 table green; demo `dsolve(D2y + y = 0)`.
- **Deliverable:** **Phase 4 close.** package `…v0.8…`.

---

## 7. Phase 5 — IDE Experience (docs/07 §P5; docs/06 is absolute)

### Step 5.1 — Math keyboard (docs/06 §6, 4 pages)
- **DoD:** [ ] all 4 pages with Craft Icons; [ ] haptics + CDS pressables; [ ] widget tests.
- **Risks:** R8 (UI quality) — every screen reviewed against docs/06 anti-patterns before merge.

### Step 5.2 — Workspace browser + variable inspector
- **Goal:** see and edit workspace variables like MATLAB's Workspace panel.
- **Tasks:** variable list (name/class/size/value preview); matrix card viewer (docs/06);
  edit-in-place for scalars and small matrices; virtualized rendering for large arrays.
- **DoD:** [ ] 1M-cell matrix scrolls smoothly (docs/07 §P5); [ ] widget tests + device check.
- **Risks:** memory vs preview — never materialize full arrays in Dart (windowed reads via FFI).

### Step 5.3 — Editor pro: autocomplete + live gutter + snippets
- **Tasks:** registry-aware completion (builtins + user functions + variables); gutter
  diagnostics powered by step 2.11 spans; snippet gallery (docs/06).
- **DoD:** [ ] `lenght` → suggests `length` inline; [ ] analyze/test green.

### Step 5.4 — Help system
- **Tasks:** bilingual doc for the full registry with full-text search + "run example"
  button; `help` / `doc` builtins in the engine.
- **DoD:** [ ] every registry function documented and searchable (docs/07 §P5).
- **Risks:** doc volume (~230+ functions) → generated stubs from the registry + curated
  priority order; never shipped as empty stubs (grep-guard).

### Step 5.5 — Files & persistence
- **Tasks:** `save`/`load` workspace — MCWS format v1 (own, versioned, documented);
  `.mat` v5 **read** subset (numeric/char/struct arrays — documented boundary); CSV
  read/write; autosave + session recovery; script folder browsing (SAF already from P1).
- **DoD:** [ ] round-trip save/load tests incl. structs/complex; [ ] `.mat` subset parity
  tests vs Octave-written files; [ ] crash-kill recovery works.
- **Risks:** .mat v7.3 (HDF5) explicitly out of scope (expansion E6) — honest error.

### Step 5.6 — Polish & accessibility
- **Tasks:** onboarding/tour (docs/06 §9), final theme polish (both modes), accessibility
  pass, dynamic font 130%, both orientations, i18n completeness (FA/EN).
- **DoD (docs/07 §P5):** [ ] every screen, both orientations, 130% font; [ ] **usability
  demo: a first-time user draws their first plot within 5 minutes, unassisted** (5-person test).
- **Deliverable:** package `…v0.9…`; **Phase 5 close.**

---

## 8. Phase 6 — Beta & Stabilization → v1.0 (docs/07 §P6)

### Step 6.1 — Scale the corpus + budgets
- **Tasks:** expand golden corpus 73 → 200 scripts (docs/08 target distribution);
  property + fuzz suites hardened; measure docs/01 NFR budgets on 4 device profiles;
  memory/battery pass.
- **DoD:** [ ] 200/200 corpus green in CI; [ ] NFR budget report committed.

### Step 6.2 — Release engineering
- **Tasks:** ABI split; APK ≤ 45 MB; **reproducible build from tag** with pinned lockfiles;
  GitHub Releases pipeline; F-Droid (reproducible-build) preparation; About/licenses page
  (GPL-3.0 attribution set).
- **DoD:** [ ] tag → bit-identical APK twice; [ ] release job green.

### Step 6.3 — Open beta
- **Tasks:** distribution to 100+ real users; feedback channel (Telegram/GitHub issues);
  crash reporting triage; P0/P1 bug burn-down.
- **DoD:** [ ] crash-free ≥ 99.5% over 4 consecutive weeks (docs/07 §P6); [ ] zero open
  P0/P1 bugs.

### v1.0 — Public release close
- **DoD (docs/07 §5 Global DoD, audited item by item):** all suites green in release CI;
  reproducible tag build; docs in sync (final COMPATIBILITY coverage matrix + user manual);
  manual 4-device-profile checklist signed (both orientations, both languages, both
  themes); failure behavior reviewed (no silent wrong answers); final tag + F-Droid
  submission.
- **Deliverable:** `v1.0` tag + release artifacts; **program complete.**

---

## 9. Post-v1.0 expansion phases — named only (per user instruction, 2026-10-25)

> No step-level planning is written here, by design. Each phase below gets its own
> detailed ROADMAP addendum **only when it starts** (change-controlled like everything else).

| Phase | Theme |
|---|---|
| E1 | Desktop & iOS (Flutter Desktop / iOS on the existing engine — docs/07 v1.0 note) |
| E2 | FFT & signal processing (`fft/filter/conv2/spectrogram`) |
| E3 | Sparse matrices & large-scale linear algebra |
| E4 | Full containers: 2-D char arrays, cell arrays, tables, timetables |
| E5 | Image processing basics (`imread/imwrite/filter`) |
| E6 | Full MATLAB file interop (.mat v7.3/HDF5, Excel) |
| E7 | Advanced numerics: 2-D spline interpolation, stiff/BVP solvers, optimization toolbox subset |
| E8 | Symbolic deepening beyond CAS-3 (full Risch, polynomial systems) |

---

## 10. External-audit cross-map (15 items → current disposition)

| # | Item | Status | Step |
|---|---|---|---|
| 1 | `x(2)=[]` panic | ✅ fixed (batch 1) | — |
| 2 | OOM abort (`zeros(1e10)`, `1:1e9`) | ✅ closed (2.5) | — |
| 3 | bare-var echo `x = 5` + `ans` | ✅ fixed (batch 1.5) | — |
| 4 | indexed-assign display | ✅ fixed (batch 1) | — |
| 5 | index orientation | ✅ closed (2.4) | — |
| 6/12 | char/string model (`[65 'a']`, `'abc'+1`) | 🔶 open | 2.8 |
| 7 | `[]`/array conditions | ✅ fixed (batch 1) | — |
| 8 | `disp/fprintf/class` | ✅ closed (2.4) | — |
| 9 | `hold on` / block comments | 🔶 hold on → 3.2 · block comments ✅ closed (2.4) | 3.2 |
| 10 | empty shape `1:0` display | ✅ closed (2.4) | — |
| 11 | format short decimals + scale factor | ✅ closed (2.4) | — |
| 13 | chained indexing | 🔶 open | 2.10 |
| 14 | error IDs + positions | 🔶 open | 2.11 |
| 15 | panic across FFI boundary | ✅ closed (2.3 Option A + 2.5 panic audit) | — |

## 11. Current focus

**Step 3.4 reopened by user verdict and re-delivered as 3.4.1 (2026-11-18):**
the 3.4 viewer shipped a canvas-transform zoom (data scaled under a clip while
ticks/labels stayed fixed), arbitrary pan clamps, and a hidden hold-gesture —
"زوم افتضاح، مقیاس افتضاح، اطلاعات تقریباً هیچ". The rework replaces the
architecture, not the paint: a **data-space viewport** (`PlotViewport`) is the
single source of truth for zoom/pan; ticks are generated **in Dart for the
current viewport every frame** (faithful port of `mc_plot::ticks` — 1-2-5
ladder, `%.4g` labels, log decades `10^{k}`; pixel-adaptive tick count), so the
scale always matches what you see. Gestures: drag = pan in data units (content
follows the finger, center free like MATLAB), pinch = incremental zoom around
the data point under the focal, double-tap = home, span clamped to
home/10000..home×4. Info layers: **tap plants a persistent data tip** with
series name + (x, y); hold 500ms = live crosshair with value tags on both axes;
and an always-visible status chip `x: [a, b]  y: [c, d]`. Porting found one real
bug: Dart has no log10 and `ln/ln10` drifts on exact powers of ten (1000 →
2.999…6) — snapped to exact powers, mirrored test table green. Also: single-tap
is instant now (raw double-tap detection in the Listener instead of the
double-tap recognizer, which delays every single tap 300 ms).

**Pending for 3.4.1 close:** user push → CI → APK → device check: zoom feels
MATLAB-like (labels re-tick), pan natural, tap-tip discoverable, status chip
readable, crosshair tags accurate. Then 3.5 (CI golden images + perf budget),
then P3-b (3D, `surf(peaks)` demo).

## 12. Change log

| Doc version | Date | Change | Approved by |
|---|---|---|---|
| 1.0 | 2026-10-25 | Initial binding plan: baseline P0/P1/P2 recorded; steps 2.3–2.11, 3.1–3.6, 4.1–4.4, 5.1–5.6, 6.1–6.3, v1.0 close defined; expansion E1–E8 named; audit cross-map added | User (scope answers, this session) |
| 1.0 | 2026-10-28 | Step 2.4 executed per plan (§2.4 tasks 1-5): ledger rows 5/8/10/11 → closed, row 9 split (block comments closed, hold on → 3.2); §11 updated. User approval: «اوکی بود. برو مرحله بعدی» | User (chat) |
| 1.0 | 2026-10-28 | Step 2.3 closed (user confirmed push + device demo) and Step 2.5 executed per plan (§2.5 tasks 1-3): ledger rows 2 and 15 → closed; doc 03 §14 row MC:E4003 added; §11 updated. User approval: «همه چیز مورد انتظار و موفق بود. من مشکلی ندیدم. میتونیم بریم گام بعدی» | User (chat) |
| 1.0 | 2026-10-29 | Step 2.6 executed per plan (§2.6 tasks 1-4 + corpus; task 6 stays blocked on 2.9): multi-output qr/svd, interp2 (linear/nearest), histcounts/histogram numeric core (documented 'auto' contract + pin flag), integral RelTol/AbsTol/Waypoints; tril/triu k-sign compat fix (rule R2 regressions); COMPATIBILITY §4.5 shrunk (integral/qr-svd rows closed); §11 updated. Task 5 (vdp1000 pin) = pending user MATLAB run. User approval: «خب در گام ۲.۵ همه چیز سبز بود. بریم گام ۲.۶» | User (chat) |
| 1.0 | 2026-11-01 | Task 5 of 2.6 closed without real MATLAB (user chat: «من متلب واقعی ندارم خودت کارا رو درست کن. برو گام بعد.»): vdp1000 pinned from scipy 1.14.1 stiff-solver triangulation (Radau/BDF/LSODA, spread ≤ 2.3e-7; Octave measured impractical — probe [0,300] > 5 min); corpus test qualitative → point (engine tol y1 5e-3 / y2 2e-5 / ymax 1e-3) with JSON-sync + triangle-coherence guards; step 2.7 = code-complete, pending device (pendulum < 1 s + FB-1/FB-2 touch); §11 rewritten. First delivery lost to a sandbox reset — fully re-derived and re-verified (≤ 7.9e-11) before re-delivery | User (chat) |
| 1.0 | 2026-11-05 | Phase 2 formally closed at step 2.7: user pushed annotated tag `p2-code-complete` at `e189195` (CI #18 + Android #18 green) and confirmed the full device checklist (pendulum ode45 demo < 1 s + progress bar, FB-1, FB-2); §2.7 DoD all ticked; §11 rewritten (Phase 2 CLOSED, next = P2-c step 2.8); STATUS §3.9 + phase map + FB rows updated; package v0.9 | User (chat: «خب الان دستور ها رو اجرا کردم و همچنین هرسه مورد درست کار میکردند») |
| 1.0 | 2026-11-13 | Step 2.8 executed per plan (§4 tasks 1-4): char type & strings — mixed concat promotion (`[65 'a']`→`'Aa'`), numeric promotion (real+complex), char indexing + indexed assignment (class-of-target wins), honest E4 errors for 2-D char, builtins char/double/ischar/blanks/str2double, MATLAB no-space quote/transpose lexer rule; corpus p2_21/p2_22 (Octave 10.3 sandbox oracle; manifest 74→76); 460 tests green; COMPATIBILITY ledger rows 6/12 closed. User approval: «خب اوکیه. بریم ادامه.» | User (chat) |
| 1.0 | 2026-11-17 | Step 2.9 executed per plan (§4 tasks 1-4): structs minimal — `Value::Struct` (ordered field map, 1×1; struct arrays = E4 honest error), syntax `s.f` / `s.f = v` / nested auto-viv / dynamic `s.(expr)`, builtins struct/fieldnames/isfield/getfield/setfield, MATLAB-style struct display, `odeset` + struct-form ode options (pair form unchanged; merge precedence via odeset(old,...)), `rng` status struct {Type:'PCG64',Seed,State} with capture/restore replay (closes §4.5 deviation), rng statement form echo-free; depth guard 256; corpus p2_23 (manifest 76→77) + p2_24 (engine-internal, outside Octave manifest); 501 tests green (was 460). User approval: «خب اوکی بریم ادامه» | User (chat) |
| 1.0 | 2026-11-17 | Step 2.9 CLOSED (user pushed 2620328 = tree bac9f15f ✓, CI green, full device checklist confirmed). Step 2.10 executed per plan (§4 tasks 1-2): chained indexing — `Expr::Index` postfix on any expression result (literals/calls/anon/parens/transpose chains/char literals/matrix-literal elements), `s.f(2)` read+write incl. nested + auto-viv from empty, cell idiom `x(1)(2)` = honest E4 (MATLAB rejects; Octave chains freely = documented delta); adversarial suite written before code (chained_step210, 14 tests); corpus p2_25 (manifest 77→78, Octave 10.3 oracle); 516 tests green (was 501). **Governance note (user):** Python/other languages acceptable as *tools* (product stays Rust-in-APK); multiple short steps may be batched per turn at the assistant's judgment — 2.11 deliberately deferred to a dedicated session (span+FFI+UI work). User approval: «خب بیلد شد. چک لیست رو تست کردم اوکی بود…» | User (chat) |
| 1.0 | 2026-11-17 | Step 2.10 CLOSED (user pushed 21a7b59 = tree ef961d00 ✓, CI green, in-app tests confirmed). Step 2.11 executed per plan (§4 tasks 1-3) — **P2-c code-complete; with it P2 (numeric + language) fully closed pending this delivery's push/CI/checklist**: E3001 → syntax-only; 90 semantic sites reclassified (E1002 undefined fn with did-you-mean, E2001 bounds, E2002 new type/usage family, E2003 dims, E9xxx boundaries); AST spans for Unary/Range/Logic; three position backstops → no user error keeps line 0:0; UI error card shows the position (Dart-only, l10n en/fa, gen-l10n-verified, flutter analyze clean — FFI DTO already carried id+span); doc 03 §14 rows E1002/E2001/E2002 (+E2004/E2005 pre-existing gaps); lexer recovery stays deferred; 11 diagnostics tests; 527 green (was 516). Same delivery: user's logo → Android launcher icon (overlay over flutter-create template, verified against the real 3.24.5 template). User approval: «بله بیلد کامل شد. تست های داخل برنامه موفق بود. بریم گام بعد.» | User (chat) |
| 1.0 | 2026-11-17 | Steps 3.1 + 3.2 + 3.3 executed in one turn (user-approved batching): **PlotSpec contract v2 frozen** (PlotColor palette/exact-RGB, Rust-side 1-2-5 ticks + %.4g, Area/Fill/ErrorBar, Bar series-mode, HistNorm, series id+raw_len for zoom re-decimation, caxis/colorbar reserved, PlotFigure + get_plots pull; PLOT_CONTRACT_VERSION=2); mc-plot v2 (ops grammar + FigureState fold + LTTB ≤4000 with <1% area-error quality test + linespec parser + histogram binning, 52 tests); 17 statement plot builtins in mc-interp (disp/rng pattern — no echo/no ans; expression use = clear error), hold/figure/subplot/legend/axis/grid/labels/fplot(400 samples); **histogram data-form retired for MATLAB parity — Bin* options merged into histcounts (COMPATIBILITY §4.5 closed)**, corpus p2_20 migrated; FFI: Engine.figure_state + push/pull plots + 5 round-trip tests; FRB glue regenerated (6 files, byte-stable re-run) + flutter analyze clean; 617 tests green (was 527). Next per §11: 3.4 Flutter plot viewer | User (chat: «بریم سراغ فاز ۳… بدون افت کیفیت در یک نوبت») |
| 1.0 | 2026-11-17 | Step 3.4 executed per plan (§7.3 tasks): Plots tab live — pull `get_plots()` on tab activation + push-merge of `EvalResult.plots`; CustomPainter for contract v2 (axes/grid/ticks incl. `10^{k}` superscripts/line-scatter-bar-histogram-area-fill-polar-errorbar/legend chips/subplot grid); gestures pinch-around-focal + pan + double-tap reset + hold-500ms crosshair (raw Listener+Timer — scale recognizer never wins the arena on a stationary press); PNG export + share (path_provider + share_plus 10.x, Craft-Icons floating bar); LTR island; honest 3D/Quiver state; format_g4 Dart mirror with manual round-half-even (Dart toStringAsFixed is half-away — divergence from Rust/C caught by the mirrored table test); 30 new tests (37 total, analyze clean). Device verification pending user push | User (chat: «بریم گام بعدی. Ci سبز شد.») |
| 1.0 | 2026-11-18 | Step 3.4 reopened by user verdict («بدترین چیزی که اجرا شده» — canvas-transform zoom, fixed ticks, hidden hold-gesture) and re-delivered as 3.4.1: data-space PlotViewport as single source of truth; Dart-side tick generation per viewport (faithful mc_plot::ticks port + exact-log10 fix + pixel-adaptive count) so scale always matches view; natural pan in data units; pinch-around-focal incremental zoom; span clamps home/10000..×4; instant-tap data tip with series name; hold-crosshair with axis value tags; always-on viewport status chip; raw double-tap in Listener (single tap no longer 300ms-delayed); 18 new tests (55 total green, analyze clean) | User (chat: بازخورد کیفیت نمودار) |
