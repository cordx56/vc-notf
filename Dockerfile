FROM rust:1.54

WORKDIR /usr/src/vc-notf
COPY . .

RUN cargo install --path .

CMD ["vc-notf"]
