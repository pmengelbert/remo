FROM --platform=$TARGETPLATFORM rust:1-alpine3.15 as builder

RUN apk add openssl-dev pkgconf musl-dev
WORKDIR /rust
COPY Cargo.toml Cargo.lock ./
RUN sed -i 's#src/bin/remo.rs#dummy.rs#' Cargo.toml
RUN mkdir src; echo 'fn main() { println!("dummy") }' > dummy.rs
RUN cargo build --release
RUN rm -rf src Cargo.toml

COPY Cargo.toml Cargo.toml
COPY src src
RUN cargo build --release

FROM --platform=$TARGETPLATFORM scratch as bin
COPY --from=builder /rust/target/release/remo /remo
ENTRYPOINT ["/remo"]
