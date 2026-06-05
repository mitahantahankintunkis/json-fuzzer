#!/bin/env bash

# sudo sqlite3 db.sqlite -header -cmd ".mode tabs" "select distinct parser0, parser1, json, output0, output1 from results where output0 != 'PARSE_ERROR' and output0 != 'PARSE_ERROR' and output1 != 'PARSE_ERROR' and output1 != 'PARSE_ERROR'"

# sudo sqlite3 db.sqlite -header -cmd ".mode tabs" <<EOF
# SELECT distinct parser0, parser1, json, output0, output1
# FROM results
# WHERE output0 != 'PARSE_ERROR' and output0 != 'PARSE_ERROR' and output1 != 'PARSE_ERROR' and output1 != 'PARSE_ERROR' AND
# (parser0 == 'cpp_yajl' OR parser1 == 'cpp_yajl') AND
# ((output0 == '2' AND output1 == '3') OR (output0 == '3' AND output1 == '2'))
# GROUP BY parser0, parser1
# ORDER BY LENGTH(json)
# EOF
#
# echo ""

# sudo sqlite3 db.sqlite -header -cmd ".mode tabs" <<EOF
# SELECT distinct parser0, parser1
# FROM results
# WHERE output0 != 'PARSE_ERROR' and output0 != 'PARSE_ERROR' and output1 != 'PARSE_ERROR' and output1 != 'PARSE_ERROR' AND
# (parser0 == 'cpp_yajl' OR parser1 == 'cpp_yajl') AND
# ((output0 == '2' AND output1 == '3') OR (output0 == '3' AND output1 == '2'))
# EOF

# sudo sqlite3 db.sqlite -header -cmd ".mode tabs" <<EOF
# SELECT distinct parser0, parser1, json, output0, output1
# FROM results
# WHERE output0 != 'PARSE_ERROR' and output0 != 'PARSE_ERROR' and output1 != 'PARSE_ERROR' and output1 != 'PARSE_ERROR' AND
# -- (parser0 == 'cpp_yajl' OR parser1 == 'cpp_yajl') AND
# ((output0 == '2' AND output1 == '3') OR (output0 == '3' AND output1 == '2'))
# GROUP BY parser0, parser1
# ORDER BY LENGTH(json)
# EOF
#
#
#
# (sqlite3 ../analyzed/db.sqlite -cmd ".mode tabs" <<EOF > duplicate_key_access.csv
# --SELECT DISTINCT json
# --SELECT distinct parser0, parser1, json, output0, output1
# SELECT DISTINCT parser0, parser1, json
# FROM results
# WHERE
# ((output0 == '2' AND output1 == '3') OR (output0 == '3' AND output1 == '2'))
# -- output0 != 'PARSE_ERROR' and output0 != 'PARSE_ERROR' and output1 != 'PARSE_ERROR' and output1 != 'PARSE_ERROR' AND
# -- (parser0 == 'cpp_yajl' OR parser1 == 'cpp_yajl') AND
# -- (parser0 == 'go_std' OR parser1 == 'go_std') AND
# -- (parser0 == 'java_jackson' OR parser1 == 'java_jackson') AND
# -- ((output0 == '2' AND output1 == '3') OR (output0 == '3' AND output1 == '2'))
# -- AND (json LIKE '%ff%' OR json LIKE '%c1%')
# GROUP BY parser0, parser1
# ORDER BY LENGTH(json)
# EOF)


sqlite3 ../analyzed/db.sqlite -cmd ".mode tabs" <<EOF > duplicate_key_access.csv
SELECT DISTINCT parser0, parser1, json, output0, output1
FROM results
WHERE
	(NOT ((parser0 == 'go_gojay' AND root_json != '{"q":2,"q":3}') OR (parser1 == 'go_gojay' AND root_json != '{"q":2,"q":3}'))) AND
	((output0 == '2' AND output1 == '3') OR (output0 == '3' AND output1 == '2'))
GROUP BY parser0, parser1
ORDER BY LENGTH(json)
EOF


sqlite3 ../analyzed/db.sqlite -cmd ".mode tabs" <<EOF > utf8_errors.csv
SELECT DISTINCT *
FROM (
	SELECT DISTINCT parser0 AS parser, json, output0 AS output
	FROM results
	UNION
	SELECT DISTINCT parser1 AS parser, json, output1 AS output
	FROM results
)
WHERE output == 'UTF8_ERROR'
GROUP BY parser
ORDER BY LENGTH(json)
EOF


sqlite3 ../analyzed/db.sqlite -cmd ".mode tabs" <<EOF > error_discrepancy.csv
SELECT DISTINCT parser0, parser1, json, output0, output1
FROM results
WHERE
	--(parser0 == 'sqlite3' OR parser1 == 'sqlite3') AND
	--(parser0 LIKE 'cpp%' OR parser1 LIKE 'cpp*') AND
	(((output0 == 'PARSE_ERROR' OR output0 == 'KEY_NOT_FOUND') AND (output1 != 'PARSE_ERROR' AND output1 != 'KEY_NOT_FOUND')) OR
	((output0 != 'PARSE_ERROR' AND output0 != 'KEY_NOT_FOUND') AND (output1 == 'PARSE_ERROR' OR output1 == 'KEY_NOT_FOUND')))
GROUP BY parser0, parser1
ORDER BY LENGTH(json)
EOF


sqlite3 ../analyzed/db.sqlite -cmd ".mode tabs" <<EOF > null_byte_injection.csv
SELECT DISTINCT parser0, parser1, json, output0, output1
FROM results
WHERE
	--(NOT ((parser0 == 'go_gojay' AND root_json != '{"q":2,"q":3}') OR (parser1 == 'go_gojay' AND root_json != '{"q":2,"q":3}'))) AND
	((output0 == '2' AND output1 == '3') OR (output0 == '3' AND output1 == '2')) AND
	(json LIKE '%"%\u0000%"%' OR json LIKE '%"%\x00%"%')
GROUP BY parser0, parser1
ORDER BY LENGTH(json)
EOF
