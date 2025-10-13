#!/bin/env bash

(trap 'kill 0' SIGINT
    for parser_number in {0..10}; do
		rust-client $parser_number &
        cpp-client $parser_number &
		go-client $parser_number &
		./main.lua $parser_number &
		java -jar clojure-client.jar $parser_number &
		php main.php $parser_number &
		ruby main.rb $parser_number &
		python3 python/main.py $parser_number &
		./dotnet_client/dotnet_client $parser_number &

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

    test-server
wait)

# (trap 'kill 0' SIGINT
#     echo "Starting clients..."
#
#     cd clients/rust/
#     cargo build -r
#     cargo run -q -r &
#
#     cd ../../clients/go/
#     # go run main.go 4 &
#     for parser_number in {0..7}; do
#         if [[ $parser_number -eq 5 ]]; then
#             continue
#         fi
#         # echo "Start go parser '$parser_number'"
#         go run main.go $parser_number &
#     done
#
#     cd ../../clients/python/
#     python3 main.py &
# wait)

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

# cd ../c/
# echo -e "Building C..."
# make

# cd ../../fuzzer/

# (trap 'kill 0' SIGINT
#     cd ../clients/rust/
#     for parser_number in {0..1}; do
#         cargo run -q -r -- $parser_number &
#         sleep 0.1
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
#         sleep 0.1
#
#         if [ $? -eq 1 ]; then
#             echo "Started ${parser_number} go parsers"
#             break
#         fi
#     done
#
#     cd ../c/
#     for parser_number in {0..3}; do
#         valgrind --leak-check=full -s ./client $parser_number &
#     done
#
#     cd ../cpp/
#     for parser_number in {0..1}; do
#         valgrind --leak-check=full -s ./build/cpp_client $parser_number &
#     done
#
#     cd ../python/
#     source venv/bin/activate
#
#     for parser_number in {0..5}; do
#         python3 main.py $parser_number &
#         sleep 0.1
#
#         if [ $? -eq 1 ]; then
#             echo "Started ${parser_number} python parsers"
#             break
#         fi
#     done
#
#     cd ../../test-server/
#     cargo build -r
#     cd target/release/
#     ./test-server
# wait)

# (trap 'kill 0' SIGINT
#     echo "Starting clients..."
#
#     cd clients/rust/
#     cargo build -r
#     cargo run -q -r &
#
#     cd ../../clients/go/
#     # go run main.go 4 &
#     for parser_number in {0..7}; do
#         if [[ $parser_number -eq 5 ]]; then
#             continue
#         fi
#         # echo "Start go parser '$parser_number'"
#         go run main.go $parser_number &
#     done
#
#     cd ../../clients/python/
#     python3 main.py &
# wait)

