# Rust
FROM rust:latest AS rust

WORKDIR /app

COPY programs/fuzzer ./fuzzer
COPY programs/test-server ./test-server
COPY programs/clients/rust ./rust

RUN cargo install --path fuzzer && \
	cargo install --path test-server && \
	cargo install --path rust

# https://stackoverflow.com/questions/58473606/cache-rust-dependencies-with-docker-build
#	--mount=type=cache,target=/usr/local/cargo/registry \
#    --mount=type=cache,target=/home/root/app/target \
	# cd fuzzer && cargo build -r && \
	# cd ../test-server && cargo build -r && \
	# cd ../rust && cargo build -r && \
	# cd .. && \
	# mv fuzzer/target/release/json-fuzzer . && \
	# mv test-server/target/release/test-server . && \
	# mv rust/target/release/rust-client . && \
	# rm -rf fuzzer test-server rust


# Go
FROM golang:alpine AS go

WORKDIR /app

# pre-copy/cache go.mod for pre-downloading dependencies and only redownloading them in subsequent builds if they change
COPY programs/clients/go/go.mod programs/clients/go/go.sum ./
RUN go mod download

COPY programs/clients/go/* .
RUN go build -v -o /usr/local/bin/go-client main.go


# JVM
FROM clojure:lein AS jvm

# WORKDIR /app
#
# COPY ./programs/clients/clojure/app/ .

# RUN lein uberjar && \
# 	mv target/uberjar/app-0.1.0-SNAPSHOT-standalone.jar ./clojure-client.jar

RUN mkdir -p /usr/src/app
WORKDIR /usr/src/app
COPY ./programs/clients/clojure/app/project.clj /usr/src/app/
RUN lein deps
COPY ./programs/clients/clojure/app/ /usr/src/app
RUN mv "$(lein uberjar | sed -n 's/^Created \(.*standalone\.jar\)/\1/p')" ./clojure-client.jar


# C++
FROM ubuntu:latest AS cpp

RUN --mount=target=/var/lib/apt,type=cache,sharing=locked \
    --mount=target=/var/cache/apt,type=cache,sharing=locked \
    rm -f /etc/apt/apt.conf.d/docker-clean && \
	apt-get update && \
	apt-get install -y cmake g++ libjansson-dev libyajl-dev

# C++
WORKDIR /app
COPY ./programs/clients/cpp ./cpp
WORKDIR /app/cpp/build
RUN cmake .. && make && cp cpp_client /usr/local/bin/cpp-client


# Final
FROM ubuntu:latest

WORKDIR /app

# RUN --mount=target=/var/lib/apt/lists,type=cache,sharing=locked \
#     --mount=target=/var/cache/apt,type=cache,sharing=locked \
RUN --mount=target=/var/lib/apt,type=cache,sharing=locked \
    --mount=target=/var/cache/apt,type=cache,sharing=locked \
    rm -f /etc/apt/apt.conf.d/docker-clean && \
	apt-get update && \
	apt-get install -y lua5.4 luarocks openjdk-21-jre libyajl-dev php

# RUN apt-get install -y gcc g++ make curl wget python3 make cmake git apt-utils \
# 	autoconf automake build-essential libcurl4-openssl-dev libgeoip-dev liblmdb-dev libpcre2-dev \
# 	libtool libxml2-dev pkgconf zlib1g-dev 

RUN luarocks install luasocket && \
	luarocks install lua-cjson

COPY --from=rust /usr/local/cargo/bin/json-fuzzer /usr/local/bin/
COPY --from=rust /usr/local/cargo/bin/test-server /usr/local/bin/
COPY --from=rust /usr/local/cargo/bin/rust-client /usr/local/bin/
COPY --from=cpp /usr/local/bin/cpp-client /usr/local/bin/
COPY --from=go /usr/local/bin/go-client /usr/local/bin/
COPY --from=jvm /usr/src/app/clojure-client.jar .

COPY ./programs/clients/python/ ./python
COPY ./programs/clients/lua/main.lua .
COPY ./programs/clients/php/main.php .
COPY ./programs/test.sh .
COPY ./programs/run.sh .
COPY ./programs/payloads.toml .
COPY ./scripts/analyze.sh .

# Rust
# WORKDIR /tmp
# RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs > rustup.sh && sh rustup.sh -y

# Golang
# RUN wget "https://golang.google.cn/dl/go1.25.1.linux-amd64.tar.gz"
# RUN rm -rf /usr/local/go && tar -C /usr/local -xzf go1.25.1.linux-amd64.tar.gz
# RUN export PATH=$PATH:/usr/local/go/bin

# C
# Jansson
# WORKDIR /app/programs/clients/c
# RUN wget https://github.com/akheron/jansson/releases/download/v2.14.1/jansson-2.14.1.tar.gz
# RUN tar -xf jansson-2.14.1.tar.gz
# WORKDIR jansson-2.14.1
# RUN ./configure
# RUN make
# RUN make install

# Modsecurity
# WORKDIR /app/programs/clients/cpp
# RUN git clone --depth=1 https://github.com/owasp-modsecurity/ModSecurity ModSecurity
# WORKDIR ModSecurity
# RUN git submodule init
# RUN git submodule update
# RUN sh build.sh
# RUN ./configure --with-pcre2
# RUN make
# RUN make install

