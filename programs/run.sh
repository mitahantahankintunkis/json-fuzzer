#!/bin/env bash

(trap 'kill 0' SIGINT
      # valgrind --tool=massif  cpp-client 1 &

    for parser_number in {0..20}; do
        # rust-client $parser_number &
        # go-client $parser_number &
        cpp-client $parser_number &
        # ./main.lua $parser_number &
        # java -jar java-client.jar $parser_number &
        # php main.php $parser_number &
        # ruby main.rb $parser_number &
        # python3 ./python/main.py $parser_number &
        # ./dotnet_client/dotnet_client $parser_number &
    done

    # json-fuzzer -f radamsa
    json-fuzzer "$@"
wait)

