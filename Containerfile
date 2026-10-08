# Rust 1.99.0 slim-bookworm, multi-platform index reviewed 2026-10-08.
FROM docker.io/library/rust@sha256:2c3a22f0a5533ea2dd5a16627bc841228151faa2d4de2644ac9987e4a2f1f2fa AS build
WORKDIR /workspace
COPY . .
RUN cargo build --release --locked -p skrifheim

# Debian 12 nonroot, multi-platform index reviewed 2026-10-08.
FROM gcr.io/distroless/cc-debian12@sha256:9dac0a79194e45a7da0158a9c6da57b217585af0786db3845d1f0ec1a0dd182f
COPY --from=build /workspace/target/release/skrifheim /usr/local/bin/skrifheim
ENTRYPOINT ["/usr/local/bin/skrifheim"]
