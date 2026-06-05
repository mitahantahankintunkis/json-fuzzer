#!/bin/env python3
import sys
from matplotlib.colors import ListedColormap
import matplotlib.pyplot as plt
import matplotlib.patches as mpatches
# from matplotlib.text import Bbox
import numpy as np
import csv
import sqlite3
from pprint import pprint


    #sqlite3 ../analyzed/db.sqlite -cmd ".mode tabs" <<EOF > utf8_errors.csv
    #SELECT DISTINCT *
    #FROM (
    #	SELECT DISTINCT parser0 AS parser, json, output0 AS output
    #	FROM results
    #	UNION
    #	SELECT DISTINCT parser1 AS parser, json, output1 AS output
    #	FROM results
    #)
    #WHERE output == 'UTF8_ERROR'
    #GROUP BY parser
    #ORDER BY LENGTH(json)
    #EOF
    #
    #
    #sqlite3 ../analyzed/db.sqlite -cmd ".mode tabs" <<EOF > error_discrepancy.csv
    #EOF
    #
    #
    #sqlite3 ../analyzed/db.sqlite -cmd ".mode tabs" <<EOF > null_byte_injection.csv
    #SELECT DISTINCT parser0, parser1, json, output0, output1
    #FROM results
    #WHERE
    #	(NOT ((parser0 == 'go_gojay' AND root_json != '{"q":2,"q":3}') OR (parser1 == 'go_gojay' AND root_json != '{"q":2,"q":3}'))) AND
    #	((output0 == '2' AND output1 == '3') OR (output0 == '3' AND output1 == '2')) AND
    #	(json LIKE '%"%\u0000%"%' OR json LIKE '%"%\x00%"%')
    #GROUP BY parser0, parser1
    #ORDER BY LENGTH(json)
    #EOF


def print_parser_table():
    parser_lookup = {
        'c_cjson': 'https://github.com/DaveGamble/cJSON',
        'c_frozen': 'https://github.com/cesanta/frozen',
        'c_jansson': 'https://github.com/akheron/jansson',
        'c_jsmn': 'https://github.com/zserge/jsmn',
        'c_json_c': 'https://github.com/json-c/json-c',
        'c_json_parser': 'https://github.com/json-parser/json-parser',
        'c_mjson': 'https://github.com/cesanta/mjson/tree/master',
        'cpp_boost': 'https://github.com/boostorg/boost',
        'cpp_mongoose': 'https://github.com/cesanta/mongoose',
        'cpp_nlohmann': 'https://github.com/nlohmann/json',
        'cpp_poco': 'https://github.com/pocoproject/poco',
        'cpp_yajl': 'https://github.com/lloyd/yajl',
        'dotnet_newtonsoft': 'https://www.newtonsoft.com/json',
        'dotnet_std': 'https://learn.microsoft.com/en-us/dotnet/api/system.text.json',
        'go_fastjson': 'https://github.com/valyala/fastjson',
        'go_gjson': 'https://github.com/tidwall/gjson',
        'go_goccy': 'https://github.com/goccy/go-json',
        'go_gojay': 'https://github.com/francoispqt/gojay',
        'go_json_iterator': 'https://github.com/json-iterator/go',
        'go_jsonparser': 'https://github.com/buger/jsonparser',
        'go_sonnet': 'https://github.com/sugawarayuuta/sonnet',
        'go_std': 'https://pkg.go.dev/encoding/json',
        'java_gson': 'https://github.com/google/gson',
        'java_jackson': 'https://github.com/FasterXML/jackson-databind',
        'java_json': 'https://github.com/stleary/JSON-java',
        'js_json5': 'https://github.com/json5/json5',
        'js_v8': 'https://github.com/v8/v8',
        'lua_cjson': 'https://github.com/openresty/lua-cjson',
        'php_std': 'https://www.php.net/manual/en/book.json.php',
        'postgres': 'https://www.postgresql.org/docs/current/functions-json.html',
        'python_msgspec': 'https://github.com/jcrist/msgspec',
        'python_orjson': 'https://github.com/ijl/orjson',
        'python_rapidjson': 'https://github.com/python-rapidjson/python-rapidjson',
        'python_simplejson': 'https://github.com/simplejson/simplejson',
        'python_std': 'https://docs.python.org/3/library/json.html',
        'python_ujson': 'https://github.com/ultrajson/ultrajson',
        'ruby_std': 'https://docs.ruby-lang.org/en/master/JSON.html',
        'rust_json': 'https://github.com/maciejhirsz/json-rust',
        'rust_jsonc': 'https://github.com/dprint/jsonc-parser',
        'rust_serde': 'https://github.com/serde-rs/json',
        'sqlite3': 'https://sqlite.org/json1.html',
    }

    con = sqlite3.connect('../analyzed/db.sqlite')
    cur = con.cursor()

    cur.execute('''
        SELECT DISTINCT parser0
        FROM results
        UNION
        SELECT DISTINCT parser1
        FROM results
        ''')

    keys = set()

    for row in cur.fetchall():
        keys.add(row[0])

    keys = sorted(keys)

    for key in keys:
        if key not in parser_lookup:
            print(key, 'not in parser_lookup')

    for key in parser_lookup.keys():
        if key not in keys:
            print(key, 'not in results')

    print('LaTex parser table:')
    print()
    print('\\begin{table}[htb]')
    # print('    \\footnotesize')
    print('    \\centering')
    print('    \\caption{Tested JSON libraries}')
    print('    \\label{tbl:parsers}')
    print('    \\begin{tabular}{lll}')
    print('        \\textbf{n} & \\textbf{Parser ID} & \\textbf{Reference URL}\\\\')
    print('        \\hline')

    for i, (key, url) in enumerate(parser_lookup.items()):
        id = key.replace('_', '\\_')
        print(f'        {i + 1} & \\parser{{{id}}} & {{\\scriptsize \\url{{{url}}}}} \\\\')

    print('    \\end{tabular}')
    print('\\end{table}')
    print()
    print(f'Total of {len(parser_lookup)} parsers')


