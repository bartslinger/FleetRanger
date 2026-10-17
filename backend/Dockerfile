# The build stage runs natively on the build machine and cross-compiles a
# static binary for the target platform with cargo-zigbuild, so building an
# x86_64 image on an arm64 machine (or the reverse) needs no emulation.
FROM --platform=$BUILDPLATFORM rust:1.99.1-trixie@sha256:e259494b14672950a909b4c54527e19d29cae6ebf980672114c8bb85e5594e4a AS build

ARG BUILDARCH
ARG TARGETARCH

ARG ZIG_VERSION=0.16.0
ARG ZIG_SHA256_AMD64=70e49664a74374b48b51e6f3fdfbf437f6395d42509050588bd49abe52ba3d00
ARG ZIG_SHA256_ARM64=ea4b09bfb22ec6f6c6ceac57ab63efb6b46e17ab08d21f69f3a48b38e1534f17
ARG CARGO_ZIGBUILD_VERSION=0.23.4

# Use the image's toolchain, which matches rust-toolchain.toml, instead of
# letting rustup download components we don't need here.
ENV RUSTUP_TOOLCHAIN=1.99.1

RUN set -eu; \
    case "$BUILDARCH" in \
      amd64) zig_arch=x86_64; zig_sha256=$ZIG_SHA256_AMD64 ;; \
      arm64) zig_arch=aarch64; zig_sha256=$ZIG_SHA256_ARM64 ;; \
      *) echo "unsupported build architecture: $BUILDARCH" >&2; exit 1 ;; \
    esac; \
    curl -fsSL -o /tmp/zig.tar.xz "https://ziglang.org/download/${ZIG_VERSION}/zig-${zig_arch}-linux-${ZIG_VERSION}.tar.xz"; \
    echo "${zig_sha256}  /tmp/zig.tar.xz" | sha256sum -c -; \
    mkdir /opt/zig; \
    tar -xJf /tmp/zig.tar.xz -C /opt/zig --strip-components=1; \
    rm /tmp/zig.tar.xz; \
    ln -s /opt/zig/zig /usr/local/bin/zig

RUN cargo install --locked cargo-zigbuild --version "$CARGO_ZIGBUILD_VERSION"

RUN set -eu; \
    case "$TARGETARCH" in \
      amd64) echo x86_64-unknown-linux-musl > /rust-target ;; \
      arm64) echo aarch64-unknown-linux-musl > /rust-target ;; \
      *) echo "unsupported target architecture: $TARGETARCH" >&2; exit 1 ;; \
    esac; \
    rustup target add "$(cat /rust-target)"

WORKDIR /src
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY src src
COPY migrations migrations

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/src/target \
    set -eu; \
    target="$(cat /rust-target)"; \
    cargo zigbuild --locked --release --target "$target"; \
    cp "target/$target/release/fleetranger" /fleetranger

# Static binary on a minimal base: no shell or package manager, runs as a
# non-root user, and includes CA certificates for outbound HTTPS.
FROM gcr.io/distroless/static-debian13:nonroot@sha256:e2e927ec666bae08560abb3c55d0659eceabb657f56b6782ab500a9fc7f555e3

COPY --from=build /fleetranger /usr/local/bin/fleetranger

EXPOSE 8080
ENTRYPOINT ["/usr/local/bin/fleetranger"]
