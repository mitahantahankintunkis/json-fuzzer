#!/bin/env bash

cd json-fuzzer/

(trap 'kill 0' SIGINT
    cd ../json-fuzzer/
    cargo build -r

    payload='{"q":1,"q":2}'
    # payload='{"q":1}'
    echo -e "\n\nFuzzing $payload\n"
    cargo run -q -r -- --payload $payload &

    sleep 1

    cd ../rust/
    cargo run -q -r &

    cd ../go/
    # go run main.go 4 &
    for parser_number in {0..7}; do
        if [[ $parser_number -eq 5 ]]; then
            continue
        fi
        echo "Start go parser '$parser_number'"
        go run main.go $parser_number &
    done

    # cd ../echo/
    # echo "Start echo client"
    # cargo run -r &

    # cd ../echo/
    # echo "Start echo client"
    # cargo run -r &
    #
    # cd ../echo/
    # echo "Start echo client"
    # cargo run -r &
    #
    # cd ../echo/
    # echo "Start echo client"
    # cargo run -r &
    #
    # cd ../echo/
    # echo "Start echo client"
    # cargo run -r &
    #
    # cd ../echo/
    # echo "Start echo client"
    # cargo run -r &
    #
    # cd ../echo/
    # echo "Start echo client"
    # cargo run -r &
    #
    # cd ../echo/
    # echo "Start echo client"
    # cargo run -r &
    #
    # cd ../echo/
    # echo "Start echo client"
    # cargo run -r &
    #
    # cd ../echo/
    # echo "Start echo client"
    # cargo run -r &
wait)