def plot_discrepancies():
    # print(plt.style.available)
    plt.style.use('seaborn-v0_8-paper')

    con = sqlite3.connect('../analyzed/db.sqlite')
    cur = con.cursor()

    cur.execute('''
        -- SELECT DISTINCT parser0, parser1
        -- FROM results
        -- WHERE
        --     (NOT ((parser0 == 'go_gojay' AND root_json != '{"q":2,"q":3}')
        --         OR (parser1 == 'go_gojay' AND root_json != '{"q":2,"q":3}')))
        --     AND ((output0 == '2' AND output1 == '3')
        --         OR (output0 == '3' AND output1 == '2'))
CREATE TEMP TABLE filtered_results AS
	SELECT *
	FROM results
	WHERE
		((output0 == '2' AND output1 == '3') OR (output0 == '3' AND output1 == '2'));
        ''')

    # dotnet_newtonsoft is case insensitive by default. An optimization accidentally made
    # it case sensitive, so this adds the expected results back to the database
    # cur.execute('''INSERT INTO filtered_results VALUES
    #     ('dotnet_newtonsoft', 'c_cjson', '{"q":2,"q":3}', '{"q":2,"Q":3}', '3', '2', '1', 'replace_single', 0);''')
    # cur.execute('''INSERT INTO filtered_results VALUES
    #     ('dotnet_newtonsoft', 'c_frozen', '{"q":2,"q":3}', '{"q":2,"Q":3}', '3', '2', '1', 'replace_single', 0);''')
    # cur.execute('''INSERT INTO filtered_results VALUES
    #     ('dotnet_newtonsoft', 'c_jansson', '{"q":2,"q":3}', '{"q":2,"Q":3}', '3', '2', '1', 'replace_single', 0);''')
    # cur.execute('''INSERT INTO filtered_results VALUES
    #     ('dotnet_newtonsoft', 'c_jsmn', '{"q":2,"q":3}', '{"q":2,"Q":3}', '3', '2', '1', 'replace_single', 0);''')
    # cur.execute('''INSERT INTO filtered_results VALUES
    #     ('dotnet_newtonsoft', 'c_json_c', '{"q":2,"q":3}', '{"q":2,"Q":3}', '3', '2', '1', 'replace_single', 0);''')
    # cur.execute('''INSERT INTO filtered_results VALUES
    #     ('dotnet_newtonsoft', 'c_json_parser', '{"q":2,"q":3}', '{"q":2,"Q":3}', '3', '2', '1', 'replace_single', 0);''')
    # cur.execute('''INSERT INTO filtered_results VALUES
    #     ('dotnet_newtonsoft', 'c_mjson', '{"q":2,"q":3}', '{"q":2,"Q":3}', '3', '2', '1', 'replace_single', 0);''')
    # cur.execute('''INSERT INTO filtered_results VALUES
    #     ('dotnet_newtonsoft', 'cpp_boost', '{"q":2,"q":3}', '{"q":2,"Q":3}', '3', '2', '1', 'replace_single', 0);''')
    # cur.execute('''INSERT INTO filtered_results VALUES
    #     ('dotnet_newtonsoft', 'cpp_mongoose', '{"q":2,"q":3}', '{"q":2,"Q":3}', '3', '2', '1', 'replace_single', 0);''')
    # cur.execute('''INSERT INTO filtered_results VALUES
    #     ('dotnet_newtonsoft', 'cpp_nlohmann', '{"q":2,"q":3}', '{"q":2,"Q":3}', '3', '2', '1', 'replace_single', 0);''')
    # cur.execute('''INSERT INTO filtered_results VALUES
    #     ('dotnet_newtonsoft', 'cpp_poco', '{"q":2,"q":3}', '{"q":2,"Q":3}', '3', '2', '1', 'replace_single', 0);''')
    # cur.execute('''INSERT INTO filtered_results VALUES
    #     ('dotnet_newtonsoft', 'cpp_yajl', '{"q":2,"q":3}', '{"q":2,"Q":3}', '3', '2', '1', 'replace_single', 0);''')
    # cur.execute('''INSERT INTO filtered_results VALUES
    #     ('dotnet_newtonsoft', 'dotnet_std', '{"q":2,"q":3}', '{"q":2,"Q":3}', '3', '2', '1', 'replace_single', 0);''')
    # cur.execute('''INSERT INTO filtered_results VALUES
    #     ('dotnet_newtonsoft', 'go_fastjson', '{"q":2,"q":3}', '{"q":2,"Q":3}', '3', '2', '1', 'replace_single', 0);''')
    # cur.execute('''INSERT INTO filtered_results VALUES
    #     ('dotnet_newtonsoft', 'go_gjson', '{"q":2,"q":3}', '{"q":2,"Q":3}', '3', '2', '1', 'replace_single', 0);''')
    # cur.execute('''INSERT INTO filtered_results VALUES
    #     ('dotnet_newtonsoft', 'go_gojay', '{"q":2,"q":3}', '{"q":2,"Q":3}', '3', '2', '1', 'replace_single', 0);''')
    # cur.execute('''INSERT INTO filtered_results VALUES
    #     ('dotnet_newtonsoft', 'go_jsonparser', '{"q":2,"q":3}', '{"q":2,"Q":3}', '3', '2', '1', 'replace_single', 0);''')
    # cur.execute('''INSERT INTO filtered_results VALUES
    #     ('dotnet_newtonsoft', 'go_sonnet', '{"q":2,"q":3}', '{"q":2,"Q":3}', '3', '2', '1', 'replace_single', 0);''')
    # cur.execute('''INSERT INTO filtered_results VALUES
    #     ('dotnet_newtonsoft', 'java_gson', '{"q":2,"q":3}', '{"q":2,"Q":3}', '3', '2', '1', 'replace_single', 0);''')
    # cur.execute('''INSERT INTO filtered_results VALUES
    #     ('dotnet_newtonsoft', 'java_json', '{"q":2,"q":3}', '{"q":2,"Q":3}', '3', '2', '1', 'replace_single', 0);''')
    # cur.execute('''INSERT INTO filtered_results VALUES
    #     ('dotnet_newtonsoft', 'js_json5', '{"q":2,"q":3}', '{"q":2,"Q":3}', '3', '2', '1', 'replace_single', 0);''')
    # cur.execute('''INSERT INTO filtered_results VALUES
    #     ('dotnet_newtonsoft', 'js_v8', '{"q":2,"q":3}', '{"q":2,"Q":3}', '3', '2', '1', 'replace_single', 0);''')
    # cur.execute('''INSERT INTO filtered_results VALUES
    #     ('dotnet_newtonsoft', 'lua_cjson', '{"q":2,"q":3}', '{"q":2,"Q":3}', '3', '2', '1', 'replace_single', 0);''')
    # cur.execute('''INSERT INTO filtered_results VALUES
    #     ('dotnet_newtonsoft', 'php_std', '{"q":2,"q":3}', '{"q":2,"Q":3}', '3', '2', '1', 'replace_single', 0);''')
    # cur.execute('''INSERT INTO filtered_results VALUES
    #     ('dotnet_newtonsoft', 'postgres', '{"q":2,"q":3}', '{"q":2,"Q":3}', '3', '2', '1', 'replace_single', 0);''')
    # cur.execute('''INSERT INTO filtered_results VALUES
    #     ('dotnet_newtonsoft', 'python_msgspec', '{"q":2,"q":3}', '{"q":2,"Q":3}', '3', '2', '1', 'replace_single', 0);''')
    # cur.execute('''INSERT INTO filtered_results VALUES
    #     ('dotnet_newtonsoft', 'python_orjson', '{"q":2,"q":3}', '{"q":2,"Q":3}', '3', '2', '1', 'replace_single', 0);''')
    # cur.execute('''INSERT INTO filtered_results VALUES
    #     ('dotnet_newtonsoft', 'python_rapidjson', '{"q":2,"q":3}', '{"q":2,"Q":3}', '3', '2', '1', 'replace_single', 0);''')
    # cur.execute('''INSERT INTO filtered_results VALUES
    #     ('dotnet_newtonsoft', 'python_simplejson', '{"q":2,"q":3}', '{"q":2,"Q":3}', '3', '2', '1', 'replace_single', 0);''')
    # cur.execute('''INSERT INTO filtered_results VALUES
    #     ('dotnet_newtonsoft', 'python_std', '{"q":2,"q":3}', '{"q":2,"Q":3}', '3', '2', '1', 'replace_single', 0);''')
    # cur.execute('''INSERT INTO filtered_results VALUES
    #     ('dotnet_newtonsoft', 'python_ujson', '{"q":2,"q":3}', '{"q":2,"Q":3}', '3', '2', '1', 'replace_single', 0);''')
    # cur.execute('''INSERT INTO filtered_results VALUES
    #     ('dotnet_newtonsoft', 'ruby_std', '{"q":2,"q":3}', '{"q":2,"Q":3}', '3', '2', '1', 'replace_single', 0);''')
    # cur.execute('''INSERT INTO filtered_results VALUES
    #     ('dotnet_newtonsoft', 'rust_json', '{"q":2,"q":3}', '{"q":2,"Q":3}', '3', '2', '1', 'replace_single', 0);''')
    # cur.execute('''INSERT INTO filtered_results VALUES
    #     ('dotnet_newtonsoft', 'rust_jsonc', '{"q":2,"q":3}', '{"q":2,"Q":3}', '3', '2', '1', 'replace_single', 0);''')
    # cur.execute('''INSERT INTO filtered_results VALUES
    #     ('dotnet_newtonsoft', 'rust_serde', '{"q":2,"q":3}', '{"q":2,"Q":3}', '3', '2', '1', 'replace_single', 0);''')
    # cur.execute('''INSERT INTO filtered_results VALUES
    #     ('dotnet_newtonsoft', 'sqlite3', '{"q":2,"q":3}', '{"q":2,"Q":3}', '3', '2', '1', 'replace_single', 0);''')

    # expanded results
    cur.execute('''
    SELECT DISTINCT
        parser0, parser1
        --parser0, parser1, root_json, json, output0, output1
    FROM (
        SELECT parser0, parser1, root_json, json, output0, output1
        FROM filtered_results
        UNION ALL
        SELECT parser1 AS parser0, parser0 AS parser1, root_json, json, output1 AS output0, output0 AS output1
        FROM filtered_results
    )
    ORDER BY parser0, parser1, root_json;
        ''')

    data = {}
    keys = set()
    matrix = []

    for row in cur.fetchall():
        if row[0] not in data: data[row[0]] = set()
        if row[1] not in data: data[row[1]] = set()
        data[row[0]].add(row[1])
        data[row[1]].add(row[0])
        keys.add(row[0])
        keys.add(row[1])

    keys = sorted(keys)

    for key in keys:
        matrix.append([k in data[key] for k in keys])

    cmap = ListedColormap(['w', '#19207b'])
    plt.matshow(matrix, cmap=cmap)
    plt.tick_params(axis='both', which='major', labelsize=6)

    x_pos = np.arange(len(keys))
    plt.xticks(x_pos, keys, rotation=90)

    y_pos = np.arange(len(keys))
    plt.yticks(y_pos, keys)
    plt.title('Object value access discrepancies between JSON parsers', loc='center')

    file_name = 'different_key_access.png'
    plt.savefig(file_name, bbox_inches='tight', pad_inches=0.1, dpi=200)
    print('\nSaved plot to', file_name)

    n = 0

    for row in matrix:
        n += sum(row)

    print(f'\nMismatches: {n}  Total combinations: {len(keys)**2}  ({round(n / len(keys)**2 * 100, 1)}%)')


