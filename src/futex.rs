use core::ops::Deref;

use axhal::mem::VirtAddr;
use axprocess::current_process;
use axtask::AxTaskRef;


/// Futexes are matched on equal values of this key.
/// 
/// The key type depends on whether it's a shared or private mapping.
#[derive(Copy, Clone, Default, Ord, PartialOrd, Eq, PartialEq)]
pub struct FutexKey {
    /// mm_struct pointer for private mappings, or inode for shared mappings.
    pub ptr: u64,
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
    fn new(ptr: u64, word: usize, offset: u32) -> Self {
        Self { ptr, word, offset }
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
pub fn get_futex_key(uaddr: VirtAddr, _flags: i32) -> FutexKey {
    let mm = current_process().memory_set.lock().deref().lock().deref() as *const _ as u64;
    let offset = uaddr.align_offset_4k() as u32;
    let word = uaddr.align_down_4k().as_usize();
    FutexKey::new(mm, word, offset)
}


/// struct futex_q - The hashed futex queue entry, one per waiting task
pub struct FutexQ {
    /// the key the futex is hashed on
    pub key: FutexKey, 
    /// the task waiting on the futex
    pub task: (AxTaskRef, u32),
}