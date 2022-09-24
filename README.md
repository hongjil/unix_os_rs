# Unix-like OS on RISC-V in Rust

This is a toy project referencing the [rCore OS tutorial](https://rcore-os.cn/rCore-Tutorial-Book-v3/) and their [github page](https://github.com/rcore-os/rCore-Tutorial-v3) for learning purpose.

## Environment Setup
The OS runs with `qemu-system-riscv64` simulator on Mac M1 and check the [prerequisite](https://github.com/rcore-os/rCore-Tutorial-v3#prerequisites) for detailed environment setup.

## Build and run
1. Build the binary
```
$ cd os/
$ cargo build --release
```
2. Strip the metadata out of binary beforing loading it into QEMU
```
$ rust-objcopy --strip-all target/riscv64gc-unknown-none-elf/release/os -O binary target/riscv64gc-unknown-none-elf/release/os.bin
```
3. Run the OS on QEMU RISC-V
```
$ qemu-system-riscv64 \
    -machine virt \
    -nographic \
    -bios ../bootloader/rustsbi-qemu.bin \
    -device loader,file=target/riscv64gc-unknown-none-elf/release/os.bin,addr=0x80200000 \
    -s -S
```
where `-s -S` is for [GDB usage](https://www.qemu.org/docs/master/system/gdb.html#gdb-usage); Remove this if you don't want to bring up with GDB.

4. (If you use GDB in the last step), Open another terminal and run GDB client
```
$ riscv64-unknown-elf-gdb \
    -ex 'file target/riscv64gc-unknown-none-elf/release/os' \
    -ex 'set arch riscv:rv64' \
    -ex 'target remote localhost:1234'
```
check [this gdb cheatsheet](docs/gdb_cheatsheet.md) in case you need help :)