FROM rust:1.54 AS builder
WORKDIR /usr/src/vc-notf
COPY . .
RUN cargo install --path .

FROM debian:buster-slim
COPY --from=builder /usr/local/cargo/bin/vc-notf /usr/local/bin/vc-notf
CMD ["vc-notf"]
