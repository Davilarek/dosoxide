@echo off
setlocal enabledelayedexpansion

clang -target i386-unknown-none -c asm/header.s -o header.o
ld.lld --oformat binary -o header.bin header.o

clang -target i386-unknown-none -m32 -c asm/start.s -o start.o

cargo +nightly rustc --release ^
    -Z build-std=core,alloc ^
    -Z build-std-features=compiler-builtins-mem ^
    -Z json-target-spec ^
    --target i386-dos.json ^
    --lib -- ^
    --emit=obj ^
    -C panic=abort

set RUST_OBJ=
for %%f in (target\i386-dos\release\deps\dosoxide-*.o) do (
    set RUST_OBJ=%%f
    goto :found
)
:found

ld.lld -T linker.ld --oformat binary -nmagic -o body.bin start.o %RUST_OBJ%

copy /b header.bin + body.bin PROGRAM.EXE

del header.o start.o header.bin body.bin

echo Done! PROGRAM.EXE created.
