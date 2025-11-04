# Rust - fuzzer dependency caching
FROM rust:latest AS fuzzer-base
RUN cargo install cargo-chef --version ^0.1

FROM fuzzer-base AS fuzzer-planner
WORKDIR /app
# COPY . .
# RUN cargo install --path fuzzer
COPY programs/fuzzer .
RUN cargo chef prepare --recipe-path recipe.json

# Rust - fuzzer
FROM fuzzer-base AS fuzzer
WORKDIR /app

COPY --from=fuzzer-planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

COPY programs/fuzzer .
RUN cargo install --path .
# COPY programs/fuzzer ./fuzzer
# RUN cargo install --path fuzzer

# Rust - test server
FROM rust:latest AS test-server
WORKDIR /app
COPY programs/test-server ./test-server
RUN cargo install --path test-server

# Rust - client
FROM rust:latest AS rust-client
WORKDIR /app
COPY programs/clients/rust ./rust
RUN cargo install --path rust

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
FROM maven:3.9.11 AS jvm

WORKDIR /app

# Cache dependencies
COPY ./programs/clients/java/JavaClient/pom.xml ./pom.xml
RUN mvn dependency:go-offline -B

COPY ./programs/clients/java/JavaClient/src ./src
RUN mvn clean compile assembly:single && \
	cp target/java-client-jar-with-dependencies.jar java-client.jar && \
	rm -rf target

# FROM clojure:lein AS jvm
#
# # WORKDIR /app
# #
# # COPY ./programs/clients/clojure/app/ .
#
# # RUN lein uberjar && \
# # 	mv target/uberjar/app-0.1.0-SNAPSHOT-standalone.jar ./clojure-client.jar
#
# RUN mkdir -p /usr/src/app
# WORKDIR /usr/src/app
# COPY ./programs/clients/clojure/app/project.clj /usr/src/app/
# RUN lein deps
# COPY ./programs/clients/clojure/app/ /usr/src/app
# RUN mv "$(lein uberjar | sed -n 's/^Created \(.*standalone\.jar\)/\1/p')" ./clojure-client.jar


# C++
FROM ubuntu:latest AS cpp

RUN --mount=target=/var/lib/apt,type=cache,sharing=locked \
    --mount=target=/var/cache/apt,type=cache,sharing=locked \
    rm -f /etc/apt/apt.conf.d/docker-clean && \
	apt-get update && \
	apt-get install -y cmake g++ libjansson-dev libyajl-dev git libjson-c-dev libpoco-dev


# RUN --mount=target=/root/.cache/fuzzer,type=cache,sharing=locked \
# 	git clone --depth 1 https://github.com/boostorg/json.git && mv json boost-json

WORKDIR /app
ADD https://archives.boost.io/release/1.82.0/source/boost_1_82_0.tar.gz .
RUN tar -xf boost_1_82_0.tar.gz

COPY ./programs/clients/cpp ./cpp
# RUN --mount=target=~/root/.cache/fuzzer,type=cache,sharing=locked \
# 	wget https://archives.boost.io/release/1.82.0/source/boost_1_82_0.tar.gz && \

WORKDIR /app/cpp/build
RUN cmake .. && make && cp cpp_client /usr/local/bin/cpp-client


# C#
FROM mcr.microsoft.com/dotnet/sdk:9.0@sha256:3fcf6f1e809c0553f9feb222369f58749af314af6f063f389cbd2f913b4ad556 AS dotnet
WORKDIR /app

COPY ./programs/clients/dotnet/ ./
RUN dotnet restore
RUN dotnet publish -o out

## Build runtime image
#FROM mcr.microsoft.com/dotnet/aspnet:9.0@sha256:b4bea3a52a0a77317fa93c5bbdb076623f81e3e2f201078d89914da71318b5d8
#WORKDIR /app
#COPY --from=dotnet /app/out .
#ENTRYPOINT ["dotnet", "DotNet.Docker.dll"]


