set -ex

main() {
    cross build --target $TARGET

    if [ $TARGET = x86_64-unknown-linux-gnu ]; then
        cross test --target $TARGET
    fi
}

main
