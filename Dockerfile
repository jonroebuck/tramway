FROM rust:alpine AS builder

RUN apk add --no-cache musl-dev

WORKDIR /app

COPY . .

RUN cargo build --release -p tramway-server

FROM alpine AS runtime

RUN addgroup -S tramway && adduser -S tramway -G tramway

COPY --from=builder /app/target/release/tramway-server /usr/local/bin/tramway-server

USER tramway

EXPOSE 8080

CMD ["tramway-server"]
