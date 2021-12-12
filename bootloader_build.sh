cd ../bootloader
cargo builder  --kernel-manifest  ../hash_os/Cargo.toml --kernel-binary ../hash_os/target/x86_64-blog_os/release/hash_os
cd -
