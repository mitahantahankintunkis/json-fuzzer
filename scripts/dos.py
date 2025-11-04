#!/bin/env python3

with open('../analyzed/dos.csv', 'r') as f:
    lines = []

    for line in f.readlines():
        spl = line.split('\t')

        if len(spl) != 4:
            continue

        try:
            int(spl[3])
        except:
            continue

        lines.append([
            spl[0],
            spl[1],
            spl[2],
            int(spl[3])
        ])

    lines.sort(key=lambda x: -x[-1])

    for line in lines[:100]:
        print('\t'.join(map(str, line)))
