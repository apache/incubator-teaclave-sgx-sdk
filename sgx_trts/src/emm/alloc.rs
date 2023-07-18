use core::alloc::{AllocError, Allocator, Layout};
use core::ptr::NonNull;

use crate::emm::interior::{RES_ALLOCATOR, STATIC};

/// alloc layout memory from Reserve region
#[derive(Clone, Copy)]
pub struct ResAlloc;

unsafe impl Allocator for ResAlloc {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        let size = layout.size();
        RES_ALLOCATOR
            .get()
            .unwrap()
            .lock()
            .emalloc(size)
            .map(|addr| NonNull::slice_from_raw_parts(NonNull::new(addr as *mut u8).unwrap(), size))
            .map_err(|_| AllocError)
    }

    #[inline]
    unsafe fn deallocate(&self, ptr: NonNull<u8>, _layout: Layout) {
        RES_ALLOCATOR.get().unwrap().lock().efree(ptr.addr().get())
    }
}

#[derive(Clone, Copy)]
pub struct StaticAlloc;

unsafe impl Allocator for StaticAlloc {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        STATIC
            .lock()
            .alloc(layout)
            .map(|addr| NonNull::slice_from_raw_parts(addr, layout.size()))
            .map_err(|_| AllocError)
    }

    #[inline]
    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        STATIC.lock().dealloc(ptr, layout);
    }
}
