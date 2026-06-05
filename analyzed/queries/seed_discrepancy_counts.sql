-- how many discrepancies each seed produced
SELECT DISTINCT
	COUNT(1), root_json
FROM results
WHERE ((output0 == '2' AND output1 == '3') OR (output0 == '3' AND output1 == '2'))
	-- removes some duplicates
	AND json NOT LIKE '% %'
	AND json NOT LIKE '%\n%'
	AND json NOT LIKE '%\t%'
	AND json NOT LIKE '\f%'
	AND json NOT LIKE '\v%'
	AND json NOT LIKE '\b%'
	AND json NOT LIKE '\r%'
	AND json NOT LIKE '%\r'
	AND json NOT LIKE '%3\r%'
	AND json NOT LIKE '%3e%'
	AND json NOT LIKE '%3E%'
	AND json NOT LIKE '%2e%'
	AND json NOT LIKE '%2E%'
	AND json NOT LIKE '%3.%'
	AND json NOT LIKE '%+3%'
	AND json NOT LIKE '%2.%'
	AND json NOT LIKE '%+2%'
	AND json NOT LIKE '%02%'
	AND json NOT LIKE '%03%'
GROUP BY root_json;
