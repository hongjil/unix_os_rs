# Unix-like OS on RISC-V in Rust

This is a toy project referencing the [rCore OS tutorial](https://rcore-os.cn/rCore-Tutorial-Book-v3/) and their [github page](https://github.com/rcore-os/rCore-Tutorial-v3) for learning purpose.

## Environment Setup
The OS runs with `qemu-system-riscv64` simulator on Mac M1 and check the [prerequisite](https://github.com/rcore-os/rCore-Tutorial-v3#prerequisites) for detailed environment setup.

## Build and run
### Run directly 
```bash
$ cd os/
$ make run
```
[![asciicast](https://asciinema.org/a/63amL5TRnLvmzG7dxxHKmIkWH.svg)](https://asciinema.org/a/63amL5TRnLvmzG7dxxHKmIkWH)
### Run with GDB
```bash
$ cd os/
# In the current terminal, run
$ make gdbserver
# Open another terminal, still in os folder, run
$ make gdbclient
```
#### Or instead:
```bash
$ cd os/
# Start a tmux session which hold client/server on two sides.
$ make debug
```
Check [this gdb cheatsheet](docs/gdb_cheatsheet.md) in case you need help :)