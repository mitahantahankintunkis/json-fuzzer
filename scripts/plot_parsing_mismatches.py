#!/bin/env python3
import sys
from matplotlib.colors import ListedColormap
import matplotlib.pyplot as plt
# from matplotlib.text import Bbox
import numpy as np
import csv
from pprint import pprint

# print(plt.style.available)
plt.style.use('seaborn-v0_8-paper')

matrix = []

parser_lookup = {
    'c_cjson': 'https://github.com/DaveGamble/cJSON',
    'c_jansson': 'https://github.com/akheron/jansson',
    'c_json_parser': 'https://github.com/json-parser/json-parser',
    'c_mjson': 'https://github.com/cesanta/mjson/tree/master',
    'cpp_modsecurity': 'https://github.com/owasp-modsecurity/ModSecurity',
    'go_buger_jsonparser': 'https://github.com/buger/jsonparser',
    'go_francoispqt_gojay': 'https://github.com/francoispqt/gojay',
    'go_json_iterator': 'https://github.com/json-iterator/go',
    'go_std': 'https://pkg.go.dev/encoding/json',
    'go_sugawarayuuta_sonnet': 'https://github.com/sugawarayuuta/sonnet',
    'go_tidwall_gjson': 'https://github.com/tidwall/gjson',
    'go_tidwall_gjson_safe': 'https://github.com/tidwall/gjson',
    'go_valyala_fastjson': 'https://github.com/valyala/fastjson',
    'lua_cjson': 'https://github.com/openresty/lua-cjson',
    'python_json': 'https://docs.python.org/3/library/json.html',
    'python_msgspec': 'https://github.com/jcrist/msgspec',
    'python_orjson': 'https://github.com/ijl/orjson',
    'python_rapidjson': 'https://github.com/python-rapidjson/python-rapidjson',
    'python_simplejson': 'https://github.com/simplejson/simplejson',
    'python_ujson': 'https://github.com/ultrajson/ultrajson',
    'rust_json': 'https://github.com/maciejhirsz/json-rust',
    'rust_serde': 'https://github.com/serde-rs/json',
}

with open('../analyzed/parsing_mismatches.csv', 'r') as f:
    reader = csv.reader(f, delimiter='\t')

    data = {}
    keys = set()

    # go_buger_jsonparser	go_francoispqt_gojay	\t{"q":2,"q":3}	2	3
    for row in reader:
        if row[0] not in data:
            data[row[0]] = []
        if row[1] not in data:
            data[row[1]] = []

        data[row[0]].append(row[1:])
        data[row[1]].append([row[0]] + row[2:])
        keys.add(row[0])
        keys.add(row[1])

    keys = sorted(keys)
    pprint(data)

    print('Gitlab Markdown table:')
    print('|P1|P2|JSON|P1\\["q"]|P2\\["q"]|')
    print('|-------|-------|----|:-----:|:-----:|')

    for key in keys:
        row = data[key]
        row_keys = [c[0] for c in row]
        matrix_row = [k in row_keys for k in keys]
        matrix.append(matrix_row)

        for key1 in sorted(set(row_keys)):
            cols = [c for c in row if c[0] == key1]
            best = cols[0]

            # Find payload with the shortest length
            for c in cols:
                if len(best[1]) > len(c[1]):
                    best = c

            l0 = key
            l1 = best[0]

            if l0 not in parser_lookup:
                print(l0, 'not in parser_lookup')
                sys.exit(1)

            if l1 not in parser_lookup:
                print(l1, 'not in parser_lookup')
                sys.exit(1)

            if l0 in parser_lookup:
                l0 = f'[{l0}]({parser_lookup[l0]})'

            if l1 in parser_lookup:
                l1 = f'[{l1}]({parser_lookup[l1]})'

            # print(f'|{l0}|{l1}|<pre lang="json">{best[1]}</pre>|`{best[2]}`|`{best[3]}`|')
            print(f'|{l0}|{l1}|{best[1]}|{best[2]}|{best[3]}|')

    # m = [
    # [1,0,2,0,0],
    # [1,1,1,2,0],
    # [0,4,1,0,0],
    # [0,4,4,1,2],
    # [1,3,0,0,1],
    # ]
    cmap = ListedColormap(['w', '#19207b'])
    fig = plt.figure()
    plt.matshow(matrix, cmap=cmap)

    # groups = ['Blues','Jazz','Rock','House','Dance']
    x_pos = np.arange(len(keys))
    plt.xticks(x_pos, keys, rotation=90)

    y_pos = np.arange(len(keys))
    plt.yticks(y_pos, keys)
    plt.title('JSON parser discrepancy matrix', loc='left')

    # plt.tight_layout()
    # plt.subplots_adjust(top=10.925, 
    #                     bottom=0.20, 
    #                     left=0.07, 
    #                     right=0.90, 
    #                     hspace=0.01, 
    #                     wspace=0.01)
    file_name = '../paper/figures/parsing_mismatches.png'
    plt.savefig(file_name, bbox_inches='tight', pad_inches=0.1, dpi=150)
    # plt.savefig(file_name, bbox_inches=Bbox([[-1,0],[5, 5.5]]))
    print('\nSaved plot to', file_name)

    # plt.show()




