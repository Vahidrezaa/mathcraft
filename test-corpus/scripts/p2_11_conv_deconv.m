% P2 corpus — conv/deconv round-trip
u = [1 2 3];
v = [1 1];
c = conv(u, v);
[q, r] = deconv(c, v);