# Python
# FROM python:3.14-trixie AS python-client
FROM ubuntu:latest AS python-client
ENV PYTHONUNBUFFERED=1

WORKDIR /app/

RUN --mount=target=/var/lib/apt,type=cache,sharing=locked \
    --mount=target=/var/cache/apt,type=cache,sharing=locked \
    rm -f /etc/apt/apt.conf.d/docker-clean && \
	apt-get update && \
	apt-get install -y python3 python3-pip python3.12-venv

RUN python3 -m venv /opt/venv
# Enable venv
ENV PATH="/opt/venv/bin:$PATH"

COPY ./programs/clients/python/requirements.txt ./requirements.txt
RUN pip3 install -Ur requirements.txt


# Radamsa
FROM ubuntu:latest AS radamsa

WORKDIR /app

RUN --mount=target=/var/lib/apt,type=cache,sharing=locked \
    --mount=target=/var/cache/apt,type=cache,sharing=locked \
    rm -f /etc/apt/apt.conf.d/docker-clean && \
	apt-get update && \
	apt-get install -y gcc make git wget

RUN git clone https://gitlab.com/akihe/radamsa.git && cd radamsa && make && cp bin/radamsa /usr/local/bin/radamsa


# Final
FROM ubuntu:latest

WORKDIR /app

# RUN --mount=target=/var/lib/apt/lists,type=cache,sharing=locked \
#     --mount=target=/var/cache/apt,type=cache,sharing=locked \
RUN --mount=target=/var/lib/apt,type=cache,sharing=locked \
    --mount=target=/var/cache/apt,type=cache,sharing=locked \
    rm -f /etc/apt/apt.conf.d/docker-clean && \
	apt-get update && \
	apt-get install -y lua5.4 luarocks openjdk-21-jre php ruby python3 python3-pip python3.12-venv valgrind \
			libyajl-dev dotnet-runtime-8.0 libpoco-dev

	# python3 -m venv venv
	# source ./venv/bin/activate
	# cd ..

# RUN apt-get install -y gcc g++ make curl wget python3 make cmake git apt-utils \
# 	autoconf automake build-essential libcurl4-openssl-dev libgeoip-dev liblmdb-dev libpcre2-dev \
# 	libtool libxml2-dev pkgconf zlib1g-dev 

RUN luarocks install luasocket && \
	luarocks install lua-cjson

COPY ./programs/clients/python/main.py ./python/main.py
COPY --from=python-client /opt/venv /opt/venv

# Enable venv
ENV VIRTUAL_ENV=/opt/venv
ENV PATH="$VIRTUAL_ENV/bin:$PATH"

#COPY ./programs/clients/python/requirements.txt ./python/requirements.txt

#RUN pip3 install -r ./python/requirements.txt

# WORKDIR /app/python/
# RUN python3 -m venv venv && source ./venv/bin/activate && \
# 	pip3 install -r requirements.txt
#
# WORKDIR /app

COPY --from=fuzzer /usr/local/cargo/bin/json-fuzzer /usr/local/bin/
COPY --from=test-server /usr/local/cargo/bin/test-server /usr/local/bin/
COPY --from=rust-client /usr/local/cargo/bin/rust-client /usr/local/bin/
COPY --from=cpp /usr/local/bin/cpp-client /usr/local/bin/
COPY --from=go /usr/local/bin/go-client /usr/local/bin/
#COPY --from=jvm /usr/src/app/clojure-client.jar .
 COPY --from=jvm /app/java-client.jar .
COPY --from=dotnet /app/out ./dotnet_client
COPY --from=radamsa /usr/local/bin/radamsa /usr/local/bin/

# COPY ./programs/clients/java/JavaClient/java-client.jar java-client.jar
COPY ./programs/clients/lua/main.lua .
COPY ./programs/clients/php/main.php .
COPY ./programs/clients/ruby/main.rb .
COPY ./programs/test.sh .
COPY ./programs/run.sh .
# COPY ./programs/payloads.toml .
COPY ./programs/payloads.csv .
# COPY ./programs/payloads_dos.toml .
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

