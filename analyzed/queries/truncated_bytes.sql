-- filtered discrepancies
CREATE TEMP TABLE filtered_results AS
	SELECT *
	FROM results
	WHERE
		((output0 == '2' AND output1 == '3') OR (output0 == '3' AND output1 == '2'))
		AND root_json = '{"q":2,"q":3}'; -- OR root_json = '{"0":2,"0":3}';

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

		AND (false
		  OR json LIKE '{"\x__\x__\x__\x__q":2,"q":3}'
		  OR json LIKE '{"\x__\x__\x__q":2,"q":3}'
		  OR json LIKE '{"\x__\x__q":2,"q":3}'
		  OR json LIKE '{"\x__q":2,"q":3}'
		  OR json LIKE '{"____q":2,"q":3}'
		  OR json LIKE '{"___q":2,"q":3}'
		  OR json LIKE '{"__q":2,"q":3}'
		  OR json LIKE '{"_q":2,"q":3}'
		  OR json LIKE '{"q\x__":2,"q":3}'
		  OR json LIKE '{"q\x__\x__":2,"q":3}'
		  OR json LIKE '{"q\x__\x__\x__":2,"q":3}'
		  OR json LIKE '{"q\x__\x__\x__\x__":2,"q":3}'
		  OR json LIKE '{"q_":2,"q":3}'
		  OR json LIKE '{"q__":2,"q":3}'
		  OR json LIKE '{"q___":2,"q":3}'
		  OR json LIKE '{"q____":2,"q":3}'

		  OR json LIKE '{"q":2,"\x__\x__\x__\x__q":3}'
		  OR json LIKE '{"q":2,"\x__\x__\x__q":3}'
		  OR json LIKE '{"q":2,"\x__\x__q":3}'
		  OR json LIKE '{"q":2,"\x__q":3}'
		  OR json LIKE '{"q":2,"____q":3}'
		  OR json LIKE '{"q":2,"___q":3}'
		  OR json LIKE '{"q":2,"__q":3}'
		  OR json LIKE '{"q":2,"_q":3}'
		  OR json LIKE '{"q":2,"q\x__":3}'
		  OR json LIKE '{"q":2,"q\x__\x__":3}'
		  OR json LIKE '{"q":2,"q\x__\x__\x__":3}'
		  OR json LIKE '{"q":2,"q\x__\x__\x__\x__":3}'
		  OR json LIKE '{"q":2,"q_":3}'
		  OR json LIKE '{"q":2,"q__":3}'
		  OR json LIKE '{"q":2,"q___":3}'
		  OR json LIKE '{"q":2,"q____":3}'
		)

		-- -- Null byte injection
		-- AND json NOT LIKE '%q\u0000%"'
		-- AND json NOT LIKE '%0\u0000%"'
		-- AND json NOT LIKE '%\x80\u0000%"'

		-- -- removes some duplicates
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

		-- -- Numbers
		-- AND json NOT LIKE '%+2%'
		-- AND json NOT LIKE '%+3%'
		-- AND json NOT LIKE '%02%'
		-- AND json NOT LIKE '%03%'
		-- AND json NOT LIKE '%2.%'
		-- AND json NOT LIKE '%2E%'
		-- AND json NOT LIKE '%2e%'
		-- AND json NOT LIKE '%3.%'
		-- AND json NOT LIKE '%3E%'
		-- AND json NOT LIKE '%3e%'

		-- -- Whitespace and control characters
		-- AND json NOT LIKE '% %'
		-- AND json NOT LIKE '%"\a%'
		-- AND json NOT LIKE '%"\b%'
		-- AND json NOT LIKE '%,\f%'
		-- AND json NOT LIKE '%,\r%'
		-- AND json NOT LIKE '%,\v%'
		-- AND json NOT LIKE '%2\a%'
		-- AND json NOT LIKE '%2\b%'
		-- AND json NOT LIKE '%2\f%'
		-- AND json NOT LIKE '%2\r%'
		-- AND json NOT LIKE '%2\v%'
		-- AND json NOT LIKE '%3\a%'
		-- AND json NOT LIKE '%3\b%'
		-- AND json NOT LIKE '%3\b%'
		-- AND json NOT LIKE '%3\f%'
		-- AND json NOT LIKE '%3\r%'
		-- AND json NOT LIKE '%3\r%'
		-- AND json NOT LIKE '%3\v%'
		-- AND json NOT LIKE '%:\f%'
		-- AND json NOT LIKE '%:\r%'
		-- AND json NOT LIKE '%:\v%'
		-- AND json NOT LIKE '%\a2%'
		-- AND json NOT LIKE '%\a3%'
		-- AND json NOT LIKE '%\b2%'
		-- AND json NOT LIKE '%\b3%'
		-- AND json NOT LIKE '%\f"%'
		-- AND json NOT LIKE '%\f,%'
		-- AND json NOT LIKE '%\f2%'
		-- AND json NOT LIKE '%\f3%'
		-- AND json NOT LIKE '%\f:%'
		-- AND json NOT LIKE '%\n%'
		-- AND json NOT LIKE '%\r"%'
		-- AND json NOT LIKE '%\r"%'
		-- AND json NOT LIKE '%\r'
		-- AND json NOT LIKE '%\r,%'
		-- AND json NOT LIKE '%\r2%'
		-- AND json NOT LIKE '%\r3%'
		-- AND json NOT LIKE '%\r:%'
		-- AND json NOT LIKE '%\t%'
		-- AND json NOT LIKE '%\v"%'
		-- AND json NOT LIKE '%\v,%'
		-- AND json NOT LIKE '%\v2%'
		-- AND json NOT LIKE '%\v3%'
		-- AND json NOT LIKE '%\v:%'
		-- AND json NOT LIKE '%{\a%'
		-- AND json NOT LIKE '%{\b%'
		-- AND json NOT LIKE '\b%'
		-- AND json NOT LIKE '\f%'
		-- AND json NOT LIKE '\r%'
		-- AND json NOT LIKE '\v%'

		-- -- Trailing characters
		-- AND json NOT LIKE '%}_'
		-- AND json NOT LIKE '%}__'
		-- AND json NOT LIKE '%}\x__'
		-- AND json NOT LIKE '%}\x__\x__'
		-- AND json NOT LIKE '%}\u____'
		-- AND json NOT LIKE '%}\u____\u____'
		-- AND json NOT LIKE '%}_\x__'
		-- AND json NOT LIKE '%}\x___'
		-- AND json NOT LIKE '%}\_\x__'
		-- AND json NOT LIKE '%}\x__\_'
		-- AND json NOT LIKE '%}\__'
		-- AND json NOT LIKE '%}_\_'

		-- AND json NOT LIKE '%03%';

		-- AND json LIKE '%\u%'
		-- AND json NOT LIKE '%\u0000%'
		-- AND json NOT LIKE '%\u0071%'
		-- AND json NOT LIKE '%\u0051%'
		-- AND json NOT LIKE '%\u0030%'
		;
	--ORDER BY parser0, parser1, root_json;


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
)
WHERE rank < 100
ORDER BY p0, p1, root_json;

-- root_json discrepancy counts
--SELECT DISTINCT
--	COUNT(1), root_json
--FROM results
--WHERE ((output0 == '2' AND output1 == '3') OR (output0 == '3' AND output1 == '2'))
--GROUP BY root_json;
