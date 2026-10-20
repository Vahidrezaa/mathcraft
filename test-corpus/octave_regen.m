% octave_regen.m — بازتولید خروجی مرجع کورپوس با Octave (سند ۰۸ §2).
% اجرا: octave --no-gui --eval "run('test-corpus/octave_regen.m')"
% خروجی: test-corpus/generated/*.json (به‌صورت موقت — با expected/ مقایسه می‌شود،
% نه overwrite؛ «expected هرگز از خود MathCraft تولید نمی‌شود» و اینجا هم از Octave).
% نیازمند Octave >= 7 (jsondecode/jsonencode داخلی).

root = fileparts(mfilename('fullpath'));
cd(root);
man = jsondecode(fileread(fullfile(root, 'manifest.json')));

if ~exist(fullfile(root, 'generated'), 'dir')
    mkdir(fullfile(root, 'generated'));
end

n = numel(man.cases);
for k = 1:n
    c = man.cases(k);
    cd(fullfile(root, 'scripts'));
    run(c.script);
    cd(root);
    rec = struct();
    for j = 1:numel(c.vars)
        name = c.vars{j};
        val = eval(name);
        if isempty(val)
            rec.(name) = '[]';
        elseif isscalar(val)
            rec.(name) = val;
        else
            rec.(name) = val;  % jsonencode ماتریس → آرایه تودرتو
        end
    end
    json = jsonencode(rec);
    fname = fullfile(root, 'generated', strrep(c.script, '.m', '.json'));
    fid = fopen(fname, 'w');
    fwrite(fid, json, 'char');
    fclose(fid);
    fprintf('regen %d/%d: %s\n', k, n, c.script);
end
fprintf('OK: %d files in test-corpus/generated/\n', n);
