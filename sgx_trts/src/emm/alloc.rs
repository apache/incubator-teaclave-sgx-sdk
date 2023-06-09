use core::alloc::{AllocError, Allocator, Layout};
use core::ptr::NonNull;

/// alloc layout memory from Reserve region
#[derive(Clone)]
pub struct ResAlloc;

unsafe impl Allocator for ResAlloc {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        todo!()
    }

    #[inline]
    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        todo!()
    }
}

#[derive(Clone)]
pub struct StaticAlloc;

unsafe impl Allocator for StaticAlloc {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        todo!()
    }

    #[inline]
    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        todo!()
    }
}
