//! clone 任务时指定的参数。

use bitflags::*;

bitflags! {
    pub struct FutexFlags: u32 {
        /// 用于指定 futex 的操作
        const FUTEX_WAIT = 0;
        const FUTEX_WAKE = 1;
        const FUTEX_FD = 2;
        const FUTEX_REQUEUE = 3;
        const FUTEX_CMP_REQUEUE = 4;
        const FUTEX_WAKE_OP = 5;
        const FUTEX_PRIVATE_FLAG = 128;
    }
}