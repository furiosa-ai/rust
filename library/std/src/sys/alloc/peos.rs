//! System allocator for peOS
//!
//! This allocator uses the peos-abi syscall interface to allocate memory
//! from the peOS kernel.

use crate::alloc::{GlobalAlloc, Layout, System};
use crate::ptr;

// Import peos-abi functions
unsafe extern "C" {
    fn alloc(size: usize) -> *mut u8;
    fn free(ptr: *mut u8, size: usize);
}

unsafe impl GlobalAlloc for System {
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // Call peOS allocator via syscall table
        // Note: peos-abi only takes size, not alignment
        // The kernel allocator should provide sufficient alignment
        unsafe { alloc(layout.size()) }
    }

    #[inline]
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // Call peOS deallocator via syscall table
        unsafe { free(ptr, layout.size()) }
    }

    #[inline]
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        // peOS doesn't have a realloc syscall, so we implement it manually
        unsafe {
            let new_layout = Layout::from_size_align_unchecked(new_size, layout.align());
            let new_ptr = self.alloc(new_layout);
            if !new_ptr.is_null() && !ptr.is_null() {
                let copy_size = layout.size().min(new_size);
                ptr::copy_nonoverlapping(ptr, new_ptr, copy_size);
                self.dealloc(ptr, layout);
            }
            new_ptr
        }
    }

    #[inline]
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        // Allocate and zero the memory
        unsafe {
            let ptr = self.alloc(layout);
            if !ptr.is_null() {
                ptr::write_bytes(ptr, 0, layout.size());
            }
            ptr
        }
    }
}