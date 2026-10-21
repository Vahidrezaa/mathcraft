% P2 corpus — بازیابی 2x^2 - x + 3
x = [-2 -1 0 1 2];
y = 2*x.^2 - x + 3;
p = polyfit(x, y, 2);
