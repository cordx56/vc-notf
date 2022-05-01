FROM rustlang/rust:nightly AS builder
WORKDIR /usr/src/vc-notf
RUN cargo install sqlx-cli
COPY . .
RUN cargo install --path .

FROM debian:buster-slim
RUN apt-get update && apt-get install -y libssl-dev
COPY --from=builder /usr/local/cargo/bin/vc-notf /usr/local/bin/vc-notf
COPY --from=builder /usr/local/cargo/bin/sqlx /usr/local/bin/sqlx
COPY migrations/ /migrations
CMD ["vc-notf"]
