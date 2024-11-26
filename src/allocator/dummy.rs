use super::Locked;
use alloc::alloc::{GlobalAlloc, Layout};

pub struct Dummy;

impl Dummy {
    pub unsafe fn init(&mut self, _heap_start: usize, _heap_size: usize) {}
}

unsafe impl GlobalAlloc for Locked<Dummy> {
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        core::ptr::null_mut()
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        panic!("dealloc should be never called")
    }
}
