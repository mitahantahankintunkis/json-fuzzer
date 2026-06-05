-- filtered discrepancies
CREATE TEMP TABLE filtered_results AS
	SELECT *
	FROM results
	WHERE
		((output0 == '2' AND output1 == '3') OR (output0 == '3' AND output1 == '2'))
		AND root_json = '{"q":2,"q":3}';


-- actual results
CREATE TEMP TABLE results AS
	SELECT
		parser0 AS p0, parser1 AS p1, root_json, json, output0 AS o0, output1 AS o1,
		RANK() OVER ( PARTITION BY parser0, parser1, root_json ORDER BY LENGTH(json), json) AS rank
	FROM filtered_results AS a
	WHERE
		-- remove results that are equal with results from simple duplicated keys
		root_json NOT IN (
			SELECT json
			FROM filtered_results AS b
			WHERE a.parser0 == b.parser0 AND a.output0 == b.output0 AND a.parser1 == b.parser1 AND a.output1 == b.output1
		)
		AND ((json LIKE '%"q_"%' OR json LIKE '%"_q"%')
			OR (json LIKE '%"q\x__"%' OR json LIKE '%"\x__q"%'));

		-- removes some duplicates
		-- AND json NOT LIKE '% %'
		-- AND json NOT LIKE '%\n%'
		-- AND json NOT LIKE '%\t%'
		-- AND json NOT LIKE '\f%'
		-- AND json NOT LIKE '\v%'
		-- AND json NOT LIKE '\b%'
		-- AND json NOT LIKE '\r%'
		-- AND json NOT LIKE '%\r'
		-- AND json NOT LIKE '%3\r%'
		-- AND json NOT LIKE '%3e%'
		-- AND json NOT LIKE '%3E%'
		-- AND json NOT LIKE '%2e%'
		-- AND json NOT LIKE '%2E%'
		-- AND json NOT LIKE '%3.%'
		-- AND json NOT LIKE '%+3%'
		-- AND json NOT LIKE '%2.%'
		-- AND json NOT LIKE '%+2%'
		-- AND json NOT LIKE '%02%'
		-- AND json NOT LIKE '%03%';

		-- AND json LIKE '%\u%'
		-- AND json NOT LIKE '%\u0000%'
		-- AND json NOT LIKE '%\u0071%'
		-- AND json NOT LIKE '%\u0051%'
		-- AND json NOT LIKE '%\u0030%'
	--ORDER BY parser0, parser1, root_json;

-- expanded results
SELECT DISTINCT
	p0, p1, root_json, json, o0, o1
FROM (
	SELECT *
	FROM temp.results
	UNION ALL
	SELECT p1 AS p0, p0 AS p1, root_json, json, o1 AS o0, o0 AS o1, rank
	FROM temp.results
)
WHERE rank < 10
ORDER BY p0, p1, root_json;

-- root_json discrepancy counts
--SELECT DISTINCT
--	COUNT(1), root_json
--FROM results
--WHERE ((output0 == '2' AND output1 == '3') OR (output0 == '3' AND output1 == '2'))
--GROUP BY root_json;
