//! This module provides the Fast Userspace Mutexes (Futex) management API for the operating system.

#![no_std]
#![feature(new_uninit)]

extern crate alloc;

use axlog::info;
mod core;
mod futex;
mod requque;
mod waitwake;
mod flags;

pub use waitwake::*;
pub use requque::*;

use core::{FUTEX_QUEUES, FutexHashBuckets};

const FUTEX_HASH_SIZE: usize = 256;

/// Initializes 
pub fn init_futex() {
    info!("Initialize futex...");
    FUTEX_QUEUES.init_by(FutexHashBuckets::new(FUTEX_HASH_SIZE));
}