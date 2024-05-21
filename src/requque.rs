use alloc::collections::VecDeque;
use axerrno::AxResult;
use axhal::mem::VirtAddr;

use crate::{core::FUTEX_QUEUES, futex::{get_futex_key, FutexQ}};

use axtask::RUN_QUEUE;


/// Futex requeue
/// 
/// nr_wake: number of waiters to wake (must be 1 for requeue_pi)
/// nr_requeue:	number of waiters to requeue (0-INT_MAX)
/// 若原队列中的任务数大于nr_wake，则将多余的任务移动到vaddr2对应的futex bucket, 移动的任务数目至多为move_num
/// Return:
///     >=0 - on success, the number of tasks requeued or woken;
///     <0 - on error
pub fn futex_requeue(
    vaddr1: VirtAddr,
    flags1: u32,
    vaddr2: VirtAddr,
    flags2: u32,
    nr_wake: u32,
    nr_requeue: u32,
    _cmpval: Option<u32>,
    _requeue_pi: u32,
) -> AxResult<usize> {
    let mut task_count = 0;
    let mut requeue_q = VecDeque::new();
    let key1 = get_futex_key(vaddr1, flags1);
    let key2 = get_futex_key(vaddr2, flags2);
    let hash1 = key1.futex_hash();
    let hash2 = key2.futex_hash();
    let mut hb1 = FUTEX_QUEUES.get_bucket(hash1).lock();
    hb1.retain(|fq| {
        if task_count - nr_wake >= nr_requeue {
            return true;
        }
        if !fq.match_key(&key1) {
            return true;
        }
        if task_count < nr_wake {
            RUN_QUEUE.lock().unblock_task(fq.task.clone(), false);
        } else {
            requeue_q.push_back(fq.task.clone());
        }
        task_count += 1;
        false
    });
    drop(hb1);
    let mut hb2 = FUTEX_QUEUES.get_bucket(hash2).lock();
    while !requeue_q.is_empty() {
        let task = requeue_q.pop_front().unwrap();
        let fq = FutexQ::new(key2.clone(), task);
        hb2.push_back(fq);
    }
    Ok(task_count as usize)
}