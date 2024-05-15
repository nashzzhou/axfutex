use crate::futex::FutexQ;
use alloc::collections::VecDeque;
use alloc::boxed::Box;
use spinlock::SpinNoIrq;
use lazy_init::LazyInit;

pub struct FutexHashBuckets {
    pub buckets: Box<[SpinNoIrq<VecDeque<FutexQ>>]>,
}


impl FutexHashBuckets {
    pub fn new(hash_size: usize) -> Self {
        let buckets = unsafe {
            let mut buf = Box::new_uninit_slice(hash_size);
            for i in 0..hash_size {
                buf[i].write(SpinNoIrq::new(VecDeque::new()));
            }
            buf.assume_init()
        };
        Self {
            buckets,
        }
    }
}

pub static FUTEX_HASH_BUCKETS: LazyInit<FutexHashBuckets> = LazyInit::new();