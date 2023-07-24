set -euxo pipefail

main() {
    if [ $TARGET = x86_64-unknown-linux-gnu ]; then
        cargo test --target $TARGET
    else
        cargo check --target $TARGET
    fi
}

main