def plot_errors():
    # print(plt.style.available)
    plt.style.use('seaborn-v0_8-paper')

    con = sqlite3.connect('../analyzed/db.sqlite')
    cur = con.cursor()

    cur.execute('''
        SELECT DISTINCT
            parser0,
            parser1,
            (output0 == 'PARSE_ERROR' OR output0 == 'KEY_NOT_FOUND') AS err0,
            (output1 == 'PARSE_ERROR' OR output1 == 'KEY_NOT_FOUND') AS err1
        FROM results
        WHERE
            --root_json == '{"q":2,"q":3}'
        	(((output0 == 'PARSE_ERROR' OR output0 == 'KEY_NOT_FOUND')
                    AND (output1 != 'PARSE_ERROR' AND output1 != 'KEY_NOT_FOUND'))
                OR ((output0 != 'PARSE_ERROR' AND output0 != 'KEY_NOT_FOUND')
                    AND (output1 == 'PARSE_ERROR' OR output1 == 'KEY_NOT_FOUND')))
        ''')

    data = {}
    keys = set()
    matrix = []

    for row in cur.fetchall():
        if row[0] not in data: data[row[0]] = set()
        if row[1] not in data: data[row[1]] = set()
        data[row[0]].add((row[1], row[2], row[3]))
        data[row[1]].add((row[0], row[3], row[2]))
        keys.add(row[0])
        keys.add(row[1])

    keys = sorted(keys)

    for key in keys:
        row = []
        for k in keys:
            a = (k, True, False)
            b = (k, False, True)

            if a in data[key] and b in data[key]:
                row.append(3 / 3)
            elif a in data[key]:
                row.append(2 / 3)
            elif b in data[key]:
                row.append(1 / 3)
            else:
                row.append(0)

        matrix.append(row)

    colors = ['#F5F5F5', '#F15B5B', '#4A90E2', '#8c3b87']
    cmap = ListedColormap(colors)
    plt.matshow(matrix, cmap=cmap)
    plt.tick_params(axis='both', which='major', labelsize=6)

    labels = ["Neither errors\nwhile other parses", "Row errors,\nCol parses", "Col errors,\nRow parses", "Both error while\nother parses"]
    patches = [mpatches.Patch(color=colors[i], label=labels[i]) for i in range(len(colors))]
    plt.legend(handles=patches, bbox_to_anchor=(1.05, 1), loc='upper left', borderaxespad=0.)

    x_pos = np.arange(len(keys))
    plt.xticks(x_pos, keys, rotation=90)

    y_pos = np.arange(len(keys))
    plt.yticks(y_pos, keys)
    plt.title('JSON values rejected by one parser while accepted by others', loc='center')

    file_name = 'errors.png'
    plt.savefig(file_name, bbox_inches='tight', pad_inches=0.1, dpi=200)
    print('\nSaved plot to', file_name)


