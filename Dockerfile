FROM --platform=$TARGETPLATFORM rust:1-alpine3.15 as builder

RUN apk add openssl openssl-dev pkgconf musl-dev curl perl make linux-headers
WORKDIR /rust
COPY Cargo.toml Cargo.lock ./
RUN xarch="$(apk --print-arch)"; [[ "$xarch" == "aarch64" ]] && ln -s /usr/bin/cc /usr/local/rustup/toolchains/1.60.0-aarch64-unknown-linux-musl/lib/rustlib/aarch64-unknown-linux-musl/bin/cc || true
RUN xarch="$(apk --print-arch)"; [[ "$xarch" == "aarch64" ]] && ln -s /usr/bin/cc /usr/local/cargo/bin/cc || true
RUN xarch="$(apk --print-arch)"; [[ "$xarch" == "aarch64" ]] && ln -s /usr/bin/ar /usr/local/cargo/bin/ar || true
RUN xarch="$(apk --print-arch)"; [[ "$xarch" == "aarch64" ]] && ln -s /usr/bin/perl /usr/local/cargo/bin/perl || true

WORKDIR /openssl
RUN mkdir /musl
RUN curl -LO https://github.com/openssl/openssl/archive/OpenSSL_1_1_1f.tar.gz
RUN tar zxvf OpenSSL_1_1_1f.tar.gz

WORKDIR /openssl/openssl-OpenSSL_1_1_1f/
RUN xarch="$(apk --print-arch)"; CC="gcc -fPIE -pie" ./Configure no-shared no-async --prefix=/musl --openssldir=/musl/ssl "linux-${xarch}"
RUN make depend
RUN make
RUN make install
ENV PKG_CONFIG_ALLOW_CROSS=1
ENV OPENSSL_STATIC=1
ENV OPENSSL_DIR=/musl

WORKDIR /rust
COPY Cargo.toml Cargo.toml
RUN sed -i 's#src/bin/remo.rs#dummy.rs#' Cargo.toml
RUN sed -i 's#src/bin/milkmilk.rs#dummy.rs#' Cargo.toml
RUN mkdir src; echo 'fn main() { println!("dummy") }' > dummy.rs
RUN cargo rustc --release --bin milkmilk -- -C linker=rust-lld
RUN rm -rf src Cargo.toml

COPY Cargo.toml Cargo.toml
COPY src src
RUN cargo rustc --release --bin remo -- -C linker=rust-lld
RUN cargo rustc --release --bin milkmilk -- -C linker=rust-lld

FROM --platform=$TARGETPLATFORM scratch as remo-bin
COPY --from=builder /rust/target/release/remo /remo
ENTRYPOINT ["/remo"]

FROM --platform=$TARGETPLATFORM scratch as milkmilk-bin
COPY --from=builder /rust/target/release/milkmilk /milkmilk
ENTRYPOINT ["/milkmilk"]
