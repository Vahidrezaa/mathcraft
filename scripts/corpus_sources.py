# -*- coding: utf-8 -*-
"""کورپوس P1 — سورس‌های .m و متغیرهای قابل‌گرفت (منبع یگانه حقیقت کورپوس).

قواعد سند ۰۸ §2: اسکریپت‌ها قطعی‌اند (بدون rand)، متغیرهای انتهایی capture می‌شوند،
tol پیش‌فرض 1e-9. expected هرگز از خود MathCraft تولید نمی‌شود (اوراکل: numpy/LAPACK).
"""
# {filename: (source, [captured_vars])}
SOURCES = {}

def _add(name, src, vars_):
    SOURCES[name] = (src.strip() + "\n", vars_)

# ---------- 01-20: اسکریپت‌های ماتریس/جبرخطی (از نسخه inline قبلی) ----------
_add("p1_01_linear_system_2x2.m", """
A = [4 2; 2 3];
b = [1; 2];
x = A\\b;
""", ["x"])

_add("p1_02_linear_system_5x5.m", """
A = [10 1 0 0 0; 1 10 1 0 0; 0 1 10 1 0; 0 0 1 10 1; 0 0 0 1 10];
b = [11; 12; 12; 12; 11];
x = A\\b;
r = A*x - b;
maxabs = max(abs(r));
""", ["maxabs"])

_add("p1_03_backslash_tridiagonal.m", """
A = [4 1 0 0; 1 4 1 0; 0 1 4 1; 0 0 1 4];
b = [5; 6; 6; 5];
x = A\\b;
r = max(abs(A*x - b));
""", ["r"])

_add("p1_04_overdetermined_exact.m", """
A = [1 0; 0 1; 1 1];
b = [1; 2; 3];
x = A\\b;
r = max(abs(A*x - b));
""", ["x", "r"])

_add("p1_05_underdetermined_min_norm.m", """
A = [1 0 0; 0 1 0];
b = [1; 2];
x = A\\b;
""", ["x"])

_add("p1_06_rref_free_variable.m", """
r = rref([1 2 3; 2 4 6]);
""", ["r"])

_add("p1_07_det_and_inv.m", """
A = [4 2; 2 3];
d = det(A);
I = inv(A);
""", ["d", "I"])

_add("p1_08_eig_symmetric.m", """
v = eig([2 1; 1 2]);
""", ["v"])

_add("p1_09_chol_and_norm.m", """
L = chol([4 2; 2 3]);
err = max(max(abs(L*L' - [4 2; 2 3])));
n = norm([3 4]);
""", ["err", "n"])

_add("p1_10_transpose_identity.m", """
A = [1 2; 3 4];
B = [0 1; 1 0];
lhs = (A*B)';
rhs = B'*A';
err = max(max(abs(lhs - rhs)));
""", ["err"])

_add("p1_11_for_sum_squares.m", """
s = 0;
for k = 1:5
    s = s + k^2;
end
""", ["s"])

_add("p1_12_for_matrix_columns.m", """
M = [1 3; 2 4];
t = 0;
for c = M
    t = t + c(2);
end
""", ["t"])

_add("p1_13_while_break.m", """
s = 0; k = 0;
while true
    k = k + 1;
    s = s + k;
    if s > 10
        break;
    end
end
""", ["s", "k"])

_add("p1_14_anon_capture.m", """
a = 10;
f = @(x) x + a;
a = 0;
y = f(5);
""", ["y"])

_add("p1_15_nested_anon_closure.m", """
g = @(a) @(x) a.*x;
h = g(3);
y = h(4);
""", ["y"])

_add("p1_16_colon_semantics.m", """
v = 0:0.1:1;
n = numel(v);
last = v(n);
""", ["n", "last"])

_add("p1_17_ans_suppression.m", """
ans = 7;
dummy = 1;
2;
""", ["ans"])

_add("p1_18_indexed_growth.m", """
x = [];
x(3) = 5;
""", ["x"])

_add("p1_19_end_indexing.m", """
v = [10 20 30];
a = v(end);
w = v(1:end-1);
s = sum(w);
""", ["a", "s"])

