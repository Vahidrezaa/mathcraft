#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""مقایسه خروجی بازتولیدشده Octave (test-corpus/generated/) با خروجی مرجع
_checked-in_ (test-corpus/expected/) با tolerance هر case — سند ۰۸ §2.

قرارداد NaN: null در JSON (مثل jsonencode اکتاو). ±Inf: رشته "Infinity"/"-Infinity".
خروجی: 0 = همه مطابق؛ 1 = مغایرت (CI را قرمز می‌کند).
"""
import json
import math
import os
import sys

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
GEN = os.path.join(ROOT, "test-corpus", "generated")
EXP = os.path.join(ROOT, "test-corpus", "expected")


def as_num(v):
    """تبدیل مقدار JSON به عدد (null→NaN، رشته‌های Inf)."""
    if v is None:
        return float("nan")
    if isinstance(v, str):
        if v == "Infinity":
            return float("inf")
        if v == "-Infinity":
            return float("-inf")
        raise ValueError(f"unexpected string value {v!r}")
    return float(v)


def flat(v):
    """ماتریس تودرتوی JSON → لیست تخت اعداد."""
    if isinstance(v, list):
        out = []
        for e in v:
            out.extend(flat(e))
        return out
    return [as_num(v)]


def close(a, b, tol):
    if math.isnan(a) and math.isnan(b):
        return True
    if math.isinf(a) or math.isinf(b):
        return a == b
    return abs(a - b) <= tol * (1.0 + abs(b))


def main():
    if not os.path.isdir(GEN) or not os.listdir(GEN):
        print("ERROR: test-corpus/generated/ is empty — run octave_regen.m first")
        return 1
    bad = 0
    exp_files = sorted(f for f in os.listdir(EXP) if f.endswith(".json"))
    for fname in exp_files:
        with open(os.path.join(EXP, fname), encoding="utf-8") as f:
            exp = json.load(f)
        gpath = os.path.join(GEN, fname)
        if not os.path.exists(gpath):
            print(f"MISSING generated/{fname}")
            bad += 1
            continue
        with open(gpath, encoding="utf-8") as f:
            gen = json.load(f)
        # خروجی octave_regen ساختار تخت دارد {var: val}؛ expected ساختار {vars:{...}}
        gen_vars = gen.get("vars", gen)
        tol = float(exp.get("tol", 1e-9))
        for var, want in exp["vars"].items():
            if var not in gen_vars:
                print(f"{fname}:{var} MISSING in Octave output")
                bad += 1
                continue
            wv, gv = flat(want), flat(gen_vars[var])
            if len(wv) != len(gv):
                print(f"{fname}:{var} SHAPE {len(gv)} vs {len(wv)}")
                bad += 1
                continue
            for i, (g, w) in enumerate(zip(gv, wv)):
                if not close(g, w, tol):
                    print(f"{fname}:{var}[{i}] {g} vs {w} (tol {tol})")
                    bad += 1
    if bad:
        print(f"FAIL: {bad} mismatch(es) — Octave output differs from expected/")
        return 1
    print(f"OK: {len(exp_files)} scripts match Octave within tolerance")
    return 0


if __name__ == "__main__":
    sys.exit(main())
