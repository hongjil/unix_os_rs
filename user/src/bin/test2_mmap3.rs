#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::mmap;

#[no_mangle]
fn main() -> i32 {
    let start: usize = 0x10000000;
    let len: usize = 4096;
    let prot: usize = 3;
    assert_eq!(0, mmap(start, len, prot));
    assert_eq!(mmap(start - len, len + 1, prot), -1);
    assert_eq!(mmap(start + len + 1, len, prot), -1);
    assert_eq!(mmap(start + len, len, 0), -1);
    assert_eq!(mmap(start + len, len, prot | 8), -1);
    println!("Test2 mmap3 test OK!");
    0
}
