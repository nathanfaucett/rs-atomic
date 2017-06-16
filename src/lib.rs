#![feature(const_fn)]
#![feature(alloc)]
#![no_std]


extern crate alloc;

#[cfg(test)] extern crate std;
#[cfg(test)] extern crate rng;
#[cfg(test)] extern crate prng;


mod atomic;


pub use self::atomic::{Atomic, Ordering};
