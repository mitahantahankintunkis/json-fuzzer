#!/bin/bash

docker build . -t fuzzer

docker run \
	-it \
	--rm \
	--name fuzzer \
	-v ./analyzed:/app/analyzed \
	-v ./data:/app/data \
	fuzzer
