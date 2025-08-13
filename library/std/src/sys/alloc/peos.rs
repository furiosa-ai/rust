//! System allocator for peOS

use crate::alloc::{GlobalAlloc, Layout, System};
use crate::ptr;

unsafe impl GlobalAlloc for System {
    #[inline]
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        // For peOS, we'll use a simple bump allocator or return null for now
        // In a real implementation, this would call peOS memory management
        ptr::null_mut()
    }

    #[inline]
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // For peOS, this would call peOS memory deallocation
        // For now, this is a no-op
    }

    #[inline]
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        // For peOS, implement a simple realloc
        // In a real implementation, this would call peOS memory management
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
        // For peOS, allocate and zero the memory
        unsafe {
            let ptr = self.alloc(layout);
            if !ptr.is_null() {
                ptr::write_bytes(ptr, 0, layout.size());
            }
            ptr
        }
    }
}