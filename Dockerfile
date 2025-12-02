FROM rust:slim-trixie

# 有需要可以取消注释
# RUN sed -i 's/deb.debian.org/mirrors.ustc.edu.cn/g' /etc/apt/sources.list.d/debian.sources

RUN apt-get update \
    && apt-get install -y --no-install-recommends libclang-20-dev wget make python3 \
        xz-utils python3-venv ninja-build bzip2 meson \
        pkg-config libglib2.0-dev git libslirp-dev \
    && rm -rf /var/lib/apt/lists/*

RUN cargo install cargo-binutils axconfig-gen cargo-axplat

COPY rust-toolchain.toml /workspace/rust-toolchain.toml
WORKDIR /workspace
RUN rustup show

RUN wget -q https://github.com/arceos-org/setup-musl/releases/download/prebuilt/aarch64-linux-musl-cross.tgz \
    && wget -q https://github.com/arceos-org/setup-musl/releases/download/prebuilt/riscv64-linux-musl-cross.tgz \
    && wget -q https://github.com/arceos-org/setup-musl/releases/download/prebuilt/x86_64-linux-musl-cross.tgz \
    && wget -q https://github.com/arceos-org/setup-musl/releases/download/prebuilt/loongarch64-linux-musl-cross.tgz \
    && tar zxf aarch64-linux-musl-cross.tgz -C / \
    && tar zxf riscv64-linux-musl-cross.tgz -C / \
    && tar zxf x86_64-linux-musl-cross.tgz -C / \
    && tar zxf loongarch64-linux-musl-cross.tgz -C / \
    && rm -f *.tgz

RUN wget -q https://download.qemu.org/qemu-10.1.2.tar.xz \
    && tar xJf qemu-10.1.2.tar.xz \
    && cd qemu-10.1.2 \
    && ./configure --prefix=/qemu-bin-10.1.2 \
        --target-list=loongarch64-softmmu,riscv64-softmmu,aarch64-softmmu,x86_64-softmmu \
        --enable-slirp \
    && make -j$(nproc) \
    && make install
RUN rm -rf qemu-10.1.2 qemu-10.1.2.tar.xz

ENV PATH="/x86_64-linux-musl-cross/bin:/aarch64-linux-musl-cross/bin:/riscv64-linux-musl-cross/bin:/loongarch64-linux-musl-cross/bin:$PATH"
ENV PATH="/qemu-bin-10.1.2/bin:$PATH"

CMD [ "bash" ]