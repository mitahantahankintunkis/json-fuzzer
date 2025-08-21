FROM alpine:latest

RUN apk add libjansson-dev gcc make curl wget
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
RUN rm -rf /usr/local/go && tar -C /usr/local -xzf go1.25.0.linux-amd64.tar.gz
RUN wget "https://go.dev/dl/go1.25.0.src.tar.gz"
RUN export PATH=$PATH:/usr/local/go/bin

