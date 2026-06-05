-- how many discrepancies each seed produced

CREATE TEMP TABLE filtered_results AS
	SELECT *
	FROM results
	WHERE output0 == 'UTF8_ERROR' OR output1 == 'UTF8_ERROR';



-- actual results
CREATE TEMP TABLE results AS
	SELECT
		parser0 AS p0, parser1 AS p1, root_json, json, output0 AS o0, output1 AS o1,
		RANK() OVER ( PARTITION BY parser0, parser1, root_json ORDER BY LENGTH(json), json) AS rank
	FROM filtered_results AS a;

-- expanded results
SELECT DISTINCT
	p0, json
FROM (
	SELECT p0, root_json, json, o0, rank
	FROM temp.results
	WHERE o0 == 'UTF8_ERROR'

	UNION ALL
	SELECT p1 AS p0, root_json, json, o1 AS o0, rank
	FROM temp.results
	WHERE o1 == 'UTF8_ERROR'
)
WHERE rank < 10
ORDER BY p0, json;
