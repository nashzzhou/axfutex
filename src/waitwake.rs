use core::time::Duration;

use alloc::sync::Arc;
use axerrno::{AxError, AxResult};
use axhal::mem::VirtAddr;
use axlog::info;
use axprocess::{current_task, signal::current_have_signals};
use axtask::{cancel_alarm, is_in_timer_list, RUN_QUEUE};

use crate::{core::*, futex::{get_futex_key, FutexQ}};

fn _futex_wait(
    vaddr: VirtAddr,
    flags: u32,
    val: u32,
    deadline: Option<Duration>,
    _bitset: u32,  
) -> AxResult<usize> {
    info!("vaddr: {:?}, flags: {:?}, val: {:?}, deadline: {:?}", vaddr, flags, val, deadline);
    loop {
        let key = get_futex_key(vaddr, flags);
        let uval = futex_get_value_locked(vaddr)?;
        if uval != val {
            return Err(AxError::WouldBlock);
        }
        // queue
        let hash = key.futex_hash();
        let mut hb = FUTEX_QUEUES.get_bucket(hash).lock();
        let fq = FutexQ::new(key, current_task().as_task_ref().clone());
        if let Some(deadline) = deadline {
            hb.push_back(fq.clone());
            drop(hb);
            RUN_QUEUE.lock().sleep_until(deadline);
        } else {
            RUN_QUEUE.lock().block_current(|_| {
                hb.push_back(fq.clone());
                drop(hb);
            });
        }
        // Remove the futex_q from its futex_hash_bucket
        let mut hb = FUTEX_QUEUES.get_bucket(hash).lock();
        if let Some(index) = hb.iter().position(|fq1| Arc::ptr_eq(&fq, fq1)) {
            hb.remove(index);
            // the timer has already expired
            if deadline.is_some() && !is_in_timer_list(&fq.task) {
                return Err(AxError::Timeout);
            }
            // 被信号打断
            if current_have_signals() {
                return Err(AxError::Interrupted);
            }
        } else {
            // If we were woken (and unqueued), we succeeded, whatever.
            return Ok(0);
        }
    }
}



pub fn futex_wait(
    vaddr: VirtAddr,
    flags: u32,
    val: u32,
    timeout: usize,
    _bitset: u32,
) -> AxResult<usize> {
    let deadline = if timeout != 0 {
        Some(Duration::from_nanos(timeout as u64) + axhal::time::current_time())
    } else {
        None
    };
    let ret = _futex_wait(vaddr, flags, val, deadline, _bitset);
    // if we have timeout, clean up timer
    if deadline.is_some() {
        cancel_alarm(&current_task().as_task_ref().clone());
    }
    ret
}


pub fn futex_wake(
    vaddr: VirtAddr,
    flags: u32,
    nr_wake: u32,
    _bitset: u32,
) -> AxResult<usize> {
    info!("vaddr: {:?}, flags: {:?}, nr_wake: {:?}", vaddr, flags, nr_wake);
    let key = get_futex_key(vaddr, flags);
    let hash = key.futex_hash();
    let mut hb = FUTEX_QUEUES.get_bucket(hash).lock();
    // Make sure we really have tasks to wakeup
    if hb.is_empty() {
        return Ok(0);
    }
    let mut woken = 0;
    hb.retain(|fq| {
        if woken < nr_wake {
            if fq.match_key(&key) {
                RUN_QUEUE.lock().unblock_task(fq.task.clone(), false);
                woken += 1;
                return false
            }
        }
        true
    });
    // yield_now_task();
    Ok(woken as usize)
}