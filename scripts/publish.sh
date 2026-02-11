# Layer 1 (no internal dependencies)
cargo publish -p axconfig --allow-dirty

# Layer 2
cargo publish -p axalloc --allow-dirty
cargo publish -p axhal --allow-dirty

# Layer 3
cargo publish -p axtask --allow-dirty
cargo publish -p axmm --allow-dirty
cargo publish -p axsync --allow-dirty
cargo publish -p axipi --allow-dirty

# Layer 4
cargo publish -p axdma --allow-dirty
cargo publish -p axdriver --allow-dirty

# Layer 5
cargo publish -p axnet --allow-dirty
cargo publish -p axdisplay --allow-dirty
cargo publish -p axinput --allow-dirty
cargo publish -p axfs --allow-dirty

# Layer 6
cargo publish -p axruntime --allow-dirty

# Layer 7
cargo publish -p axfeat --allow-dirty

# Layer 8
cargo publish -p arceos_api --allow-dirty
cargo publish -p arceos_posix_api --allow-dirty

# Layer 9
cargo publish -p axstd --allow-dirty
cargo publish -p axlibc --allow-dirty
