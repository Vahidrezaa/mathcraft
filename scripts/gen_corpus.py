# -*- coding: utf-8 -*-
"""تولید خروجی مرجع کورپوس P1 با اوراکل مستقل numpy/LAPACK (سند ۰۸ §2).

- اسکریپت‌های .m را در test-corpus/scripts/ می‌نویسد.
- مقادیر مرجع را با numpy (غیر از موتور Rust ما) می‌سازد → test-corpus/expected/*.json
- manifest.json (لیست اسکریپت‌ها و متغیرهای capture) را می‌نویسد.
- round به ۱۵ رقم معنادار (کاننical سند ۰۸ §2).
"""
import json
import math
import os
import sys

import numpy as np

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from corpus_sources import SOURCES

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
SCRIPTS = os.path.join(ROOT, "test-corpus", "scripts")
EXPECTED = os.path.join(ROOT, "test-corpus", "expected")


def r15(x):
    """گرد کردن به ۱۵ رقم معنادار — کاننیکال سند ۰۸ §2."""
    if isinstance(x, (int, float)) and (math.isnan(x) or math.isinf(x)):
        return x
    return float(f"{float(x):.15g}")


def canon(v):
    # قرارداد JSON: NaN → null (مثل jsonencode اکتاو)؛ ±Inf → رشته نشانه‌دار
    def num(x):
        x = r15(x)
        if isinstance(x, float) and math.isnan(x):
            return None
        if isinstance(x, float) and math.isinf(x):
            return "Infinity" if x > 0 else "-Infinity"
        return x
    if isinstance(v, np.ndarray):
        return [[num(float(e)) for e in row] for row in v]
    if isinstance(v, (list, tuple)):
        return [canon(e) for e in v]
    return num(float(v))


# ---------- اوراکل numpy: مقدار هر متغیر هر اسکریپت (مستقل از موتور ما) ----------
O = {}

