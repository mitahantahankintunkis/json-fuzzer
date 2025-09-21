#!/bin/env bash

(trap 'kill 0' SIGINT
    for parser_number in {0..10}; do
		rust-client $parser_number &
        go-client $parser_number &
        cpp-client $parser_number &
		./main.lua $parser_number &
		java -jar clojure-client.jar $parser_number &
		php main.php $parser_number &
        # valgrind --leak-check=full -s ./client $parser_number &
        # valgrind --leak-check=full -s cpp_client $parser_number &
    done

    # cd ../python/
    # source venv/bin/activate
    #
    # for parser_number in {0..5}; do
    #     python3 main.py $parser_number &
    #     sleep 0.1
    #
    #     if [ $? -eq 1 ]; then
    #         echo "Started ${parser_number} python parsers"
    #         break
    #     fi
    # done

	json-fuzzer
wait)

# cd fuzzer/
#
# cd ../fuzzer/
# echo -e "Building fuzzer..."
# cargo build -r
#
# cd ../clients/rust/
# echo -e "Building rust..."
# cargo build -r
#
# cd ../go/
# echo -e "Building go..."
# go build main.go
#
# cd ../c/
# echo -e "Building C..."
# make
#
# cd ../../fuzzer/
#
# (trap 'kill 0' SIGINT
#     cd ../clients/rust/
#     for parser_number in {0..1}; do
#         cargo run -q -r -- $parser_number &
#         # sleep 0.1
#
#         if [ $? -eq 1 ]; then
#             echo "Started ${parser_number} rust parsers"
#             break
#         fi
#     done
#
#     cd ../go/
#     for parser_number in {0..7}; do
#         go run main.go $parser_number &
#         # sleep 0.1
#
#         if [ $? -eq 1 ]; then
#             echo "Started ${parser_number} go parsers"
#             break
#         fi
#     done
#
#     cd ../c/
#     for parser_number in {0..3}; do
#         ./client $parser_number &
#     done
#
#     cd ../python/
#     source venv/bin/activate
#
#     for parser_number in {0..5}; do
#         python3 main.py $parser_number &
#
#         if [ $? -eq 1 ]; then
#             echo "Started ${parser_number} python parsers"
#             break
#         fi
#     done
#
#     sleep 1
#
#     # cd ../echo/
#     # echo "Start echo client"
#     # cargo run -r &
#     # payload=${1:-'{"q":1,"q":2}'}
#     #payload='{"q":1\"q":2}'
#     # payload='{"q":1}'
#     # echo -e "\n\nFuzzing $payload\n"
#     # cargo run -q -r -- --payload $payload &
#     echo -e "\n\nFuzzing\n"
#     cd ../../fuzzer/
#     cargo run -q -r &
# wait)