_add("p1_20_matrix_vs_elementwise_power.m", """
A = [1 2; 3 4];
M = A^2;
E = A.^2;
d1 = M(1,1) + M(2,2);
d2 = E(1,1) + E(2,2);
""", ["d1", "d2"])

# ---------- 21-30: توابع عنصری ----------
_add("p1_21_elementwise_identities.m", """
y1 = sin(pi/6);
y2 = cos(pi/3);
y3 = exp(log(7));
y4 = sqrt(16);
""", ["y1", "y2", "y3", "y4"])

_add("p1_22_trig_identity.m", """
x = 0.7;
s = sin(x)^2 + cos(x)^2;
t = tan(pi/4);
""", ["s", "t"])

_add("p1_23_inverse_trig.m", """
a = asin(0.5);
b = acos(0.5);
c = atan(1);
""", ["a", "b", "c"])

_add("p1_24_hyperbolic_identity.m", """
a = sinh(1);
b = cosh(1);
idd = cosh(1)^2 - sinh(1)^2;
""", ["a", "b", "idd"])

_add("p1_25_rounding_family.m", """
v = [2.5 -2.5 2.7 -2.7];
r = round(v);
f = fix(v);
fl = floor(v);
ce = ceil(v);
""", ["r", "f", "fl", "ce"])

_add("p1_26_sign_abs.m", """
s = sign([-3 0 2]);
a = abs([-3 4]);
""", ["s", "a"])

_add("p1_27_mod_rem_signs.m", """
a = mod(-7, 3);
b = rem(-7, 3);
c = mod(7, -3);
d = rem(7, -3);
""", ["a", "b", "c", "d"])

_add("p1_28_log_family.m", """
l1 = log2(8);
l2 = log10(1000);
l3 = log(exp(2));
""", ["l1", "l2", "l3"])

_add("p1_29_sqrt_exp_elementwise.m", """
A = [1 4; 9 16];
S = sqrt(A);
e = exp([0 1]);
""", ["S", "e"])

_add("p1_30_atan2_quadrants.m", """
q1 = atan2(1, 1);
q2 = atan2(1, -1);
q3 = atan2(-1, -1);
q4 = atan2(-1, 1);
""", ["q1", "q2", "q3", "q4"])

# ---------- 31-38: کاهشی/آمار پایه و NaN ----------
_add("p1_31_reductions_vector.m", """
v = [4 1 3];
s = sum(v);
p = prod(v);
m = max(v);
mn = min(v);
cs = cumsum(v);
""", ["s", "p", "m", "mn", "cs"])

_add("p1_32_reductions_matrix_dims.m", """
A = [1 2; 3 4];
c = sum(A);
r = sum(A, 2);
t = sum(sum(A));
""", ["c", "r", "t"])

_add("p1_33_cumprod_diff.m", """
v = [1 2 3 4];
cp = cumprod(v);
d = diff(v);
""", ["cp", "d"])

_add("p1_34_mean_all_any.m", """
A = [1 5; 3 1];
m = mean(A);
a1 = all([1 0; 1 1]);
q = any([0 0; 1 0], 2);
""", ["m", "a1", "q"])

_add("p1_35_find_column_major.m", """
A = [0 2 0; 3 0 4];
f = find(A);
s = sum(f);
""", ["f", "s"])

_add("p1_36_sort_vector_flags.m", """
a = sort([3 1 2]);
d = sort([3 1 2], 'descend');
""", ["a", "d"])

_add("p1_37_sort_matrix_columns.m", """
A = [2 5; 1 4];
a = sort(A);
d = sort(A, 'descend');
""", ["a", "d"])

_add("p1_38_nan_semantics.m", """
s = sort([3 1 NaN 2]);
mx = max([1 NaN 3]);
mn = min([NaN 2 1]);
sm = sum([1 NaN 3]);
""", ["s", "mx", "mn", "sm"])

# ---------- 39-46: سازه‌ها و شکل ----------
_add("p1_39_constructors.m", """
zsum = sum(sum(zeros(2, 3)));
osum = sum(ones(1, 3));
esum = sum(sum(eye(2, 3)));
""", ["zsum", "osum", "esum"])

_add("p1_40_linspace_logspace.m", """
l = linspace(0, 1, 5);
n = numel(l);
lg = logspace(0, 3, 4);
""", ["l", "n", "lg"])

