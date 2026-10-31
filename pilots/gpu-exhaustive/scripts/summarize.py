#!/usr/bin/env python3
"""Summarize a directory of run logs (from scripts/run-all.sh) into Markdown tables.

Usage: python3 scripts/summarize.py <run-dir> [<extra-run-dir> ...] > summary.md
"""
import os
import re
import statistics
import sys

KV = re.compile(r'(\w[\w\[\],\-]*)=("[^"]*"|\S+)')


def parse(line):
    return {k: v.strip('"') for k, v in KV.findall(line)}


def load(dirs):
    logs = {}
    for d in dirs:
        for f in sorted(os.listdir(d)):
            if f.endswith('.log'):
                with open(os.path.join(d, f)) as fh:
                    logs[f[:-4]] = fh.read().splitlines()
    return logs


def results(lines, tag):
    return [parse(l[len('RESULT ' + tag):]) for l in lines if l.startswith('RESULT ' + tag + ' ')]


def first(lines, tag):
    r = results(lines, tag)
    return r[0] if r else {}


def jit_of(lines):
    for l in lines:
        if l.startswith('RESULT jit '):
            return parse(l).get('jit_estimate_s', '')
    return ''


def check_table(logs, prefix):
    out = ['| path | backend | runs (s) | median (s) | JIT (s) | setup (s) | max ULP (normal / subnormal) | hash | identical across reps |',
           '|---|---|---|---|---|---|---|---|---|']
    for name, lines in logs.items():
        if not name.startswith(prefix):
            continue
        setup = first(lines, 'setup')
        checks = results(lines, 'check')
        if not checks:
            err = [l for l in lines if 'panicked' in l or 'rror' in l][:1]
            out.append(f"| {name} | {setup.get('backend', '?')} | failed: {err[0][:80] if err else '?'} | | | | | | |")
            continue
        secs = [float(c['secs']) for c in checks]
        hashes = {c['hash'] for c in checks}
        g = first(lines, 'golden')
        up = first(lines, 'golden_upload_s') if False else None
        upl = [l for l in lines if l.startswith('RESULT golden_upload_s=')]
        setup_s = ''
        if g:
            setup_s = f"golden {float(g['scan_s']) + float(g['fill_s']):.2f}"
            if upl:
                setup_s += f" + upload {upl[0].split('=')[1]}"
        c0 = checks[0]
        out.append(
            f"| {name} | {setup.get('backend', '?')} | {', '.join(f'{s:.3f}' for s in secs)} | {statistics.median(secs):.3f} | {jit_of(lines)} | {setup_s} | "
            f"{c0.get('max_ulp_normal')} / {c0.get('max_ulp_subnormal')} | {c0['hash']} | {'yes' if len(hashes) == 1 else 'NO'} |")
        mins = results(lines, 'check_minimal')
        if mins:
            ms = [float(m['secs']) for m in mins]
            res = f"mismatches={mins[0]['mismatches']}" if 'mismatches' in mins[0] else f"max_ulp={mins[0]['max_ulp']}"
            out.append(f"| {name} (minimal loop) | rayon | {', '.join(f'{x:.3f}' for x in ms)} | {statistics.median(ms):.3f} | | | {res} | | |")
    return '\n'.join(out)


def diff_table(logs, prefix):
    out = ['| diff | A | B | differing outputs | by class of B [zero, sub, normal, inf, NaN] | max ULP (normal / subnormal) | A time (s) | B time (s) | compare (s) | total (s) |',
           '|---|---|---|---|---|---|---|---|---|---|']
    for name, lines in logs.items():
        if not name.startswith(prefix):
            continue
        s = first(lines, 'diff_setup')
        st = first(lines, 'diff_stats')
        su = first(lines, 'diff_summary')
        if not su:
            out.append(f"| {name} | {s.get('a', '?')} | {s.get('b', '?')} | failed | | | | | | |")
            continue
        cls = [l for l in lines if l.startswith('RESULT diff_summary')][0]
        m = re.search(r'by_class_of_b\[[^\]]*\]=(\[[^\]]*\])', cls)
        out.append(f"| {name} | {s['a']} | {s['b']} | {su['differing_outputs']} | {m.group(1) if m else ''} | {st.get('max_ulp_normal')} / {st.get('max_ulp_subnormal')} | "
                   f"{su['a_s']} | {su['b_s']} | {su['compare_s']} | {su['total_s']} |")
    return '\n'.join(out)


