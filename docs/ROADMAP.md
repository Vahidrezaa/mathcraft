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
  5. Pin the van der Pol μ=1000 **point reference** obtained from the user's real MATLAB
     (one device run — user action item) into the corpus; Octave stays too slow for CI.
  6. `odeset` structured form: blocked on `Value::Struct` (step 2.9) — named pairs remain,
     deviation stays documented until then.
- **DoD:**
  - [ ] All items in corpus `p2_*` golden with recorded references; ≥ 325 tests green.
  - [ ] COMPATIBILITY §4.5 deviation table updated (shrunk); vdp1000 point reference pinned.
- **Tests:** golden corpus entries with Octave/analytic/user-MATLAB references.
- **Risks:** user-MATLAB reference requires user cooperation — schedule explicitly, do not
  fake it; interp2 grid validation edge cases.
- **Deliverable:** commits + package.
- **Stop / rollback:** independent feature commits; corpus manifest additive.

### Step 2.7 — Phase 2 close (numeric milestone gate)
- **Goal:** formal docs/07 §P2 milestone close — nothing "almost done".
- **DoD (all must hold):**
  - [ ] van der Pol μ=1000 matches the pinned point reference; `rng(42)` reproducible.
  - [ ] Cancel mid-ode45 leaves a consistent workspace (step 2.3, device-verified).
  - [ ] Pendulum demo on device: runs in < 1 s with a progress bar (docs/07 §P2 demo).
  - [ ] FB-1/FB-2 final touch verification by user (carried from P2 start) — confirmed here.
  - [ ] STATUS + COMPATIBILITY updated; tag + package; phase declared closed in this file's
        Current Focus section.
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
  - [ ] Audit repro tests for items 6 and 12 green.
  - [ ] ≥ 20 new tests; corpus extended (char cases with Octave reference).
  - [ ] COMPATIBILITY §1 rows updated.
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
  - [ ] `opts = odeset('RelTol',1e-6); [T,Y]=ode45(f,tspan,y0,opts);` works; named-pair form
        still works (regression).
  - [ ] ≥ 15 new tests.
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
  - [ ] Audit item 13 repro tests green; ≥ 8 new tests.
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
  - [ ] Audit item 14 repro cases produce distinct IDs with correct `line:col`.
  - [ ] Error-catalog table in COMPATIBILITY updated; ≥ 10 new tests.
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
- **DoD:** [ ] contract v1 committed with tests; [ ] LTTB quality test green.
- **Deliverable:** commits. **Risks:** none serious — pure data model. **Stop/rollback:** additive crate work.

### Step 3.2 — Figure state + 2-D builtins
- **Goal:** engine-side figure/axes/hold state and the 2-D plotting registry.
- **Tasks:** `figure`, `hold on/off` (**closes audit item 9's hold-on part**), `plot`,
  `scatter`, `bar`, `histogram`, `area`, `fill`, `polarplot`, `errorbar`, `fplot`,
  `subplot`, `legend`, `axis`, `grid`, `xlabel/ylabel/title`; MATLAB line-spec parser.
- **DoD:** [ ] ≥ 25 builtin-level tests; [ ] line-spec parity table vs MATLAB.
- **Risks:** hold/figure state model — keep one active figure + axes list (MATLAB-lite,
  documented); full figure management = expansion.

### Step 3.3 — FFI transport
- **Goal:** versioned retrieval of the figure list (pull model per docs/02 §8).
- **Tasks:** `get_plots()` DTO extension; contract bump; glue freshness gate.
- **DoD:** [ ] round-trip tests; [ ] mc-cli unaffected.
- **Risks:** DTO size on big series → decimation applied engine-side before transport.

### Step 3.4 — Flutter plot viewer
- **Goal:** interactive plotting on device, Craft-look, RTL-aware.
- **Tasks:** plot page; zoom/pan/crosshair; export PNG + share; CDS palette tokens; LTR
  island inside RTL layout (docs/06); Craft Icons only.
- **DoD:** [ ] widget tests; [ ] device verification by user (demo: crosshair on
  `plot(sin(x))`).
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
| 2 | OOM abort (`zeros(1e10)`, `1:1e9`) | 🔶 open | 2.5 |
| 3 | bare-var echo `x = 5` + `ans` | ✅ fixed (batch 1.5) | — |
| 4 | indexed-assign display | ✅ fixed (batch 1) | — |
| 5 | index orientation | 🔶 open | 2.4 |
| 6/12 | char/string model (`[65 'a']`, `'abc'+1`) | 🔶 open | 2.8 |
| 7 | `[]`/array conditions | ✅ fixed (batch 1) | — |
| 8 | `disp/fprintf/class` | 🔶 open | 2.4 |
| 9 | `hold on` / block comments | 🔶 open | 3.2 / 2.4 |
| 10 | empty shape `1:0` display | 🔶 open | 2.4 |
| 11 | format short decimals + scale factor | 🔶 open | 2.4 |
| 13 | chained indexing | 🔶 open | 2.10 |
| 14 | error IDs + positions | 🔶 open | 2.11 |
| 15 | panic across FFI boundary | 🔶 half-closed (`catch_unwind` in FFI; `panic="abort"` decision) | 2.3 + 2.5 |

## 11. Current focus

**Step 2.3 executed (2026-10-27) — code complete, all sandbox gates green** (contract v2 +
run_script/cancel_job/active_job_id; cooperative cancel in loops + ode45/ode23/integral;
panic policy = Option A recorded in COMPATIBILITY §4.8 row 15). Remaining DoD for closure:
the on-device pendulum demo (progress bar + Stop + consistent workspace) and
`while(1) end` cancel from the UI — user verification. **After user confirms: next step is
2.4.** No session may start any other step without the user explicitly reordering (§ Governance).

## 12. Change log

| Doc version | Date | Change | Approved by |
|---|---|---|---|
| 1.0 | 2026-10-25 | Initial binding plan: baseline P0/P1/P2 recorded; steps 2.3–2.11, 3.1–3.6, 4.1–4.4, 5.1–5.6, 6.1–6.3, v1.0 close defined; expansion E1–E8 named; audit cross-map added | User (scope answers, this session) |
