#!/bin/env bash

cd json-fuzzer/

(trap 'kill 0' SIGINT
    cd ../rust/
    echo "Start rust parser '0'"
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

    cd ../test-server/
    cargo build -r
    cd target/release/
    ./test-server
wait)