def mutant_table(logs, names):
    runs = {}
    for n in names:
        if n in logs:
            runs[n] = {int(r['id']): r for r in results(logs[n], 'mutant')}
    ids = sorted({i for r in runs.values() for i in r})
    head = '| id | description | ' + ' | '.join(f'{n}: verdict (max ULP normal/sub, s)' for n in runs) + ' |'
    out = [head, '|' + '---|' * (2 + len(runs))]
    for i in ids:
        desc = ''
        cells = []
        for n, r in runs.items():
            m = r.get(i)
            if not m:
                cells.append('')
                continue
            desc = m.get('desc', desc) or desc
            st = m['status'] + ('=' if m.get('same_outputs_as_original') == 'yes' else '')
            nm = f", NaN-mismatch {m['nan_mismatch']}" if m.get('nan_mismatch', '0') != '0' else ''
            cells.append(f"{st} ({m['max_ulp_normal']}/{m['max_ulp_subnormal']}{nm}, {float(m['secs']):.2f})")
        out.append(f"| {i} | {desc or 'original'} | " + ' | '.join(cells) + ' |')
    summ = []
    for n in runs:
        s = first(logs[n], 'mutants_summary')
        b = [l for l in logs[n] if l.startswith('RESULT bound')]
        summ.append(f"- {n}: killed {s.get('killed')} of {s.get('of')}, mutant sweep {s.get('total_secs')} s; {b[0][len('RESULT '):] if b else ''}")
    return '\n'.join(out) + '\n\n' + '\n'.join(summ)


def main():
    logs = load(sys.argv[1:])
    print('## fp-probe\n')
    cases, cols, cell = [], [], {}
    for n, lines in logs.items():
        if not n.startswith('fp-probe'):
            continue
        for l in lines:
            if l.startswith('RESULT fp_probe'):
                r = parse(l)
                if r['case'] not in cases:
                    cases.append(r['case'])
                if r['backend'] not in cols:
                    cols.append(r['backend'])
                cell[(r['case'], r['backend'])] = ('ok' if l.rstrip().endswith('OK') else 'DIFFERS') + f" ({r['got']})"
    print('| case | ' + ' | '.join(cols) + ' |')
    print('|' + '---|' * (len(cols) + 1))
    for c in cases:
        print(f'| {c} | ' + ' | '.join(cell.get((c, b), '') for b in cols) + ' |')
    print('\n## spec validation\n')
    for n, lines in logs.items():
        if n.startswith('spec-validate'):
            for l in lines:
                if l.startswith('RESULT'):
                    print('- ' + l[7:])
    for k, p in [('Kernel A check', 'a-check'), ('Kernel A native check', 'a-native-check'), ('Kernel B check', 'b-check'), ('Kernel B shifter check', 'b-shifter')]:
        print(f'\n## {k}\n')
        print(check_table(logs, p))
    for k, p in [('Kernel A diff', 'a-diff'), ('Kernel A native diff', 'a-native-diff'), ('Kernel B diff', 'b-diff')]:
        print(f'\n## {k}\n')
        print(diff_table(logs, p))
    print('\n## Mutants (exhaustive)\n')
    print(mutant_table(logs, [n for n in logs if n.startswith('mut-') and 'sample' not in n]))
    print('\n## Mutants (2^24 sample)\n')
    print(mutant_table(logs, [n for n in logs if n.startswith('mut-') and 'sample' in n]))


if __name__ == '__main__':
    main()
