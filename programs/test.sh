#!/bin/env bash

# json-fuzzer --debug data/*test*
# rm data/*test*

(trap 'kill 0' SIGINT
    for parser_number in {0..10}; do
		rust-client $parser_number &
        cpp-client $parser_number &
		go-client $parser_number &
		./main.lua $parser_number &
        java -jar java-client.jar $parser_number &
		php main.php $parser_number &
		ruby main.rb $parser_number &
		python3 python/main.py $parser_number &
		./dotnet_client/dotnet_client $parser_number &

		#java -jar clojure-client.jar $parser_number &
        # valgrind --leak-check=full -s ./client $parser_number &
        # valgrind --leak-check=full -s cpp_client $parser_number &
    done

    test-server "$@"
wait)

