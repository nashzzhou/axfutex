use core::ops::Deref;

use alloc::sync::Arc;
use axhal::mem::VirtAddr;
use axprocess::current_process;
use axtask::AxTaskRef;

use crate::FUTEX_HASH_SIZE;

pub struct FutexQ {
    /// The key of the futex
    pub key: FutexKey,
    /// The task that is waiting on the futex
    pub task: AxTaskRef,
}


impl FutexQ {
    /// Create a new futex queue
    pub fn new(key: FutexKey, task: AxTaskRef) -> Arc<Self> {
        Arc::new(Self { key, task })
    }

    /// Check if the futex queue matches the key
    pub fn match_key(&self, key: &FutexKey) -> bool {
        self.key == *key
    }
}



/// Futexes are matched on equal values of this key.
/// 
/// The key type depends on whether it's a shared or private mapping.
#[derive(Clone, Eq, PartialEq)]
pub struct FutexKey {
    /// mm_struct pointer for private mappings, or inode for shared mappings.
    pub ptr: usize,
    /// address for private mappings, or page index for shared mappings.
    pub word: usize,
    /// offset is aligned to a multiple of sizeof(u32) (== 4) by definition.
    /// We use the two low order bits of offset to tell what is the kind of key :
    ///     00 : Private process futex (PTHREAD_PROCESS_PRIVATE) 
    ///             (no reference on an inode or mm)
    ///     01 : Shared futex (PTHREAD_PROCESS_SHARED) 
    ///             mapped on a file (reference on the underlying inode)
    ///     10 : Shared futex (PTHREAD_PROCESS_SHARED) 
    ///             (but private mapping on an mm, and reference taken on it)
    pub offset: u32,
}

impl FutexKey {
    fn new(ptr: usize, word: usize, offset: u32) -> Self {
        Self { ptr, word, offset }
    }

    /// Jhash function for futexes
    pub fn futex_hash(&self) -> usize {
        let hash = self.ptr + self.offset as usize + self.word;
        hash % FUTEX_HASH_SIZE
    }
}

/// get_futex_key() - Get parameters which are the keys for a futex
/// 
/// Return: a negative error code or 0
/// 
/// For private mappings (or when !@fshared), the key is:
///     ( current->mm, address, 0 )
/// 
/// TODO: For shared mappings (when @fshared), the key is:
///     ( inode->i_sequence, page->index, offset_within_page )
pub fn get_futex_key(uaddr: VirtAddr, flags: u32) -> FutexKey {
    let is_shared = flags & 0x0010 != 0;
    let ptr = current_process().memory_set.lock().lock().deref() as *const _ as usize;
    let mut offset = uaddr.align_offset_4k() as u32;
    if is_shared {
        offset &= 2;
    }
    let word = uaddr.align_down_4k().as_usize();
    FutexKey::new(ptr, word, offset)
}

#[derive(Default)]
/// 用于存储 robust list 的结构
pub struct FutexRobustList {
    /// The location of the head of the robust list in user space
    pub head: usize,
    /// The length of the robust list
    pub len: usize,
}

impl FutexRobustList {
    /// Create a new robust list
    pub fn new(head: usize, len: usize) -> Self {
        Self { head, len }
    }
}