if __name__ == '__main__':
    print_parser_table()
    plot_discrepancies()
    plot_errors()




    # print('Gitlab Markdown table:')
    # print('|P1|P2|JSON|P1\\["q"]|P2\\["q"]|')
    # print('|-------|-------|----|:-----:|:-----:|')
    #
    # for key in keys:
    #     row = data[key]
    #     row_keys = [c[0] for c in row]
    #     matrix_row = [k in row_keys for k in keys]
    #     matrix.append(matrix_row)
    #
    #     for key1 in sorted(set(row_keys)):
    #         cols = [c for c in row if c[0] == key1]
    #         best = cols[0]
    #
    #         # Find payload with the shortest length
    #         for c in cols:
    #             if len(best[1]) > len(c[1]):
    #                 best = c
    #
    #         l0 = key
    #         l1 = best[0]
    #
    #         if l0 not in parser_lookup:
    #             print(l0, 'not in parser_lookup')
    #             sys.exit(1)
    #
    #         if l1 not in parser_lookup:
    #             print(l1, 'not in parser_lookup')
    #             sys.exit(1)
    #
    #         if l0 in parser_lookup:
    #             l0 = f'[{l0}]({parser_lookup[l0]})'
    #
    #         if l1 in parser_lookup:
    #             l1 = f'[{l1}]({parser_lookup[l1]})'
    #
    #         # print(f'|{l0}|{l1}|<pre lang="json">{best[1]}</pre>|`{best[2]}`|`{best[3]}`|')
    #         print(f'|{l0}|{l1}|{best[1]}|{best[2]}|{best[3]}|')
    #
    # print('')
    # print('')
    # print('LaTex table:')
    # print('\\begin{longtable}{lllll}')
    # print('    P1 & P2 & JSON & P1["q"] & P2["q"] \\\\')
    # print('    \\hline \\\\')
    #
    # for key in keys:
    #     row = data[key]
    #     row_keys = [c[0] for c in row]
    #
    #     for key1 in sorted(set(row_keys)):
    #         cols = [c for c in row if c[0] == key1]
    #         best = cols[0]
    #
    #         # Find payload with the shortest length
    #         for c in cols:
    #             if len(best[1]) > len(c[1]):
    #                 best = c
    #
    #         l0 = key
    #         l1 = best[0]
    #         l0 = l0.replace('_', '\\_')
    #         l1 = l1.replace('_', '\\_')
    #         best[1] = best[1].replace('\\', '\\textbackslash ')
    #         best[1] = best[1].replace('{', '\\{')
    #         best[1] = best[1].replace('}', '\\}')
    #
    #         print(f'    {l0} & {l1} & \\str{{{best[1]}}} & \\str{{{best[2]}}} & \\str{{{best[3]}}} \\\\')
    # print('    &&&&')
    # print('\\end{longtable}')
    #
    # print('')
    # print('')
