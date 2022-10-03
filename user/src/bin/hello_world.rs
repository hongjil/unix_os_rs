#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

// No main but it will be jumped from lib.rs.
#[no_mangle]
pub fn main() -> i32 {
    println!("Hello world from user mode program!");
    0
}
