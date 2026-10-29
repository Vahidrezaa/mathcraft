% P2 corpus — رگرسیون tril/triu با k (فیکس ROADMAP 2.6: k>0 = بالای قطر اصلی)
A = [1 2 3; 4 5 6; 7 8 9];
L1 = tril(A, 1);
Lm1 = tril(A, -1);
U1 = triu(A, 1);
Um1 = triu(A, -1);
