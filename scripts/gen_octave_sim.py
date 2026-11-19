#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""شبیه‌ساز وفادار نویسندهٔ .mat فایل‌های Octave (بازتولید محلی شکست CI#33).

منبع هر قاعده: سورس واقعی Octave — libinterp/corefcn/ls-mat5.cc (شاخهٔ stable)
و load-save.cc — که این نوبت خوانده شد؛ هیچ چیز از خودم:
  - write_mat5_tag: قالب کوچک فقط وقتی 0 < n <= 4: u32 = (n<<16)|type
  - نام آرایه: بعد از تگ (کوچک یا عادی) همیشه پد ۸بایتی (PAD(namelen))
  - Array Flags: word2 (nzmax) وقتی 0 باشد → 1 («Matlab seems to have trouble…»)
  - پرچم‌ها: logical=0x0200, global=0x0400, complex=0x0800؛ کلاس در بایت پایین
  - رشته: miUTF8 با طول numel (مسیر conv_u16=false برای ASCII)
  - struct: طول نام فیلد = 64 (max_namelen=63 + 1)، نام‌ها پدشده به ۶۴
  - نام فیلد تهی: tag(miINT8, 0) عادی (n=0 از شرط قالب کوچک بیرون است)
  - mat7: عنصر کامل (تگ+داده) zlib می‌شود؛ بعد از miCOMPRESSED پد نیست
  - هدر: متن «MATLAB 5.0 MAT-file, written by Octave …» تا ۱۲۴ بایت + 00 01 49 4d
