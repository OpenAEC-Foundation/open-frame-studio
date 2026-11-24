FROM rust:1.99.0-alpine3.23@sha256:42c2519fdf75d9e34cc61a1aaf34b1a684da9bfd65fe5daefef98d92a801b001 AS build
WORKDIR /build
COPY . .
RUN cargo build --release --locked --package ofs-cloud

FROM alpine:3.23@sha256:469c9ea410b9215eaee7976ba6c0639039b444be8e5d638f7abbada189971eb2
COPY --from=build /build/target/release/ofs-cloud /usr/local/bin/ofs-cloud
USER 65534:65534
ENV OFS_LISTEN_ADDR=0.0.0.0:3456
EXPOSE 3456
HEALTHCHECK --interval=30s --timeout=5s CMD ["wget", "-q", "-T", "3", "-O", "/dev/null", "http://127.0.0.1:3456/api/v1/health"]
ENTRYPOINT ["/usr/local/bin/ofs-cloud", "--service"]
