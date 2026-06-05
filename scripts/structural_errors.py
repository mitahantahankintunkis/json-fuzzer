#!/bin/env python3
import sqlite3
import json


def decode_escaped_bytes(s: str) -> bytes:
    out = bytearray()
    i = 0

    while i < len(s):
        if (
            s[i] == "\\"
            and i + 3 < len(s)
            and s[i + 1] == "x"
        ):
            hex_part = s[i + 2:i + 4]
            try:
                out.append(int(hex_part, 16))
                i += 4
                continue
            except ValueError:
                pass

        if s[i:i + 2] == "\\a":
            out.append(0x07)
            i += 2
            continue

        if s[i:i + 2] == "\\b":
            out.append(0x08)
            i += 2
            continue

        if s[i:i + 2] == "\\t":
            out.append(0x09)
            i += 2
            continue

        if s[i:i + 2] == "\\n":
            out.append(0x0a)
            i += 2
            continue

        if s[i:i + 2] == "\\v":
            out.append(0x0b)
            i += 2
            continue

        if s[i:i + 2] == "\\f":
            out.append(0x0c)
            i += 2
            continue

        if s[i:i + 2] == "\\r":
            out.append(0x0d)
            i += 2
            continue

        # Leave everything else unchanged
        out.extend(s[i].encode("utf-8"))
        i += 1

    return bytes(out)


con = sqlite3.connect('../analyzed/db.sqlite')
cur = con.cursor()

with open('../analyzed/discrepancies.sql', 'r') as f:
    discrepancies_sql = f.read();

cur.executescript(discrepancies_sql)

cur.execute(r'''
SELECT DISTINCT
	p0 AS parser0, p1 AS parser1, root_json AS seed, REPLACE(json, '\xf0\x9f\x98\x80', '😀') AS testcase, o0 AS output0, o1 AS output1
FROM (
	-- discrepancies
	SELECT p0, p1, root_json, json, o0, o1, rank
	FROM temp.results
	UNION ALL
	SELECT p1 AS p0, p0 AS p1, root_json, json, o1 AS o0, o0 AS o1, rank
	FROM temp.results
)
WHERE rank < 10
ORDER BY p0, p1, root_json;
''')

# ('sqlite3', 'ruby_std', '{"q":2,"q":3}', '{"q":2,"q":3}', '2', '3')
rows = cur.fetchall()
max_len = 0

for row in rows:
    if max_len < len(row[0]): max_len = len(row[0])
    if max_len < len(row[1]): max_len = len(row[1])

for row in rows:
        #b = row[3].encode().decode('unicode_escape').encode('latin-1')

    b = decode_escaped_bytes(row[3])
    try:
        json.loads(b)
    except:
        print(f'{row[0]:<{max_len}} {row[1]:<{max_len}}\t{row[3]}\t{row[4]}\t{row[5]}')