"""
import struct
import zlib

MI_INT8, MI_UINT8, MI_INT32, MI_UINT32 = 1, 2, 5, 6
MI_SINGLE, MI_DOUBLE, MI_MATRIX, MI_COMPRESSED, MI_UTF8 = 7, 9, 14, 15, 16

MX_CHAR, MX_DOUBLE, MX_SINGLE, MX_INT8, MX_UINT8 = 4, 6, 7, 8, 9
MX_INT16, MX_UINT16, MX_INT32, MX_UINT32, MX_INT64, MX_UINT64 = 10, 11, 12, 13, 14, 15

FLAG_LOGICAL, FLAG_GLOBAL, FLAG_COMPLEX = 0x0200, 0x0400, 0x0800


def pad(n):
    """PAD واقعی Octave (ls-mat5.cc:90): 0<n<=4 → 4؛ وگرنه گرد به ۸."""
    if 0 < n <= 4:
        return 4
    return ((n + 7) // 8) * 8


def tag(t, n):
    """write_mat5_tag — قالب کوچک فقط برای 0 < n <= 4."""
    if 0 < n <= 4:
        return struct.pack("<I", (n << 16) | t)
    return struct.pack("<II", t, n)


def header():
    text = b"MATLAB 5.0 MAT-file, written by Octave 9.4.0 (sim), 2026-11-19 00:00:00 UTC"
    text = text[:124].ljust(124, b"\x00")
    return text + b"\x00\x01IM"  # version 0x0100 LE + "IM"


def array_flags(cls, flags=0):
    # word2 = nzmax = 1 (هرگز 0 — سورس Octave)
    return tag(MI_UINT32, 8) + struct.pack("<II", cls | flags, 1)


def dims(rs, cs):
    return tag(MI_INT32, 8) + struct.pack("<ii", rs, cs)


def name_element(name):
    n = len(name.encode())
    # نویسندهٔ Octave: paddedlength = PAD(n) — برای n<=4 همین ۴ است (تگ کوچک)
    return tag(MI_INT8, n) + name.encode().ljust(pad(n), b"\x00")


def doubles(vals):
    b = struct.pack(f"<{len(vals)}d", *vals)
    return tag(MI_DOUBLE, len(b)) + b + b"\x00" * (pad(len(b)) - len(b))


def singles(vals):
    b = struct.pack(f"<{len(vals)}f", *vals)
    return tag(MI_SINGLE, len(b)) + b + b"\x00" * (pad(len(b)) - len(b))


def int32s(vals):
    b = struct.pack(f"<{len(vals)}i", *vals)
    return tag(MI_INT32, len(b)) + b + b"\x00" * (pad(len(b)) - len(b))


def uint8s(vals):
    b = bytes(vals)
    return tag(MI_UINT8, len(b)) + b + b"\x00" * (pad(len(b)) - len(b))


def utf8(s):
    b = s.encode()
    return tag(MI_UTF8, len(b)) + b + b"\x00" * (pad(len(b)) - len(b))


def element(sub, name=""):
    """miMATRIX کامل — نام باید داخل sub در جای درست (بعد dims) باشد؛
    این فقط تگ miMATRIX را می‌پیچد (طول = کل بدنه)."""
    return tag(MI_MATRIX, len(sub)) + sub


def numeric_double(name, vals, rows, cols):
    sub = array_flags(MX_DOUBLE) + dims(rows, cols) + name_element(name) + doubles(vals)
    return element(sub)


def numeric_complex(name, re, im, rows, cols):
    sub = (array_flags(MX_DOUBLE, FLAG_COMPLEX) + dims(rows, cols) + name_element(name)
           + doubles(re) + doubles(im))
    return element(sub)


def numeric_single(name, vals, rows, cols):
    sub = array_flags(MX_SINGLE) + dims(rows, cols) + name_element(name) + singles(vals)
    return element(sub)


def numeric_int32(name, vals, rows, cols):
    sub = array_flags(MX_INT32) + dims(rows, cols) + name_element(name) + int32s(vals)
    return element(sub)


def logical(name, vals):
    sub = (array_flags(MX_UINT8, FLAG_LOGICAL) + dims(1, len(vals)) + name_element(name)
           + uint8s(vals))
    return element(sub)


def char_str(name, s):
    sub = array_flags(MX_CHAR) + dims(1, len(s)) + name_element(name) + utf8(s)
    return element(sub)


def struct_1x1(name, fields):
    """fields: [(name, element_bytes)] — مقدارها عنصرهای کامل با نام تهی."""
    flen = 64  # max_namelen(63) + 1 — سورس Octave
    names_block = tag(MI_INT32, 4) + struct.pack("<i", flen)
    nb = b""
    for fname, _ in fields:
        nb += fname.encode()[: flen - 1].ljust(flen, b"\x00")
    names_block += tag(MI_INT8, len(nb)) + nb
    body = array_flags(2) + dims(1, 1) + name_element(name) + names_block
    for _, value_element in fields:
        body += value_element
    return tag(MI_MATRIX, len(body)) + body


def write(path, data, compressed=False):
    if compressed:
        comp = zlib.compress(data)
        payload = tag(MI_COMPRESSED, len(comp)) + comp
    else:
        payload = data
    with open(path, "wb") as f:
        f.write(header() + payload)


import os
import sys

out = sys.argv[1] if len(sys.argv) > 1 else "/tmp/oct-sim"
os.makedirs(out, exist_ok=True)

# ---- همان ده فیکسچر gen_octave_fixtures.m ----
M = numeric_double("M", [1, 4, 2, 5, 3, 6], 2, 3)  # ستونی
write(f"{out}/dbl_2x3.mat", M, compressed=False)  # save -mat-binary

C = numeric_complex(
    "C",
    [1.0, 3.5, -0.0, 0.0],  # ::real() در Octave برای -1i
    [2.0, -4.0, -1.0, 0.5],
    2,
    2,
)
write(f"{out}/cplx_2x2.mat", C, compressed=True)  # save -mat7-binary

s = char_str("s", "hello")
write(f"{out}/char_row.mat", s, compressed=True)

inner = numeric_double("", [1.0, 2.0], 2, 1)
st = struct_1x1(
    "st",
    [
        ("a", numeric_double("", [1.25], 1, 1)),
        ("b", char_str("", "txt")),
        ("inner", struct_1x1("", [("x", inner)])),
    ],
)
write(f"{out}/struct_1x1.mat", st, compressed=True)

# z = [NaN, Inf; -Inf, -0] → چیدمان ستونی [NaN, -Inf, Inf, -0]
z_vals = [float("nan"), float("-inf"), float("inf"), -0.0]
z = numeric_double("z", z_vals, 2, 2)
write(f"{out}/nan_inf.mat", z, compressed=True)

v = numeric_int32("v", [7, -8, 9], 3, 1)
write(f"{out}/int32_col.mat", v, compressed=True)

S = numeric_single("S", [0.5, -2.5, 1.5, 3.5], 2, 2)
write(f"{out}/single_2x2.mat", S, compressed=True)

L = logical("L", [1, 0, 1])
write(f"{out}/logical_1x3.mat", L, compressed=True)

e = numeric_double("e", [], 0, 0)
write(f"{out}/empty_0x0.mat", e, compressed=True)

print("simulated octave fixtures →", out, len(os.listdir(out)), "files")
