use core::time::Duration;

use crate::futex::{FutexKey, FutexQ};
use alloc::{collections::VecDeque, sync::Arc};
use alloc::boxed::Box;
use axerrno::{AxError, AxResult};
use axhal::mem::VirtAddr;
use spinlock::SpinNoIrq;
use lazy_init::LazyInit;
use axprocess::current_process;

pub static FUTEX_QUEUES: LazyInit<FutexHashBuckets> = LazyInit::new();

// SpinRaw<VecDeque<AxTaskRef>>
type FutexHashBucket = SpinNoIrq<VecDeque<Arc<FutexQ>>>;


pub struct FutexHashBuckets(Box<[FutexHashBucket]>);

impl FutexHashBuckets {
    pub fn new(hash_size: usize) -> Self {
        let inner = unsafe {
            let mut buf = Box::new_uninit_slice(hash_size);
            for i in 0..hash_size {
                buf[i].write(SpinNoIrq::new(VecDeque::new()));
            }
            buf.assume_init()
        };
        Self(inner)
    }

    pub fn get_bucket(&self, hash: usize) -> &FutexHashBucket {
        &self.0[hash]
    }
}

pub fn futex_get_value_locked(vaddr: VirtAddr) -> AxResult<u32> {
    let process = current_process();
    if process.manual_alloc_for_lazy(vaddr).is_ok() {
        let real_futex_val =
                        unsafe { (vaddr.as_usize() as *const u32).read_volatile() };
        Ok(real_futex_val)
    } else {
        Err(AxError::BadAddress)
    }
}