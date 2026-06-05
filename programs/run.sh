#!/bin/env bash

(trap 'kill 0' SIGINT
      # cpp-client 1 &
      #valgrind --tool=massif  cpp-client 1 &
    nvm install 24
    service postgresql start

    for parser_number in {0..20}; do
        rust-client $parser_number 2> client_log &
        go-client $parser_number 2> client_log &
        cpp-client $parser_number 2> client_log &
        ./main.lua $parser_number 2> client_log &
        java -jar java-client.jar $parser_number 2> client_log &
        php main.php $parser_number 2> client_log &
        ruby main.rb $parser_number 2> client_log &
        python3 ./python/main.py $parser_number 2> client_log &
        ./dotnet_client/dotnet_client $parser_number 2> client_log &
        node ./js_client/index.js $parser_number 2> client_log &
    done

    json-fuzzer "$@" 2> log
wait)

