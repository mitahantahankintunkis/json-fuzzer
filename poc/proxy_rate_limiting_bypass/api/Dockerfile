FROM golang:alpine

WORKDIR /app

# pre-copy/cache go.mod for pre-downloading dependencies and only redownloading them in subsequent builds if they change
COPY ./go.mod ./
RUN go mod download

COPY ./main.go .
RUN go build -v -o /usr/local/bin/api main.go

ENTRYPOINT [ "api" ]
