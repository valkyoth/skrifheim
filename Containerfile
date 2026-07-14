FROM docker.io/library/rust@sha256:606f3248aa86ce49e0b98d9e0bbffde042adeb18982320f97bcc218615de1c99 AS build
WORKDIR /workspace
COPY . .
RUN cargo build --release --locked -p skrifheim

FROM gcr.io/distroless/cc-debian12@sha256:66aa873a4a14fb164aa01296058efd8253744606d72715e45acface073359faa
COPY --from=build /workspace/target/release/skrifheim /usr/local/bin/skrifheim
ENTRYPOINT ["/usr/local/bin/skrifheim"]