pi, e_ = math.pi, math.e
O["p1_01_linear_system_2x2.m"] = {"x": np.linalg.solve([[4, 2], [2, 3]], [[1], [2]])}
A5 = np.array([[10, 1, 0, 0, 0], [1, 10, 1, 0, 0], [0, 1, 10, 1, 0], [0, 0, 1, 10, 1], [0, 0, 0, 1, 10.0]])
b5 = np.array([11, 12, 12, 12, 11.0])
x5 = np.linalg.solve(A5, b5)
O["p1_02_linear_system_5x5.m"] = {"maxabs": float(np.max(np.abs(A5 @ x5 - b5)))}
A3 = np.array([[4, 1, 0, 0], [1, 4, 1, 0], [0, 1, 4, 1], [0, 0, 1, 4.0]])
b3 = np.array([5, 6, 6, 5.0])
x3 = np.linalg.solve(A3, b3)
O["p1_03_backslash_tridiagonal.m"] = {"r": float(np.max(np.abs(A3 @ x3 - b3)))}
O["p1_04_overdetermined_exact.m"] = {"x": [[1.0], [2.0]], "r": 0.0}
O["p1_05_underdetermined_min_norm.m"] = {"x": [[1.0], [2.0], [0.0]]}
O["p1_06_rref_free_variable.m"] = {"r": [[1, 2, 3], [0, 0, 0]]}
O["p1_07_det_and_inv.m"] = {"d": 8.0, "I": np.linalg.inv([[4, 2], [2, 3]])}
O["p1_08_eig_symmetric.m"] = {"v": [[1.0], [3.0]]}
# MATLAB/Octave chol returns the UPPER triangular R with R'*R = A.
# The script computes err = max(abs(R*R' - A)) which is 1.0 for upper chol
# (R*R' != A) — the value Octave gives; numpy's lower cholesky gave 0.0 and
# was an oracle orientation bug caught by the Octave differential job.
_R9 = np.linalg.cholesky(np.array([[4, 2], [2, 3.0]])).T  # upper, MATLAB convention
O["p1_09_chol_and_norm.m"] = {
    "err": float(np.abs(_R9 @ _R9.T - np.array([[4, 2], [2, 3.0]])).max()),
    "n": 5.0,
}
O["p1_10_transpose_identity.m"] = {"err": 0.0}
O["p1_11_for_sum_squares.m"] = {"s": 55.0}
O["p1_12_for_matrix_columns.m"] = {"t": 6.0}
O["p1_13_while_break.m"] = {"s": 15.0, "k": 5.0}
O["p1_14_anon_capture.m"] = {"y": 15.0}
O["p1_15_nested_anon_closure.m"] = {"y": 12.0}
O["p1_16_colon_semantics.m"] = {"n": 11.0, "last": 1.0}
O["p1_17_ans_suppression.m"] = {"ans": 7.0}
O["p1_18_indexed_growth.m"] = {"x": [[0.0], [0.0], [5.0]]}
O["p1_19_end_indexing.m"] = {"a": 30.0, "s": 30.0}
O["p1_20_matrix_vs_elementwise_power.m"] = {"d1": 29.0, "d2": 17.0}
O["p1_21_elementwise_identities.m"] = {"y1": 0.5, "y2": 0.5, "y3": 7.0, "y4": 4.0}
O["p1_22_trig_identity.m"] = {"s": 1.0, "t": 1.0}
O["p1_23_inverse_trig.m"] = {"a": pi / 6, "b": pi / 3, "c": pi / 4}
O["p1_24_hyperbolic_identity.m"] = {"a": math.sinh(1.0), "b": math.cosh(1.0), "idd": 1.0}
O["p1_25_rounding_family.m"] = {"r": [[3, -3, 3, -3]], "f": [[2, -2, 2, -2]], "fl": [[2, -3, 2, -3]], "ce": [[3, -2, 3, -2]]}
O["p1_26_sign_abs.m"] = {"s": [[-1, 0, 1]], "a": [[3, 4]]}
O["p1_27_mod_rem_signs.m"] = {"a": 2.0, "b": -1.0, "c": -2.0, "d": 1.0}
O["p1_28_log_family.m"] = {"l1": 3.0, "l2": 3.0, "l3": 2.0}
O["p1_29_sqrt_exp_elementwise.m"] = {"S": [[1, 2], [3, 4]], "e": [1.0, math.e]}
O["p1_30_atan2_quadrants.m"] = {"q1": pi / 4, "q2": 3 * pi / 4, "q3": -3 * pi / 4, "q4": -pi / 4}
O["p1_31_reductions_vector.m"] = {"s": 8.0, "p": 12.0, "m": 4.0, "mn": 1.0, "cs": [[4, 5, 8]]}
O["p1_32_reductions_matrix_dims.m"] = {"c": [[4, 6]], "r": [[3.0], [7.0]], "t": 10.0}
O["p1_33_cumprod_diff.m"] = {"cp": [[1, 2, 6, 24]], "d": [[1, 1, 1]]}
O["p1_34_mean_all_any.m"] = {"m": [[2, 3]], "a1": [[1, 0]], "q": [[0.0], [1.0]]}
O["p1_35_find_column_major.m"] = {"f": [[2.0], [3.0], [6.0]], "s": 11.0}
O["p1_36_sort_vector_flags.m"] = {"a": [[1, 2, 3]], "d": [[3, 2, 1]]}
O["p1_37_sort_matrix_columns.m"] = {"a": [[1, 4], [2, 5]], "d": [[2, 5], [1, 4]]}
O["p1_38_nan_semantics.m"] = {"s": [[1, 2, 3, float("nan")]], "mx": 3.0, "mn": 1.0, "sm": float("nan")}
O["p1_39_constructors.m"] = {"zsum": 0.0, "osum": 3.0, "esum": 2.0}
O["p1_40_linspace_logspace.m"] = {"l": [0.0, 0.25, 0.5, 0.75, 1.0], "n": 5.0, "lg": [1.0, 10.0, 100.0, 1000.0]}
O["p1_41_colon_steps.m"] = {"a": [5, 4, 3, 2, 1], "b": [0.0, 0.5, 1.0, 1.5, 2.0], "n": 11.0}
O["p1_42_diag_modes.m"] = {"m": [[2, 0], [0, 3]], "dd": [[2.0], [3.0]]}
O["p1_43_tril_triu.m"] = {"L": [[1, 0], [3, 4]], "U": [[1, 2], [0, 4]], "zs": 5.0}
O["p1_44_reshape_repmat.m"] = {"A": [[1, 3, 5], [2, 4, 6]], "t": 15.0, "R": [[1, 2, 1, 2], [1, 2, 1, 2]]}
O["p1_45_size_length_numel.m"] = {"s": [[2, 3]], "l": 3.0, "n": 6.0, "q": 0.0}
O["p1_46_transpose_forms.m"] = {"B": [[1, 3], [2, 4]], "err": 0.0}
O["p1_47_det_inv_diagonal.m"] = {"d": 24.0, "I": [[0.5, 0, 0], [0, 1 / 3, 0], [0, 0, 0.25]], "t": 9.0}
O["p1_48_det_upper_triangular.m"] = {"d": 24.0, "d2": 24.0}
O["p1_49_rank_trace.m"] = {"rk": 1.0, "rk2": 4.0, "t": 5.0}
O["p1_50_rref_two_cases.m"] = {"R": [[1, 2, 3], [0, 0, 0]], "R2": [[1, 0], [0, 1]]}
O["p1_51_solve_exact_2x2.m"] = {"x": [[2.0], [3.0]], "res": 0.0}
O["p1_52_solve_overdetermined.m"] = {"x": [[1.0], [2.0]], "r": 0.0}
O["p1_53_least_squares_general.m"] = {"x": [[5.0], [-3.0]], "rn": math.sqrt(6.0)}
O["p1_54_triangular_solve.m"] = {"x": [[1.0], [2.0]], "chk": 0.0}
_R55 = np.linalg.cholesky(np.array([[4, 2], [2, 3.0]])).T  # upper, MATLAB convention
O["p1_55_chol_reconstruction.m"] = {
    "err": float(np.abs(_R55 @ _R55.T - np.array([[4, 2], [2, 3.0]])).max()),
    "l11": float(_R55[0, 0]),
}
O["p1_56_eig_properties.m"] = {"v": [[1.0], [3.0]], "sumv": 4.0}
sv = np.linalg.svd(np.array([[4, 11, 14], [8, 7, -2.0]]), compute_uv=False)
O["p1_57_svd_values.m"] = {"s": [[3.0], [2.0]], "s1": float(sv[0]), "s2": float(sv[1])}
R = np.linalg.qr(np.array([[1, 2], [3, 4.0]]))[1]  # 2-output form: upper R (script updated accordingly)
O["p1_58_qr_upper_factor.m"] = {"r11": float(R[0, 0]), "r21": float(R[1, 0]), "r22": float(R[1, 1])}
O["p1_59_dot_cross_kron.m"] = {"d": 32.0, "c": [[0.0], [0.0], [1.0]], "k": [[10, 100, 20, 200]]}
O["p1_60_cond_and_norms.m"] = {"c1": 4.0, "ci": 1.0, "n1": 7.0, "ni": 4.0, "nf": math.sqrt(30.0)}


