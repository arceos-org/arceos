FROM rust:alpine

RUN apk add --no-cache \
    build-base qemu-system-x86_64 qemu-system-loongarch64 \
    qemu-system-riscv64 qemu-system-aarch64 \
    clang-dev bash coreutils wget python3 xz \
    py3-pip samurai bzip2 meson make \
    git pkgconf glib-dev libslirp \
    ca-certificates openssl diffutils findutils vim cmake

RUN cargo install cargo-binutils axconfig-gen cargo-axplat

RUN wget https://github.com/arceos-org/setup-musl/releases/download/prebuilt/aarch64-linux-musl-cross.tgz \
&& wget https://github.com/arceos-org/setup-musl/releases/download/prebuilt/riscv64-linux-musl-cross.tgz \
&& wget https://github.com/arceos-org/setup-musl/releases/download/prebuilt/x86_64-linux-musl-cross.tgz \
&& wget https://github.com/arceos-org/setup-musl/releases/download/prebuilt/loongarch64-linux-musl-cross.tgz \
&& tar zxf aarch64-linux-musl-cross.tgz \
&& tar zxf riscv64-linux-musl-cross.tgz \
&& tar zxf x86_64-linux-musl-cross.tgz \
&& tar zxf loongarch64-linux-musl-cross.tgz \
&& rm -f *.tgz

ENV PATH="/x86_64-linux-musl-cross/bin:/aarch64-linux-musl-cross/bin:/riscv64-linux-musl-cross/bin:/loongarch64-linux-musl-cross/bin:$PATH"

COPY ./rust-toolchain.toml /workspace/rust-toolchain.toml

RUN rustup target list --installed