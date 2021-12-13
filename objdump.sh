rm -rf target/x86_64-blog_os/debug/_bootimage-hash_os.bin.extracted/
binwalk --dd='.*' target/x86_64-blog_os/debug/bootimage-hash_os.bin
objdump -D target/x86_64-blog_os/debug/_bootimage-hash_os.bin.extracted/EA00|less
