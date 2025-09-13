FROM golang:alpine

WORKDIR /app

# pre-copy/cache go.mod for pre-downloading dependencies and only redownloading them in subsequent builds if they change
COPY ./go.mod ./go.sum ./
RUN go mod download
RUN apk add gcc build-base

COPY ./main.go .
ENV CGO_ENABLED=1
RUN go build -v -o /usr/local/bin/api main.go

ENTRYPOINT [ "api" ]
