#!/bin/bash

echo "Brute forcing passwords from '0000' to '9999' for user 'admin'..."

for i in {0..9999}; do
	printf -v pwd '%04d' $i

	echo -ne "Testing '$pwd'...    \r"

	status=$(curl localhost:8080/api/login \
		-H "Content-Type: application/json" \
		-d "{\"username\":\"admin\",\"password\":\"$pwd\"}"\
		-w "%{response_code}"\
		-s -o /dev/null)

	if [ $status == 200 ]; then
		echo -e "\nLogin succesfull with password $pwd"
		break
	fi

	if [ $status == 429 ]; then
		echo -e "\nError: Rate limited after $i attempts. Stopping\n"
		break
	fi
done
