//! Platform-specific extensions to `std` for peOS (PE Operating System).
//!
//! This module contains the platform abstraction layer (PAL) implementation
//! for peOS, providing the interface between Rust's std library and the
//! peOS kernel. peOS is a custom operating system designed for PE (Processing Element)
//! cores in NPU (Neural Processing Unit) hardware.

#![allow(missing_docs, nonstandard_style, dead_code, unused)]

#[path = "../unsupported/os.rs"]
pub mod os;
#[path = "../unsupported/pipe.rs"]
pub mod pipe;
#[path = "../unsupported/thread.rs"]
pub mod thread;
#[path = "../unsupported/time.rs"]
pub mod time;

pub type RawOsError = i32;

use crate::io as std_io;

#[inline]
pub const fn unsupported<T>() -> std_io::Result<T> {
    Err(unsupported_err())
}

#[inline]
pub const fn unsupported_err() -> std_io::Error {
    std_io::const_error!(std_io::ErrorKind::Unsupported, "operation not supported on peOS")
}

/// # SAFETY
/// Must be called only once during runtime initialization.
pub unsafe fn init(_argc: isize, _argv: *const *const u8, _sigpipe: u8) {
    // peOS initialization - minimal for now
}

/// # SAFETY  
/// Must be called only once during runtime cleanup.
pub unsafe fn cleanup() {
    // peOS cleanup - minimal for now
}

pub fn decode_error_kind(errno: RawOsError) -> crate::io::ErrorKind {
    use crate::io::ErrorKind;

    // Basic errno mapping for peOS
    match errno {
        1 => ErrorKind::PermissionDenied,  // EPERM
        2 => ErrorKind::NotFound,          // ENOENT
        4 => ErrorKind::Interrupted,       // EINTR
        11 => ErrorKind::WouldBlock,       // EAGAIN/EWOULDBLOCK
        13 => ErrorKind::PermissionDenied, // EACCES
        17 => ErrorKind::AlreadyExists,    // EEXIST
        22 => ErrorKind::InvalidInput,     // EINVAL
        32 => ErrorKind::BrokenPipe,       // EPIPE
        _ => ErrorKind::Uncategorized,
    }
}

pub fn abort_internal() -> ! {
    // For peOS, we use a simple abort mechanism
    core::intrinsics::abort();
}

// This function is needed by the panic runtime
#[cfg(not(test))]
#[unsafe(no_mangle)]
pub extern "C" fn __rust_abort() {
    abort_internal();
}

pub fn is_interrupted(errno: RawOsError) -> bool {
    errno == 4 // EINTR
}

#[doc(hidden)]
pub trait IsNegative {
    fn is_negative(&self) -> bool;
    fn negate(&self) -> i32;
}

impl IsNegative for i32 {
    fn is_negative(&self) -> bool {
        *self < 0
    }

    fn negate(&self) -> i32 {
        -(*self)
    }
}

pub fn cvt<T: IsNegative>(t: T) -> crate::io::Result<T> {
    if t.is_negative() {
        let e = decode_error_kind(t.negate());
        Err(crate::io::Error::from(e))
    } else {
        Ok(t)
    }
}

pub fn cvt_r<T, F>(mut f: F) -> crate::io::Result<T>
where
    T: IsNegative,
    F: FnMut() -> T,
{
    loop {
        match cvt(f()) {
            Err(ref e) if e.is_interrupted() => {}
            other => return other,
        }
    }
}
