FROM --platform=$TARGETPLATFORM rust:1-alpine3.15 as builder

WORKDIR /rust
COPY Cargo.toml Cargo.lock ./
RUN mkdir src; echo 'fn main() { println!("dummy") }' > src/main.rs
RUN cargo build --release
RUN rm -rf src

COPY src src
RUN cargo build --release

FROM --platform=$TARGETPLATFORM scratch as bin
COPY --from=builder /rust/target/release/remo /remo
ENTRYPOINT ["/remo"]
