//! C++ runtime shims for `wasm32-unknown-unknown` (#20).
//!
//! The ymfm objects are compiled by zig's clang against wasi libc++ headers
//! but linked with no C++ standard library (see build.rs). rust-lld's
//! `--allow-undefined` would otherwise turn the runtime symbols the code
//! pulls in into wasm imports the browser cannot satisfy, so this module
//! provides them over Rust's allocator instead. The exact needed set was
//! enumerated with wasm-tools on a linked module; new/delete are implemented
//! for both sized and unsized flavours by prefixing each allocation with a
//! 16-byte header that stores the full layout size.

use std::alloc::{Layout, alloc, dealloc};
use std::ffi::c_char;

/// Header keeps the payload aligned to 16 — wasm32's `max_align_t`
/// (long double) alignment, which C++ `operator new` must honour.
const HEADER_SIZE: usize = 16;
const CXX_ALIGN: usize = 16;

// The header cast is sound: cxx_alloc allocates with CXX_ALIGN (16), which
// exceeds usize's alignment.
#[allow(clippy::cast_ptr_alignment)]
fn cxx_alloc(payload_size: usize) -> *mut u8 {
    let total = payload_size
        .max(1)
        .checked_add(HEADER_SIZE)
        .unwrap_or_else(|| std::process::abort());
    let Ok(layout) = Layout::from_size_align(total, CXX_ALIGN) else {
        std::process::abort();
    };
    // SAFETY: layout has non-zero size (HEADER_SIZE is added above).
    let base = unsafe { alloc(layout) };
    if base.is_null() {
        // ymfm never handles allocation failure; C++ new would throw and the
        // objects are compiled -fno-exceptions, so abort is the contract.
        std::process::abort();
    }
    // SAFETY: base is a fresh allocation of at least HEADER_SIZE + 1 bytes.
    unsafe {
        (base.cast::<usize>()).write(total);
        base.add(HEADER_SIZE)
    }
}

/// SAFETY contract: `payload` must be a pointer previously returned by
/// [`cxx_alloc`] (C++ guarantees delete only sees pointers from new).
// The header cast is sound: the base pointer came from a CXX_ALIGN (16)
// allocation, which exceeds usize's alignment.
#[allow(clippy::cast_ptr_alignment)]
unsafe fn cxx_dealloc(payload: *mut u8) {
    if payload.is_null() {
        // C++ delete of nullptr is a no-op.
        return;
    }
    // SAFETY: per contract, payload sits HEADER_SIZE past an allocation whose
    // first word is the total layout size written by cxx_alloc.
    unsafe {
        let base = payload.sub(HEADER_SIZE);
        let total = base.cast::<usize>().read();
        let layout = Layout::from_size_align_unchecked(total, CXX_ALIGN);
        dealloc(base, layout);
    }
}

/// C++ `operator new(size_t)`.
#[unsafe(no_mangle)]
extern "C" fn _Znwm(size: usize) -> *mut u8 {
    cxx_alloc(size)
}

/// C++ `operator new[](size_t)`.
#[unsafe(no_mangle)]
extern "C" fn _Znam(size: usize) -> *mut u8 {
    cxx_alloc(size)
}

/// C++ `operator delete(void*)`.
#[unsafe(no_mangle)]
extern "C" fn _ZdlPv(ptr: *mut u8) {
    // SAFETY: C++ only passes pointers obtained from operator new above.
    unsafe { cxx_dealloc(ptr) }
}

/// C++ `operator delete[](void*)`.
#[unsafe(no_mangle)]
extern "C" fn _ZdaPv(ptr: *mut u8) {
    // SAFETY: C++ only passes pointers obtained from operator new[] above.
    unsafe { cxx_dealloc(ptr) }
}

/// C++ sized `operator delete(void*, size_t)`. The header already records the
/// size, so the hint is ignored.
#[unsafe(no_mangle)]
extern "C" fn _ZdlPvm(ptr: *mut u8, _size: usize) {
    // SAFETY: C++ only passes pointers obtained from operator new above.
    unsafe { cxx_dealloc(ptr) }
}

/// C++ sized `operator delete[](void*, size_t)`.
#[unsafe(no_mangle)]
extern "C" fn _ZdaPvm(ptr: *mut u8, _size: usize) {
    // SAFETY: C++ only passes pointers obtained from operator new[] above.
    unsafe { cxx_dealloc(ptr) }
}

/// Called through a devirtualised pure-virtual slot — a C++ UB trap door.
#[unsafe(no_mangle)]
extern "C" fn __cxa_pure_virtual() -> ! {
    std::process::abort()
}

/// Stack-smashing canary failure (kept even though build.rs passes
/// -fno-stack-protector, in case a future flag change reintroduces it).
#[unsafe(no_mangle)]
extern "C" fn __stack_chk_fail() -> ! {
    std::process::abort()
}

/// libc `assert` failure hook referenced by wasi libc++ headers.
#[unsafe(no_mangle)]
extern "C" fn __assert_fail(
    _assertion: *const c_char,
    _file: *const c_char,
    _line: u32,
    _function: *const c_char,
) -> ! {
    std::process::abort()
}

/// `std::__1::__libcpp_verbose_abort(char const*, ...)` — libc++'s hardening
/// failure hook.
#[unsafe(no_mangle)]
extern "C" fn _ZNSt3__122__libcpp_verbose_abortEPKcz(_format: *const c_char) -> ! {
    std::process::abort()
}