_add("p1_41_colon_steps.m", """
a = 5:-1:1;
b = 0:0.5:2;
n = numel(0:0.1:1);
""", ["a", "b", "n"])

_add("p1_42_diag_modes.m", """
m = diag([2 3]);
dd = diag(m);
""", ["m", "dd"])

_add("p1_43_tril_triu.m", """
A = [1 2; 3 4];
L = tril(A);
U = triu(A);
zs = sum(sum(L + U - A));
""", ["L", "U", "zs"])

_add("p1_44_reshape_repmat.m", """
A = reshape([1 2 3 4 5 6], 2, 3);
t = trace(reshape(1:9, 3, 3));
R = repmat([1 2], 2, 2);
""", ["A", "t", "R"])

_add("p1_45_size_length_numel.m", """
A = [1 2 3; 4 5 6];
s = size(A);
l = length(A);
n = numel(A);
q = isempty(A);
""", ["s", "l", "n", "q"])

_add("p1_46_transpose_forms.m", """
A = [1 2; 3 4];
B = A';
err = max(max(abs(A' - A.')));
""", ["B", "err"])

# ---------- 47-58: جبر خطی ----------
_add("p1_47_det_inv_diagonal.m", """
A = diag([2 3 4]);
d = det(A);
I = inv(A);
t = trace(A);
""", ["d", "I", "t"])

_add("p1_48_det_upper_triangular.m", """
A = [2 1 0; 0 3 1; 0 0 4];
d = det(A);
d2 = det(triu(A));
""", ["d", "d2"])

_add("p1_49_rank_trace.m", """
A = [1 2 3; 2 4 6];
rk = rank(A);
rk2 = rank(eye(4));
t = trace([1 2; 3 4]);
""", ["rk", "rk2", "t"])

_add("p1_50_rref_two_cases.m", """
R = rref([1 2 3; 2 4 6]);
R2 = rref([1 2; 3 4]);
""", ["R", "R2"])

_add("p1_51_solve_exact_2x2.m", """
A = [3 1; 1 2];
b = [9; 8];
x = A\\b;
res = max(abs(A*x - b));
""", ["x", "res"])

_add("p1_52_solve_overdetermined.m", """
A = [1 0; 0 1; 1 1];
b = [1; 2; 3];
x = A\\b;
r = max(abs(A*x - b));
""", ["x", "r"])

_add("p1_53_least_squares_general.m", """
A = [1 0; 1 1; 1 2];
b = [6; 0; 0];
x = A\\b;
rn = norm(A*x - b);
""", ["x", "rn"])

_add("p1_54_triangular_solve.m", """
U = [2 1; 0 4];
b = [4; 8];
x = U\\b;
chk = max(abs(U*x - b));
""", ["x", "chk"])

_add("p1_55_chol_reconstruction.m", """
A = [4 2; 2 3];
L = chol(A);
err = max(max(abs(L*L' - A)));
l11 = L(1,1);
""", ["err", "l11"])

_add("p1_56_eig_properties.m", """
A = [2 1; 1 2];
v = eig(A);
sumv = sum(v);
""", ["v", "sumv"])

_add("p1_57_svd_values.m", """
s = svd([3 0; 0 2]);
A2 = [4 11 14; 8 7 -2];
sv = svd(A2);
s1 = sv(1);
s2 = sv(2);
""", ["s", "s1", "s2"])

_add("p1_58_qr_upper_factor.m", """
A = [1 2; 3 4];
R = qr(A);
r11 = R(1,1);
r21 = R(2,1);
r22 = R(2,2);
""", ["r11", "r21", "r22"])

# ---------- 59-60: هندسی/نُرم‌ها ----------
_add("p1_59_dot_cross_kron.m", """
d = dot([1 2 3], [4 5 6]);
c = cross([1 0 0], [0 1 0]);
k = kron([1 2], [10 100]);
""", ["d", "c", "k"])

_add("p1_60_cond_and_norms.m", """
c1 = cond([2 0; 0 0.5]);
ci = cond(eye(3));
n1 = norm([3 4], 1);
ni = norm([3 4], inf);
nf = norm([1 2; 3 4], 'fro');
""", ["c1", "ci", "n1", "ni", "nf"])
