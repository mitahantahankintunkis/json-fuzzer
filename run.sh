#!/bin/env bash

cd json-fuzzer/

(trap 'kill 0' SIGINT;
    cd ../json-fuzzer/;
    cargo build -r
    echo "Start server on port 5000"
    cargo run -r &

    sleep 1;

    cd ../echo/;
    echo "Start echo client"
    cargo run -r &

    # cd ../echo/;
    # echo "Start echo client"
    # cargo run -r &
    #
    # cd ../echo/;
    # echo "Start echo client"
    # cargo run -r &
    #
    # cd ../echo/;
    # echo "Start echo client"
    # cargo run -r &
    #
    # cd ../echo/;
    # echo "Start echo client"
    # cargo run -r &
    #
    # cd ../echo/;
    # echo "Start echo client"
    # cargo run -r &
    #
    # cd ../echo/;
    # echo "Start echo client"
    # cargo run -r &
    #
    # cd ../echo/;
    # echo "Start echo client"
    # cargo run -r &
    #
    # cd ../echo/;
    # echo "Start echo client"
    # cargo run -r &
    #
    # cd ../echo/;
    # echo "Start echo client"
    # cargo run -r &
wait)

