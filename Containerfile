FROM docker.io/library/rust:1.96-bookworm AS build
WORKDIR /workspace
COPY . .
RUN cargo build --release -p skrifheim

FROM gcr.io/distroless/cc-debian12:nonroot
COPY --from=build /workspace/target/release/skrifheim /usr/local/bin/skrifheim
ENTRYPOINT ["/usr/local/bin/skrifheim"]