def main():
    os.makedirs(SCRIPTS, exist_ok=True)
    os.makedirs(EXPECTED, exist_ok=True)
    manifest = []
    for fname, (src, vars_) in sorted(SOURCES.items()):
        with open(os.path.join(SCRIPTS, fname), "w", encoding="utf-8") as f:
            f.write("% MathCraft corpus P1 (doc 08 s2) — deterministic, tol 1e-9\n")
            f.write(src)
        exp = O[fname]
        missing = [v for v in vars_ if v not in exp]
        if missing:
            raise SystemExit(f"{fname}: oracle missing vars {missing}")
        doc = {"script": fname, "tol": 1e-9, "generator": "numpy-2.1.3-seed",
               "vars": {k: canon(v) for k, v in exp.items()}}
        with open(os.path.join(EXPECTED, fname.replace(".m", ".json")), "w", encoding="utf-8") as f:
            json.dump(doc, f, indent=1, allow_nan=False)
            f.write("\n")
        manifest.append({"script": fname, "vars": vars_})
    with open(os.path.join(ROOT, "test-corpus", "manifest.json"), "w", encoding="utf-8") as f:
        json.dump({"tol_default": 1e-9, "cases": manifest}, f, indent=1)
        f.write("\n")
    print(f"OK: {len(manifest)} scripts + expected JSONs written")


if __name__ == "__main__":
    main()
