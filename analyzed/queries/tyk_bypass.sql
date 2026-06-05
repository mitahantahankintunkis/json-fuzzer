PRAGMA case_sensitive_like = true;

-- filtered discrepancies
CREATE TEMP TABLE filtered_results AS
	SELECT *
	FROM results
	WHERE
		((output0 == '2' AND output1 == '3') OR (output0 == '3' AND output1 == '2'))
		AND (parser0 == 'go_std' OR parser1 == 'go_std')
		AND root_json = '{"0":2,"0":3}';
		-- AND ((parser0 == 'js_v8' OR parser1 == 'js_v8') OR (parser0 == 'rust_serde' OR parser1 == 'rust_serde'))
		-- AND root_json = '{"q":2,"q":3}';

CREATE TEMP TABLE key_precedence AS
	SELECT DISTINCT parser0, parser1, json, output0, output1
	FROM filtered_results
	WHERE root_json == json
	UNION
	SELECT DISTINCT parser1 AS parser0, parser0 AS parser1, json, output1 AS output0, output0 AS output1
	FROM filtered_results
	WHERE root_json == json;

-- actual results
CREATE TEMP TABLE results AS
	SELECT
		parser0 AS p0, parser1 AS p1, root_json, json, output0 AS o0, output1 AS o1,
		RANK() OVER ( PARTITION BY parser0, parser1, root_json ORDER BY LENGTH(json), json) AS rank
	FROM filtered_results AS a
	WHERE 1==1
		-- remove results that are equal with results from simple duplicated keys
		AND NOT EXISTS (SELECT 1 FROM key_precedence AS b WHERE a.parser0 == b.parser0 AND a.parser1 == b.parser1 LIMIT 1)
		;


-- expanded results
SELECT DISTINCT
	p0 AS parser0, p1 AS parser1, root_json AS seed, REPLACE(json, '\xf0\x9f\x98\x80', '😀') AS testcase, o0 AS output0, o1 AS output1
FROM (
	-- discrepancies
	SELECT p0, p1, root_json, json, o0, o1, rank
	FROM temp.results
	UNION ALL
	SELECT p1 AS p0, p0 AS p1, root_json, json, o1 AS o0, o0 AS o1, rank
	FROM temp.results

	-- duplicate keys
	UNION ALL
	SELECT parser0 AS p0, parser1 AS p1, json AS root_json, json, output0 AS o0, output1 AS o1, 0 AS rank
	FROM key_precedence
	WHERE (parser0 == 'go_std' OR parser1 == 'go_std')
	-- WHERE ((parser0 == 'js_v8' OR parser1 == 'js_v8') OR (parser0 == 'rust_serde' OR parser1 == 'rust_serde'))
)
WHERE rank < 100
ORDER BY p0, p1, root_json;

-- root_json discrepancy counts
--SELECT DISTINCT
--	COUNT(1), root_json
--FROM results
--WHERE ((output0 == '2' AND output1 == '3') OR (output0 == '3' AND output1 == '2'))
--GROUP BY root_json;
