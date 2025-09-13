#!/bin/bash

echo "Send config"

config=`(cat ./openapi.json)`
host='http://localhost:8080'
auth='x-tyk-authorization: foo'

curl --location --request POST "${host}/tyk/apis/oas" \
	--header "$auth" \
	--header "Content-Type: text/plain" \
	--data-raw "$config"

echo -e "\nRestart Tyk"
curl -H "$auth" -s "${host}/tyk/reload/group"
