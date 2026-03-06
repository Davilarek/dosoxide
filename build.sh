#!/bin/bash
set -e

clang -target i386-unknown-none -c asm/header.s -o header.o
ld.lld --oformat binary -o header.bin header.o

clang -target i386-unknown-none -m32 -c asm/start.s -o start.o

cargo +nightly rustc --release \
    -Z build-std=core,alloc \
    -Z build-std-features=compiler-builtins-mem \
    -Z json-target-spec \
    --target i386-dos.json \
    --lib -- \
    --emit=obj \
    -C panic=abort 

RUST_OBJ=$(ls target/i386-dos/release/deps/dosoxide-*.o | head -n 1)

ld.lld -T linker.ld --oformat binary -nmagic -o body.bin start.o $RUST_OBJ

cat header.bin body.bin > PROGRAM.EXE

rm header.o start.o header.bin body.bin

echo "Done! PROGRAM.EXE created."
