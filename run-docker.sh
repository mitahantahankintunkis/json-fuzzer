#!/bin/bash

docker compose up -d --no-deps --build fuzzer && docker compose exec fuzzer bash

#docker build . -t fuzzer
#
#docker run \
#	-it \
#	--rm \
#	--name fuzzer \
#	-v ./analyzed:/app/analyzed \
#	fuzzer
